use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::TempDir;

fn run(cwd: &Path, program: &str, args: &[&str]) -> Output {
    Command::new(program)
        .current_dir(cwd)
        .args(args)
        .output()
        .unwrap_or_else(|error| panic!("cannot run {program}: {error}"))
}

fn must_run(cwd: &Path, program: &str, args: &[&str]) -> Output {
    let output = run(cwd, program, args);
    assert!(
        output.status.success(),
        "{} {} failed\nstdout:\n{}\nstderr:\n{}",
        program,
        args.join(" "),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn combined(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

struct Fixture {
    _tmp: TempDir,
    root: PathBuf,
    remote: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path().join("repo");
        let remote = tmp.path().join("remote.git");
        fs::create_dir_all(root.join("app")).unwrap();
        must_run(&root, "git", &["init", "-b", "main"]);
        must_run(&root, "git", &["config", "user.email", "test@example.com"]);
        must_run(&root, "git", &["config", "user.name", "Test"]);
        fs::write(root.join("README.md"), "root\n").unwrap();
        fs::write(root.join("app/base.txt"), "base\n").unwrap();
        must_run(&root, "git", &["add", "."]);
        must_run(&root, "git", &["commit", "-m", "init"]);
        must_run(&root, "git", &["init", "--bare", remote.to_str().unwrap()]);
        must_run(
            &root,
            "git",
            &["remote", "add", "origin", remote.to_str().unwrap()],
        );
        must_run(&root, "git", &["push", "-u", "origin", "main"]);
        Self {
            _tmp: tmp,
            root,
            remote,
        }
    }

    fn agent_on(&self, cwd: &Path, args: &[&str]) -> Output {
        run(cwd, env!("CARGO_BIN_EXE_agent-on"), args)
    }

    fn must_agent_on(&self, cwd: &Path, args: &[&str]) -> Output {
        must_run(cwd, env!("CARGO_BIN_EXE_agent-on"), args)
    }

    fn install(&self) {
        self.must_agent_on(
            &self.root,
            &[
                "worktree",
                "hooks",
                "install",
                "--repo",
                self.root.to_str().unwrap(),
            ],
        );
    }
}

#[cfg(unix)]
struct FakeScheduleEnv {
    home: PathBuf,
    state: PathBuf,
    config: PathBuf,
    bin: PathBuf,
    path: String,
}

#[cfg(unix)]
impl FakeScheduleEnv {
    fn new(base: &Path) -> Self {
        use std::os::unix::fs::PermissionsExt;

        let home = base.join("home");
        let state = base.join("state");
        let config = base.join("config");
        let bin = base.join("fake-bin");
        for directory in [&home, &state, &config, &bin] {
            fs::create_dir_all(directory).unwrap();
        }
        for name in ["launchctl", "systemctl"] {
            let path = bin.join(name);
            fs::write(&path, "#!/bin/sh\nexit 0\n").unwrap();
            let mut permissions = fs::metadata(&path).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(path, permissions).unwrap();
        }
        let inherited = std::env::var("PATH").unwrap_or_default();
        let path = format!("{}:{inherited}", bin.display());
        Self {
            home,
            state,
            config,
            bin,
            path,
        }
    }

    fn agent_on(&self, cwd: &Path, args: &[&str]) -> Output {
        self.agent_on_with(env!("CARGO_BIN_EXE_agent-on"), cwd, args, &self.path)
    }

    fn agent_on_with(&self, program: &str, cwd: &Path, args: &[&str], path_env: &str) -> Output {
        Command::new(program)
            .current_dir(cwd)
            .args(args)
            .env("HOME", &self.home)
            .env("XDG_STATE_HOME", &self.state)
            .env("XDG_CONFIG_HOME", &self.config)
            .env("PATH", path_env)
            .output()
            .unwrap()
    }

    fn scheduler_files(&self) -> Vec<PathBuf> {
        fn collect(root: &Path, out: &mut Vec<PathBuf>) {
            let Ok(entries) = fs::read_dir(root) else {
                return;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    collect(&path, out);
                } else if path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| {
                        name.starts_with("io.agent-on.worktree-gc.")
                            || name.starts_with("agent-on-worktree-gc-")
                    })
                {
                    out.push(path);
                }
            }
        }
        let mut files = Vec::new();
        collect(&self.home, &mut files);
        collect(&self.config, &mut files);
        files.sort();
        files
    }

    fn install_state_files(&self) -> Vec<PathBuf> {
        fn collect(root: &Path, out: &mut Vec<PathBuf>) {
            let Ok(entries) = fs::read_dir(root) else {
                return;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    collect(&path, out);
                } else if path.file_name().and_then(|name| name.to_str())
                    == Some("install-state.json")
                {
                    out.push(path);
                }
            }
        }
        let mut files = Vec::new();
        collect(&self.state, &mut files);
        files.sort();
        files
    }
}

