# AGENTS.md — [项目名] 行为宪法(骨架)

> Agent 在本仓的最高规则。与人类沟通语言跟随用户;本文件条款冲突时,越靠前越优先。

## §0 agent-on 映射

方法论来自 agent-on(版本与偏离登记见项目根 `agent-on.lock.md`,只映射不复制)。口令:「agent-on 结账」(沉淀回流)/「agent-on 升级」(bump pin)。

## §0.5 项目一句话(握手第一步的读取位)

- **项目一句话**:[做什么,给谁用] <!-- 必填。session-handshake 第一步「复述项目总目标」读的就是这一行;空着 = 握手第一步无从可读 -->
- **北极星指针**:[指向哪份文档 / 哪个指标算「做成了」] <!-- 与上一行同批填;runtime 约束与产品终局分栏写,见 §1 runtime ≠ product surface -->

> 源流 2026-09-06 CryptoQuant 首次结账:M 档骨架实例化后,握手第一步「从 AGENTS.md 读总目标」**无处可读**,项目只能自加 §0.5。槽位做进模板,各项目不必自创节——**握手执行书要求读的东西,骨架必须提供一个确定位置**。

## §1 硬约束(违反=事故)

| 约束 | 内容 |
|---|---|
| [架构红线] | 例:Thin Client 只走网关,不直连数据库,不在客户端写业务规则 |
| [数据红线] | 例:金额一律 string+NUMERIC;生产库 [旧系统名] 绝不写 |
| [安全红线] | 密钥只进本地 gitignored .env 与部署平台 env;签名 URL/token 禁入日志;service 级凭证只许在 [封装模块路径] 出现 |
| **不写死暂停项** | [用户说「以后再聊」的清单,逐条列]=**未获明确指令前不实现、不假设**(删掉=留缺口给幻觉,禁令=钉死);MVP 后置的**渠道/触点**(推送/移动端/多租户)必须入此表——只活在对话「以后做」= 实现会话当 soft backlog 偷做。**局部解禁**允许:用 requirements **D 表**写清「已拍什么 / 仍禁什么」,同批同步 AGENTS 暂停表述、dashboard、TODOS、威胁模型相关句——**禁止**聊天默示全解、禁止只改业务 docs 不改暂停表述(Euan D18 2026-07-19) |
| **不发明花名册** | 邮件/IM 里出现过的邮箱 ≠ 可写组织目录。只在人类确认后登记身份,再 regenerate 派生映射。禁止为了「分到人」而 invent roster 行 |
| 外向操作 | **两类分开,别一锅端**。①**本轨内部动作 = 自己做,不问**:提交 · 推**自己的**分支(`git push origin <本轨分支>`)· 开 PR / draft PR · 跑测试——本地独有提交久留不推**才是**事故(机器一坏全丢),为它讨点头是噪音不是护栏。②**外向硬门 = 须用户点头**:merge · 打 tag / push tag · 发 release · 直推受保护分支 · force-push · 删远端分支 · 关别人的 PR;PR·Issue 评论与一切代表本项目对外发言(邮件/IM/webhook);部署 · 建远程资源 · 改共享云配置 · 跑数据库迁移;花钱。**授权幂等**:同一项目内同类动作**一次点头长期有效**,不逐次追问;只有跨到没点过头的**新类别**才重新问。**假定一切 CLI 在非交互环境自动确认**(--dry-run 不存在就先在无害目标试行为) |
| 高风险域 preflight(可选) | 碰钱/真实用户数据/批处理毁库时:本仓 SessionStart 写会话回执 + 高风险 Bash fail-closed(无回执不 push/不批跑);模式见 kit/guard/README「L-进场·会话回执」。IDE hook 非生产护栏(生产见 anti-hallucination dev floor vs prod API)。**真钱 / 签名命令固定由人执行**:AI 写代码与命令 → 人执行并贴回原始终端输出 → AI 逐条对照验收标准转录进 `docs/evidence/` 并关卡;phase 卡的 live 条目在原始输出回来前一律标 ⏸,**禁止以 dry-run 或推断冒充**(aster-agent S0.2/S0.3 三里程碑零伪造零事故) |
| **runtime ≠ product surface** | 生产线/采集的运行时约束(本机常驻、礼貌限速、私网)与**产品交付终局**(用户装哪里、云上是否可用)必须分栏写进 requirements/本表——禁止把「crawl 只能本机」合并成「产品只能本机安装」(hk-sfc-licensees D19/D20) |

## §2 纪律四件套

