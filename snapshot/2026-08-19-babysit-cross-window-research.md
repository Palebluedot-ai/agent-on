# 跨窗口值守调研：Claude Code 并行能力实测 × babysit 缺口（2026-08-19）

> 职责边界：本页是一次**调研快照**——记录 2026-08-19 对 Claude Code 并行/跨会话能力的本机实测，以及它与本仓 babysit 机制的接合点与缺口。它不改任何机制：机制真相仍在 `docs/babysit.md`（值守循环体）、`kit/babysit/MERGE-POLICY.md`（授权与时延）、`kit/output-contract.md`（一轮怎么说话）、`kit/landing-control-plane.md` 与 `kit/worktree-control-plane.md`（控制面）。本轮唯一落地的机制改动是 `kit/output-contract.md` 的三处增补（见 §7）。
> 触发：用户提出「跨窗口协作增强」需求，列了五条痛点与一个终极目标——「任务进入统一 TODO 后，能一个一个被找到，并由系统自主完成开发」。
> 版本坐标：Claude Code CLI 2.1.201 / 在跑会话 2.1.234；agent-on CLI 0.12.1；本仓 v0.15.0（v0.16.0 封版在途）。

## 一句话

**缺口不在设计，在强制点与状态可读性。** 结构化输出契约与授权分级已经落地并写进 git；真正缺的是三样：①窗口是 interactive 所以机器读不到状态 ②TODO 只活在单会话 ③冲突治理只有事后一层。终极目标的完全体今天做不到（Claude Code 无跨会话共享 task list），但**值守当中心化调度器**这个次优形态可以增量做到，且对「一个值守 + N 个窗口」本来就是正确架构。

## §1 需求方四条前提，实测后不成立

按 `kit/deep-research-prompt-template.md` v2 补丁纪律 B（授权推翻前提）单列。

| 原前提 | 实况 | 证据 |
|---|---|---|
| 「输出不结构化，非值守窗口输出大量混杂文字」 | `kit/output-contract.md` 已是需求方模板的**超集**，多三样：拍板默认值、撤销两栏、球在谁那。**缺的是机械强制点**，不是模板 | 仓内 204 行原文 |
| 「Agent View 一屏管理所有窗口，显示 Needs input / Working」 | 官方文档：**interactive 会话不进 Agent View**。本机 `claude agents --json` 确实列出 interactive 会话（与文档措辞不同），但**只有 background 会话带 `status` / `state` 字段** | 见 §2 探针 |
| 「真相之页已实现，可清晰查看全局状态」 | landing 控制面是**按需只读快照**，不驻后台；且 `docs/babysit.md` §5 自己记着「`landing status --human` 面板渲染未挂」 | `kit/landing-control-plane.md` + 值守文档遗留清单 |
| 「多进程与统一管理不足」 | **隔离已经很强**（lane 合同 + `owns` 非重叠闸 + 活跃轨上限 3 + shared git hook + PreToolUse guard，全部 fail-closed）。不足的是**调度**——没人回答「下一个该谁干什么」 | `agent-on worktree check` → RESULT: PASS |

## §2 实测：状态字段只存在于 background 会话

起一条 background 探针会话，与在班的 interactive 会话对比 `claude agents --json --all` 的字段：

```text
interactive（本机 9 个窗口全是这种）:
  { pid, id, cwd, kind:"interactive", startedAt, sessionId, name }
                                            ← 无 status，无 state
background（探针 bf7be00b）:
  { pid, id, cwd, kind:"background", startedAt, sessionId, name,
    status:"idle", state:"blocked" }        ← 状态在这里
```

探针跑完即 `claude stop` + `claude rm` 回收，未留残留。

**结论**：值守想「先于窗口自报家门就发现谁卡住」，前提是那条轨跑在 background。这是本次唯一有真实代价的建议（见 §5-d）。

## §3 跨窗口现状：口令与真相源

