//! Shared project facts and a delivery brief. Reading never dispatches work or opens a server.
use crate::{coordination as c, landing, oncall, worktree};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

pub(crate) fn run(repo: Option<PathBuf>, json: bool) -> i32 {
    let repo =
        repo.unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    match collect(&repo) {
        Ok(value) => {
            println!(
                "{}",
                if json {
                    serde_json::to_string_pretty(&value).unwrap_or_default()
                } else {
                    render(&value)
                }
            );
            0
        }
        Err(error) => {
            eprintln!("agent-on status: {error}");
            1
        }
    }
}

fn rows(value: &Value) -> &[Value] {
    value.as_array().map(Vec::as_slice).unwrap_or(&[])
}

fn label(value: &Value) -> String {
    let text = value.as_str().unwrap_or("未知");
    let mut result: String = text
        .chars()
        .take(200)
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect();
    if text.chars().count() > 200 {
        result.push('…');
    }
    result
}

fn next_action(task: &Value) -> &'static str {
    match task["state"].as_str() {
        Some("reported") => "核验当前需求与相关检查，再按项目规则交单；回执尚不代表验收或合入",
        Some("blocked") => "处理已报告卡点，只暂停受影响的工作",
        Some("waiting") => "核对依赖或外部输入，满足条件后继续",
        Some("working") => "继续已授权工作，完成后回传本批验证证据",
        Some("planned" | "queued") if !rows(&task["dependencies"]).is_empty() => {
            "核对前置成果已合入，再推进已授权工作"
        }
        Some("planned" | "queued") => "按目标推进已授权工作",
        _ => "核对任务登记与当前责任窗口",
    }
}

