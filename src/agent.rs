use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentKind {
    ClaudeCode,
    Codex,
    Cursor,
    Copilot,
    Windsurf,
    GeminiCli,
    Aider,
    Cline,
    OpenCode,
    OpenClaw,
    RooCode,
    KiloCode,
    Crush,
    Factory,
    ContinueDev,
    QwenCode,
    Pi,
    AugmentCode,
    Trae,
    Codeium,
    Zed,
    Warp,
    Tabby,
    // New agents 2026
    Antigravity,
    KimiCode,
    KiroIde,
    Goose,
    AmazonQ,
    Cody,
    Devin,
    Poolside,
    Cosine,
}

impl AgentKind {
    /// Machine-readable identifier used in registry Compatibility.
    pub fn kind_id(&self) -> &str {
        match self {
            AgentKind::ClaudeCode => "claude-code",
            AgentKind::Codex => "codex",
            AgentKind::Cursor => "cursor",
            AgentKind::Copilot => "copilot",
            AgentKind::Windsurf => "windsurf",
            AgentKind::GeminiCli => "gemini-cli",
            AgentKind::Aider => "aider",
            AgentKind::Cline => "cline",
            AgentKind::OpenCode => "opencode",
            AgentKind::OpenClaw => "openclaw",
            AgentKind::RooCode => "roo-code",
            AgentKind::KiloCode => "kilo-code",
            AgentKind::Crush => "crush",
            AgentKind::Factory => "factory",
            AgentKind::ContinueDev => "continue-dev",
            AgentKind::QwenCode => "qwen-code",
            AgentKind::Pi => "pi",
            AgentKind::AugmentCode => "augment-code",
            AgentKind::Trae => "trae",
            AgentKind::Codeium => "codeium",
            AgentKind::Zed => "zed",
            AgentKind::Warp => "warp",
            AgentKind::Tabby => "tabby",
            AgentKind::Antigravity => "antigravity",
            AgentKind::KimiCode => "kimi-code",
            AgentKind::KiroIde => "kiro-ide",
            AgentKind::Goose => "goose",
            AgentKind::AmazonQ => "amazon-q",
            AgentKind::Cody => "cody",
            AgentKind::Devin => "devin",
            AgentKind::Poolside => "poolside",
            AgentKind::Cosine => "cosine",
        }
    }

