mod agent;
mod cli;
mod config;
mod error;
mod installer;
mod progress;
mod registry;
mod scanner;

use std::path::PathBuf;

use clap::Parser;
use colored::Colorize;

use cli::{Cli, Commands};
use config::Config;
use error::SkillHubError;
use installer::Installer;
use registry::{GITHUB_API, RegistryClient, SkillRef};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.effective_command() {
        Commands::Setup => cmd_setup(cli.json),
        Commands::Help => {
            if !Config::load()?.is_setup() {
                if cli.json {
                    println!("{}", json_err("not configured"));
                    return Ok(());
                }
                println!("skillhub isn't configured yet.\n");
                return cmd_setup(cli.json);
            }
            if cli.json {
                println!(
                    "{}",
                    serde_json::json!({"help": "skillhub — universal skill registry", "commands": ["search", "install", "list", "update", "setup", "agents", "help"]})
                );
                return Ok(());
            }
            println!("{}", "skillhub — universal skill registry".cyan().bold());
            println!("Usage: skillhub <command> [options]");
            println!("       skillhub search  <query>");
            println!("       skillhub install <name>");
            println!("       skillhub list");
            println!("       skillhub update");
            println!("       skillhub setup");
            println!("       skillhub agents");
            let n = Installer::new(Config::load()?).list_installed()?.len();
            if n > 0 {
                println!("\nInstalled skills: {}", n);
            }
            Ok(())
        }
        Commands::Agents => cmd_agents(cli.json),
        cmd => {
            let config = Config::load()?;
            let client = RegistryClient::new(config.clone());
            let mut installer = Installer::new(config);
            installer.no_progress = cli.json;

            match cmd {
                Commands::Setup | Commands::Help | Commands::Agents => unreachable!(),
                Commands::Search {
                    query,
                    verbose,
                    github,
                } => cmd_search(&client, query, *verbose, *github, cli.json),
                Commands::Install {
                    name,
                    yes,
                    no_scan,
                    project,
                } => cmd_install(
                    &client, &installer, name, *yes, *no_scan, *project, cli.json,
                ),
                Commands::Uninstall { name, yes } => {
                    cmd_uninstall(&installer, name, *yes, cli.json)
                }
                Commands::List => cmd_list(&installer, cli.json),
                Commands::Update => cmd_update(&client, cli.json),
                Commands::Upgrade { dry_run, all } => {
                    cmd_upgrade(&client, &installer, *dry_run, *all, cli.json)
                }
                Commands::Info { name } => cmd_info(&client, name, cli.json),
                Commands::Doctor { fix } => cmd_doctor(&client, &installer, *fix, cli.json),
                Commands::Completions { shell } => cmd_completions(shell, cli.json),
                Commands::Share => cmd_share(&installer, cli.json),
                Commands::Restore { source } => cmd_restore(&client, &installer, source, cli.json),
                Commands::Suggest => cmd_suggest(&client, cli.json),
                Commands::Publish { path, force } => {
                    cmd_publish(&client, path.as_deref(), *force, cli.json)
                }
                Commands::Import { source, dry_run } => {
                    cmd_import(&installer, source, *dry_run, cli.json)
                }
                Commands::Stats => cmd_stats(&client, &installer, cli.json),
                Commands::Badge { format: badge_fmt } => cmd_badge(&installer, badge_fmt, cli.json),
                Commands::Sync { action, target } => {
                    cmd_sync(&client, &installer, action, target.as_deref(), cli.json)
                }
                Commands::Migrate { rollback } => cmd_migrate(&installer, *rollback, cli.json),
            }
        }
    }
}

// ---------------------------------------------------------------------------
// share / restore
// ---------------------------------------------------------------------------

fn cmd_share(installer: &Installer, json: bool) -> anyhow::Result<()> {
    let items = installer.list_installed()?;

    if items.is_empty() {
        if json {
            println!(
                "{}",
                serde_json::json!({"error": "nothing installed", "skills": []})
            );
        } else {
            println!("{}", "Nothing installed to share.".yellow());
        }
        return Ok(());
    }

    let manifest: Vec<serde_json::Value> = items
        .iter()
        .map(|(_, s)| {
            serde_json::json!({
                "name": s.name,
                "version": s.version,
                "author": s.author,
            })
        })
        .collect();

    let manifest_str = serde_json::to_string_pretty(&serde_json::json!({
        "skillhub_version": 1,
        "skills": manifest
    }))?;

    if json {
        println!(
            "{}",
            serde_json::json!({"manifest": manifest, "count": manifest.len()})
        );
        return Ok(());
    }

    println!("{}", "Your skill manifest:".cyan().bold());
    println!("{}", manifest_str.dimmed());
    println!();
    println!(
        "{}",
        "Save this to a file or gist, then restore with:".dimmed()
    );
    println!("  skillhub restore <url-or-path>");
    Ok(())
}

fn cmd_restore(
    client: &RegistryClient,
    installer: &Installer,
    source: &str,
    json: bool,
) -> anyhow::Result<()> {
    // Fetch manifest (from URL or local file)
    let raw = if source.starts_with("http://") || source.starts_with("https://") {
        client.gh_get(source)?
    } else {
        std::fs::read_to_string(source).map_err(SkillHubError::Io)?
    };

    // Parse manifest
    #[derive(serde::Deserialize)]
    struct Manifest {
        #[allow(dead_code)]
        skillhub_version: u32,
        skills: Vec<ManifestSkill>,
    }

    #[derive(serde::Deserialize)]
    struct ManifestSkill {
        name: String,
        #[allow(dead_code)]
        #[serde(default)]
        version: String,
    }

    let manifest: Manifest =
        serde_json::from_str(&raw).map_err(|e| SkillHubError::ConfigParse(e.to_string()))?;

    let desired: Vec<String> = manifest.skills.iter().map(|s| s.name.clone()).collect();

    if desired.is_empty() {
        if json {
            println!(
                "{}",
                serde_json::json!({"error": "empty manifest", "installed": 0})
            );
        } else {
            println!("{}", "Manifest is empty. Nothing to restore.".yellow());
        }
        return Ok(());
    }

    // Fetch registry if needed
    let registry = match client.load_cache() {
        Ok(r) => r,
        Err(_) => {
            if !json {
                println!("{}", "Fetching registry...".dimmed());
            }
            let r = client.fetch()?;
            client.save_cache(&r)?;
            r
        }
    };

    let mut installed = 0u32;
    let mut skipped = 0u32;

    for skill_name in &desired {
        let ref_ = SkillRef::parse(skill_name)?;

        if installer.is_installed(&ref_)? {
            skipped += 1;
            continue;
        }

        let Some(skill) = registry.skills.iter().find(|s| ref_.matches(s)) else {
            if !json {
                println!("  {} not found in registry", skill_name.yellow());
            }
            continue;
        };

        installer.install(skill)?;
        installed += 1;

        if !json {
            println!("  {} installed", skill_name.green());
        }
    }

    if json {
        println!(
            "{}",
            serde_json::json!({"installed": installed, "skipped": skipped, "total": desired.len()})
        );
    } else {
        println!(
            "{}",
            format!(
                "Installed {}, skipped {} (already had).",
                installed, skipped
            )
            .green()
            .bold()
        );
    }
    Ok(())
}

