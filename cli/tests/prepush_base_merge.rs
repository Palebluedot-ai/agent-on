//! pre-push: a branch with an open PR takes base only through the server.
//!
//! Dartify PR #204 (2026-08-19): a session merged origin/main into its PR
//! branch locally 34 seconds before the server-side update-branch did the
//! same. These tests push for real through the installed hook. The remote URL
//! reads as GitHub (`insteadOf` sends it to a local bare repo) and `gh` is a
//! fake on PATH that answers the open-PR lookup.
#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use tempfile::TempDir;

const URL: &str = "https://github.com/acme/widget.git";

struct Repo {
    _tmp: TempDir,
    work: PathBuf,
    remote: PathBuf,
    fake_bin: PathBuf,
    gh_log: PathBuf,
}

fn combined(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

impl Repo {
    fn new() -> Self {
        let tmp = TempDir::new().unwrap();
        let work = tmp.path().join("work");
        let remote = tmp.path().join("remote.git");
        let fake_bin = tmp.path().join("fake-bin");
        let gh_log = tmp.path().join("gh.log");
        fs::create_dir_all(&work).unwrap();
        fs::create_dir_all(&fake_bin).unwrap();
        let gh = fake_bin.join("gh");
        fs::write(
            &gh,
            "#!/bin/sh\nprintf '%s\\n' \"$*\" >> \"$FAKE_GH_LOG\"\n[ -n \"$FAKE_GH_FAIL\" ] && exit 1\nprintf '%s\\n' \"${FAKE_GH_PRS:-[]}\"\n",
        )
        .unwrap();
        fs::set_permissions(&gh, fs::Permissions::from_mode(0o755)).unwrap();
        let repo = Self {
            _tmp: tmp,
            work,
            remote,
            fake_bin,
            gh_log,
        };
        repo.git_in(
            repo.work.parent().unwrap(),
            &[
                "init",
                "-q",
                "--bare",
                "-b",
                "main",
                repo.remote.to_str().unwrap(),
            ],
        );
        repo.git(&["init", "-q", "-b", "main"]);
        repo.git(&["config", "user.email", "test@example.com"]);
        repo.git(&["config", "user.name", "Test"]);
        repo.git(&["remote", "add", "origin", URL]);
        repo.git(&[
            "config",
            &format!("url.{}.insteadOf", repo.remote.display()),
            URL,
        ]);
        fs::write(repo.work.join("README.md"), "one\ntwo\nthree\n").unwrap();
        repo.git(&["add", "."]);
        repo.git(&["commit", "-qm", "init"]);
        repo.git(&["push", "-q", "-u", "origin", "main"]);
        let install = Command::new(env!("CARGO_BIN_EXE_agent-on"))
            .args(["worktree", "hooks", "install", "--repo"])
            .arg(&repo.work)
            .output()
            .unwrap();
        assert!(install.status.success(), "{}", combined(&install));
        repo
    }

    fn git_in(&self, cwd: &Path, args: &[&str]) -> Output {
        let out = Command::new("git")
            .current_dir(cwd)
            .args(args)
            .output()
            .unwrap();
        assert!(out.status.success(), "git {args:?}: {}", combined(&out));
        out
    }

    fn git(&self, args: &[&str]) -> Output {
        self.git_in(&self.work, args)
    }

    fn path_env(&self) -> String {
        format!(
            "{}:{}",
            self.fake_bin.display(),
            std::env::var("PATH").unwrap_or_default()
        )
    }

    /// `gh pr list` answers `prs` (a JSON array), or fails when `prs` is None.
    fn push(&self, args: &[&str], prs: Option<&str>) -> Output {
        let mut cmd = Command::new("git");
        cmd.current_dir(&self.work)
            .arg("push")
            .args(args)
            .env("PATH", self.path_env())
            .env("FAKE_GH_LOG", &self.gh_log);
        match prs {
            Some(prs) => cmd.env("FAKE_GH_PRS", prs),
            None => cmd.env("FAKE_GH_FAIL", "1"),
        };
        cmd.output().unwrap()
    }

    fn remote_rev(&self, refname: &str) -> String {
        let out = self.git(&[
            "--git-dir",
            self.remote.to_str().unwrap(),
            "rev-parse",
            refname,
        ]);
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    fn rev(&self, rev: &str) -> String {
        String::from_utf8_lossy(&self.git(&["rev-parse", rev]).stdout)
            .trim()
            .to_string()
    }

    /// feat/x exists on the server, then main moves on the server. feat/x
    /// rewrites README line two; main rewrites the same line when `conflict`,
    /// and only adds a file otherwise.
    fn pr_branch_behind_main(&self, conflict: bool) {
        self.git(&["checkout", "-q", "-b", "feat/x"]);
        fs::write(self.work.join("feature.txt"), "feature\n").unwrap();
        let readme = fs::read_to_string(self.work.join("README.md")).unwrap();
        fs::write(
            self.work.join("README.md"),
            readme.replace("two", "two-feature"),
        )
        .unwrap();
        self.git(&["add", "."]);
        self.git(&["commit", "-qm", "feature work"]);
        let first = self.push(&["-q", "-u", "origin", "feat/x"], Some("[]"));
        assert!(first.status.success(), "{}", combined(&first));
        self.git(&["checkout", "-q", "main"]);
        if conflict {
            let readme = fs::read_to_string(self.work.join("README.md")).unwrap();
            fs::write(
                self.work.join("README.md"),
                readme.replace("two", "two-main"),
            )
            .unwrap();
        } else {
            fs::write(self.work.join("main.txt"), "main\n").unwrap();
        }
        self.git(&["add", "."]);
        self.git(&["commit", "-qm", "main moves"]);
        let main = self.push(&["-q", "origin", "main"], Some("[]"));
        assert!(main.status.success(), "{}", combined(&main));
        self.git(&["checkout", "-q", "feat/x"]);
    }
}

#[test]
fn clean_local_merge_of_origin_main_into_an_open_pr_branch_is_blocked() {
    let repo = Repo::new();
    // Different lines: git merges it cleanly, so the server could have.
    repo.pr_branch_behind_main(false);
    let before = repo.remote_rev("refs/heads/feat/x");
    let pre_merge = repo.rev("HEAD");
    repo.git(&["merge", "-q", "--no-edit", "origin/main"]);

    let out = repo.push(&["origin", "feat/x"], Some(r#"[{"number":42}]"#));
    let text = combined(&out);
    assert!(!out.status.success(), "push went through:\n{text}");
    assert!(text.contains("BLOCKED by Agent-On pre-push"), "{text}");
    assert!(
        text.contains("gh api -X PUT repos/acme/widget/pulls/42/update-branch"),
        "{text}"
    );
    assert!(
        text.contains(&format!("git reset --keep {}", &pre_merge[..12])),
        "{text}"
    );
    assert_eq!(repo.remote_rev("refs/heads/feat/x"), before);
    let asked = fs::read_to_string(&repo.gh_log).unwrap();
    assert!(
        asked.contains("acme/widget") && asked.contains("feat/x"),
        "{asked}"
    );
}

#[test]
fn when_the_pr_cannot_be_looked_up_the_block_says_how_to_find_n() {
    let repo = Repo::new();
    repo.pr_branch_behind_main(false);
    repo.git(&["merge", "-q", "--no-edit", "origin/main"]);
    let out = repo.push(&["origin", "feat/x"], None);
    let text = combined(&out);
    assert!(!out.status.success(), "{text}");
    assert!(
        text.contains("gh api -X PUT repos/acme/widget/pulls/<N>/update-branch"),
        "{text}"
    );
    assert!(text.contains("gh pr view feat/x --json number"), "{text}");
}

#[test]
fn a_branch_without_an_open_pr_is_not_covered() {
    let repo = Repo::new();
    repo.pr_branch_behind_main(false);
    repo.git(&["merge", "-q", "--no-edit", "origin/main"]);
    let out = repo.push(&["-q", "origin", "feat/x"], Some("[]"));
    assert!(out.status.success(), "{}", combined(&out));
}

/// update-branch cannot resolve conflicts (GitHub answers 422), so a local
/// merge is the only way through. Blocking it would leave no exit (case 40).
#[test]
fn a_conflicting_merge_resolved_locally_goes_through() {
    let repo = Repo::new();
    repo.pr_branch_behind_main(true);
    let merge = Command::new("git")
        .current_dir(&repo.work)
        .args(["merge", "-q", "--no-edit", "origin/main"])
        .output()
        .unwrap();
    assert!(!merge.status.success(), "the fixture must conflict");
    fs::write(repo.work.join("README.md"), "one\ntwo-both\nthree\n").unwrap();
    repo.git(&["commit", "-qam", "resolve conflict with main"]);
    let out = repo.push(&["-q", "origin", "feat/x"], Some(r#"[{"number":42}]"#));
    assert!(out.status.success(), "{}", combined(&out));
}

#[test]
fn a_merge_committed_by_github_goes_through() {
    let repo = Repo::new();
    repo.pr_branch_behind_main(false);
    let merge = Command::new("git")
        .current_dir(&repo.work)
        .args(["merge", "-q", "--no-edit", "origin/main"])
        .env("GIT_COMMITTER_NAME", "GitHub")
        .env("GIT_COMMITTER_EMAIL", "noreply@github.com")
        .output()
        .unwrap();
    assert!(merge.status.success(), "{}", combined(&merge));
    let out = repo.push(&["-q", "origin", "feat/x"], Some(r#"[{"number":42}]"#));
    assert!(out.status.success(), "{}", combined(&out));
}

/// The rule is about PR branches. `git pull` on the default branch itself is
/// someone else's business.
#[test]
fn pushing_the_default_branch_is_not_covered() {
    let repo = Repo::new();
    let other = repo.work.parent().unwrap().join("other");
    repo.git_in(
        repo.work.parent().unwrap(),
        &[
            "clone",
            "-q",
            repo.remote.to_str().unwrap(),
            other.to_str().unwrap(),
        ],
    );
    repo.git_in(&other, &["config", "user.email", "o@example.com"]);
    repo.git_in(&other, &["config", "user.name", "O"]);
    fs::write(other.join("other.txt"), "other\n").unwrap();
    repo.git_in(&other, &["add", "."]);
    repo.git_in(&other, &["commit", "-qm", "someone else on main"]);
    repo.git_in(&other, &["push", "-q", "origin", "main"]);
    fs::write(repo.work.join("mine.txt"), "mine\n").unwrap();
    repo.git(&["add", "."]);
    repo.git(&["commit", "-qm", "mine on main"]);
    repo.git(&["fetch", "-q", "origin"]);
    repo.git(&["merge", "-q", "--no-edit", "origin/main"]);
    let out = repo.push(&["-q", "origin", "main"], Some(r#"[{"number":42}]"#));
    assert!(out.status.success(), "{}", combined(&out));
}

/// Hook scripts installed before this check pass no arguments: the remote
/// then defaults to origin, and the verdict is the same.
#[test]
fn hook_scripts_that_pass_no_arguments_still_get_the_check() {
    let repo = Repo::new();
    repo.pr_branch_behind_main(false);
    repo.git(&["merge", "-q", "--no-edit", "origin/main"]);
    let local = repo.rev("HEAD");
    let remote = repo.remote_rev("refs/heads/feat/x");
    let mut child = Command::new(env!("CARGO_BIN_EXE_agent-on"))
        .args(["worktree", "hooks", "run", "--hook", "pre-push", "--repo"])
        .arg(&repo.work)
        .env("PATH", repo.path_env())
        .env("FAKE_GH_LOG", &repo.gh_log)
        .env("FAKE_GH_PRS", r#"[{"number":7}]"#)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    use std::io::Write;
    child
        .stdin
        .take()
        .unwrap()
        .write_all(format!("refs/heads/feat/x {local} refs/heads/feat/x {remote}\n").as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    let text = combined(&out);
    assert!(!out.status.success(), "{text}");
    assert!(
        text.contains("gh api -X PUT repos/acme/widget/pulls/7/update-branch"),
        "{text}"
    );
}
