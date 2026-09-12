use mikrogit_lib::git::{diff, files, ops, repo_root, run_git, status};
use std::path::PathBuf;
use std::process::Command as StdCommand;

fn git(args: &[&str], dir: &std::path::Path) {
    let out = StdCommand::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .output()
        .expect("spawn git");
    assert!(
        out.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn git_may_fail(args: &[&str], dir: &std::path::Path) -> bool {
    StdCommand::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn fixture() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("mikrogit-it-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    git(&["init", "-b", "main"], &dir);
    git(&["config", "user.email", "t@t.t"], &dir);
    git(&["config", "user.name", "T"], &dir);
    git(&["config", "core.autocrlf", "false"], &dir);
    std::fs::write(dir.join("a.txt"), "hello\n").unwrap();
    git(&["add", "."], &dir);
    git(&["commit", "-m", "initial"], &dir);
    std::fs::write(dir.join("a.txt"), "hello\nworld\n").unwrap();
    std::fs::write(dir.join("b.txt"), "new\n").unwrap();
    dir
}

#[tokio::test]
async fn end_to_end_status_diff_stage_commit() {
    let dir = fixture();

    let root = repo_root(&dir.join("a.txt")).expect("repo_root");
    assert_eq!(root, dir);

    let v = run_git(Some(&dir), &["--version"])
        .await
        .expect("git version");
    assert!(v.stdout.contains("git version"));

    let s = status::get_status(&dir).await.expect("status");
    assert_eq!(s.branch.name.as_deref(), Some("main"));
    assert_eq!(s.unstaged.len(), 1);
    assert_eq!(s.unstaged[0].path, "a.txt");
    assert_eq!(s.untracked.len(), 1);

    let d = diff::get_diff(&dir, "a.txt", false).await.expect("diff");
    assert!(!d.binary);
    assert_eq!(d.added, 1);
    assert!(!d.hunks.is_empty());

    let u = diff::get_diff(&dir, "b.txt", false)
        .await
        .expect("untracked diff");
    assert_eq!(u.added, 1);
    assert_eq!(u.hunks.len(), 1);

    ops::stage(&dir, &["a.txt".to_string(), "b.txt".to_string()])
        .await
        .expect("stage");
    let s2 = status::get_status(&dir).await.expect("status2");
    assert_eq!(s2.staged.len(), 2);
    assert!(s2.unstaged.is_empty());

    let hash = ops::commit(&dir, "second", false).await.expect("commit");
    assert_eq!(hash.len(), 40);

    let log = ops::get_log(&dir, 10, 0).await.expect("log");
    assert_eq!(log.len(), 2);
    assert_eq!(log[0].subject, "second");

    let files = ops::get_commit_files(&dir, &hash).await.expect("files");
    assert!(files.contains(&"a.txt".to_string()));

    let cd = diff::get_commit_diff(&dir, &hash, "a.txt")
        .await
        .expect("commit diff");
    assert!(!cd.hunks.is_empty());

    let branches = ops::list_branches(&dir).await.expect("branches");
    assert!(branches.iter().any(|b| b.name == "main" && b.current));

    ops::create_branch(&dir, "feature", true)
        .await
        .expect("create");
    ops::checkout(&dir, "main").await.expect("checkout");
    ops::delete_branch(&dir, "feature", true)
        .await
        .expect("delete");

    std::fs::write(dir.join("a.txt"), "hello\nworld\nwip-change\n").unwrap();
    ops::stash_create(&dir, Some("wip"), false)
        .await
        .expect("stash");
    let stashes = ops::stash_list(&dir).await.expect("stash list");
    assert_eq!(stashes.len(), 1);
    ops::stash_pop(&dir, 0).await.expect("pop");

    let _ = std::fs::remove_dir_all(&dir);
}

fn conflict_fixture(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("mikrogit-conf-{label}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    git(&["init", "-b", "main"], &dir);
    git(&["config", "user.email", "t@t.t"], &dir);
    git(&["config", "user.name", "T"], &dir);
    git(&["config", "core.autocrlf", "false"], &dir);
    std::fs::write(dir.join("file.txt"), "line1\nshared\n").unwrap();
    git(&["add", "."], &dir);
    git(&["commit", "-m", "base"], &dir);
    git(&["checkout", "-b", "feat"], &dir);
    std::fs::write(dir.join("file.txt"), "line1\nfeat change\n").unwrap();
    git(&["commit", "-am", "feat"], &dir);
    git(&["checkout", "main"], &dir);
    std::fs::write(dir.join("file.txt"), "line1\nmain change\n").unwrap();
    git(&["commit", "-am", "main"], &dir);
    assert!(
        !git_may_fail(&["merge", "feat"], &dir),
        "merge should conflict"
    );
    dir
}

#[tokio::test]
async fn merge_conflict_resolution_end_to_end() {
    let dir = conflict_fixture("merge");

    // 1. Status reports the unmerged file.
    let s = status::get_status(&dir).await.expect("status");
    assert_eq!(s.unmerged.len(), 1);
    assert_eq!(s.unmerged[0].path, "file.txt");

    // 2. The merge editor parses the conflict regions.
    let raw = std::fs::read_to_string(dir.join("file.txt")).unwrap();
    let cf = files::parse_conflicts(&raw, "file.txt");
    assert!(cf.has_conflicts);
    assert_eq!(cf.regions.len(), 1);
    assert_eq!(cf.regions[0].current_header, "HEAD");
    assert_eq!(cf.regions[0].current_lines, vec!["main change"]);
    assert_eq!(cf.regions[0].incoming_header, "feat");
    assert_eq!(cf.regions[0].incoming_lines, vec!["feat change"]);
    let cf = files::read_conflicts(&dir, "file.txt").await.expect("read_conflicts");
    assert_eq!(cf.original_content, raw);

    // 3. User accepts incoming -> resolve, save, stage, commit the merge.
    let resolved =
        files::resolve_conflicts(&cf.original_content, &["incoming".to_string()]).unwrap();
    assert_eq!(resolved, "line1\nfeat change\n");
    files::save_file(&dir, "file.txt", &resolved).await.expect("save");
    ops::stage(&dir, &["file.txt".to_string()]).await.expect("stage");
    let s2 = status::get_status(&dir).await.expect("status2");
    assert!(s2.unmerged.is_empty());
    assert_eq!(s2.staged.len(), 1);
    ops::commit(&dir, "merge feat", false).await.expect("commit");

    // 4. Repo is clean again; merge commit has two parents.
    let s3 = status::get_status(&dir).await.expect("status3");
    assert!(s3.unmerged.is_empty());
    assert!(s3.staged.is_empty() && s3.unstaged.is_empty());
    let log = ops::get_log(&dir, 5, 0).await.expect("log");
    assert_eq!(log[0].subject, "merge feat");
    assert_eq!(log[0].parents.len(), 2);
    let content = files::read_file(&dir, "file.txt", 1024).await.expect("read");
    assert_eq!(content, "line1\nfeat change\n");

    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn explorer_file_ops_end_to_end() {
    let dir = conflict_fixture("explorer");

    // Root listing shows tracked files; read_conflicts sees the markers.
    let root = files::list_files(&dir, None).await.expect("list");
    assert!(root.iter().any(|e| e.path == "file.txt" && e.kind == "file"));
    let cf = files::read_conflicts(&dir, "file.txt").await.expect("conflicts");
    assert!(cf.has_conflicts);

    // Create / rename / read / save / delete through the explorer commands.
    files::create_file(&dir, "explorer-new.txt").await.expect("create");
    files::create_dir(&dir, "explorer-dir/nested").await.expect("mkdir");
    files::rename_entry(&dir, "explorer-new.txt", "explorer-dir/nested/renamed.txt")
        .await
        .expect("rename");
    let nested = files::list_files(&dir, Some("explorer-dir/nested")).await.expect("sub");
    assert_eq!(nested.len(), 1);
    assert_eq!(nested[0].path, "explorer-dir/nested/renamed.txt");

    files::save_file(&dir, "explorer-dir/nested/renamed.txt", "hello\n")
        .await
        .expect("save");
    let text = files::read_file(&dir, "explorer-dir/nested/renamed.txt", 1024)
        .await
        .expect("read");
    assert_eq!(text, "hello\n");

    // Editing an unmerged file via the editor, then staging from the tree.
    let resolved =
        files::resolve_conflicts(&cf.original_content, &["both".to_string()]).unwrap();
    files::save_file(&dir, "file.txt", &resolved).await.expect("save2");
    ops::stage(&dir, &["file.txt".to_string()]).await.expect("stage2");
    let s = status::get_status(&dir).await.expect("status");
    assert!(s.unmerged.is_empty());
    assert_eq!(s.staged.len(), 1);

    files::delete_entry(&dir, "explorer-dir").await.expect("delete");
    assert!(files::list_files(&dir, Some("explorer-dir")).await.is_err());

    let _ = std::fs::remove_dir_all(&dir);
}
