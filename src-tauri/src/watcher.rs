use notify::RecursiveMode;
use notify_debouncer_mini::{new_debouncer, DebounceEventResult, DebouncedEventKind};
use tauri::{AppHandle, Emitter, Manager};

fn is_ignored(path: &std::path::Path) -> bool {
    let s = path.to_string_lossy();
    s.contains("/node_modules/")
        || s.contains("/target/")
        || s.contains("/.git/objects/")
        || s.contains("/.git/lfs/")
}

pub fn watch_repo(app: &AppHandle, repo: std::path::PathBuf) {
    let handle = app.clone();
    std::thread::spawn(move || {
        let handler = move |res: DebounceEventResult| {
            let Ok(events) = res else { return };
            let relevant = events.iter().any(|e| {
                if e.kind == DebouncedEventKind::AnyContinuous {
                    return false;
                }
                !is_ignored(&e.path)
            });
            if relevant {
                let _ = handle.emit("repo-changed", "refresh");
            }
        };
        let mut debouncer = match new_debouncer(std::time::Duration::from_millis(300), handler) {
            Ok(d) => d,
            Err(_) => return,
        };
        {
            let _ = debouncer.watcher().watch(&repo, RecursiveMode::Recursive);
        }
        loop {
            std::thread::park();
        }
    });
}

pub fn watch_current(app: &AppHandle) {
    if let Some(state) = app.try_state::<crate::state::AppState>() {
        if let Ok(repo) = state.current_repo() {
            watch_repo(app, repo);
        }
    }
}
