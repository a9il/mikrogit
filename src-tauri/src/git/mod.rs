pub mod console;
pub mod diff;
pub mod files;
pub mod ops;
pub mod status;

pub use diff::{DiffHunk, DiffLine, FileDiff};
pub use files::{ConflictFile, ConflictRegion, TreeEntry};
pub use ops::{Branch, CommitEntry, StashEntry, StatusBarInfo};
pub use status::{BranchInfo, FileEntry, FileStatus, RepoStatus};

use std::path::{Path, PathBuf};
use std::process::Stdio;
use thiserror::Error;
use tokio::process::Command;

#[derive(Debug, Error)]
pub enum GitError {
    #[error("git failed (exit {code:?}): {stderr}")]
    Failed { code: Option<i32>, stderr: String },
    #[error("no repository open")]
    NoRepo,
    #[error("io error: {0}")]
    Io(String),
    #[error("git binary not found: {0}")]
    GitNotFound(String),
}

impl serde::Serialize for GitError {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<std::io::Error> for GitError {
    fn from(e: std::io::Error) -> Self {
        GitError::Io(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, GitError>;

pub struct GitOutput {
    pub stdout: String,
    pub stderr: String,
}

pub fn git_binary() -> String {
    std::env::var("MIKROGIT_GIT").unwrap_or_else(|_| "git".to_string())
}

pub async fn run_git(repo: Option<&Path>, args: &[&str]) -> Result<GitOutput> {
    run_git_inner(repo, args, false).await
}

/// Like run_git but always records the command in the console log
/// (used for commands typed by the user in the terminal panel).
pub async fn run_git_verbose(repo: Option<&Path>, args: &[&str]) -> Result<GitOutput> {
    run_git_inner(repo, args, true).await
}

async fn run_git_inner(repo: Option<&Path>, args: &[&str], force_log: bool) -> Result<GitOutput> {
    let mut cmd = Command::new(git_binary());
    // Avoid paging prompts from interactive git. Only for subcommands
    // (bare flags like `--version` reject `-c`).
    let needs_config = args.first().is_some_and(|a| !a.starts_with('-'));
    if needs_config {
        cmd.arg("-c").arg("core.pager=cat");
    }
    cmd.args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("GIT_OPTIONAL_LOCKS", "0")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_PAGER", "cat");
    if let Some(dir) = repo {
        cmd.current_dir(dir);
    }

    let child = cmd.spawn().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            GitError::GitNotFound(format!(
                "could not execute `{}`: is git installed and on PATH?",
                git_binary()
            ))
        } else {
            GitError::Io(e.to_string())
        }
    })?;
    let out = child.wait_with_output().await?;
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    console::log_git(
        args,
        &GitOutput {
            stdout: stdout.clone(),
            stderr: stderr.clone(),
        },
        out.status.code(),
        force_log,
    );
    if out.status.success() {
        Ok(GitOutput { stdout, stderr })
    } else {
        Err(GitError::Failed {
            code: out.status.code(),
            stderr: stderr.trim().to_string(),
        })
    }
}

pub fn repo_root(start: &Path) -> Result<PathBuf> {
    let mut current: PathBuf = if start.is_file() {
        match start.parent() {
            Some(parent) => parent.to_path_buf(),
            None => start.to_path_buf(),
        }
    } else {
        start.to_path_buf()
    };
    loop {
        if current.join(".git").exists() {
            return Ok(current);
        }
        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => break,
        }
    }
    Err(GitError::Failed {
        code: None,
        stderr: format!("not a git repository: {}", start.display()),
    })
}

pub fn to_repo_relative(repo: &Path, path: &str) -> String {
    let p = Path::new(path);
    if p.is_absolute() {
        if let Ok(rel) = p.strip_prefix(repo) {
            return rel.to_string_lossy().replace('\\', "/");
        }
    }
    path.replace('\\', "/")
}
