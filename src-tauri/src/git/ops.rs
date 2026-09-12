use super::{run_git, to_repo_relative, Result};
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct Branch {
    pub name: String,
    pub current: bool,
    pub remote: bool,
    pub upstream: Option<String>,
    pub ahead: u32,
    pub behind: u32,
    pub last_commit: String,
    pub last_commit_subject: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct StashEntry {
    pub index: usize,
    pub message: String,
    pub branch: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CommitEntry {
    pub hash: String,
    pub short_hash: String,
    pub author: String,
    pub email: String,
    pub date: String,
    pub subject: String,
    pub body: String,
    pub parents: Vec<String>,
    pub refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StatusBarInfo {
    pub branch: Option<String>,
    pub ahead: u32,
    pub behind: u32,
    pub staged_count: usize,
    pub unstaged_count: usize,
    pub untracked_count: usize,
    pub sync_state: String,
}

pub async fn stage(repo: &Path, paths: &[String]) -> Result<()> {
    let rels: Vec<String> = paths.iter().map(|p| to_repo_relative(repo, p)).collect();
    let mut args: Vec<&str> = vec!["add", "--"];
    args.extend(rels.iter().map(String::as_str));
    run_git(Some(repo), &args).await.map(|_| ())
}

pub async fn stage_all(repo: &Path) -> Result<()> {
    run_git(Some(repo), &["add", "-A"]).await.map(|_| ())
}

pub async fn unstage(repo: &Path, paths: &[String]) -> Result<()> {
    let rels: Vec<String> = paths.iter().map(|p| to_repo_relative(repo, p)).collect();
    let mut args: Vec<&str> = vec!["reset", "HEAD", "--"];
    args.extend(rels.iter().map(String::as_str));
    run_git(Some(repo), &args).await.map(|_| ())
}

pub async fn unstage_all(repo: &Path) -> Result<()> {
    run_git(Some(repo), &["reset", "HEAD"]).await.map(|_| ())
}

pub async fn discard(repo: &Path, paths: &[String]) -> Result<()> {
    let rels: Vec<String> = paths.iter().map(|p| to_repo_relative(repo, p)).collect();
    let mut args: Vec<&str> = vec!["checkout", "--", "."];
    let _ = args.pop();
    let mut full: Vec<&str> = vec!["checkout", "--"];
    full.extend(rels.iter().map(String::as_str));
    // checkout -- <paths> restores tracked modifications; for added-to-index files use reset+checkout.
    run_git(Some(repo), &full).await.map(|_| ())
}

pub async fn clean(repo: &Path, paths: &[String]) -> Result<()> {
    let rels: Vec<String> = paths.iter().map(|p| to_repo_relative(repo, p)).collect();
    let mut args: Vec<&str> = vec!["clean", "-f", "--"];
    args.extend(rels.iter().map(String::as_str));
    run_git(Some(repo), &args).await.map(|_| ())
}

pub async fn commit(repo: &Path, message: &str, amend: bool) -> Result<String> {
    let mut args: Vec<&str> = vec!["commit", "-m", message];
    if amend {
        args.push("--amend");
    }
    let out = run_git(Some(repo), &args).await?;
    let rev = run_git(Some(repo), &["rev-parse", "HEAD"])
        .await
        .map(|o| o.stdout.trim().to_string())
        .unwrap_or_default();
    let _ = out;
    Ok(rev)
}

pub async fn fetch(repo: &Path, remote: Option<&str>) -> Result<String> {
    let out = if let Some(r) = remote {
        run_git(Some(repo), &["fetch", "--prune", r]).await?
    } else {
        run_git(Some(repo), &["fetch", "--prune"]).await?
    };
    Ok(out.stderr.trim().to_string())
}

pub async fn pull(repo: &Path) -> Result<String> {
    let out = run_git(Some(repo), &["pull", "--ff-only"]).await?;
    Ok(format!("{}{}", out.stdout, out.stderr))
}

pub async fn push(repo: &Path, set_upstream: bool) -> Result<String> {
    let out = if set_upstream {
        let branch = current_branch_name(repo).await.unwrap_or_default();
        run_git(Some(repo), &["push", "-u", "origin", &branch]).await?
    } else {
        run_git(Some(repo), &["push"]).await?
    };
    Ok(format!("{}{}", out.stdout, out.stderr))
}

pub async fn sync(repo: &Path) -> Result<String> {
    // VS Code "Sync" = pull (rebase off, ff) then push.
    let pull_out = run_git(Some(repo), &["pull", "--ff-only"]).await?;
    let push_out = run_git(Some(repo), &["push"]).await?;
    Ok(format!("{}{}", pull_out.stdout, push_out.stdout))
}

async fn current_branch_name(repo: &Path) -> Result<String> {
    Ok(run_git(Some(repo), &["branch", "--show-current"])
        .await?
        .stdout
        .trim()
        .to_string())
}

pub async fn list_branches(repo: &Path) -> Result<Vec<Branch>> {
    let out = run_git(
        Some(repo),
        &[
            "for-each-ref",
            "--format=%(refname)%00%(refname:short)%00%(upstream:short)%00%(objectname:short)%00%(subject)",
            "refs/heads",
            "refs/remotes",
        ],
    )
    .await?;
    let current = current_branch_name(repo).await.unwrap_or_default();
    let mut branches = Vec::new();
    for line in out.stdout.lines() {
        let f: Vec<&str> = line.split('\0').collect();
        if f.len() < 5 {
            continue;
        }
        let full = f[0];
        let short = f[1].to_string();
        let upstream = if f[2].is_empty() {
            None
        } else {
            Some(f[2].to_string())
        };
        let (ahead, behind) = match &upstream {
            Some(up) => ahead_behind(repo, &short, up, full.starts_with("refs/remotes/")).await,
            None => (0, 0),
        };
        let remote = full.starts_with("refs/remotes/");
        branches.push(Branch {
            name: short.clone(),
            current: !remote && short == current,
            remote,
            upstream,
            ahead,
            behind,
            last_commit: f[3].to_string(),
            last_commit_subject: f[4].to_string(),
        });
    }
    branches.sort_by(|a, b| {
        b.current
            .cmp(&a.current)
            .then(a.remote.cmp(&b.remote))
            .then(a.name.cmp(&b.name))
    });
    Ok(branches)
}

async fn ahead_behind(repo: &Path, branch: &str, upstream: &str, is_remote: bool) -> (u32, u32) {
    if is_remote {
        return (0, 0);
    }
    let range = format!("{upstream}...{branch}");
    run_git(Some(repo), &["rev-list", "--count", "--left-right", &range])
        .await
        .ok()
        .and_then(|o| {
            let mut it = o.stdout.split_whitespace();
            let behind: u32 = it.next()?.parse().ok()?;
            let ahead: u32 = it.next()?.parse().ok()?;
            Some((ahead, behind))
        })
        .unwrap_or((0, 0))
}

pub async fn checkout(repo: &Path, name: &str) -> Result<()> {
    run_git(Some(repo), &["checkout", name]).await.map(|_| ())
}

pub async fn create_branch(repo: &Path, name: &str, checkout: bool) -> Result<()> {
    if checkout {
        run_git(Some(repo), &["checkout", "-b", name])
            .await
            .map(|_| ())
    } else {
        run_git(Some(repo), &["branch", name]).await.map(|_| ())
    }
}

pub async fn delete_branch(repo: &Path, name: &str, force: bool) -> Result<()> {
    let flag = if force { "-D" } else { "-d" };
    run_git(Some(repo), &["branch", flag, name])
        .await
        .map(|_| ())
}

pub async fn stash_list(repo: &Path) -> Result<Vec<StashEntry>> {
    let out = run_git(Some(repo), &["stash", "list", "--format=%gd%x00%gs%x00%D"]).await?;
    let mut entries = Vec::new();
    for line in out.stdout.lines() {
        let f: Vec<&str> = line.split('\0').collect();
        if f.len() < 2 {
            continue;
        }
        let index = f[0]
            .trim_start_matches("stash@{")
            .trim_end_matches('}')
            .parse()
            .unwrap_or(0);
        let message = f[1].to_string();
        let branch = message.split(':').next().unwrap_or("").trim().to_string();
        entries.push(StashEntry {
            index,
            message,
            branch,
        });
    }
    Ok(entries)
}

pub async fn stash_create(
    repo: &Path,
    message: Option<&str>,
    include_untracked: bool,
) -> Result<()> {
    let mut owned: Vec<String> = Vec::new();
    let mut args: Vec<&str> = vec!["stash", "push"];
    if include_untracked {
        args.push("-u");
    }
    if let Some(m) = message {
        if !m.trim().is_empty() {
            args.push("-m");
            owned.push(m.to_string());
            args.push(&owned[0]);
        }
    }
    run_git(Some(repo), &args).await.map(|_| ())
}

pub async fn stash_apply(repo: &Path, index: usize) -> Result<()> {
    let target = format!("stash@{{{index}}}");
    run_git(Some(repo), &["stash", "apply", &target])
        .await
        .map(|_| ())
}

pub async fn stash_pop(repo: &Path, index: usize) -> Result<()> {
    let target = format!("stash@{{{index}}}");
    run_git(Some(repo), &["stash", "pop", &target])
        .await
        .map(|_| ())
}

pub async fn stash_drop(repo: &Path, index: usize) -> Result<()> {
    let target = format!("stash@{{{index}}}");
    run_git(Some(repo), &["stash", "drop", &target])
        .await
        .map(|_| ())
}

const LOG_FORMAT: &str = "%H%x00%h%x00%an%x00%ae%x00%ad%x00%s%x00%b%x00%P%x00%D";

pub async fn get_log(repo: &Path, limit: usize, skip: usize) -> Result<Vec<CommitEntry>> {
    let limit_s = format!("--max-count={limit}");
    let skip_s = format!("--skip={skip}");
    let format_s = format!("--format={LOG_FORMAT}");
    let out = run_git(
        Some(repo),
        &["log", &limit_s, &skip_s, "--date=iso", &format_s],
    )
    .await?;
    let mut commits = Vec::new();
    for line in out.stdout.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let f: Vec<&str> = line.split('\0').collect();
        if f.len() < 9 {
            continue;
        }
        commits.push(CommitEntry {
            hash: f[0].to_string(),
            short_hash: f[1].to_string(),
            author: f[2].to_string(),
            email: f[3].to_string(),
            date: f[4].to_string(),
            subject: f[5].to_string(),
            body: f[6].trim().to_string(),
            parents: f[7].split_whitespace().map(String::from).collect(),
            refs: f[8]
                .split(", ")
                .filter(|s| !s.is_empty())
                .map(String::from)
                .collect(),
        });
    }
    Ok(commits)
}

pub async fn get_commit_files(repo: &Path, hash: &str) -> Result<Vec<String>> {
    // --root handles the root commit (no parents).
    let out = run_git(
        Some(repo),
        &[
            "diff-tree",
            "--no-commit-id",
            "--name-only",
            "-r",
            "--root",
            hash,
        ],
    )
    .await?;
    Ok(out
        .stdout
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(String::from)
        .collect())
}
