# 值守文档（babysit loop）——<项目名>

> 接入：本文件由 `kit/babysit/BABYSIT-TEMPLATE.md` 复制为项目仓 `docs/babysit.md`，填掉全部 `<占位符>`。
> 启动：新开一条干净会话，跑 `/loop 读 docs/babysit.md 全文并执行本轮值守`（宿主支持动态自定节奏时优先；要固定节奏就 `/loop 5m 读 docs/babysit.md 全文并执行本轮值守`）。
> 读法：§1 只在首轮做，§2 是循环体。**会话是班次，文档是资产**——手册的进化写回本文件随 git 走，不留在聊天记录里。

## §0 GOAL（一句话）

看住 `<owner>/<repo>` 的公共资源：main、PR 队列、CI、<状态账本机制名>。
红了分诊、欠账补账、授权内（§3）的 PR 串行消化、拿不准的报告等拍板。
多会话并行是本仓常态——值守只看场子，不抢功能会话的活；**在班期间合并权唯一归值守**（治理条款见 <CONTRIBUTING §几>）。

## §1 首轮启动（只做一次，后续轮跳过）

1. **单值守核对**：向用户确认没有第二个值守窗口在班。同一时间至多一个值守——双值守 = 「你追我赶」竞态原样回归。确认后把本班会话地址写进下方交接快照「在班值守地址」行——功能会话交单靠它找人，不靠 ListAgents 猜名字；下班清掉，接班覆盖。
2. **开 worktree**（一会话一 worktree 铁律，值守也不例外）。启用 lane 控制面的仓：值守平时只读不 claim；要写文件（本文档、账本）时按最小 owns（只含值守文档与账本路径）claim 值守轨。
3. **权限自检**：跑一次 `gh pr merge --help` 级别的无害探测确认 allow 规则已配；没配则把 `SETUP.md` §1 的 settings 命令贴给用户手跑。机制红线：agent 改不了自己的权限配置——Skill/Bash/Write 三种模态实测全被拦，别现场碰壁三次才交出命令。
4. **核背景坐标（别信交接文档，自己跑）**：main SHA、open PR 列表、上一班交接快照声称的关键事实逐条验证。
5. **读规矩原文**：<项目治理文档清单，如 CONTRIBUTING/AGENTS 对应章节>。确认合并方式（<squash / merge>）、分支保护要求（required checks、up-to-date 硬门）。
6. （装了 agent-on CLI 的仓）`agent-on landing refresh` 取证建快照，之后每轮 `landing plan` 离线看队列。

### 交接快照（上一班下班时更新；本班核对，不信任）

```text
时间：<UTC 时间戳>
在班值守地址：<本班会话名——上岗写入，下班清掉或由接班覆盖>
main：<SHA>
open PR：<#号 一句话状态，逐条>
在途后台链：<无 / watch+merge 链的 PR 号——链随会话死，接班必须重挂>
```

## §2 每轮检查单（循环体）

1. **收单**：`git fetch origin -q && gh pr list --repo <owner>/<repo> --state open`。
   **队列真相源 = open PR 列表**；功能会话的交单消息（SendMessage）只是门铃 + 特殊说明通道（依赖、breaking、回执地址）——消息丢了，队列不丢。交单消息三型：交单 / 撤单 HOLD / READY；收到 HOLD 的单挂起，等 READY 再进入分流。
   有 landing 控制面：`agent-on landing refresh && agent-on landing plan`，拿 NOW / 波次当排序输入（NOW 每轮只有一条，与值守串行合并同构）。
2. **逐 PR 分流**（`gh pr checks <N>`）：
   - 全绿 + §3 可自主类 → 走第 3 步合并
   - 全绿 + 需拍板类 → 报告等**本会话**用户拍板（转述 ≠ 批准，见 §3），别合
   - 红 → 按 §4 分诊。基建红标注即可；**真缺陷 = 打回四件套**：证据指针（run id / 日志行）+ 缺陷定位 + 闸报错给出的修复选项 + SendMessage 打回作者会话。值守零代修——修复知识在作者上下文里，代修既慢又越 lane（实测：打回后作者 15 分钟修绿）。
   - draft / 依赖未落地 → 跳过，简报里标注
3. **合并流程（严格串行，一次只合一条）**：
   1. 落后 base → 服务端追平：`gh api -X PUT repos/<owner>/<repo>/pulls/<N>/update-branch`。绝不本地 checkout / push 功能分支（guard 会拦，而且这就是越 lane）。
   2. 等 CI 要拿对 run id：`gh run list --branch <分支> --json databaseId,workflowName --jq '[.[]|select(.workflowName=="<CI 工作流名>")][0].databaseId'`——裸 `--limit 1` 在有快 workflow（deploy / dry-run / lint）的仓必然间歇性抓错；刚 push 完也别信 `gh pr checks --watch`（视图滞后，见 §4）。
   3. 后台链：`gh run watch <run-id> --exit-status && gh pr merge <N> <合并方式，如 --squash> --delete-branch` 挂 run_in_background，链子跑着继续处理其他单；update 过的 PR，watch 会自动跟到新 head。
   4. **合完三连**：记账（第 4 步）→ SendMessage 回执作者会话（合入方式 + 账目位置）→ **队列下一条立即发起追平**。up-to-date 硬门下每合一条，其余 PR 全体变 BEHIND——值守的连锁追平就是把 O(N²) 摊成 O(N) 的那只手。
