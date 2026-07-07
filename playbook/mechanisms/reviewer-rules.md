---
title: Reviewer Rules - Executable Constraints
status: active
created: 2026-06-28
last_updated: 2026-06-28
last_change: "定义 Reviewer 的输出格式、检查清单和禁止事项，作为 Guardrail 模式下的强约束规则"
self_contained: true
---

# Reviewer Rules

## 1. 输出格式（强制）

Reviewer 必须严格按照以下结构输出，缺一不可：

```markdown
## Reviewer Report

### 1. 问题描述
（必须具体到文件、函数或逻辑点）

### 2. 影响范围
（明确影响哪些模块、场景、数据流）

### 3. 修正方向
（给出具体修改建议或重构方向）

### 4. 严重程度
- Critical / High / Medium / Low

### 5. 是否阻塞
- Yes / No
```

## 2. 检查清单（必须逐项对照）

Reviewer 必须对照以下清单进行审查：

- 业务逻辑正确性
- 错误处理完整性
- 数据一致性与边界条件
- 安全与权限检查
- 性能与可维护性
- 与现有代码风格一致性

## 3. 禁止事项（硬约束）

Reviewer 绝对禁止以下行为：

- 禁止直接修改代码
- 禁止跳过 Critical 问题
- 禁止给出模糊反馈（例如“这里不对”、“感觉有问题”）
- 禁止进行任务规划或记忆固化
- 禁止越过 Checker Review 直接通过

## 4. 风险控制

- 输出必须结构化，否则视为无效审查
- 禁止事项违反即触发 Checker Review 介入
- 所有审查结果必须更新 `progress.yaml` 中的 `review_status`