fn render(value: &Value) -> String {
    let mut lines = vec![format!(
        "Agent-On · 项目简报\n{}\n观察时间：{}",
        label(&value["repo"]),
        label(&value["observed_at"])
    )];
    let oncall = &value["oncall"];
    lines.push(if oncall["available"] != true {
        format!(
            "值守：来源不可用，不能判断是否在班（{}）",
            label(&oncall["error"])
        )
    } else if oncall["data"]["present"] == true {
        format!(
            "值守：{}{}",
            label(&oncall["data"]["session"]),
            if oncall["data"]["identity_status"] == "legacy-unverified" {
                "（旧登记缺实际身份，待原负责人确认）"
            } else {
                ""
            }
        )
    } else {
        "值守：无人在班，按项目既有交付规则推进".into()
    });

    let patrol = &value["patrol"];
    if patrol["available"] != true {
        lines.push(format!(
            "任务来源不可用，不能判定为零任务：{}",
            label(&patrol["error"])
        ));
    } else {
        let data = &patrol["data"];
        let active: Vec<_> = rows(&data["tasks"])
            .iter()
            .filter(|t| {
                !matches!(t["state"].as_str(), Some("released" | "reclaimed"))
                    && t["released_at"].is_null()
            })
            .collect();
        lines.push(format!(
            "未释放任务 {} · 文件重叠 {} · 未完整取证 {}",
            active.len(),
            rows(&data["conflicts"]).len(),
            rows(&data["unknown"]).len()
        ));
        if active.is_empty() {
            lines.push("没有已登记的未释放任务；未登记窗口的工作不在此统计。".into());
        }
        for task in active.iter().take(20) {
            let stage = match task["state"].as_str() {
                Some("reported") => "待验收".into(),
                Some("blocked") => "已报卡点".into(),
                Some("waiting") => "等待输入".into(),
                Some("working") => "已报进行中".into(),
                Some("planned" | "queued") => "已登记".into(),
                _ => label(&task["state"]),
            };
            lines.push(format!(
                "\n{} · {} · {}",
                label(&task["id"]),
                stage,
                label(
                    task.get("title")
                        .filter(|v| v.is_string())
                        .unwrap_or(&task["goal"])
                )
            ));
            if task["title"].is_string() {
                lines.push(format!("  目标：{}", label(&task["goal"])));
            }
            let owners: Vec<_> = rows(&data["sessions"])
                .iter()
                .filter(|s| s["task"] == task["id"])
                .map(|s| {
                    format!(
                        "{}/{}（{}{}）",
                        label(&s["host"]),
                        label(
                            s.get("host_session_id")
                                .filter(|v| v.is_string())
                                .unwrap_or(&s["id"])
                        ),
                        label(&s["state"]),
                        if s["host_session_id"].is_string() {
                            ""
                        } else {
                            "，宿主未绑定"
                        }
                    )
                })
                .collect();
            lines.push(if owners.is_empty() {
                "  未绑定责任窗口".into()
            } else {
                format!("  窗口登记：{}", owners.join("、"))
            });
            if let Some(reporter) = task.get("last_reporter").filter(|v| v.is_object()) {
                lines.push(format!(
                    "  最近回报：{}/{}（不代表责任认领或正在执行）",
                    label(&reporter["host"]),
                    label(&reporter["session_id"])
                ));
            }
            let deps: Vec<_> = rows(&task["dependencies"]).iter().map(label).collect();
            if !deps.is_empty() {
                lines.push(format!(
                    "  依赖：{}（合入情况需取证，释放不代表已合入）",
                    deps.join("、")
                ));
            }
            if task["progress"]["note"].is_string() {
                lines.push(format!(
                    "  最近进展（{}）：{}",
                    label(&task["progress"]["at"]),
                    label(&task["progress"]["note"])
                ));
            }
            lines.push(format!("  下一步：{}", next_action(task)));
        }
        if active.len() > 20 {
            lines.push(format!(
                "另有 {} 项，完整记录见 agent-on status --json。",
                active.len() - 20
            ));
        }
        for conflict in rows(&data["conflicts"]).iter().take(5) {
            lines.push(format!(
                "重叠待协调：{} · {}",
                label(&conflict["kind"]),
                rows(&conflict["paths"])
                    .iter()
                    .map(label)
                    .collect::<Vec<_>>()
                    .join("、")
            ));
        }
    }
    let landing = &value["landing"];
    if landing["available"] == true {
        for track in rows(&landing["data"]["snapshot"]["tracks"])
            .iter()
            .filter(|t| t["pr_number"].is_number() && t["pr_state"] == "open")
            .take(20)
        {
            lines.push(format!(
                "PR #{} · {} · {}（已有快照，合入前刷新取证）",
                track["pr_number"],
                label(&track["category"]),
                label(&track["reason"])
            ));
        }
    }
    for note in rows(&value["attention"]) {
        lines.push(format!("提示：{}", label(note)));
    }
    lines.push("任务登记、自述与窗口活动分开判断；本命令不派工、不改状态、不联网。".into());
    lines.join("\n")
}

fn section(result: Result<Value, String>) -> Value {
    match result {
        Ok(value) => json!({"available":true,"data":value}),
        Err(error) => json!({"available":false,"error":error}),
    }
}

fn parsed(code: i32, text: String) -> Result<Value, String> {
    if code != 0 {
        return Err(text.trim().to_string());
    }
    serde_json::from_str(&text).map_err(|e| e.to_string())
}

/// Task stages and session readiness are not a live turn-activity feed.
/// Keep the missing observation explicit rather than publishing a misleading zero.
pub(crate) fn activity_coverage() -> Value {
    json!({"available":false,"state":"unknown","running_windows":null,
        "reason":"实时窗口活动回执未接入；任务阶段、宿主 ready 回执和进程存在均不能证明当前在执行。"})
}

