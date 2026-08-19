# 模式：worktree 回收报告 + 孤本保护

> 职责边界：把 worktree 生命周期盘点做成可重复的 **report-only** 执行体，而不是在 TODO 里留一个没人读的清理日期。Agent-On 只检查和报告，不自动删除目录或分支。
> 源流：Dartify 2026-08-01 至 08-16 的真实回收、squash 误判与 babysit 值守；其中项目特有假脏白名单只算来源项目经验，不是通用规则。

多会话项目先启用 [worktree-control-plane.md](worktree-control-plane.md)：`agent-on worktree status` 管轨道合同与边界；本篇的 `agent-on worktree gc --dry-run` 管回收证据、磁盘占用和动态候选。两者都坚持**拿不准只报告、不自动删**。

## 日历死线

- **禁**:只在 TODOS 写 `截止日期:YYYY-MM-DD` 而无任何进程读取
- **要**：每天盘点一次并写本机日志/JSON；删除仍是单独的人工作业，不由定时任务执行

## 陈年树是债务，清账责任归常设议程（2026-08-20 立）

> 源流：某功能窗口 2026-08-20 撞上边界闸全场 FAIL——两棵死了很久的陈年树（落后 165 / 落后 124）把五条在跑的 lane 全锁住。详见 bench 案 40 与 [snapshot/2026-08-20-gate-exit-reachability.md](../snapshot/2026-08-20-gate-exit-reachability.md)。

回收是 report-only、删除永远人工，这两条不变。但「不自动删」不等于「没人负责删」——**没有常设清账议程的连坐闸，等于给全场装了一颗随时间累积、随机引爆的雷**：树越老债越大，而 FAIL 砸中的是下一个恰好要提交的会话，那个会话既不是肇事者，也没有删树的权限。

三条硬规矩：

1. **陈年树按债务记，不按垃圾扔**。可判口径：`gc --dry-run` 报 `rescue`/`review` 且相对 base 落后超过项目自定阈值（无经验值时先用 100 个提交起步，落后到这个量级说明它已经错过整轮迭代），或最近一次活动早于当前迭代周期。命中即入债务清单，一条一行写清「归谁 / 下一动作」。
2. **清账是值守/控制轨的常设议程，不是撞闸者的意外任务**。被闸拦住的会话**只负责报告债务，不负责偿还**——它照常把自己的活换非阻塞路径推进，把陈年树连同证据交单给值守/控制轨。让撞闸者顺手清树，等于要求一个没上下文、没权限的人替全场还债，还会诱发「为了通过闸而绕过闸」（回填 owns、JSON 直改、`--no-verify`）。
3. **禁止为了让闸变绿而改账**。把 OUT-OF-BOUNDS 清单回填进 owns、直改 lane JSON 扩边界，都是**用假账换绿灯**：闸绿了，互斥保护也没了，而且多棵脏树同时回填必然撞出 OVERLAP——一条 FAIL 换成另一条（2026-08-20 实测）。真解是清债或修机制，不是改账。

**被连坐锁死时的处置阶梯（从非破坏往破坏走，2026-08-20 实测）**：

| 顺序 | 情形 | 动作 | 破坏性 |
|---|---|---|---|
| ① | worktree 早已不存在，只剩 lane 记录（幽灵） | `agent-on worktree forget --id X` | 无 |
| ② | worktree 还在，越界来自**已提交发散**（死分支 behind 100+） | `agent-on worktree edit --id X --base <该 lane 分支头>` —— 重钉后 `diff(base,HEAD)` 归零、越界清零；**detached 的要钉 worktree 真实 HEAD，钉分支头会让 diff 炸开** | 无，零工作损失 |
| ③ | 未登记的**干净**树 | 占位 claim + park | 无 |
| ④ | 未提交脏文件 / untracked 造成的越界 | 重钉 base **无效**（`git diff` 与 untracked 不看 base）。先 push 消孤本，按债务交单 | 清脏/删树是破坏性，归用户拍板 |

**「同名 FAIL 不等于同一个病」**：OUT-OF-BOUNDS 由已提交发散引起时②就是干净出口；由未提交脏文件引起时②无效。判「无解」之前先把成因分类穷举完，否则会把「我没找到出口」写成「不存在出口」，把可解问题上报成破坏性决策。

**别指望「先 commit 再说」**：guard 把 `commit` 与 `push` 一起列为审计点（`cli/src/guard.rs`），全场没绿之前两个都拦——抢救未提交的工作**不是**永远可以先做。

**给闸的设计约束**（通用，见 playbook §三½.5 出口面 4B）：每个 FAIL 条件至少留一条出口，落在被拦者当下权限内且非破坏性。删 worktree、清脏文件、`--force` 全是破坏性动作，Agent 会话默认没有这些权限——**出口只落在这些动作上的闸，对 AI 会话等于没有出口**。

## 三判据：全中才列为候选

| 条件 | 说明 |
|---|---|
| ① 已进入 base | PR 状态优先；squash 后本地 hash 不可信。`merge-base` 的否不能单独推翻 MERGED，但 MERGED 后又长出的本地提交仍是孤本 |
| ② 已保存 | 无未推送提交，也无未被 base 或 MERGED PR 权威覆盖、只活在该 worktree 的 post-merge / unique commit；无 upstream 或远端分支消失必须结合 PR/base 判据，不能直接当 0 |
| ③ 工作区 clean | 通用层只把 clean 判通过。dirty 先看 diff；历史清洗反向、生成物、机器配置是否无价值必须由项目规则或人确认 |

