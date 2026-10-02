//! Real Git operations prove that only the effective index / pushed history
//! participates in the same-file gate. Fixtures never touch the user's repo.
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};
use tempfile::TempDir;

fn git(root: &Path, args: &[&str]) -> Output {
    Command::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .unwrap()
}
fn text(out: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}
fn must(root: &Path, args: &[&str]) -> Output {
    let out = git(root, args);
    assert!(out.status.success(), "{args:?}: {}", text(&out));
    out
}
struct Field {
    _tmp: TempDir,
    root: PathBuf,
    other: PathBuf,
}
impl Field {
    fn new() -> Self {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path().join("repo");
        let other = tmp.path().join("other");
        let remote = tmp.path().join("remote.git");
        fs::create_dir(&root).unwrap();
        must(&root, &["init", "-b", "main"]);
        must(&root, &["config", "user.name", "Scope test"]);
        must(&root, &["config", "user.email", "scope@example.com"]);
        for name in ["shared.txt", "notes.txt"] {
            fs::write(root.join(name), "base\n").unwrap();
        }
        must(&root, &["add", "."]);
        must(&root, &["commit", "-m", "base"]);
        must(&root, &["init", "--bare", remote.to_str().unwrap()]);
        must(
            &root,
            &["remote", "add", "origin", remote.to_str().unwrap()],
        );
        must(&root, &["push", "-u", "origin", "main"]);
        must(
            &root,
            &["worktree", "add", "-b", "worker", other.to_str().unwrap()],
        );
        let out = Command::new(env!("CARGO_BIN_EXE_agent-on"))
            .current_dir(&root)
            .args(["worktree", "hooks", "install"])
            .output()
            .unwrap();
        assert!(out.status.success(), "{}", text(&out));
        Self {
            _tmp: tmp,
            root,
            other,
        }
    }
    fn collide(&self) {
        fs::write(self.root.join("shared.txt"), "main dirty\n").unwrap();
        fs::write(self.other.join("shared.txt"), "other dirty\n").unwrap();
    }
    fn note(&self) {
        fs::write(self.root.join("notes.txt"), "notes changed\n").unwrap();
    }
    fn committed_paths(&self) -> String {
        String::from_utf8(
            must(
                &self.root,
                &["diff-tree", "--no-commit-id", "--name-only", "-r", "HEAD"],
            )
            .stdout,
        )
        .unwrap()
    }
}

#[test]
fn unrelated_staged_commit_passes_with_shared_dirty_files() {
    let f = Field::new();
    f.collide();
    f.note();
    must(&f.root, &["add", "notes.txt"]);
    must(&f.root, &["commit", "-m", "notes only"]);
    assert_eq!(f.committed_paths(), "notes.txt\n");
    assert_eq!(
        fs::read_to_string(f.root.join("shared.txt")).unwrap(),
        "main dirty\n"
    );
}

#[test]
fn only_uses_gits_temporary_index_and_preserves_other_staged_work() {
    let f = Field::new();
    f.collide();
    f.note();
    must(&f.root, &["add", "shared.txt"]);
    must(
        &f.root,
        &["commit", "--only", "-m", "notes only", "--", "notes.txt"],
    );
    assert_eq!(f.committed_paths(), "notes.txt\n");
    assert_eq!(
        String::from_utf8(must(&f.root, &["diff", "--cached", "--name-only"]).stdout).unwrap(),
        "shared.txt\n"
    );
}

#[test]
fn all_blocks_the_shared_file_even_when_real_index_is_empty() {
    let f = Field::new();
    f.collide();
    let out = git(&f.root, &["commit", "-a", "-m", "all"]);
    assert!(!out.status.success());
    assert!(text(&out).contains("blocked: shared.txt"), "{}", text(&out));
}

