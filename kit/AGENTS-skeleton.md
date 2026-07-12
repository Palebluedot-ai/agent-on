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
| **不写死暂停项** | [用户说「以后再聊」的清单,逐条列]=**未获明确指令前不实现、不假设**(删掉=留缺口给幻觉,禁令=钉死) |
| 外向操作 | push/部署/建远程资源/改共享云配置,首次须用户确认;**假定一切 CLI 在非交互环境自动确认**(--dry-run 不存在就先在无害目标试行为) |

## §2 纪律四件套

1. **TDD**:没有失败测试不许写生产代码。
2. **Error Signal 四要素**:异常上报必含 What/Where/How(复现)/Severity;禁止静默绕过。
3. **验证后才说完成**:任何「完成」声明必须附验证命令的实际输出;外部依赖缺位=标 ⏸ 挂账+写清事后步骤,**严禁伪造证据**。
4. **单一状态写者**:`docs/state/progress.yaml` 只有 orchestrator(主会话)写;轨道 agent 不写状态、不 push。

## §9 动态需求协议(用户中途提新想法时)

① 复述确认边界 → ② 定位置(本切片/新切片/暂停项)→ ③ 更新对应文档(requirements D 表 / TODOS / qa 三桶)→ ④ 继续当前工作,不被打断主线。
**想法类捷径**:若只是产品想法/待办(非本切片需求、非 debug、非状态询问),AI 当场代笔一行进 `thoughts-and-ideas.md` 📥速记区(带日期+「对话捕获」标),口头确认一句即继续——只进速记区不进已整理,升级成需求永远由用户拍板;拿不准就不记,宁漏勿噪。

## §10 编排并行协议(orchestrated-parallel)

1. **契约先冻结**:`contracts/fixtures/*.json` 只许主会话改;冻结时把**语义**(排序/空值/上限/口径)一起写死。
2. **轨道=目录+git worktree**:每轨只许改自己目录,双向禁入,禁碰 contracts/ docs/。
3. **互相 Fake**:每轨用 fixture 种子造对方的假实现,自身闭环可测。
4. **契约测试当裁判**:双端各自直接 import 同一份 fixture 断言。
5. **报告即数据**:轨道最终回复=逐条验收 ✅/❌/⏸ + 测试输出末行 + 文件清单 + **契约悬点**(把假设显式交出来)+ commit hash;不 push。
6. **合流顺序**:先契约后实现;悬点集中裁决;翻转 Fake→真;全量回归;上机;记 run-ledger。

## §QA 三桶(跑通阶段只记账不停下)

A 未建功能(切片卡管)/ B 疑似缺陷(统一修)/ C 视觉体验(统一 design-review)。

## §skill 路由(本机已装哪套强 skill,各环节走谁)

> 填这张表定「审查/发布/调试」各走哪套工具;agent-on 的 kit 模板(review-prompt / merge-checklist 等)是**没有对应 skill 时的 fallback**,避免与本机 skill 撞出双制度。BOOTSTRAP §1 第 5 问采集本机 skill 体系。

| 环节 | 本项目用哪套 | 无则 fallback |
|---|---|---|
| 规划设计 | [如 GStack /autoplan] | — |
| 实现执行 | [如 Superpowers subagent-driven-development] | agent-on 六步协议 |
| 代码/PR 审查 | [如 GStack /review + Codex] | kit/review-prompt-template.md |
| 合流验收 | [如 GStack /qa] | kit/merge-checklist.md |
| 发布部署 | [如 GStack /ship] | — |
| 调试 | [如 GStack /investigate] | playbook/anti-hallucination.md |

**压制条款（防抢跑）**：被路由掉的 skill 在此点名禁用（例：「规划设计走 /autoplan，禁用 superpowers:brainstorming 抢跑」）——本文件是双工具共读层，压制写在这里才对 Claude 与 Codex 同时生效；只写进单工具配置（如 `~/.claude/CLAUDE.md`）另一家照样抢跑。

## §二车道(见 agent-on 的 playbook/freedom-vs-discipline.md)

Explore(视觉/原型/概念:一把梭可丢弃,不写测试,只守 token 色/真实感数据/触达底线)× Ship(碰数据/钱/安全:全纪律)。**两道不许串**:Explore 代码不直接 merge(重写),Ship 流程不管 Explore。
