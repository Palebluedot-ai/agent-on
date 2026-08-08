# 案例 32:worktree 当最新装机 + 环境免责带偏 + 闸拒命令字面

> 层级:L2 | 来源:Dartify 2026-08-08 真机交付 | 入册:2026-08-08

## 症状
1. 在落后 main **17** commit 的 worktree 直接 `flutter install`,播报「装的是最新」;用户真机感觉改动全无。  
2. 播报预埋 iOS 26 vs 18.1 免责 → 用户先问是不是系统问题;真因一条 `rev-list` 即见。  
3. classifier 拒 `reset --hard`/管道命令;换 `ff-only`、去管道、重试即过——若当成「禁止同步」会误缩交付。

## 修法
交付前 fetch 对表;没生效先查 commit/构建/安装;闸拒先换保守等价手段+重试。

## 可复用规则
worktree≠最新;交付链先于环境差;闸拒命令字面≠禁目标。

## 已固化到哪
kit/worktree-gc-pattern 交付前对表; multi-contributor; merge §6; sop/anti-hallucination; guard README。
