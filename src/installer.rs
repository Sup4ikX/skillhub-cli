use std::fs;
use std::path::{Path, PathBuf};

use crate::agent::AgentKind;
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
        Installer { config, no_progress: false }
    }

    pub fn install(&self, skill: &Skill) -> Result<Vec<PathBuf>> {
        let mut installed = Vec::new();

        // always install to canonical skillhub dir
        let skill_dir = self.skill_dir(&skill.name)?;
        let content = self.download_skill(&skill.download_url)?;

        if !skill_dir.exists() {
            fs::create_dir_all(&skill_dir)?;
            fs::write(skill_dir.join("SKILL.md"), &content)?;

            let meta = serde_json::to_string_pretty(skill)?;
            fs::write(skill_dir.join("meta.json"), meta)?;
            installed.push(skill_dir.clone());
        }

        // deploy to each configured agent
        for agent in &self.config.deploy_agents {
            let target = agent.skills_dir();
            if target.is_none() {
                continue;
            }
            let target = target.unwrap();

            let agent_skill_dir = target.join(&skill.name.replace('@', "").replace('/', "_"));
            if agent_skill_dir.exists() {
                continue;
            }

            fs::create_dir_all(&target)?;
            fs::write(&agent_skill_dir, &content)?;
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
        Ok(self.list_installed()?
            .into_iter()
            .any(|(_, s)| reference.matches(&s)))
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
                        name: format!("@{}/{}", owner_name.to_string_lossy(), skill_name.to_string_lossy()),
                        description: String::new(),
                        author: owner_name.to_string_lossy().to_string(),
                        version: String::new(),
                        tags: vec![],
                        download_url: String::new(),
                        checksum: None,
                        compatibility: crate::registry::Compatibility {
                            agents: vec![],
                        },
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
        fs::write(dir.join("meta.json"), serde_json::to_string_pretty(&skill).unwrap()).unwrap();

        let cfg = Config {
            registry_url: String::new(),
            skills_dir: tmp.path().join("skills"),
            cache_dir: tmp.path().join("cache"),
            github_token: None,
            deploy_agents: vec![],
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
        };
        let inst = Installer::new(cfg);
        assert!(inst.skill_dir("no-slash").is_err());
    }
}
