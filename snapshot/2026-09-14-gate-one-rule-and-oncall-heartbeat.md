# 边界闸只拦一件事 + 值守登记带心跳（2026-09-14 决策快照）

> 职责边界：本页记**这次改了什么判定语义、为什么、拿什么证据**。机制正文在 [kit/worktree-control-plane.md](../kit/worktree-control-plane.md)「闸只拦真冲突」一节与 [kit/babysit/ROUTING.md](../kit/babysit/ROUTING.md) §4「在班登记」；本页不重复展开判据表。
> 触发：2026-09-14 用户原话——「这个库在其他地方用的时候，经常提示某个类没注册就被挡住了……一个类没有注册，流程就卡住了，无法 commit，全变红了。我并不想制造太多限制……值守的窗口有时候没开又会卡住」。
> 拍板：同一句话即拍板——用户要的是**少限制、不卡流程**，本页据此**推翻 2026-08-17「连坐维持」的决策**。

## 一句话

**闸从此只拦一件事：本 worktree 未提交的改动落进了别的活轨（active/blocked/ready）的 `owns`。** 其他一切——别的树没登记、别的树越界、登记指向的树已经删掉、纸面 owns 互相重叠、休眠的脏树——只报告、不红、不算到别人头上。值守登记同理：**窗口活着才算在班**，靠心跳证明，不靠一份写完就不动的 JSON。

## 病灶：两个「登记」都把「有记录」当成了「有人」

| 闸 | 旧判据 | 实际后果 |
|---|---|---|
| worktree 边界闸 | 全场任一未登记树 / 任一越界 / 任一 OVERLAP / 任一幽灵登记 → **所有 worktree 的 commit 与 push 全拦** | 桌面宿主每开一个会话就建一棵 `.claude/worktrees/<name>`，从不 `claim`；于是多窗口的仓**大部分时间是红的**，被拦的往往不是肇事者。Orbit 2026-09-08、Dartify 2026-08-16/19、本仓 2026-08-20 各撞一次，三个项目的记忆文件都留了「解锁配方」 |
| 值守路由闸 | 登记的 worktree 目录还在 → 值守「在班」 | 窗口关了、`release` 没跑，目录当然还在；于是**全仓其他窗口的合并 / 对外通信被一个不存在的窗口锁着**。本仓实测：值守登记停在 2026-08-19，26 天后（37606 分钟）旧版 `oncall status` 仍报「在班」 |

两处是同一个错误：**把「有一条记录」当成「有一个活人」**。上一轮（#29）已经在边界闸里把「有没有未落地改动」和「有没有人在写」分开了，但只分了一半——未登记树与幽灵登记还走「有记录 = 出事」这条老路，值守登记则完全没分。

## 决策

### 边界闸：一条红灯，其余全是提示

- **红灯只有一条**：本树**未提交**（staged / unstaged / untracked）的改动，落在另一条**活轨**（`active` / `blocked` / `ready`，且 worktree 还在）的 `owns` 里。报 `CONFLICT`，只拦**这棵树**的 commit / push。
- 只看未提交改动，不看相对 base 的已提交发散：已提交的东西在它提交那一刻已经过过闸（或被人有意 `--no-verify` 跳过），死分支落后 165 个提交的旧发散不该跟着每一次新提交跑。这一条同时消掉了「squash 后永远 changed N」和「主树本地 merge 完 push 被自己刚合的 lane 拦住」两类假红。
- **改成提示（不红、不拦任何人）**：`UNREGISTERED`（没登记的树不持有任何边界）、自己 owns 之外但无人持有的 `OUT-OF-BOUNDS`、纸面 `OVERLAP`、指向已删树的活登记（新增 `MISSING` 行，出口一条命令 `worktree forget`——现在任何状态都能 forget，树都没了还守什么生命周期）、休眠树（照旧 `RESCUE-DEBT`）。
- **主树不再「按身份」是控制轨**：并行期主树能不能 commit，看它碰没碰活轨的地盘，不看有没有活轨存在。merge / squash / cherry-pick / rebase 进行中照旧一律放行。
- **休眠的写者不参与冲突**：30 天没人碰的脏文件是债，不是第二个写者（#29 的口径，这次贯彻到未登记树）。
- `check` 的 `RESULT: FAIL` 也只剩两种成因：某处有 CONFLICT，或审计跑不起来。JSON 新增 `conflicts[]`、`missing[]`、`primary_worktree`。
- 出口写在报错里且全部在被拦者权限内、零删除：①只提交自己 owns 内的路径 / 到那条轨的树里改 ②那条轨是你的或会话已经不在：`set-status parked --id X`（干净树 park 不丢东西）或 `edit --id X --owns …` 收窄 ③拿不准归谁：`oncall route --path`。

