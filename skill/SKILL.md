---
name: agent-on
description: Agent-On 项目脚手架入口：初始化与续接、经验回流、可选独立巡逻窗口和跨模型派工。通过 /agent-on 或 $agent-on 显式调用，按目标读取对应执行书。
disable-model-invocation: true
argument-hint: "[init|adopt|handshake|patrol|dispatch|agent-os|task|janitor|worktree|settle|digest|upgrade|doctor]"
---

# agent-on 调用入口

> **本文件是唯一内核**：Claude Code 经 `/agent-on`、Codex 经 `$agent-on` 调用的是**同一份本文件**（plugin 装入或 symlink）。改路由只改这里，别造第二份入口。
> **路径产品约定（2026-07-16）**：`~/Projects/Agent-On` **不是**产品默认。装机面（A）与可写工作仓（B）分离；B 必须显式登记，路径任意（Windows/mac/Linux 皆可）。

用户敲了 `/agent-on $ARGUMENTS`（或 Codex 的 `$agent-on …`）。先解析路径，再按参数第一个词查表执行。

## 路径：A 运行包 vs B 工作仓

| 符号 | 角色 | 能否写 | 解析序（实现与 `agent-on doctor` / `cli` 一致） |
|---|---|---|---|
| **`$READ_ROOT`** | 读 BOOTSTRAP/kit/playbook | 只读即可 | plugin 根 → 否则同 B；都无 → 提示 `agent-on setup` / plugin，**禁止**猜 `Projects` |
| **`$WRITE_ROOT`** | 结账写 intake、消化改 canonical | **必须可写 git clone** | ① `AGENT_ON_ROOT` ② config `work_root` ③ lock「本地路径」 ④ 默认目录 mac/linux `~/.local/share/agent-on`、Windows `%LOCALAPPDATA%\agent-on`（须已是合法仓）⑤ 都无 → settle/digest **拒绝** |

「像 agent-on 仓」= 根有 `CHARTER.md` + `BOOTSTRAP.md`。

**登记 B（推荐 setup）**：

```bash
cargo install --path cli --force   # 一次
agent-on setup                     # → 默认目录 + config
# 或 export AGENT_ON_ROOT=... / 手写 config / lock 本地路径
```

自检：`agent-on doctor` 或 `/agent-on doctor`；贡献前 `agent-on intake-lint`。

## 子命令表

| 子命令 | 读这份文件 | 做什么 | 路径要求 |
|---|---|---|---|
| `init` | `$READ_ROOT/BOOTSTRAP.md` | 新项目从零初始化 | 仅需 `$READ_ROOT` |
| `adopt` | `$READ_ROOT/boot/adopt.md` | 接管已开工项目 | 仅需 `$READ_ROOT` |
| `handshake` | `$READ_ROOT/boot/session-handshake.md` | 换会话续跑三步握手 | 仅需 `$READ_ROOT` |
| `worktree` | `$READ_ROOT/kit/worktree-control-plane.md` | 多会话轨道登记 / 边界审计 / shared Git hooks / 合流与保守回收；空后缀先跑 `agent-on worktree status`，回收盘点走 report-only `gc --dry-run` | 需 `$READ_ROOT` + 项目已是 git 仓；lane 与 hooks 状态只写本机 git common dir，可选调度只写用户本机配置/日志，不写 B |
| `settle` | `$READ_ROOT/boot/settlement.md`（上半场） | 教训回流 intake | **必须 `$WRITE_ROOT`**；intake 只写 B，禁止写 plugin cache |
| `digest` | `$READ_ROOT/boot/settlement.md`（下半场） | 消化落地 canonical | **必须在 `$WRITE_ROOT` 的主树会话**（开场第四检；宿主不让挪会话时按出口②清场后在 worktree 里消化）；无 B 则拒绝 |
| `upgrade` | `$READ_ROOT/boot/settlement.md`（升级节） | bump 项目 lock pin | 需 `$READ_ROOT`（读 CHANGELOG） |
| `doctor` | （本文件 + 跑 `agent-on doctor`） | 打印 read_root / work_root / 登记指引 | 无 |
| `patrol` | `$READ_ROOT/kit/patrol-control-plane.md` | 创建独立巡逻会话，之后复用，原窗口仍是入口 | 项目 git 仓；本机 control 回执 |
| `dispatch` / `agent-os` | 同上 | 明确目标和共享路径后调用 Claude/Codex/Grok | 项目 git 仓 + 已有执行器；在班派工经值守 |
| `task` | 同上 | 登记、真实结果回执、带证据释放 | 项目 git 仓 |
| `janitor` | 同上 | 默认报告；enable 授权有限回收受管 checkout | 项目 git 仓；不扩展旧 GC 权限 |

