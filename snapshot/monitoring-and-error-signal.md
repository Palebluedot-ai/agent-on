---
title: Monitoring & Error Signal - Executable Rules
status: active
created: 2026-06-28
last_updated: 2026-06-28
last_change: "将 Monitoring 和 Error Signal 从建议升级为强制执行规则；定义最小强度标准、四项必备要素、强制暂停机制及禁止行为"
self_contained: true
---

# Monitoring 与 Error Signal 的可执行规则

目标：把可观测性和错误信号从「建议」升级为 Agent 执行时必须遵守的硬性规则。

## 1. Monitoring（状态显示管理）强制规则

### 1.1 必须记录的核心信号

state-manager 必须在每次状态变更时，记录以下**最小信号集**：

- 当前任务 ID + 精确范围（file:line-line）
- 变更前状态 → 变更后状态
- 触发原因（Review 失败 / 测试失败 / 人工指令 / 超时等）
- 当前瓶颈（current_bottleneck）
- 本次反馈回路的耗时

### 1.2 监控暴露方式

- 所有信号必须以结构化 JSON 格式写入 `docs/state/logs/`
- state-manager 每完成一个阶段，必须在 `progress.yaml` 中更新 `monitoring_summary` 字段
- 禁止只写自然语言日志，必须包含可被程序解析的字段

### 1.3 Monitoring 职责分配

- **state-manager**：唯一负责写入和维护监控数据的角色
- 其他 Agent 只能读取，不得直接修改监控数据
- state-manager 必须在每个反馈回路结束时，输出「本轮 Monitoring 摘要」

## 2. Error Signal 的强度强制规则

### 2.1 Error Signal 最小强度标准

Reviewer 输出的错误信号必须同时包含以下四项，否则视为无效反馈：

1. **具体问题描述**（What is wrong）
2. **影响范围**（Where it affects）
3. **修正方向**（How to fix）
4. **严重程度分级**（Critical / High / Medium / Low）

### 2.2 Error Signal 分级定义

- **Critical**：导致无法继续执行或产生幻觉风险
- **High**：严重偏离验收标准
- **Medium**：可改进但不阻塞
- **Low**：风格/命名等非功能性问题

### 2.3 强制执行机制

- Reviewer 必须在反馈中明确标注分级
- state-manager 收到 Critical 或 High 级 Error Signal 时，必须：
  - 暂停当前任务
  - 要求 Implementer 在下一次输出前先回应该 Error Signal
  - 在 log 中记录「Error Signal 未被有效处理」的次数
- 连续两次未有效处理 Critical Error Signal，state-manager 必须上报并要求重新拆分任务

### 2.4 禁止行为

- 禁止 Reviewer 只输出「这里不对」「需要改一下」这类无强度信号
- 禁止 Implementer 忽略 Error Signal 直接进入下一轮

## 3. 协同机制

- Monitoring 负责「看清楚发生了什么」
- Error Signal 负责「把问题转化为可行动的输入」
- 两者结合，形成闭环的「感知 → 判断 → 行动」控制回路

## 相关文件

- `self-contained-file-system.md`
- `three-stabilizers.md`
- `docs/state/progress.yaml`
- `agents/state-manager.md`