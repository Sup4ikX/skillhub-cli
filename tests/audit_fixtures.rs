use std::fs;
use std::path::PathBuf;

use skillhub::Verdict;
use skillhub::audit_run;

fn fixture(name: &str) -> String {
    let p: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name);
    fs::read_to_string(&p).unwrap_or_else(|_| panic!("missing fixture: {:?}", p))
}

#[test]
fn fixture_clean_skill_passes() {
    let content = fixture("clean-skill.md");
    let r = audit_run(
        "@test/clean",
        &content,
        &["claude-code".to_string(), "opencode".to_string()],
    );
    assert_eq!(r.verdict, Verdict::Pass);
    assert!(r.security.passed);
    assert!(r.quality.score >= 70);
    assert!(r.overall_score >= 70);
}

#[test]
fn fixture_dirty_skill_fails() {
    let content = fixture("dirty-skill.md");
    let r = audit_run("@test/dirty", &content, &[]);
    assert_eq!(r.verdict, Verdict::Fail);
    assert!(!r.security.passed);
    assert!(r.security.high_count >= 1);
}

#[test]
fn fixture_thin_skill_warns_or_fails() {
    let content = fixture("thin-skill.md");
    let r = audit_run("@test/thin", &content, &["claude-code".to_string()]);
    assert!(r.quality.score < 60);
    assert!(matches!(r.verdict, Verdict::Warn | Verdict::Fail));
}

#[test]
fn fixture_claude_style_valid_frontmatter() {
    let content = fixture("claude-style.md");
    let r = audit_run("@test/claude", &content, &["claude-code".to_string()]);
    assert!(r.quality.frontmatter_present);
    assert!(r.quality.frontmatter_valid);
    assert!(r.quality.score >= 70);
}

#[test]
fn fixture_compat_per_agent_matches_claims() {
    let content = fixture("clean-skill.md");
    let r = audit_run(
        "@test/x",
        &content,
        &[
            "claude-code".to_string(),
            "opencode".to_string(),
            "cursor".to_string(),
        ],
    );
    assert_eq!(r.compatibility.verified_agents.len(), 3);
    assert!(r.compatibility.unverified_agents.is_empty());
}

#[test]
fn fixture_audit_serialization_roundtrip() {
    let content = fixture("clean-skill.md");
    let r = audit_run("@test/rt", &content, &["claude-code".to_string()]);
    let s = serde_json::to_string(&r).unwrap();
    let v: serde_json::Value = serde_json::from_str(&s).unwrap();
    assert_eq!(v["name"], "@test/rt");
    assert!(v["security"]["score"].is_number());
    assert!(v["quality"]["score"].is_number());
    assert!(v["compatibility"]["score"].is_number());
    assert!(v["overall_score"].is_number());
    assert!(v["verdict"].is_string());
}