| 环节 | 实际口令 / 命令 | 真相源 |
|---|---|---|
| 值守起班 | `/loop 读 docs/babysit.md 全文并执行本轮值守` | — |
| 功能窗口交单 | 自然语言让 Claude 发 SendMessage；地址 = 值守写在 `docs/babysit.md` §1 的会话名前缀 | **不是**消息 |
| 队列真相源 | `gh pr list --repo Palebluedot-ai/agent-on --state open` | **open PR 列表** |
| 看谁在班 | `/list-agents`（别名 `/peers`）；Claude 侧是 ListAgents 工具 | 会话 socket 注册 |
| 点名某窗口 | prompt 里 `@` + 会话名前几字母选 typeahead（需 v2.1.232+） | — |
| 切心跳档 | 口令「值守加速」→ 3–5 分钟；「值守回落」→ 常态；连续 3 轮 noop 自动回落 | — |
| lane 边界 | `agent-on worktree claim / edit / set-status / check / status / gc` | common git dir 的 lane JSON |
| 排队波次 | `agent-on landing refresh`（唯一联网）→ `landing plan` | `agent-on/landing/snapshot.json` |

**一条实况陷阱**：`/list-agents` 是**全机器**的，不按仓分——本机实测同时列出 Agent-On 与 Dartify 两个仓的 7 条 peer 会话。交单时点错名字是真实风险；值守应该用 `claude agents --json --cwd <repo>` 按目录过滤。

## §4 官方四种并行模式（唯一权威对照）

官方 `docs/en/agents` 把并行工作分为四种，本仓只用到其中一种半：

| 模式 | 是什么 | 对本仓的适配 |
|---|---|---|
| **Subagents** | 一个会话内的委派工，独立上下文，结果回报给调用者 | 已在用（Explore / 派工） |
| **Agent view / background** | `claude agents` 一屏调度后台会话，六态：Working / Needs input / Idle / Completed / Failed / Stopped | **未用**——窗口全 interactive |
| **Agent Teams** | lead + teammate，共享 task list、依赖自动解锁、自主抢单、文件锁防竞态 | 最接近终极目标，但**只在一个 session 内**，见 §6 |
| **Dynamic workflows** | 脚本持有计划，跑大量 subagent 并交叉核验；`/workflows` 看进度 | 适合一次性大审计/迁移，不适合常驻值守 |

三条支撑设施：**worktrees**（隔离）、**cross-session messaging**（会话间传话）、**`/batch` skill**（把一个大改动拆成 5–30 个 worktree 隔离的 subagent，**每个各开一个 PR**）。

### 两条与本仓控制面直接冲突的实况（重要）

1. **Agent view 会自动给每条被派发的会话建 worktree。** 这些 worktree **不经过 `agent-on worktree claim`**，落地即「未登记 worktree」——按 `kit/worktree-control-plane.md` 的设计会**连坐全场 `check` FAIL**。background 化之前必须先决定：是每条派发后补占位 claim，还是给 `WorktreeCreate` hook 挂自动登记。
2. **Agent Teams 不给 teammate 做 worktree 隔离**（官方原话是「partition the work so each teammate owns a different set of files」）。此前社区文章声称「每个 teammate 独立 worktree」，与官方文档冲突，**按官方为准，社区说法已剔除**。所以 Agent Teams 若引入，必须靠 `owns` 式的文件分区自律，拿不到 lane 闸的机械保护。
3. **`/batch` 会瞬间灌爆 babysit 队列**——5–30 个 PR 同时进 open 列表，且都是未登记 worktree。在本仓属于「必须先问档」，值守不许自行消化。

## §5 改进方案（四件，按投入产出排序）

### (a) 自动编号：不发明新编号，拼两层现成命名空间

`<会话名>#<任务 id>`。会话名来自 `~/.claude/sessions/<pid>.json` 的 `name`（`--name` / `/rename` 可定），任务 id 来自 `~/.claude/tasks/<sessionId>/<n>.json`（字段含 `id / subject / description / status / blocks / blockedBy`，**会话死了文件还在**）。这个编号天然可寻址：会话名就是 SendMessage 地址，也是 `@` 点名的键。

规范已写进 `kit/output-contract.md` §2（本轮落地）。

**跨窗口 TODO 板原型已跑通**（join 两个目录，约 30 行 Python），但**读数是空的**：

```text
9 条在班会话 → 只有 2 条有 task 文件 → 未完成任务 0 条
本会话（claude-desktop 入口）甚至没有 Task 工具
```

