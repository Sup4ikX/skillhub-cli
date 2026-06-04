use thiserror::Error;

#[derive(Error, Debug)]
pub enum SkillHubError {
    #[error("registry not found, run `skillhub update` to fetch")]
    RegistryNotFound,

    #[error("skill `{0}` not found in registry")]
    SkillNotFound(String),

    #[error("skill `{0}` is already installed")]
    AlreadyInstalled(String),

    #[error("skill `{0}` is not installed")]
    NotInstalled(String),

    #[error("failed to read/write file: {0}")]
    Io(#[from] std::io::Error),

    #[error("failed to parse JSON: {0}")]
    Json(#[from] serde_json::Error),

    #[error("failed to parse config: {0}")]
    ConfigParse(String),

    #[error("invalid skill name `{0}`, expected `@owner/name` or `name`")]
    InvalidSkillName(String),

    #[error("GitHub API rate limited, try again later or set GITHUB_TOKEN")]
    RateLimited,

    #[error("GitHub API error: {0}")]
    GitHubApi(String),
}

pub type Result<T> = std::result::Result<T, SkillHubError>;