1. **风险相称的验证**:行为/数据/权限/回归缺陷优先用失败测试；样式、文案和文档用相关预览与检查，不机械要求 TDD。
2. **Error Signal 四要素**:异常上报必含 What/Where/How(复现)/Severity;禁止静默绕过。
3. **验证后才说完成**:任何「完成」声明必须附验证命令的实际输出;外部依赖缺位=标 ⏸ 挂账+写清事后步骤,**严禁伪造证据**。
4. **共享状态一个写者**:只对已采用的共享状态源指定入口写者；任务作者维护自己的记录。子助手不独立发布，功能会话可推自己的分支并交 PR，不强制新建 progress.yaml。

**提交纪律(半句)**:声明原子提交前 `git status --short` 读**全暂存区**——`git add <路径>` 不限定提交范围,残留会被吞进 commit。

**上下文续接**：同一任务默认在原窗口使用宿主原生 compact。自然收口或准备主动压缩时更新已有任务记录中的目标、用户决定、未完项和证据位置；压缩后按需重读原文件与实际 diff，直接续跑。反复偏离且校正无效才考虑新会话，不按压缩次数强制开窗，不为每次压缩新建交接文档。完整约定见 agent-on `boot/session-handshake.md`。

**浏览器入口不得 re-export Node**:给 CSR/浏览器的公共 barrel 禁止再导出引用 `node:fs` / `path` / `child_process` 的模块。读数与写盘分成两个入口;禁止用「反正 tree-shake」赌打包器。

**不可逆动作前验证作用域**:config push / deploy / publish 前,用只读命令或 diff 方向确认工具读的是**你以为的那份文件**(cwd 常压过 flag;多 worktree 尤其致命)。

**排障纪律(半句)**:列「让对方逐项试」清单前先问——能不能直接看到(截图/只读 API)?能看就先观测,别用试错代替。

**机制须带闸**:写进本文件的协作/状态规则,自问「本条靠什么闸?」(CI/脚本/无则明写靠自觉)。空转两周+的纸面机制机械化或删除,不许装样子(见 multi-contributor-protocol §三½)。

## §9 动态需求协议(用户中途提新想法时)