/// Reads only the primary/common-dir facts, so linked worktrees see the same project.
pub(crate) fn collect(repo: &Path) -> Result<Value, String> {
    let store = c::Store::open(repo)?;
    let common = worktree::common_git_dir(&store.repo)?;
    let config = section(
        store
            .config()
            .and_then(|v| serde_json::to_value(v).map_err(|e| e.to_string())),
    );
    let mut patrol = section(c::scan(&store, None));
    if let Some(sessions) = patrol["data"]["sessions"].as_array_mut() {
        for session in sessions {
            let age = session["heartbeat_at"]
                .as_str()
                .and_then(|at| chrono::DateTime::parse_from_rfc3339(at).ok())
                .map(|at| {
                    (chrono::Utc::now() - at.with_timezone(&chrono::Utc))
                        .num_seconds()
                        .max(0)
                });
            let pids: [Option<u32>; 2] = [
                session["pid"].as_u64().map(|p| p as u32),
                session["child_pid"].as_u64().map(|p| p as u32),
            ];
            session["heartbeat_age_seconds"] = json!(age);
            session["process_alive"] = json!(if pids.iter().any(Option::is_some) {
                Some(pids.into_iter().flatten().any(c::pid_alive))
            } else {
                None
            });
        }
    }
    let raw = c::git(&store.repo, &["worktree", "list", "--porcelain"])?;
    let mut trees = Vec::new();
    let tasks = store.tasks();
    for (i, tree) in worktree::parse_worktrees(&raw).into_iter().enumerate() {
        let facts = (|| -> Result<Value, String> {
            let status = c::git(&tree.path, &["status", "--porcelain=v1", "-z"])?;
            let unique = c::git(
                &tree.path,
                &["rev-list", "--count", "HEAD", "--not", "--remotes"],
            )?;
            Ok(json!({"clean":status.is_empty(),"local_only_commits":unique.parse::<u64>().ok()}))
        })();
        trees.push(json!({"path":tree.path,"head":tree.head,"branch":tree.branch,
            "primary":i==0,"locked":tree.locked,"prunable":tree.prunable,"facts":section(facts),
            "managed":tasks.as_ref().ok().map(|tasks|tasks.iter().any(|t|t.managed && t.worktree==tree.path))}));
    }
    let (code, output) = oncall::status(&store.repo, true);
    let oncall = section(parsed(code, output));
    let opts = landing::LandingOpts {
        json: true,
        ..Default::default()
    };
    let (code, output) = landing::run_status(&store.repo, &opts);
    let landing = section(parsed(code, output));
    for tree in &mut trees {
        if let Some(track) = landing["data"]["snapshot"]["tracks"]
            .as_array()
            .into_iter()
            .flatten()
            .find(|track| track["worktree"] == tree["path"])
        {
            tree["lifecycle"] = track["lifecycle"].clone();
            tree["lifecycle_reason"] = track["lifecycle_reason"].clone();
        }
    }
    let snapshot = landing::load_snapshot(&common).ok().flatten();
    let age = snapshot
        .as_ref()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s.generated_at).ok())
        .map(|at| {
            (chrono::Utc::now() - at.with_timezone(&chrono::Utc))
                .num_seconds()
                .max(0)
        });
    let repo_head = c::git(&store.repo, &["rev-parse", "HEAD"])?;
    let mut attention = Vec::new();
    if config["data"]["enabled"] != true {
        attention.push("巡逻未启用：现存 Git 树可见，宿主任务与会话不保证自动捕获。".to_string());
    }
    if !landing["available"].as_bool().unwrap_or(false) {
        attention.push("尚无可用 PR 快照；需要合入取证时运行 agent-on landing refresh。".into());
    } else if age.is_none_or(|n| n > 120) {
        attention.push("PR 取证已超过两分钟；这是上次观察，不能据此直接决定合并。".into());
    }
    let hooks = worktree::git(&store.repo, &["config", "--get", "core.hooksPath"]).ok();
    if hooks.is_none() {
        attention.push("未设置原生 Git hooks；需要 agent-on worktree hooks install。".into());
    }
    let daily_gc = crate::worktree_schedule::inspect(&store.repo);
    if daily_gc.installed && !daily_gc.healthy() {
        attention.push(crate::worktree_schedule::render_status(&daily_gc));
    }
    let daily_gc = section(serde_json::to_value(daily_gc).map_err(|error| error.to_string()));
    Ok(
        json!({"schema_version":1,"observed_at":c::now(),"repo":store.repo,"head":repo_head,
        "config":config,"patrol":patrol,"oncall":oncall,"landing":landing,"activity":crate::activity::collect_dir(&store.dir),
        "landing_age_seconds":age,"worktrees":trees,"attention":attention,
        "hooks_path":hooks,"daily_gc":daily_gc,
        "coverage":"本机 Git 事实与已有回执；Stop/退出/结果自述不等于验证通过或任务释放"}),
    )
}
