use super::{to_repo_relative, GitError, Result};
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct TreeEntry {
    pub path: String,
    pub kind: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConflictRegion {
    pub current_header: String,
    pub current_lines: Vec<String>,
    pub incoming_header: String,
    pub incoming_lines: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConflictFile {
    pub path: String,
    pub regions: Vec<ConflictRegion>,
    pub has_conflicts: bool,
    pub original_content: String,
}

pub fn parse_conflicts(content: &str, path: &str) -> ConflictFile {
    let mut regions = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim_start();
        if trimmed.starts_with("<<<<<<< ") {
            let current_header = trimmed.trim_start_matches("<<<<<<< ").to_string();
            let mut current_lines = Vec::new();
            let mut incoming_header = String::new();
            let mut incoming_lines = Vec::new();
            i += 1;
            while i < lines.len() && !lines[i].trim_start().starts_with("=======") {
                current_lines.push(lines[i].to_string());
                i += 1;
            }
            if i < lines.len() {
                i += 1;
            }
            while i < lines.len() && !lines[i].trim_start().starts_with(">>>>>>> ") {
                incoming_lines.push(lines[i].to_string());
                i += 1;
            }
            if i < lines.len() {
                incoming_header = lines[i]
                    .trim_start()
                    .trim_start_matches(">>>>>>> ")
                    .to_string();
                i += 1;
            }
            regions.push(ConflictRegion {
                current_header,
                current_lines,
                incoming_header,
                incoming_lines,
            });
        } else {
            i += 1;
        }
    }
    ConflictFile {
        path: path.to_string(),
        has_conflicts: !regions.is_empty(),
        regions,
        original_content: content.to_string(),
    }
}

pub async fn read_conflicts(repo: &Path, path: &str) -> Result<ConflictFile> {
    let text = read_file(repo, path, 1024 * 1024).await?;
    Ok(parse_conflicts(&text, path))
}

pub fn resolve_conflicts(content: &str, choices: &[String]) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    let mut out = String::new();
    let mut region_idx: usize = 0;
    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim_start();
        if trimmed.starts_with("<<<<<<< ") {
            // Collect the full conflict block.
            let start = i;
            let mut current_lines = Vec::new();
            let mut incoming_lines = Vec::new();
            i += 1;
            while i < lines.len() && !lines[i].trim_start().starts_with("=======") {
                current_lines.push(lines[i]);
                i += 1;
            }
            if i < lines.len() {
                i += 1; // skip "======="
            }
            while i < lines.len() && !lines[i].trim_start().starts_with(">>>>>>> ") {
                incoming_lines.push(lines[i]);
                i += 1;
            }
            let end = i; // index of ">>>>>>>"
            if i < lines.len() {
                i += 1;
            }
            let choice = choices.get(region_idx);
            region_idx += 1;
            match choice {
                Some(c) if c == "current" => {
                    for l in &current_lines {
                        out.push_str(l);
                        out.push('\n');
                    }
                }
                Some(c) if c == "incoming" => {
                    for l in &incoming_lines {
                        out.push_str(l);
                        out.push('\n');
                    }
                }
                Some(c) if c == "both" => {
                    for l in &current_lines {
                        out.push_str(l);
                        out.push('\n');
                    }
                    for l in &incoming_lines {
                        out.push_str(l);
                        out.push('\n');
                    }
                }
                _ => {
                    // Unresolved — replay original lines from start..=end.
                    for idx in start..=end {
                        out.push_str(lines[idx]);
                        out.push('\n');
                    }
                }
            }
        } else {
            out.push_str(lines[i]);
            out.push('\n');
            i += 1;
        }
    }
    Some(out)
}