候选还必须满足：不是 primary、未 locked、没有 open PR / active lane，并经过静默窗口（默认 24h）。静默同时看工作目录与 linked worktree 自己的 git admin dir（HEAD/index/reflog 等）活动，但仍只是证据，不证明聊天窗口已经关闭。无 PR 且未进入 base、detached 归属不明、`gh` 失败、CLOSED 但未合、任一关键事实 unknown，都只报告。

“候选”不等于“已授权删除”。人工执行时默认只拆 worktree，分支是否删除另行拍板；远端分支删除永远算外向硬门。

## 孤本抢救三步(与回收解耦)

1. **push 远端**消单点——推自己的分支是本轨内部动作，不用等授权（孤本多留一天就多一天丢盘风险）；`--force`、删远端分支才是硬门
2. **再**回收 worktree  
3. 择期 rebase/开 PR 落地  

跨大跨度 rebase:契约锁/棘轮测试红 → **显式更新锁**随契约走,不放宽。

## dry-run：唯一产品入口

```bash
agent-on worktree gc --dry-run
agent-on worktree gc --dry-run --json
```

完整参数：

```text
agent-on worktree gc --dry-run [--json] [--repo PATH] [--base REF] [--quiet-hours N]
```

- 不带 `--dry-run` 直接拒绝；没有 apply/delete 模式。
- 命令只读，不更新 registry、不写文件；需要留档时由调用方把 JSON 写进本机 common git dir 或日志目录。
- JSON 顶层含 `repo / mode / base / quiet_hours / github_pr_query / worktrees / candidates / errors`。`candidates` 是**本次实时推导的 known reclaim list**，不是手填常青名单；下次盘点必须重算。
- `dirty_entries`、`criteria`、`decision`、`reasons` 都保留在每棵树记录里，让人能看到为什么是 `primary|keep|review|rescue|candidate`。

## 定时与握手频率

- 会话握手：`agent-on worktree status`，确认自己 cwd / branch / lane；
- 每天一次：`gc --dry-run --json` + `df -h`，JSON 仅落本机；需要自动触发才显式安装可选调度；
- 每次 merge/read-back 后：标记 lane `landed`，立刻重跑一次 GC 报告；
- 磁盘告急时可加跑，但不能因为空间压力放宽判据。

产品入口只有一组，不要求用户手写 plist/unit：

```bash
agent-on worktree hooks install --daily-gc
agent-on worktree hooks status
agent-on worktree hooks uninstall
```

- 不加 `--daily-gc` 就不安装调度；固定每日 03:30；macOS 用 LaunchAgent，Linux 用 systemd user timer，无 daemon；
- 从任意 linked worktree 调用都会归一到 primary，`WorkingDirectory` 与 `--repo` 不随短命功能树消失；
- 调度命令精确固定为 `agent-on worktree gc --dry-run --json --repo <primary>`；没有 apply/delete 参数；
- stdout/stderr 只写 XDG state 或 `~/.local/state/agent-on/worktree-gc/<repo-key>/`，卸载保留历史报告；
- `status` 报 optional absent / active / inactive / drift；foreign 或 drifted 配置不覆盖、不删除；
- `gh` 或远端不可用时沿用 GC 的 fail-closed 分类，只产 unknown/review，不把查询失败翻译成候选。

## 权限红线

- 自动：只读检查、写本机报告；GC 不更新 lane metadata；
- 人工/目标明确授权：删除目录、删除本地/远端分支、任何 `--force`、跨 worktree add/commit；
- 永不直接删：locked、dirty、unknown。先解除占用、逐项分类或抢救，再从头跑报告。

## 交付前对表(硬门 · Dartify 2026-08-08)

worktree 是**创建那一刻** default branch 的快照,之后 main 前进**不会**提醒你。凡装机 / 演示 / 截图给人看 / 对外发布:

1. `git fetch origin`
2. `git rev-list --count HEAD..origin/<default>` —— **非 0 则先对齐再构建**(ff-only 或 rebase)
3. 播报必须写:**commit hash + 与 origin/default 落差**(例:`@3fcbbc4,与 origin/main 落差 0`)
4. **禁止**从本地 HEAD 随口说「最新」——交付物(已装 App)往往不自带版本号,用户只能靠「感觉没变」发现,反馈链极长

与「squash 换 hash 误判」同族:都是 worktree 与 main 的**时间差**在不同环节咬人。

## PR DIRTY + CI 零 job(两种成因,同一张脸)

`gh pr view --json mergeStateStatus` 出 `DIRTY` / `CONFLICTING`,且 `gh pr checks` 只剩外部检查、仓内 job 一个没有——不要先查 workflow 配置。GitHub 不为 DIRTY PR 起 checkout。两种常见成因:

1. **分支起点错**(本篇既有):squash 换 hash 后新枝从旧 HEAD 长
2. **base 刚被直推进**(Dartify 2026-08-15):自己往 main 推了一笔记账/chore,与开着的 PR 改同一批文件

判据一行:DIRTY → 先看 base 是否刚动、分支是否从过期点长出。模式见 playbook/multi-contributor §三½.5。