① 复述确认边界 → ② 定位置(本切片/新切片/暂停项)→ ③ 更新对应文档(requirements D 表 / TODOS / qa 三桶)→ ④ 继续当前工作,不被打断主线。
**想法类捷径**:若只是产品想法/待办(非本切片需求、非 debug、非状态询问),AI 当场代笔一行进 `thoughts-and-ideas.md` 📥速记区(带日期+「对话捕获」标),口头确认一句即继续——只进速记区不进已整理,升级成需求永远由用户拍板;拿不准就不记,宁漏勿噪。
**暂停项局部解禁**:D 表划界(已拍/仍禁)+ 同批多面同步(宪法摘要·仪表盘·TODOS·威胁模型);口头「可以做一点加密」≠ 整栈 E2E/KMS 解禁。
**决策/切片取号即落盘**:占用 D-N / phase 号等共享顺序编号时,先把占位行以最小 diff 写入共享真相面(requirements/进度)再写正文——并行会话会撞号(见 multi-contributor 三种并行事故之三)。
**事后追认(代码先行止损通道,不是特权)**:需求变更协议拦不住「先写后认」时,不装看不见——**merge 后 48h 内**补 D 表/台账;追认检查单**必含**隐私/法务/配额/披露等外围义务(先行最容易漏的恰是这些)。同一轨道两次先行 → 收紧该轨 PR 审查。(Euan 2026-07-30:requirements 明文「语音 v1 不做」而 PR #18 上线;追认 D23 连带抓出隐私零披露与上线前置债)

## §10 编排并行协议(orchestrated-parallel)

1. **契约先冻结**:`contracts/fixtures/*.json` 只许主会话改;冻结时把**语义**(排序/空值/上限/口径)一起写死。
2. **轨道按共享改动划分**:先把共享组件、token、路由和锁文件归一个任务；不同页面不自动开新轨。确实并行写入时各用独立 worktree，claim/owns 只是可选记账，不是开工前提。可选巡逻自动记录意图与实际 diff；同文件闸在实际暂存区/推送历史范围内检查未提交重叠；PreToolUse 保留路由与跨仓授权。
3. **互相 Fake**:每轨用 fixture 种子造对方的假实现,自身闭环可测。
4. **契约测试当裁判**:双端各自直接 import 同一份 fixture 断言。
5. **报告即数据**:输出按复杂度使用 agent-on `kit/output-contract.md`；普通回复直接给结果与必要证据，多任务交付才用完整结构(状态面板在前 → 拍板收成一节带默认值 → 结论三格 → 撤销两栏 → 球在谁那 → 之后才是过程)。轨道最终回复在该契约内必填:逐条验收 ✅/❌/⏸ + 测试输出末行 + 文件清单 + **「我按这个假设做了,你不否就当成立」**(把假设显式交出来,每条写清否掉要重做什么)+ commit hash;不 push。类别一律中文人话,机器类别名只准放括号里。
6. **合流顺序**:需要契约时先冻结，再集成实现；未验证假设集中裁决。涉及 Fake→真时验证真实接线；按受影响行为选择回归与上机验证，复用任务结果记录，不给每页重复跑整套 CI。

**衍生功能先归并**:执行中长出独立目标先记录，由入口判断与现有任务合并还是另开隔离窗口，用 `--depends-on` 显式排顺序;当前不做 → 想法箱/暂停项。`agent-on worktree status` 是本机全场视图;回收只按 `safe|review|rescue` 分类人工执行,禁止自动删孤本。模式见 agent-on `kit/worktree-control-plane.md`。

**开轨与回收硬句**：优先用宿主原生 worktree 工具；手工路径沿用本项目声明的 root，未声明可用 `.worktrees/<lane-id>`（Claude 原生路径 `.claude/worktrees/<lane-id>` 同样合法）。分支名 `<type>/<issue-or-lane>-<slug>`，新写任务从入口确定的集成基线创建；一批任务可共享一次已核对的 base，不为每页重复 fetch。握手、每日一次、每次合流后盘点；每日命令为 `agent-on worktree gc --dry-run --json`，也可选择 `agent-on worktree hooks install --daily-gc` 安装同一 report-only 盘点，其 `candidates` 是动态 known reclaim list，不另写静态清单，原有 gc 永不删除。单独启用 janitor 才允许有限回收受管、已释放且可恢复的 checkout；不删除分支。

**回收权限**：原有 GC 只报告。用户启用 janitor 后，有限策略可回收本功能创建的 checkout（执行书 `kit/patrol-control-plane.md`）；其余删除、本地或远端分支、`--force`、跨 worktree add/commit 仍须目标明确授权。locked、dirty、unknown 永不删；squash 场景以 PR 状态为权威，不能只信 `merge-base --is-ancestor` 的否定结果。

## §QA 三桶(跑通阶段只记账不停下)

A 未建功能(切片卡管)/ B 疑似缺陷(统一修)/ C 视觉体验(统一 design-review)。

## §skill 路由(制度在 agent-on;环节 skill 可选)

> **默认立场(2026-08 起)**：agent-on 管制度(证据/边界/单写者/结账回流);**不**把 Superpowers 当默认执行栈(偏重、易抢跑)。有 GStack 则环节点名走 GStack;无强 skill → kit 模板 fallback。BOOTSTRAP §1 第 5 问只采集「机器上还装了啥」,不暗示必须双栈。

| 环节 | 本项目默认 | 无则 fallback |
|---|---|---|
| 规划设计 | [GStack /autoplan · 若已装] | 主会话 + 用户拍板;禁 brainstorming 抢规划 |
| 实现执行 | **主会话 / 按需子代理** + 风险相称的验证与完成证据 | 实际接口并行时选 agent-on 协作协议；默认沿已授权目标推进 |
| 代码/PR 审查 | [GStack /review · 若已装] | kit/review-prompt-template.md(只保留一套审查) |
| 合流验收 | [GStack /qa · 若已装] | kit/merge-checklist.md |
| 发布部署 | [GStack /ship · 若已装] | 项目自有 checklist;agent-on pin/结账照常 |
| 调试 | [GStack /investigate · 若已装] | playbook/anti-hallucination + 完成贴证据 |

**压制条款（防抢跑·必填）**：本文件是双工具共读层——在此点名禁用才同时管住 Claude 与 Codex。默认至少写：
- 禁用 `superpowers:brainstorming` / `superpowers:writing-plans` 抢跑 init 与规划
- 实现**不**默认 `superpowers:subagent-driven-development`(用户点名才用)
- 审查/发布/调试**不**并行再开一套 Superpowers 同名流程(避免双制度)
只写进 `~/.claude/CLAUDE.md` 不够(AINVESTMENT: Codex 侧仍被抢跑)。

## §二车道(见 agent-on 的 playbook/freedom-vs-discipline.md)

Explore(视觉/原型/概念:一把梭可丢弃,不写测试,只守 token 色/真实感数据/触达底线)× Ship(碰数据/钱/安全:对应风险的验证与授权)。Explore 转 Ship 前补齐对应风险的验证；合格部分可复用，不强制重写。
