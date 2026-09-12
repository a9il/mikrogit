use super::GitOutput;
use serde::Serialize;
use std::sync::OnceLock;
use tauri::{AppHandle, Emitter};

static HANDLE: OnceLock<AppHandle> = OnceLock::new();

pub fn set_handle(h: AppHandle) {
    let _ = HANDLE.set(h);
}

#[derive(Debug, Clone, Serialize)]
pub struct GitLogEntry {
    pub cmd: String,
    pub stdout: String,
    pub stderr: String,
    pub code: Option<i32>,
    pub ts: u64,
}

/// Read-only commands that run on every refresh are not logged — the console
/// records mutating / user-triggered operations only.
pub fn quiet(args: &[&str]) -> bool {
    let first = args.first().copied().unwrap_or("");
    matches!(
        first,
        "status"
            | "diff"
            | "show"
            | "ls-files"
            | "log"
            | "for-each-ref"
            | "rev-list"
            | "cat-file"
            | "diff-tree"
            | "check-ignore"
    ) || (first == "branch" && args.contains(&"--show-current"))
        || (first == "stash" && args.get(1) == Some(&"list"))
}

pub fn log_git(args: &[&str], out: &GitOutput, code: Option<i32>, force: bool) {
    if !force && quiet(args) {
        return;
    }
    let Some(app) = HANDLE.get() else { return };
    let entry = GitLogEntry {
        cmd: format!("git {}", args.join(" ")),
        stdout: out.stdout.clone(),
        stderr: out.stderr.clone(),
        code,
        ts: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0),
    };
    let _ = app.emit("git-log", entry);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refresh_commands_are_quiet() {
        assert!(quiet(&["status", "--porcelain=v2"]));
        assert!(quiet(&["diff", "--cached", "--", "a.txt"]));
        assert!(quiet(&["branch", "--show-current"]));
        assert!(quiet(&["stash", "list", "--format=x"]));
        assert!(quiet(&["log", "--max-count=200"]));
    }

    #[test]
    fn mutating_commands_are_logged() {
        assert!(!quiet(&["add", "src/"]));
        assert!(!quiet(&["reset", "HEAD", "--", "a.txt"]));
        assert!(!quiet(&["commit", "-m", "x"]));
        assert!(!quiet(&["push"]));
        assert!(!quiet(&["branch", "-D", "feature"]));
        assert!(!quiet(&["stash", "pop", "0"]));
    }
}
