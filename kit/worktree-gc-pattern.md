# 模式:worktree 回收执行体 + 孤本保护

> 职责边界:给「定期清 worktree / 扫日历死线」配 **launchd/cron 执行体**,不是只写 TODO 日期。判据模板可移植;脚本本体由项目自持。
> 源流:Dartify 2026-08-06(日历死线过期零后果;首跑回收 11 worktree / 5.2G;2 孤本抢救后合 #84/#85)。

多会话项目先启用 [worktree-control-plane.md](worktree-control-plane.md):`agent-on worktree status` 把合同状态与本地 git 事实汇成 `safe|review|rescue`;本篇负责更长期的定时执行体与远端 PR 判据。两者都坚持**拿不准只报告、不自动删**。

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

Agent-On CLI 自身不提供 delete 子命令:先让 `worktree status` 分类,再由项目自持脚本按本篇完整判据处理;否则一条通用命令无法可靠识别 squash merge / 无 PR 孤本。

## 交付前对表(硬门 · Dartify 2026-08-08)

worktree 是**创建那一刻** default branch 的快照,之后 main 前进**不会**提醒你。凡装机 / 演示 / 截图给人看 / 对外发布:

1. `git fetch origin`
2. `git rev-list --count HEAD..origin/<default>` —— **非 0 则先对齐再构建**(ff-only 或 rebase)
3. 播报必须写:**commit hash + 与 origin/default 落差**(例:`@3fcbbc4,与 origin/main 落差 0`)
4. **禁止**从本地 HEAD 随口说「最新」——交付物(已装 App)往往不自带版本号,用户只能靠「感觉没变」发现,反馈链极长

与「squash 换 hash 误判」同族:都是 worktree 与 main 的**时间差**在不同环节咬人。

## PR DIRTY + CI 零 job(两种成因,同一张脸)

`gh pr view --json mergeStateStatus` 出 `DIRTY` / `CONFLICTING`,且 `gh pr checks` 只剩外部检查、仓内 job 一个没有——不要先查 workflow 配置。GitHub 不为 DIRTY PR 起 checkout。两种常见成因:

1. **分支起点错**(本篇既有):squash 换 hash 后新枝从旧 HEAD 长
2. **base 刚被直推进**(Dartify 2026-08-15):自己往 main 推了一笔记账/chore,与开着的 PR 改同一批文件

判据一行:DIRTY → 先看 base 是否刚动、分支是否从过期点长出。模式见 playbook/multi-contributor §三½.5。
