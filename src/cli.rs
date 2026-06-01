use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "skillhub",
    about = "Universal skill registry for AI agents",
    version,
    propagate_version = true
)]
pub struct Cli {
    /// Output JSON instead of colored terminal output
    #[arg(global = true, long)]
    pub json: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

impl Cli {
    pub fn effective_command(&self) -> &Commands {
        self.command.as_ref().unwrap_or(&Commands::Help)
    }
}

#[derive(Subcommand)]
pub enum Commands {
    /// First-time setup (token, agents)
    Setup,

    /// Search available skills
    Search {
        query: String,
        #[arg(short, long)]
        verbose: bool,
        /// Also search GitHub repos with skillhub-skill topic
        #[arg(short = 'g', long)]
        github: bool,
    },

    /// Install a skill
    Install {
        /// skillhub name (@owner/name or short name)
        name: String,
        #[arg(short, long)]
        yes: bool,
        /// skip security scan
        #[arg(long)]
        no_scan: bool,
        /// install in current project directory instead of global
        #[arg(long)]
        project: bool,
    },

    /// Remove an installed skill
    Uninstall {
        name: String,
        #[arg(short, long)]
        yes: bool,
    },

    /// List installed skills
    List,

    /// Refresh the registry from GitHub
    Update,

    /// Upgrade installed skills to latest versions
    Upgrade {
        /// show what would upgrade without changing anything
        #[arg(long)]
        dry_run: bool,
        /// upgrade all without confirmation
        #[arg(short, long)]
        all: bool,
    },

    /// Show details about a skill
    Info { name: String },

    /// Detect AI agents on this machine
    Agents,

    /// Diagnose configuration and environment
    Doctor {
        /// attempt to fix common issues
        #[arg(long)]
        fix: bool,
    },

    /// Generate shell completions
    Completions {
        /// shell type: bash, zsh, fish, powershell
        shell: String,
    },

    /// Share your installed skills as a restore command
    Share,

    /// Restore skills from a share URL or file
    Restore {
        /// gist URL or file path to the skills manifest
        source: String,
    },

    /// Show one skill suggestion per day
    Suggest,

    /// Publish a skill to the registry
    Publish {
        /// Path to SKILL.md (defaults to ./SKILL.md)
        path: Option<String>,
        /// Skip validation
        #[arg(long)]
        force: bool,
    },

    /// Import skills from other formats (.claude, .cursor, .windsurf, awesome-claude-skills)
    Import {
        /// source: claude, cursor, windsurf, awesome
        source: String,
        /// show what would import without copying
        #[arg(long)]
        dry_run: bool,
    },

    /// Show installation statistics
    Stats,

    /// Generate a shield.io badge for your skill set
    Badge {
        /// output format: markdown, url, html
        #[arg(long, default_value = "markdown")]
        format: String,
    },

    /// Sync skills (export to file / import from file or gist)
    Sync {
        /// export | import
        action: String,
        /// file path or gist URL
        target: Option<String>,
    },

    /// Migrate configuration and skills from older versions
    Migrate {
        /// rollback to previous version layout
        #[arg(long)]
        rollback: bool,
    },

    /// Show this message (default)
    Help,
}
