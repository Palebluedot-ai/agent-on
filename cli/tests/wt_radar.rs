use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_agent-on"))
}

fn git(cwd: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .arg("-C")
        .arg(cwd)
        .args(args)
        .output()
        .expect("run git");
    assert!(
        out.status.success(),
        "git {:?} in {} failed: {}",
        args,
        cwd.display(),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn init_repo(dir: &Path) {
    git(dir, &["init", "-b", "main"]);
    git(dir, &["config", "user.email", "test@example.com"]);
    git(dir, &["config", "user.name", "Test"]);
    fs::write(dir.join("shared.rs"), "fn main() {}\n").unwrap();
    fs::write(dir.join("中文说明.md"), "# 初始文档\n").unwrap();
    git(dir, &["add", "."]);
    git(dir, &["commit", "-m", "init"]);
}

#[test]
fn wt_radar_detects_cross_client_collisions_lag_and_prunes_only_reapable() {
    let tmp = TempDir::new().unwrap();
    let remote = tmp.path().join("origin.git");
    fs::create_dir_all(&remote).unwrap();
    git(&remote, &["init", "--bare", "-b", "main"]);

    let primary = tmp.path().join("repo");
    fs::create_dir_all(&primary).unwrap();
    init_repo(&primary);
    git(
        &primary,
        &["remote", "add", "origin", remote.to_str().unwrap()],
    );
    git(&primary, &["push", "-u", "origin", "main"]);

    // 1. Reapable worktree under .claude/worktrees/merged-one
    let claude_reapable = tmp.path().join(".claude/worktrees/merged-one");
    fs::create_dir_all(claude_reapable.parent().unwrap()).unwrap();
    git(
        &primary,
        &[
            "worktree",
            "add",
            "-b",
            "claude/merged-one",
            claude_reapable.to_str().unwrap(),
            "main",
        ],
    );

    // 2. Active dirty worktree under .codex/worktrees/active-a colliding on 中文说明.md
    let codex_dirty = tmp.path().join(".codex/worktrees/active-a");
    fs::create_dir_all(codex_dirty.parent().unwrap()).unwrap();
    git(
        &primary,
        &[
            "worktree",
            "add",
            "-b",
            "codex/active-a",
            codex_dirty.to_str().unwrap(),
            "main",
        ],
    );
    fs::write(codex_dirty.join("中文说明.md"), "# Codex 修改\n").unwrap();

    // 3. Active dirty worktree under .gemini/antigravity/worktrees/active-b also editing 中文说明.md
    let agy_dirty = tmp.path().join(".gemini/antigravity/worktrees/active-b");
    fs::create_dir_all(agy_dirty.parent().unwrap()).unwrap();
    git(
        &primary,
        &[
            "worktree",
            "add",
            "-b",
            "agy/active-b",
            agy_dirty.to_str().unwrap(),
            "main",
        ],
    );
    fs::write(agy_dirty.join("中文说明.md"), "# Antigravity 修改\n").unwrap();

    // 4. Ahead clean worktree with genuinely unmerged commit
    let ahead_wt = tmp.path().join(".claude/worktrees/unmerged-feat");
    git(
        &primary,
        &[
            "worktree",
            "add",
            "-b",
            "claude/unmerged-feat",
            ahead_wt.to_str().unwrap(),
            "main",
        ],
    );
    fs::write(ahead_wt.join("unmerged.txt"), "draft feature\n").unwrap();
    git(&ahead_wt, &["add", "."]);
    git(&ahead_wt, &["commit", "-m", "feat: unmerged draft"]);

    // 5. Patch-equivalent (cherry-picked / single-commit squashed) worktree
    let squashed_wt = tmp.path().join(".codex/worktrees/squashed-feat");
    git(
        &primary,
        &[
            "worktree",
            "add",
            "-b",
            "codex/squashed-feat",
            squashed_wt.to_str().unwrap(),
            "main",
        ],
    );
    fs::write(squashed_wt.join("squashed.txt"), "landed via cherry-pick\n").unwrap();
    git(&squashed_wt, &["add", "."]);
    git(&squashed_wt, &["commit", "-m", "feat: squashed work"]);
    let squashed_oid = git(&squashed_wt, &["rev-parse", "HEAD"]);

    // Advance main with that cherry-picked commit + 1 more commit so codex_dirty lags behind origin/main by 2!
    git(&primary, &["cherry-pick", "--no-commit", &squashed_oid]);
    git(&primary, &["commit", "-m", "squash-landed work on main"]);
    fs::write(primary.join("main_next.txt"), "advance main\n").unwrap();
    git(&primary, &["add", "."]);
    git(&primary, &["commit", "-m", "chore: advance main"]);
    git(&primary, &["push", "origin", "main"]);

    // Run `agent-on wt --json` from inside `codex_dirty` (behind origin/main by 2!)
    let out = Command::new(bin())
        .args(["wt", "--json", "--repo", codex_dirty.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(out.status.success(), "wt failed: {:?}", out);
    let report: Value = serde_json::from_slice(&out.stdout).unwrap();

    assert_eq!(report["base_ref"], "origin/main");
    assert_eq!(report["current_behind_base"], 2);
    assert_eq!(report["summary"]["total"], 6);
    assert_eq!(report["summary"]["reapable"], 1);
    assert_eq!(report["summary"]["likely_squashed"], 1);
    assert_eq!(report["summary"]["ahead_clean"], 1);
    assert_eq!(report["summary"]["active_dirty"], 2);

    // Check Unicode uncommitted collision caught across codex and antigravity
    let collisions = report["uncommitted_collisions"].as_array().unwrap();
    assert_eq!(collisions.len(), 1);
    assert_eq!(collisions[0]["file"], "中文说明.md");
    let owners_str = collisions[0]["worktrees"].to_string();
    assert!(owners_str.contains("codex"));
    assert!(owners_str.contains("antigravity"));

    // Now run `--prune` and verify ONLY `claude_reapable` is removed, and its branch ref still exists!
    let prune_out = Command::new(bin())
        .args([
            "wt",
            "--prune",
            "--json",
            "--repo",
            primary.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(prune_out.status.success());
    let prune_report: Value = serde_json::from_slice(&prune_out.stdout).unwrap();
    assert_eq!(
        prune_report["pruned_worktrees"].as_array().unwrap().len(),
        1
    );
    assert!(
        !claude_reapable.exists(),
        "reapable worktree should be removed"
    );
    assert!(codex_dirty.exists(), "dirty worktree must be preserved");
    assert!(agy_dirty.exists(), "dirty worktree must be preserved");
    assert!(ahead_wt.exists(), "unmerged worktree must be preserved");
    assert!(
        squashed_wt.exists(),
        "likely-squashed worktree must NOT be auto-deleted without review"
    );
    // Branch ref of pruned worktree must still exist in git!
    let branch_sha = git(
        &primary,
        &["rev-parse", "--verify", "refs/heads/claude/merged-one"],
    );
    assert!(!branch_sha.is_empty());
}

#[test]
fn wt_with_prs_and_legacy_shims_behave_safely() {
    let tmp = TempDir::new().unwrap();
    let repo = tmp.path().join("repo");
    fs::create_dir_all(&repo).unwrap();
    init_repo(&repo);

    // Edit shared.rs locally on a feature branch
    let feat_wt = tmp.path().join(".claude/worktrees/feat-local");
    fs::create_dir_all(feat_wt.parent().unwrap()).unwrap();
    git(
        &repo,
        &[
            "worktree",
            "add",
            "-b",
            "feat/local",
            feat_wt.to_str().unwrap(),
            "main",
        ],
    );
    fs::write(
        feat_wt.join("shared.rs"),
        "fn main() { println!(\"hi\"); }\n",
    )
    .unwrap();

    // Fake `gh` binary returning 2 PRs: one for `feat/local` itself (must be ignored) and one for `feat/other` touching `shared.rs`
    let bin_dir = tmp.path().join("fake-bin");
    fs::create_dir_all(&bin_dir).unwrap();
    let fake_gh = bin_dir.join("gh");
    fs::write(
        &fake_gh,
        r#"#!/bin/sh
cat <<'EOF'
[
  {
    "number": 10,
    "title": "Own PR",
    "headRefName": "feat/local",
    "url": "https://example.com/pr/10",
    "files": [{"path": "shared.rs"}]
  },
  {
    "number": 11,
    "title": "Competing PR",
    "headRefName": "feat/other",
    "url": "https://example.com/pr/11",
    "files": [{"path": "shared.rs"}, {"path": "other.rs"}]
  }
]
EOF
"#,
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&fake_gh, fs::Permissions::from_mode(0o755)).unwrap();
    }

    let orig_path = std::env::var_os("PATH").unwrap_or_default();
    let mut paths = vec![bin_dir];
    paths.extend(std::env::split_paths(&orig_path));
    let new_path = std::env::join_paths(paths).unwrap();

    let out = Command::new(bin())
        .env("PATH", &new_path)
        .args([
            "wt",
            "--with-prs",
            "--json",
            "--repo",
            repo.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let report: Value = serde_json::from_slice(&out.stdout).unwrap();
    let overlaps = report["pr_overlaps"].as_array().unwrap();
    assert_eq!(overlaps.len(), 1);
    assert_eq!(overlaps[0]["pr_number"], 11);
    assert_eq!(overlaps[0]["overlapping_files"][0], "shared.rs");

    // Verify hidden legacy compatibility shims exit 0 cleanly
    for legacy_args in [
        vec!["guard"],
        vec!["capture"],
        vec![
            "worktree",
            "hooks",
            "run",
            "--hook",
            "pre-commit",
            "--repo",
            repo.to_str().unwrap(),
        ],
        vec![
            "worktree",
            "gc",
            "--dry-run",
            "--json",
            "--repo",
            repo.to_str().unwrap(),
        ],
    ] {
        let st = Command::new(bin()).args(&legacy_args).output().unwrap();
        assert!(
            st.status.success(),
            "legacy shim {:?} failed: {}",
            legacy_args,
            String::from_utf8_lossy(&st.stderr)
        );
    }
}

#[test]
fn wt_handles_non_main_default_branch_nested_untracked_and_protects_caller_cwd() {
    let tmp = TempDir::new().unwrap();
    let primary = tmp.path().join("trunk-repo");
    fs::create_dir_all(&primary).unwrap();

    // Repo default branch is `trunk` (no `main` or `master` or `origin`), with `status.showUntrackedFiles=no`
    git(&primary, &["init", "-b", "trunk"]);
    git(&primary, &["config", "user.email", "test@example.com"]);
    git(&primary, &["config", "user.name", "Test"]);
    git(&primary, &["config", "status.showUntrackedFiles", "no"]);
    fs::write(primary.join("lib.rs"), "pub fn v1() {}\n").unwrap();
    git(&primary, &["add", "."]);
    git(&primary, &["commit", "-m", "init trunk"]);

    let wt_caller = tmp.path().join(".claude/worktrees/caller-cwd");
    let wt_untracked = tmp.path().join(".codex/worktrees/untracked-nested");
    let wt_lag = tmp.path().join(".gemini/antigravity/worktrees/lagging");
    for p in [&wt_caller, &wt_untracked, &wt_lag] {
        fs::create_dir_all(p.parent().unwrap()).unwrap();
    }
    git(
        &primary,
        &[
            "worktree",
            "add",
            "-b",
            "wt/caller",
            wt_caller.to_str().unwrap(),
            "trunk",
        ],
    );
    git(
        &primary,
        &[
            "worktree",
            "add",
            "-b",
            "wt/untracked",
            wt_untracked.to_str().unwrap(),
            "trunk",
        ],
    );
    git(
        &primary,
        &[
            "worktree",
            "add",
            "-b",
            "wt/lag",
            wt_lag.to_str().unwrap(),
            "trunk",
        ],
    );

    // Advance trunk by 1 commit so `wt_lag` is behind `trunk` by 1
    fs::write(primary.join("lib.rs"), "pub fn v2() {}\n").unwrap();
    git(&primary, &["add", "."]);
    git(&primary, &["commit", "-m", "advance trunk"]);

    // Fast-forward wt_caller and wt_untracked to trunk, then put a nested untracked file in wt_untracked
    git(&wt_caller, &["merge", "--ff-only", "trunk"]);
    git(&wt_untracked, &["merge", "--ff-only", "trunk"]);
    fs::create_dir_all(wt_untracked.join("nested/new_dir")).unwrap();
    fs::write(wt_untracked.join("nested/new_dir/draft.rs"), "// draft\n").unwrap();

    // Check lag from wt_lag: must resolve base_ref="trunk" and current_behind_base=1 (not 0!)
    let lag_out = Command::new(bin())
        .args(["wt", "--json", "--repo", wt_lag.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(lag_out.status.success());
    let lag_json: Value = serde_json::from_slice(&lag_out.stdout).unwrap();
    assert_eq!(lag_json["base_ref"], "trunk");
    assert_eq!(lag_json["current_behind_base"], 1);

    // Run `--prune --repo <primary>` while standing inside `wt_caller`!
    // Neither `wt_caller` (caller's CWD) nor `wt_untracked` (nested untracked file) may be deleted;
    // `wt_lag` is clean & ancestor of trunk so ONLY `wt_lag` is reapable!
    let prune_out = Command::new(bin())
        .current_dir(&wt_caller)
        .args([
            "wt",
            "--prune",
            "--json",
            "--repo",
            primary.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(prune_out.status.success());
    let prune_json: Value = serde_json::from_slice(&prune_out.stdout).unwrap();

    assert!(
        wt_caller.exists(),
        "caller CWD worktree must NOT be deleted"
    );
    assert!(
        wt_untracked.exists(),
        "worktree with nested untracked file must NOT be deleted even under showUntrackedFiles=no"
    );
    assert!(!wt_lag.exists(), "clean ancestor worktree should be pruned");

    let untracked_entry = prune_json["worktrees"]
        .as_array()
        .unwrap()
        .iter()
        .find(|w| w["branch"] == "wt/untracked")
        .unwrap();
    assert_eq!(untracked_entry["state"], "active-dirty");
    assert_eq!(untracked_entry["dirty_files"][0], "nested/new_dir/draft.rs");
}
