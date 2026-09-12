use crate::git::{diff, ops, status, GitError, Result};
use crate::state::AppState;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, State};

fn repo_of(state: &State<AppState>) -> Result<PathBuf> {
    state.current_repo()
}

#[tauri::command]
pub async fn open_repo(app: AppHandle, state: State<'_, AppState>, path: String) -> Result<String> {
    let p = PathBuf::from(path);
    let root = crate::git::repo_root(&p)?;
    state.set_repo(root.clone());
    crate::watcher::watch_repo(&app, root.clone());
    let _ = app.emit("repo-opened", root.to_string_lossy().to_string());
    Ok(root.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn close_repo(state: State<'_, AppState>) -> Result<()> {
    state.clear_repo();
    Ok(())
}

#[tauri::command]
pub async fn current_repo(state: State<'_, AppState>) -> Result<Option<String>> {
    Ok(state
        .repo
        .lock()
        .map_err(|e| GitError::Io(e.to_string()))?
        .clone()
        .map(|p| p.to_string_lossy().to_string()))
}

#[tauri::command]
pub async fn get_status(state: State<'_, AppState>) -> Result<status::RepoStatus> {
    let repo = repo_of(&state)?;
    status::get_status(&repo).await
}

#[tauri::command]
pub async fn get_diff(
    state: State<'_, AppState>,
    path: String,
    staged: bool,
) -> Result<diff::FileDiff> {
    let repo = repo_of(&state)?;
    diff::get_diff(&repo, &path, staged).await
}

#[tauri::command]
pub async fn stage(state: State<'_, AppState>, paths: Vec<String>) -> Result<()> {
    let repo = repo_of(&state)?;
    ops::stage(&repo, &paths).await
}

#[tauri::command]
pub async fn stage_all(state: State<'_, AppState>) -> Result<()> {
    let repo = repo_of(&state)?;
    ops::stage_all(&repo).await
}

#[tauri::command]
pub async fn unstage(state: State<'_, AppState>, paths: Vec<String>) -> Result<()> {
    let repo = repo_of(&state)?;
    ops::unstage(&repo, &paths).await
}

#[tauri::command]
pub async fn unstage_all(state: State<'_, AppState>) -> Result<()> {
    let repo = repo_of(&state)?;
    ops::unstage_all(&repo).await
}

#[tauri::command]
pub async fn discard(state: State<'_, AppState>, paths: Vec<String>) -> Result<()> {
    let repo = repo_of(&state)?;
    ops::discard(&repo, &paths).await
}

#[tauri::command]
pub async fn clean(state: State<'_, AppState>, paths: Vec<String>) -> Result<()> {
    let repo = repo_of(&state)?;
    ops::clean(&repo, &paths).await
}

#[tauri::command]
pub async fn commit(state: State<'_, AppState>, message: String, amend: bool) -> Result<String> {
    let repo = repo_of(&state)?;
    ops::commit(&repo, &message, amend).await
}

#[tauri::command]
pub async fn fetch(state: State<'_, AppState>, remote: Option<String>) -> Result<String> {
    let repo = repo_of(&state)?;
    ops::fetch(&repo, remote.as_deref()).await
}

#[tauri::command]
pub async fn pull(state: State<'_, AppState>) -> Result<String> {
    let repo = repo_of(&state)?;
    ops::pull(&repo).await
}

#[tauri::command]
pub async fn push(state: State<'_, AppState>, set_upstream: bool) -> Result<String> {
    let repo = repo_of(&state)?;
    ops::push(&repo, set_upstream).await
}

#[tauri::command]
pub async fn sync(state: State<'_, AppState>) -> Result<String> {
    let repo = repo_of(&state)?;
    ops::sync(&repo).await
}

#[tauri::command]
pub async fn list_branches(state: State<'_, AppState>) -> Result<Vec<ops::Branch>> {
    let repo = repo_of(&state)?;
    ops::list_branches(&repo).await
}

#[tauri::command]
pub async fn checkout(state: State<'_, AppState>, name: String) -> Result<()> {
    let repo = repo_of(&state)?;
    ops::checkout(&repo, &name).await
}

#[tauri::command]
pub async fn create_branch(state: State<'_, AppState>, name: String, checkout: bool) -> Result<()> {
    let repo = repo_of(&state)?;
    ops::create_branch(&repo, &name, checkout).await
}

#[tauri::command]
pub async fn delete_branch(state: State<'_, AppState>, name: String, force: bool) -> Result<()> {
    let repo = repo_of(&state)?;
    ops::delete_branch(&repo, &name, force).await
}

#[tauri::command]
pub async fn stash_list(state: State<'_, AppState>) -> Result<Vec<ops::StashEntry>> {
    let repo = repo_of(&state)?;
    ops::stash_list(&repo).await
}

#[tauri::command]
pub async fn stash_create(
    state: State<'_, AppState>,
    message: Option<String>,
    include_untracked: bool,
) -> Result<()> {
    let repo = repo_of(&state)?;
    ops::stash_create(&repo, message.as_deref(), include_untracked).await
}

#[tauri::command]
pub async fn stash_apply(state: State<'_, AppState>, index: usize) -> Result<()> {
    let repo = repo_of(&state)?;
    ops::stash_apply(&repo, index).await
}

#[tauri::command]
pub async fn stash_pop(state: State<'_, AppState>, index: usize) -> Result<()> {
    let repo = repo_of(&state)?;
    ops::stash_pop(&repo, index).await
}

#[tauri::command]
pub async fn stash_drop(state: State<'_, AppState>, index: usize) -> Result<()> {
    let repo = repo_of(&state)?;
    ops::stash_drop(&repo, index).await
}

#[tauri::command]
pub async fn get_log(
    state: State<'_, AppState>,
    limit: usize,
    skip: usize,
) -> Result<Vec<ops::CommitEntry>> {
    let repo = repo_of(&state)?;
    ops::get_log(&repo, limit.min(500), skip).await
}

#[tauri::command]
pub async fn get_commit_files(state: State<'_, AppState>, hash: String) -> Result<Vec<String>> {
    let repo = repo_of(&state)?;
    ops::get_commit_files(&repo, &hash).await
}

#[tauri::command]
pub async fn get_commit_diff(
    state: State<'_, AppState>,
    hash: String,
    path: String,
) -> Result<diff::FileDiff> {
    let repo = repo_of(&state)?;
    diff::get_commit_diff(&repo, &hash, &path).await
}

#[tauri::command]
pub async fn status_bar(state: State<'_, AppState>) -> Result<ops::StatusBarInfo> {
    let repo = repo_of(&state)?;
    let s = status::get_status(&repo).await?;
    Ok(ops::StatusBarInfo {
        branch: s.branch.name,
        ahead: s.branch.ahead,
        behind: s.branch.behind,
        staged_count: s.staged.len(),
        unstaged_count: s.unstaged.len(),
        untracked_count: s.untracked.len(),
        sync_state: "idle".to_string(),
    })
}

#[tauri::command]
pub async fn detect_git() -> Result<String> {
    let out = crate::git::run_git(None, &["--version"]).await?;
    Ok(out.stdout.trim().to_string())
}

#[tauri::command]
pub async fn list_files(
    state: State<'_, AppState>,
    dir: Option<String>,
) -> Result<Vec<crate::git::files::TreeEntry>> {
    let repo = repo_of(&state)?;
    crate::git::files::list_files(&repo, dir.as_deref()).await
}

#[tauri::command]
pub async fn read_file(
    state: State<'_, AppState>,
    path: String,
    max_bytes: Option<usize>,
) -> Result<String> {
    let repo = repo_of(&state)?;
    crate::git::files::read_file(&repo, &path, max_bytes.unwrap_or(64 * 1024).min(1024 * 1024)).await
}

#[tauri::command]
pub async fn create_file(state: State<'_, AppState>, path: String) -> Result<()> {
    let repo = repo_of(&state)?;
    crate::git::files::create_file(&repo, &path).await
}

#[tauri::command]
pub async fn create_dir(state: State<'_, AppState>, path: String) -> Result<()> {
    let repo = repo_of(&state)?;
    crate::git::files::create_dir(&repo, &path).await
}

#[tauri::command]
pub async fn rename_entry(state: State<'_, AppState>, from: String, to: String) -> Result<()> {
    let repo = repo_of(&state)?;
    crate::git::files::rename_entry(&repo, &from, &to).await
}

#[tauri::command]
pub async fn delete_entry(state: State<'_, AppState>, path: String) -> Result<()> {
    let repo = repo_of(&state)?;
    crate::git::files::delete_entry(&repo, &path).await
}

#[tauri::command]
pub async fn read_conflicts(
    state: State<'_, AppState>,
    path: String,
) -> Result<crate::git::files::ConflictFile> {
    let repo = repo_of(&state)?;
    crate::git::files::read_conflicts(&repo, &path).await
}

#[tauri::command]
pub async fn save_file(state: State<'_, AppState>, path: String, content: String) -> Result<()> {
    let repo = repo_of(&state)?;
    crate::git::files::save_file(&repo, &path, &content).await
}

#[tauri::command]
pub async fn save_and_stage(
    state: State<'_, AppState>,
    path: String,
    content: String,
) -> Result<()> {
    let repo = repo_of(&state)?;
    crate::git::files::save_file(&repo, &path, &content).await?;
    crate::git::ops::stage(&repo, &[path]).await?;
    Ok(())
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CustomGitResult {
    pub stdout: String,
    pub stderr: String,
    pub code: Option<i32>,
}

#[tauri::command]
pub async fn run_custom(
    state: State<'_, AppState>,
    args: Vec<String>,
) -> Result<CustomGitResult> {
    let repo = repo_of(&state)?;
    let argrefs: Vec<&str> = args.iter().map(String::as_str).collect();
    if argrefs.is_empty() {
        return Err(GitError::Failed {
            code: None,
            stderr: "empty command".to_string(),
        });
    }
    match crate::git::run_git_verbose(Some(&repo), &argrefs).await {
        Ok(out) => Ok(CustomGitResult {
            stdout: out.stdout,
            stderr: out.stderr,
            code: Some(0),
        }),
        Err(GitError::Failed { code, stderr }) => Ok(CustomGitResult {
            stdout: String::new(),
            stderr,
            code,
        }),
        Err(e) => Err(e),
    }
}