#[test]
fn real_pre_commit_blocks_primary_business_commit_and_allows_squash_merge() {
    let fixture = Fixture::new();
    must_run(
        &fixture.root,
        "git",
        &["config", "extensions.worktreeConfig", "true"],
    );
    fixture.install();
    let lane = fixture._tmp.path().join("lane-a");
    must_run(
        &fixture.root,
        "git",
        &[
            "worktree",
            "add",
            "-b",
            "lane/a",
            lane.to_str().unwrap(),
            "main",
        ],
    );
    fixture.must_agent_on(
        &lane,
        &[
            "worktree",
            "claim",
            "--id",
            "lane-a",
            "--goal",
            "change app",
            "--base",
            "main",
            "--owns",
            "app",
        ],
    );
    let linked_status = fixture.must_agent_on(
        &lane,
        &[
            "worktree",
            "hooks",
            "status",
            "--repo",
            lane.to_str().unwrap(),
        ],
    );
    assert!(
        combined(&linked_status).contains("WORKTREE HOOKS: healthy"),
        "{}",
        combined(&linked_status)
    );

    fs::write(fixture.root.join("README.md"), "ordinary main edit\n").unwrap();
    must_run(&fixture.root, "git", &["add", "README.md"]);
    let blocked = run(&fixture.root, "git", &["commit", "-m", "must be blocked"]);
    assert!(!blocked.status.success(), "commit unexpectedly succeeded");
    let blocked_text = combined(&blocked);
    assert!(
        blocked_text.contains("BLOCKED by Agent-On pre-commit"),
        "{blocked_text}"
    );
    assert!(
        blocked_text.contains("primary worktree is a control track"),
        "{blocked_text}"
    );
    must_run(&fixture.root, "git", &["restore", "--staged", "README.md"]);
    must_run(&fixture.root, "git", &["restore", "README.md"]);

    fs::write(lane.join("README.md"), "escaped lane edit\n").unwrap();
    must_run(&lane, "git", &["add", "README.md"]);
    let escaped = run(&lane, "git", &["commit", "-m", "escape owns"]);
    assert!(!escaped.status.success(), "out-of-bound commit succeeded");
    let escaped_text = combined(&escaped);
    assert!(
        escaped_text.contains("OUT-OF-BOUNDS: README.md"),
        "{escaped_text}"
    );
    must_run(&lane, "git", &["restore", "--staged", "README.md"]);
    must_run(&lane, "git", &["restore", "README.md"]);

    fs::write(lane.join("app/feature.txt"), "feature\n").unwrap();
    must_run(&lane, "git", &["add", "app/feature.txt"]);
    must_run(&lane, "git", &["commit", "-m", "feature"]);
    must_run(&lane, "git", &["push", "-u", "origin", "lane/a"]);
    fixture.must_agent_on(&lane, &["worktree", "set-status", "ready"]);

    must_run(&fixture.root, "git", &["merge", "--squash", "lane/a"]);
    let merged = must_run(&fixture.root, "git", &["commit", "-m", "squash lane a"]);
    assert!(combined(&merged).contains("squash lane a"));
    assert_eq!(
        fs::read_to_string(fixture.root.join("app/feature.txt")).unwrap(),
        "feature\n"
    );
}

