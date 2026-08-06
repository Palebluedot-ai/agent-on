# 模式:worktree 回收执行体 + 孤本保护

> 职责边界:给「定期清 worktree / 扫日历死线」配 **launchd/cron 执行体**,不是只写 TODO 日期。判据模板可移植;脚本本体由项目自持。
> 源流:Dartify 2026-08-06(日历死线过期零后果;首跑回收 11 worktree / 5.2G;2 孤本抢救后合 #84/#85)。

## 日历死线

- **禁**:只在 TODOS 写 `截止日期:YYYY-MM-DD` 而无任何进程读取
- **要**:每日 job 扫过期行 → 写入日志/日报;清理类动作只在判据全自动满足时执行

## worktree 删除:全中才删

| 条件 | 说明 |
|---|---|
| PR 状态 | 以 `gh pr view` 为准(squash 后本地 hash 不可信) |
| 无未推提交 | `git log @{u}..` 空或无 upstream 且已评估 |
| 工作区 | clean,或仅假脏白名单(`.DS_Store` 等) |
| 静默窗口 | 如 24h 无活动 |
| **无 PR 档** | **只报告不删**——可能是从未开 PR 的唯一副本 |

默认:删 worktree 目录,**保留**远程/本地分支名除非另有策略。

## 孤本抢救三步(与回收解耦)

1. **push 远端**消单点  
2. **再**回收 worktree  
3. 择期 rebase/开 PR 落地  

跨大跨度 rebase:契约锁/棘轮测试红 → **显式更新锁**随契约走,不放宽。

## macOS launchd 要点

- `StartCalendarInterval`(非纯 `StartInterval`)— 睡眠错过可补跑  
- `PATH` / `HOME` 显式  
- 原子锁(无依赖 `flock` 也可 `mkdir`)  
- 日志: `~/Library/Logs/<job>.log`  
- `gh` 在用户会话 keyring 下免交互需实测  

## dry-run 必做

首跑 `--dry-run`:抓变量名编码 bug、误把 `.DS_Store` 当活动文件等——再启用真删。
