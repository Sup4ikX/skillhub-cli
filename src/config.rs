use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::agent::AgentKind;
use crate::error::{Result, SkillHubError};

const DEFAULT_REGISTRY_URL: &str =
    "https://raw.githubusercontent.com/skillhub/registry/main/skills.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub registry_url: String,
    pub skills_dir: PathBuf,
    pub cache_dir: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub github_token: Option<String>,
    #[serde(default)]
    pub deploy_agents: Vec<AgentKind>,
    /// Last date a suggestion was shown (YYYY-MM-DD), to enforce 1/day limit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_suggest_date: Option<String>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = config_path()?;

        if config_path.exists() {
            let contents = std::fs::read_to_string(&config_path)?;
            let config: Config =
                toml::from_str(&contents).map_err(|e| SkillHubError::ConfigParse(e.to_string()))?;
            return Ok(config);
        }

        let config = Config::default();
        config.save()?;
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let config_path = config_path()?;

        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let contents =
            toml::to_string_pretty(self).map_err(|e| SkillHubError::ConfigParse(e.to_string()))?;
        std::fs::write(&config_path, contents)?;
        Ok(())
    }

    pub fn is_setup(&self) -> bool {
        self.github_token.is_some() || std::env::var("GITHUB_TOKEN").is_ok()
    }

    pub fn github_token(&self) -> Option<&str> {
        self.github_token.as_deref()
    }
}

impl Default for Config {
    fn default() -> Self {
        let base = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".skillhub");

        Config {
            registry_url: DEFAULT_REGISTRY_URL.to_string(),
            skills_dir: base.join("skills"),
            cache_dir: base.join("cache"),
            github_token: None,
            deploy_agents: Vec::new(),
        }
    }
}

fn config_path() -> Result<PathBuf> {
    let base = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".skillhub");
    Ok(base.join("config.toml"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn default_paths_are_under_home() {
        let cfg = Config::default();
        let home = dirs::home_dir().unwrap();
        assert!(cfg.skills_dir.starts_with(&home));
        assert!(cfg.cache_dir.starts_with(&home));
    }

    #[test]
    fn save_load_matches() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path().join(".skillhub");
        std::fs::create_dir_all(&base).unwrap();

        let cfg = Config {
            registry_url: "https://example.com/r.json".to_string(),
            skills_dir: base.join("skills"),
            cache_dir: base.join("cache"),
            github_token: Some("ghp_x".to_string()),
            deploy_agents: vec![AgentKind::ClaudeCode],
        };

        let p = base.join("config.toml");
        let raw = toml::to_string_pretty(&cfg).unwrap();
        std::fs::write(&p, raw).unwrap();

        let loaded_raw = std::fs::read_to_string(&p).unwrap();
        let loaded: Config = toml::from_str(&loaded_raw).unwrap();

        assert_eq!(cfg.registry_url, loaded.registry_url);
        assert_eq!(cfg.github_token, loaded.github_token);
        assert_eq!(loaded.deploy_agents.len(), 1);
    }

    #[test]
    fn token_optional() {
        let cfg = Config::default();
        assert!(!cfg.is_setup());
        assert!(cfg.github_token().is_none());
    }

    #[test]
    fn bad_toml_fails() {
        let r: std::result::Result<Config, _> = toml::from_str("[[[");
        assert!(r.is_err());
    }
}
