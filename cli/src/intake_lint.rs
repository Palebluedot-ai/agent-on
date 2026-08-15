//! Promotion Card intake lint.

use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};

const REQUIRED: &[&str] = &[
    "source",
    "evidence",
    "confidence",
    "claim",
    "suggested_landing",
    "rollback",
    "trace",
    "状态",
];

fn confidence_ok() -> [&'static str; 3] {
    ["high", "medium", "low"]
}

pub fn split_cards(text: &str) -> Vec<(String, usize, String)> {
    let lines: Vec<&str> = text.lines().collect();
    let mut cards = Vec::new();
    let mut cur: Vec<&str> = Vec::new();
    let mut title: Option<String> = None;
    let mut start = 0usize;
    for (i, line) in lines.iter().enumerate() {
        let line_no = i + 1;
        if let Some(heading) = line.strip_prefix("### ") {
            if let Some(ref t) = title {
                if cur.iter().any(|l| l.contains("- source")) {
                    cards.push((t.clone(), start, cur.join("\n")));
                }
            }
            title = Some(heading.trim().to_string());
            start = line_no;
            cur.clear();
        } else if title.is_some() {
            cur.push(line);
        }
    }
    if let Some(ref t) = title {
        if cur.iter().any(|l| l.contains("- source")) {
            cards.push((t.clone(), start, cur.join("\n")));
        }
    }
    cards
}

pub fn field(block: &str, name: &str) -> Option<String> {
    let re = Regex::new(&format!(r"(?m)^-\s*{}\s*[:：](.*)$", regex::escape(name))).ok()?;
    let m = re.captures(block)?;
    let mut v = m.get(1)?.as_str().to_string();
    let strip = Regex::new(r"<!--.*?-->").ok()?;
    v = strip.replace_all(&v, "").trim().to_string();
    Some(v)
}

pub fn lint_card(block: &str) -> Vec<String> {
    let mut errs = Vec::new();
    for f in REQUIRED {
        match field(block, f) {
            None => errs.push(format!("缺必填字段:{f}")),
            Some(v) if v.is_empty() => {
                if *f == "evidence" {
                    errs.push("evidence 空 = 这张卡不存在(「我记得」不是证据)".into());
                } else {
                    errs.push(format!("字段 {f} 值为空"));
                }
            }
            _ => {}
        }
    }
    if let Some(conf) = field(block, "confidence") {
        if !conf.is_empty() {
            let head = conf
                .split(['（', '('])
                .next()
                .unwrap_or("")
                .trim();
            if !confidence_ok().contains(&head) {
                errs.push(format!("confidence 取值非法:'{head}'(应为 high|medium|low)"));
            }
        }
    }
    if let Some(status) = field(block, "状态") {
        if !status.is_empty() {
            let ok = Regex::new(r"(pending|landed@|rejected|deferred|半落@)").unwrap();
            if !ok.is_match(&status) {
                errs.push(format!(
                    "状态取值非法:'{}'(应含 pending | landed@… | rejected… | deferred | 半落@…)",
                    status.chars().take(30).collect::<String>()
                ));
            }
        }
    }
    errs
}

/// Lint files; returns (exit_code, stdout).
pub fn lint_paths(paths: &[PathBuf]) -> (i32, String) {
    if paths.is_empty() {
        return (
            2,
            "没有找到 intake 卡文件。用法:agent-on intake-lint intake/*.md\n".into(),
        );
    }
    let mut out = String::new();
    let mut total_cards = 0usize;
    let mut total_errs = 0usize;
    for path in paths {
        if !path.exists() {
            return (2, format!("文件不存在:{}\n", path.display()));
        }
        let text = match fs::read_to_string(path) {
            Ok(t) => t,
            Err(e) => return (2, format!("读失败 {}: {e}\n", path.display())),
        };
        for (title, start, block) in split_cards(&text) {
            total_cards += 1;
            let errs = lint_card(&block);
            if !errs.is_empty() {
                total_errs += errs.len();
                out.push_str(&format!(
                    "✗ {}:{}  {}\n",
                    path.file_name().unwrap_or_default().to_string_lossy(),
                    start,
                    title
                ));
                for e in errs {
                    out.push_str(&format!("    - {e}\n"));
                }
            }
        }
    }
    if total_errs == 0 {
        out.push_str(&format!("承接层校验通过:{total_cards} 张卡,六项齐全。\n"));
        (0, out)
    } else {
        out.push_str(&format!(
            "承接层校验不通过:{total_cards} 张卡,发现 {total_errs} 处问题(见上)。\n"
        ));
        (1, out)
    }
}

pub fn default_intake_paths(repo: &Path) -> Vec<PathBuf> {
    let dir = repo.join("intake");
    let mut v = Vec::new();
    if let Ok(rd) = fs::read_dir(&dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.extension().and_then(|x| x.to_str()) == Some("md")
                && p.file_name().and_then(|x| x.to_str()) != Some("README.md")
            {
                v.push(p);
            }
        }
    }
    v.sort();
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_evidence() {
        let block = r#"- source: x
- evidence:
- confidence: high
- claim: c
- suggested_landing: l
- rollback: r
- trace: t
- 状态: pending"#;
        let errs = lint_card(block);
        assert!(errs.iter().any(|e| e.contains("evidence")));
    }

    #[test]
    fn accepts_full_card() {
        let block = r#"- source: x
- evidence: pointer
- confidence: high
- claim: c
- suggested_landing: l
- rollback: r
- trace: t
- 状态: pending"#;
        assert!(lint_card(block).is_empty());
    }
}