**Loop / 定时开火（用户可见）**: cadence（开火间隔）≠ 任务截止或工时预算。stop condition 必须对齐用户「做完」范围；局部完成（只文档/局部 UI）时默认**不自删**调度，并在回报写明未完成轨（见 kit/phase-card-template §2b）。

**上游贡献**(非独立子命令):用户说「贡献上游 / 开 intake PR」时,读 `$READ_ROOT/boot/settlement.md`「上游贡献形态」——只运 `intake/` 或 Issue 卡片,**禁止**指导用户 PR 直改 playbook/kit。

## 规则

- **空参数**：列子命令表 + 若可能则跑 doctor 一行结论，问用户要哪个。
- **项目根没有 `agent-on.lock.md`**：判断全新 vs 存量 → init 或 adopt，报一句即可。
- **worktree 参数**：`/agent-on worktree` 空后缀 = 读模式并跑只读 `agent-on worktree status`；有 `claim|set-status|status|check|hooks|gc|forget` 后缀时，先按模式核边界，再把后缀原样交给同名 CLI。并行模式首次用 `hooks install`，可选每日报告才加 `--daily-gc`；`gc` 必须显式带 `--dry-run`，不存在 apply/delete 模式。不得把任何 status/check/hooks/gc 偷换成删除或静默改写用户 Claude/Codex 配置。
- **settle/digest 前**：若 `$WRITE_ROOT` 为空，**停止**，提示 `agent-on setup`；不要写进 plugin cache，不要假设 `~/Projects/Agent-On`。
- **消化开场粘贴命令**：用已解析的 `$WRITE_ROOT` 绝对路径（或口令「消化」），禁止写死 Chao 本机路径。
- **对表/升级诚实播报**：若 B 的 HEAD 领先最新 tag，报「未发布变化 N commit」——未发布 ≠ 可升级版本。
- **消化收尾硬门**：canonical 有改动则必须封版打 tag 并 push（`agent-on tag-release`）；禁止「消化完成、tag 留到以后」。
- **认不出的子命令**：列表让用户重选。
- **patrol**：空后缀启用独立巡逻窗口；先复用已有回执。CLI 默认 Terminal（或用户选 tmux），host 跟随当前工具；Codex 桌面用下述原生桥接。status/scan 不建聊；hooks 的安装/信任按宿主正常流程，不改全局权限。
- **dispatch / agent-os**：目标已明确就登记并派工，只补实际缺失信息。先归并共享文件，按需隔离树；模型默认沿用 host 设置，不固定角色与模型。中文口令走同一执行书。
- **janitor**：enable 只授权执行书里的有限范围，不接管现有旧树后直接删。原生 worktree 的归档走宿主 archive；CLI 清道夫只处理它创建的 checkout。
- 读到目标文件后**照它执行**，不在这里复述改写步骤。
- **口令 ≠ Skill 调用**：中文口令「agent-on 结账 / 升级 / 消化」= 读本表目标文件照做，**不走 Skill 工具**。本 skill 挂了 `disable-model-invocation`，禁止 `Skill(agent-on)` 代调。工具若回「Do not replicate this skill's workflow」只禁 Skill 绕行，**不禁口令路径**。斜杠 / `$agent-on` 与口令**结果等价、调用面不等价**（斜杠管确定性，口令管自然语言）。

## Codex 桌面桥接（用户请求启用独立窗口/派工时）

1. `patrol start --host codex-app --window external` 或对应 dispatch 返回回执和 bootstrap；已有 bound thread 直接复用。
2. 用 `list_projects` 核对当前项目，`create_thread` 带 bootstrap 与项目，默认 local。用户请求 worktree 时由宿主创建，不另外造 Git 树；不覆盖用户的模型选择。
3. `clientThreadId` 表示准备中，不传给需要 threadId 的工具。取得真实 id 后 `wait_threads` 等一次进展，再 `patrol bind <receipt-id> --thread-id <实际id>`；要求 worktree 的加 `--worktree-path <实际返回路径>`。失败保留回执；结果未知先查现有聊天，不能盲目重复创建。
4. 创建成功按宿主规则输出 created-thread directive。bootstrap 自动启动初次报告，用户不用重复发送。原生独立聊天不保证第二个系统窗口；需要真实终端弹窗用 Terminal 适配器。
5. 用户已启用周期巡逻/回收时用宿主 heartbeat，记录实际 automation id；无可行动变化时安静，config 关闭就停止对应调度。缺少创建/heartbeat 工具则用终端适配器并说明覆盖，不用预填链接冒充运行。
