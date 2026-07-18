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

## 1. 验收标准

<!-- 铁律:每条 ≈ 一个测试名;机器可验证或人眼一步可验;≤10 条(Opus 执行 ≤8 条);
     外部依赖缺位的条目直接写 ⏸ 挂账+事后验证步骤 -->
- [ ] [功能条目:端点/组件 + 形状=哪个 fixture + 边界条件]
- [ ] [安全条目:攻击者测试(B token 打 A 资源→404)/防枚举(响应逐字节相同)——写成断言不是「注意安全」]
- [ ] [幻觉高发区条目:时区/金额/幂等 → 纯函数 + 边界单测(列出具体边界值)]
- [ ] [体验条目(UI 卡):四态齐/44pt/token 化(硬编码色=0)/1.3× 不破版]
- [ ] [回归条目:全量测试命令 + 零删测声明]

## 2. 内联要点

<!-- 压幻觉的主力区:参考模式指针 > 文字描述 -->
- 参考模式:照 [现有文件路径] 的 [哪个手法] 写(指针,不是转述)
- 环境坑:[worktree 里要先装依赖/env 拷贝法/已知平台脾气]
- 集成探针(接新外部服务时必含):先 dump 真实载荷与代码假设对账,再写映射
- 扫坑指针(接外部服务/上并行/交付前):对照 agent-on `bench/cases/README.md` 使用时机表,同类坑动手前先认

## 2b. 可选·长任务 Loop 台账（跨会话 / 一天多 Loop 时启用）

<!-- 源流:IPONews phase-p-rag 实证 2026-07-16;digest long-task-loop-ledger-on-phase -->
跨会话长主线（一天多 Loop、多日同一 phase）时，在本卡开篇或专节建 **append-only Loop 台账**，勿只靠会话记忆或 loop-notes 散文：

| 日期 | Loop# | 目标（一句话） | 验收命令 | 结果（贴输出或 commit） |
|---|---|---|---|---|
| YYYY-MM-DD | 0 | … | `…` | … |

- 新会话：读 `progress.yaml` 的 `next` / `current_phase` + **台账最后一行**断点续跑
- 单 Loop ≤ `max_feedback_loop_min` 可验证时间盒；做不到就拆 Loop 或拆卡
- 业务实现细节不回流 agent-on；可复用的是「台账编排」本身
