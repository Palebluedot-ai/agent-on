---
title: Checker Review Rules
status: active
created: 2026-06-28
last_updated: 2026-06-28
last_change: "定义 Checker Review 的定位、输出格式、触发条件及风险补丁"
self_contained: true
---

# Checker Review Rules

## 1. 定位

Checker Review 是第二轮把关，重点关注：

- 长期影响
- 合规性
- 记忆污染风险
- 是否需要 Curator 固化

## 2. 输出格式（与 Reviewer 保持一致，增加以下字段）

```markdown
### 6. 长期影响评估
（对可维护性、记忆一致性、跨任务复用的影响）

### 7. 是否建议 Curator 固化
- Yes / No + 理由
```

## 3. 触发条件

- Reviewer 输出 `严重程度: Critical` 或 `是否阻塞: Yes`
- Reviewer 主动标记需要第二轮复核

## 4. 约束

- Checker Review 必须独立于 Reviewer，不能复用同一轮上下文
- Checker Review 结论具有更高优先级，可覆盖 Reviewer 的 `passed` 状态
- 必须更新 `review_status` 为 `passed` 或 `failed`

## 5. 风险补丁

- 防止幻觉传递：Checker Review 独立上下文
- 防止状态绕过：Checker Review 可强制推翻 Reviewer 结论
- 防止记忆污染：固化建议必须经过 Checker Review 确认