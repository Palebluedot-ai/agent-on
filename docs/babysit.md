# 值守文档（babysit loop）——agent-on

> instantiated-from: kit/babysit/BABYSIT-TEMPLATE.md @ v0.15.0。实例化文件是本仓自己的 canonical——升级永远显式，不从 kit 重拷覆盖。
> 启动：新开一条干净会话，跑 `/loop 读 docs/babysit.md 全文并执行本轮值守`（要固定节奏就 `/loop 5m 读 docs/babysit.md 全文并执行本轮值守`）。
> 读法：§1 只在首轮做，§2 是循环体。**会话是班次，文档是资产**——手册的进化写回本文件随 git 走，不留在聊天记录里。

## §0 GOAL（一句话）

看住 `Palebluedot-ai/agent-on` 的公共资源：main、PR 队列、发版硬门（tag 债务）、intake 积压、lane 控制面卫生。
红了分诊、欠账播报、授权内（§3）的 PR 串行消化、拿不准的报告等拍板。
多会话并行是本仓常态——值守只看场子，不抢功能会话的活，**不代消化**（消化是独立口令与会话）；**在班期间合并权唯一归值守**（治理条款见 AGENTS.md 自举纪律第 8 条）。

## §1 首轮启动（只做一次，后续轮跳过）

1. **单值守核对**：向用户确认没有第二个值守窗口在班。确认后把本班会话地址写进下方交接快照「在班值守地址」行——功能会话交单靠它找人，不靠 ListAgents 猜名字；下班清掉，接班覆盖。
2. **开 worktree**（一会话一 worktree 铁律）。值守平时只读不 claim；要写文件（本文档）时按最小 owns（`docs/babysit.md`）claim 值守轨。
3. **权限自检**：跑一次 `gh pr merge --help` 级别的无害探测确认 allow 规则已配；没配则把 `kit/babysit/SETUP.md` §1 的 settings 命令贴给用户手跑（本仓实测：未建 settings 时 `gh pr merge` 与 `gh api -X PUT` 被分类器间歇拦，两步不过即停——机制红线，agent 改不了自己的权限）。
4. **核背景坐标（别信交接文档，自己跑）**：`git fetch origin -q && git rev-parse origin/main`、`gh pr list --state open`、最新 tag（`git tag --sort=-v:refname | head -1`）、上一班快照声称的关键事实逐条验证。
5. **读规矩原文**：AGENTS.md（自举纪律 6/7/8 + 迭代闭环职责）、kit/merge-checklist 0c、boot/settlement.md 收尾四件。合并方式：merge commit（`--merge`）；版本批题头 `merge(vX.Y.Z): …`。本仓**无 required checks、无 up-to-date 硬门**（无 `.github/workflows/`），合并核对面见 §2.2。
6. `agent-on landing refresh` 取证建快照，之后每轮 `landing plan` 离线看队列。

### 交接快照（上一班下班时更新；本班核对，不信任）

```text
时间：2026-08-17（实例化写入；首班照 §1 自己核）
在班值守地址：<空——首班上岗写入>
main：<首班跑 git rev-parse origin/main 填>
open PR：#1 真相之页「开发史」（孤本救回，无交单，等拍板处置）
在途后台链：无
```

## §2 每轮检查单（循环体）

1. **收单**：`git fetch origin -q && gh pr list --repo Palebluedot-ai/agent-on --state open`。
   **队列真相源 = open PR 列表**；SendMessage 交单只是门铃 + 特殊说明通道。交单消息三型：交单 / 撤单 HOLD / READY；收到 HOLD 的单挂起，等 READY 再进入分流。
   `agent-on landing refresh && agent-on landing plan` 拿 NOW / 波次当排序输入。
