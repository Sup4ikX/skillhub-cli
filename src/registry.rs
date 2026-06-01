use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::error::{Result, SkillHubError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registry {
    pub version: u32,
    pub skills: Vec<Skill>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub author: String,
    pub version: String,
    pub tags: Vec<String>,
    pub download_url: String,
    pub checksum: Option<String>,
    pub compatibility: Compatibility,
    pub quality: Option<Quality>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Compatibility {
    /// Agent kind IDs (e.g. `"claude-code"`, `"opencode"`) that this skill works with.
    #[serde(default)]
    pub agents: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quality {
    pub score: u8,
    pub security: u8,
    pub clarity: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkillRef {
    Qualified { owner: String, name: String },
    Short(String),
}

impl SkillRef {
    pub fn parse(input: &str) -> Result<Self> {
        if input.starts_with('@') {
            let without_at = &input[1..];
            if let Some((owner, name)) = without_at.split_once('/') {
                if owner.is_empty() || name.is_empty() {
                    return Err(SkillHubError::InvalidSkillName(input.to_string()));
                }
                return Ok(SkillRef::Qualified {
                    owner: owner.to_string(),
                    name: name.to_string(),
                });
            }
            return Err(SkillHubError::InvalidSkillName(input.to_string()));
        }

        if input.contains('/') {
            return Err(SkillHubError::InvalidSkillName(input.to_string()));
        }

        Ok(SkillRef::Short(input.to_string()))
    }

    pub fn matches(&self, skill: &Skill) -> bool {
        match self {
            SkillRef::Qualified { owner, name } => {
                skill.name == format!("@{}/{}", owner, name)
                    || skill.name == format!("{}/{}", owner, name)
            }
            SkillRef::Short(short) => {
                skill.name == *short
                    || skill.name.ends_with(&format!("/{}", short))
                    || skill.name == format!("@{}", short)
            }
        }
    }

    pub fn display(&self) -> String {
        match self {
            SkillRef::Qualified { owner, name } => format!("@{}/{}", owner, name),
            SkillRef::Short(name) => name.clone(),
        }
    }
}

// ---------------------------------------------------------------------------
// GitHub Search API types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct GitHubRepoItem {
    full_name: String,
    description: Option<String>,
    #[serde(default)]
    stargazers_count: usize,
    #[serde(default)]
    topics: Vec<String>,
    default_branch: String,
}

#[derive(Debug, Deserialize)]
struct GitHubSearchRepos {
    items: Vec<GitHubRepoItem>,
    total_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct GitHubSearchCache {
    skills: Vec<Skill>,
    cached_at: u64,
}

fn cache_key(query: &str) -> String {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    query.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

pub const GITHUB_API: &str = "https://api.github.com";

pub struct RegistryClient {
    config: Config,
}

impl RegistryClient {
    pub fn new(config: Config) -> Self {
        RegistryClient { config }
    }

    /// Get the registry source URL.
    pub fn registry_url(&self) -> &str {
        &self.config.registry_url
    }

    /// Get a reference to the GitHub token, if set.
    pub fn github_token(&self) -> Option<&str> {
        self.config.github_token()
    }

    /// Search GitHub repositories opted into `skillhub-skill` topic.
    /// Results are cached for 1 hour in the cache directory.
    /// Combine with an optional `query` for full-text filtering.
    pub fn search_github(&self, query: &str) -> Result<Vec<Skill>> {
        let cache_key = format!("github_search_{}.json", cache_key(query));
        let cache_path = self.config.cache_dir.join(&cache_key);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        // Check cache: valid for 1 hour
        if cache_path.exists() {
            if let Ok(raw) = std::fs::read_to_string(&cache_path) {
                if let Ok(cached) = serde_json::from_str::<GitHubSearchCache>(&raw) {
                    if now < cached.cached_at + 3600 {
                        return Ok(cached.skills);
                    }
                }
            }
        }

        let q = if query.is_empty() {
            "topic:skillhub-skill".to_string()
        } else {
            format!("topic:skillhub-skill+{}", query.replace(' ', "+"))
        };

        let url = format!("{}/search/repositories?q={}&per_page=30&sort=stars", GITHUB_API, q);
        let body = self.gh_get(&url)?;

        let resp: GitHubSearchRepos = serde_json::from_str(&body)
            .map_err(|e| SkillHubError::GitHubApi(format!("json: {}", e)))?;

        let mut out: Vec<Skill> = Vec::new();
        for item in resp.items {
            let raw_url = format!(
                "https://raw.githubusercontent.com/{}/{}/SKILL.md",
                item.full_name, item.default_branch
            );
            let description = item.description.unwrap_or_default();
            let tags: Vec<String> = item.topics.iter()
                .filter(|t| *t != "skillhub-skill")
                .map(|t| t.to_string())
                .collect();

            out.push(Skill {
                name: format!("@{}", item.full_name),
                description,
                author: item.full_name.split('/').next().unwrap_or("").to_string(),
                version: String::new(),
                tags,
                download_url: raw_url,
                checksum: None,
                compatibility: Compatibility { agents: vec![] },
                quality: None,
            });
        }

        // Save to cache
        if let Some(parent) = cache_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let cache_entry = GitHubSearchCache {
            skills: out.clone(),
            cached_at: now,
        };
        if let Ok(json) = serde_json::to_string(&cache_entry) {
            let _ = std::fs::write(&cache_path, json);
        }

        Ok(out)
    }

    pub fn fetch(&self) -> Result<Registry> {
        let body = self.gh_get(&self.config.registry_url)?;
        let registry: Registry = serde_json::from_str(&body)?;
        Ok(registry)
    }

    pub fn load_cache(&self) -> Result<Registry> {
        let cache_path = self.cache_path();

        if !cache_path.exists() {
            return Err(SkillHubError::RegistryNotFound);
        }

        let contents = fs::read_to_string(&cache_path)?;
        let registry: Registry = serde_json::from_str(&contents)?;
        Ok(registry)
    }

    pub fn save_cache(&self, registry: &Registry) -> Result<()> {
        let cache_path = self.cache_path();

        if let Some(parent) = cache_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let contents = serde_json::to_string_pretty(registry)?;
        fs::write(&cache_path, contents)?;
        Ok(())
    }

    pub fn update(&self) -> Result<Registry> {
        let registry = self.fetch()?;
        self.save_cache(&registry)?;
        Ok(registry)
    }

    pub fn search(&self, query: &str) -> Result<Vec<Skill>> {
        let registry = self.load_cache()?;
        let query_lower = query.to_lowercase();

        let results: Vec<Skill> = registry
            .skills
            .into_iter()
            .filter(|skill| {
                skill.name.to_lowercase().contains(&query_lower)
                    || skill.description.to_lowercase().contains(&query_lower)
                    || skill
                        .tags
                        .iter()
                        .any(|tag| tag.to_lowercase().contains(&query_lower))
            })
            .collect();

        Ok(results)
    }

    pub fn find_skill(&self, reference: &SkillRef) -> Result<Skill> {
        let registry = self.load_cache()?;

        registry
            .skills
            .into_iter()
            .find(|skill| reference.matches(skill))
            .ok_or_else(|| SkillHubError::SkillNotFound(reference.display()))
    }

    pub fn gh_get(&self, url: &str) -> Result<String> {
        let mut req = ureq::get(url).set("User-Agent", "skillhub/0.1.0");

        if let Some(token) = self.config.github_token() {
            req = req.set("Authorization", &format!("Bearer {}", token));
        } else if let Ok(token) = std::env::var("GITHUB_TOKEN") {
            req = req.set("Authorization", &format!("Bearer {}", token));
        }

        let response = req.call().map_err(|e| match e {
            ureq::Error::Status(status, _) => {
                if status == 401 || status == 403 {
                    SkillHubError::RateLimited
                } else {
                    SkillHubError::GitHubApi(format!("HTTP {}", status))
                }
            }
            other => SkillHubError::GitHubApi(format!("request failed: {}", other)),
        })?;

        response.into_string()
            .map_err(|e| SkillHubError::GitHubApi(format!("read failed: {}", e)))
    }

    fn cache_path(&self) -> PathBuf {
        self.config.cache_dir.join("registry.json")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn sample_skill(name: &str) -> Skill {
        Skill {
            name: name.to_string(),
            description: format!("A skill called {}", name),
            author: "test".to_string(),
            version: "1.0.0".to_string(),
            tags: vec!["test".to_string()],
            download_url: format!("https://example.com/{}.tar.gz", name),
            checksum: None,
            compatibility: Compatibility {
                agents: vec!["claude-code".to_string(), "opencode".to_string()],
            },
            quality: None,
        }
    }

    fn sample_registry() -> Registry {
        Registry {
            version: 1,
            skills: vec![
                sample_skill("@alice/code-review"),
                sample_skill("@bob/test-runner"),
                sample_skill("@alice/doc-writer"),
            ],
        }
    }

    fn tmp_config(dir: &Path) -> Config {
        Config {
            cache_dir: dir.to_path_buf(),
            skills_dir: dir.join("skills"),
            registry_url: String::new(),
            github_token: None,
            deploy_agents: vec![],
            last_suggest_date: None,
        }
    }

    #[test]
    fn skill_ref_qualified_parse() {
        let r = SkillRef::parse("@alice/code-review").unwrap();
        assert_eq!(
            r,
            SkillRef::Qualified {
                owner: "alice".to_string(),
                name: "code-review".to_string()
            }
        );
    }

    #[test]
    fn skill_ref_short_parse() {
        let r = SkillRef::parse("code-review").unwrap();
        assert_eq!(r, SkillRef::Short("code-review".to_string()));
    }

    #[test]
    fn skill_ref_invalid_no_name() {
        assert!(SkillRef::parse("@alice/").is_err());
    }

    #[test]
    fn skill_ref_invalid_no_owner() {
        assert!(SkillRef::parse("@/code-review").is_err());
    }

    #[test]
    fn skill_ref_invalid_slash_without_at() {
        assert!(SkillRef::parse("alice/code-review").is_err());
    }

    #[test]
    fn skill_ref_matches_qualified() {
        let r = SkillRef::Qualified {
            owner: "alice".to_string(),
            name: "code-review".to_string(),
        };
        let skill = sample_skill("@alice/code-review");
        assert!(r.matches(&skill));
    }

    #[test]
    fn skill_ref_matches_short() {
        let r = SkillRef::Short("code-review".to_string());
        let skill = sample_skill("@alice/code-review");
        assert!(r.matches(&skill));
    }

    #[test]
    fn skill_ref_no_match() {
        let r = SkillRef::Short("other".to_string());
        let skill = sample_skill("@alice/code-review");
        assert!(!r.matches(&skill));
    }

    #[test]
    fn search_finds_by_name() {
        let tmp = TempDir::new().unwrap();
        let client = RegistryClient::new(tmp_config(tmp.path()));
        client.save_cache(&sample_registry()).unwrap();

        let results = client.search("code").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "@alice/code-review");
    }

    #[test]
    fn search_finds_by_description() {
        let tmp = TempDir::new().unwrap();
        let client = RegistryClient::new(tmp_config(tmp.path()));
        client.save_cache(&sample_registry()).unwrap();

        let results = client.search("runner").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "@bob/test-runner");
    }

    #[test]
    fn search_finds_by_tag() {
        let tmp = TempDir::new().unwrap();
        let mut skill = sample_skill("@alice/special");
        skill.tags = vec!["documentation".to_string()];
        let registry = Registry {
            version: 1,
            skills: vec![skill],
        };

        let client = RegistryClient::new(tmp_config(tmp.path()));
        client.save_cache(&registry).unwrap();

        let results = client.search("documentation").unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn search_case_insensitive() {
        let tmp = TempDir::new().unwrap();
        let client = RegistryClient::new(tmp_config(tmp.path()));
        client.save_cache(&sample_registry()).unwrap();

        let results = client.search("CODE").unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn search_returns_empty_for_no_match() {
        let tmp = TempDir::new().unwrap();
        let client = RegistryClient::new(tmp_config(tmp.path()));
        client.save_cache(&sample_registry()).unwrap();

        let results = client.search("xyznonexistent").unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn find_skill_returns_error_when_missing() {
        let tmp = TempDir::new().unwrap();
        let client = RegistryClient::new(tmp_config(tmp.path()));
        client.save_cache(&sample_registry()).unwrap();

        let r = SkillRef::Short("nonexistent".to_string());
        assert!(client.find_skill(&r).is_err());
    }

    #[test]
    fn load_cache_missing_file_returns_error() {
        let tmp = TempDir::new().unwrap();
        let config = Config {
            cache_dir: tmp.path().join("empty"),
            skills_dir: tmp.path().join("skills"),
            registry_url: String::new(),
            github_token: None,
            deploy_agents: vec![],
            last_suggest_date: None,
        };

        let client = RegistryClient::new(config);
        assert!(client.load_cache().is_err());
    }

    #[test]
    fn save_and_load_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let client = RegistryClient::new(tmp_config(tmp.path()));

        let registry = sample_registry();
        client.save_cache(&registry).unwrap();

        let loaded = client.load_cache().unwrap();
        assert_eq!(loaded.version, 1);
        assert_eq!(loaded.skills.len(), 3);
    }
}
