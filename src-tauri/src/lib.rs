pub mod commands;
pub mod git;
pub mod state;
pub mod watcher;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::open_repo,
            commands::close_repo,
            commands::current_repo,
            commands::get_status,
            commands::get_diff,
            commands::stage,
            commands::stage_all,
            commands::unstage,
            commands::unstage_all,
            commands::discard,
            commands::clean,
            commands::commit,
            commands::fetch,
            commands::pull,
            commands::push,
            commands::sync,
            commands::list_branches,
            commands::checkout,
            commands::create_branch,
            commands::delete_branch,
            commands::stash_list,
            commands::stash_create,
            commands::stash_apply,
            commands::stash_pop,
            commands::stash_drop,
            commands::get_log,
            commands::get_commit_files,
            commands::get_commit_diff,
            commands::status_bar,
            commands::detect_git,
            commands::list_files,
            commands::read_file,
            commands::create_file,
            commands::create_dir,
            commands::rename_entry,
            commands::delete_entry,
            commands::read_conflicts,
            commands::save_file,
            commands::save_and_stage,
            commands::run_custom,
        ])
        .setup(|app| {
            crate::git::console::set_handle(app.handle().clone());
            // Auto-open a repo passed via env (used by the WebDriver E2E suite
            // and handy for launching directly into a repo).
            if let Ok(p) = std::env::var("MIKROGIT_REPO") {
                if let Ok(root) = crate::git::repo_root(std::path::Path::new(&p)) {
                    use tauri::{Emitter, Manager};
                    let handle = app.handle().clone();
                    handle.state::<AppState>().set_repo(root.clone());
                    crate::watcher::watch_repo(&handle, root.clone());
                    let _ = handle.emit("repo-opened", root.to_string_lossy().to_string());
                }
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
