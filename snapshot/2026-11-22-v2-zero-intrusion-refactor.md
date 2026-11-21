# Snapshot — 2026-11-22 · v2.0.0 新时代零侵入重构（薄 Skills + 薄 CLI + 核心铁律）

> 职责边界：记录 2026-11-22 用户拍板将 Agent-On 从 v0.x 跨仓重型脚手架重构为 v2.0.0 零侵入跨客户端工具包的背景、四项架构拍板、实现与验证证据。

## 一、背景与重构判据

在 Dartify 项目的复盘会话（`659337ee-43a4-4f37-8074-5ed9274bf493`）中，对比了 2026 年 7–11 月的多项目实战演进：

1. **已被大模型与宿主内化、继续维护收益为负的部分**：
   - 基础子代理分派、任务模板与自动创建 Git worktree（Claude Code / Codex / Antigravity 均已原生内置）；
   - 固定模型档位探针与混编套餐表（随模型快速换代立即过期）；
   - 业务仓锁版本（`agent-on.lock.md`）与跨仓 `settle / digest / upgrade` 台账对账仪式；
   - 常驻值守窗口（`oncall / babysit`）与粗粒度 `PreToolUse` 命令字面拦截钩子。
2. **受限于单会话盲区、同上下文圆谎偏见与本地-真实环境断层，至今仍具高含金量的五块资产**：
   - **`prove-it` 防假绿验证**：真实业务入口核验 + 独立错误对照（故意改坏看测试红不红）+ 判别式探针 + 写入后独立回读 + 交付链自查；
   - **`thin-review` 空白上下文薄对抗复核**：无聊天历史包袱的独立子代理 + 作者已贴当前 HEAD 测试输出则免重跑全套 + 第二轮只看增量 diff；
   - **`wt-doctor` 跨客户端 Worktree 雷达与安全回收**：打通 `.claude/worktrees`、`.codex/worktrees`、`.gemini/antigravity/worktrees`，检测未提交撞车、开放 PR 重叠、落后主线发版风险，并安全回收已合并死目录；
   - **`merge-check` 高并发冲刺期合并树预演**：基于 `git merge-tree --write-tree` 在内存中零副作用检测无重叠免同步（`SYNC_FREE`）、热点文件与两两合并冲突波次；
   - **`rules/core-guardrails.md` 一页排障与追问铁律** + **`bench/cases/` 50 个真实翻车案例库**。

## 二、用户拍板的四项决策（2026-11-22）

1. **旧体系去留（宪章方向）**：彻底焕新 `CHARTER.md` 与 `AGENTS.md`，将旧 `BOOTSTRAP / boot / kit / playbook / intake / ledger / codex / hooks / skill / tools / scripts / memory / 旧 docs` 完整归档至 `legacy/v0.30-scaffold/`，保留 `bench/cases/` 50 个真实案例作参考库。
2. **可执行载体**：精简保留 Rust 单二进制 CLI（`agent-on`），移除旧台账/值守/拦截钩子模块（Rust 源码从 `19,721` 行降至 `3,269` 行，缩减 `83.4%`；移除 `regex`、`shlex`、`chrono` 外部依赖），保留 `wt`、`merge-check`、`install`、`doctor`、`build-info`，并为未清理老项目内置静默放行兼容桩（`guard`、`capture`、`worktree`）。
3. **跨客户端暴露方式**：拆分为 `prove-it`、`thin-review`、`wt-doctor`、`merge-check` 四个独立原子 Skills + 一页 `rules/core-guardrails.md`，提供根目录 `plugin.json`（Antigravity 原生插件）、`.claude-plugin`、`.codex-plugin` 及 `agent-on install` 一键三端软链。
4. **版本号与交付动作**：本地完整验证通过后提交、打 `v2.0.0` annotated tag 并推送至 `origin/main`，附 [v2.0 迁移指南](../docs/migration-v2.0.md)。

## 三、验证证据摘要

- **编译与静态检查**：`cargo fmt --check` 通过；`cargo clippy --manifest-path cli/Cargo.toml --locked --all-targets -- -D warnings` 零警告通过；最低版本 `cargo +1.89.0 check --manifest-path cli/Cargo.toml --locked --all-targets` 通过。
- **集成与单元测试**：4 套测试共 10 项用例（`source_identity` 1 项、`wt_radar` 3 项、`merge_rehearsal` 2 项、`install_and_doctor` 4 项）全部通过，覆盖跨三端 worktree 归属、非 `main` 默认分支（如 `trunk`）、中文与引号文件名、嵌套未跟踪文件与调用方 CWD 保护、Squash 分支保护、开放 PR 重叠与孤立历史分支、内存累计 `git merge-tree` 波次规划、三端隔离安装/卸载、悬空软链与非 UTF-8 文件保护、宿主 `hooks.json` / `settings.json` 旧钩子清理及旧版 `core.hooksPath` 恢复。
- **错误对照（Prove-It Counterfactual Check）**：在 `cli/src/wt.rs` 临时注入将 `LikelySquashed` 混入 `--prune` 的错误后，`wt_radar` 断言立即报红（`left: 2, right: 1`）；恢复代码后重新变绿。
- **空白上下文两轮对抗复核（Thin-Review 自举实测）**：首轮独立子代理检出 8 项边界缺陷（宿主旧 `kit/guard/agent-on-git-guard` 脚本 `exit 127` 清理、悬空软链与非 UTF-8 规则文件保护、非 `main` 默认分支 `base_oid` 比较、`--prune --repo` 调用方 CWD 保护、`--untracked-files=all`、孤立历史分支与 `--pr` 缺失校验），全部修复并补齐 4 套专项回归后，二轮增量复核 `PASSED`（0 阻塞项）。
- **CI 范围与文档不变量闸**：`python3 -m unittest discover -s .github/scripts -p 'test_*.py'` 与 `python3 .github/scripts/check_docs.py` 全绿。
