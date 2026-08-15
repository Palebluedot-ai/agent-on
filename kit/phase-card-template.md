---
phase: "[S编号.轨号]"
self_contained: true          # 承诺:执行者不需要会话记忆,断连可原地续跑
required_context:
  - domain: "[契约=哪些 fixtures(冻结);上游文档=哪份的哪节]"
pointer_format: "file:line-line"
max_feedback_loop_min: 30     # 30 分钟内必须能跑一次验证,做不到就拆卡
setpoint: "[一句话:做到什么算到位——可机械判定]"
disturbance: "[禁区,与 setpoint 同权重:不许碰哪些目录/不许发明什么规则/哪些暂停项在此卡附近]"
---

# Phase [编号] — [名字]

## 0. 前置检查（按卡类型勾；不适用则整节删）

<!-- Demo-as-spec / 设计管线：untracked 的规格 = 会蒸发的锚点，禁止对着仓外文件开工
     源流:Euan design/demo-v9 8374abc; digest demo-anchor-into-repo-before-build -->
- [ ] **Demo 锚点四件套已入仓**（若本卡以 demo/交互稿为规格）:① demo 本体 ② 全部运行依赖 ③ 逐屏/关键基线截图 ④「微调后怎么同步」的更新流程 README——四件齐才算锚点；缺一不开工

## 1. 验收标准

<!-- 铁律:每条 ≈ 一个测试名;机器可验证或人眼一步可验;≤10 条(Opus 执行 ≤8 条);
     外部依赖缺位的条目直接写 ⏸ 挂账+事后验证步骤 -->
- [ ] [功能条目:端点/组件 + 形状=哪个 fixture + 边界条件]  <!-- 旁注:本项验证的真实依赖是什么?硬件/密钥/真人控制台?可替代环境能拿到同等证据吗 -->
- [ ] [安全条目:攻击者测试(B token 打 A 资源→404)/防枚举(响应逐字节相同)——写成断言不是「注意安全」]
- [ ] [幻觉高发区条目:时区/金额/幂等 → 纯函数 + 边界单测(列出具体边界值)]
- [ ] [体验条目(UI 卡):四态齐/44pt/token 化(硬编码色=0)/1.3× 不破版]
- [ ] [回归条目:全量测试命令 + 零删测声明]
- [ ] [接线锚·若本卡新建共享组件/主题/常量]:pump 真实消费方断言生效 + 变异一次(拆接线必须红)——形状测试全绿 ≠ 已接线(anti-hallucination C 附)
- [ ] [护栏实体·若本卡改 CODEOWNERS/审批人/告警/webhook]:实体存在性验证证据(API 或真实触发)+ 文档写清生效边界
- [ ] [运行面·若本卡接定时/Webhook/开放平台]:配置勾选 / agent 已 load / 日历开火都不算完成;须 runs+exit+产物,或 tail 见入站事件。一应用一事件 URL(anti-hallucination C 附3)
- [ ] [配置锚·若本卡改 TOML/YAML/JSON 等结构化配置]:语义层断言(真解析器或位置/结构判别)+ **反向验证**(改坏必须红)——纯文本「字符串在文件里」假绿(anti-hallucination C 附2)
- [ ] [收口·计划状态]:本卡相关 TODO/阶段表当前态已随验收证据更新,指向 progress 权威源——禁止幽灵 P0
- [ ] [生产 preflight·若碰 load/deploy]:只读机器可判 pass/block 列表;skip/unsupported 保持整体红;门禁接在唯一执行入口(非仅 runbook)
- [ ] [capability scope·窄发布时]:本 scope 必绿门 / 明确不适用门 / 禁用能力 三列写清

## 2. 内联要点

<!-- 压幻觉的主力区:参考模式指针 > 文字描述 -->
- 参考模式:照 [现有文件路径] 的 [哪个手法] 写(指针,不是转述)
- 环境坑:[worktree 里要先装依赖/env 拷贝法/已知平台脾气]
- 集成探针(接新外部服务时必含):先 dump 真实载荷与代码假设对账,再写映射
- 扫坑指针(接外部服务/上并行/交付前):对照 agent-on `bench/cases/README.md` 使用时机表,同类坑动手前先认
- **文件域**(并行 implementer 时必填):本卡只许改 `[路径前缀…]`;禁碰 `[…]`
- **共享文件 owner**(契约表/索引/清单/barrel 等跨切片面):`[路径]` → owner=`[本卡|收口人|某轨名]`——禁止「各补一行」却不点名;漏派 = 条款集体落空(见 playbook/workflow-orchestration §二½)

## 2b. 可选·长任务 Loop 台账（跨会话 / 一天多 Loop 时启用）

<!-- 源流:IPONews phase-p-rag 实证 2026-07-16;digest long-task-loop-ledger-on-phase
     hk-sfc-licensees 2026-08-01: cadence≠deadline; stop 对齐用户「做完」范围 -->
跨会话长主线（一天多 Loop、多日同一 phase）时，在本卡开篇或专节建 **append-only Loop 台账**，勿只靠会话记忆或 loop-notes 散文：

| 日期 | Loop# | 目标（一句话） | 验收命令 | 结果（贴输出或 commit） |
|---|---|---|---|---|
| YYYY-MM-DD | 0 | … | `…` | … |

- 新会话：读 `progress.yaml` 的 `next` / `current_phase` + **台账最后一行**断点续跑
- 单 Loop ≤ `max_feedback_loop_min` 可验证时间盒；做不到就拆 Loop 或拆卡
- **cadence ≠ 截止**: recurring 开火间隔（如 30m）是调度间距，**不是**「30 分钟内做完全部代码」的工时预算——对用户说明时必须拆开讲
- **stop 对齐产品范围**: 自动 loop 的停止条件必须覆盖用户心中的「做完」；若 stop 只覆盖文档/局部 UI，回报须写明「云上线/数据轨未完成」，**默认不自删 scheduler**，或拆成命名清晰的多 loop
- 业务实现细节不回流 agent-on；可复用的是「台账编排」本身
