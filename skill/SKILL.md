---
name: agent-on
description: agent-on 方法论脚手架的显式调用入口。/agent-on <子命令>——init 新项目初始化 · adopt 接管已开工项目 · handshake 会话续接 · settle 结账回流 · digest 消化 · upgrade 升级 pin。按子命令读取 agent-on 仓对应文件并逐步执行。
disable-model-invocation: true
argument-hint: [init|adopt|handshake|settle|digest|upgrade]
---

# agent-on 调用入口

用户敲了 `/agent-on $ARGUMENTS`。按 `$ARGUMENTS` 的第一个词查下表，读对应文件，**完整按其中的指令逐步执行**（那些文件是自包含执行书，本 skill 只负责把你导到正确的一份）。agent-on 仓在本机 `~/Projects/Agent-On/`。

| 子命令 | 读这份文件 | 做什么 | 在哪个仓的会话执行 |
|---|---|---|---|
| `init` | `~/Projects/Agent-On/BOOTSTRAP.md` | 新项目从零初始化（定档三问 → 播骨架） | 目标项目仓 |
| `adopt` | `~/Projects/Agent-On/boot/adopt.md` | 接管一个已开工的项目（考古 → 定档 → 增量补件） | 目标项目仓 |
| `handshake` | `~/Projects/Agent-On/boot/session-handshake.md` | 同一项目换会话续跑的三步握手 | 目标项目仓 |
| `settle` | `~/Projects/Agent-On/boot/settlement.md`（上半场「结账」） | 把本项目攒的教训回流进 agent-on 的 intake/ 承接层 | 目标项目仓 |
| `digest` | `~/Projects/Agent-On/boot/settlement.md`（下半场「消化」） | 把 intake/ 的卡分诊、落成 playbook/kit 的具体修改、打 tag | **必须在 agent-on 仓会话**——若当前不在 `~/Projects/Agent-On/`，先提醒用户新开或 cd 过去，不要跨仓改 canonical |
| `upgrade` | `~/Projects/Agent-On/boot/settlement.md`（「升级」节） | 读 CHANGELOG 区间，bump 项目 lock 的 pin | 目标项目仓 |

## 规则

- **空参数**：读 `~/Projects/Agent-On/README.md` 的「现在怎么用」表 + 上面的子命令表，一句话列给用户看，问他要哪个。
- **认不出的子命令**：别猜，把上表列出来让用户重选。
- 读到目标文件后**照它执行**，不要在这里复述或改写它的步骤——它是唯一权威（truth-hierarchy 的 Canonical 层）。
- 本 skill 只是入口壳；说中文口令「agent-on 结账 / 升级」走全局 CLAUDE.md 路由是等价的另一条路，两条都通。