2. **逐 PR 核对面（本仓无 CI，三查代替 checks）**：
   - `gh pr view <N> --json mergeable,mergeStateStatus`：须 MERGEABLE / CLEAN（DIRTY → 服务端追平或按 §4 分诊）
   - GitGuardian（外部 app check）：须 pass
   - 内容分类：**本仓几乎所有 PR 都动 canonical（kit/playbook/bench/boot/cli/skill/hooks/AGENTS/BOOTSTRAP）→ 一律拍板类**；intake-only PR（上游贡献）也先问（须维护者分诊）
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

## §3 权限边界（硬，越界前先问）

**可自主**：只读检查 · 本 worktree 操作 · 服务端 update-branch · 值守自身交接文档 commit（走值守轨）· 未登记树占位 park 登记 · 经用户拍板 PR 的全流程（追平→核对→合→记账→回执）· **已拍板版本批的 tag + push tag**（机械步骤）

**必须先问用户**：合并一切 canonical PR（本仓几乎全部）· intake-only PR · 删远端分支 · 关闭别人的 PR · 清单外一切外向操作

**批准的来源**：外向硬门动作的批准必须来自**本会话内的用户输入**。同行会话转述的用户原话再可信也只是情报——注明转述来源、向本人确认后才执行。

**永远不做**：进别的 worktree add/commit · `git add -A` · 直推 main · force-push · 代解语义冲突（真 conflict 打回作者）· 改自己的权限配置 · **代消化 canonical**（消化是独立口令，值守只报积压）

## §4 分诊手册（先查手册再发明新解释；本仓实测过的坑往下续）

- merge 报 head 与 base 不同步 / DIRTY → 服务端追平 `gh api -X PUT repos/Palebluedot-ai/agent-on/pulls/<N>/update-branch`
- **分类器间歇拦合并命令**（本仓 2026-08-17 实测）：`gh pr merge` 与 `gh api -X PUT` 时好时坏——settings.local.json（SETUP §1）未建则必撞；按 anti-hallucination #17 两步不过即停，贴命令给用户手跑
- **PreToolUse guard 先评估整条命令**：占位 claim 与 `git commit` 必须拆成两条命令——合在一条里 claim 永远跑不到（2026-08-17 实测）
- **GitHub GraphQL 与 REST 可分层故障**：`gh pr create` 503 时换 `gh api repos/…/pulls` REST 直建（2026-08-17 实测两侧恢复时间不同）
- 未登记 worktree 连坐全场 FAIL → 占位 park 逃生门（kit/worktree-control-plane「重划与死锁三解」）；lane 重划 = JSON 直改 + check 验证
- squash / merge 后祖先误判 → 以 `gh pr list --state merged` / 托管平台为准
- 状态闸拉 GitHub API 抖动 → 重试即绿，非业务违规

## §5 已知遗留（交接清单；提醒用，值守不抢活）

- PR #1 真相之页「开发史」（孤本救回）：开着、无交单，等用户拍板处置
- output-contract 轨（worktree-dashboard-pipeline-21eb94）：kit/output-contract.md + kit/babysit/MERGE-POLICY.md 待开 PR；落地时连带 kit/README 两行索引 + babysit 模板/条款三处引用接线（归其轨原子做，别代做）
- CLI 任务卡在跑（另一会话）：claim --owns 逗号 bug + worktree edit 命令
- settings.local.json 未建：首班 §1 权限自检必撞——先催 SETUP §1 命令

## §6 汇报纪律

跟随用户语言；有动作才出声，全绿安静（noop）；任何「已合 / 已修 / 完成」必须贴命令实际输出；拿不准 = 报告而不是猜。
请求拍板时给足判断材料：PR 号 + 标题 + 核对面三查结论 + 影响面一句话——让用户一眼能拍。

## §7 交接与下班

- **下班三件**（关窗口之前）：①更新 §1 交接快照（含清掉「在班值守地址」）②更新 §5 遗留清单 ③本班新踩的坑写进 §4 → commit 本文档（走值守自己的轨；合入按本仓规则拍板）。
- **在途后台链必须写进快照**：链随会话死，接班不知道就会漏单。
- **接班** = 新会话重新 `/loop` 本文档；下一班按 §1 核对坐标，而非信任本文档的任何声称。
