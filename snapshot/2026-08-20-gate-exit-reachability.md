# 快照：闸的出口必须走得通（连坐死锁的制度解）

> 日期：2026-08-20 | 层级：L2 治理原则 + L3 落点 | 状态：本轨落地制度层，CLI 机制层归 `multi-lane-docs-conflict` 轨
> 职责边界：本篇记**为什么会连坐死锁、制度上怎么根除**。边界闸的代码语义怎么改（化石轨退出互斥闸）不在这里，归 `multi-lane-docs-conflict` 轨与 `kit/worktree-control-plane.md`。

## 事故

某功能窗口跑 worktree 边界闸，全场 FAIL。现场五条 lane 各带约 90 个脏文件，另有两棵陈年树（`d35-pr117-legacy` 落后 165、`legacy-crm-agent-nlp` 落后 124）死了很久没人清。会话按 `kit/worktree-control-plane.md`「重划与死锁三解」照做——回填 OUT-OF-BOUNDS 清单进 owns——发现回填会造出大量 OVERLAP，而 OVERLAP 同样是 FAIL 条件。**文档给的解法，本身就是另一条 FAIL 的成因。**

会话最后判断真解是删那两棵陈年树。但那是破坏性动作，auto-mode 硬墙拦 `worktree remove`，必须用户拍板。于是：**闸把所有人拦住了，而唯一的出口不在被拦者的权限里。**

## 复现（本轮实测，非推理）

用仓内源码构建的 `agent-on`（`cli/target/debug/agent-on`）在 scratch 仓造两棵「陈年脏树」，两棵都改了同一个共享文件：

| 步骤 | 动作 | 结果 |
|---|---|---|
| 1 | 两轨各自登记、边界互斥、全场干净 | `RESULT: PASS` |
| 2 | 两棵树变脏，各自碰同一个共享文件 | `OUT-OF-BOUNDS: docs/shared.md` ×2 → `FAIL` |
| 3a | 按文档解 ②，走正门 `worktree edit --owns` 回填 | 第二条轨被入口闸拒：`owned path docs/shared.md overlaps still-writing lane a` |
| 3b | 退到文档明说可用的 JSON 直改回填 | `OVERLAP: a:docs/shared.md <-> b:docs/shared.md` → `FAIL` |
| 4a | 想靠状态转移脱身：`set-status parked → landed` / `→ ready` | 两条都 `ERROR: invalid lane transition` |
| 4b | 缩回 owns 解 OVERLAP | 打回 `OUT-OF-BOUNDS` → `FAIL` |
| 4c | `edit --status ready`（文档明说它绕转移图） | `ready requires a clean worktree` |
| 4d | `edit --status landed`（同上） | 记录**写成功**且无干净树守卫，但边界不释放：check 仍 `OUT-OF-BOUNDS` → `FAIL`。只多了一条假账 |

**OUT-OF-BOUNDS 与 OVERLAP 互为对方的唯一解，可行域为空。** 剩下的真出口只有两个——清空脏文件、删 worktree——**全是破坏性动作，全部在 Agent 权限之外。**

### 本轮中途的一次实拦（旁证）

写这份快照的会话自己在提交时被连坐拦了一次：另一个窗口新开的 worktree（`decision-default-firing`）还没登记，PreToolUse guard 当场 `UNREGISTERED` + `RESULT: FAIL`，拦下本轨的 `git commit`。

这次两分钟就解开了，而且解法恰好落在权限内、非破坏性——因为那棵树是**干净的**（`dirty=false, unique=0`）。它反过来印证了下面根因二的分界：**park 类逃生门只对干净树成立**。同样是连坐，干净树两分钟解开，脏陈年树把五条 lane 锁死到只剩破坏性出口。**分界不在「登不登记」，在「树干不干净」。**

（附带一条小发现：`claim` 在这里报的是 `worktree already has lane record ... (active)`，而两秒前 `check` 还报它 UNREGISTERED——那棵树的会话在这几秒里自己登记了。连坐闸对这种正常 race 也会开火，报错文案值得能区分「没人管」与「刚好晚了两秒」。）

## 三条根因

### 根因一：出口面只要求「说得清」，没要求「走得通」

`playbook/multi-contributor-protocol.md` §三½.5 的第四面写的是「报错文案即工单」——自解释。这次报错文案**是**合格工单：它写清了 OUT-OF-BOUNDS 是哪些文件、下一条命令是什么。会话照单开工，然后撞进 OVERLAP。

自解释是必要条件，不是充分条件。**一个出口指令写得再清楚，只要照做的终点仍是 FAIL，出口面就是坏的。** 出口面漏了第二个必要条件：出口得真能到绿，且落在被拦者当下的权限内。

