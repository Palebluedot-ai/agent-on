# 案例 37:等 CI 的信号源(checks --watch 滞后 / run id 裸抓)

> 层级:L2 | 来源:Dartify 2026-08-16/17 | 入册:2026-08-17

## 症状
1. push 后数秒内 `gh pr checks --watch | tail && gh pr merge` 只见 GitGuardian 一行即收工,merge 被 "base branch policy prohibits" 弹回——Actions run 注册前,checks 视图只看得到外部 app check,连续两次误判全绿(#162)。
2. `gh run list --limit 1` 抓到 Deploy Worker 的 dry-run 快 run(26 秒完),watch 完去合 #164 又被弹;同刻真 CI run 还在 queued(31956970664 vs 31956970653)。

## 修法
刚 push 完不信 `gh pr checks --watch`。先按 workflow 名拿对 run id,再盯它:
`gh run list --branch <br> --json databaseId,workflowName --jq '[.[]|select(.workflowName=="CI")][0].databaseId'` → `gh run watch <id> --exit-status`。多 workflow 仓禁止裸 `--limit 1`。

## 可复用规则
「等 CI 完成再动作」的自动化,信号源必须显式指定(workflowName 过滤 + run id),不用聚合视图的即时快照——视图滞后与快 workflow 都会造出假全绿。

## 已固化到哪
kit/babysit/BABYSIT-TEMPLATE.md §2/§4;multi-contributor §三½.6(值守调度)。
