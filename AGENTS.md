# AGENTS.md — agent-on 仓开发纪律

> 职责边界：本文件管**这个仓自己怎么开发**。产品总目标见 [CHARTER.md](CHARTER.md)（唯一权威）；新项目怎么用本仓见 [BOOTSTRAP.md](BOOTSTRAP.md)。对外装机见 README「给朋友的 5 分钟装机」。

## 一句话

把「AI 协作开发项目怎么启动和推进」产品化：开箱可用的脚手架，辅助 Claude Code / Codex / Grok。

## 当前阶段

**最新推荐 pin：`v0.8.0`**（minor：记账棘轮 / worktree 回收模式；CLI 见 v0.7.0）。  
版本真相 = git tag；细节见 [CHANGELOG.md](CHANGELOG.md)。  
**下一里程碑 v1.0**：诚实验收定义见 [snapshot/2026-07-16-v10-and-setup.md](snapshot/2026-07-16-v10-and-setup.md)（外人用过 + 至少一次回流进官方消化）。  
冷启动读：本文件 + CHARTER + CHANGELOG 最新 tag 节 + 上列 snapshot。

历史：v0.2 融合 → v0.3 首次闭环 → v0.4 dogfood+guard → v0.5 分发 → **v0.6 消化攒批发版**。融合地图仍在 [snapshot/2026-07-07-fusion-map.md](snapshot/2026-07-07-fusion-map.md)（考古，非当前阶段）。

## 自举纪律（本仓遵守自己卖的方法论）

1. **决策入快照**：方向性决策写进 `snapshot/`，带日期；宪章改动需用户拍板
2. **完成贴证据**：任何「完成」声明要有命令实际输出，禁止「应该没问题」
3. **单一权威**：一个主题只有一份 canonical 文件；历史材料进 `legacy/`，不许双头
4. **commit 分层**：decision / docs / refactor / chore 分开提交，一 commit 一件事
5. **反思回流**：dogfood 中发现的方法论缺陷，修 playbook 本身并在 commit 里说明
6. **本仓对话 commit 必打 tag（2026-08-03 硬门，用户拍板）**：在 **agent-on 本仓直接对话**里，凡落地 `git commit` 并交付/push 的改动，**收尾必须** annotated tag + push tag（先封 CHANGELOG `[未发布]`、更新 README/AGENTS 推荐 pin，再 `agent-on tag-release --level … --title "…" --push`）。**禁止**只 commit/push、HEAD 仍领先最新 tag。同一交付轮次可分层多个 commit，但 **push 结束时 tag 必须钉在当前 HEAD**（一批一 tag 即可，覆盖这批全部 commit）。goal/plan 写「不要求 tag」**无效**，以本条为准。major 仍须迁移注记。可执行物为 **Rust CLI**（`cli/`，`cargo install --path cli`）。

## 文档纪律（继承 kickoff-os 六条，全文照旧）

1. 每份文档必须尽量自包含
2. 不依赖「先读另一份文档」才能理解
3. 可以重复关键事实，不为了去重而交叉引用
4. 未确认事项必须单列，不能伪装成已确认事实
5. 每份文档开头都说明自己的职责边界
6. 先固化规则，再抽象成模板，再考虑自动化

## 迭代闭环中的本仓职责（机制全文 playbook/iteration-loop.md）

- **intake/ 是承接层**：项目「结账」只许写那里；canonical（playbook/kit/bench 正文）只有本仓的消化会话能改——没读本文件的会话不许动 canonical
- **消化会话收尾四件缺一不可**：至少一处具体文件改动、CHANGELOG 条目（L3 改动成对列 playbook+kit 双落点）、**annotated tag 必打并 push**（封 `[未发布]`、更新推荐 pin）、README 对表。**范围扩大**：不限消化——见上「本仓对话 commit 必打 tag」。major 无迁移注记不许打。助手：`agent-on tag-release`
- **上游贡献**：社区只交 intake-only PR / Issue；禁止直改 playbook/kit（见 boot/settlement.md「上游贡献形态」）
- 本仓是唯一对外供货源；Euan 等项目侧仅 lock + loop-notes 等采集件

## 不做的事（宪章边界的执行版）

- 不写编排运行时代码；本仓以文档和模板为主，可执行物为 **Rust CLI**（`cli/`：doctor / guard / lint / setup / tag-release）
- 不 push / 不建远程仓 / 不动三个前身仓的内容，除非用户明确确认
- 不引入与 GStack / Superpowers 重叠的环节型功能
