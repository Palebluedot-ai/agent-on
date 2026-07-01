# Snapshot: agent-on 工具定义与边界

**日期**: 2026-07-01  
**触发**: Bigchao 在 PaleBlueDot / #agent-on 线程的指示  
**目标库**: agent-on（~/projects/Agent-On）

## 核心目的
把 Agent-On 产品化，成为新项目脚手架 + Loop Engineering 的可复用机制，让所有新项目都能直接使用这套体系，而不需要每次重复说明规则、上下文管理、snapshot 流程。

## 工具形态约束（当前共识）
- 可以是 Skill 形态，但不是必须
- 产品最终形态尚未完全确定，后续可调整
- **必须与 GStack 等已有的强 skill 完全不冲突**
- 核心是给 Codex 和 Claude Code 提供边界 + 文档驱动的启动机制
- 不做复杂的 Agent 编排，主要依赖模型自身能力，只负责设定清晰边界

## 自举要求
- 本工具的开发必须严格遵循 Agent-On 自己定义的流程（context / policies / snapshot / templates）
- 先做 Snapshot，再结合 GStack 进行融合落地

## 交付物（当前定位，可后期调整）
- 一个可 clone 的 starter 包
- 配套的协议文档

## 非目标
- 不与 GStack、Superpowers 等现有优质 skill 产生任何冲突或重叠
- 不强制自己成为一个全功能的 Agent 框架

## 下一步
- 将本 Snapshot 固化到 snapshot/agent-on-tool-definition.md
- 后续再决定具体产品形态（Skill / 轻协议 / starter 包等）
