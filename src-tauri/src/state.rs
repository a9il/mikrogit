use crate::git::{GitError, Result};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};

pub struct AppState {
    pub repo: Mutex<Option<PathBuf>>,
    pub git_path: String,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            repo: Mutex::new(None),
            git_path: super::git::git_binary(),
        }
    }

    pub fn current_repo(&self) -> Result<PathBuf> {
        self.repo
            .lock()
            .map_err(|e| GitError::Io(e.to_string()))?
            .clone()
            .ok_or(GitError::NoRepo)
    }

    pub fn set_repo(&self, path: PathBuf) {
        if let Ok(mut guard) = self.repo.lock() {
            *guard = Some(path);
        }
    }

    pub fn clear_repo(&self) {
        if let Ok(mut guard) = self.repo.lock() {
            *guard = None;
        }
    }
}

pub fn emit_refresh(app: &AppHandle) {
    let _ = app.emit("repo-changed", "refresh");
}
