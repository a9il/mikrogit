use super::{run_git, GitError, Result};
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FileStatus {
    Modified,
    Added,
    Deleted,
    Renamed,
    Copied,
    Untracked,
    Ignored,
    Unmerged,
}

impl FileStatus {
    fn from_xy(c: char) -> Option<Self> {
        match c {
            'M' => Some(FileStatus::Modified),
            'A' => Some(FileStatus::Added),
            'D' => Some(FileStatus::Deleted),
            'R' => Some(FileStatus::Renamed),
            'C' => Some(FileStatus::Copied),
            '?' => Some(FileStatus::Untracked),
            '!' => Some(FileStatus::Ignored),
            'U' => Some(FileStatus::Unmerged),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct FileEntry {
    pub path: String,
    pub original_path: Option<String>,
    pub index_status: Option<FileStatus>,
    pub worktree_status: Option<FileStatus>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct BranchInfo {
    pub name: Option<String>,
    pub upstream: Option<String>,
    pub ahead: u32,
    pub behind: u32,
    pub detached: bool,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct RepoStatus {
    pub branch: BranchInfo,
    pub staged: Vec<FileEntry>,
    pub unstaged: Vec<FileEntry>,
    pub untracked: Vec<FileEntry>,
    pub unmerged: Vec<FileEntry>,
}

fn parse_branch_header(line: &str, branch: &mut BranchInfo) {
    let body = line.strip_prefix("# branch.").unwrap_or(line);
    if let Some(head) = body.strip_prefix("head ") {
        if head == "(detached)" {
            branch.detached = true;
            branch.name = None;
        } else {
            branch.name = Some(head.to_string());
        }
    } else if let Some(up) = body.strip_prefix("upstream ") {
        branch.upstream = if up == "(upstream)" {
            None
        } else {
            Some(up.to_string())
        };
    } else if let Some(ab) = body.strip_prefix("ab ") {
        for part in ab.split_whitespace() {
            if let Some(n) = part.strip_prefix('+') {
                branch.ahead = n.parse().unwrap_or(0);
            } else if let Some(n) = part.strip_prefix('-') {
                branch.behind = n.parse().unwrap_or(0);
            }
        }
    }
}

pub fn parse_porcelain_v2(output: &str) -> RepoStatus {
    let mut status = RepoStatus::default();
    for line in output.lines() {
        if line.starts_with("# branch.") {
            parse_branch_header(line, &mut status.branch);
            continue;
        }
        if line.starts_with("# ") {
            continue;
        }
        let mut chars = line.chars();
        let kind = chars.next().unwrap_or(' ');
        match kind {
            '1' => {
                // 1 <XY> <subm> <mH> <mI> <mW> <hH> <hI> <path>
                let rest = &line[2..];
                let mut parts = rest.splitn(9, ' ');
                let xy = parts.next().unwrap_or("..");
                for _ in 0..6 {
                    parts.next();
                }
                let path = parts.next().unwrap_or("").to_string();
                let x: char = xy.chars().next().unwrap_or('.');
                let y: char = xy.chars().nth(1).unwrap_or('.');
                push_change(&mut status, &path, None, x, y);
            }
            '2' => {
                // 2 <XY> ... <score> <path><TAB><origPath>  (or space-separated when no rename)
                let tab_split: Vec<&str> = line.split('\t').collect();
                let (head, orig) = if tab_split.len() == 2 {
                    (tab_split[0], Some(tab_split[1].to_string()))
                } else {
                    (line, None)
                };
                let head_parts: Vec<&str> = head.splitn(10, ' ').collect();
                let xy = head_parts.get(1).copied().unwrap_or("..");
                let path = head_parts.get(9).copied().unwrap_or("").to_string();
                let x: char = xy.chars().next().unwrap_or('.');
                let y: char = xy.chars().nth(1).unwrap_or('.');
                push_change(&mut status, &path, orig, x, y);
            }
            'u' => {
                // u <XY> ... <path>
                let parts: Vec<&str> = line.splitn(11, ' ').collect();
                let xy = parts.get(1).copied().unwrap_or("UU");
                let path = parts.get(10).copied().unwrap_or("").to_string();
                let _ = xy;
                status.unmerged.push(FileEntry {
                    path,
                    original_path: None,
                    index_status: Some(FileStatus::Unmerged),
                    worktree_status: Some(FileStatus::Unmerged),
                });
            }
            '?' => {
                let path = line[2..].to_string();
                status.untracked.push(FileEntry {
                    path,
                    original_path: None,
                    index_status: None,
                    worktree_status: Some(FileStatus::Untracked),
                });
            }
            '!' => {}
            _ => {}
        }
    }
    status
}

fn push_change(status: &mut RepoStatus, path: &str, orig: Option<String>, x: char, y: char) {
    let xs = FileStatus::from_xy(x);
    let ys = FileStatus::from_xy(y);
    if x != '.' && x != '?' && x != '!' {
        status.staged.push(FileEntry {
            path: path.to_string(),
            original_path: orig.clone(),
            index_status: xs,
            worktree_status: None,
        });
    }
    if y != '.' && y != '?' && y != '!' {
        status.unstaged.push(FileEntry {
            path: path.to_string(),
            original_path: orig,
            index_status: None,
            worktree_status: ys,
        });
    }
}

pub async fn get_status(repo: &Path) -> Result<RepoStatus> {
    let out = run_git(
        Some(repo),
        &[
            "status",
            "--porcelain=v2",
            "--branch",
            "-uall",
            "--no-renames",
            "--ignore-submodules=dirty",
        ],
    )
    .await
    .map_err(|e| match e {
        GitError::Failed { stderr, .. } => GitError::Failed { code: None, stderr },
        other => other,
    })?;
    Ok(parse_porcelain_v2(&out.stdout))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_branch_and_changes() {
        let out = "# branch.head main\n# branch.upstream origin/main\n# branch.ab +2 -1\n1 M. .... 100644 100644 100644 abc123 abc123 src/a.ts\n1 .M .... 100644 100644 100644 abc123 abc123 src/b.ts\n? new.txt\n";
        let s = parse_porcelain_v2(out);
        assert_eq!(s.branch.name.as_deref(), Some("main"));
        assert_eq!(s.branch.ahead, 2);
        assert_eq!(s.branch.behind, 1);
        assert_eq!(s.staged.len(), 1);
        assert_eq!(s.staged[0].path, "src/a.ts");
        assert_eq!(s.unstaged.len(), 1);
        assert_eq!(s.unstaged[0].path, "src/b.ts");
        assert_eq!(s.untracked.len(), 1);
    }

    #[test]
    fn parses_rename_with_tab() {
        let out = "2 R. .... 100644 100644 100644 abc def R100 new.ts\told.ts\n";
        let s = parse_porcelain_v2(out);
        assert_eq!(s.staged.len(), 1);
        assert_eq!(s.staged[0].path, "new.ts");
        assert_eq!(s.staged[0].original_path.as_deref(), Some("old.ts"));
    }

    #[test]
    fn parses_unmerged_and_detached() {
        let out =
            "# branch.head (detached)\nu UU .... 100644 100644 100644 100644 a b c conflict.ts\n";
        let s = parse_porcelain_v2(out);
        assert!(s.branch.detached);
        assert_eq!(s.unmerged.len(), 1);
    }
}
