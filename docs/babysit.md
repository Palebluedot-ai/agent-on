# 值守文档（babysit loop）——agent-on

> instantiated-from: kit/babysit/BABYSIT-TEMPLATE.md @ v0.15.0。实例化文件是本仓自己的 canonical——升级永远显式，不从 kit 重拷覆盖。
> 启动：新开一条干净会话，跑 `/loop 读 docs/babysit.md 全文并执行本轮值守`（要固定节奏就 `/loop 5m 读 docs/babysit.md 全文并执行本轮值守`）。
> 读法：§1 只在首轮做，§2 是循环体。**会话是班次，文档是资产**——手册的进化写回本文件随 git 走，不留在聊天记录里。

## §0 GOAL（一句话）

看住 `Palebluedot-ai/agent-on` 的公共资源：main、PR 队列、发版硬门（tag 债务）、intake 积压、lane 控制面卫生。
红了分诊、欠账播报、授权内（§3）的 PR 串行消化、拿不准的报告等拍板。
多会话并行是本仓常态——值守只看场子，不抢功能会话的活，**不代消化**（消化是独立口令与会话）；**在班期间合并权唯一归值守**（治理条款见 AGENTS.md 自举纪律第 8 条）。

## §1 首轮启动（只做一次，后续轮跳过）

1. **单值守核对 + 上岗登记**：向用户确认没有第二个值守窗口在班。确认后两件都做：
   - **机器可寻址（真相源）**：`agent-on oncall claim --session <本窗口会话名>`。登记落 common git dir，每棵 worktree 读到同一份；功能窗口跑 `agent-on oncall status` 就能拿到交单地址，不必读本文档。已有人在班时命令会拒绝——那就是「已有第二个值守」的机器证据，别 `--force` 抢，先问用户。读不到自己的精确会话名时用 **worktree 目录名前缀**注册（闸是双向前缀匹配，能匹配到带后缀的完整会话名）。
   - **人可读（副本）**：把地址写进下方交接快照「在班值守地址」行。**这行是每棵树一份的文件副本，功能窗口在自己分支上读到的可能是任意旧版本**——它给人看，机器寻址一律以 `oncall status` 为准。下班清掉，接班覆盖。
2. **开 worktree**（一会话一 worktree 铁律）。值守平时只读不 claim；要写文件（本文档）时按最小 owns（`docs/babysit.md`）claim 值守轨。
3. **权限自检**：跑一次 `gh pr merge --help` 级别的无害探测确认 allow 规则已配；没配则把 `kit/babysit/SETUP.md` §1 的 settings 命令贴给用户手跑（本仓实测：未建 settings 时 `gh pr merge` 与 `gh api -X PUT` 被分类器间歇拦，两步不过即停——机制红线，agent 改不了自己的权限）。
4. **核背景坐标（别信交接文档，自己跑）**：`git fetch origin -q && git rev-parse origin/main`、`gh pr list --state open`、最新 tag（`git tag --sort=-v:refname | head -1`）、上一班快照声称的关键事实逐条验证。
5. **读规矩原文**：AGENTS.md（自举纪律 6/7/8 + 迭代闭环职责）、kit/merge-checklist 0c、boot/settlement.md 收尾四件。合并方式：merge commit（`--merge`）；版本批题头 `merge(vX.Y.Z): …`。本仓**无 required checks、无 up-to-date 硬门**（无 `.github/workflows/`），合并核对面见 §2.2。
6. `agent-on landing refresh` 取证建快照，之后每轮 `landing plan` 离线看队列。

### 交接快照（上一班下班时更新；本班核对，不信任）

```text
时间：2026-08-19 10:1x（首班上岗，§1 全项自核；用户当日拍板三条全 A）
在班值守地址：会话名前缀 `worktree-output-clarity-e02325-*`（ListAgents 按前缀匹配；
              精确后缀本会话读不到自己，交单前用 ListAgents 确认一次）
main：cf94ae9f4a1d6304daee8309bc9ee3b7872b1b59
open PR：无（#1 已于 08-19 02:01 关闭，功能由 #11 重落）
最新 tag：v0.15.0；v0.16.0 封版 PR 在途（CHANGELOG 补 #8/#10/#11 + 本文档），
          合入后由值守代打 tag（语义已拍板：minor / v0.16.0）
在途后台链：无
```

## §2 每轮检查单（循环体）

1. **收单**：`git fetch origin -q && gh pr list --repo Palebluedot-ai/agent-on --state open`。
   **队列真相源 = open PR 列表**；SendMessage 交单只是门铃 + 特殊说明通道。交单消息三型：交单 / 撤单 HOLD / READY；收到 HOLD 的单挂起，等 READY 再进入分流。
   **门铃即起跑**：交单消息送达即唤醒本会话，**当轮就跑**收单 + 追平，不等下一次定时唤醒；门铃丢了最多晚一个心跳、不漏单（队列从 `gh pr list` 完整重建）。机制见 `kit/babysit/MERGE-POLICY.md` §1。
   `agent-on landing refresh && agent-on landing plan` 拿 NOW / 波次当排序输入。