#[test]
fn real_pre_push_blocks_failed_worktree_check() {
    let fixture = Fixture::new();
    fixture.install();
    let orphan = fixture._tmp.path().join("orphan");
    must_run(
        &fixture.root,
        "git",
        &[
            "worktree",
            "add",
            "-b",
            "orphan",
            orphan.to_str().unwrap(),
            "main",
        ],
    );

    fs::write(fixture.root.join("README.md"), "push check\n").unwrap();
    must_run(&fixture.root, "git", &["add", "README.md"]);
    let commit = run(&fixture.root, "git", &["commit", "-m", "must stop early"]);
    assert!(
        !commit.status.success(),
        "unregistered worktree commit succeeded"
    );
    let commit_text = combined(&commit);
    assert!(
        commit_text.contains("BLOCKED by Agent-On pre-commit"),
        "{commit_text}"
    );
    assert!(commit_text.contains("UNREGISTERED:"), "{commit_text}");
    must_run(
        &fixture.root,
        "git",
        &["commit", "--no-verify", "-m", "prepare push"],
    );
    let pushed = run(&fixture.root, "git", &["push", "origin", "main"]);
    assert!(!pushed.status.success(), "push unexpectedly succeeded");
    let pushed_text = combined(&pushed);
    assert!(
        pushed_text.contains("BLOCKED by Agent-On pre-push"),
        "{pushed_text}"
    );
    assert!(pushed_text.contains("UNREGISTERED:"), "{pushed_text}");

    let remote_head = String::from_utf8_lossy(
        &must_run(
            &fixture.root,
            "git",
            &[
                "--git-dir",
                fixture.remote.to_str().unwrap(),
                "rev-parse",
                "refs/heads/main",
            ],
        )
        .stdout,
    )
    .trim()
    .to_string();
    let local_parent =
        String::from_utf8_lossy(&must_run(&fixture.root, "git", &["rev-parse", "HEAD^"]).stdout)
            .trim()
            .to_string();
    assert_eq!(
        remote_head, local_parent,
        "remote advanced despite hook block"
    );
}

#[test]
fn cli_status_and_uninstall_are_real_and_idempotent() {
    let fixture = Fixture::new();
    let missing = fixture.agent_on(
        &fixture.root,
        &[
            "worktree",
            "hooks",
            "status",
            "--repo",
            fixture.root.to_str().unwrap(),
        ],
    );
    assert!(!missing.status.success());
    assert!(combined(&missing).contains("not installed"));

    fixture.install();
    let healthy = fixture.must_agent_on(
        &fixture.root,
        &[
            "worktree",
            "hooks",
            "status",
            "--repo",
            fixture.root.to_str().unwrap(),
        ],
    );
    assert!(combined(&healthy).contains("WORKTREE HOOKS: healthy"));

    fixture.must_agent_on(
        &fixture.root,
        &[
            "worktree",
            "hooks",
            "uninstall",
            "--repo",
            fixture.root.to_str().unwrap(),
        ],
    );
    let config = run(
        &fixture.root,
        "git",
        &["config", "--local", "--get", "core.hooksPath"],
    );
    assert!(!config.status.success());
    let again = fixture.must_agent_on(
        &fixture.root,
        &[
            "worktree",
            "hooks",
            "uninstall",
            "--repo",
            fixture.root.to_str().unwrap(),
        ],
    );
    assert!(combined(&again).contains("already uninstalled"));
}

#[cfg(unix)]
#[test]
fn daily_gc_cli_is_integrated_and_scheduler_drift_makes_uninstall_atomic() {
    let fixture = Fixture::new();
    let schedule = FakeScheduleEnv::new(&fixture._tmp.path().join("schedule-env"));
    let repo = fixture.root.to_str().unwrap();
    let installed = schedule.agent_on(
        &fixture.root,
        &["worktree", "hooks", "install", "--daily-gc", "--repo", repo],
    );
    assert!(installed.status.success(), "{}", combined(&installed));
    assert!(
        combined(&installed).contains("worktree GC schedule: installed and active"),
        "{}",
        combined(&installed)
    );
    let scheduled_files = schedule.scheduler_files();
    assert!(
        !scheduled_files.is_empty(),
        "scheduler config was not written"
    );

    let status = schedule.agent_on(
        &fixture.root,
        &["worktree", "hooks", "status", "--repo", repo],
    );
    assert!(status.status.success(), "{}", combined(&status));
    assert!(
        combined(&status).contains("daily report-only GC: active"),
        "{}",
        combined(&status)
    );

    fs::write(&scheduled_files[0], "user drift\n").unwrap();
    let uninstall = schedule.agent_on(
        &fixture.root,
        &["worktree", "hooks", "uninstall", "--repo", repo],
    );
    assert!(!uninstall.status.success(), "uninstall ignored drift");
    let text = combined(&uninstall);
    assert!(
        text.contains("refusing to disable or remove changed scheduler assets"),
        "{text}"
    );
    assert!(text.contains("Git hooks were left installed"), "{text}");

    let hooks_path = must_run(
        &fixture.root,
        "git",
        &["config", "--local", "--get", "core.hooksPath"],
    );
    let hooks_path = PathBuf::from(String::from_utf8_lossy(&hooks_path.stdout).trim());
    assert!(hooks_path.join("pre-commit").exists());
    assert!(hooks_path.join("pre-push").exists());
}