**所以真实前置条件是纪律不是工具**：功能会话必须把工作拆进 todo，否则板子永远是空的。这一句已写进 output-contract §2 的「前置纪律」。

### (b) 结构化输出强制：Stop hook + 退出码 2

官方 `Stop` hook 退出码 2 的语义是「挡住停止，把 stderr 注回去让它继续」——正是缺的强制点。

```jsonc
// .claude/settings.json
{ "hooks": { "Stop": [{ "matcher": "", "hooks": [
  { "type": "command", "command": "bash .claude/hooks/output-contract-gate.sh", "timeout": 15 }
]}]}}
```

判据：①本轮没动过 git → exit 0 不管；②动过 → 查最后一条 assistant 消息是否含四锚点（状态面板 / 要你拍的板 / 撤销面 / 球在谁那）；③缺任一 → 写理由到 stderr + exit 2。

**诚实边界两条**：Stop **每回合都触发**，第①条触发条件不可省——`kit/claude-hooks-template.md` 已写过「别挂 Stop 每回合跑快照」这条教训；hook 拿不到「最后一条 assistant 消息」的现成参数，得从 transcript JSONL 读，这是要写代码的一步，不是配置。

### (c) merge conflict：补上事前与事中两层

| 层 | 机制 | 现状 |
|---|---|---|
| 事前 | `owns` 非重叠闸——两条 active 轨文件域重叠时 `claim` 直接拒绝 | 已有且很硬，但一直被当「边界闸」讲，没被当**冲突预防**讲 |
| 事前 | landing 的**文件重叠图**算出「谁必须错峰」，产出 `NEXT（等 #N）` | 已有未用——文档现在写的是「波次是建议，不是执行」 |
| 事中 | `git rerere`：同一冲突解一次之后自动复用 | **本机未开**（`git config --get rerere.enabled` 无输出）。多 worktree 长期并行下性价比最高的一条 |
| 事中 | 服务端追平 `gh api -X PUT …/update-branch` | 已有，不动 |
| 事后 | 真冲突打回作者四件套 + 面板写「这单已不占你注意力」 | 已有，不动 |

```bash
git config --global rerere.enabled true
git config --global rerere.autoUpdate true
```

### (d) 多进程统一管理：执行轨 background 化（唯一有代价的一条）

换成 `claude --bg` 后值守才拿得到 `status`，代价是**用户自己看不见那个窗口的输出**，要 `claude attach <id>`。建议折中：保留 1–2 个 interactive 窗口（含值守本身），批量执行轨走 background。

**前置**：先解决 §4 的「自动建 worktree 不登记」冲突，否则一开就全场 `check` FAIL。

## §6 终极目标的诚实天花板

「跨窗口统一 TODO，任务被自主领走并做完」的**完全体今天做不到**——Claude Code 的共享 task list 只存在于**一个 session 的 team 内部**（官方硬限制：one team per session、lead 固定不可转移、不跨会话、不能嵌套、`/resume` 不恢复 in-process teammate）。

**能做到的次优形态，而且它够用**：值守当**中心化调度器**——读文件层拿全场任务与依赖 → 按 landing 波次排序 → SendMessage 把「下一件事」推给具体窗口 → Stop hook 保证回来的是结构化的 → `status` 字段发现卡壳。

这是中心化调度而非去中心化抢单，但对「一个值守 + N 个窗口」的场景，中心化本来就是对的架构——**本仓的合并权本来就唯一归值守**。

### 落地四步（每步带验收）

| 步 | 做什么 | 验收 |
|---|---|---|
| 1（15 分钟） | 开 `rerere`；值守每轮加一条 `claude agents --json --cwd <repo>` 写进面板；派工词加「把工作拆进 todo」 | 值守面板出现在班窗口清单，且至少一个功能窗口 task 文件非空 |
| 2（半天） | TODO 板收进 `cli/` 作 `agent-on landing sessions`（`--json` 给值守 / 等宽表给人）；顺手做掉 `landing status --human` | 一条命令列出全部在班窗口 + 未完成任务 + 依赖 |
| 3（半天） | 写 `output-contract-gate.sh`，先在**一个**窗口试挂一周 | 故意写混杂文字收尾，被 hook 拦下并注入「缺 X 段」 |
| 4（评估） | 选 1 条轨试 `--bg`；Agent Teams **只在单一功能轨内部**试点 | 值守在窗口自报之前，先从 `status` 发现卡壳 |

