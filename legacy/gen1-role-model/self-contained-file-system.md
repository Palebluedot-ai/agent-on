---
title: Self-Contained File System Pattern
status: active
created: 2026-06-28
last_updated: 2026-06-28
last_change: "新增控制论映射章节；完善 Logging & Observability 内容；统一 Header 格式并加入 last_change 记录"
self_contained: true
---

# Self-Contained File System Pattern

## 核心原则

每个被 Agent 执行的文件必须是自包含的（Self-Contained）。

- 高内聚、低耦合
- 执行时禁止跨文件做 semantic reference
- Agent 拿到一个文件 + 精确行范围后，即可直接工作，无需读取其他文件

此原则优先级高于 DRY（适当的重复是允许的）。

## 实现机制

### 1. 文件级自包含声明

每个可被 Agent 执行的 phase 文件必须以 YAML frontmatter 开头：

```yaml
---
phase: 3.2
self_contained: true
required_context:
  - domain: "Order Aggregate 定义（已内联在本文档 120-145 行）"
  - example: "Repository 实现样例（已内联在本文档 200-230 行）"
pointer_format: "file:line-line"
---
```

### 2. 精确范围锚点（Pointer）

使用统一标记实现行级精准指向：

```dart
// [PHASE-3.2:AGGREGATE:START]
// ... 完整、独立的 Order Aggregate 实现（包含所有必要领域知识） ...
// [PHASE-3.2:AGGREGATE:END]
```

`progress.yaml` 中的指针格式：

```yaml
tasks:
  - id: phase-3.2-aggregate
    file: docs/development/phases/phase-3.2.md
    range: 87-156
    self_contained: true
    status: in_progress
```

### 3. Agent 编排约束

- **state-manager Agent** 是唯一可以生成和验证指针的角色
- Implementer / Reviewer Agent **禁止主动读取指定范围之外的文件**
- 所有状态变更必须通过 state-manager

### 4. 阶段文件设计要求

每个 `phase-*.md` 必须内联以下内容（保证自包含）：

- 本阶段目标与验收标准
- 必要的领域模型片段（从 architecture 层复制关键定义）
- 关键代码样例
- 明确的范围锚点

## 控制论映射（Engineering Cybernetics）

将控制论思想系统性映射到文件与 Agent 体系：

| 控制论概念         | 映射到框架                          | 具体实现要求                              | 目的                     |
|--------------------|-------------------------------------|-------------------------------------------|--------------------------|
| **Feedback Loop**  | 每个 phase 的执行闭环               | phase 文件必须包含「执行 → 审查 → 修正 → 验证」结构 | 保证可迭代、可收敛       |
| **Observability**  | 状态 + 日志作为传感器               | progress.yaml + 结构化 execution log      | 让系统状态可见、可追溯   |
| **Controllability**| 单一控制点                          | 所有状态变更必须经由 state-manager        | 避免多头控制导致混乱     |
| **Setpoint**       | 验收标准作为目标值                  | 明确定义「可测量的 setpoint + 误差阈值」  | 使 review 有明确停止条件 |
| **Error Signal**   | Review 失败或测试失败               | 必须记录 error signal 并触发反馈回路      | 驱动修正而非忽略问题     |
| **Disturbance Rejection** | 上下文衰减、scope creep         | 在 phase 文件中列出主要干扰源 + 抑制策略  | 提高系统鲁棒性           |
| **Stability**      | Phase-lock + Scope-lock             | 阶段切换必须显式锁定，禁止静默漂移        | 防止需求/执行发散        |

**强制要求**：每个 phase 文件必须在 frontmatter 或独立区块中声明本阶段的 **Setpoint** 和主要 **Disturbance**。

## Logging & Observability

**核心目的**：让系统在出现问题时可快速 debug，尤其是在多 Agent 长时间运行后。

### 必须记录的日志类型

1. **状态变更日志**（最高优先级）
   - 任何对 `progress.yaml` 的修改
   - 记录：时间、操作者（哪个 Agent）、任务 ID、变更前状态 → 变更后状态、触发原因

2. **Agent 执行日志**
   - 任务接收、范围读取、决策过程、输出结果、是否成功
   - 必须包含精确的 `file:line-line` 信息

3. **决策审计日志**（Decision Audit）
   - 需求澄清、范围锁定、验收通过/失败、重要冲突解决
   - 这些日志对事后复盘和防止幻觉至关重要

4. **Error & Disturbance 日志**
   - 每次 feedback loop 中的 error signal
   - 检测到的干扰（如上下文不足、指令模糊）

### 日志存放与格式建议

- 存放位置：`docs/state/logs/`（按 phase 或日期分文件）
- 推荐格式（结构化，便于 grep 和后续分析）：
  ```json
  {
    "timestamp": "2026-06-28T14:23:05+08:00",
    "agent": "state-manager",
    "action": "update_task_status",
    "task_id": "phase-3.2-aggregate",
    "range": "docs/development/phases/phase-3.2.md:87-156",
    "from": "in_progress",
    "to": "review_failed",
    "reason": "Setpoint not met: missing error handling",
    "log_id": "log-20260628-142305-001"
  }
  ```

### Debug 价值

- 当系统出现反复失败或幻觉时，可通过 log 快速定位是哪个环节的 setpoint 未达成、哪个干扰未被抑制。
- log 成为「系统黑匣子」，极大降低事后调查成本。

## 权衡与约束

- 接受一定程度的知识重复，换取执行时的零耦合
- 如果一个 phase 文件过大，应拆分为多个**自包含的子阶段**，而非拆成互相引用的多个文件
- 架构层文档（`docs/architecture/`）主要供人阅读，Agent 执行时不直接依赖

## 适用场景

- DDD 项目 + AI 多 Agent 协作开发
- 需要严格控制上下文衰减（Context Decay）的场景
- 要求高可审计性与可回滚性的工程流程
- 需要长期可 debug 的复杂 Agent 系统

---

## 相关文件

本文件已自包含，无需跳转阅读其他文档。

### 核心框架

| 文件 | 定位 | 状态 |
|------|------|------|
| `self-contained-file-system.md` | 自包含原则 + Pointer 机制 + 控制论映射 | 基础框架 |
| `three-stabilizers.md` | Lean / TOC / XP 三个固化剂 | 质量控制 |
| `monitoring-and-error-signal.md` | Monitoring 强制规则 + Error Signal 强度标准 | 可观测性与反馈 |

### 存档与清单

| 文件 | 定位 | 状态 |
|------|------|------|
| `file-system-archive.md` | 当前框架文件系统存档清单 | 存档索引 |

### Agent 角色与全局规则

- `agents/state-manager.md`
- `AGENTS.md`