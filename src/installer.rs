use std::fs;
use std::path::PathBuf;

use crate::config::Config;
use crate::error::{Result, SkillHubError};
use crate::progress;
use crate::registry::{Skill, SkillRef};

pub struct Installer {
    pub config: Config,
    /// When true, suppress progress bars and spinner output.
    pub no_progress: bool,
}

impl Installer {
    pub fn new(config: Config) -> Self {
        Installer {
            config,
            no_progress: false,
        }
    }

    pub fn install(&self, skill: &Skill, content: &str) -> Result<Vec<PathBuf>> {
        let mut installed = Vec::new();

        let skill_dir = self.skill_dir(&skill.name)?;
        let body = if content.is_empty() {
            self.download_skill(&skill.download_url)?
        } else {
            content.to_string()
        };

        if !skill_dir.exists() {
            fs::create_dir_all(&skill_dir)?;
            fs::write(skill_dir.join("SKILL.md"), &body)?;

            let meta = serde_json::to_string_pretty(skill)?;
            fs::write(skill_dir.join("meta.json"), meta)?;
            installed.push(skill_dir.clone());
        }

        for agent in &self.config.deploy_agents {
            let Some(target) = agent.skills_dir() else {
                continue;
            };

            let agent_skill_dir = target.join(skill.name.replace('@', "").replace('/', "_"));
            if agent_skill_dir.join("SKILL.md").exists() {
                continue;
            }

            fs::create_dir_all(&agent_skill_dir)?;
            fs::write(agent_skill_dir.join("SKILL.md"), &body)?;
            installed.push(agent_skill_dir);
        }

        if installed.is_empty() {
            return Err(SkillHubError::AlreadyInstalled(skill.name.clone()));
        }

        Ok(installed)
    }

    pub fn uninstall(&self, reference: &SkillRef) -> Result<PathBuf> {
        let paths = self.list_installed()?;
        let found = paths.into_iter().find(|(_, s)| reference.matches(s));

        match found {
            Some((dir, _)) => {
                let display = dir.display().to_string();
                fs::remove_dir_all(&dir)?;
                Ok(PathBuf::from(display))
            }
            None => Err(SkillHubError::NotInstalled(reference.display())),
        }
    }