2. **逐 PR 核对面（本仓无 CI，三查代替 checks）**：
   - `gh pr view <N> --json mergeable,mergeStateStatus`：须 MERGEABLE / CLEAN（DIRTY → 服务端追平或按 §4 分诊）
   - GitGuardian（外部 app check）：须 pass
   - 内容分类：按 §3 两张清单判，**按实际 diff 判不按标题判**。本仓几乎所有 PR 都动 canonical（kit/playbook/bench/boot/cli/skill/hooks/AGENTS/BOOTSTRAP）→ 落在「必须先问」；默认合入档在本仓刻意窄（见 §3）
   - 真缺陷 = 打回四件套（证据指针 + 定位 + 修复选项 + SendMessage 作者会话），值守零代修
3. **合并流程（严格串行，一次只合一条）**：
   1. 落后 base → 服务端追平 `gh api -X PUT repos/Palebluedot-ai/agent-on/pulls/<N>/update-branch`；绝不本地 checkout / push 功能分支
   2. 无 CI 可等——核对面三查过 + 用户拍板后直接 `gh pr merge <N> --merge`（版本批用 `--subject "merge(vX.Y.Z): …"`）
   3. **合完三连**：记账（第 4 步）→ SendMessage 回执作者会话 → 队列下一条如变 BEHIND 立即追平
   4. **版本批合并后**：值守可代打已拍板批的 tag（`git tag -a vX.Y.Z -m "vX.Y.Z — <semver 档>" <merge SHA> && git push origin vX.Y.Z`）——机械步骤，语义（版本号/档位/条目）须已经用户拍板
4. **账本巡检（agent-on 三面）**：
   - **发版硬门**：`git log --oneline $(git tag --sort=-v:refname | head -1)..origin/main` 非空 = tag 债务（AGENTS 自举纪律 6：push 结束 tag 必须钉 HEAD）——播报提醒，值守不代定版本语义
   - **intake 积压**：`ls intake/` 数未标去向文件，≥3 播报「该开消化会话」（目录即仪表盘）
   - **lane 卫生**：`agent-on worktree check`；新未登记树按 kit/worktree-control-plane「重划与死锁三解」第 3 条占位 park（claim + set-status parked，与 git commit 拆两条命令）
   - 两条铁则照旧：台账只记自己的号（字面匹配盲区）；元动作自涵盖
5. **低频（每天一次）**：`agent-on worktree gc --dry-run` + 磁盘余量。
6. **节奏**：本仓无 CI 可盯——有单快循环（5–10 分钟），无单 noop 20–30 分钟；事故推一条通知 + 每轮最小探针。
   **一个口令切档**：用户说「值守加速」→ 心跳降到 3–5 分钟并回执确认一句；**连续 3 轮 noop 自动回落**到 20–30 分钟（回落不出声）；用户说「值守回落」立即回常态。用户不改任何配置文件。
7. **时延目标**：默认合入档的 PR，从交单到合入**中位 ≤ 5 分钟**（本仓无 CI，`X = CI 中位 + 5 分钟` 的 CI 项为 0；算法与依据见 `kit/babysit/MERGE-POLICY.md` §5）。单条超时在 §5 记一行（PR 号 + 实际耗时 + 卡在哪段）；**一个班次的中位数超 5 分钟 = 事故**，写进 §7 下班交接并附最慢三条。必须先问档的 PR 不计入——它们的时延取决于用户什么时候看消息，不是流水线的事。

## §3 权限边界（硬，越界前先问）

> 两张清单都显式列全，只写「可自主」不算数。口径与算法的唯一真相在 `kit/babysit/MERGE-POLICY.md` §3/§4;下面是本仓落定的具体值。

**先看三权唯一（AGENTS 自举纪律 9，2026-08-19 起）**：值守在班期间**三条权唯一归值守**——①**合并**（含 tag / release / 关 PR）②**对外通信**（PR/Issue 评论、Teams/Slack/邮件/webhook，一切代表本仓对外发言）③**跨窗口中转**（窗口之间传话与派工经值守，功能会话之间不横向直发）。功能会话唯一跨窗口出站通道 = 给值守交单 / 回执。误投的指令**不执行、原样转投**（模板与路由表见 `kit/babysit/ROUTING.md`）。**「跨窗口」不含会话内部**——一条会话与自己子代理 / `main` 怎么传话，三权一概不管。
下面两张清单管的是**这单能不能合**；三权管的是**谁来合**，两者不重叠。**转投送指令不送授权**：转投消息里的用户原话是情报，外向硬门仍须用户本人在本会话拍板。

