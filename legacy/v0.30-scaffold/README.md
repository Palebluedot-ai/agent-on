# legacy/v0.30-scaffold — v0.x 重型跨仓脚手架归档

> 职责边界：保存 v0.30.0 及更早版本的跨仓方法论脚手架（`BOOTSTRAP.md`、`boot/`、`kit/`、`playbook/`、`intake/`、`ledger/`、`codex/`、`hooks/`、`skill/`、`tools/`、`scripts/`、`memory/` 与旧 `docs/`），仅供历史追溯，不再作为现行供货面维护。

2026-11-22 用户拍板将 `agent-on` 重构为 **v2.0.0 零侵入轻量工具包**（见 `snapshot/2026-11-22-v2-zero-intrusion-refactor.md`）：

- 大模型与主流宿主（Claude Code / Codex / Antigravity）已原生内化子代理编排、任务分派与 worktree 创建；跨仓锁版本（`agent-on.lock.md`）、手动结账消化台账、常驻值守（`oncall/babysit`）与粗粒度 `PreToolUse` 命令字面拦截归档至此。
- 5 个月实战提炼出的高价值核心能力已浓缩为仓库根目录的 4 个独立 Skills（`prove-it`、`thin-review`、`wt-doctor`、`merge-check`）、精简版 `agent-on` CLI 与一页全局铁律 `rules/core-guardrails.md`。
- 50 个真实翻车案例继续保留在 `bench/cases/` 供按需检索。
