# project-kickoff-os

一个给 AI 协作开发项目用的启动操作系统。

它不解决某个单一业务问题，而是把“新项目怎么启动、怎么澄清需求、怎么写自包含文档、怎么拆 milestone、怎么把规则固化成 skill”这整套流程沉淀成可复用资产。

## 目标

- 不再每个新项目都从零开始聊
- 把项目启动阶段的沟通方式产品化
- 降低文档漂移、上下文丢失和大模型幻觉
- 形成可复用的 SOP、template 和 skill

## 第一版范围

- 项目启动规则
- 需求澄清框架
- 自包含文档规则
- PRD 模板
- milestone 模板
- 开发拆解模板
- skill 拆解方案

## 当前原则

- 每份文档尽量单独可读
- 不依赖文档间交叉引用
- 未确认事项必须显式列出
- 先有文档闭环，再进入代码闭环

## 当前目录

```text
context/
  00_vision.md
  01_prd.md
  02_skill-map.md
  03_rollout-plan.md
policies/
  doc-rules.md
  communication-rules.md
templates/
  requirement-pack-template.md
  prd-template.md
  milestone-template.md
skills/
  .gitkeep
AGENTS.md
```

## 当前状态

刚初始化。  
当前内容来自 `hashkey-otc-crm-v1` 项目里抽象出来的一次真实需求澄清与文档驱动开发流程。
