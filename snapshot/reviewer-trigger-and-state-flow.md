---
title: Reviewer Trigger and State Flow
status: active
created: 2026-06-28
last_updated: 2026-06-28
last_change: "定义 Reviewer 的触发条件、状态流转机制及风险补丁"
self_contained: true
---

# Reviewer Trigger and State Flow

## 1. 触发条件（必须满足其一）

- `progress.yaml` 中 `review_status` 变为 `pending`
- 当前 phase 文件末尾出现 `## Reviewer Activation` 区块
- Worker 明确输出“代码交付完成，请求审查”

## 2. 状态流转（progress.yaml）

```yaml
review_status:
  pending        # 等待 Reviewer 介入
  in_review      # Reviewer 正在审查
  passed         # 通过，可进入下一阶段
  failed         # 未通过，需返回 Worker 修正
  checker_review # 进入 Checker Review 阶段
```

## 3. 状态流转约束

- Reviewer 必须在输出审查结果后更新 `review_status`
- 禁止在 `review_status` 未更新的情况下进入下一阶段
- 增加 `reviewer_invoked_at` 时间戳字段，防止审查被遗漏

## 4. 风险补丁

- 触发必须显性，避免自然语言模糊触发
- 状态必须作为单一真相源（progress.yaml），禁止多处维护
- 时间戳机制防止审查超时或遗漏