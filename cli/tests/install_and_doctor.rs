use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_agent-on"))
}

fn source_repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

fn git(cwd: &Path, args: &[&str]) {
    let out = Command::new("git")
        .arg("-C")
        .arg(cwd)
        .args(args)
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
}

#[test]
fn install_doctor_uninstall_and_clean_legacy_preserve_user_assets() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path().join("home");
    fs::create_dir_all(home.join(".claude")).unwrap();
    fs::create_dir_all(home.join(".codex")).unwrap();

    // Seed pre-existing user instructions that MUST never be lost
    fs::write(
        home.join(".claude/CLAUDE.md"),
        "# Personal Claude Preferences\n\nAlways answer concisely.\n",
    )
    .unwrap();
    fs::write(
        home.join(".codex/AGENTS.md"),
        "# Personal Codex Rules\n\nPrefer Rust and Python.\n",
    )
    .unwrap();

    // Seed legacy v0.x router symlink & legacy repo core.hooksPath
    fs::create_dir_all(home.join(".claude/skills")).unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(
        source_repo().join("legacy/v0.30-scaffold/skill"),
        home.join(".claude/skills/agent-on"),
    )
    .unwrap();

    let legacy_repo = tmp.path().join("project");
    fs::create_dir_all(&legacy_repo).unwrap();
    git(&legacy_repo, &["init", "-b", "main"]);
    fs::create_dir_all(legacy_repo.join(".git/agent-on/hooks")).unwrap();
    fs::write(
        legacy_repo.join(".git/agent-on/worktree-hooks.json"),
        r#"{"version":1,"previous_hooks_path":".githooks"}"#,
    )
    .unwrap();
    git(
        &legacy_repo,
        &[
            "config",
            "--local",
            "core.hooksPath",
            legacy_repo.join(".git/agent-on/hooks").to_str().unwrap(),
        ],
    );

    // 1. Doctor before install detects 0/4 skills and both legacy residues
    let doc_before = Command::new(bin())
        .args([
            "doctor",
            "--json",
            "--home",
            home.to_str().unwrap(),
            "--source",
            source_repo().to_str().unwrap(),
            "--repo",
            legacy_repo.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(doc_before.status.success());
    let doc_json: Value = serde_json::from_slice(&doc_before.stdout).unwrap();
    assert_eq!(doc_json["binary_matches_source"], true);
    assert_eq!(doc_json["legacy_residues"].as_array().unwrap().len(), 2);

    // 2. Install all 3 hosts + rules + clean-legacy
    let inst = Command::new(bin())
        .args([
            "install",
            "--json",
            "--home",
            home.to_str().unwrap(),
            "--source",
            source_repo().to_str().unwrap(),
            "--with-rules",
            "--clean-legacy",
            "--repo",
            legacy_repo.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        inst.status.success(),
        "install failed: {}",
        String::from_utf8_lossy(&inst.stderr)
    );

    // Legacy router symlink removed; legacy core.hooksPath restored to `.githooks`!
    assert!(fs::symlink_metadata(home.join(".claude/skills/agent-on")).is_err());
    let restored_hp = Command::new("git")
        .arg("-C")
        .arg(&legacy_repo)
        .args(["config", "--local", "--get", "core.hooksPath"])
        .output()
        .unwrap();
    assert_eq!(
        String::from_utf8_lossy(&restored_hp.stdout).trim(),
        ".githooks"
    );

    // 3. Doctor after install reports 4/4 skills on all 3 hosts, rules_active=true, 0 residues!
    let doc_after = Command::new(bin())
        .args([
            "doctor",
            "--json",
            "--home",
            home.to_str().unwrap(),
            "--source",
            source_repo().to_str().unwrap(),
            "--repo",
            legacy_repo.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    let after_json: Value = serde_json::from_slice(&doc_after.stdout).unwrap();
    assert_eq!(after_json["legacy_residues"].as_array().unwrap().len(), 0);
    for host in after_json["hosts"].as_array().unwrap() {
        assert_eq!(host["linked_skills"].as_array().unwrap().len(), 4);
        assert_eq!(host["rules_active"], true);
    }

    // 4. Re-running install is idempotent (`unchanged`) and preserves user header
    let claude_md = fs::read_to_string(home.join(".claude/CLAUDE.md")).unwrap();
    assert!(claude_md.starts_with("# Personal Claude Preferences\n\nAlways answer concisely."));
    assert!(claude_md.contains("<!-- agent-on:core-guardrails:start -->"));

    // 5. Uninstall strips ONLY Agent-On symlinks & guardrail block, preserving personal CLAUDE.md!
    let uninst = Command::new(bin())
        .args([
            "install",
            "--uninstall",
            "--json",
            "--home",
            home.to_str().unwrap(),
            "--source",
            source_repo().to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(uninst.status.success());
    let claude_md_after = fs::read_to_string(home.join(".claude/CLAUDE.md")).unwrap();
    assert_eq!(
        claude_md_after,
        "# Personal Claude Preferences\n\nAlways answer concisely.\n"
    );
    assert!(!home.join(".claude/skills/prove-it").exists());
    assert!(!home.join(".gemini/config/plugins/agent-on").exists());
}

#[test]
fn install_refuses_to_overwrite_unmanaged_real_directory() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path().join("home");
    let custom_skill = home.join(".claude/skills/prove-it");
    fs::create_dir_all(&custom_skill).unwrap();
    fs::write(custom_skill.join("SKILL.md"), "my custom prove-it\n").unwrap();

    let out = Command::new(bin())
        .args([
            "install",
            "--json",
            "--home",
            home.to_str().unwrap(),
            "--source",
            source_repo().to_str().unwrap(),
            "--targets",
            "claude",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let report: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(report["conflicts"].as_array().unwrap().len(), 1);
    assert_eq!(
        fs::read_to_string(custom_skill.join("SKILL.md")).unwrap(),
        "my custom prove-it\n"
    );
}

#[test]
fn clean_legacy_strips_host_hooks_preserving_siblings_and_worktree_configs() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path().join("home");
    fs::create_dir_all(home.join(".codex")).unwrap();
    fs::create_dir_all(home.join(".claude")).unwrap();

    // Seed `~/.codex/hooks.json` with legacy `kit/guard/agent-on-git-guard` AND user's `auto-sync.sh`
    fs::write(
        home.join(".codex/hooks.json"),
        r#"{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "bash \"$HOME/Projects/Agent-On/kit/guard/agent-on-git-guard\""
          }
        ]
      }
    ],
    "Stop": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "bash ~/.codex/auto-sync.sh"
          }
        ]
      }
    ]
  }
}
"#,
    )
    .unwrap();

    // Seed `~/.claude/settings.json` with user permissions AND legacy `agent-on guard`
    fs::write(
        home.join(".claude/settings.json"),
        r#"{
  "permissions": {
    "allow": ["Read"]
  },
  "hooks": {
    "SessionStart": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "agent-on capture"
          }
        ]
      }
    ]
  }
}
"#,
    )
    .unwrap();

    // Seed repo with `extensions.worktreeConfig=true` and `--worktree` core.hooksPath on linked worktree
    let primary = tmp.path().join("repo");
    fs::create_dir_all(&primary).unwrap();
    git(&primary, &["init", "-b", "main"]);
    git(&primary, &["config", "user.email", "t@e.com"]);
    git(&primary, &["config", "user.name", "T"]);
    fs::write(primary.join("f.txt"), "1\n").unwrap();
    git(&primary, &["add", "."]);
    git(&primary, &["commit", "-m", "init"]);
    git(&primary, &["config", "extensions.worktreeConfig", "true"]);

    let linked = tmp.path().join("linked-wt");
    git(
        &primary,
        &[
            "worktree",
            "add",
            "-b",
            "feat/wt",
            linked.to_str().unwrap(),
            "main",
        ],
    );
    git(
        &linked,
        &[
            "config",
            "--worktree",
            "core.hooksPath",
            primary.join(".git/agent-on/hooks").to_str().unwrap(),
        ],
    );

    let out = Command::new(bin())
        .args([
            "install",
            "--clean-legacy",
            "--json",
            "--home",
            home.to_str().unwrap(),
            "--source",
            source_repo().to_str().unwrap(),
            "--repo",
            primary.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "clean-legacy failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let codex_after: Value =
        serde_json::from_str(&fs::read_to_string(home.join(".codex/hooks.json")).unwrap()).unwrap();
    assert!(codex_after["hooks"].get("PreToolUse").is_none());
    assert_eq!(
        codex_after["hooks"]["Stop"][0]["hooks"][0]["command"],
        "bash ~/.codex/auto-sync.sh"
    );

    let claude_after: Value =
        serde_json::from_str(&fs::read_to_string(home.join(".claude/settings.json")).unwrap())
            .unwrap();
    assert_eq!(claude_after["permissions"]["allow"][0], "Read");
    assert!(claude_after.get("hooks").is_none());

    let wt_hp = Command::new("git")
        .arg("-C")
        .arg(&linked)
        .args(["config", "--worktree", "--get", "core.hooksPath"])
        .output()
        .unwrap();
    assert!(
        !wt_hp.status.success(),
        "worktree core.hooksPath must be unset"
    );
}

#[test]
#[cfg(unix)]
fn install_protects_dangling_symlinks_non_utf8_rules_and_dotfiles_symlinks() {
    use std::os::unix::fs::symlink;

    let tmp = TempDir::new().unwrap();
    let home = tmp.path().join("home");
    fs::create_dir_all(home.join(".claude/skills")).unwrap();
    fs::create_dir_all(home.join(".agents/skills")).unwrap();
    fs::create_dir_all(home.join(".codex")).unwrap();

    // 1. Dangling user symlink at `.claude/skills/prove-it` -> MUST NOT be overwritten
    symlink(
        tmp.path().join("unmounted-volume/my-prove-it"),
        home.join(".claude/skills/prove-it"),
    )
    .unwrap();

    // 2. Unrelated user directory whose parent path happens to contain `agent-on` -> MUST NOT be claimed/deleted
    let unrelated = tmp.path().join("agent-on-experiments/my-custom-skill");
    fs::create_dir_all(&unrelated).unwrap();
    symlink(&unrelated, home.join(".agents/skills/thin-review")).unwrap();

    // 3. Non-UTF-8 bytes in `.claude/CLAUDE.md` -> MUST NOT be wiped to empty!
    let raw_bytes: &[u8] = &[0xff, 0xfe, b'u', b's', b'e', b'r'];
    fs::write(home.join(".claude/CLAUDE.md"), raw_bytes).unwrap();

    // 4. Valid dotfiles symlink at `.codex/AGENTS.md` -> symlink itself MUST stay a symlink!
    let dotfiles = tmp.path().join("dotfiles");
    fs::create_dir_all(&dotfiles).unwrap();
    let dot_agents = dotfiles.join("AGENTS.md");
    fs::write(&dot_agents, "# Dotfiles Rules\n").unwrap();
    symlink(&dot_agents, home.join(".codex/AGENTS.md")).unwrap();

    let out = Command::new(bin())
        .args([
            "install",
            "--with-rules",
            "--json",
            "--home",
            home.to_str().unwrap(),
            "--source",
            source_repo().to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1), "conflicts should return exit 1");

    // Verify dangling symlink target was preserved untouched
    assert_eq!(
        fs::read_link(home.join(".claude/skills/prove-it")).unwrap(),
        tmp.path().join("unmounted-volume/my-prove-it")
    );
    // Verify unrelated `agent-on-experiments` symlink was preserved untouched
    assert_eq!(
        fs::read_link(home.join(".agents/skills/thin-review")).unwrap(),
        unrelated
    );
    // Verify non-UTF-8 `CLAUDE.md` bytes were preserved 100% untouched!
    assert_eq!(fs::read(home.join(".claude/CLAUDE.md")).unwrap(), raw_bytes);
    // Verify `.codex/AGENTS.md` is STILL a symlink AND dotfiles file got the rules block!
    assert!(fs::symlink_metadata(home.join(".codex/AGENTS.md"))
        .unwrap()
        .file_type()
        .is_symlink());
    let dot_content = fs::read_to_string(&dot_agents).unwrap();
    assert!(dot_content.starts_with("# Dotfiles Rules\n"));
    assert!(dot_content.contains("<!-- agent-on:core-guardrails:start -->"));

    // Also verify `--uninstall` does NOT remove `agent-on-experiments/my-custom-skill` symlink!
    let uninst = Command::new(bin())
        .args([
            "install",
            "--uninstall",
            "--json",
            "--home",
            home.to_str().unwrap(),
            "--source",
            source_repo().to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(uninst.status.success());
    assert_eq!(
        fs::read_link(home.join(".agents/skills/thin-review")).unwrap(),
        unrelated
    );
}
