//! Project delivery brief through the public CLI, using real Git and local receipts.
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::{Command, Output};

fn git(repo: &Path, args: &[&str]) {
    let output = Command::new("git")
        .current_dir(repo)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn cli(repo: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_agent-on"))
        .current_dir(repo)
        .env_remove("CODEX_THREAD_ID")
        .env_remove("CLAUDE_CODE_SESSION_ID")
        .env_remove("CLAUDE_PROJECT_DIR")
        .env_remove("CODEX_PROJECT_DIR")
        .env_remove("AGENT_ON_CONTROL_ID")
        .env_remove("AGENT_ON_CONTROL_REPO")
        .env("AGENT_ON_HOST", "codex")
        .env("AGENT_ON_SESSION_ID", "status-fixture")
        .args(args)
        .output()
        .unwrap()
}

fn ok(repo: &Path, args: &[&str]) -> String {
    let output = cli(repo, args);
    assert!(
        output.status.success(),
        "{args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

fn fixture() -> tempfile::TempDir {
    let temp = tempfile::tempdir().unwrap();
    git(temp.path(), &["init", "-b", "main"]);
    git(temp.path(), &["config", "user.name", "Test"]);
    git(temp.path(), &["config", "user.email", "test@example.com"]);
    fs::write(temp.path().join("base.txt"), "base\n").unwrap();
    git(temp.path(), &["add", "base.txt"]);
    git(temp.path(), &["commit", "-qm", "base"]);
    temp
}

#[test]
fn delivery_brief_exposes_goal_blocker_and_unverified_result_without_opening_a_panel() {
    let temp = fixture();
    let repo = temp.path();
    ok(
        repo,
        &[
            "task",
            "add",
            "shared",
            "--goal",
            "统一导入接口",
            "--path",
            "shared.rs",
        ],
    );
    ok(
        repo,
        &[
            "task",
            "progress",
            "shared",
            "--state",
            "blocked",
            "--note",
            "等待用户确认导入格式",
        ],
    );
    ok(
        repo,
        &[
            "task",
            "add",
            "consumer",
            "--goal",
            "接通桌面导入",
            "--depends-on",
            "shared",
            "--path",
            "desktop.rs",
        ],
    );
    fs::write(repo.join("result.md"), "已实现，相关测试待核验").unwrap();
    ok(repo, &["task", "result", "consumer", "--file", "result.md"]);
    let brief = ok(repo, &["status"]);
    for text in [
        "统一导入接口",
        "等待用户确认导入格式",
        "接通桌面导入",
        "待验收",
        "依赖：shared",
        "下一步",
        "最近回报",
        "未绑定责任窗口",
    ] {
        assert!(brief.contains(text), "missing {text}: {brief}");
    }
    assert!(!brief.contains("打开实时面板"));
    let status: Value = serde_json::from_str(&ok(repo, &["status", "--json"])).unwrap();
    let dashboard: Value = serde_json::from_str(&ok(repo, &["dashboard", "--json"])).unwrap();
    assert_eq!(status["patrol"], dashboard["patrol"]);
    assert_eq!(status["patrol"]["data"]["results"][0]["verified"], false);
    assert!(!repo.join(".git/agent-on/control/config.json").exists());
}

#[test]
fn linked_worktree_reads_the_same_project_and_a_damaged_task_source_is_not_empty() {
    let temp = fixture();
    let repo = temp.path();
    ok(
        repo,
        &[
            "task",
            "add",
            "handoff",
            "--goal",
            "交付共享接口",
            "--path",
            "shared.rs",
        ],
    );
    let linked = repo.join("linked");
    git(
        repo,
        &["worktree", "add", "-b", "feature", linked.to_str().unwrap()],
    );
    let primary: Value = serde_json::from_str(&ok(repo, &["status", "--json"])).unwrap();
    let other: Value = serde_json::from_str(&ok(&linked, &["status", "--json"])).unwrap();
    assert_eq!(primary["repo"], other["repo"]);
    assert_eq!(
        primary["patrol"]["data"]["tasks"],
        other["patrol"]["data"]["tasks"]
    );
    fs::write(
        repo.join(".git/agent-on/control/tasks/handoff.json"),
        "{broken",
    )
    .unwrap();
    let brief = ok(&linked, &["status"]);
    assert!(brief.contains("任务来源不可用"), "{brief}");
    assert!(!brief.contains("未释放任务 0"), "{brief}");
    let damaged: Value = serde_json::from_str(&ok(&linked, &["status", "--json"])).unwrap();
    assert_eq!(damaged["patrol"]["available"], false);
    assert_eq!(damaged["worktrees"].as_array().unwrap().len(), 2);
}
