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

/// `- source:` / `- **source**：` — the line that makes a `###` block a card.
fn is_source_line(line: &str) -> bool {
    Regex::new(r"^-\s*\*{0,2}source\*{0,2}\s*[:：]")
        .map(|re| re.is_match(line))
        .unwrap_or(false)
}

/// A top-level field line: `- name:` at column 0, field name optionally bold.
fn is_field_line(line: &str) -> bool {
    Regex::new(r"^-\s*\*{0,2}[^\s:：*]+\*{0,2}\s*[:：]")
        .map(|re| re.is_match(line))
        .unwrap_or(false)
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
                if cur.iter().any(|l| is_source_line(l)) {
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
        if cur.iter().any(|l| is_source_line(l)) {
            cards.push((t.clone(), start, cur.join("\n")));
        }
    }
    cards
}

/// `###` blocks that carry a `source` line in a shape the parser does not
/// accept (`* source:`, indented, …): the author tried to write a card.
/// Topic pieces whose `###` are plain section titles have no source line.
fn near_miss_blocks(text: &str) -> Vec<(String, usize)> {
    let loose = Regex::new(r"^\s*[-*+]\s*\*{0,2}\s*source\s*\*{0,2}\s*[:：]").unwrap();
    let mut out = Vec::new();
    let mut title: Option<(String, usize)> = None;
    let (mut has_loose, mut has_strict) = (false, false);
    let mut close = |t: &Option<(String, usize)>, loose_hit: bool, strict_hit: bool| {
        if let Some((name, line)) = t {
            if loose_hit && !strict_hit {
                out.push((name.clone(), *line));
            }
        }
    };
    for (i, line) in text.lines().enumerate() {
        if let Some(heading) = line.strip_prefix("### ") {
            close(&title, has_loose, has_strict);
            title = Some((heading.trim().to_string(), i + 1));
            has_loose = false;
            has_strict = false;
        } else if title.is_some() {
            has_loose |= loose.is_match(line);
            has_strict |= is_source_line(line);
        }
    }
    close(&title, has_loose, has_strict);
    out
}

/// Value of `- <name>:`. The field name may be bold (`- **name**：`). When the
/// first line is empty, the value is the indented lines under it, up to the
/// next field.
pub fn field(block: &str, name: &str) -> Option<String> {
    let re = Regex::new(&format!(
        r"^-\s*\*{{0,2}}{}\*{{0,2}}\s*[:：](.*)$",
        regex::escape(name)
    ))
    .ok()?;
    let strip = Regex::new(r"<!--.*?-->").ok()?;
    let lines: Vec<&str> = block.lines().collect();
    let (at, first) = lines
        .iter()
        .enumerate()
        .find_map(|(i, l)| re.captures(l).map(|c| (i, c[1].to_string())))?;
    let mut v = strip.replace_all(&first, "").trim().to_string();
    if v.is_empty() {
        let body: Vec<String> = lines[at + 1..]
            .iter()
            .take_while(|l| !is_field_line(l) && !l.starts_with("---"))
            .map(|l| strip.replace_all(l, "").trim().to_string())
            .filter(|l| !l.is_empty())
            .collect();
        v = body.join("\n");
    }
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
            let head = conf.split(['（', '(']).next().unwrap_or("").trim();
            if !confidence_ok().contains(&head) {
                errs.push(format!(
                    "confidence 取值非法:'{head}'(应为 high|medium|low)"
                ));
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
        let cards = split_cards(&text);
        let misses = near_miss_blocks(&text);
        let fname = path.file_name().unwrap_or_default().to_string_lossy();
        if !misses.is_empty() && cards.is_empty() {
            // Dartify 2026-09-26: a whole file of cards went invisible and the
            // lint still said "passed: 0 cards".
            total_errs += misses.len();
            out.push_str(&format!(
                "✗ {fname}  有 {} 个 ### 块写了 source,却一张卡都没认出来——字段要写成 `- source: …` 这种形式(列表符用 `-`,顶格)\n",
                misses.len()
            ));
        } else {
            for (title, line) in &misses {
                total_errs += 1;
                out.push_str(&format!(
                    "✗ {fname}:{line}  {title}\n    - 有 source 行但格式认不出,这张卡被跳过了——字段要写成 `- source: …`(列表符用 `-`,顶格)\n"
                ));
            }
        }
        for (title, start, block) in cards {
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

    /// Dartify 2026-09-26: bold field names made a whole card invisible.
    #[test]
    fn bold_field_names_still_make_a_card() {
        let text = "### slug-a（标题）\n\n- **source**：Dartify @ `abc`\n- **evidence**：现场一条\n- **confidence**：medium\n- **claim**：c\n- **suggested_landing**：l\n- **rollback**：r\n- **trace**：t\n- **状态**：pending\n";
        let cards = split_cards(text);
        assert_eq!(cards.len(), 1);
        let errs = lint_card(&cards[0].2);
        assert!(errs.is_empty(), "{errs:?}");
    }

    #[test]
    fn multiline_field_reads_its_continuation_lines() {
        let block = "- source: x\n- evidence：\n  - `git log -1` → abc123\n  - 复现命令见下\n- confidence: high\n- claim: c\n- suggested_landing:\n  1. kit/x.md\n- rollback: r\n- trace: t\n- 状态: pending";
        let errs = lint_card(block);
        assert!(errs.is_empty(), "{errs:?}");
    }

    #[test]
    fn empty_field_followed_by_next_field_is_still_empty() {
        let block = "- source: x\n- evidence:\n- confidence: high\n- claim: c\n- suggested_landing: l\n- rollback: r\n- trace: t\n- 状态: pending";
        assert!(lint_card(block).iter().any(|e| e.contains("evidence")));
    }

    #[test]
    fn headings_without_a_single_card_do_not_pass() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("2026-01-01-x.md");
        fs::write(
            &path,
            "# intake\n\n### slug-a\n\n* source: x\n* evidence: y\n\n### slug-b\n\n* source: z\n",
        )
        .unwrap();
        let (code, out) = lint_paths(&[path]);
        assert_eq!(code, 1, "{out}");
        assert!(out.contains("一张卡都没认出来"), "{out}");
    }
}