    pub fn is_installed(&self, reference: &SkillRef) -> Result<bool> {
        for dir in self.iter_installed_dirs()? {
            let raw = match fs::read_to_string(dir.join("meta.json")) {
                Ok(r) => r,
                Err(_) => continue,
            };
            if let Ok(s) = serde_json::from_str::<Skill>(&raw) {
                if reference.matches(&s) {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    fn iter_installed_dirs(&self) -> Result<Vec<PathBuf>> {
        let dir = &self.config.skills_dir;
        if !dir.exists() {
            return Ok(vec![]);
        }
        let mut out = Vec::new();
        for owner in fs::read_dir(dir)? {
            let owner = owner?;
            if !owner.file_type()?.is_dir() {
                continue;
            }
            for skill_dir in fs::read_dir(owner.path())? {
                let skill_dir = skill_dir?;
                if skill_dir.file_type()?.is_dir() {
                    out.push(skill_dir.path());
                }
            }
        }
        Ok(out)
    }

    pub fn list_installed(&self) -> Result<Vec<(PathBuf, Skill)>> {
        let dir = &self.config.skills_dir;
        if !dir.exists() {
            return Ok(vec![]);
        }

        let mut out = Vec::new();

        for owner in fs::read_dir(dir)? {
            let owner = owner?;
            if !owner.file_type()?.is_dir() {
                continue;
            }
            let owner_name = owner.file_name();

            for skill_dir in fs::read_dir(owner.path())? {
                let skill_dir = skill_dir?;
                if !skill_dir.file_type()?.is_dir() {
                    continue;
                }
                let skill_name = skill_dir.file_name();
                let meta_path = skill_dir.path().join("meta.json");

                if meta_path.exists() {
                    if let Ok(raw) = fs::read_to_string(&meta_path) {
                        if let Ok(s) = serde_json::from_str::<Skill>(&raw) {
                            out.push((skill_dir.path(), s));
                            continue;
                        }
                    }
                }

                out.push((
                    skill_dir.path(),
                    Skill {
                        name: format!(
                            "@{}/{}",
                            owner_name.to_string_lossy(),
                            skill_name.to_string_lossy()
                        ),
                        description: String::new(),
                        author: owner_name.to_string_lossy().to_string(),
                        version: String::new(),
                        tags: vec![],
                        download_url: String::new(),
                        checksum: None,
                        compatibility: crate::registry::Compatibility { agents: vec![] },
                        quality: None,
                    },
                ));
            }
        }

        Ok(out)
    }

    pub fn skill_path(&self, reference: &SkillRef) -> Result<PathBuf> {
        for (dir, skill) in self.list_installed()? {
            if reference.matches(&skill) {
                return Ok(dir);
            }
        }
        Err(SkillHubError::NotInstalled(reference.display()))
    }

    fn skill_dir(&self, name: &str) -> Result<PathBuf> {
        let name = name.strip_prefix('@').unwrap_or(name);
        let (owner, skill_name) = name
            .split_once('/')
            .ok_or_else(|| SkillHubError::InvalidSkillName(name.to_string()))?;

        if owner.is_empty() || skill_name.is_empty() {
            return Err(SkillHubError::InvalidSkillName(name.to_string()));
        }

        Ok(self.config.skills_dir.join(owner).join(skill_name))
    }

    pub fn download_skill(&self, url: &str) -> Result<String> {
        progress::download_with_progress(url, &self.config, self.no_progress)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::Compatibility;
    use tempfile::TempDir;

    fn test_skill(name: &str) -> Skill {
        Skill {
            name: name.to_string(),
            description: "x".to_string(),
            author: "t".to_string(),
            version: "1".to_string(),
            tags: vec![],
            download_url: "https://example.com/s.md".to_string(),
            checksum: None,
            compatibility: Compatibility {
                agents: vec!["claude-code".to_string()],
            },
            quality: None,
        }
    }

    #[test]
    fn empty_dir() {
        let tmp = TempDir::new().unwrap();
        let cfg = Config {
            registry_url: String::new(),
            skills_dir: tmp.path().join("skills"),
            cache_dir: tmp.path().join("cache"),
            github_token: None,
            deploy_agents: vec![],
            last_suggest_date: None,
        };
        let inst = Installer::new(cfg);
        assert!(inst.list_installed().unwrap().is_empty());
    }

    #[test]
    fn reads_existing() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("skills").join("alice").join("test-s");
        fs::create_dir_all(&dir).unwrap();
        let skill = test_skill("@alice/test-s");
        fs::write(
            dir.join("meta.json"),
            serde_json::to_string_pretty(&skill).unwrap(),
        )
        .unwrap();

        let cfg = Config {
            registry_url: String::new(),
            skills_dir: tmp.path().join("skills"),
            cache_dir: tmp.path().join("cache"),
            github_token: None,
            deploy_agents: vec![],
            last_suggest_date: None,
        };
        let inst = Installer::new(cfg);
        let list = inst.list_installed().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].1.name, "@alice/test-s");
    }

    #[test]
    fn path_no_owner() {
        let tmp = TempDir::new().unwrap();
        let cfg = Config {
            registry_url: String::new(),
            skills_dir: tmp.path().join("skills"),
            cache_dir: tmp.path().join("cache"),
            github_token: None,
            deploy_agents: vec![],
            last_suggest_date: None,
        };
        let inst = Installer::new(cfg);
        assert!(inst.skill_dir("no-slash").is_err());
    }

    #[test]
    fn is_installed_false_for_empty_dir() {
        let tmp = TempDir::new().unwrap();
        let cfg = Config {
            registry_url: String::new(),
            skills_dir: tmp.path().join("skills"),
            cache_dir: tmp.path().join("cache"),
            github_token: None,
            deploy_agents: vec![],
            last_suggest_date: None,
        };
        let inst = Installer::new(cfg);
        let r = SkillRef::Short("nope".to_string());
        assert!(!inst.is_installed(&r).unwrap());
    }

    #[test]
    fn is_installed_true_after_meta_written() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("skills").join("alice").join("test-s");
        fs::create_dir_all(&dir).unwrap();
        let skill = test_skill("@alice/test-s");
        fs::write(
            dir.join("meta.json"),
            serde_json::to_string_pretty(&skill).unwrap(),
        )
        .unwrap();

        let cfg = Config {
            registry_url: String::new(),
            skills_dir: tmp.path().join("skills"),
            cache_dir: tmp.path().join("cache"),
            github_token: None,
            deploy_agents: vec![],
            last_suggest_date: None,
        };
        let inst = Installer::new(cfg);
        let r = SkillRef::Short("test-s".to_string());
        assert!(inst.is_installed(&r).unwrap());
    }

    #[test]
    fn skill_path_resolves_known_skill() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("skills").join("alice").join("test-s");
        fs::create_dir_all(&dir).unwrap();
        let skill = test_skill("@alice/test-s");
        fs::write(
            dir.join("meta.json"),
            serde_json::to_string_pretty(&skill).unwrap(),
        )
        .unwrap();

        let cfg = Config {
            registry_url: String::new(),
            skills_dir: tmp.path().join("skills"),
            cache_dir: tmp.path().join("cache"),
            github_token: None,
            deploy_agents: vec![],
            last_suggest_date: None,
        };
        let inst = Installer::new(cfg);
        let r = SkillRef::Short("test-s".to_string());
        let p = inst.skill_path(&r).unwrap();
        assert_eq!(p, dir);
    }

