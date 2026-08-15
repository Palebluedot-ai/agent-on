//! Open-box skill-routing / demotion protocol checks.

use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};

fn read(path: &Path) -> Result<String, String> {
    if !path.is_file() {
        return Err(format!("missing file: {}", path.display()));
    }
    fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))
}

fn must_contain(path: &Path, text: &str, patterns: &[&str]) -> Result<(), String> {
    for pat in patterns {
        let re = Regex::new(&format!("(?im){pat}")).map_err(|e| e.to_string())?;
        if !re.is_match(text) {
            return Err(format!("{}: missing /{pat}/", path.display()));
        }
    }
    Ok(())
}

fn must_not_contain(path: &Path, text: &str, patterns: &[&str]) -> Result<(), String> {
    for pat in patterns {
        let re = Regex::new(&format!("(?im){pat}")).map_err(|e| e.to_string())?;
        if re.is_match(text) {
            return Err(format!("{}: forbidden /{pat}/", path.display()));
        }
    }
    Ok(())
}

pub fn check_agent_on(root: &Path) -> Result<String, String> {
    let lite = root.join("kit/AGENTS-lite.md");
    let skel = root.join("kit/AGENTS-skeleton.md");
    let bootstrap = root.join("BOOTSTRAP.md");
    let readme = root.join("README.md");
    let ledger = root.join("ledger/run-card-logging.md");
    let mrd = root.join("snapshot/2026-08-02-light-hard-premium-mrd.md");
    let adopt = root.join("boot/adopt.md");
    let schemas = root.join("kit/schemas/README.md");
    let phase_gates = root.join("playbook/phase-gates.md");
    let audit = root.join("snapshot/2026-08-03-research-residual-audit.md");
    let skill = root.join("skill/SKILL.md");
    let worktree = root.join("kit/worktree-control-plane.md");

    let lite_t = read(&lite)?;
    let skel_t = read(&skel)?;
    let boot_t = read(&bootstrap)?;
    let readme_t = read(&readme)?;
    let ledger_t = read(&ledger)?;
    let mrd_t = read(&mrd)?;
    let adopt_t = read(&adopt)?;
    let schemas_t = read(&schemas)?;
    let phase_t = read(&phase_gates)?;
    let audit_t = read(&audit)?;
    let skill_t = read(&skill)?;
    let worktree_t = read(&worktree)?;

    must_contain(
        &lite,
        &lite_t,
        &[
            r"制度在 agent-on",
            r"不默认 Superpowers",
            r"brainstorming",
            r"subagent-driven-development",
        ],
    )?;
    must_not_contain(
        &skel,
        &skel_t,
        &[
            r"实现执行\s*\|\s*\[?如 Superpowers subagent-driven-development",
            r"实现执行\s*\|\s*Superpowers",
        ],
    )?;
    must_contain(
        &skel,
        &skel_t,
        &[
            r"不.*默认 Superpowers|不默认.*subagent-driven-development",
            r"主会话",
            r"brainstorming",
        ],
    )?;
    must_contain(
        &bootstrap,
        &boot_t,
        &[
            r"默认心态偏 S|拿不准取低档",
            r"不默认 Superpowers",
            r"制度",
            r"adopt\.md.*§三|降档.*§三|§三.*降档",
        ],
    )?;
    must_contain(
        &readme,
        &readme_t,
        &[
            r"默认心态偏 S|拿不准取低档",
            r"旁路|尚未在真实项目验证|零真实项目跑通|开箱勿启用",
            r"Superpowers 退出默认|不默认 Superpowers",
        ],
    )?;
    must_contain(
        &ledger,
        &ledger_t,
        &[r"旁路", r"未.*真实项目跑通|尚未在真实项目验证"],
    )?;
    must_contain(&mrd, &mrd_t, &[r"B1", r"C1", r"已拍板"])?;
    must_contain(
        &adopt,
        &adopt_t,
        &[
            r"## 三、降档",
            r"禁止静默降档",
            r"只删不用的件",
            r"不重播",
            r"local_deviations",
            r"显式批准",
            r"agent-on worktree status",
            r"rescue",
            r"逐棵补 claim",
        ],
    )?;
    must_contain(
        &skill,
        &skill_t,
        &[
            r"argument-hint:.*worktree",
            r"`worktree`.*worktree-control-plane",
            r"agent-on worktree status",
            r"无 worktree 删除命令",
        ],
    )?;
    must_contain(
        &worktree,
        &worktree_t,
        &[
            r"common git dir|git common dir",
            r"worktree claim",
            r"worktree check",
            r"set-status",
            r"primary",
            r"rescue",
            r"不提供自动删除命令",
        ],
    )?;
    must_contain(
        &schemas,
        &schemas_t,
        &[
            r"旁路",
            r"开箱.*勿启用|勿启用",
            r"未.*真实项目跑通|尚未在真实项目验证",
        ],
    )?;
    must_contain(
        &phase_gates,
        &phase_t,
        &[
            r"每轮口令复述",
            r"不要照搬|扔掉的是.*形式|禁止复活",
            r"fail-closed",
        ],
    )?;
    must_not_contain(
        &phase_gates,
        &phase_t,
        &[
            r"开箱默认.*每轮.*Global\s*/\s*In-Scope",
            r"推荐路径.*每轮口令复述硬拦",
        ],
    )?;
    must_contain(
        &audit,
        &audit_t,
        &[
            r"三不变量|完成=证据|单一状态写者",
            r"jsonl|旁路",
            r"锁口令|每轮口令复述",
            r"下次顺手|降档协议",
        ],
    )?;

    Ok(format!(
        "OK agent-on: {} {} {} {} {} {} {} {} {} {} {} {}",
        lite.file_name().unwrap().to_string_lossy(),
        skel.file_name().unwrap().to_string_lossy(),
        bootstrap.file_name().unwrap().to_string_lossy(),
        readme.file_name().unwrap().to_string_lossy(),
        ledger.file_name().unwrap().to_string_lossy(),
        mrd.file_name().unwrap().to_string_lossy(),
        adopt.file_name().unwrap().to_string_lossy(),
        schemas.file_name().unwrap().to_string_lossy(),
        phase_gates.file_name().unwrap().to_string_lossy(),
        audit.file_name().unwrap().to_string_lossy(),
        skill.file_name().unwrap().to_string_lossy(),
        worktree.file_name().unwrap().to_string_lossy(),
    ))
}

