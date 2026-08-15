# AGENTS.md — [项目名] 行为宪法(骨架)

> Agent 在本仓的最高规则。与人类沟通语言跟随用户;本文件条款冲突时,越靠前越优先。

## §0 agent-on 映射

方法论来自 agent-on(版本与偏离登记见项目根 `agent-on.lock.md`,只映射不复制)。口令:「agent-on 结账」(沉淀回流)/「agent-on 升级」(bump pin)。

## §1 硬约束(违反=事故)

| 约束 | 内容 |
|---|---|
| [架构红线] | 例:Thin Client 只走网关,不直连数据库,不在客户端写业务规则 |
| [数据红线] | 例:金额一律 string+NUMERIC;生产库 [旧系统名] 绝不写 |
| [安全红线] | 密钥只进本地 gitignored .env 与部署平台 env;签名 URL/token 禁入日志;service 级凭证只许在 [封装模块路径] 出现 |
| **不写死暂停项** | [用户说「以后再聊」的清单,逐条列]=**未获明确指令前不实现、不假设**(删掉=留缺口给幻觉,禁令=钉死);MVP 后置的**渠道/触点**(推送/移动端/多租户)必须入此表——只活在对话「以后做」= 实现会话当 soft backlog 偷做。**局部解禁**允许:用 requirements **D 表**写清「已拍什么 / 仍禁什么」,同批同步 AGENTS 暂停表述、dashboard、TODOS、威胁模型相关句——**禁止**聊天默示全解、禁止只改业务 docs 不改暂停表述(Euan D18 2026-07-19) |
| **不发明花名册** | 邮件/IM 里出现过的邮箱 ≠ 可写组织目录。只在人类确认后登记身份,再 regenerate 派生映射。禁止为了「分到人」而 invent roster 行 |
| 外向操作 | push/部署/建远程资源/改共享云配置,首次须用户确认;**假定一切 CLI 在非交互环境自动确认**(--dry-run 不存在就先在无害目标试行为) |
| 高风险域 preflight(可选) | 碰钱/真实用户数据/批处理毁库时:本仓 SessionStart 写会话回执 + 高风险 Bash fail-closed(无回执不 push/不批跑);模式见 kit/guard/README「L-进场·会话回执」。IDE hook 非生产护栏(生产见 anti-hallucination dev floor vs prod API) |
| **runtime ≠ product surface** | 生产线/采集的运行时约束(本机常驻、礼貌限速、私网)与**产品交付终局**(用户装哪里、云上是否可用)必须分栏写进 requirements/本表——禁止把「crawl 只能本机」合并成「产品只能本机安装」(hk-sfc-licensees D19/D20) |

## §2 纪律四件套

1. **TDD**:没有失败测试不许写生产代码。
2. **Error Signal 四要素**:异常上报必含 What/Where/How(复现)/Severity;禁止静默绕过。
3. **验证后才说完成**:任何「完成」声明必须附验证命令的实际输出;外部依赖缺位=标 ⏸ 挂账+写清事后步骤,**严禁伪造证据**。
4. **单一状态写者**:`docs/state/progress.yaml` 只有 orchestrator(主会话)写;轨道 agent 不写状态、不 push。

**提交纪律(半句)**:声明原子提交前 `git status --short` 读**全暂存区**——`git add <路径>` 不限定提交范围,残留会被吞进 commit。

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
2. **轨道=目录+git worktree+合同**:每个写会话独占一个 worktree;开工前用 `agent-on worktree claim` 登记单一目标/互斥 `owns`/依赖/base。每轨只许改自己目录,双向禁入,禁碰 contracts/ docs/;提交与合流前 `agent-on worktree check` 非零即停。
3. **互相 Fake**:每轨用 fixture 种子造对方的假实现,自身闭环可测。
4. **契约测试当裁判**:双端各自直接 import 同一份 fixture 断言。
5. **报告即数据**:轨道最终回复=逐条验收 ✅/❌/⏸ + 测试输出末行 + 文件清单 + **契约悬点**(把假设显式交出来)+ commit hash;不 push。
6. **合流顺序**:先契约后实现;悬点集中裁决;翻转 Fake→真;全量回归;上机;记 run-ledger。

**衍生功能不扩轨**:执行中长出可独立目标 → 新 phase + 新 worktree/lane,用 `--depends-on` 显式排顺序;当前不做 → 想法箱/暂停项。`agent-on worktree status` 是本机全场视图;回收只按 `safe|review|rescue` 分类人工执行,禁止自动删孤本。模式见 agent-on `kit/worktree-control-plane.md`。

## §QA 三桶(跑通阶段只记账不停下)

A 未建功能(切片卡管)/ B 疑似缺陷(统一修)/ C 视觉体验(统一 design-review)。

## §skill 路由(制度在 agent-on;环节 skill 可选)

> **默认立场(2026-08 起)**：agent-on 管制度(证据/边界/单写者/结账回流);**不**把 Superpowers 当默认执行栈(偏重、易抢跑)。有 GStack 则环节点名走 GStack;无强 skill → kit 模板 fallback。BOOTSTRAP §1 第 5 问只采集「机器上还装了啥」,不暗示必须双栈。

| 环节 | 本项目默认 | 无则 fallback |
|---|---|---|
| 规划设计 | [GStack /autoplan · 若已装] | 主会话 + 用户拍板;禁 brainstorming 抢规划 |
| 实现执行 | **主会话 / 按需子代理** + agent-on 铁律(TDD·完成贴证据) | agent-on 六步协议(并行时);**不**默认 Superpowers subagent-driven-development |
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

Explore(视觉/原型/概念:一把梭可丢弃,不写测试,只守 token 色/真实感数据/触达底线)× Ship(碰数据/钱/安全:全纪律)。**两道不许串**:Explore 代码不直接 merge(重写),Ship 流程不管 Explore。