fn json_err(msg: &str) -> String {
    serde_json::json!({"error": msg}).to_string()
}

// ---------------------------------------------------------------------------
// setup
// ---------------------------------------------------------------------------

fn cmd_setup(json: bool) -> anyhow::Result<()> {
    if json {
        println!("{}", json_err("setup is interactive, run without --json"));
        return Ok(());
    }
    let mut config = Config::load()?;

    if config.is_setup() {
        let ok = dialoguer::Confirm::new()
            .with_prompt("Already configured. Re-run setup?")
            .default(false)
            .interact()?;
        if !ok {
            println!("{}", "canceled".yellow());
            return Ok(());
        }
    }

    // GitHub token
    println!("{}", "GitHub API token".green().bold());
    println!("skillhub fetches its skill registry from GitHub.");
    println!("Without a token:    60 requests/hour (may hit limits fast).");
    println!("With a token:     5 000 requests/hour.");
    println!();
    println!("Create one at https://github.com/settings/tokens (no scopes needed).");
    println!();

    let token: String = dialoguer::Input::new()
        .with_prompt("Paste your token (or leave empty for anonymous)")
        .allow_empty(true)
        .interact_text()?;

    let t = token.trim();
    if !t.is_empty() {
        config.github_token = Some(t.to_string());
        println!("{}", "token saved".green());
    } else {
        println!("{}", "no token — 60 req/h limit".yellow());
    }

    // Registry URL
    let url: String = dialoguer::Input::new()
        .with_prompt("Registry URL (Enter for default)")
        .default(config.registry_url.clone())
        .interact_text()?;
    if !url.trim().is_empty() {
        config.registry_url = url.trim().to_string();
    }

    // Agent detection
    println!();
    let detected = agent::AgentConfig::detect();
    if detected.any_found() {
        println!("{}", "AI agents detected on this machine".green().bold());
        for a in &detected.agents {
            let path = a.detect_home().unwrap();
            println!("  {}  {:?}", a.label().cyan(), path);
        }
        println!();

        let deploy = dialoguer::Confirm::new()
            .with_prompt("Deploy skills to these agents automatically?")
            .default(true)
            .interact()?;

        if deploy {
            config.deploy_agents = detected.agents.clone();
            println!("{}", "auto-deploy enabled".green());
        } else {
            println!(
                "{}",
                "skills will be stored only in ~/.skillhub/skills/".dimmed()
            );
        }
    } else {
        println!("{}", "No AI agents detected on this machine.".dimmed());
        println!("Skills will be stored in ~/.skillhub/skills/.");
        println!("Run 'skillhub agents' later to re-scan.");
    }

    config.save()?;

    // Fetch registry
    println!();
    let fetch = dialoguer::Confirm::new()
        .with_prompt("Fetch the skill registry now?")
        .default(true)
        .interact()?;

    if fetch {
        let client = RegistryClient::new(config);
        match client.fetch() {
            Ok(r) => {
                let n = r.skills.len();
                client.save_cache(&r)?;
                println!("{}", format!("{} skills available", n).green().bold());
            }
            Err(e) => {
                println!("{}", format!("fetch failed: {}", e).yellow());
                println!("You can retry later with 'skillhub update'.");
            }
        }
    }

    println!();
    println!("{}", "Ready.".green().bold());
    println!("  skillhub search <query>   — find skills");
    println!("  skillhub install <name>   — install one");
    println!("  skillhub list             — what you've got");
    println!("  skillhub agents           — detect AI tools");
    Ok(())
}

// ---------------------------------------------------------------------------
// doctor
// ---------------------------------------------------------------------------

fn cmd_doctor(
    client: &RegistryClient,
    _installer: &Installer,
    fix: bool,
    json: bool,
) -> anyhow::Result<()> {
    let config = Config::load()?;
    let mut issues: Vec<String> = Vec::new();
    let mut fixed: Vec<String> = Vec::new();

    // Check 1: Config exists and parses
    let config_path = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".skillhub")
        .join("config.toml");

    if !config_path.exists() {
        issues.push("config.toml not found".to_string());
        if fix {
            config.save()?;
            fixed.push("created config.toml".to_string());
        }
    }

    // Check 2: GitHub token
    if config.github_token().is_some() {
        // just check it exists, don't validate against API (rate limit concerns)
        if json {
            // fine
        }
    } else {
        issues.push("no GitHub token set (60 req/h limit)".to_string());
        if fix {
            // can't auto-fix a missing token — user must run skillhub setup
        }
    }

    // Check 3: Cache exists and is fresh
    let cache_path = config.cache_dir.join("registry.json");
    let cache_age = cache_path
        .metadata()
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.elapsed().ok());

    if !cache_path.exists() {
        issues.push("registry cache missing, run 'skillhub update'".to_string());
    } else if let Some(age) = cache_age {
        if age.as_secs() > 86400 {
            issues.push(format!(
                "registry cache is {} old",
                humantime(age.as_secs())
            ));
            if fix {
                match client.fetch() {
                    Ok(r) => {
                        client.save_cache(&r)?;
                        fixed.push("refreshed registry cache".to_string());
                    }
                    Err(e) => {
                        if !json {
                            eprintln!("  {} while refreshing cache: {}", "warn".yellow(), e);
                        }
                    }
                }
            }
        }
    }

    // Check 4: Skills directory
    if !config.skills_dir.exists() {
        issues.push(format!("skills dir {:?} not found", config.skills_dir));
        if fix {
            std::fs::create_dir_all(&config.skills_dir)?;
            fixed.push(format!("created {:?}", config.skills_dir));
        }
    }

    // Check 5: Cache directory
    if !config.cache_dir.exists() {
        issues.push(format!("cache dir {:?} not found", config.cache_dir));
        if fix {
            std::fs::create_dir_all(&config.cache_dir)?;
            fixed.push(format!("created {:?}", config.cache_dir));
        }
    }

    // Check 6: Agent directories
    for agent in &config.deploy_agents {
        if let Some(home) = agent.detect_home() {
            if !home.exists() {
                issues.push(format!("{:?} exists (agent {:?})", home, agent.label()));
                if fix {
                    std::fs::create_dir_all(&home).ok();
                    fixed.push(format!("created {:?}", home));
                }
            }
            if let Some(skills) = agent.skills_dir() {
                if !skills.exists() {
                    // just a warning, not a critical issue
                }
            }
        }
    }

    if json {
        println!(
            "{}",
            serde_json::json!({
                "healthy": issues.is_empty(),
                "issues": issues,
                "fixed": fixed,
                "fix_applied": fix,
            })
        );
        return Ok(());
    }

    if issues.is_empty() {
        println!("{}", "All checks passed.".green().bold());
    } else {
        println!(
            "{}",
            format!("{} issue(s) found:", issues.len()).yellow().bold()
        );
        for i in &issues {
            println!("  {}", i.yellow());
        }
    }

    if !fixed.is_empty() {
        println!();
        println!("{}", "Fixed:".green().bold());
        for f in &fixed {
            println!("  {}", f.green());
        }
    }

    Ok(())
}