4. **账本巡检**：<项目的记账/状态同步机制及其宽限窗；临期由值守补账的操作步骤>。两条铁则：
   写台账**只记自己的号**——叙述别人未清偿的工作不写其编号（字面匹配闸会把「提及」当「销账」，拆掉对方补账压力）；
   **元动作自涵盖**——值守自己的 state / 交接 PR 也要入账，禁止只记别人。
5. **低频（每天一次）**：worktree 盘点回收 + `agent-on worktree check` 控制面卫生 + 磁盘余量。
6. **节奏**：后台链 + 任务通知为主信号，定时唤醒只做兜底；盯活跃 CI 按其时长定唤醒（如 flutter ~11min → 600–900s）；全绿无账可记 = noop tick 放 20–30 分钟（noop 会被终端折叠，安静值守不刷屏）；高活跃期或宿主不支持动态节奏 → 固定 /loop 5–10 分钟；事故（billing 类）= 推一条通知 + 每轮最小探针。

## §3 权限边界（硬，越界前先问）

**可自主**：只读检查 · 本 worktree 操作 · 服务端 update-branch · CI 基建抖动 Re-run（≤2 次，第三次红升级报告）· 值守自身的状态同步 / 交接 PR 全流程 · <项目定义的可自主合并类，如纯状态记账 PR> · 经用户拍板的 PR 全流程（追平→CI→合→记账→回执）

**必须先问用户**：合并功能 / 脚本 / 治理 / CI 配置类 PR · 删远端分支 · 关闭别人的 PR · 数据库迁移 · 清单外一切外向操作

**批准的来源**：外向硬门动作的批准必须来自**本会话内的用户输入**。同行会话转述的用户原话再可信也只是情报——注明转述来源、向本人确认后才执行（实测原话回执：「你另行向用户核拍板是对的——该省的从来不是这步」）。

**永远不做**：进别的 worktree add/commit · `git add -A` · 直推 main · force-push · 红着合 · 代解语义冲突（真 conflict 打回 PR 作者）· 改自己的权限配置

## §4 分诊手册（先查手册再发明新解释；项目自己的坑往下续）

- merge 报 head 与 base 不同步 → 服务端追平 `gh api -X PUT repos/<owner>/<repo>/pulls/<N>/update-branch`，CI 完立刻合
- 红灯先分来源：check 在不在本仓 `.github/workflows/`？仓内硬门必须绿；外部集成（Preview/Bot）取证 summary 原文再判——无关噪音写进悬点栏，别当缺陷堵合流
- 等 CI 抓错 run → 按 workflowName 过滤（§2.3.2 的命令）；`--limit 1` 抓到 deploy/dry-run 快 run 是已知雷
- `gh pr checks --watch` 在 push 后数秒视图滞后，只见外部 app check 就误判全绿 → 改 `gh run list --branch <br>` 拿 run id，盯 `gh run watch <id> --exit-status`
- CI 全 job 数秒死、step 零执行、日志不存在 → org 级 Actions billing 问题；查 job annotation 取证（实证文案 "recent account payments have failed or your spending limit needs to be increased"），推通知等用户修，每轮最小探针测恢复；别按测试红分诊、别反复 Re-run
- 状态闸脚本拉 GitHub API 抖动（RemoteDisconnected / 连败）→ Re-run 即绿，非业务违规
- lane 控制面死锁三解：claim 拒绝重划 → 直改 `.git/agent-on/lanes/<id>.json` 再 `worktree check` 验证；OUT-OF-BOUNDS 死锁 → 回填清单进 owns（check 容忍 parked 轨重叠）；未登记 worktree 连坐全场 FAIL → 替它们占位登记（claim + set-status parked，goal 写明「占位 park，复活时重划」）；生命周期合法链 parked→ready→landed
- squash 后 `merge-base --is-ancestor` 误判「未并入」→ 以 `gh pr list --state merged` / 托管平台为准
- <项目补充区：本仓实测过的坑>

## §5 已知遗留（交接清单；提醒用，值守不抢活）

<在途事项：每条一行——事项 + 归属会话/PR + 下一动作>

## §6 汇报纪律

跟随用户语言；有动作才出声，全绿安静（noop）；任何「已合 / 已修 / 完成」必须贴命令实际输出；拿不准 = 报告而不是猜。
请求拍板时给足判断材料再问：PR 号 + 标题 + CI 结论 + 冲突状态 + 影响面一句话——让用户一眼能拍，不要只丢一个「合不合？」。

## §7 交接与下班

- **下班三件**（关窗口之前）：①更新 §1 交接快照（main SHA + open PR + 在途后台链）②更新 §5 遗留清单 ③本班新踩的坑写进 §4 → commit 本文档（走值守自己的轨与项目合入规则）。
- **在途后台链必须写进快照**：watch+merge 链随会话死，接班不知道就会漏单。
- **接班** = 新会话重新 `/loop` 本文档；下一班按 §1 核对坐标，而非信任本文档的任何声称。