### 根因二：文档把「对干净树成立」的解法，当成通解写给了脏树

「死锁三解」第 3 条说占位 park 是连坐的逃生门，并要求「有脏文件 / 独有提交的按其**实际改动**写真 owns」。这两句合起来，在多棵脏树的场景下**必然**造出 OVERLAP。

机制上的原因：`park` 只是登记，而互斥闸判的是事实——`holds_unmerged_work`（脏，或有 base 未收的独有提交）会把 `parked` 轨重新拉回互斥集。所以**占位 park 只对干净树是完解**；脏树 park 完边界照占，OUT-OF-BOUNDS 与 OVERLAP 一个都躲不掉。文档没写这个前提，读者只会读到「park 就能解」。

这条事实在本仓自己的 `worktree status` 里天天在打印（`STATUS-DRIFT: registered parked but the worktree still holds work ...; the boundary gate keeps its owns`），文档却没跟上。

顺带查实两条文档硬错，同源：
- 「check 容忍 parked 轨与活跃轨重叠」——只对**干净**的 parked 轨成立，脏 parked 轨照样进互斥集。
- 「生命周期合法链 `parked→ready→landed`」——**转移图里没有 `parked→ready` 这条边**（`transition_allowed` 只给了 `parked→active`），实测报 `invalid lane transition`。

### 根因三：连坐的成本，落在最没权限清理的人身上

连坐（一棵未登记树 → 全场 FAIL）2026-08-17 拍板维持，理由是保证账实一致——这个理由现在依然成立，不推翻。

问题不在连坐本身，在**成本归属**：陈年树是全仓的历史债，清理它需要破坏性权限和用户拍板；而 FAIL 是随机砸在**下一个恰好要提交的会话**头上，那个会话既没有上下文（树不是它开的），也没有权限（删树轮不到它），还正被闸拦着做不了自己的活。

而且债只进不出：`gc` 是 report-only，删除永远是人工。没有任何常设议程负责把陈年树清掉，所以「未登记树是少数且新鲜的」这个连坐赖以成立的前提，没有任何机制保证它。落后 165 的树能活着，就是这个空缺的证据。

## 改法

### 本轨落地（制度层）

1. **§三½.5 出口面升级成两个必要条件**：A 自解释（报错即工单，原有）+ B **可达性**——每个 FAIL 条件至少有一条出口，落在被拦者当下权限内且非破坏性。配「闸自检三问」，其中第三问专治本次这类**互锁 FAIL 对**。
2. **bench 案 40 入册**：连坐死锁，带上面的复现表。
3. **`kit/worktree-gc-pattern.md`**：陈年树定性为**债务**，清理责任显式归值守/控制轨常设议程，不归撞闸的会话；给「陈年」一个可判的口径。
4. **`kit/merge-checklist.md` 0b**：撞上 OUT-OF-BOUNDS/OVERLAP 时禁止就地回填绕闸，改走债务口径。
5. **`kit/babysit/BABYSIT-TEMPLATE.md`**：值守每轮加陈年树盘点；同时修掉分诊手册里那三条抄错的死锁解法。

### 归别轨（不在本轨 owns，已交单）

- **CLI 机制**：化石轨退出互斥闸（`multi-lane-docs-conflict` 轨，其 goal 已覆盖）。这是根因一二的根治——机制修好后，制度层这几条从「唯一解」退回「兜底」。
- **`kit/worktree-control-plane.md`**：死锁三解节要按上面查实的三条硬错重写（park 对脏树无效 / OVERLAP 无出口 / `parked→ready` 不存在）。该文件在 `multi-lane-docs-conflict` 轨 owns 内。
- **守卫不对称（新发现，附给同一轨）**：`edit --status ready` 有干净树守卫，`edit --status landed` **一道都没有**——脏树、有独有 commit 的树都能被直接记成 landed。闸本身没被骗过（边界照占、check 照 FAIL），但 CLI 允许写下一条假账，而假账正是「为了让闸变绿而改账」这条反模式的入口。建议给 `landed` 补对称守卫。
- **`CHANGELOG.md` / `docs/babysit.md`**：在班值守轨 owns，交单给值守。

## 未决（留给用户）

- 现场那两棵陈年树（`d35-pr117-legacy` / `legacy-crm-agent-nlp`）删不删，是破坏性动作，用户拍板。本轮不代做、不代授权。
- 连坐维持不变（2026-08-17 拍板）。本轮改的是成本归属与出口可达性，不是拆闸。
