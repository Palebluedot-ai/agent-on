# 2026-08-20 · 投影漂移：判定错与幽灵 lane 是同一个病

> 职责边界：本快照记一次结构性决策——把「原件 / 投影」立成第一类区分，并给投影配一条对账命令。机制正文在 [playbook/truth-hierarchy.md](../playbook/truth-hierarchy.md)「仓内投影」一节，实现在 `cli/src/drift.rs`（<!-- src: cli/src/drift.rs#base_verdict -->）与 `cli/src/main.rs`（<!-- src: cli/src/main.rs#Drift -->）。本文件只留为什么链与当日实测数字。
> 触发：用户 2026-08-20 原话——「为什么会判定错？怎样会判定错？然后怎么样是幽灵 lane 之类的？」并指出本仓「低内聚、高耦合」是通篇的，要的是架构层的解，不是这一单的解。

## 一句话

**这个仓的错误率不由文档质量决定，由投影数量决定。** 一个机器事实在仓里有几十份副本，副本不知道自己是副本，也没有任何东西在它和真身之间对账——于是五份全对、全过期得不一样、全不指向源，已经足够产生一个错误结论。

## 一、先把两类记录分开（这是内聚的切口）

| | 原件 | 投影 |
|---|---|---|
| 是什么 | 这里第一次产生的判断 | 别处事实的副本 |
| 例子 | 「恒红的闸等于没有闸」「角色不是架构原语」 | 「guard 拦哪些 git 操作」「这条 lane 在哪个 branch」 |
| 真身在哪 | 就在这里 | `cli/src/*.rs` / git / `oncall.json` |
| 会不会过期 | **不会**——当时的判断永远是当时的判断 | **会，且没有声音** |
| 读者分得清吗 | — | 分不清：两类写在同一份文件、同一种语气、同一个时态 |

本仓一直把这两类当同一种东西写。`AGENTS.md` 文档纪律第 3 条「可以重复关键事实，不为了去重而交叉引用」是**对原件成立、对投影致命**的一条规矩：原件重复只是啰嗦，投影重复是把一个会过期的事实复制了 N 份，而 N 是多少、在哪里，没有任何人知道。

**当日实测（同口径：仅 md，排除 `.git` / `target` / `legacy` / `intake`，共 139 份）**：

| 机器事实 | 有多少份 md 在谈它 | 定义它的地方 |
|---|---|---|
| `landed` 的语义 | **70 / 139**（半数以上） | `cli/src/worktree.rs` 一处 |
| `owns` 边界规则 | 47 | 同上一处 |
| `PreToolUse` 拦什么 | 32 | `cli/src/guard.rs` 一处 |
| `OUT-OF-BOUNDS` 成因 | 20 | 同上 |

## 二、判定错的三段式：不需要任何一份文件写错

1. 一个机器事实有 N 份投影（见上表）；
2. 投影**不带回指，或只回指到文件**；
3. 读者落在任意一份上，把它当事实。

第 2 条是要害。本仓其实**已经自发形成了回指习惯**——多份快照头部写着「机制正文在 X、实现在 Y」。但它写在散文里、指到**文件**、没有任何东西验它。而**文件路径不会因为里面的行为变了就失效**。

实测反例就在本仓：`snapshot/2026-08-20-gate-exit-reachability.md` 确实点名了 guard 的实现文件，指针至今有效——而当时基于它的那条行为断言是错的，最后由值守独立读源码复核才纠正（`commit` 与 `push` 一起进 `commit_push_dirs`，<!-- src: cli/src/guard.rs#commit_push_dirs -->）。**指到文件的指针，不能证明任何一句关于行为的话。**

这与两天前刚立的「[转述 ≠ 出处](../playbook/multi-contributor-protocol.md)」是同一根问题的两半：那半管**跨窗口**转述要溯源（纪律），这半管**仓内**副本要带可验的锚（机械闸）。按本仓元原则第七条，没有闸的规则是纸面的——所以那条纪律必须配这条闸。

## 三、幽灵 lane：同一个病，换一条介质

lane 记录 12 个字段，只有 4 个是原件（`id` / `goal` / `owns` / `depends_on`）。`worktree` / `branch` / `base` / `base_sha_at_claim` / `status` 全是**git 已经拥有的事实的副本**，而且**没有任何对账回路**。

`truth-hierarchy` §五¾.2 早就写过「长期状态文件不 self-pin live HEAD」——那条规矩是为文档写的，**从来没有被应用到本仓自己的工具身上**。lane 台账干的正是它禁止的事。

三个物种，当日在本仓全部实测到：

| 物种 | 记录说 | git 说 | 后果 |
|---|---|---|---|
| **幽灵 A：referent 没了** | worktree 在某路径 | 目录已不存在 | 5 条。`landed/parked` 的只是垃圾（分类 `metadata`，不连坐）；若 status 还是 live 则整场 FAIL |
| **幽灵 B：base 漂了** | `base = main` | 本地分支，每次合流都在动 | 2 条。边界闸拿它 diff，一炸就是全树 OUT-OF-BOUNDS |
| **幽灵 C：branch 对不上** | `docs/v0170-release` | 树在 `claude/gracious-shtern-07ae02` | 1 条，且**现有 `check` 把它连同 `reclaim safe` 一起打印出来** |

