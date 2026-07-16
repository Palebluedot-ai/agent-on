---
name: agent-on
description: agent-on 方法论脚手架的显式调用入口。/agent-on <子命令>——init 新项目初始化 · adopt 接管已开工项目 · handshake 会话续接 · settle 结账回流 · digest 消化 · upgrade 升级 pin · doctor 路径自检。按子命令读取 agent-on 对应文件并逐步执行。
disable-model-invocation: true
argument-hint: [init|adopt|handshake|settle|digest|upgrade|doctor]
---

# agent-on 调用入口

> **本文件是唯一内核**：Claude Code 经 `/agent-on`、Codex 经 `$agent-on` 调用的是**同一份本文件**（plugin 装入或 symlink）。改路由只改这里，别造第二份入口。
> **路径产品约定（2026-07-16）**：`~/Projects/Agent-On` **不是**产品默认。装机面（A）与可写工作仓（B）分离；B 必须显式登记，路径任意（Windows/mac/Linux 皆可）。

用户敲了 `/agent-on $ARGUMENTS`（或 Codex 的 `$agent-on …`）。先解析路径，再按参数第一个词查表执行。

## 路径：A 运行包 vs B 工作仓

| 符号 | 角色 | 能否写 | 解析序（实现与 `kit/guard/agent_on_paths.py` 一致） |
|---|---|---|---|
| **`$READ_ROOT`** | 读 skill 路由到的 BOOTSTRAP/kit/playbook | 只读即可 | ① `CLAUDE_PLUGIN_ROOT` / `PLUGIN_ROOT`（目录含 `CHARTER.md`+`BOOTSTRAP.md`）② 已登记的 **B** ③ 都无 → 提示装 plugin 或登记 B，**禁止**猜 `Projects` |
| **`$WRITE_ROOT`** | 结账写 `intake/`、消化改 canonical | **必须可写 git clone** | ① 环境变量 `AGENT_ON_ROOT` ② `~/.config/agent-on/config.json` 的 `work_root` ③ 当前项目 `agent-on.lock.md` 的「本地路径」且该路径存在且像 agent-on 仓 ④ 都无 → **settle/digest 拒绝执行**并打印登记法 |

「像 agent-on 仓」= 根目录同时有 `CHARTER.md` 与 `BOOTSTRAP.md`。

**登记 B（任选其一，一次即可）**：

```bash
# 1) 环境变量（当前 shell / 用户 profile）
export AGENT_ON_ROOT=/absolute/path/to/clone

# 2) 本机配置（跨会话推荐；Windows 同样是用户主目录下 .config）
mkdir -p ~/.config/agent-on
echo '{"work_root":"/absolute/path/to/clone"}' > ~/.config/agent-on/config.json

# 3) 项目 lock 的「本地路径」行（仅绑定该项目）
```

clone 到**任意路径**均可，不必叫 `Projects` 或 `Agent-On`：

```bash
git clone git@github.com:Palebluedot-ai/agent-on.git /anywhere/you/like
```

自检（优先）：

```bash
python3 "$READ_ROOT/kit/guard/agent_on_paths.py"
# 或子命令 doctor
```

## 子命令表

| 子命令 | 读这份文件 | 做什么 | 路径要求 |
|---|---|---|---|
| `init` | `$READ_ROOT/BOOTSTRAP.md` | 新项目从零初始化 | 仅需 `$READ_ROOT` |
| `adopt` | `$READ_ROOT/boot/adopt.md` | 接管已开工项目 | 仅需 `$READ_ROOT` |
| `handshake` | `$READ_ROOT/boot/session-handshake.md` | 换会话续跑三步握手 | 仅需 `$READ_ROOT` |
| `settle` | `$READ_ROOT/boot/settlement.md`（上半场） | 教训回流 intake | **必须 `$WRITE_ROOT`**；intake 只写 B，禁止写 plugin cache |
| `digest` | `$READ_ROOT/boot/settlement.md`（下半场） | 消化落地 canonical | **必须在 `$WRITE_ROOT` 会话**；无 B 则拒绝 |
| `upgrade` | `$READ_ROOT/boot/settlement.md`（升级节） | bump 项目 lock pin | 需 `$READ_ROOT`（读 CHANGELOG） |
| `doctor` | （本文件 + 跑 `agent_on_paths.py`） | 打印 read_root / work_root / 登记指引 | 无 |

**上游贡献**(非独立子命令):用户说「贡献上游 / 开 intake PR」时,读 `$READ_ROOT/boot/settlement.md`「上游贡献形态」——只运 `intake/` 或 Issue 卡片,**禁止**指导用户 PR 直改 playbook/kit。

## 规则

- **空参数**：列子命令表 + 若可能则跑 doctor 一行结论，问用户要哪个。
- **项目根没有 `agent-on.lock.md`**：判断全新 vs 存量 → init 或 adopt，报一句即可。
- **settle/digest 前**：若 `$WRITE_ROOT` 为空，**停止**，贴登记 B 的三选一命令；不要写进 plugin cache，不要假设 `~/Projects/Agent-On`。
- **消化开场粘贴命令**：用已解析的 `$WRITE_ROOT` 绝对路径（或口令「消化」），禁止写死 Chao 本机路径。
- **对表/升级诚实播报**：若 B 的 HEAD 领先最新 tag，报「未发布变化 N commit」——未发布 ≠ 可升级版本。
- **认不出的子命令**：列表让用户重选。
- 读到目标文件后**照它执行**，不在这里复述改写步骤。
- 中文口令「agent-on 结账 / 升级」与本 skill 等价。