    pub fn all() -> &'static [AgentKind] {
        &[
            AgentKind::ClaudeCode,
            AgentKind::Codex,
            AgentKind::Cursor,
            AgentKind::Copilot,
            AgentKind::Windsurf,
            AgentKind::GeminiCli,
            AgentKind::Aider,
            AgentKind::Cline,
            AgentKind::OpenCode,
            AgentKind::OpenClaw,
            AgentKind::RooCode,
            AgentKind::KiloCode,
            AgentKind::Crush,
            AgentKind::Factory,
            AgentKind::ContinueDev,
            AgentKind::QwenCode,
            AgentKind::Pi,
            AgentKind::AugmentCode,
            AgentKind::Trae,
            AgentKind::Codeium,
            AgentKind::Zed,
            AgentKind::Warp,
            AgentKind::Tabby,
            AgentKind::Antigravity,
            AgentKind::KimiCode,
            AgentKind::KiroIde,
            AgentKind::Goose,
            AgentKind::AmazonQ,
            AgentKind::Cody,
            AgentKind::Devin,
            AgentKind::Poolside,
            AgentKind::Cosine,
        ]
    }

    pub fn label(&self) -> &str {
        match self {
            AgentKind::ClaudeCode => "Claude Code",
            AgentKind::Codex => "Codex",
            AgentKind::Cursor => "Cursor",
            AgentKind::Copilot => "Copilot",
            AgentKind::Windsurf => "Windsurf",
            AgentKind::GeminiCli => "Gemini CLI",
            AgentKind::Aider => "Aider",
            AgentKind::Cline => "Cline",
            AgentKind::OpenCode => "OpenCode",
            AgentKind::OpenClaw => "OpenClaw",
            AgentKind::RooCode => "Roo Code",
            AgentKind::KiloCode => "Kilo Code",
            AgentKind::Crush => "Crush",
            AgentKind::Factory => "Factory",
            AgentKind::ContinueDev => "Continue.dev",
            AgentKind::QwenCode => "Qwen Code",
            AgentKind::Pi => "Pi",
            AgentKind::AugmentCode => "Augment Code",
            AgentKind::Trae => "Trae",
            AgentKind::Codeium => "Codeium",
            AgentKind::Zed => "Zed",
            AgentKind::Warp => "Warp",
            AgentKind::Tabby => "Tabby",
            AgentKind::Antigravity => "Antigravity",
            AgentKind::KimiCode => "Kimi Code",
            AgentKind::KiroIde => "Kiro IDE",
            AgentKind::Goose => "Goose",
            AgentKind::AmazonQ => "Amazon Q",
            AgentKind::Cody => "Cody",
            AgentKind::Devin => "Devin",
            AgentKind::Poolside => "Poolside",
            AgentKind::Cosine => "Cosine",
        }
    }

    /// Home config directory for this agent (global, not per-project).
    pub fn detect_home(&self) -> Option<PathBuf> {
        let home = dirs::home_dir()?;
        let cfg = dirs::config_dir();

        match self {
            AgentKind::ClaudeCode => Some(home.join(".claude")),
            AgentKind::Codex => Some(home.join(".codex")),
            AgentKind::Cursor => Some(home.join(".cursor")),
            AgentKind::Copilot => cfg
                .as_deref()
                .map(|d| d.join("github-copilot"))
                .or_else(|| Some(home.join(".config").join("github-copilot"))),
            AgentKind::Windsurf => Some(home.join(".windsurf")),
            AgentKind::GeminiCli => Some(home.join(".gemini")),
            AgentKind::Aider => Some(home.join(".aider")),
            AgentKind::Cline => Some(home.join(".cline")),
            AgentKind::OpenCode => cfg
                .as_deref()
                .map(|d| d.join("opencode"))
                .or_else(|| Some(home.join(".config").join("opencode"))),
            AgentKind::OpenClaw => Some(home.join(".openclaw")),
            AgentKind::RooCode => Some(home.join(".roo")),
            AgentKind::KiloCode => Some(home.join(".kilo")),
            AgentKind::Crush => Some(home.join(".crush")),
            AgentKind::Factory => Some(home.join(".factory")),
            AgentKind::ContinueDev => Some(home.join(".continue")),
            AgentKind::QwenCode => Some(home.join(".qwen")),
            AgentKind::Pi => Some(home.join(".pi")),
            AgentKind::AugmentCode => Some(home.join(".augment")),
            AgentKind::Trae => Some(home.join(".trae")),
            AgentKind::Codeium => Some(home.join(".codeium")),
            AgentKind::Zed => cfg
                .as_deref()
                .map(|d| d.join("zed"))
                .or_else(|| Some(home.join(".config").join("zed"))),
            AgentKind::Warp => Some(home.join(".warp")),
            AgentKind::Tabby => Some(home.join(".tabby")),
            AgentKind::Antigravity => Some(home.join(".antigravity")),
            AgentKind::KimiCode => Some(home.join(".kimi")),
            AgentKind::KiroIde => Some(home.join(".kiro")),
            AgentKind::Goose => cfg
                .as_deref()
                .map(|d| d.join("goose"))
                .or_else(|| Some(home.join(".config").join("goose"))),
            AgentKind::AmazonQ => cfg
                .as_deref()
                .map(|d| d.join("amazon-q"))
                .or_else(|| Some(home.join(".aws").join("q"))),
            AgentKind::Cody => Some(home.join(".cody")),
            AgentKind::Devin => Some(home.join(".devin")),
            AgentKind::Poolside => Some(home.join(".poolside")),
            AgentKind::Cosine => Some(home.join(".cosine")),
        }
    }

    pub fn detect(&self) -> bool {
        self.detect_home().map(|p| p.exists()).unwrap_or(false)
    }

    /// Where this agent looks for global (user-wide) skills or instructions.
    pub fn skills_dir(&self) -> Option<PathBuf> {
        let base = self.detect_home()?;
        match self {
            // agents with a dedicated skills/ sub-directory
            AgentKind::ClaudeCode => Some(base.join("skills")),
            AgentKind::Codex => Some(base.join("memories")),
            AgentKind::Cursor => Some(base.join("skills")),
            AgentKind::Cline => Some(base.join("data").join("skills")),
            AgentKind::Copilot => Some(base.join("instructions")),
            AgentKind::Crush => Some(base.join("skills")),
            AgentKind::Pi => Some(base.join("skills")),
            AgentKind::Factory => Some(base.join("skills")),
            AgentKind::Trae => Some(base.join("skills")),
            AgentKind::Codeium => Some(base.join("skills")),
            AgentKind::OpenCode => Some(base.join("agents")),
            AgentKind::OpenClaw => Some(base.join("skills")),

            // agents that use commands/ or rules/ — skills land in a shared spot
            AgentKind::GeminiCli => Some(base.join("commands")),
            AgentKind::QwenCode => Some(base.join("commands")),
            AgentKind::KiloCode => Some(base.join("rules")),
            AgentKind::RooCode => Some(base.join("rules")),
            AgentKind::AugmentCode => Some(base.join("rules")),

            // agents that pull from project-level markdown; store as global fallback
            AgentKind::Windsurf => Some(base.join("skills")),
            AgentKind::Aider => Some(base),
            AgentKind::ContinueDev => Some(base.join("prompts")),
            AgentKind::Zed => Some(base.join("skills")),
            AgentKind::Warp => Some(base.join("skills")),
            AgentKind::Tabby => Some(base.join("skills")),
            AgentKind::Antigravity => Some(base.join("skills")),
            AgentKind::KimiCode => Some(base.join("commands")),
            AgentKind::KiroIde => Some(base.join("agents")),
            AgentKind::Goose => Some(base.join("skills")),
            AgentKind::AmazonQ => Some(base.join("instructions")),
            AgentKind::Cody => Some(base.join("skills")),
            AgentKind::Devin => Some(base.join("skills")),
            AgentKind::Poolside => Some(base.join("skills")),
            AgentKind::Cosine => Some(base.join("skills")),
        }
    }

    /// Per-project file / directory name (e.g. `CLAUDE.md`, `.cursor/rules/`).
    /// Returns `None` if this agent doesn't have a well-known project-level artefact.
    pub fn project_file(&self) -> Option<&str> {
        match self {
            AgentKind::ClaudeCode => Some("CLAUDE.md"),
            AgentKind::Codex => Some("AGENTS.md"),
            AgentKind::Cursor => Some(".cursor/rules"),
            AgentKind::Copilot => Some(".github/copilot-instructions.md"),
            AgentKind::Windsurf => Some(".windsurf/rules"),
            AgentKind::GeminiCli => Some("GEMINI.md"),
            AgentKind::Aider => Some("AGENTS.md"),
            AgentKind::Cline => Some(".clinerules"),
            AgentKind::OpenCode => Some(".opencode"),
            AgentKind::OpenClaw => Some(".openclaw"),
            AgentKind::RooCode => Some(".roo/rules"),
            AgentKind::KiloCode => Some(".kilo/rules"),
            AgentKind::Crush => Some("CRUSH.md"),
            AgentKind::Factory => Some(".factory"),
            AgentKind::ContinueDev => Some(".continue"),
            AgentKind::QwenCode => Some("QWEN.md"),
            AgentKind::Pi => Some(".pi"),
            AgentKind::AugmentCode => Some(".augment/rules"),
            AgentKind::Trae => Some(".trae"),
            AgentKind::Codeium => Some(".codeium"),
            AgentKind::Zed => Some(".zed"),
            AgentKind::Warp => Some("AGENTS.md"),
            AgentKind::Tabby => Some(".tabby"),
            AgentKind::Antigravity => Some("ANTIGRAVITY.md"),
            AgentKind::KimiCode => Some("KIMI.md"),
            AgentKind::KiroIde => Some(".kiro"),
            AgentKind::Goose => Some("GOOSE.md"),
            AgentKind::AmazonQ => Some(".amazon-q"),
            AgentKind::Cody => Some(".cody"),
            AgentKind::Devin => Some(".devin"),
            AgentKind::Poolside => Some(".poolside"),
            AgentKind::Cosine => Some(".cosine"),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentConfig {
    pub agents: Vec<AgentKind>,
}

impl AgentConfig {
    pub fn detect() -> Self {
        let agents: Vec<AgentKind> = AgentKind::all()
            .iter()
            .filter(|a| a.detect())
            .cloned()
            .collect();
        AgentConfig { agents }
    }

    pub fn any_found(&self) -> bool {
        !self.agents.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_agents_have_labels() {
        for a in AgentKind::all() {
            assert!(!a.label().is_empty());
        }
    }

    #[test]
    fn all_agents_have_skills_dir() {
        for a in AgentKind::all() {
            assert!(a.skills_dir().is_some(), "{} has no skills_dir", a.label());
        }
    }

    #[test]
    fn all_agents_have_project_file() {
        for a in AgentKind::all() {
            assert!(
                a.project_file().is_some(),
                "{} has no project_file",
                a.label()
            );
        }
    }

    #[test]
    fn detect_home_always_returns() {
        for a in AgentKind::all() {
            assert!(a.detect_home().is_some(), "{} has no home", a.label());
        }
    }

    #[test]
    fn label_is_unique() {
        let mut seen = std::collections::HashSet::new();
        for a in AgentKind::all() {
            assert!(seen.insert(a.label()));
        }
    }
}
