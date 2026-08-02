//! Audit event jsonl state-machine lint.

use chrono::DateTime;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

fn allowed() -> HashMap<&'static str, HashSet<&'static str>> {
    let mut m = HashMap::new();
    m.insert("none", ["queued"].into_iter().collect());
    m.insert("queued", ["triaged", "failed"].into_iter().collect());
    m.insert("triaged", ["planned", "failed"].into_iter().collect());
    m.insert("planned", ["running", "failed"].into_iter().collect());
    m.insert("running", ["reviewing", "failed"].into_iter().collect());
    m.insert("reviewing", ["done", "failed"].into_iter().collect());
    m.insert("done", HashSet::new());
    m.insert("failed", HashSet::new());
    m
}

const REQUIRED: &[&str] = &[
    "run_id",
    "task_id",
    "timestamp",
    "state_from",
    "state_to",
    "actor_role",
    "action",
];

pub fn lint_text(text: &str) -> (usize, Vec<(usize, String)>) {
    let mut events = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let line_no = i + 1;
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<Value>(line) {
            Ok(v) => events.push((line_no, v)),
            Err(e) => events.push((
                line_no,
                serde_json::json!({"_parse_error": e.to_string()}),
            )),
        }
    }

    let allowed = allowed();
    let mut errors = Vec::new();
    let mut last_state: HashMap<(String, String), String> = HashMap::new();

    for (line_no, ev) in &events {
        if let Some(pe) = ev.get("_parse_error") {
            errors.push((
                *line_no,
                format!("JSON 解析失败:{}", pe.as_str().unwrap_or("?")),
            ));
            continue;
        }
        let keys: HashSet<&str> = ev
            .as_object()
            .map(|o| o.keys().map(|k| k.as_str()).collect())
            .unwrap_or_default();
        let miss: Vec<_> = REQUIRED.iter().filter(|k| !keys.contains(*k)).copied().collect();
        if !miss.is_empty() {
            errors.push((*line_no, format!("缺必填字段:{}", miss.join(", "))));
            continue;
        }

        let ts = ev["timestamp"].as_str().unwrap_or("");
        let ts_norm = ts.replace('Z', "+00:00");
        if DateTime::parse_from_rfc3339(&ts_norm).is_err() {
            // also try chrono flexible
            if DateTime::parse_from_str(ts, "%Y-%m-%dT%H:%M:%S%.f%z").is_err()
                && DateTime::parse_from_str(&ts_norm, "%Y-%m-%dT%H:%M:%S%z").is_err()
            {
                // from_isoformat style: 2026-07-07T12:00:00Z already tried
                errors.push((
                    *line_no,
                    format!("时间戳格式非法(需 ISO-8601):{ts}"),
                ));
            }
        }

        if ev["state_from"].is_null() {
            errors.push((
                *line_no,
                "state_from 是 null:按 schema(audit-event.schema.json)应为字符串,首个事件请写 \"none\""
                    .into(),
            ));
            continue;
        }

        let run_id = ev["run_id"].as_str().unwrap_or("").to_string();
        let task_id = ev["task_id"].as_str().unwrap_or("").to_string();
        let state_from = ev["state_from"].as_str().unwrap_or("").to_string();
        let state_to = ev["state_to"].as_str().unwrap_or("").to_string();
        let key = (run_id, task_id);
        let prev = last_state
            .get(&key)
            .cloned()
            .unwrap_or_else(|| "none".into());

        if state_from != prev {
            errors.push((
                *line_no,
                format!("state_from 断裂:该 task 上一状态应为 '{prev}',本行却写 '{state_from}'"),
            ));
        }

        let allowed_next = allowed.get(state_from.as_str()).cloned().unwrap_or_default();
        if !allowed_next.contains(state_to.as_str()) {
            let legal = if allowed_next.is_empty() {
                "(终态,不可再迁移)".into()
            } else {
                let mut v: Vec<_> = allowed_next.iter().copied().collect();
                v.sort();
                v.join(", ")
            };
            errors.push((
                *line_no,
                format!(
                    "非法跳转:'{state_from}' → '{state_to}';从 '{state_from}' 只允许迁到 {legal}"
                ),
            ));
        } else {
            last_state.insert(key, state_to.clone());
        }

        if state_to == "failed" {
            let et = ev.get("error_type").and_then(|v| v.as_str()).unwrap_or("");
            if et.is_empty() {
                errors.push((*line_no, "迁到 failed 却未记 error_type".into()));
            }
        }
    }

    (events.len(), errors)
}

pub fn lint_file(path: &Path) -> (i32, String) {
    if !path.exists() {
        return (2, format!("文件不存在:{}\n", path.display()));
    }
    let text = match fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => return (2, format!("读失败: {e}\n")),
    };
    let (n, errors) = lint_text(&text);
    if errors.is_empty() {
        (
            0,
            format!("审计通过:{n} 个事件,状态机全合法。\n"),
        )
    } else {
        let mut out = format!("审计不通过:{n} 个事件,发现 {} 处问题:\n", errors.len());
        for (line_no, msg) in errors {
            out.push_str(&format!("  第 {line_no} 行:{msg}\n"));
        }
        (1, out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legal_chain_passes() {
        let text = r#"{"run_id":"r1","task_id":"t1","timestamp":"2026-07-07T12:00:00Z","state_from":"none","state_to":"queued","actor_role":"orchestrator","action":"enqueue"}
{"run_id":"r1","task_id":"t1","timestamp":"2026-07-07T12:01:00Z","state_from":"queued","state_to":"triaged","actor_role":"orchestrator","action":"triage"}
"#;
        let (n, e) = lint_text(text);
        assert_eq!(n, 2);
        assert!(e.is_empty(), "{e:?}");
    }

    #[test]
    fn illegal_jump_fails() {
        let text = r#"{"run_id":"r1","task_id":"t1","timestamp":"2026-07-07T12:00:00Z","state_from":"none","state_to":"queued","actor_role":"o","action":"a"}
{"run_id":"r1","task_id":"t1","timestamp":"2026-07-07T12:01:00Z","state_from":"queued","state_to":"running","actor_role":"o","action":"a"}
"#;
        let (_, e) = lint_text(text);
        assert!(e.iter().any(|(_, m)| m.contains("非法跳转")));
    }
}
