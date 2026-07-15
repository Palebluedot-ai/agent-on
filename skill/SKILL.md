---
name: agent-on
description: agent-on 方法论脚手架的显式调用入口。/agent-on <子命令>——init 新项目初始化 · adopt 接管已开工项目 · handshake 会话续接 · settle 结账回流 · digest 消化 · upgrade 升级 pin。按子命令读取 agent-on 仓对应文件并逐步执行。
disable-model-invocation: true
argument-hint: [init|adopt|handshake|settle|digest|upgrade]
---

# agent-on 调用入口

> **本文件是唯一内核**：Claude Code 经 `/agent-on`、Codex 经 `$agent-on` 调用的是**同一份本文件**（symlink 挂 `~/.claude/skills/agent-on` / `~/.agents/skills/agent-on`，或经 plugin 装入）。改路由只改这里，别造第二份入口。

用户敲了 `/agent-on $ARGUMENTS`（或 Codex 的 `$agent-on …`）。按参数的第一个词查下表，读对应文件，**完整按其中的指令逐步执行**（那些文件是自包含执行书，本 skill 只负责把你导到正确的一份）。

## 先解析 agent-on 根路径（`$ROOT`）

按序取第一个「目录存在且含 `CHARTER.md` + `BOOTSTRAP.md`」的路径，记为 `$ROOT`：

1. 环境变量 `AGENT_ON_ROOT`（显式可写工作仓；结账 intake 必须写这里）
2. `CLAUDE_PLUGIN_ROOT` 或 `PLUGIN_ROOT`（plugin 安装缓存——**只读**模板与执行书够用；**不要**往这里写 intake）
3. 默认 `~/Projects/Agent-On`（本机常见 clone 布局）

若 1 与 3 都不存在、只有 plugin 缓存：`init` / `adopt` / `handshake` / `upgrade` 可读缓存继续；**`settle` 必须先有可写 clone**（提示用户 `git clone` 本仓并设 `AGENT_ON_ROOT`，或把 clone 放到默认路径）——结账写入面不能是会随 plugin 更新被覆盖的 cache。

| 子命令 | 读这份文件 | 做什么 | 在哪个仓的会话执行 |
|---|---|---|---|
| `init` | `$ROOT/BOOTSTRAP.md` | 新项目从零初始化（定档三问 → 播骨架） | 目标项目仓 |
| `adopt` | `$ROOT/boot/adopt.md` | 接管一个已开工的项目（考古 → 定档 → 增量补件） | 目标项目仓 |
| `handshake` | `$ROOT/boot/session-handshake.md` | 同一项目换会话续跑的三步握手 | 目标项目仓 |
| `settle` | `$ROOT/boot/settlement.md`（上半场「结账」） | 把本项目攒的教训回流进 agent-on 的 intake/ 承接层 | 目标项目仓；**intake 写入可写 `$ROOT`（优先 `AGENT_ON_ROOT` / clone，禁止写 plugin cache）** |
| `digest` | `$ROOT/boot/settlement.md`（下半场「消化」） | 把 intake/ 的卡分诊、落成 playbook/kit 的具体修改、打 tag | **必须在可写 agent-on 工作仓会话**——若当前不在该仓，先提醒用户新开或 cd 过去，不要跨仓改 canonical |
| `upgrade` | `$ROOT/boot/settlement.md`（「升级」节） | 读 CHANGELOG 区间，bump 项目 lock 的 pin | 目标项目仓 |

## 规则

- **空参数**：读 `$ROOT/README.md` 的指令速查表 + 上面的子命令表，一句话列给用户看，问他要哪个。
- **项目根没有 `agent-on.lock.md`**（未接入项目）：自己判断是全新（目录近空/无代码史）还是已开工存量——全新走 `init`，存量走 `adopt`，判断结果报一句即可，别让用户选入口。
- **对表/升级时的诚实播报**：若可写工作仓 HEAD 领先最新 tag（存在未发布 commit），播报「另有未发布变化 N 个 commit」——未发布不构成可升级版本，别伪装。
- **认不出的子命令**：别猜，把上表列出来让用户重选。
- 读到目标文件后**照它执行**，不要在这里复述或改写它的步骤——它是唯一权威（truth-hierarchy 的 Canonical 层）。
- 本 skill 只是入口壳；说中文口令「agent-on 结账 / 升级」走全局规则路由是等价的另一条路，两条都通。
