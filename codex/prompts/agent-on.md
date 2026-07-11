# agent-on 调用入口（Codex 版）

用户敲了 `/agent-on`，可能在后面带了一个子命令词（init / adopt / handshake / settle / digest / upgrade）。**在用户消息里找这个词**，按下表读对应文件并完整照其中指令逐步执行（那些文件是自包含执行书，本 prompt 只负责导航）。agent-on 仓在本机 `~/Projects/Agent-On/`（若用户 clone 在别处，以项目根 `agent-on.lock.md` 的「本地路径」行为准）。

| 子命令 | 读这份文件 | 做什么 | 在哪个仓的会话执行 |
|---|---|---|---|
| `init` | `~/Projects/Agent-On/BOOTSTRAP.md` | 新项目从零初始化（定档三问 → 播骨架） | 目标项目仓 |
| `adopt` | `~/Projects/Agent-On/boot/adopt.md` | 接管一个已开工的项目（考古 → 定档 → 增量补件） | 目标项目仓 |
| `handshake` | `~/Projects/Agent-On/boot/session-handshake.md` | 同一项目换会话续跑的三步握手 | 目标项目仓 |
| `settle` | `~/Projects/Agent-On/boot/settlement.md`（上半场「结账」） | 把本项目攒的教训回流进 agent-on 的 intake/ 承接层 | 目标项目仓 |
| `digest` | `~/Projects/Agent-On/boot/settlement.md`（下半场「消化」） | 把 intake/ 的卡分诊、落成 playbook/kit 修改、打 tag | **必须在 agent-on 仓会话**——若当前不在，先提醒用户切过去，不要跨仓改 canonical |
| `upgrade` | `~/Projects/Agent-On/boot/settlement.md`（「升级」节） | 读 CHANGELOG 区间，bump 项目 lock 的 pin | 目标项目仓 |

## 规则

- **没带子命令**：读 `~/Projects/Agent-On/README.md` 的「现在怎么用」表，把三入口两口令一句话列给用户，问他要哪个
- **认不出的词**：别猜，列出上表让用户重选
- 读到目标文件后**照它执行**，不要复述或改写它的步骤——它是唯一权威
- 中文口令（「agent-on 结账」等）与本命令等价，两条路都通
