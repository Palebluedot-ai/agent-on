//! Current checkout facts and existing delivery evidence. No task store, observer or server.
use crate::{landing, oncall, state_io, worktree};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;

pub(crate) fn run(repo: Option<PathBuf>, json_output: bool) -> i32 {
    let repo =
        repo.unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    match collect(&repo) {
        Ok(value) => {
            println!(
                "{}",
                if json_output {
                    serde_json::to_string_pretty(&value).unwrap()
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
fn section(code: i32, output: String) -> Value {
    if code != 0 {
        return json!({"available":false,"error":output.trim()});
    }
    match serde_json::from_str::<Value>(&output) {
        Ok(data) => json!({"available":true,"data":data}),
        Err(error) => json!({"available":false,"error":error.to_string()}),
    }
}
pub(crate) fn collect(repo: &Path) -> Result<Value, String> {
    let repo = worktree::repo_root(repo)?;
    let common = worktree::common_git_dir(&repo)?;
    let out = Command::new("git")
        .current_dir(&repo)
        .env("GIT_OPTIONAL_LOCKS", "0")
        .args(["status", "--porcelain=v1", "-z"])
        .output()
        .map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).into());
    }
    let dirty = !out.stdout.is_empty();
    let (code, output) = oncall::status(&repo, true);
    let oncall = section(code, output);
    let (code, output) = landing::run_status(
        &repo,
        &landing::LandingOpts {
            json: true,
            ..Default::default()
        },
    );
    let landing = section(code, output);
    let age = landing::load_snapshot(&common)
        .ok()
        .flatten()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s.generated_at).ok())
        .map(|t| {
            (chrono::Utc::now() - t.with_timezone(&chrono::Utc))
                .num_seconds()
                .max(0)
        });
    // Point to existing project records; never infer completion or manufacture tasks.
    let records: Vec<_> = [
        "AGENTS.md",
        "project-state.md",
        "status.md",
        "state.md",
        "thoughts-and-ideas.md",
    ]
    .iter()
    .filter(|name| repo.join(name).is_file())
    .copied()
    .collect();
    Ok(
        json!({"schema_version":2,"observed_at":state_io::now(),"repo":repo,
        "head":worktree::git(&repo,&["rev-parse","--verify","HEAD"]).ok(),
        "branch":worktree::git(&repo,&["symbolic-ref","--quiet","--short","HEAD"]).ok(),
        "dirty":dirty,"goal":null,"progress":null,"project_records":records,
        "oncall":oncall,"landing":landing,"landing_age_seconds":age,
        "next_action":if dirty {"核验当前修改与验收，再按项目规则交付"} else {"沿当前对话目标与项目记录继续"},
        "coverage":"当前 checkout 的 Git 事实和已有合入快照；不联网，不推断任务完成或其他窗口活动"}),
    )
}
fn safe(value: &Value) -> String {
    value
        .as_str()
        .unwrap_or("未知")
        .chars()
        .take(300)
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect()
}
fn render(v: &Value) -> String {
    let mut lines = vec![format!(
        "Agent-On · 项目简报\n{}\n分支：{} · {}",
        safe(&v["repo"]),
        safe(&v["branch"]),
        if v["dirty"] == true {
            "有未提交修改"
        } else {
            "工作区干净"
        }
    )];
    lines.push("目标／完成进度：由当前对话与项目记录确认，Git 状态不代表任务完成。".into());
    lines.push(format!(
        "项目记录：{}",
        v["project_records"]
            .as_array()
            .unwrap()
            .iter()
            .map(safe)
            .collect::<Vec<_>>()
            .join("、")
    ));
    let o = &v["oncall"];
    lines.push(if o["available"] != true {
        format!("值守：来源不可用（{}）", safe(&o["error"]))
    } else if o["data"]["present"] == true {
        format!("值守：{}", safe(&o["data"]["session"]))
    } else {
        "值守：无人在班，按项目既有规则交付".into()
    });
    let l = &v["landing"];
    if l["available"] == true {
        for t in l["data"]["snapshot"]["tracks"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|t| t["pr_state"] == "open")
            .take(20)
        {
            lines.push(format!(
                "PR #{} · {} · {}（缓存，合入前刷新）",
                t["pr_number"],
                safe(&t["category"]),
                safe(&t["reason"])
            ));
        }
    } else {
        lines.push("尚无可用 PR 快照；合入时由 landing refresh 取证。".into());
    }
    if let Some(age) = v["landing_age_seconds"].as_i64() {
        lines.push(format!("PR 快照距今 {age} 秒，不代表当前 CI 结果。"));
    }
    lines.push(format!("下一步：{}", safe(&v["next_action"])));
    lines.join("\n")
}
