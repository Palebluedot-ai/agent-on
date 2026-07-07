## 1. Skill 拆解原则

不要一开始做一个超级大的 skill。  
先拆成多个边界清楚的小 skill。

## 2. 第一批 skill

### project-bootstrap

负责新项目启动入口，生成项目级规则和文档骨架。

### spec-clarifier

负责需求澄清，区分：

- 已确认
- 有方向但没定死
- 缺失信息

### prd-writer

把需求澄清收敛成 PRD。

### milestone-planner

把 PRD 拆成 milestone 和阶段目标。

### doc-selfcontain-checker

检查：

- 是否依赖文档交叉引用
- 是否有“详见另一份文档”式依赖
- 是否把未定事项写成了既定事实

## 3. 第一版顺序

先做：

1. project-bootstrap
2. spec-clarifier
3. prd-writer

后做：

4. milestone-planner
5. doc-selfcontain-checker
