# 会话续接握手:第 N 次接续怎么开场

> 职责边界:本篇管「同一项目换会话 / 换窗口 / 换模型续跑时,新会话第一条怎么开场」;BOOTSTRAP.md 管「第一次启动」,本篇补的是「第 N 次接续」这道缝。
> 源流:二代 communication-governance-playbook/imported/evolving-memory-system/04_protocols/global-inscope-lock-protocol-v1.md §Session Start Handshake,2026-07-07 批二移植

## 一、为什么单独要一道握手

BOOTSTRAP.md 把「第一次开项目」管好了。但项目一旦跑起来,真正高频的是**接续**:换个窗口继续、明天换台机器、这活儿切给另一个模型跑、orchestrator 主会话重开。

底层原则:**prompt 易挥发,文件系统持久。** 上一个会话脑子里的主线、锁定的下一步、暂停项,新会话一个字都不继承。如果新会话上来就闷头干,极容易接错地方——从别的阶段抄起、把暂停项当待办、把「已提议」当「已批准」。所以接续的第一条输出,**不许直接干活,先握手对齐。**

## 二、握手三步(新会话第一条输出必须完成)

新会话(换窗口 / 换模型 / 换机器都算)的第一条回复,必须依次完成:

1. **复述项目总目标**(一句话)——证明你读到了主线,不是从零猜。
2. **复述当前 phase 与下一步**——现在在哪个阶段、锁定的下一步(`Next`)是什么。
3. **请用户三选一确认**,然后才开工:
   - **按锁继续** —— 沿着当前 `Next` 往下干;
   - **显式切换** —— 换阶段 / 换任务(等于触发 `request_transition`,见 [../playbook/phase-gates.md](../playbook/phase-gates.md));
   - **重排优先级** —— 调整下一步该做什么。

三选一没回来之前,**保持 fail-closed,不擅自推进**。这一步花的十几秒,换回的是不接错地方。

## 三、握手信息从哪读(别问用户你能自己查的)

三步复述靠**读文件**得来,不靠问用户,也不靠上个会话的记忆:

| 要复述的 | 从哪读 |
|---|---|
| 项目总目标、硬约束、In-Scope / 暂停禁令 | 项目根 `AGENTS.md` |
| 当前 phase、phase_status、锁定的 `Next` | `docs/state/progress.yaml`(单一状态写者,最权威) |
| 最近发生了什么、上次卡在哪 | 最新 snapshot(`docs/snapshots/` 里时间戳最新那份)、`loop-notes.md` |
| 当前阶段具体在干什么 | 当前那张 phase 卡(`docs/phases/phase-*.md`) |
| **有没有卡等着回 agent-on 消化** | `loop-notes.md` 顶部待办位「agent-on 待消化 N 张」(结账收尾写的)——有就在开场提一句,附可粘贴的消化命令 |
| **pin 的模型档位是否还对得上** | `agent-on.lock.md` 的 pin 段 model 行——本次会话的模型若与记录的不同,提示「模型变了,建议跑 bench/capability-probe 重定保费档位」 |
| **pin 的版本落后了吗** | lock 的 pin 行 vs agent-on 最新 tag——落后就开场播报「pin vX 落后 N 版(含/不含 major),可说『agent-on 升级』」;**只提示不动手**,升级永远显式口令;HEAD 领先 tag 的未发布 commit 不算版本、不催 |
| **仪表盘还新鲜吗**(M/L) | `dashboard.html` 头部「最后更新」日期 vs progress.yaml 最近变更——落后就提一句「仪表盘陈旧,要我更新吗」 |
| **速记区有没有没整理的想法** | `thoughts-and-ideas.md` 的📥速记区——非空就在开场提一句「想法收集箱有新内容,要我整理吗」,别让零散想法烂在速记区 |
| **工作树有没有蔓延 + 结账回执是否困死枝** | `git worktree list`——死枝 ≥3 个就开场提醒清理(会话生命周期与 worktree 生命周期天然脱钩,不巡就烂);**顺手核 lock 最近 `last_settlement` 对应 commit**:`git merge-base --is-ancestor <回执commit> origin/main`(或本地 main)为假 → 回执困在 feature/worktree 死枝,当场 cherry-pick/合入 main 或重做回执,别让账本只活在旁支(IPONews `acf6e4a` 实证;与 settlement 上半场 step5 default-branch 硬门同族) |

**顺序建议**:先 `AGENTS.md` 定主线 → 再 `progress.yaml` 定当前位置 → 再最新 snapshot 补「刚才发生了什么」。三份读完,三步复述就齐了。读不到 `progress.yaml` 或它和 snapshot 打架,别硬猜——按真相源仲裁次序(见 [../playbook/truth-hierarchy.md](../playbook/truth-hierarchy.md)),以 Canonical 层的 `progress.yaml` 为准,并把冲突当场跟用户点明。

**快照陈旧标红(痛感门)**:最新 snapshot 若落后于最近 5 个 commit,开场第一行标「⚠️ 快照陈旧(落后 N 个 commit)——建议本段收口时补一份」;并在握手输出里写明**本次读的是哪份快照**——哪类快照攒几次冷启动都从没当过「被读的那份」,就是死资产实锤,照砍(不许用感觉替代不变量)。

## 四、一个照抄即用的开场模板

```md
[接续握手]
总目标:<一句话,读自 AGENTS.md>
当前阶段:<phase 名> / 状态:<in_progress 等,读自 progress.yaml>
锁定的下一步:<Next,读自 progress.yaml>
最近进展:<一句话,读自最新 snapshot>
读的快照:<文件名>(落后最近 commit N 个;N≥5 行首标 ⚠️陈旧)

请三选一后我再开工:
① 按锁继续(沿当前下一步干)
② 显式切换(换阶段/换任务)
③ 重排优先级
```

## 五、和「第一次启动」的分工(别混)

- **BOOTSTRAP.md**:项目从无到有——搭骨架、填 `AGENTS.md`、写第一张 phase 卡。一个项目只走一次。
- **本篇握手**:项目已存在,只是换了会话在续跑——不搭骨架,只对齐后接着干。每次新会话都走一次。

一句话带走:**第一次靠 BOOTSTRAP 搭地基,之后每一次靠握手对表——地基不用重砌,但表必须每次对。**
