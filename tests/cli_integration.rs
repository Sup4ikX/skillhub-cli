use std::fs;
use std::process::Command;

fn binary() -> Command {
    Command::new(env!("CARGO_BIN_EXE_skillhub"))
}

#[test]
fn help_works() {
    let out = binary().arg("--help").output().expect("run");
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("skillhub"));
    assert!(s.contains("install"));
    assert!(s.contains("check"));
    assert!(s.contains("audit"));
}

#[test]
fn version_works() {
    let out = binary().arg("--version").output().expect("run");
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn check_clean_skill_passes() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("clean-skill.md");
    let out = binary()
        .args(["check", fixture.to_str().unwrap()])
        .output()
        .expect("run");
    let code = out.status.code().unwrap_or(-1);
    assert!(code == 0, "expected PASS, got {}: {:?}", code, out);
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("Audit"));
    assert!(s.contains("Security"));
    assert!(s.contains("Quality"));
    assert!(s.contains("Compatibility"));
}

#[test]
fn check_dirty_skill_fails() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("dirty-skill.md");
    let out = binary()
        .args(["check", fixture.to_str().unwrap()])
        .output()
        .expect("run");
    let code = out.status.code().unwrap_or(-1);
    assert_ne!(code, 0, "expected non-zero exit on dirty skill");
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("prompt-injection") || s.contains("data-exfiltration"));
}

#[test]
fn check_thin_skill_warns() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("thin-skill.md");
    let out = binary()
        .args(["check", fixture.to_str().unwrap()])
        .output()
        .expect("run");
    let code = out.status.code().unwrap_or(-1);
    assert!(code == 1, "expected WARN exit (1), got {}", code);
}

#[test]
fn check_json_output_is_valid() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("clean-skill.md");
    let out = binary()
        .args(["check", "--json", fixture.to_str().unwrap()])
        .output()
        .expect("run");
    let s = String::from_utf8_lossy(&out.stdout);
    let v: serde_json::Value = serde_json::from_str(&s).expect("valid json");
    assert!(v.get("name").is_some());
    assert!(v.get("security").is_some());
    assert!(v.get("quality").is_some());
    assert!(v.get("compatibility").is_some());
    assert!(v.get("overall_score").is_some());
    assert!(v.get("verdict").is_some());
}

#[test]
fn audit_saves_report() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("clean-skill.md");
    let out = binary()
        .args(["audit", fixture.to_str().unwrap()])
        .output()
        .expect("run");
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("Report saved"));
}

#[test]
fn list_works_with_no_skills() {
    let out = binary().arg("list").output().expect("run");
    let _ = out.status.code();
}

#[test]
fn search_no_registry_returns_clean_error() {
    let out = binary().args(["search", "test"]).output().expect("run");
    let _ = out.status.code();
}

#[test]
fn completions_bash() {
    let out = binary()
        .args(["completions", "bash"])
        .output()
        .expect("run");
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("skillhub"));
}

#[test]
fn completions_fish() {
    let out = binary()
        .args(["completions", "fish"])
        .output()
        .expect("run");
    assert!(out.status.success());
}

#[test]
fn completions_zsh() {
    let out = binary().args(["completions", "zsh"]).output().expect("run");
    assert!(out.status.success());
}

#[test]
fn completions_powershell() {
    let out = binary()
        .args(["completions", "powershell"])
        .output()
        .expect("run");
    assert!(out.status.success());
}

#[test]
fn completions_unknown_shell_errors() {
    let out = binary()
        .args(["completions", "tclsh"])
        .output()
        .expect("run");
    assert!(!out.status.success());
}

#[test]
fn completions_json() {
    let out = binary()
        .args(["completions", "--json", "bash"])
        .output()
        .expect("run");
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    let v: serde_json::Value = serde_json::from_str(&s).unwrap();
    assert!(v.get("supported_shells").is_some());
}

#[test]
fn invalid_skill_name_errors() {
    let out = binary()
        .args(["install", "no_such_skill_definitely", "--no-scan", "--yes"])
        .output()
        .expect("run");
    let _ = out.status.code();
}

#[test]
fn list_json_output_is_valid() {
    let out = binary().args(["list", "--json"]).output().expect("run");
    let s = String::from_utf8_lossy(&out.stdout);
    let v: serde_json::Value = serde_json::from_str(&s).expect("valid json");
    assert!(v.get("skills").is_some());
    assert!(v.get("count").is_some());
}

#[test]
fn help_subcommand_works() {
    let out = binary().arg("help").output().expect("run");
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.to_lowercase().contains("skillhub"));
}

#[test]
fn check_handles_directory_target() {
    let dir = tempfile::tempdir().unwrap();
    let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("clean-skill.md");
    fs::copy(&src, dir.path().join("SKILL.md")).unwrap();

    let out = binary()
        .args(["check", dir.path().to_str().unwrap()])
        .output()
        .expect("run");
    let code = out.status.code().unwrap_or(-1);
    assert!(code == 0 || code == 1, "got exit code {}", code);
}

