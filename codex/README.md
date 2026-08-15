# codex/ — Codex CLI 适配层

> 职责边界：让 Codex CLI 用户获得与 Claude Code 等价的 agent-on 体验。项目侧本来就是 Codex 原生的（AGENTS.md 是两家共同标准，lock / 模板 / 口令全是文件）；本目录只补**机器侧**的两个接入件。与 `../skill/`（Claude Code 适配层）互为镜像。

## 两个接入件（一台机器一次）

**路径 1 · symlink（现状默认，存量三项目零改动）**

1. **全局路由**（Codex 不读 `~/.claude/CLAUDE.md`，读 `~/.codex/AGENTS.md`）：
   把 [AGENTS-global-snippet.md](AGENTS-global-snippet.md) 的内容并入 `~/.codex/AGENTS.md`——之后任何 Codex 会话都认识「agent-on 结账 / 升级 / 整理想法 / 更新仪表盘」这些中文口令和三个入口。
2. **`$agent-on` skill**（主路，2026-07-12 起）：Codex 原生读跨工具共享目录 `~/.agents/skills/`，与 Claude 用**同一份内核**：
   ```bash
   ln -s <本仓路径>/skill ~/.agents/skills/agent-on
   ```
   之后在 Codex 里 `$agent-on <子命令>` 即触发。（旧的 `~/.codex/prompts/agent-on.md` 已降为迁移壳，v0.4 dogfood 后删除。）
3. **guard**：plugin 默认接入与 Claude 共用的 `hooks/hooks.json`；首次在 `/hooks` 检查并信任。旧 `~/.codex/hooks.json` 个人注册可删除，迁移口径见 [kit/guard/README.md](../kit/guard/README.md)。

**路径 2 · Plugin（v0.5 推荐，与 symlink 可并存）**

官方仓已 Public，远端 marketplace：

```bash
codex plugin marketplace add Palebluedot-ai/agent-on
codex plugin install agent-on@agent-on
```

仅内网/离线时：先 `git clone https://github.com/Palebluedot-ai/agent-on.git <路径>`，再  
`codex plugin marketplace add <路径>` → `codex plugin install agent-on@agent-on`。  
钉版本：clone 后 `git checkout v0.12.1`（plugin 远程装以 marketplace/manifest version 为准）。

- skill 内核与 PreToolUse guard 都经 plugin 装入；`.codex-plugin/plugin.json` 指向 Claude/Codex 共用的 canonical `hooks/hooks.json`，不存在第二份 Codex 规则。
- 项目出现并行写会话时，再在该 git 仓跑一次 `agent-on worktree hooks install`：shared `pre-commit/pre-push` 覆盖所有 linked worktree，和 Codex 是否在线无关。可选 `--daily-gc` 只装 report-only 定时报告。
- 结账需可写工作仓 **B**（任意路径）：`AGENT_ON_ROOT` 或 `~/.config/agent-on/config.json` 的 `work_root` 或 lock「本地路径」。见 `skill/SKILL.md` 与 `agent-on doctor`。
- **朋友向总入口**（含 Claude / Grok 对照）：根目录 [README.md](../README.md)「给朋友的 5 分钟装机」。

## 谁写谁读

- 写者：本仓消化会话（随版本演进）；`~/.codex/` 里的两处由用户接入时一次性挂上
- 读者：Codex 会话（AGENTS.md 每会话自动读；prompt 敲命令时读）

## 卸载

`codex plugin remove agent-on@agent-on`（若经 plugin 安装）+ `rm ~/.codex/prompts/agent-on.md`（若仍有旧 symlink）+ 从 `~/.codex/AGENTS.md` 删掉 Agent-On 一节。项目内 shared Git hooks 另在该仓执行 `agent-on worktree hooks uninstall`；它不改用户 Codex 配置。

## 诚实边界

- 斜杠命令依赖 Codex 的 prompts 约定（0.2x 起支持）；若某版本不识别，**中文口令永远是主路**（走 AGENTS.md 路由，不依赖任何命令系统）
- Claude Code 独有的能力（子代理编排、Workflow 扇出）在 Codex 下按 `playbook/workflow-orchestration.md` §四退化为手工纪律——六步协议、结账、消化全部照跑
