# 新场景问卷 — 复杂/高风险项目深挖版

> 职责边界:本问卷是 BOOTSTRAP.md §1「六问快问快答」的**深挖版**。六问用来给普通新项目快速定盘;当项目复杂(多轨编排)或高风险(碰钱/真实用户数据/对外服务)时,填完六问后再走这份六步问卷,把编排草案与边界一次性钉死。产物是 task_card 的输入,不是又一份闲聊。
> 源流:一代 agent-orchestration-playbook/05_playbooks/new-scenario-onboarding-template-v1.md,2026-07-07 批二移植去平台化。

## 何时用

- 六问答完发现是 Ship 车道(碰钱/数据/对外)→ 用。
- 单 agent 一把梭能干完 → **不用**,直接进 BOOTSTRAP §2 搭骨架。
- 判据:「要不要多轨并行」拿不准时,填到 Step 2 就有答案。

## Step 0:场景摘要
- scenario_name:
- business_owner:(谁拍板、谁验收)
- external_risk: low | medium | high（碰钱/真实用户数据/对外服务即 high）
- output_type: internal-note | customer-facing | public-post

## Step 1:5 维标签
- task_kind:（编码 / 设计 / 调研 / 文档 / 混合）
- uncertainty:（需求多确定;越不确定越该先 Explore 车道）
- risk_level: low | medium | high
- coupling:（是否共享中间状态;shared-write 是并行的头号敌人)
- (可选) volatility:（需求多久变一次,决定契约冻结粒度）

## Step 2:泛化决策（硬规则,别绕）
1. 默认新增 **capability**(给现有 agent 加本事),**不新增永久 role**。
2. 默认 single-agent-first —— 单个子代理能干完就别拆多轨。
3. 可干净切块且不共享中间状态 → 才升级为多子代理并行(轨道 = 目录 + git worktree 物理隔离)。
4. 升多 agent/团队协作需**硬门槛**,满足以下至少 2 条才升:
   - 持续共享中间状态,单轨绕不开;
   - 多角色互相影响决策;
   - single-agent 连续不达标(**有证据**,不是感觉)。

## Step 3:编排草案
- executor: claude_code | codex | human
- mode: single-agent | orchestrator-worker(主会话派子代理) | multi-track(worktree 并行)
- required_roles:（worker/reviewer/curator,按需;不轻易新增)
- review_gate: standard | strict（high-risk 强制 strict）
- hitl_required: yes | no（对外发布动作强制 yes）

## Step 4:上下文边界
- track_read:（本轨能读什么)
- shared_read:（能读哪些共享状态:progress.yaml / main 契约)
- forbidden_context:（明令不许碰的,写成禁令)

## Step 5:验收标准（最少 3 条,每条能翻成测试名或命令输出）
1.
2.
3.

## Step 6:试运行与审计
- pilot_task_count: 3~5
- collect_metrics: 首过率 / strict review 覆盖率 / shared-write 冲突数 / 越界事件数
- promotion_rule: 指标达标后再把做法固化成 memory_card / playbook

## 输出产物
- task_card（每张子任务一张）
- result_card（交活时）
- （可选)memory_card（仅 reviewer=pass 且过写入门禁）