#[test]
fn explicit_shared_path_blocks_instead_of_checking_preexisting_notes_index() {
    let f = Field::new();
    f.collide();
    f.note();
    must(&f.root, &["add", "notes.txt"]);
    let out = git(&f.root, &["commit", "-m", "shared", "--", "shared.txt"]);
    assert!(!out.status.success());
    assert!(text(&out).contains("blocked: shared.txt"), "{}", text(&out));
}

#[test]
fn an_empty_commit_carries_no_dirty_paths() {
    let f = Field::new();
    f.collide();
    must(&f.root, &["commit", "--allow-empty", "-m", "empty"]);
    assert_eq!(f.committed_paths(), "");
}

#[test]
fn renaming_a_shared_file_checks_its_source_path() {
    let f = Field::new();
    fs::write(f.other.join("shared.txt"), "other dirty\n").unwrap();
    must(&f.root, &["mv", "shared.txt", "renamed.txt"]);
    let out = git(&f.root, &["commit", "-m", "rename shared"]);
    assert!(!out.status.success());
    assert!(text(&out).contains("blocked: shared.txt"), "{}", text(&out));
}

#[test]
fn malformed_push_input_does_not_become_an_empty_scope() {
    let f = Field::new();
    let mut child = Command::new(env!("CARGO_BIN_EXE_agent-on"))
        .current_dir(&f.root)
        .args(["worktree", "hooks", "run", "--hook", "pre-push"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"refs/heads/main bad refs/heads/main bad extra\n")
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(!out.status.success());
    assert!(
        text(&out).contains("malformed ref update"),
        "{}",
        text(&out)
    );
}

#[test]
fn an_unrelated_push_passes_and_reaches_the_remote() {
    let f = Field::new();
    f.collide();
    f.note();
    must(&f.root, &["add", "notes.txt"]);
    must(&f.root, &["commit", "-m", "notes"]);
    must(&f.root, &["push", "origin", "main"]);
    assert_eq!(
        must(&f.root, &["rev-parse", "HEAD"]).stdout,
        must(&f.root, &["rev-parse", "origin/main"]).stdout
    );
}

#[test]
fn push_checks_each_commit_even_when_shared_change_was_reverted() {
    let f = Field::new();
    fs::write(f.root.join("shared.txt"), "committed change\n").unwrap();
    must(&f.root, &["add", "shared.txt"]);
    must(&f.root, &["commit", "-m", "shared change"]);
    fs::write(f.root.join("shared.txt"), "base\n").unwrap();
    must(&f.root, &["add", "shared.txt"]);
    must(&f.root, &["commit", "-m", "revert shared"]);
    f.collide();
    let out = git(&f.root, &["push", "origin", "main"]);
    assert!(!out.status.success());
    assert!(text(&out).contains("blocked: shared.txt"), "{}", text(&out));
    assert_ne!(
        must(&f.root, &["rev-parse", "HEAD"]).stdout,
        must(&f.root, &["rev-parse", "origin/main"]).stdout
    );
}

#[test]
fn tag_only_push_has_no_worktree_write_scope() {
    let f = Field::new();
    f.collide();
    must(&f.root, &["tag", "-a", "v-test", "-m", "evidence"]);
    must(&f.root, &["push", "origin", "v-test"]);
}

#[test]
fn a_pretool_commit_defers_scope_to_git_while_status_still_reports_overlap() {
    let f = Field::new();
    f.collide();
    let mut child = Command::new(env!("CARGO_BIN_EXE_agent-on"))
        .current_dir(&f.root)
        .env("AGENT_ON_ROOT", "/nonexistent/operation-scope")
        .arg("guard")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(serde_json::json!({"tool_name":"Bash","cwd":f.root,"tool_input":{"command":"git commit --only notes.txt"}}).to_string().as_bytes()).unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success(), "{}", text(&out));
    let status = Command::new(env!("CARGO_BIN_EXE_agent-on"))
        .current_dir(&f.root)
        .args(["worktree", "check"])
        .output()
        .unwrap();
    assert!(!status.status.success());
    assert!(text(&status).contains("blocked: shared.txt"));
}
