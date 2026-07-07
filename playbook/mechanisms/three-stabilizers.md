---
title: Three Stabilizers for Agent Loop
status: active
created: 2026-06-28
last_updated: 2026-06-28
last_change: "定义 Lean / TOC / XP 三个固化剂的强制规则；明确时间盒、瓶颈管理和简单设计约束"
self_contained: true
---

# Agent Loop 的三个固化剂

目标：让 Agent 在长时间、多轮执行中保持高质量，减少退化。

## 1. Lean 的短反馈 + 消除浪费

**核心规则**：
- 每个完整反馈回路（执行 → 审查 → 修正 → 验证）必须控制在 **15-30 分钟** 内。
- 禁止「大而全」的任务拆分，强制使用「最小可工作自包含单元」。
- 任何跨文件引用都被视为浪费，必须内联或重构。

**Agent 约束**：
- Implementer 必须在收到任务后 5 分钟内输出第一版可验证结果。
- Reviewer 必须在 10 分钟内给出带「具体差哪里 + 修正方向」的反馈。
- 超过时间盒的任务必须被 state-manager 打断并重新拆分。

**固化位置**：每个 phase 文件的 frontmatter 必须声明本阶段的「最大反馈回路时长」。

## 2. TOC 的瓶颈管理

**核心规则**：
- 每个阶段开始前必须显式回答：「当前这个 loop 的瓶颈是哪个环节？」
- 常见瓶颈：state-manager 的指针生成、implementer 的领域知识不足、reviewer 的反馈质量低。

**Agent 约束**：
- state-manager 每完成一个阶段，必须输出「当前瓶颈分析」。
- 所有资源（示例、log 强度、约束规则）优先分配给当前瓶颈环节。
- 瓶颈未解决前，禁止开启新阶段。

**固化位置**：progress.yaml 增加 `current_bottleneck` 字段，由 state-manager 维护。

## 3. XP 的简单设计 + 持续重构

**核心规则**：
- 永远优先「能跑的最简单实现」，而不是「看起来完美的设计」。
- 每完成一个反馈回路，必须回答：「这段代码是否可以更简单？」
- 禁止「为了未来扩展」而提前复杂化。

**Agent 约束**：
- Implementer 输出代码时，必须同时输出「本实现最复杂的三处」并说明是否可以简化。
- Reviewer 必须把「过度设计」作为高优先级问题反馈。
- 每 3 个反馈回路必须进行一次显式重构（哪怕只是小范围）。

**固化位置**：phase 文件模板中增加「简单设计检查点」区块。

## 三者的协同关系

- **Lean** 保证 loop 不变长（效率）
- **TOC** 保证资源集中在最该加强的地方（精准）
- **XP** 保证代码和设计不随时间腐烂（可持续）

三者结合，能显著降低 Agent 在长时间运行后的质量衰减。

## 相关文件

- `self-contained-file-system.md`
- `docs/state/progress.yaml`
- `agents/state-manager.md`