pub fn check_agent_memory(home: &Path) -> Result<String, String> {
    let claude = home.join("agent-memory/dotfiles/claude/CLAUDE.md");
    let routing = home.join("agent-memory/memory/project_skill_routing.md");
    let mem = home.join("agent-memory/memory/MEMORY.md");
    let claude_t = read(&claude)?;
    let routing_t = read(&routing)?;
    let mem_t = read(&mem)?;
    must_contain(
        &claude,
        &claude_t,
        &[
            r"不默认 Superpowers|退出默认",
            r"agent-on.*制度|制度层",
            r"完成.*=.*验证|贴验证|实际输出",
            r"brainstorming",
        ],
    )?;
    must_not_contain(
        &claude,
        &claude_t,
        &[
            r"用 Superpowers subagent-driven-development 的执行引擎",
            r"Implementation → Superpowers subagent-driven-development",
        ],
    )?;
    must_contain(
        &routing,
        &routing_t,
        &[
            r"Superpowers",
            r"not default|不默认|off default|退出默认",
            r"agent-on",
        ],
    )?;
    must_not_contain(&mem, &mem_t, &[r"impl=Superpowers"])?;
    must_contain(
        &mem,
        &mem_t,
        &[
            r"project_skill_routing\.md",
            r"Superpowers off default|off default|不默认",
        ],
    )?;
    Ok(format!(
        "OK agent-memory: {} {} {}",
        claude.display(),
        routing.file_name().unwrap().to_string_lossy(),
        mem.file_name().unwrap().to_string_lossy()
    ))
}

pub fn run_check(repo: &Path, with_memory: bool, home: Option<PathBuf>) -> (i32, String) {
    match check_agent_on(repo) {
        Ok(mut msg) => {
            msg.push('\n');
            if with_memory {
                let h = home.unwrap_or_else(|| {
                    std::env::var_os("HOME")
                        .map(PathBuf::from)
                        .unwrap_or_else(|| PathBuf::from("."))
                });
                match check_agent_memory(&h) {
                    Ok(m) => {
                        msg.push_str(&m);
                        msg.push('\n');
                    }
                    Err(e) => return (1, format!("FAIL: {e}\n")),
                }
            }
            msg.push_str("ALL CHECKS PASSED\n");
            (0, msg)
        }
        Err(e) => (1, format!("FAIL: {e}\n")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn fails_when_lite_missing_phrase() {
        let d = tempdir().unwrap();
        // minimal tree missing required phrases
        fs::create_dir_all(d.path().join("kit")).unwrap();
        fs::write(d.path().join("kit/AGENTS-lite.md"), "empty").unwrap();
        let r = check_agent_on(d.path());
        assert!(r.is_err());
    }
}