    #[test]
    fn iter_installed_dirs_empty() {
        let tmp = TempDir::new().unwrap();
        let cfg = Config {
            registry_url: String::new(),
            skills_dir: tmp.path().join("skills"),
            cache_dir: tmp.path().join("cache"),
            github_token: None,
            deploy_agents: vec![],
            last_suggest_date: None,
        };
        let inst = Installer::new(cfg);
        assert!(inst.iter_installed_dirs().unwrap().is_empty());
    }

    #[test]
    fn install_writes_canonical_skill_md() {
        let tmp = TempDir::new().unwrap();
        let cfg = Config {
            registry_url: String::new(),
            skills_dir: tmp.path().join("skills"),
            cache_dir: tmp.path().join("cache"),
            github_token: None,
            deploy_agents: vec![],
            last_suggest_date: None,
        };
        let inst = Installer::new(cfg);
        let skill = test_skill("@alice/code-review");
        let paths = inst.install(&skill, "# code review\n\nbody").unwrap();
        assert_eq!(paths.len(), 1);

        let canonical = tmp.path().join("skills").join("alice").join("code-review");
        assert!(canonical.join("SKILL.md").exists());
        assert!(canonical.join("meta.json").exists());

        let body = fs::read_to_string(canonical.join("SKILL.md")).unwrap();
        assert_eq!(body, "# code review\n\nbody");

        let meta_raw = fs::read_to_string(canonical.join("meta.json")).unwrap();
        let meta: Skill = serde_json::from_str(&meta_raw).unwrap();
        assert_eq!(meta.name, "@alice/code-review");
    }

    #[test]
    fn install_twice_does_not_overwrite_canonical() {
        let tmp = TempDir::new().unwrap();
        let cfg = Config {
            registry_url: String::new(),
            skills_dir: tmp.path().join("skills"),
            cache_dir: tmp.path().join("cache"),
            github_token: None,
            deploy_agents: vec![],
            last_suggest_date: None,
        };
        let inst = Installer::new(cfg);
        let skill = test_skill("@alice/x");
        inst.install(&skill, "first").unwrap();
        let second = inst.install(&skill, "second");
        assert!(matches!(second, Err(SkillHubError::AlreadyInstalled(_))));

        let body = fs::read_to_string(
            tmp.path()
                .join("skills")
                .join("alice")
                .join("x")
                .join("SKILL.md"),
        )
        .unwrap();
        assert_eq!(body, "first");
    }

    #[test]
    fn install_empty_content_falls_back_to_download() {
        let tmp = TempDir::new().unwrap();
        let cfg = Config {
            registry_url: String::new(),
            skills_dir: tmp.path().join("skills"),
            cache_dir: tmp.path().join("cache"),
            github_token: None,
            deploy_agents: vec![],
            last_suggest_date: None,
        };
        let inst = Installer::new(cfg);
        let skill = test_skill("@alice/bad");
        let r = inst.install(&skill, "");
        assert!(r.is_err());
    }

    #[test]
    fn uninstall_missing_returns_error() {
        let tmp = TempDir::new().unwrap();
        let cfg = Config {
            registry_url: String::new(),
            skills_dir: tmp.path().join("skills"),
            cache_dir: tmp.path().join("cache"),
            github_token: None,
            deploy_agents: vec![],
            last_suggest_date: None,
        };
        let inst = Installer::new(cfg);
        let r = SkillRef::Short("missing".to_string());
        assert!(inst.uninstall(&r).is_err());
    }
}
