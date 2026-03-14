use std::collections::HashMap;
use std::process::Command;

/// Map from TTY (e.g. "ttys010") to tmux session name.
pub fn tty_to_session_map() -> HashMap<String, String> {
    let output = match Command::new("tmux")
        .args(["list-panes", "-a", "-F", "#{session_name}\t#{pane_tty}"])
        .output()
    {
        Ok(o) if o.status.success() => o,
        _ => return HashMap::new(),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut map = HashMap::new();

    for line in stdout.lines() {
        let mut parts = line.splitn(2, '\t');
        if let (Some(session), Some(tty)) = (parts.next(), parts.next()) {
            // pane_tty is like "/dev/ttys010", ps shows "ttys010"
            let tty_short = tty.strip_prefix("/dev/").unwrap_or(tty);
            map.insert(tty_short.to_string(), session.to_string());
        }
    }

    map
}

/// Switch to a tmux session.
pub fn switch_to_session(name: &str) {
    // switch-client works even from run-shell context
    let _ = Command::new("tmux")
        .args(["switch-client", "-t", name])
        .status();
}

/// Launch claude in a new tmux session with the given name and working directory.
/// Returns the session name on success.
pub fn create_session(name: &str, cwd: &str) -> Result<String, String> {
    let base_name = sanitize_session_name(name);

    // Always create a new session — append -2, -3, etc. if name taken
    let session_name = if !session_exists(&base_name) {
        base_name.clone()
    } else {
        let mut n = 2;
        loop {
            let candidate = format!("{base_name}-{n}");
            if !session_exists(&candidate) {
                break candidate;
            }
            n += 1;
        }
    };

    let claude_path = which_claude().unwrap_or_else(|| "claude".to_string());
    let status = Command::new("tmux")
        .args([
            "new-session",
            "-d",
            "-s",
            &session_name,
            "-c",
            cwd,
            &claude_path,
        ])
        .status()
        .map_err(|e| format!("Failed to create tmux session: {e}"))?;

    if !status.success() {
        return Err("tmux new-session failed".to_string());
    }

    Ok(session_name)
}

/// Get default session name and cwd for a new session.
pub fn default_new_session_info() -> (String, String) {
    let cwd = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| ".".to_string());

    let name = std::path::Path::new(&cwd)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "claude".to_string());

    (name, cwd)
}

fn session_exists(name: &str) -> bool {
    Command::new("tmux")
        .args(["has-session", "-t", name])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn which_claude() -> Option<String> {
    let output = Command::new("which").arg("claude").output().ok()?;
    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if path.is_empty() { None } else { Some(path) }
}

/// Sanitize a string for use as a tmux session name (no dots or colons).
fn sanitize_session_name(name: &str) -> String {
    name.replace('.', "-").replace(':', "-")
}