fn humantime(secs: u64) -> String {
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m", secs / 60)
    } else if secs < 86400 {
        format!("{}h", secs / 3600)
    } else {
        format!("{}d", secs / 86400)
    }
}

// ---------------------------------------------------------------------------
// agents
// ---------------------------------------------------------------------------

fn cmd_agents(json: bool) -> anyhow::Result<()> {
    let config = Config::load()?;

    if json {
        let agents: Vec<serde_json::Value> = agent::AgentKind::all()
            .iter()
            .map(|a| {
                let path = a.detect_home().map(|p| p.display().to_string());
                let installed = a.detect();
                let deploying = config.deploy_agents.contains(a);
                serde_json::json!({
                    "name": a.label(),
                    "kind_id": a.kind_id(),
                    "path": path,
                    "detected": installed,
                    "deploying": deploying
                })
            })
            .collect();
        println!("{}", serde_json::json!({"agents": agents}));
        return Ok(());
    }

    println!("{}", "AI agents".cyan().bold());
    for a in agent::AgentKind::all() {
        let path = a.detect_home();
        let installed = a.detect();
        let deploying = config.deploy_agents.contains(a);

        let status = if deploying {
            "deploying".green()
        } else if installed {
            "detected".yellow()
        } else {
            "not found".dimmed()
        };

        let p = path
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "-".to_string());
        println!("  {:<15} {}  {:?}", a.label(), status, p);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// search
// ---------------------------------------------------------------------------

fn cmd_search(
    client: &RegistryClient,
    query: &str,
    verbose: bool,
    github: bool,
    json: bool,
) -> anyhow::Result<()> {
    let mut results = Vec::new();
    let registry_found = match client.search(query) {
        Ok(r) => {
            results = r;
            true
        }
        Err(SkillHubError::RegistryNotFound) => false,
        Err(e) => return Err(e.into()),
    };

    let registry_count = results.len();
    let mut gh_count = 0;

    if github {
        match client.search_github(query) {
            Ok(gh_results) => {
                gh_count = gh_results.len();
                results.extend(gh_results);
            }
            Err(e) => {
                if json {
                    // silently ignore, results still valid
                } else {
                    println!("{}", format!("GitHub search failed: {}", e).yellow());
                }
            }
        }
    }

    if !registry_found && !github {
        if json {
            println!("{}", json_err("no registry cache"));
        } else {
            println!(
                "{}",
                "No registry cache. Run 'skillhub update' first.".yellow()
            );
        }
        return Ok(());
    }

    if results.is_empty() {
        if json {
            println!(
                "{}",
                serde_json::json!({"skills": [], "count": 0, "query": query})
            );
        } else {
            println!("{}", "Nothing found.".yellow());
            println!("Try a different query, or 'skillhub update' to refresh.");
        }
        return Ok(());
    }

    if json {
        println!(
            "{}",
            serde_json::json!({
                "skills": results,
                "count": results.len(),
                "query": query,
                "sources": {"registry": registry_count, "github": gh_count}
            })
        );
        return Ok(());
    }

    if github && gh_count > 0 {
        let src = format!("registry:{}  github:{}", registry_count, gh_count);
        println!(
            "{} ({})",
            format!("{} result(s):", results.len()).green().bold(),
            src.dimmed()
        );
    } else {
        println!("{}", format!("{} result(s):", results.len()).green().bold());
    }
    println!();

    for s in &results {
        let name = format!("{:<40}", s.name).cyan().bold();
        let v = if verbose {
            format!(" v{}", s.version)
        } else {
            String::new()
        };
        println!("  {}{}", name, v);
        println!("    {}\n", s.description.dimmed());

        if verbose {
            let agents = &s.compatibility.agents;
            if !agents.is_empty() {
                println!("    agents: {}", agents.join(", ").dimmed());
            }
            println!("    skillhub install {}", s.name.dimmed());
            println!();
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// install
// ---------------------------------------------------------------------------

fn cmd_install(
    client: &RegistryClient,
    installer: &Installer,
    name: &str,
    yes: bool,
    no_scan: bool,
    project: bool,
    json: bool,
) -> anyhow::Result<()> {
    if json {
        println!(
            "{}",
            json_err("install is interactive, use --yes to skip confirmation with --json")
        );
        return Ok(());
    }
    let ref_ = SkillRef::parse(name)?;

    if installer.is_installed(&ref_)? {
        println!("{}", format!("'{}' is already installed.", name).yellow());
        return Ok(());
    }

    let skill = match client.load_cache() {
        Ok(r) => r
            .skills
            .into_iter()
            .find(|s| ref_.matches(s))
            .ok_or(SkillHubError::SkillNotFound(name.to_string()))?,
        Err(_) => {
            println!("{}", "No cache, fetching registry...".dimmed());
            let r = client.fetch()?;
            r.skills
                .into_iter()
                .find(|s| ref_.matches(s))
                .ok_or(SkillHubError::SkillNotFound(name.to_string()))?
        }
    };

    // Security scan
    if !no_scan {
        let content = match installer.download_skill(&skill.download_url) {
            Ok(c) => c,
            Err(e) => {
                if !json {
                    println!(
                        "{}",
                        format!("security scan: could not download — {}", e).yellow()
                    );
                }
                String::new()
            }
        };
        if content.is_empty() {
            if !json {
                println!("{}", "security scan: skipped".dimmed());
            }
        } else {
            let scan_result = scanner::scan(&content);
            if scan_result.passed {
                if !json {
                    println!("{}", "Security scan: passed".green());
                }
            } else {
                println!("{}", "Security warnings:".yellow().bold());
                for f in &scan_result.findings {
                    let sev = match f.severity {
                        scanner::Severity::High => "HIGH".red(),
                        scanner::Severity::Medium => "MED".yellow(),
                        scanner::Severity::Low => "low".dimmed(),
                    };
                    println!(
                        "  [{}] {} (line {}): {}",
                        sev,
                        f.kind,
                        f.line,
                        f.snippet.dimmed()
                    );
                }
                if !yes {
                    let ok = dialoguer::Confirm::new()
                        .with_prompt("Install anyway?")
                        .default(false)
                        .interact()?;
                    if !ok {
                        println!("{}", "canceled".yellow());
                        return Ok(());
                    }
                }
            }
        }
    }

    println!("{}", format!("Installing {}", skill.name).cyan().bold());
    println!("  author:   {}", skill.author);
    println!("  version:  {}", skill.version);
    println!("  {}", skill.description);

    if !yes {
        let ok = dialoguer::Confirm::new()
            .with_prompt("Proceed?")
            .default(true)
            .interact()?;
        if !ok {
            println!("{}", "canceled".yellow());
            return Ok(());
        }
    }

    if project {
        let cwd = std::env::current_dir()?;
        let mut project_file = None;

        // Detect which agent project is active
        for a in agent::AgentKind::all() {
            if let Some(pf) = a.project_file() {
                let path = cwd.join(pf);
                if path.exists() {
                    project_file = Some((a.label().to_string(), path));
                    break;
                }
            }
        }

        let (agent_label, pfile) = match project_file {
            Some((l, p)) => (l, p),
            None => {
                // Default to CLAUDE.md if nothing detected
                let p = cwd.join("CLAUDE.md");
                ("Claude Code".to_string(), p)
            }
        };

        // Download content and write to project file
        let content = installer
            .download_skill(&skill.download_url)
            .unwrap_or_else(|_| format!("# {}\n\n{}", skill.name, skill.description));

        // Append a reference to the skill
        let skill_ref = format!("\n## {} (v{})\n\n{}", skill.name, skill.version, content);
        let existing = if pfile.exists() {
            std::fs::read_to_string(&pfile)?
        } else {
            String::new()
        };
        std::fs::write(&pfile, existing + &skill_ref)?;

        if json {
            println!(
                "{}",
                serde_json::json!({"installed_to_project": true, "agent": agent_label, "file": pfile.display().to_string()})
            );
        } else {
            println!(
                "{}",
                format!("Installed to project file: {:?} ({})", pfile, agent_label)
                    .green()
                    .bold()
            );
        }
    } else {
        let paths = installer.install(&skill)?;
        for p in &paths {
            println!("  {}", p.display().to_string().dimmed());
        }
    }

    if !json {
        println!("{}", "Done.".green().bold());
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// uninstall
// ---------------------------------------------------------------------------

fn cmd_uninstall(installer: &Installer, name: &str, yes: bool, json: bool) -> anyhow::Result<()> {
    let ref_ = SkillRef::parse(name)?;

    if !installer.is_installed(&ref_)? {
        if json {
            println!("{}", json_err("not installed"));
        } else {
            println!("{}", format!("'{}' is not installed.", name).yellow());
        }
        return Ok(());
    }

    if !yes && !json {
        let ok = dialoguer::Confirm::new()
            .with_prompt(format!("Remove '{}'?", name))
            .default(false)
            .interact()?;
        if !ok {
            println!("{}", "canceled".yellow());
            return Ok(());
        }
    }

    let path = installer.uninstall(&ref_)?;
    if json {
        println!(
            "{}",
            serde_json::json!({"removed": true, "name": name, "path": path.display().to_string()})
        );
    } else {
        println!("{}", format!("removed {:?}", path).dimmed());
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// list
// ---------------------------------------------------------------------------

fn cmd_list(installer: &Installer, json: bool) -> anyhow::Result<()> {
    let items = installer.list_installed()?;

    if json {
        let skills: Vec<serde_json::Value> = items
            .iter()
            .map(|(path, skill)| {
                serde_json::json!({
                    "name": skill.name,
                    "description": skill.description,
                    "author": skill.author,
                    "version": skill.version,
                    "tags": skill.tags,
                    "path": path.display().to_string()
                })
            })
            .collect();
        println!(
            "{}",
            serde_json::json!({"skills": skills, "count": skills.len()})
        );
        return Ok(());
    }

    if items.is_empty() {
        println!("{}", "Nothing installed.".yellow());
        println!("Try 'skillhub search <query>' or 'skillhub install <name>'.");
        return Ok(());
    }

    println!("{}", format!("{} skill(s):", items.len()).green().bold());
    println!();

    for (path, skill) in &items {
        let name = format!("{:<40}", skill.name).cyan().bold();
        let v = if skill.version.is_empty() {
            String::new()
        } else {
            format!(" v{}", skill.version)
        };
        println!("  {}{}", name, v);
        if !skill.description.is_empty() {
            println!("    {}", skill.description.dimmed());
        }
        println!("    {}", path.display().to_string().dimmed());
        println!();
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// update
// ---------------------------------------------------------------------------

fn cmd_update(client: &RegistryClient, json: bool) -> anyhow::Result<()> {
    if !json {
        println!("{}", "Fetching registry...".dimmed());
    }
    let r = client.fetch()?;
    let n = r.skills.len();
    client.save_cache(&r)?;
    if json {
        println!(
            "{}",
            serde_json::json!({"skills_available": n, "status": "ok"})
        );
    } else {
        println!("{}", format!("{} skills available.", n).green().bold());
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// upgrade
// ---------------------------------------------------------------------------

/// Simple version comparison (assumes semver-like "x.y.z").
fn version_gt(a: &str, b: &str) -> bool {
    let va: Vec<u64> = a.split('.').filter_map(|s| s.parse::<u64>().ok()).collect();
    let vb: Vec<u64> = b.split('.').filter_map(|s| s.parse::<u64>().ok()).collect();
    for i in 0..va.len().max(vb.len()) {
        let na = va.get(i).copied().unwrap_or(0);
        let nb = vb.get(i).copied().unwrap_or(0);
        if na != nb {
            return na > nb;
        }
    }
    false
}

fn cmd_upgrade(
    client: &RegistryClient,
    installer: &Installer,
    dry_run: bool,
    all: bool,
    json: bool,
) -> anyhow::Result<()> {
    let installed = installer.list_installed()?;
    if installed.is_empty() {
        if json {
            println!(
                "{}",
                serde_json::json!({"status": "nothing to upgrade", "upgraded": 0})
            );
        } else {
            println!("{}", "Nothing installed. Nothing to upgrade.".yellow());
        }
        return Ok(());
    }

    let registry = match client.load_cache() {
        Ok(r) => r,
        Err(_) => {
            if json {
                println!("{}", json_err("no registry cache"));
            } else {
                println!(
                    "{}",
                    "No registry cache. Run 'skillhub update' first.".yellow()
                );
            }
            return Ok(());
        }
    };

    let mut to_upgrade: Vec<(String, String, String)> = Vec::new(); // (name, old_ver, new_ver)

    for (_, skill) in &installed {
        if let Some(reg) = registry.skills.iter().find(|s| s.name == skill.name) {
            if version_gt(&reg.version, &skill.version) {
                to_upgrade.push((
                    skill.name.clone(),
                    skill.version.clone(),
                    reg.version.clone(),
                ));
            }
        }
    }

    if to_upgrade.is_empty() {
        if json {
            println!(
                "{}",
                serde_json::json!({"status": "up to date", "upgraded": 0})
            );
        } else {
            println!("{}", "All skills are up to date.".green());
        }
        return Ok(());
    }

    if dry_run {
        if json {
            println!(
                "{}",
                serde_json::json!({"to_upgrade": to_upgrade.iter().map(|(n, o, nv)| {
                serde_json::json!({"name": n, "old_version": o, "new_version": nv})
            }).collect::<Vec<_>>(), "count": to_upgrade.len()})
            );
        } else {
            println!(
                "{}",
                format!("{} skill(s) would upgrade:", to_upgrade.len())
                    .yellow()
                    .bold()
            );
            for (name, old_v, new_v) in &to_upgrade {
                println!(
                    "  {}  v{} → v{}",
                    name.cyan(),
                    old_v.dimmed(),
                    new_v.green()
                );
            }
            println!();
            println!("{}", "Run without --dry-run to upgrade.".dimmed());
        }
        return Ok(());
    }

    let mut upgraded = 0u32;

    for (name, old_v, new_v) in &to_upgrade {
        if !all {
            let ok = dialoguer::Confirm::new()
                .with_prompt(format!("Upgrade {} v{} → v{}?", name, old_v, new_v))
                .default(true)
                .interact()?;
            if !ok {
                continue;
            }
        }

        let ref_ = SkillRef::parse(name)?;
        let reg_skill = registry.skills.iter().find(|s| ref_.matches(s)).unwrap();

        // remove old, install new
        let _ = installer.uninstall(&ref_);
        installer.install(reg_skill)?;
        upgraded += 1;

        if !json {
            println!("  {} v{} → {}", name.cyan(), old_v.dimmed(), new_v.green());
        }
    }

    if json {
        println!(
            "{}",
            serde_json::json!({"status": "done", "upgraded": upgraded})
        );
    } else {
        println!(
            "{}",
            format!("Upgraded {} skill(s).", upgraded).green().bold()
        );
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// info
// ---------------------------------------------------------------------------

fn cmd_info(client: &RegistryClient, name: &str, json: bool) -> anyhow::Result<()> {
    let ref_ = SkillRef::parse(name)?;
    let skill = client.find_skill(&ref_)?;

    if json {
        println!("{}", serde_json::to_string_pretty(&skill)?);
        return Ok(());
    }

    println!("{}", skill.name.cyan().bold());
    println!();
    println!("  {}", skill.description);

    println!("  author:   {}", skill.author);
    println!("  version:  {}", skill.version);

    if !skill.tags.is_empty() {
        println!("  tags:     {}", skill.tags.join(", "));
    }

    println!("  agents:   {}", skill.compatibility.agents.join(", "));

    if let Some(q) = &skill.quality {
        println!(
            "  quality:  {}/100  (security {}  clarity {})",
            q.score, q.security, q.clarity
        );
    }

    println!();
    println!("  install:  skillhub install {}", skill.name.dimmed());
    Ok(())
}

// ---------------------------------------------------------------------------
// publish
// ---------------------------------------------------------------------------

fn cmd_publish(
    client: &RegistryClient,
    path: Option<&str>,
    force: bool,
    json: bool,
) -> anyhow::Result<()> {
    let skill_path = path.unwrap_or("./SKILL.md");
    let content = std::fs::read_to_string(skill_path)
        .map_err(|e| anyhow::anyhow!("Cannot read {}: {}", skill_path, e))?;

    if content.trim().is_empty() {
        anyhow::bail!("SKILL.md is empty");
    }

    // Basic validation
    if !force {
        let has_title = content.lines().any(|l| l.starts_with("# "));
        let has_desc = content.len() > 50;
        if !has_title {
            anyhow::bail!("SKILL.md must have a title (line starting with '# ')");
        }
        if !has_desc {
            anyhow::bail!("SKILL.md content is too short (minimum 50 chars)");
        }
        if !(content.contains("name:") || content.contains("Name:")) {
            println!(
                "{}",
                "warning: SKILL.md should contain a `name:` field".yellow()
            );
        }
    }

    // Derive skill name from the filename's directory or content
    let skill_name = std::path::Path::new(skill_path)
        .parent()
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // Create a GitHub issue in the registry repo
    // URL like raw.githubusercontent.com/skillhub/registry/main/skills.json
    // repo = skillhub/registry (path segments [0]/[1])
    let registry_url_parsed = url::Url::parse(client.registry_url())
        .map_err(|e| anyhow::anyhow!("invalid registry URL: {}", e))?;
    let segments: Vec<String> = registry_url_parsed
        .path_segments()
        .map(|s| s.filter(|p| !p.is_empty()).map(String::from).collect())
        .unwrap_or_default();
    let repo = if segments.len() >= 2 {
        format!("{}/{}", segments[0], segments[1])
    } else {
        anyhow::bail!(
            "cannot determine registry repo from URL: {}",
            client.registry_url()
        );
    };

    let title = format!("skill submission: {}", skill_name);
    let body = format!(
        "## Skill Submission\n\n**Name:** {}\n**Source file:** `{}`\n\n```markdown\n{}\n```",
        skill_name, skill_path, content
    );

    let api_url = format!("{}/repos/{}/issues", GITHUB_API, repo);
    let payload = serde_json::json!({
        "title": title,
        "body": body,
        "labels": ["skill-submission"]
    });

    let body_str = serde_json::to_string(&payload)?;
    let mut req = ureq::post(&api_url)
        .set("User-Agent", "skillhub/0.1.0")
        .set("Content-Type", "application/json");

    if let Some(t) = client.github_token() {
        req = req.set("Authorization", &format!("Bearer {}", t));
    } else {
        anyhow::bail!("GitHub token required to publish. Run 'skillhub setup' first.");
    }

    let resp = req
        .send_string(&body_str)
        .map_err(|e| anyhow::anyhow!("Failed to create issue: {}", e))?;

    let status = resp.status();
    let resp_body = resp.into_string().unwrap_or_default();

    if status != 201 {
        anyhow::bail!("GitHub API returned {}: {}", status, resp_body);
    }

    let issue_url: String = serde_json::from_str::<serde_json::Value>(&resp_body)
        .ok()
        .and_then(|v| v.get("html_url").and_then(|u| u.as_str()).map(String::from))
        .unwrap_or_else(|| api_url.clone());

    if json {
        println!(
            "{}",
            serde_json::json!({"status": "submitted", "url": issue_url, "name": skill_name})
        );
    } else {
        println!("{}", "Skill submitted!".green().bold());
        println!("  Review at: {}", issue_url);
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// suggest
// ---------------------------------------------------------------------------

fn today_str() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let days = secs / 86400;
    let mut y = 1970i64;
    let mut remaining = days as i64;
    loop {
        let days_in_year = if (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0) {
            366
        } else {
            365
        };
        if remaining < days_in_year {
            break;
        }
        remaining -= days_in_year;
        y += 1;
    }
    let is_leap = (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0);
    let month_days: [i64; 12] = [
        31,
        if is_leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut m = 1;
    for &md in &month_days {
        if remaining < md {
            break;
        }
        remaining -= md;
        m += 1;
    }
    format!("{:04}-{:02}-{:02}", y, m, remaining + 1)
}

fn cmd_suggest(client: &RegistryClient, json: bool) -> anyhow::Result<()> {
    let mut config = Config::load()?;
    let today = today_str();

    if config.last_suggest_date.as_deref() == Some(&today) {
        if json {
            println!(
                "{}",
                serde_json::json!({"error": "already suggested today"})
            );
        } else {
            println!(
                "{}",
                "Already suggested today. Come back tomorrow.".yellow()
            );
        }
        return Ok(());
    }

    // Get the most popular skill from the registry
    let registry = match client.load_cache() {
        Ok(r) => r,
        Err(_) => {
            if json {
                println!("{}", json_err("no registry cache"));
            } else {
                println!(
                    "{}",
                    "No registry cache. Run 'skillhub update' first.".yellow()
                );
            }
            return Ok(());
        }
    };

    if registry.skills.is_empty() {
        if json {
            println!("{}", serde_json::json!({"error": "registry is empty"}));
        } else {
            println!("{}", "Registry is empty. Nothing to suggest.".yellow());
        }
        return Ok(());
    }

    // Simple pick: highest quality score, or first if no quality scores
    let pick = registry
        .skills
        .iter()
        .max_by(|a, b| {
            let qa = a.quality.as_ref().map(|q| q.score).unwrap_or(0);
            let qb = b.quality.as_ref().map(|q| q.score).unwrap_or(0);
            qa.cmp(&qb)
        })
        .unwrap();

    config.last_suggest_date = Some(today);
    config.save()?;

    if json {
        println!(
            "{}",
            serde_json::json!({
                "suggestion": {
                    "name": pick.name,
                    "description": pick.description,
                    "author": pick.author,
                    "version": pick.version,
                    "agents": pick.compatibility.agents,
                }
            })
        );
        return Ok(());
    }

    println!("{}", "Suggested skill for today:".cyan().bold());
    println!();
    println!("  {}", pick.name.green().bold());
    println!("  {}", pick.description);
    println!("  author: {}", pick.author);
    println!("  version: {}", pick.version);
    if !pick.compatibility.agents.is_empty() {
        println!("  agents: {}", pick.compatibility.agents.join(", "));
    }
    println!();
    println!("  skillhub install {}", pick.name.dimmed());
    Ok(())
}

// ---------------------------------------------------------------------------
// completions
// ---------------------------------------------------------------------------

fn cmd_completions(shell: &str, json: bool) -> anyhow::Result<()> {
    use clap::CommandFactory;
    use clap_complete::{generate, shells::*};
    use std::io;

    let mut cmd = Cli::command();
    let name = "skillhub";

    if json {
        let shells = ["bash", "zsh", "fish", "powershell"];
        println!(
            "{}",
            serde_json::json!({
                "supported_shells": shells,
                "usage": format!("source <(skillhub completions {})", shell)
            })
        );
        return Ok(());
    }

    match shell {
        "bash" => generate(Bash, &mut cmd, name, &mut io::stdout()),
        "zsh" => generate(Zsh, &mut cmd, name, &mut io::stdout()),
        "fish" => generate(Fish, &mut cmd, name, &mut io::stdout()),
        "powershell" => generate(PowerShell, &mut cmd, name, &mut io::stdout()),
        other => {
            eprintln!(
                "Unknown shell '{}'. Supported: bash, zsh, fish, powershell",
                other
            );
            std::process::exit(1);
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// import
// ---------------------------------------------------------------------------

fn cmd_import(
    installer: &Installer,
    source: &str,
    dry_run: bool,
    json: bool,
) -> anyhow::Result<()> {
    let dir = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));

    let source_dir = match source {
        "claude" => dir.join(".claude").join("skills"),
        "cursor" => dir.join(".cursor").join("rules"),
        "windsurf" => dir.join(".windsurf").join("rules"),
        "awesome" => dir.join("awesome-claude-skills"),
        other => std::path::PathBuf::from(other),
    };

    if !source_dir.exists() {
        if json {
            println!(
                "{}",
                serde_json::json!({"error": format!("source not found: {}", source_dir.display()), "imported": 0})
            );
        } else {
            println!("{}", format!("Source not found: {:?}", source_dir).yellow());
        }
        return Ok(());
    }

    let mut imported = 0u32;

    if let Ok(entries) = std::fs::read_dir(&source_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                continue;
            }
            let file_name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");

            let content = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            if content.trim().is_empty() {
                continue;
            }

            let owner = "imported";
            let skill_name = file_name.to_lowercase().replace(' ', "-");

            if dry_run {
                if json {
                    // collected below
                } else {
                    println!(
                        "  would import: @{}/{} ({})",
                        owner,
                        skill_name,
                        path.display()
                    );
                }
            } else {
                let skill_dir = installer.config.skills_dir.join(owner).join(&skill_name);
                if skill_dir.exists() {
                    continue;
                }
                std::fs::create_dir_all(&skill_dir).ok();
                let _ = std::fs::write(skill_dir.join("SKILL.md"), &content);

                let meta = serde_json::json!({
                    "name": format!("@{}/{}", owner, skill_name),
                    "description": content.lines().next().unwrap_or("imported skill").trim_start_matches("# "),
                    "author": owner,
                    "version": "0.1.0",
                    "tags": [source],
                    "download_url": "",
                    "compatibility": {"agents": [source]},
                });
                let _ = std::fs::write(
                    skill_dir.join("meta.json"),
                    serde_json::to_string_pretty(&meta).unwrap_or_default(),
                );
                imported += 1;
            }
        }
    }

    if json {
        let skills: Vec<serde_json::Value> = if dry_run {
            let mut v = Vec::new();
            if let Ok(entries) = std::fs::read_dir(&source_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        continue;
                    }
                    let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("x");
                    v.push(serde_json::json!({"name": format!("@imported/{}", name), "path": path.display().to_string()}));
                }
            }
            v
        } else {
            vec![]
        };
        println!(
            "{}",
            serde_json::json!({"imported": imported, "dry_run": dry_run, "skills": skills, "source": source})
        );
    } else if dry_run {
        println!(
            "{}",
            "Dry run complete. Run without --dry-run to import.".dimmed()
        );
    } else {
        println!(
            "{}",
            format!("Imported {} skill(s) from {}.", imported, source)
                .green()
                .bold()
        );
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// stats
// ---------------------------------------------------------------------------

fn cmd_stats(client: &RegistryClient, installer: &Installer, json: bool) -> anyhow::Result<()> {
    let installed = installer.list_installed()?;
    let installed_count = installed.len();

    let mut authors: std::collections::HashSet<&str> = std::collections::HashSet::new();
    let mut agent_tags: std::collections::HashSet<&str> = std::collections::HashSet::new();
    let mut agents_found: Vec<String> = Vec::new();

    for a in agent::AgentKind::all() {
        if a.detect() {
            agents_found.push(a.label().to_string());
        }
    }

    for (_, skill) in &installed {
        authors.insert(&skill.author);
        for a in &skill.compatibility.agents {
            agent_tags.insert(a);
        }
    }

    let registry_skills = client
        .load_cache()
        .ok()
        .map(|r| r.skills.len())
        .unwrap_or(0);
    let deploy_count = installer.config.deploy_agents.len();

    if json {
        println!(
            "{}",
            serde_json::json!({
                "installed": installed_count,
                "authors": authors.len(),
                "agent_compatibility": agent_tags.len(),
                "agents_detected": agents_found,
                "agents_configured_for_deploy": deploy_count,
                "registry_skills_available": registry_skills,
            })
        );
        return Ok(());
    }

    println!("{}", "Statistics".cyan().bold());
    println!(
        "  installed skills:      {}",
        installed_count.to_string().green()
    );
    println!(
        "  unique authors:        {}",
        authors.len().to_string().cyan()
    );
    println!(
        "  agent compatibility:   {}",
        agent_tags.len().to_string().yellow()
    );
    println!(
        "  agents detected:       {}",
        agents_found.len().to_string().green()
    );
    println!(
        "  agents configured:     {}",
        deploy_count.to_string().cyan()
    );
    println!(
        "  registry skills:       {}",
        registry_skills.to_string().green()
    );

    if !agents_found.is_empty() {
        println!(
            "  detected:              {}",
            agents_found.join(", ").dimmed()
        );
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// badge
// ---------------------------------------------------------------------------

fn badge_color(n: usize) -> &'static str {
    if n == 0 {
        "lightgrey"
    } else if n < 5 {
        "yellow"
    } else if n < 15 {
        "green"
    } else {
        "brightgreen"
    }
}

fn cmd_badge(installer: &Installer, fmt: &str, json: bool) -> anyhow::Result<()> {
    let installed = installer.list_installed()?.len();
    let url = format!(
        "https://img.shields.io/badge/skills-{}-{}",
        installed,
        badge_color(installed)
    );

    if json {
        println!(
            "{}",
            serde_json::json!({"badge_url": url, "installed": installed, "format": fmt})
        );
        return Ok(());
    }

    match fmt {
        "markdown" => println!("![skillhub]({})", url),
        "url" => println!("{}", url),
        "html" => println!("<img src=\"{}\" alt=\"skills: {}\" />", url, installed),
        other => {
            println!(
                "{}",
                format!("unsupported format '{}', use: markdown, url, html", other).yellow()
            );
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// sync
// ---------------------------------------------------------------------------

fn cmd_sync(
    client: &RegistryClient,
    installer: &Installer,
    action: &str,
    target: Option<&str>,
    json: bool,
) -> anyhow::Result<()> {
    match action {
        "export" => {
            let items = installer.list_installed()?;
            let manifest = serde_json::json!({
                "skillhub_version": 1,
                "exported_at": std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
                "skills": items.iter().map(|(_, s)| {
                    serde_json::json!({
                        "name": s.name,
                        "version": s.version,
                        "author": s.author,
                        "description": s.description,
                    })
                }).collect::<Vec<_>>(),
            });

            let output = serde_json::to_string_pretty(&manifest)?;

            if let Some(path) = target {
                std::fs::write(path, &output)?;
                if json {
                    println!(
                        "{}",
                        serde_json::json!({"exported": items.len(), "path": path})
                    );
                } else {
                    println!(
                        "{}",
                        format!("Exported {} skill(s) to {}", items.len(), path)
                            .green()
                            .bold()
                    );
                }
            } else {
                if json {
                    println!("{}", manifest);
                } else {
                    println!("{}", output);
                }
            }
        }
        "import" => {
            let raw = match target {
                Some(src) if src.starts_with("http") => client.gh_get(src)?,
                Some(src) => std::fs::read_to_string(src).map_err(SkillHubError::Io)?,
                None => {
                    if json {
                        println!("{}", json_err("import requires a file path or URL"));
                    } else {
                        println!(
                            "{}",
                            "Import requires a file path or URL: skillhub sync import <path|url>"
                                .yellow()
                        );
                    }
                    return Ok(());
                }
            };

            #[derive(serde::Deserialize)]
            struct SyncManifest {
                skills: Vec<SyncSkill>,
            }
            #[derive(serde::Deserialize)]
            struct SyncSkill {
                name: String,
                #[allow(dead_code)]
                #[serde(default)]
                version: String,
            }

            let manifest: SyncManifest = serde_json::from_str(&raw)
                .map_err(|e| SkillHubError::ConfigParse(e.to_string()))?;

            let registry = match client.load_cache() {
                Ok(r) => r,
                Err(_) => {
                    let r = client.fetch()?;
                    client.save_cache(&r)?;
                    r
                }
            };

            let mut imported = 0u32;
            let mut skipped = 0u32;

            for s in &manifest.skills {
                let ref_ = SkillRef::parse(&s.name)?;
                if installer.is_installed(&ref_)? {
                    skipped += 1;
                    continue;
                }
                if let Some(skill) = registry.skills.iter().find(|rs| ref_.matches(rs)) {
                    installer.install(skill)?;
                    imported += 1;
                }
            }

            if json {
                println!(
                    "{}",
                    serde_json::json!({"imported": imported, "skipped": skipped})
                );
            } else {
                println!(
                    "{}",
                    format!("Imported {}, skipped {}.", imported, skipped)
                        .green()
                        .bold()
                );
            }
        }
        other => {
            if json {
                println!("{}", json_err(&format!("unknown sync action: {}", other)));
            } else {
                println!(
                    "{}",
                    format!("Unknown sync action '{}'. Use: export, import", other).yellow()
                );
            }
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// migrate
// ---------------------------------------------------------------------------

fn cmd_migrate(installer: &Installer, rollback: bool, json: bool) -> anyhow::Result<()> {
    let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    let old_skills = home.join(".skillhub").join("skills");
    let _old_config = home.join(".skillhub").join("config.toml");

    let new_skills = installer.config.skills_dir.clone();
    let new_cache = installer.config.cache_dir.clone();

    if rollback {
        // Rollback: move skills back to flat dir (unlikely to be needed)
        if json {
            println!(
                "{}",
                serde_json::json!({"status": "rollback not needed", "current_version": 2})
            );
        } else {
            println!("{}", "Current layout is v2. Nothing to roll back.".yellow());
            println!("Skills are in {}", new_skills.to_string_lossy().dimmed());
        }
        return Ok(());
    }

    let mut migrated = 0u32;
    let mut skipped = 0u32;

    // Migration v0 → v1: flat skills → nested @owner/name
    if old_skills.exists() && new_skills != old_skills {
        if let Ok(entries) = std::fs::read_dir(&old_skills) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    continue;
                }
                let file_name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                let content = match std::fs::read_to_string(&path) {
                    Ok(c) => c,
                    Err(_) => continue,
                };

                let owner = "migrated";
                let skill_dir = new_skills.join(owner).join(&file_name);
                if skill_dir.exists() {
                    skipped += 1;
                    continue;
                }

                std::fs::create_dir_all(&skill_dir).ok();
                let _ = std::fs::write(skill_dir.join("SKILL.md"), &content);

                let meta = serde_json::json!({
                    "name": format!("@{}/{}", owner, file_name),
                    "description": content.lines().next().unwrap_or("migrated skill").trim_start_matches("# "),
                    "author": owner,
                    "version": "0.1.0",
                    "tags": ["migrated"],
                    "download_url": "",
                    "compatibility": {"agents": []},
                });
                let _ = std::fs::write(
                    skill_dir.join("meta.json"),
                    serde_json::to_string_pretty(&meta).unwrap_or_default(),
                );
                migrated += 1;
            }
        }
    }

    // Migration v1 → v2: ensure cache dir exists
    std::fs::create_dir_all(&new_cache).ok();

    if json {
        println!(
            "{}",
            serde_json::json!({"migrated": migrated, "skipped": skipped, "rollback": rollback})
        );
    } else if migrated > 0 {
        println!(
            "{}",
            format!("Migrated {} skill(s), skipped {}.", migrated, skipped)
                .green()
                .bold()
        );
    } else {
        println!("{}", "No migration needed.".green());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_gt_newer_minor() {
        assert!(version_gt("2.0.0", "1.0.0"));
    }

    #[test]
    fn version_gt_newer_patch() {
        assert!(version_gt("1.2.0", "1.1.9"));
    }

    #[test]
    fn version_gt_equal() {
        assert!(!version_gt("1.0.0", "1.0.0"));
    }

    #[test]
    fn version_gt_older() {
        assert!(!version_gt("0.9.0", "1.0.0"));
    }

    #[test]
    fn version_gt_different_length() {
        assert!(version_gt("1.0", "0.9.9"));
        assert!(!version_gt("0.9", "1.0.0"));
    }

    #[test]
    fn version_gt_non_numeric_segment() {
        assert!(!version_gt("1.0.0-alpha", "1.0.0"));
    }

    #[test]
    fn humantime_seconds() {
        assert_eq!(humantime(30), "30s");
    }

    #[test]
    fn humantime_minutes() {
        assert_eq!(humantime(120), "2m");
    }

    #[test]
    fn humantime_hours() {
        assert_eq!(humantime(7200), "2h");
    }

    #[test]
    fn humantime_days() {
        assert_eq!(humantime(172800), "2d");
    }

    #[test]
    fn humantime_boundary_minutes() {
        assert_eq!(humantime(59), "59s");
        assert_eq!(humantime(60), "1m");
    }

    #[test]
    fn humantime_boundary_hours() {
        assert_eq!(humantime(3599), "59m");
        assert_eq!(humantime(3600), "1h");
    }

    #[test]
    fn humantime_boundary_days() {
        assert_eq!(humantime(86399), "23h");
        assert_eq!(humantime(86400), "1d");
    }

    #[test]
    fn badge_color_none() {
        assert_eq!(badge_color(0), "lightgrey");
    }

    #[test]
    fn badge_color_few() {
        assert_eq!(badge_color(1), "yellow");
        assert_eq!(badge_color(4), "yellow");
    }

    #[test]
    fn badge_color_some() {
        assert_eq!(badge_color(5), "green");
        assert_eq!(badge_color(14), "green");
    }

    #[test]
    fn badge_color_many() {
        assert_eq!(badge_color(15), "brightgreen");
        assert_eq!(badge_color(100), "brightgreen");
    }

    #[test]
    fn today_str_format() {
        let s = today_str();
        assert_eq!(s.len(), 10);
        assert_eq!(&s[4..5], "-");
        assert_eq!(&s[7..8], "-");
        assert!(s.chars().all(|c| c == '-' || c.is_ascii_digit()));
    }
}
