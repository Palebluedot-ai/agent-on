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
        "git {:?} failed: {}",
        args,
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

#[test]
fn merge_check_computes_sync_free_pairwise_conflicts_and_virtual_waves() {
    let tmp = TempDir::new().unwrap();
    let repo = tmp.path().join("repo");
    fs::create_dir_all(&repo).unwrap();

    git(&repo, &["init", "-b", "main"]);
    git(&repo, &["config", "user.email", "test@example.com"]);
    git(&repo, &["config", "user.name", "Test"]);
    fs::write(
        repo.join("common.txt"),
        "line1\nline2\nline3\nline4\nline5\nline6\n",
    )
    .unwrap();
    fs::write(repo.join("base_only.txt"), "v1\n").unwrap();
    git(&repo, &["add", "."]);
    git(&repo, &["commit", "-m", "initial"]);
    let base0 = git(&repo, &["rev-parse", "HEAD"]);

    // Track 1: feat/sync-free touches a brand-new file `alpha.txt` from base0
    git(&repo, &["checkout", "-b", "feat/sync-free", &base0]);
    fs::write(repo.join("alpha.txt"), "alpha feature\n").unwrap();
    git(&repo, &["add", "."]);
    git(&repo, &["commit", "-m", "feat: alpha"]);

    // Track 2 & Track 3: BOTH branch from base0, BOTH merge cleanly into main individually,
    // BUT they conflict with EACH OTHER on `shared_new.rs`!
    git(&repo, &["checkout", "-b", "feat/twin-1", &base0]);
    fs::write(
        repo.join("shared_new.rs"),
        "pub const MODE: &str = \"one\";\n",
    )
    .unwrap();
    git(&repo, &["add", "."]);
    git(&repo, &["commit", "-m", "feat: twin 1"]);

    git(&repo, &["checkout", "-b", "feat/twin-2", &base0]);
    fs::write(
        repo.join("shared_new.rs"),
        "pub const MODE: &str = \"two\";\n",
    )
    .unwrap();
    git(&repo, &["add", "."]);
    git(&repo, &["commit", "-m", "feat: twin 2"]);

    // Track 4: feat/base-conflict modifies `base_only.txt` conflicting with main's advancement
    git(&repo, &["checkout", "-b", "feat/base-conflict", &base0]);
    fs::write(repo.join("base_only.txt"), "conflicting branch edit\n").unwrap();
    git(&repo, &["add", "."]);
    git(&repo, &["commit", "-m", "feat: conflict with main"]);

    // Now advance `main` on `base_only.txt` and line 1 of `common.txt`
    git(&repo, &["checkout", "main"]);
    fs::write(repo.join("base_only.txt"), "v2 on main\n").unwrap();
    fs::write(
        repo.join("common.txt"),
        "MAIN_LINE1\nline2\nline3\nline4\nline5\nline6\n",
    )
    .unwrap();
    git(&repo, &["add", "."]);
    git(&repo, &["commit", "-m", "main advances"]);

    // Track 5: feat/clean-overlap branched from base0 and edits line 6 of `common.txt`
    git(&repo, &["checkout", "-b", "feat/clean-overlap", &base0]);
    fs::write(
        repo.join("common.txt"),
        "line1\nline2\nline3\nline4\nline5\nBRANCH_LINE6\n",
    )
    .unwrap();
    git(&repo, &["add", "."]);
    git(&repo, &["commit", "-m", "feat: bottom of common.txt"]);
    git(&repo, &["checkout", "main"]);

    let out = Command::new(bin())
        .args([
            "merge-check",
            "--repo",
            repo.to_str().unwrap(),
            "--base",
            "main",
            "--branches",
            "feat/sync-free,feat/twin-1,feat/twin-2,feat/base-conflict,feat/clean-overlap",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "merge-check failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let report: Value = serde_json::from_slice(&out.stdout).unwrap();

    let by_id = |id: &str| -> Value {
        report["candidates"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["id"] == id)
            .unwrap()
            .clone()
    };

    assert_eq!(by_id("feat/sync-free")["base_sync"], "sync-free");
    assert_eq!(by_id("feat/twin-1")["base_sync"], "sync-free");
    assert_eq!(by_id("feat/twin-2")["base_sync"], "sync-free");
    assert_eq!(
        by_id("feat/clean-overlap")["base_sync"],
        "clean-with-overlap"
    );
    assert_eq!(by_id("feat/base-conflict")["base_sync"], "base-conflict");

    // Hotspots must include `shared_new.rs`
    let hotspots = report["hotspots"].as_array().unwrap();
    assert_eq!(hotspots.len(), 1);
    assert_eq!(hotspots[0]["file"], "shared_new.rs");

    // Pairwise conflict between twin-1 and twin-2 caught in memory without touching working tree!
    let pairs = report["pair_conflicts"].as_array().unwrap();
    assert_eq!(pairs.len(), 1);
    assert_eq!(pairs[0]["track_a"], "feat/twin-1");
    assert_eq!(pairs[0]["track_b"], "feat/twin-2");
    assert_eq!(pairs[0]["merge_tree_conflicts"][0], "shared_new.rs");

    // Waves: Wave 1 lands `feat/sync-free`, `feat/twin-1`, and `feat/clean-overlap` together;
    // `feat/twin-2` is deferred to Wave 2; `feat/base-conflict` is in `blocked_tracks`!
    let waves = report["waves"].as_array().unwrap();
    assert_eq!(waves.len(), 2);
    let wave1: Vec<&str> = waves[0]["tracks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert_eq!(
        wave1,
        vec!["feat/sync-free", "feat/twin-1", "feat/clean-overlap"]
    );
    let wave2: Vec<&str> = waves[1]["tracks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert_eq!(wave2, vec!["feat/twin-2"]);
    assert_eq!(report["blocked_tracks"][0], "feat/base-conflict");

    // Verify repo working tree is 100% clean after in-memory rehearsal
    let status = git(&repo, &["status", "--porcelain"]);
    assert!(status.is_empty());
}

#[test]
fn merge_check_rejects_missing_pr_handles_orphan_branch_and_quoted_filenames() {
    let tmp = TempDir::new().unwrap();
    let repo = tmp.path().join("repo");
    fs::create_dir_all(&repo).unwrap();

    git(&repo, &["init", "-b", "main"]);
    git(&repo, &["config", "user.email", "test@example.com"]);
    git(&repo, &["config", "user.name", "Test"]);
    fs::write(repo.join("doc \"quoted\".md"), "v1\n").unwrap();
    git(&repo, &["add", "."]);
    git(&repo, &["commit", "-m", "init"]);
    let base0 = git(&repo, &["rev-parse", "HEAD"]);

    // Conflicting branch on filename with quotes
    git(&repo, &["checkout", "-b", "feat/quoted", &base0]);
    fs::write(repo.join("doc \"quoted\".md"), "branch edit\n").unwrap();
    git(&repo, &["add", "."]);
    git(&repo, &["commit", "-m", "feat quoted"]);
    let quoted_oid = git(&repo, &["rev-parse", "HEAD"]);

    // Orphan branch (no common ancestor with main)
    git(&repo, &["checkout", "--orphan", "gh-pages"]);
    git(&repo, &["rm", "-rf", "."]);
    fs::write(repo.join("index.html"), "<h1>docs</h1>\n").unwrap();
    git(&repo, &["add", "."]);
    git(&repo, &["commit", "-m", "orphan pages"]);
    let orphan_oid = git(&repo, &["rev-parse", "HEAD"]);

    // Advance main on `doc "quoted".md`
    git(&repo, &["checkout", "main"]);
    fs::write(repo.join("doc \"quoted\".md"), "main edit\n").unwrap();
    git(&repo, &["add", "."]);
    git(&repo, &["commit", "-m", "main quoted"]);

    // Fake `gh` returning PR #1 (quoted conflict) and PR #2 (orphan branch)
    let bin_dir = tmp.path().join("fake-bin");
    fs::create_dir_all(&bin_dir).unwrap();
    let fake_gh = bin_dir.join("gh");
    fs::write(
        &fake_gh,
        format!(
            r#"#!/bin/sh
cat <<'EOF'
[
  {{
    "number": 1,
    "title": "Quoted file PR",
    "headRefName": "feat/quoted",
    "headRefOid": "{quoted_oid}",
    "isDraft": false,
    "url": "https://example.com/pr/1",
    "files": [{{"path": "doc \"quoted\".md"}}]
  }},
  {{
    "number": 2,
    "title": "Orphan pages PR",
    "headRefName": "gh-pages",
    "headRefOid": "{orphan_oid}",
    "isDraft": true,
    "url": "https://example.com/pr/2",
    "files": [{{"path": "index.html"}}]
  }}
]
EOF
"#
        ),
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

    // 1. `--prs` handles orphan branch gracefully without crashing and preserves unescaped quoted filename
    let ok_out = Command::new(bin())
        .env("PATH", &new_path)
        .args([
            "merge-check",
            "--prs",
            "--json",
            "--repo",
            repo.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        ok_out.status.success(),
        "merge-check --prs failed: {}",
        String::from_utf8_lossy(&ok_out.stderr)
    );
    let report: Value = serde_json::from_slice(&ok_out.stdout).unwrap();
    assert_eq!(
        report["candidates"][0]["base_conflict_files"][0],
        "doc \"quoted\".md"
    );
    assert_eq!(report["candidates"][1]["base_sync"], "base-conflict");
    assert_eq!(
        report["candidates"][1]["base_conflict_files"][0],
        "<unrelated-histories>"
    );

    // 2. Requesting a non-existent/closed PR number `--pr 1,999` MUST exit non-zero (never false-green!)
    let bad_out = Command::new(bin())
        .env("PATH", &new_path)
        .args([
            "merge-check",
            "--pr",
            "1,999",
            "--repo",
            repo.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(bad_out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&bad_out.stderr).contains("#999"));
}