幽灵 C 最值得记：`clean` / `merged` 这些安全判据是**在真树上量的**，唯独 `branch` 这一个字段**只被打印、从不被核**。于是审计报告用测量的口气说出了一句副本。人照着 `reclaim safe` 去 `git worktree remove`，脑子里保留的是记录上那条分支，手上动的是另一条。**这就是判定错在机器面的同构：一个字段带着测量的权威，其实是一份拷贝。**

## 四、决策：一条命令，一个问题

`agent-on drift` 只回答一个问题——**哪些存下来的副本，已经和它的源对不上了？**

- **台账面**：`GHOST-LANE` / `BRANCH-DRIFT` / `UNSTABLE-BASE` / `UNRESOLVABLE-BASE`
- **文档面**：`ANCHOR-BROKEN`（锚指的东西没了或符号没了）/ `ANCHOR-MISSING`（散文点名了实现文件却没锚）
- **每一行带 owner 与一条非破坏出口命令**
- **默认 exit 0**；`--strict` 才非零

### 三个设计选择

| 选择 | 选了什么 | 否掉了什么 | 为什么 |
|---|---|---|---|
| 拦不拦 | **只报不拦** | 并进 `worktree check` 的 fail | 账实不符是**别人的账**。拿它拦本会话的 commit，正是[案 40](../bench/cases/40-gate-exit-unreachable.md)「出口在权限外的闸 = 死锁」那个老病。清账是常设议程，不是撞闸者的意外任务 |
| 锚指到哪 | **符号** | 文件 | 文件路径不会因为行为变了而失效——本仓已有的「机制正文在 X」全是文件级，实测挡不住这次的判定错 |
| 管到哪 | **只管机器行为断言** | 全文档 | 方法主张不会过期，给它上锚是噪音。豁免面写死：`intake/`（承接层原始素材）、`legacy/`（归档）、`CHANGELOG.md`（历史记录，不声称「现在是这样」） |

## 五、证据

```text
$ cargo test          201 passed / 0 failed（本轮新增 8 条 drift 单测）
$ cargo clippy --all-targets   0 warning

$ agent-on drift      16 drift row(s)，exit 0
  LEDGER  8 行：GHOST-LANE ×5 · UNSTABLE-BASE ×2 · BRANCH-DRIFT ×1
  DOCS    8 行：ANCHOR-MISSING ×8（分布在 6 份文件，全部另有 owner）
```

幽灵 C 的源头复核（不靠转述）：

```text
$ lane owns-split-and-lane-edit → worktree=…/epic-hertz-483308  branch=docs/v0170-release
$ git -C …/epic-hertz-483308 rev-parse --abbrev-ref HEAD  →  claude/gracious-shtern-07ae02
$ agent-on worktree check | grep owns-split-and-lane-edit
  - owns-split-and-lane-edit [landed] docs/v0170-release | … | reclaim safe
```

这 16 行在本轮之前**全部不可见**：`worktree check` 一条都不报（它答的是另一个问题——边界撞没撞），`gc --dry-run` 也不报。

## 六、诚实边界

- **`drift` 现在是报告，不是闸**。没接 CI（本仓首个 CI 归 `text-contrast-scope` 轨），所以它今天满足不了元原则「没有闸的规则是纸面的」。接线单已交值守。按[案 41](../bench/cases/41-duty-window-double-decision.md) 第 3 条，这类「挂好等接线」的机制必须留可查的激活痕迹——本节第五段那 16 行就是 day-0 存量，下次巡检对不上这个数就是没在跑。
- **`ANCHOR-MISSING` 的判据是启发式**：只认「散文里点名了 `cli/src|tests/*.rs`」。一句不点名文件的行为断言（本仓绝大多数就是这样）它抓不到。它挡的是**已经想指、但指歪了**的那一类，不是全部。
- **锚点验的是「符号还在」，不是「这句话还对」**。符号在而语义变了，锚仍绿。这是成本与收益的取舍：它把「指针有效」从 0 位信息升到 1 位，没有升到 2 位。
- **今日 8 条 ANCHOR-MISSING 我一条都没修**——全在别人的 owns 里，按 gc-pattern「撞闸者只报债不还债」交出去。
- **`--strict` 今天会红**。这是有意的：它诚实地说出存量，不是失败。
- **`ANCHOR-MISSING` 有一类已知误报**：散文只是把实现文件当**地盘**点名（「那要改 X，是别人的 owns」），并没有对它的行为下断言，闸照样要求上锚。留着不修——它的代价是一条锚，收益是判据不含糊；把「行为断言」和「地盘点名」分开需要读懂句子，那是模型的活，不是闸的活。
- **写在示例里的锚也一起验**。案 43 修法一节把锚的写法当范例展示了一次，那一处同样被解析、被验证——这是有意的：范例写成假的，读的人会照着抄假的。（本条自身就是实测：初稿在这句里写了一个占位形状的锚，`drift` 当场判它 ANCHOR-BROKEN。）

## 七、未做（本轨显式不做）

- **接 CI**：归 `text-contrast-scope` 轨（它正在做本仓首个 CI）。
- **给存量 8 条补锚**：归各自 owner。
- **`oncall.json` 与 dashboard 的投影对账**：同一个模式还能扩到这两面，但本轨不扩——先让一条介质跑实。
- **`worktree check` 里 `branch` 字段改成「记录 / 事实」双列**：那要改 `cli/src/worktree.rs` 里承载打印字段的那个结构（<!-- src: cli/src/worktree.rs#LaneAudit -->），是 live 轨的 owns，本轨不越界。已在交单里点名。