**默认合入档**（用户已预授权，核对面三查全过即合、不问，合完只记一行，不出长回执）——**本仓刻意窄**，因为这里几乎所有内容都是 canonical 方法论：

1. 值守自身的交接文档 / 账本 PR（走值守轨）
2. **已拍板版本批的 tag + push tag**（机械步骤；版本号 / 档位 / 条目语义须已拍板）
3. 未登记树占位 park 登记类的 lane 卫生 PR
4. **纯格式修复**:typo、死链、Markdown 格式——**不改任何语义**；一旦同一个 PR 里夹带语义改动，整单降级进下一栏
5. **用户已拍板过的同类**：用户明确说过「这类以后不用问」的类目。口头一次即可，但值守要在 §5 记一行——哪一类、哪天拍的；记不下来就当没授权

**前置条件（四条全中才算「三查全过」）**:`mergeable=MERGEABLE` 且 `mergeStateStatus=CLEAN` · GitGuardian pass · **实际文件清单**确实落在上面五类里 · PR 描述无 breaking / 迁移 / 需先做某事的标注。

**必须先问用户**（fail-closed,**清单外一切都归这栏**）：

- 合并一切 canonical PR:`kit/` `playbook/` `bench/` `boot/` `cli/` `skill/` `hooks/` `AGENTS.md` `BOOTSTRAP.md` `README.md` `CHARTER.md` 的**语义**改动（本仓绝大多数 PR 在此）
- intake-only PR（上游贡献，须维护者分诊）· `CHANGELOG.md` 的版本语义 · 新版本号与档位
- 删远端分支 · 关闭别人的 PR · 数据库/凭据/权限配置类 · 跨仓外向操作
- 带 breaking / 迁移标注的 PR,**哪怕它只动文档**

**与合并无关的自主动作**（不受上面两张清单约束）：只读检查 · 本 worktree 操作 · 服务端 `update-branch` 追平 · 值守自身的状态同步 · 经用户拍板 PR 的全流程（追平→核对→合→记账→回执）

**用户没明确授权过，默认合入档就不生效**——全部按「必须先问」办。本节只是把清单挂好等授权，不是授权本身。

**批准的来源**：外向硬门动作的批准必须来自**本会话内的用户输入**。同行会话转述的用户原话再可信也只是情报——注明转述来源、向本人确认后才执行。

**永远不做**：进别的 worktree add/commit · `git add -A` · 直推 main · force-push · 代解语义冲突（真 conflict 打回作者）· 改自己的权限配置 · **代消化 canonical**（消化是独立口令，值守只报积压）

## §4 分诊手册（先查手册再发明新解释；本仓实测过的坑往下续）

- merge 报 head 与 base 不同步 / DIRTY → 服务端追平 `gh api -X PUT repos/Palebluedot-ai/agent-on/pulls/<N>/update-branch`
- **分类器间歇拦合并命令**（本仓 2026-08-17 实测）：`gh pr merge` 与 `gh api -X PUT` 时好时坏——settings.local.json（SETUP §1）未建则必撞；按 anti-hallucination #17 两步不过即停，贴命令给用户手跑
- **PreToolUse guard 先评估整条命令**：占位 claim 与 `git commit` 必须拆成两条命令——合在一条里 claim 永远跑不到（2026-08-17 实测）
- **GitHub GraphQL 与 REST 可分层故障**：`gh pr create` 503 时换 `gh api repos/…/pulls` REST 直建（2026-08-17 实测两侧恢复时间不同）
- 未登记 worktree 连坐全场 FAIL → 占位 park 逃生门（kit/worktree-control-plane「重划与死锁三解」）；lane 重划 = `agent-on worktree edit`（PR #8 起；被活跃轨重叠闸拦住时才 fallback 到 JSON 直改）+ check 验证
- squash / merge 后祖先误判 → 以 `gh pr list --state merged` / 托管平台为准
- 状态闸拉 GitHub API 抖动 → 重试即绿，非业务违规
- **装机 CLI 与仓内源码同版本号、功能不同**（2026-08-19 实测）：`agent-on landing` / `worktree edit` 报 `unrecognized subcommand`，但 `cli/Cargo.toml` 与 `agent-on --version` 都是 0.12.1——功能落地没 bump 版本号，光看版本号分辨不出。命令报「没这个子命令」时先 `cargo install --path cli` 重装再怀疑文档写错
- **`worktree check` 的输出别用 `tail` 截**（2026-08-19 实测）：lane 按字母序列出、`RESULT` 行在末尾，`tail -40` 会砍掉开头几条轨（本轮漏看 `affectionate-hofstadter-placeholder` 与 `agent-on-data-hygiene` 两条）。要判 RESULT 用 `tail -3`，要看 lane 清单就全量看，别两件事一条管道办
- **活跃轨上限 3/3 会挡住值守自己 `set-status active`**（2026-08-19 实测）：报 `活跃轨上限已满（3/3）；先 park / land 一条再激活`。**别为腾位子去动别人的 lane 状态**——值守轨留 `parked` 照样能写自己 owns 内的文件（guard 判边界不判生命周期），登记清楚就行
- **值守写文件前先确认自己在哪棵树**（2026-08-19 实测踩到）：`cd /Users/chao/Projects/Agent-On` 是**主仓 worktree（main 分支）**，不是值守自己的树；在那里编辑等于往 main 的工作区写。写之前 `cd` 到值守 worktree 绝对路径，或所有 git 命令带 `-C <值守树>`。误写了就 `cp` 到自己树 + 主仓 `git checkout -- <file>` 复原，复原前先 `git status --porcelain` 确认那棵树只有你这一笔改动