## §7 本轮落地的机制改动（只有一处文件）

`kit/output-contract.md` 三处增补，全部是加法，不改既有段的顺序与语义：

1. **§2 轨名列**：新增跨窗口引用编号 `<会话名>#<任务 id>` 规范 + 两层命名空间来源表 + 「功能会话必须拆 todo」的前置纪律。
2. **§1 新增子节「表格是允许的渲染形式，不是第二份模板」**：把需求方要的三张表（需要我拍板的 / 你做的事情 / 交接给值守）**映射**进现有六段，明确不许另立第二份交接模板；并把「你做的事情」第三列钉死为**证据指针**（自举纪律 2）。
3. **§3 新增子节「默认值默认等于建议值」**：源自用户 2026-08-19 原话「为什么我不回按照不是建议的来，看起来很奇怪」。规则是默认值默认写建议值，只有**不可逆**或**超出已授权范围**两种情况允许降级且必须写明理由。理由：建议与默认系统性不一致会让用户学到「沉默 = 较差结果」，被迫每条都回，抵消拍板前移的全部收益。§11 自查表同步加一条。

**兼容性**：babysit 四条不变量一条未动——队列真相源仍是 open PR 列表；门铃丢了最多晚一个心跳不漏单；批准只认值守会话内的用户输入（官方 cross-session messaging 的「消息不能代替你的批准」正好加固了它）；在班期间合并权唯一归值守。

**待同步**：`kit/babysit/BABYSIT-TEMPLATE.md` §6 与 `docs/babysit.md` §6 都引用 output-contract，本轮**未动**（不在本轨 owns 内）——归后续消化会话按 §9 索引对表。

## §8 核验留痕

可验证断言 26 条，逐条跑命令核对：**23 条确认 / 3 条改正**。三条改正都是硬伤级（不改会误导决策）：

1. **「Agent View 显示所有窗口的 status」→ 改正**。起 background 探针后发现 interactive 会话 JSON 里根本没有 `status` 字段。不核则 §5-d 整条建立在假前提上。
2. **「跨窗口 TODO 板能立刻用起来」→ 改正**。脚本跑通，但 9 条会话只有 2 条有 task 文件、未完成任务 0 条。板子今天是空的。
3. **「Agent Teams 每个 teammate 独立 worktree」→ 改正**。这是社区文章的说法；官方文档明写不做 worktree 隔离，要靠手工分区文件。

**剔除的资料**：搜到的 2026 年 Claude Code 多 agent 社区指南（Medium 及各类 blog）全部未采信为事实来源——追不到被测对象与方法。本页所有官方能力断言只来自 code.claude.com 官方文档与本机命令输出两处。

**我按这个假设做了，你不否就当成立**：

- Stop hook 能从 transcript JSONL 读到最后一条 assistant 消息。**否掉要重做**：§5-b 换成 `PostToolUse` 挂在 `gh pr create` 上（时机更早、拿到的上下文更少）。
- `claude agents --json` 对 background 会话的 `status` 值域覆盖文档列的六态。**否掉要重做**：值守只能用 `status` 做粗分类，细分仍靠窗口自报。本轮只实测到 `idle` 一个值。

## §9 顺手发现，不代修

本 worktree 的 lane 曾出现登记分支与实际分支不符（lane 记 `claude/agent-on-data-hygiene-0d76ed`，实际 checkout `claude/agent-on-babysit-research-b28963`）。`check` 报 PASS 是因为 `changed=0`，但这棵树写任何文件都会越界。**已按用户 2026-08-19 拍板用 `agent-on worktree edit` 重划**（goal / branch / owns 三项），重划后 `check` 复绿。

教训入分诊：**`check` 的 PASS 不保证 lane 记录是对的**——`changed=0` 时分支漂移不会被发现。窗口复用（同一棵树换个题目继续干）是这条漂移的常见来源，换题目就该重划。

## 可读版

同内容的网页版（含实测输出面板与逐条对照表）：<https://claude.ai/code/artifact/bf8bd4ac-a217-4484-86c4-a2084f93aace>
