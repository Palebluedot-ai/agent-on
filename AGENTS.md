# AGENTS.md — agent-on 仓开发纪律

> 职责边界：本文件管**这个仓自己怎么开发**。产品总目标见 [CHARTER.md](CHARTER.md)（唯一权威）；新项目怎么用本仓见 [BOOTSTRAP.md](BOOTSTRAP.md)。对外装机见 README「给朋友的 5 分钟装机」。

## 一句话

把「AI 协作开发项目怎么启动和推进」产品化：开箱可用的脚手架，辅助 Claude Code / Codex / Grok。

## 当前阶段

**最新推荐 pin：`v0.19.0`**（输出契约三轮加固：不许有第三个筐 / 拍板六件含「在哪拍」/ 一句话全批 + Gen-1 角色体系归档与元原则第七条 + 互斥 owns 闸按事实判 + 跨窗口指令路由三权唯一与 `agent-on oncall`）。
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
7. **多会话与 worktree 自举**：单写会话可在主树；一旦同时有 ≥2 条独立写会话，主树先 clean 并退为控制轨，每条写会话独占 worktree + lane，从 fresh `origin/main` 开枝。并行模式首次跑 `agent-on worktree hooks install`，让 shared `pre-commit/pre-push` 自动执行严格边界闸；需要每日报告再显式加 `--daily-gc`。握手、每次合流后仍看盘点；GC 永远只报告。删除 worktree/分支、`--force`、跨树 add/commit 必须人工且目标明确授权，locked/dirty/unknown 不删。
8. **值守合并调度自举（2026-08-17 起）**：多会话并行、PR 排队时，本仓自己也开值守窗口（值守文档 = `docs/babysit.md`，接入件 = `kit/babysit/`）。值守在班时，全仓 PR 的 merge / update-branch / 已拍板版本批的 tag 统一归值守会话；功能会话开 PR 即交单交付（交单三型与收件地址见 kit/babysit/CONTRIBUTING-CLAUSE.md），不自己合。三条边界照 kit：真冲突打回作者；**合入授权以第 10 条为准**（2026-08-20 起 canonical PR 不再逐单问，只有硬停清单才停——原文这里写的「canonical PR 一律用户拍板」已被第 10 条取代，留着会与它正面打架）；批准只认值守会话内的用户输入（转述须向本人复核）；记账随合并权走。值守不在班回退原规则：维护者会话自合，收尾必 tag（第 6 条照旧）。
9. **跨窗口指令路由（2026-08-19 起，用户拍板）**：值守在班期间**三条权唯一归值守**——①合并（含 tag / release / 关 PR）②**对外通信**（PR/Issue 评论、Teams/Slack/邮件/webhook、一切代表本仓对外发言）③**跨窗口中转**（窗口之间传话与派工经值守，功能会话之间不横向直发）。功能会话唯一出站通道 = 给值守交单 / 回执。**发错窗口的指令不执行、原样转投**（【转投】模板与路由表见 `kit/babysit/ROUTING.md`），并给用户一行「已转投、球在值守那」。值守上岗 `agent-on oncall claim --session <会话名>`、下班 `release`；登记落 common git dir，PreToolUse 路由闸据此判定，**无人在班则整条 fail-open**。用户要在原窗口做，唯二出路是让值守下班或本窗口 `--force` 接班（都留痕）；**改权限、换等价命令偷跑不在选项里**。转投送指令不送授权——外向硬门仍须用户本人在值守会话拍板。

10. **值守全自动合并 + 独立审计（2026-08-20 起，用户全权授权）**：用户原话「You have all my full authorization，除特殊情况外我们全自动合并，不要再等我手动合了」——**合并授权从 fail-closed 翻成 fail-open**：值守碰到 PR 跑一次机器判定，命中硬停清单才停下来问，其余直接合。硬停四类穷举写死：①闸与权限自身（`.claude/settings*.json` · `hooks/` · `.github/workflows/` · `cli/src/{guard,oncall,routing}.rs` · **以及审计工具 `tools/merge-audit/` 自己**）②凭据与密钥 ③不可逆的文件改动 ④外部作者 PR。散文真相 `kit/babysit/MERGE-POLICY.md` §3/§4，**可执行真相 `tools/merge-audit/policy.json`（两者对不上以它为准）**，决策全文 `snapshot/2026-08-20-full-auto-merge-and-audit.md`。
    **配套两条硬约束，缺一条这条纪律就作废**：①每次合并必须 `merge_audit.py record` 记一行，不记账的合并会被报成 `UNRECORDED` 并**算越界** ②`merge_audit.py report` 跑不通或账本链断 → **立即退回逐单先问**（dead-man's switch）。拿掉事前审批而不补事后检测，中间那段是裸奔。
    诚实边界：审计只覆盖「合了什么」；删远端分支 / 关别人的 PR / force-push / 跨仓外向操作不是 PR 形状的，机器看不见，照旧必须问用户。

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

- 不写编排运行时代码；本仓以文档和模板为主，可执行物为 **Rust CLI**（`cli/`：doctor / guard / intake-lint / audit-lint / check / setup / worktree / tag-release / landing / oncall——以 `agent-on --help` 为准）与 **`tools/merge-audit/`**（Python 标准库，零依赖；值守合并的独立审计员，随 babysit 组件走）
- 不建远程仓 / 不动三个前身仓的内容，除非用户明确确认。**push 自己的分支与开 PR 不在此列**——那是本轨内部动作，自己做不问（判据见 [playbook/multi-contributor-protocol.md](playbook/multi-contributor-protocol.md) 的「外向硬门的边界」一节）；原句写「不 push」与自举纪律 6「交付轮次必须 push + 打 tag」直接打架，2026-08-19 用户拍板改正
- 不引入与 GStack / Superpowers 重叠的环节型功能