### 值守登记：心跳 + 失效窗口

- 登记多一个 `heartbeat_at`。**值守窗口每一次经 guard 的工具调用**（PreToolUse 对每条 Bash / SendMessage 都会跑）自动续心跳（一分钟内最多写一次）；`agent-on oncall heartbeat` 可显式续。
- 心跳超过 `oncall_stale_after_minutes`（默认 **90** 分钟，`<common git dir>/agent-on/config.json` 可配，`0` 关掉）→ 登记**失效**：路由闸 fail-open，`claim` 不用 `--force` 就能接班，`status` 写明「N 分钟没心跳」。
- 失效是按原始记录匹配的：值守窗口若只是安静了很久、人还在，下一条命令就把自己续活；别的窗口已经接班（记录被覆盖）则不会误续。
- 功能窗口被拦的文案现在带一行「值守最近心跳 N 分钟前；M 分钟没心跳自动失效」——被拦的人知道这把锁会自己开，不必去猜值守是不是还在。

## 为什么这不是「拆闸」

留下来的正是闸存在的唯一理由——**两个会话别同时写一个文件**——而且留得更准：

- 活轨对自己 owns 的预留照旧成立，谁进来谁被拦（`commit_is_blocked_only_when_it_writes_inside_another_live_lanes_owns`）。
- 主树进活轨的地盘照旧被拦（`primary_worktree_is_blocked_only_when_it_enters_a_live_lane`）。
- 登记完结但仍在写的轨，真在改同一路径时照旧红（`check_fails_when_two_writing_lanes_share_a_boundary`、`two_live_writers_on_one_path_still_fail` 原样通过）。
- `claim` / `edit` 的入口闸一字未动：还在写的路径别人仍然领不走。

被摘掉的全是**没有第二个写者的红**：一棵干净的未登记树、一条指向空目录的登记、一份纸面重叠、一处无人持有的越界。这些从来不是冲突，红了也清不掉，只会训练所有人「红了当没看见」——那比没有闸更糟（playbook §三½.5 出口面 4B，本仓自己的话）。

### 推翻的旧决策

2026-08-17 拍板「连坐维持，理由是账实一致」；2026-08-20 又补了「连坐可以留，但成本得落在有权限清理的常设角色身上」。这次两条都不再成立：**账实一致由 `status` / `check` 的报告面继续保证**（该报的一行不少），只是不再拿别人的 commit 当抵押品。清账议程（陈年树是债务）照旧归值守，但那是清理仓库卫生，不再是解锁别人的前提。

## 证据

- 测试：`cargo test --no-fail-fast` 全绿——单元 158 + 集成 8 (`gate_scope`) + 7 (`oncall_liveness`) + 7 + 9 + 3 + 3 + 11 + 8，共 214；`cargo clippy --all-targets -- -D warnings` 与 `cargo fmt --check` 均为 0。
- 真仓对照（同一时刻，旧版 v0.19.0 二进制 vs 本次构建）：

| 仓 | 现场 | 旧版 `worktree check` | 本次 |
|---|---|---|---|
| Orbit | 两棵桌面宿主自建、未登记、干净的树 | `RESULT: FAIL` | `RESULT: PASS`（两行 `UNREGISTERED` 提示照报） |
| Dartify | 一条 `active` 登记指向已删的树 | `ERROR: … worktree is missing` → `FAIL` | `MISSING: … exit: forget` → `PASS` |
| Agent-On | 值守登记停在 2026-08-19，窗口早关 | `oncall status` → 在班（锁着全仓合并权 26 天） | 「登记已失效（37606 分钟没有心跳）」，闸 fail-open |

- 本机接线：`~/.claude/settings.json` 与 `~/.codex/hooks.json` 的 hook 都指向本仓 `kit/guard/`，shim 解析到 `cli/target/release/agent-on`（本次 `cargo build --release` + `cargo install --path cli --force` 已更新）；插件缓存里没有第二份二进制，不存在旧逻辑残留。

## 未做（显式）

- 休眠窗口（7 天）与心跳窗口（90 分钟）都是拍脑袋的默认值，两个都可配；没做自适应。
- 没做「值守窗口进程存活探测」（跨宿主没有可移植的办法）；心跳是能移植的最小答案。
- `claim` 的活轨上限（3 条）与 `ready` 要求干净树两条入口约束没动——它们卡的是登记动作，不卡 commit。
