use super::{run_git, to_repo_relative, Result};
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct DiffLine {
    pub kind: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_no: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_no: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiffHunk {
    pub header: String,
    pub old_start: u32,
    pub old_lines: u32,
    pub new_start: u32,
    pub new_lines: u32,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FileDiff {
    pub path: String,
    pub old_path: Option<String>,
    pub hunks: Vec<DiffHunk>,
    pub binary: bool,
    pub added: u32,
    pub removed: u32,
}

fn parse_numstat(out: &str) -> (u32, u32) {
    let parts: Vec<&str> = out.split_whitespace().collect();
    if parts.len() >= 2 {
        let a = if parts[0] == "-" {
            0
        } else {
            parts[0].parse().unwrap_or(0)
        };
        let r = if parts[1] == "-" {
            0
        } else {
            parts[1].parse().unwrap_or(0)
        };
        (a, r)
    } else {
        (0, 0)
    }
}

pub async fn get_diff(repo: &Path, path: &str, staged: bool) -> Result<FileDiff> {
    let rel = to_repo_relative(repo, path);
    let mut args: Vec<&str> = vec![
        "diff",
        "--no-color",
        "--no-ext-diff",
        "--unified=3",
        "--src-prefix=a/",
        "--dst-prefix=b/",
    ];
    if staged {
        args.push("--cached");
    }
    args.push("--");
    args.push(&rel);
    let out = run_git(Some(repo), &args).await?;

    let numstat_args: Vec<&str> = if staged {
        vec!["diff", "--cached", "--numstat", "--", &rel]
    } else {
        vec!["diff", "--numstat", "--", &rel]
    };
    let numstat = run_git(Some(repo), &numstat_args)
        .await
        .map(|o| o.stdout.lines().next().map(parse_numstat).unwrap_or((0, 0)))
        .unwrap_or((0, 0));

    // Untracked files have no git diff; show whole file as added.
    if out.stdout.trim().is_empty() && !staged {
        let abs = repo.join(&rel);
        if abs.is_file() {
            if let Ok(content) = std::fs::read_to_string(&abs) {
                let line_count = content.lines().count() as u32;
                let lines: Vec<DiffLine> = content
                    .lines()
                    .enumerate()
                    .map(|(i, l)| DiffLine {
                        kind: "add".to_string(),
                        content: l.to_string(),
                        old_no: None,
                        new_no: Some(i as u32 + 1),
                    })
                    .collect();
                return Ok(FileDiff {
                    path: rel.clone(),
                    old_path: None,
                    hunks: vec![DiffHunk {
                        header: "@@ -0,0 +1 @@".to_string(),
                        old_start: 0,
                        old_lines: 0,
                        new_start: 1,
                        new_lines: line_count,
                        lines,
                    }],
                    binary: false,
                    added: line_count,
                    removed: 0,
                });
            } else {
                return Ok(FileDiff {
                    path: rel,
                    old_path: None,
                    hunks: vec![],
                    binary: true,
                    added: 0,
                    removed: 0,
                });
            }
        }
    }

    let mut hunks: Vec<DiffHunk> = Vec::new();
    let mut binary =
        out.stdout.contains("Binary files ") || out.stdout.contains("GIT binary patch");
    let mut current: Option<DiffHunk> = None;
    let mut old_no = 0u32;
    let mut new_no = 0u32;
    let mut old_path: Option<String> = None;

    for raw in out.stdout.lines() {
        if raw.starts_with("Binary files ") || raw.starts_with("GIT binary patch") {
            binary = true;
            continue;
        }
        if let Some(rest) = raw.strip_prefix("--- ") {
            if rest != "/dev/null" {
                old_path = Some(rest.trim_start_matches("a/").to_string());
            }
            continue;
        }
        if raw.starts_with("+++ ") {
            continue;
        }
        if let Some(h) = raw.strip_prefix("@@ ") {
            if let Some(cur) = current.take() {
                hunks.push(cur);
            }
            let nums: Vec<u32> = h
                .split(['-', '+', ',', '@', ' '])
                .filter_map(|s| s.parse().ok())
                .collect();
            let (os, ol, ns, nl) = match nums.as_slice() {
                [a, b, c, d] => (*a, *b, *c, *d),
                [a, c] => (*a, 1, *c, 1),
                _ => (0, 0, 0, 0),
            };
            old_no = os;
            new_no = ns;
            current = Some(DiffHunk {
                header: format!("@@ {h}"),
                old_start: os,
                old_lines: ol,
                new_start: ns,
                new_lines: nl,
                lines: vec![],
            });
            continue;
        }
        let Some(cur) = current.as_mut() else {
            continue;
        };
        if let Some(add) = raw.strip_prefix('+') {
            if raw.starts_with("+++") {
                continue;
            }
            cur.lines.push(DiffLine {
                kind: "add".to_string(),
                content: add.to_string(),
                old_no: None,
                new_no: Some(new_no),
            });
            new_no += 1;
        } else if let Some(del) = raw.strip_prefix('-') {
            cur.lines.push(DiffLine {
                kind: "del".to_string(),
                content: del.to_string(),
                old_no: Some(old_no),
                new_no: None,
            });
            old_no += 1;
        } else if let Some(ctx) = raw.strip_prefix(' ') {
            cur.lines.push(DiffLine {
                kind: "context".to_string(),
                content: ctx.to_string(),
                old_no: Some(old_no),
                new_no: Some(new_no),
            });
            old_no += 1;
            new_no += 1;
        } else if raw.starts_with('\\') {
            continue;
        }
    }
    if let Some(cur) = current.take() {
        hunks.push(cur);
    }

    Ok(FileDiff {
        path: rel,
        old_path,
        hunks,
        binary,
        added: numstat.0,
        removed: numstat.1,
    })
}

pub async fn get_commit_diff(repo: &Path, hash: &str, path: &str) -> Result<FileDiff> {
    let rel = to_repo_relative(repo, path);
    let out = run_git(
        Some(repo),
        &[
            "show",
            "--no-color",
            "--no-ext-diff",
            "--unified=3",
            "--format=",
            hash,
            "--",
            &rel,
        ],
    )
    .await?;
    // Reuse hunk parsing by delegating: parse show output same as diff.
    let mut hunks: Vec<DiffHunk> = Vec::new();
    let mut binary = out.stdout.contains("Binary files ");
    let mut current: Option<DiffHunk> = None;
    let mut old_no = 0u32;
    let mut new_no = 0u32;
    for raw in out.stdout.lines() {
        if raw.starts_with("Binary files ") || raw.starts_with("GIT binary patch") {
            binary = true;
            continue;
        }
        if raw.starts_with("--- ") || raw.starts_with("+++ ") {
            continue;
        }
        if let Some(h) = raw.strip_prefix("@@ ") {
            if let Some(cur) = current.take() {
                hunks.push(cur);
            }
            let nums: Vec<u32> = h
                .split(['-', '+', ',', '@', ' '])
                .filter_map(|s| s.parse().ok())
                .collect();
            let (os, ol, ns, nl) = match nums.as_slice() {
                [a, b, c, d] => (*a, *b, *c, *d),
                [a, c] => (*a, 1, *c, 1),
                _ => (0, 0, 0, 0),
            };
            old_no = os;
            new_no = ns;
            current = Some(DiffHunk {
                header: format!("@@ {h}"),
                old_start: os,
                old_lines: ol,
                new_start: ns,
                new_lines: nl,
                lines: vec![],
            });
            continue;
        }
        let Some(cur) = current.as_mut() else {
            continue;
        };
        if let Some(add) = raw.strip_prefix('+') {
            cur.lines.push(DiffLine {
                kind: "add".to_string(),
                content: add.to_string(),
                old_no: None,
                new_no: Some(new_no),
            });
            new_no += 1;
        } else if let Some(del) = raw.strip_prefix('-') {
            cur.lines.push(DiffLine {
                kind: "del".to_string(),
                content: del.to_string(),
                old_no: Some(old_no),
                new_no: None,
            });
            old_no += 1;
        } else if let Some(ctx) = raw.strip_prefix(' ') {
            cur.lines.push(DiffLine {
                kind: "context".to_string(),
                content: ctx.to_string(),
                old_no: Some(old_no),
                new_no: Some(new_no),
            });
            old_no += 1;
            new_no += 1;
        }
    }
    if let Some(cur) = current.take() {
        hunks.push(cur);
    }
    let (added, removed) = (0u32, 0u32);
    Ok(FileDiff {
        path: rel,
        old_path: None,
        hunks,
        binary,
        added,
        removed,
    })
}