#[cfg(unix)]
#[test]
fn hook_drift_preflight_leaves_daily_scheduler_untouched() {
    let fixture = Fixture::new();
    let schedule = FakeScheduleEnv::new(&fixture._tmp.path().join("schedule-env"));
    let repo = fixture.root.to_str().unwrap();
    let installed = schedule.agent_on(
        &fixture.root,
        &["worktree", "hooks", "install", "--daily-gc", "--repo", repo],
    );
    assert!(installed.status.success(), "{}", combined(&installed));
    let scheduled_files = schedule.scheduler_files();
    assert!(!scheduled_files.is_empty());

    let hooks_path = must_run(
        &fixture.root,
        "git",
        &["config", "--local", "--get", "core.hooksPath"],
    );
    let hooks_path = PathBuf::from(String::from_utf8_lossy(&hooks_path.stdout).trim());
    fs::write(hooks_path.join("pre-commit"), "#!/bin/sh\nexit 0\n").unwrap();

    let uninstall = schedule.agent_on(
        &fixture.root,
        &["worktree", "hooks", "uninstall", "--repo", repo],
    );
    assert!(!uninstall.status.success(), "uninstall ignored hook drift");
    let text = combined(&uninstall);
    assert!(
        text.contains("uninstall preflight failed for Git hooks"),
        "{text}"
    );
    assert!(text.contains("left unchanged"), "{text}");
    for path in scheduled_files {
        assert!(
            path.exists(),
            "scheduler file was removed: {}",
            path.display()
        );
    }
}

#[cfg(unix)]
#[test]
fn linked_worktree_install_uses_one_shared_hook_and_schedule_identity() {
    let fixture = Fixture::new();
    let linked = fixture._tmp.path().join("installer-lane");
    must_run(
        &fixture.root,
        "git",
        &[
            "worktree",
            "add",
            "-b",
            "installer-lane",
            linked.to_str().unwrap(),
            "main",
        ],
    );
    let schedule = FakeScheduleEnv::new(&fixture._tmp.path().join("schedule-env"));
    let installed = schedule.agent_on(
        &linked,
        &[
            "worktree",
            "hooks",
            "install",
            "--daily-gc",
            "--repo",
            linked.to_str().unwrap(),
        ],
    );
    assert!(installed.status.success(), "{}", combined(&installed));

    let status = schedule.agent_on(
        &fixture.root,
        &[
            "worktree",
            "hooks",
            "status",
            "--repo",
            fixture.root.to_str().unwrap(),
        ],
    );
    assert!(status.status.success(), "{}", combined(&status));
    assert!(combined(&status).contains("daily report-only GC: active"));

    let removed = schedule.agent_on(
        &fixture.root,
        &[
            "worktree",
            "hooks",
            "uninstall",
            "--repo",
            fixture.root.to_str().unwrap(),
        ],
    );
    assert!(removed.status.success(), "{}", combined(&removed));
    assert!(schedule.scheduler_files().is_empty());
    let config = run(
        &fixture.root,
        "git",
        &["config", "--local", "--get", "core.hooksPath"],
    );
    assert!(!config.status.success());
}

#[cfg(unix)]
#[test]
fn persisted_schedule_survives_caller_path_and_executable_change() {
    use std::os::unix::fs::PermissionsExt;

    let fixture = Fixture::new();
    let schedule = FakeScheduleEnv::new(&fixture._tmp.path().join("schedule-env"));
    let repo = fixture.root.to_str().unwrap();
    let installed = schedule.agent_on(
        &fixture.root,
        &["worktree", "hooks", "install", "--daily-gc", "--repo", repo],
    );
    assert!(installed.status.success(), "{}", combined(&installed));
    assert_eq!(schedule.install_state_files().len(), 1);

    let copied_cli = fixture._tmp.path().join("agent-on-relocated");
    fs::copy(env!("CARGO_BIN_EXE_agent-on"), &copied_cli).unwrap();
    let mut permissions = fs::metadata(&copied_cli).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&copied_cli, permissions).unwrap();
    let changed_path = format!("{}:/usr/bin:/bin", schedule.bin.display());
    assert_ne!(changed_path, schedule.path);

    let status = schedule.agent_on_with(
        copied_cli.to_str().unwrap(),
        &fixture.root,
        &["worktree", "hooks", "status", "--repo", repo],
        &changed_path,
    );
    assert!(status.status.success(), "{}", combined(&status));
    assert!(
        combined(&status).contains("daily report-only GC: active"),
        "{}",
        combined(&status)
    );

    let removed = schedule.agent_on_with(
        copied_cli.to_str().unwrap(),
        &fixture.root,
        &["worktree", "hooks", "uninstall", "--repo", repo],
        &changed_path,
    );
    assert!(removed.status.success(), "{}", combined(&removed));
    assert!(schedule.scheduler_files().is_empty());
    assert!(schedule.install_state_files().is_empty());
}