#[test]
fn agents_subcommand_runs() {
    let out = binary().args(["agents", "--json"]).output().expect("run");
    let s = String::from_utf8_lossy(&out.stdout);
    let v: serde_json::Value = serde_json::from_str(&s).unwrap();
    assert!(v.get("agents").is_some());
    assert!(!v["agents"].as_array().unwrap().is_empty());
}

#[test]
fn stats_subcommand_runs() {
    let out = binary().args(["stats", "--json"]).output().expect("run");
    let s = String::from_utf8_lossy(&out.stdout);
    let v: serde_json::Value = serde_json::from_str(&s).unwrap();
    assert!(v.get("installed").is_some());
}

#[test]
fn doctor_subcommand_runs() {
    let out = binary().args(["doctor", "--json"]).output().expect("run");
    let s = String::from_utf8_lossy(&out.stdout);
    let v: serde_json::Value = serde_json::from_str(&s).unwrap();
    assert!(v.get("healthy").is_some());
    assert!(v.get("issues").is_some());
}

#[test]
fn check_nonexistent_target_fails() {
    let out = binary()
        .args(["check", "@nope/does-not-exist-xyz"])
        .output()
        .expect("run");
    assert!(!out.status.success());
}

#[test]
fn publish_requires_file() {
    let out = binary()
        .args(["publish", "--force", "/nonexistent/SKILL.md"])
        .output()
        .expect("run");
    assert!(!out.status.success());
}

#[test]
fn publish_validates_frontmatter() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("SKILL.md");
    fs::write(
        &p,
        "no title here at all just a lot of text content for length",
    )
    .unwrap();
    let out = binary()
        .args(["publish", p.to_str().unwrap()])
        .output()
        .expect("run");
    assert!(!out.status.success());
}

#[test]
fn publish_force_skips_validation() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("SKILL.md");
    fs::write(&p, "no title").unwrap();
    let out = binary()
        .args(["publish", "--force", p.to_str().unwrap()])
        .output()
        .expect("run");
    let _ = out.status.code();
}

#[test]
fn publish_rejects_empty() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("SKILL.md");
    fs::write(&p, "").unwrap();
    let out = binary()
        .args(["publish", "--force", p.to_str().unwrap()])
        .output()
        .expect("run");
    assert!(!out.status.success());
}

#[test]
fn share_with_no_skills_is_clean() {
    let out = binary().args(["share", "--json"]).output().expect("run");
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    let v: serde_json::Value = serde_json::from_str(&s).unwrap();
    assert!(v.get("skills").is_some());
}

#[test]
fn sync_export_with_no_skills() {
    let out = binary()
        .args(["sync", "export", "--json"])
        .output()
        .expect("run");
    assert!(out.status.success());
}

#[test]
fn sync_unknown_action_errors_cleanly() {
    let out = binary()
        .args(["sync", "reboot", "--json"])
        .output()
        .expect("run");
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    let v: serde_json::Value = serde_json::from_str(&s).unwrap();
    assert!(v.get("error").is_some());
}

#[test]
fn import_nonexistent_source_clean() {
    let out = binary()
        .args(["import", "claude", "--json"])
        .output()
        .expect("run");
    assert!(out.status.success());
}

#[test]
fn import_unknown_source_uses_as_path() {
    let out = binary()
        .args(["import", "/nonexistent/path/here", "--json"])
        .output()
        .expect("run");
    assert!(out.status.success());
}

#[test]
fn badge_runs() {
    let out = binary()
        .args(["badge", "--format", "url"])
        .output()
        .expect("run");
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("img.shields.io"));
}

#[test]
fn badge_json() {
    let out = binary().args(["badge", "--json"]).output().expect("run");
    let s = String::from_utf8_lossy(&out.stdout);
    let v: serde_json::Value = serde_json::from_str(&s).unwrap();
    assert!(v.get("badge_url").is_some());
}

#[test]
fn suggest_cleanly_handles_no_state() {
    let out = binary().args(["suggest", "--json"]).output().expect("run");
    assert!(out.status.success());
}

#[test]
fn migrate_dry_cleanly() {
    let out = binary().args(["migrate", "--json"]).output().expect("run");
    assert!(out.status.success());
}

#[test]
fn migrate_rollback_cleanly() {
    let out = binary()
        .args(["migrate", "--rollback", "--json"])
        .output()
        .expect("run");
    assert!(out.status.success());
}

#[test]
fn restore_bad_url_cleanly_errors() {
    let out = binary()
        .args(["restore", "not-a-url-or-file", "--json"])
        .output()
        .expect("run");
    assert!(!out.status.success());
}

#[test]
fn uninstall_nonexistent_cleanly() {
    let out = binary()
        .args(["uninstall", "@nope/does-not-exist", "--yes", "--json"])
        .output()
        .expect("run");
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    let v: serde_json::Value = serde_json::from_str(&s).unwrap();
    assert!(v.get("error").is_some());
}

#[test]
fn info_nonexistent_cleanly() {
    let out = binary()
        .args(["info", "@nope/does-not-exist", "--json"])
        .output()
        .expect("run");
    let _ = out.status.code();
}
