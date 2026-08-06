# 案例 31:日历死线无执行体 + 孤本先救再收

> 层级:L2 | 来源:Dartify 2026-08-06 | 入册:2026-08-06

## 症状
TODO 日历死线过期零后果。worktree 普查发现 2 个「无 PR 但含唯一副本」——若按「无 PR 就删」会毁掉 +1358 行未推工作。

## 修法
launchd 每日回收;无 PR 档只报告不删;孤本:push 远端 → 回收 → rebase 落地。

## 可复用规则
定期纪律配执行体;自动回收必须保护孤本。

## 已固化到哪
playbook/multi-contributor §三½.2/.4;kit/worktree-gc-pattern.md。