pub async fn list_files(repo: &Path, dir: Option<&str>) -> Result<Vec<TreeEntry>> {
    let base = match dir {
        Some(d) if !d.is_empty() => {
            let rel = to_repo_relative(repo, d);
            if rel.split('/').any(|p| p == "..") {
                return Err(GitError::Failed {
                    code: None,
                    stderr: "invalid path".to_string(),
                });
            }
            repo.join(&rel)
        }
        _ => repo.to_path_buf(),
    };
    if !base.is_dir() {
        return Err(GitError::Failed {
            code: None,
            stderr: "not a directory".to_string(),
        });
    }
    let mut entries = Vec::new();
    let mut rd = std::fs::read_dir(&base)?;
    while let Some(item) = rd.next() {
        let item = item?;
        let abs = item.path();
        let name = item.file_name().to_string_lossy().into_owned();
        if name == ".git" {
            continue;
        }
        let rel = abs
            .strip_prefix(repo)
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or(name.clone());
        let ft = item.file_type()?;
        if ft.is_symlink() {
            continue;
        }
        if ft.is_dir() {
            entries.push(TreeEntry {
                path: rel,
                kind: "dir".to_string(),
                size: 0,
            });
        } else if ft.is_file() {
            let size = item.metadata().map(|m| m.len()).unwrap_or(0);
            entries.push(TreeEntry {
                path: rel,
                kind: "file".to_string(),
                size,
            });
        }
    }
    entries.sort_by(|a, b| a.kind.cmp(&b.kind).then(a.path.cmp(&b.path)));
    Ok(entries)
}

pub async fn read_file(repo: &Path, path: &str, max_bytes: usize) -> Result<String> {
    let rel = super::to_repo_relative(repo, path);
    if rel.contains("..") {
        return Err(super::GitError::Failed {
            code: None,
            stderr: "invalid path".to_string(),
        });
    }
    let abs = repo.join(&rel);
    if !abs.is_file() {
        return Err(super::GitError::Failed {
            code: None,
            stderr: format!("not a file: {rel}"),
        });
    }
    let bytes = std::fs::read(&abs)?;
    let truncated = bytes.len() > max_bytes;
    let slice = if truncated { &bytes[..max_bytes] } else { &bytes[..] };
    if slice.contains(&0) {
        return Err(super::GitError::Failed {
            code: None,
            stderr: "binary file — no text preview".to_string(),
        });
    }
    let mut text = String::from_utf8_lossy(slice).into_owned();
    if truncated {
        text.push_str("\n… [truncated]");
    }
    Ok(text)
}

fn resolve(repo: &Path, path: &str) -> Result<std::path::PathBuf> {
    let rel = to_repo_relative(repo, path);
    if rel.is_empty() || rel.split('/').any(|p| p == "..") {
        return Err(GitError::Failed {
            code: None,
            stderr: "invalid path".to_string(),
        });
    }
    Ok(repo.join(&rel))
}

pub async fn create_file(repo: &Path, path: &str) -> Result<()> {
    let abs = resolve(repo, path)?;
    if abs.exists() {
        return Err(GitError::Failed {
            code: None,
            stderr: "already exists".to_string(),
        });
    }
    if let Some(parent) = abs.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&abs, "")?;
    Ok(())
}

pub async fn create_dir(repo: &Path, path: &str) -> Result<()> {
    let abs = resolve(repo, path)?;
    if abs.exists() {
        return Err(GitError::Failed {
            code: None,
            stderr: "already exists".to_string(),
        });
    }
    std::fs::create_dir_all(&abs)?;
    Ok(())
}

pub async fn rename_entry(repo: &Path, from: &str, to: &str) -> Result<()> {
    let src = resolve(repo, from)?;
    let dst = resolve(repo, to)?;
    if !src.exists() {
        return Err(GitError::Failed {
            code: None,
            stderr: "source not found".to_string(),
        });
    }
    if dst.exists() {
        return Err(GitError::Failed {
            code: None,
            stderr: "destination already exists".to_string(),
        });
    }
    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::rename(&src, &dst)?;
    Ok(())
}

pub async fn delete_entry(repo: &Path, path: &str) -> Result<()> {
    let abs = resolve(repo, path)?;
    if !abs.exists() {
        return Err(GitError::Failed {
            code: None,
            stderr: "not found".to_string(),
        });
    }
    if abs.is_dir() {
        std::fs::remove_dir_all(&abs)?;
    } else {
        std::fs::remove_file(&abs)?;
    }
    Ok(())
}

