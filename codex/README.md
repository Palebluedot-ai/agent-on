# codex/ — Codex CLI 适配层

> 职责边界：让 Codex CLI 用户获得与 Claude Code 等价的 agent-on 体验。项目侧本来就是 Codex 原生的（AGENTS.md 是两家共同标准，lock / 模板 / 口令全是文件）；本目录只补**机器侧**的两个接入件。与 `../skill/`（Claude Code 适配层）互为镜像。

## 两个接入件（一台机器一次）

1. **全局路由**（Codex 不读 `~/.claude/CLAUDE.md`，读 `~/.codex/AGENTS.md`）：
   把 [AGENTS-global-snippet.md](AGENTS-global-snippet.md) 的内容并入 `~/.codex/AGENTS.md`——之后任何 Codex 会话都认识「agent-on 结账 / 升级 / 整理想法 / 更新仪表盘」这些中文口令和三个入口。
2. **`$agent-on` skill**（主路，2026-07-12 起）：Codex 原生读跨工具共享目录 `~/.agents/skills/`，与 Claude 用**同一份内核**：
   ```bash
   ln -s <本仓路径>/skill ~/.agents/skills/agent-on
   ```
   之后在 Codex 里 `$agent-on <子命令>` 即触发。（旧的 `~/.codex/prompts/agent-on.md` 已降为迁移壳，v0.4 dogfood 后删除。）

## 谁写谁读

- 写者：本仓消化会话（随版本演进）；`~/.codex/` 里的两处由用户接入时一次性挂上
- 读者：Codex 会话（AGENTS.md 每会话自动读；prompt 敲命令时读）

## 卸载

`rm ~/.codex/prompts/agent-on.md`（只删 symlink）+ 从 `~/.codex/AGENTS.md` 删掉 Agent-On 一节。

## 诚实边界

- 斜杠命令依赖 Codex 的 prompts 约定（0.2x 起支持）；若某版本不识别，**中文口令永远是主路**（走 AGENTS.md 路由，不依赖任何命令系统）
- Claude Code 独有的能力（子代理编排、Workflow 扇出）在 Codex 下按 `playbook/workflow-orchestration.md` §四退化为手工纪律——六步协议、结账、消化全部照跑
