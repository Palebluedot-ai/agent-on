//! End-to-end: on-call registry + cross-window routing gate through the real
//! binary and the real PreToolUse stdin contract.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use tempfile::TempDir;

fn must_run(cwd: &Path, program: &str, args: &[&str]) {
    let output = Command::new(program)
        .current_dir(cwd)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{program} {} failed: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn agent_on(cwd: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_agent-on"))
        .current_dir(cwd)
        .args(args)
        .output()
        .unwrap()
}

/// Feed a PreToolUse payload to `agent-on guard` exactly like the hook does.
fn guard(cwd: &Path, payload: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_agent-on"))
        .current_dir(cwd)
        .arg("guard")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(payload.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

fn combined(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

/// One repo, two windows: `oncall` (the primary worktree) and `feature`.
fn fixture() -> (TempDir, PathBuf, PathBuf) {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("repo");
    std::fs::create_dir_all(&root).unwrap();
    must_run(&root, "git", &["init", "-b", "main"]);
    must_run(&root, "git", &["config", "user.email", "t@example.com"]);
    must_run(&root, "git", &["config", "user.name", "T"]);
    std::fs::write(root.join("README.md"), "x\n").unwrap();
    must_run(&root, "git", &["add", "."]);
    must_run(&root, "git", &["commit", "-m", "init"]);
    let feature = tmp.path().join("feature");
    must_run(
        &root,
        "git",
        &[
            "worktree",
            "add",
            "-b",
            "feature",
            feature.to_str().unwrap(),
            "main",
        ],
    );
    // A compliant feature window also holds a lane, so the pre-existing
    // boundary guard has nothing to say about its own branch push.
    let claimed = agent_on(
        &feature,
        &[
            "worktree",
            "claim",
            "--id",
            "feature-lane",
            "--goal",
            "feature work",
            "--base",
            "main",
            "--owns",
            "README.md",
        ],
    );
    assert!(claimed.status.success(), "{}", combined(&claimed));
    (tmp, root, feature)
}

fn bash_payload(cwd: &Path, cmd: &str) -> String {
    serde_json::json!({
        "tool_name": "Bash",
        "cwd": cwd.display().to_string(),
        "tool_input": {"command": cmd}
    })
    .to_string()
}

#[test]
fn registry_is_shared_across_worktrees_of_the_same_repo() {
    let (_tmp, root, feature) = fixture();
    let out = agent_on(&root, &["oncall", "claim", "--session", "babysit-window-a"]);
    assert!(out.status.success(), "{}", combined(&out));

    // The feature window reads the same record — this is the point of storing
    // it in the common git dir instead of a per-worktree file.
    let out = agent_on(&feature, &["oncall", "status", "--json"]);
    let text = String::from_utf8_lossy(&out.stdout);
    let value: serde_json::Value = serde_json::from_str(text.trim()).unwrap();
    assert_eq!(value["present"], serde_json::json!(true));
    assert_eq!(value["session"], serde_json::json!("babysit-window-a"));
    assert_eq!(value["self_is_oncall"], serde_json::json!(false));

    let out = agent_on(&root, &["oncall", "whoami", "--json"]);
    let text = String::from_utf8_lossy(&out.stdout);
    let value: serde_json::Value = serde_json::from_str(text.trim()).unwrap();
    assert_eq!(value["role"], serde_json::json!("oncall"));
}

#[test]
fn feature_window_merge_is_blocked_with_a_reroute_template() {
    let (_tmp, root, feature) = fixture();
    agent_on(&root, &["oncall", "claim", "--session", "babysit-window-a"]);

    let out = guard(&feature, &bash_payload(&feature, "gh pr merge 17 --merge"));
    assert_eq!(out.status.code(), Some(2), "{}", combined(&out));
    let text = combined(&out);
    // The block message must carry: what class, who is on call, the reroute
    // template, and the two traceable escape hatches.
    assert!(text.contains("跨窗口指令路由拦截"), "{text}");
    assert!(text.contains("babysit-window-a"), "{text}");
    assert!(text.contains("【转投】"), "{text}");
    assert!(text.contains("oncall release --force"), "{text}");
    assert!(text.contains("oncall claim"), "{text}");
}

#[test]
fn oncall_window_runs_the_same_command_untouched() {
    let (_tmp, root, _feature) = fixture();
    agent_on(&root, &["oncall", "claim", "--session", "babysit-window-a"]);
    let out = guard(&root, &bash_payload(&root, "gh pr merge 17 --merge"));
    assert_eq!(out.status.code(), Some(0), "{}", combined(&out));
}

#[test]
fn gate_is_inert_until_someone_goes_on_call() {
    let (_tmp, _root, feature) = fixture();
    let out = guard(&feature, &bash_payload(&feature, "gh pr merge 17 --merge"));
    assert_eq!(out.status.code(), Some(0), "{}", combined(&out));
}

#[test]
fn feature_window_delivery_path_stays_open() {
    let (_tmp, root, feature) = fixture();
    agent_on(&root, &["oncall", "claim", "--session", "babysit-window-a"]);
    for cmd in [
        "gh pr create --fill",
        "gh pr list --state open",
        "gh pr view 17 --json mergeable,mergeStateStatus",
        "git push -u origin feature",
    ] {
        let out = guard(&feature, &bash_payload(&feature, cmd));
        assert_eq!(out.status.code(), Some(0), "{cmd}: {}", combined(&out));
    }
}

#[test]
fn sendmessage_is_funnelled_to_the_oncall_window() {
    let (_tmp, root, feature) = fixture();
    agent_on(&root, &["oncall", "claim", "--session", "babysit-window-a"]);
    let msg = |to: &str| {
        serde_json::json!({
            "tool_name": "SendMessage",
            "cwd": feature.display().to_string(),
            "tool_input": {"to": to, "message": "【交单】PR #17"}
        })
        .to_string()
    };
    // 交单 / 回执 → 值守：唯一放行的出站通道
    let out = guard(&feature, &msg("babysit-window-a-02"));
    assert_eq!(out.status.code(), Some(0), "{}", combined(&out));

    // 会话内部通信（子代理 / main）不归这条闸管
    for internal in ["main", "researcher"] {
        let out = guard(&feature, &msg(internal));
        assert_eq!(out.status.code(), Some(0), "{internal}: {}", combined(&out));
    }

    // 横向找**另一个已登记的窗口**：拦下并要求经值守中转
    let peer = feature.parent().unwrap().join("peer-lane-3f21");
    must_run(
        &feature,
        "git",
        &[
            "worktree",
            "add",
            "-b",
            "peer",
            peer.to_str().unwrap(),
            "main",
        ],
    );
    let claimed = agent_on(
        &peer,
        &[
            "worktree",
            "claim",
            "--id",
            "peer-lane",
            "--goal",
            "g",
            "--base",
            "main",
            "--owns",
            "peer",
        ],
    );
    assert!(claimed.status.success(), "{}", combined(&claimed));

    let out = guard(&feature, &msg("peer-lane-3f21-07"));
    assert_eq!(out.status.code(), Some(2), "{}", combined(&out));
    assert!(combined(&out).contains("跨窗口沟通"), "{}", combined(&out));
    assert!(combined(&out).contains("peer-lane"), "{}", combined(&out));
}

#[test]
fn handover_requires_force_and_release_reopens_the_gate() {
    let (_tmp, root, feature) = fixture();
    agent_on(&root, &["oncall", "claim", "--session", "babysit-window-a"]);

    let out = agent_on(&feature, &["oncall", "claim", "--session", "feature-b"]);
    assert_eq!(out.status.code(), Some(1), "{}", combined(&out));
    assert!(
        combined(&out).contains("已有值守在班"),
        "{}",
        combined(&out)
    );

    let out = agent_on(
        &feature,
        &["oncall", "claim", "--session", "feature-b", "--force"],
    );
    assert!(out.status.success(), "{}", combined(&out));

    // After handover the old on-call window is the one that gets blocked.
    let out = guard(&root, &bash_payload(&root, "gh pr merge 17 --merge"));
    assert_eq!(out.status.code(), Some(2), "{}", combined(&out));

    let out = agent_on(&feature, &["oncall", "release"]);
    assert!(out.status.success(), "{}", combined(&out));
    let out = guard(&root, &bash_payload(&root, "gh pr merge 17 --merge"));
    assert_eq!(out.status.code(), Some(0), "{}", combined(&out));
}