pub async fn save_file(repo: &Path, path: &str, content: &str) -> Result<()> {
    let abs = resolve(repo, path)?;
    if let Some(parent) = abs.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&abs, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn lists_all_filesystem_entries() {
        let dir = std::env::temp_dir().join(format!("mikrogit-files-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let git = |args: &[&str]| {
            std::process::Command::new("git")
                .args(args)
                .current_dir(&dir)
                .env("GIT_CONFIG_GLOBAL", "/dev/null")
                .env("GIT_CONFIG_SYSTEM", "/dev/null")
                .output()
                .unwrap()
        };
        git(&["init", "-b", "main"]);
        git(&["config", "user.email", "t@t.t"]);
        git(&["config", "user.name", "T"]);
        std::fs::create_dir_all(dir.join("sub")).unwrap();
        std::fs::create_dir_all(dir.join(".git")).unwrap();
        std::fs::write(dir.join("a.txt"), "hello\n").unwrap();
        std::fs::write(dir.join("sub/b.txt"), "world\n").unwrap();
        std::fs::write(dir.join("ignored.log"), "x\n").unwrap();
        std::fs::write(dir.join(".gitignore"), "*.log\n").unwrap();
        git(&["add", "a.txt", "sub/b.txt", ".gitignore"]);
        git(&["commit", "-m", "init"]);

        // Root listing: dirs first, .git skipped, ignored files included.
        let root = list_files(&dir, None).await.unwrap();
        let paths: Vec<&str> = root.iter().map(|e| e.path.as_str()).collect();
        assert!(paths.contains(&"a.txt"));
        assert!(paths.contains(&"sub"));
        assert!(paths.contains(&"ignored.log"));
        assert!(!paths.iter().any(|p| *p == ".git" || p.starts_with(".git/")));
        assert_eq!(root[0].kind, "dir");

        // Subdir listing.
        let sub = list_files(&dir, Some("sub")).await.unwrap();
        assert_eq!(sub.len(), 1);
        assert_eq!(sub[0].path, "sub/b.txt");

        // Path traversal rejected.
        assert!(list_files(&dir, Some("../outside")).await.is_err());

        let text = read_file(&dir, "a.txt", 1024).await.unwrap();
        assert_eq!(text, "hello\n");

        let err = read_file(&dir, "../outside.txt", 1024).await;
        assert!(err.is_err());

        create_file(&dir, "sub/created.txt").await.unwrap();
        assert!(dir.join("sub/created.txt").is_file());
        create_dir(&dir, "newdir/nested").await.unwrap();
        assert!(dir.join("newdir/nested").is_dir());
        rename_entry(&dir, "sub/created.txt", "sub/renamed.txt")
            .await
            .unwrap();
        assert!(dir.join("sub/renamed.txt").is_file());
        assert!(rename_entry(&dir, "../evil", "x").await.is_err());
        delete_entry(&dir, "sub/renamed.txt").await.unwrap();
        assert!(!dir.join("sub/renamed.txt").exists());
        delete_entry(&dir, "newdir").await.unwrap();
        assert!(!dir.join("newdir").exists());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn parses_conflict_markers() {
        let content = "before\n<<<<<<< HEAD\ncurrent line\n=======\nincoming line\n>>>>>>> feature\nafter\n";
        let cf = parse_conflicts(content, "f.txt");
        assert!(cf.has_conflicts);
        assert_eq!(cf.regions.len(), 1);
        let r = &cf.regions[0];
        assert_eq!(r.current_header, "HEAD");
        assert_eq!(r.current_lines, vec!["current line"]);
        assert_eq!(r.incoming_header, "feature");
        assert_eq!(r.incoming_lines, vec!["incoming line"]);
        assert_eq!(cf.original_content, content);

        let no_conflict = parse_conflicts("clean file\n", "f.txt");
        assert!(!no_conflict.has_conflicts);
    }

    #[test]
    fn resolves_conflict_choices() {
        let content = "a\n<<<<<<< HEAD\ncur\n=======\ninc\n>>>>>>> b\nc\n";
        let resolved = resolve_conflicts(content, &["incoming".to_string()]).unwrap();
        assert_eq!(resolved, "a\ninc\nc\n");

        let both = resolve_conflicts(content, &["both".to_string()]).unwrap();
        assert_eq!(both, "a\ncur\ninc\nc\n");

        let current = resolve_conflicts(content, &["current".to_string()]).unwrap();
        assert_eq!(current, "a\ncur\nc\n");

        // Unresolved keeps markers.
        let none = resolve_conflicts(content, &[]).unwrap();
        assert_eq!(none, content);
    }
}