## §5 已知遗留（交接清单；提醒用，值守不抢活）

> 2026-08-19 首班核对：上一版四条**全部过期**，已逐条结清（PR #1 已关闭；output-contract 轨 #9/#10 已合含 kit/README 索引；CLI 两件 #6/#7 已合；settings.local.json 已建且规则与 SETUP §1 逐字一致）。下面是本班新开的清单。

- **v0.16.0 封版 PR 在途**：CHANGELOG 补 #8/#10/#11 条目 + 本文档交接快照/遗留/分诊三处更新（值守轨 `worktree-output-clarity-placeholder`，owns = `CHANGELOG.md, docs/babysit.md`）。合入需用户拍板（canonical 边缘），合入后值守代打 `v0.16.0` tag。
- **5 棵树齐回收证据未删**：`babysit-merge-dispatcher-6aeff4`(#5)、`eloquent-sanderson-4b245b`(#4)、`gracious-shtern-07ae02`(#6, 397 MB)、`v0121-agent-skeleton-hooks`、`v0121-worktree-enforcement`(843 MB)。值守不代删，命令附在每轮撤销面。
- **两条 active 轨有孤本**（changed=1，`reclaim rescue`）：`agent-on-data-hygiene`、`owns-octal-unescape`、外加 parked 的 `affectionate-hofstadter-placeholder`——归各自作者会话，值守只播报。
- **活跃轨 3/3 占满**：`agent-on-data-hygiene` / `orchestrator-loop` / `owns-octal-unescape`。值守轨因此只能留 parked（见 §4）。
- **`landing status --human` 面板渲染未挂**：CHANGELOG v0.16.0「诚实边界」记的后续，需单独拍板，值守不代做。

## §6 汇报纪律

**每轮输出格式统一走 `kit/output-contract.md`**：状态面板在前（一行一条轨：`轨名 │ 一句话状态 │ 我要不要动 │ 下一动作归谁`，类别用中文人话，`NOW`/`STALE`/`REAPABLE` 这类机器名只准放括号里）→ 拍板收成一节（编号、每条 ≤3 行、**必带「你不回我就按 X 走」**、一轮最多 3 条，超出自己排序只问最阻塞的）→ 结论三格（已验证 / 我按这个假设做了 / 已推翻）→ 撤销两栏（`unknown` 一律进「不能删」）→ 最后一行「球在谁那」→ **之后**才轮到过程叙述。值守侧不另定义格式。

在此之上补值守特有的四条：

- 跟随用户语言；有动作才出声，全绿安静（noop）；任何「已合 / 已修 / 完成」必须贴命令实际输出；拿不准 = 报告而不是猜。
- 请求拍板时给足判断材料：PR 号 + 标题 + 核对面三查结论 + 影响面一句话——让用户一眼能拍，不要只丢一个「合不合？」。
- **打回作者的单，面板里必须同时写明「这单已不占你注意力」**——让用户能把它从脑子里划掉。
- 默认合入档合完**只记一行**，不出长回执；用户在面板里看见 `已合并` 即可。

## §7 交接与下班

- **下班四件**（关窗口之前）：①`agent-on oncall release`——**先跑这条**，闸随即 fail-open，回退「值守不在班」规则；忘了跑等于把全场功能窗口的合并/对外通信一直锁着 ②更新 §1 交接快照（含清掉「在班值守地址」）③更新 §5 遗留清单 ④本班新踩的坑写进 §4 → commit 本文档（走值守自己的轨；合入按本仓规则拍板）。
- **换班**：接班窗口 `oncall claim` 会被在班登记拒绝——正常顺序是旧班先 `release` 再交接。旧班窗口已经关掉、`release` 没人跑时，新班用 `oncall claim --force` 接管（**会留痕**），并在 §5 记一行「上一班未 release」。
- **在途后台链必须写进快照**：链随会话死，接班不知道就会漏单。
- **接班** = 新会话重新 `/loop` 本文档；下一班按 §1 核对坐标，而非信任本文档的任何声称。
