use indicatif::{ProgressBar, ProgressStyle};

/// Returns true if progress bars should be suppressed.
pub fn no_progress() -> bool {
    std::env::var("CI").is_ok() || std::env::var("NO_COLOR").is_ok()
}

/// Create a spinner for indeterminate operations.
pub fn spinner(force_no_progress: bool) -> Option<ProgressBar> {
    if force_no_progress || no_progress() {
        return None;
    }
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
    );
    Some(pb)
}

/// Create a progress bar for downloads with known size.
pub fn progress_bar(len: u64, msg: &str, force_no_progress: bool) -> Option<ProgressBar> {
    if force_no_progress || no_progress() {
        return None;
    }
    let pb = ProgressBar::new(len);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} {msg}")
            .unwrap()
            .progress_chars("##-"),
    );
    pb.set_message(msg.to_string());
    Some(pb)
}

/// Download a skill file with an optional progress bar.
/// Returns the body as a String.
pub fn download_with_progress(
    url: &str,
    config: &crate::config::Config,
    force_no_progress: bool,
) -> Result<String, crate::error::SkillHubError> {
    use std::io::Read;

    let parsed: url::Url = url
        .parse()
        .map_err(|e| crate::error::SkillHubError::GitHubApi(format!("bad url: {}", e)))?;

    if parsed.scheme() != "https" {
        return Err(crate::error::SkillHubError::GitHubApi(
            "https required".to_string(),
        ));
    }

    let ok_hosts = [
        "raw.githubusercontent.com",
        "github.com",
        "gist.githubusercontent.com",
    ];
    let host = parsed.host_str().unwrap_or("");
    if !ok_hosts.contains(&host) {
        return Err(crate::error::SkillHubError::GitHubApi(format!(
            "host not allowed: {}",
            host
        )));
    }

    let mut req = ureq::get(parsed.as_str()).set("User-Agent", "skillhub/0.1.0");
    if let Some(t) = config.github_token() {
        req = req.set("Authorization", &format!("Bearer {}", t));
    } else if let Ok(t) = std::env::var("GITHUB_TOKEN") {
        req = req.set("Authorization", &format!("Bearer {}", t));
    }

    let resp = req
        .call()
        .map_err(|e| crate::error::SkillHubError::GitHubApi(format!("download failed: {}", e)))?;

    let total = resp
        .header("Content-Length")
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0);

    let pb = if total > 0 {
        progress_bar(total, "downloading...", force_no_progress)
    } else {
        spinner(force_no_progress)
    };

    let mut body = String::new();
    let mut reader = resp.into_reader();

    if let Some(ref bar) = pb {
        let mut buf = [0; 4096];
        loop {
            let n = reader.read(&mut buf).map_err(|e| {
                crate::error::SkillHubError::GitHubApi(format!("read error: {}", e))
            })?;
            if n == 0 {
                break;
            }
            body.push_str(&String::from_utf8_lossy(&buf[..n]));
            bar.inc(n as u64);
        }
        bar.finish_and_clear();
    } else {
        reader
            .read_to_string(&mut body)
            .map_err(|e| crate::error::SkillHubError::GitHubApi(format!("read error: {}", e)))?;
    }

    Ok(body)
}
