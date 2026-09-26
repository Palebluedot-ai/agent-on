# 案例 34:闸的四张面(触发 / 读取 / 并发 / 出口)

> 层级:L2 | 来源:Dartify 2026-08-15(出口面增补 2026-08-17) | 入册:2026-08-16

## 症状
1. PR-only 的 api job 自己是红的却被合进 main;后续无关 PR 报同一条 `ENOENT`(gitignore 的 lock 被当必需输入)。main push 不跑该 job,零症状。
2. 未提交就跑「两真相面须同笔 commit」闸 → 全绿;一提交 CI 立刻红。闸读的是 `git log`,不是工作区。
3. 直推 main 清记账,把自己的 PR 撞成 `DIRTY`;仓内 CI 一个 job 都没触发。
4. (对照正例)check_privacy_retention 拦下销户清理缺口时,报错文案连同两条带具体语法的修复路线一起给——作者会话照单选路线②,15 分钟修绿,零来回澄清。

## 修法
红着不合。读历史的闸 commit 后再跑。直推 default branch 前对开着的 PR 做文件集对照。见 `DIRTY` 先查 base/起点,别查 CI yaml。写闸时把修复选项连语法写进报错文案——失败信息是可执行工单,不是谜语。

## 可复用规则
闸有四张面:触发面必须覆盖要保护的分支;读取面要分清工作区 vs 提交历史;并发面——豁免评审 ≠ 豁免脏别人的 PR;出口面——拦下之后要能自解释,报错即工单。

## 已固化到哪
multi-contributor §三½.5;merge-checklist 2c/7d/7g/7h;worktree-gc「DIRTY 两种成因」。