#[cfg(unix)]
#[test]
fn moved_repo_new_path_uninstalls_both_facets_from_persisted_identity() {
    let fixture = Fixture::new();
    let schedule = FakeScheduleEnv::new(&fixture._tmp.path().join("schedule-env"));
    let old = fixture.root.clone();
    let old_text = old.to_str().unwrap();
    let installed = schedule.agent_on(
        &old,
        &[
            "worktree",
            "hooks",
            "install",
            "--daily-gc",
            "--repo",
            old_text,
        ],
    );
    assert!(installed.status.success(), "{}", combined(&installed));
    assert_eq!(schedule.install_state_files().len(), 1);

    let moved = fixture._tmp.path().join("repo-moved");
    fs::rename(&old, &moved).unwrap();
    let moved_text = moved.to_str().unwrap();
    let status = schedule.agent_on(
        &moved,
        &["worktree", "hooks", "status", "--repo", moved_text],
    );
    assert!(
        !status.status.success(),
        "moved hooks path was reported healthy"
    );
    let status_text = combined(&status);
    assert!(
        status_text.contains("WORKTREE HOOKS: unhealthy"),
        "{status_text}"
    );
    assert!(
        status_text.contains("repository moved or was deleted"),
        "{status_text}"
    );

    let removed = schedule.agent_on(
        &moved,
        &["worktree", "hooks", "uninstall", "--repo", moved_text],
    );
    assert!(removed.status.success(), "{}", combined(&removed));
    assert!(schedule.scheduler_files().is_empty());
    assert!(schedule.install_state_files().is_empty());
    let config = run(
        &moved,
        "git",
        &["config", "--local", "--get", "core.hooksPath"],
    );
    assert!(!config.status.success());
}

#[cfg(unix)]
#[test]
fn unreachable_old_path_cleans_scheduler_but_reports_git_facet_unknown() {
    let fixture = Fixture::new();
    let schedule = FakeScheduleEnv::new(&fixture._tmp.path().join("schedule-env"));
    let old = fixture.root.clone();
    let old_text = old.to_str().unwrap();
    let installed = schedule.agent_on(
        &old,
        &[
            "worktree",
            "hooks",
            "install",
            "--daily-gc",
            "--repo",
            old_text,
        ],
    );
    assert!(installed.status.success(), "{}", combined(&installed));
    let moved = fixture._tmp.path().join("repo-moved");
    fs::rename(&old, &moved).unwrap();

    let status = schedule.agent_on(
        fixture._tmp.path(),
        &["worktree", "hooks", "status", "--repo", old_text],
    );
    assert!(!status.status.success());
    let status_text = combined(&status);
    assert!(
        status_text.contains("cannot inspect Agent-On worktree hooks"),
        "{status_text}"
    );
    assert!(
        status_text.contains("repository moved or was deleted"),
        "{status_text}"
    );

    let partial = schedule.agent_on(
        fixture._tmp.path(),
        &["worktree", "hooks", "uninstall", "--repo", old_text],
    );
    assert!(
        !partial.status.success(),
        "unknown Git facet was reported green"
    );
    let partial_text = combined(&partial);
    assert!(
        partial_text.contains("Git hooks: unavailable"),
        "{partial_text}"
    );
    assert!(partial_text.contains("RESULT: PARTIAL"), "{partial_text}");
    assert!(
        partial_text.contains("worktree GC schedule: uninstalled"),
        "{partial_text}"
    );
    assert!(schedule.scheduler_files().is_empty());
    assert!(schedule.install_state_files().is_empty());

    let still_configured = must_run(
        &moved,
        "git",
        &["config", "--local", "--get", "core.hooksPath"],
    );
    assert!(!String::from_utf8_lossy(&still_configured.stdout)
        .trim()
        .is_empty());
    let cleanup = schedule.agent_on(
        &moved,
        &[
            "worktree",
            "hooks",
            "uninstall",
            "--repo",
            moved.to_str().unwrap(),
        ],
    );
    assert!(cleanup.status.success(), "{}", combined(&cleanup));
}
