# intake:inbox-radar 2026-09-05 结账（自 2026-08-15 增量，6 卡）

> 范围：`last_settlement`（扫描截至 main 57cc3b6）之后 loop-notes 与主线提交里可复用的 **AI 协作过程** 教训，扫描截至 main dc88fa5。开户球权/GTT 等 **业务域规则** 留项目 `loop-notes`（仍 `sync_status=local`），不出仓。

### source-vs-channel-roles-must-be-separated（切断外部系统写回真相时，源 / 通道 / 仓库三个角色要分开写）
- source:inbox-radar @ 51006f6 | pin v0.5.1
- evidence:2026-08-30 Grok 会话 `ef560b1 chore: drop Lark status sync` + `bead978 feat: compose Grokbot overlay and send status email` 为切断 Lark/D1 回写把 Lark 展示通道也拆了，08-31 至 09-03 每天只发一封 131 行平铺邮件；`51006f6 feat: deliver composed Grokbot status as Lark cards`（PR #55）把合成结果交回原 `send_manager_fulllist`，`docs/grokbot/HANDOFF.md` 从「Lark 不是源、不是通道、不是仓库」改为「不是源、不是仓库；只是只读展示通道」；runtime 09-04 09:03 回执 13 张卡 sent。
- confidence:medium（单项目一次；机理对任何「某系统既是展示面又曾是写入面」的切断改造普适）
- claim:要切断某外部系统对真相源的写回时，先在交接文档里把「源 / 通道 / 仓库」三个角色分开列出并逐个决定；只读通道保留原渲染器，切断写回只动 apply 入口，禁止顺手换通道。
- suggested_landing:playbook/truth-hierarchy 增「源≠通道」一节；kit/phase-card-template 的 disturbance 行示例加「不许顺手换展示通道」
- rollback:revert 落地 commit
- trace:loop-notes.md §2026-09-03「『不是真相源』≠『不是通道』」
- 状态: landed@同批（playbook/truth-hierarchy.md 五⅞.1「源≠通道≠仓库」）

### skip-ci-state-commits-hide-red-main（状态提交带 [skip ci] 的项目，监控必须另看最新非 skip 的 CI 结论）
- source:inbox-radar @ cc79fc1 | pin v0.5.1
- evidence:`gh run list --branch main` 显示 2026-08-30 12:22Z 起 14 次连续 failure（4 个测试：3 个静态模板缓存失效测试在 overlay 生效后必然失败、1 个断言提交的 overlay 为空），每日 `chore: daily pipeline run state … [skip ci]` 提交让 main 顶上永远没有新 CI 结论；`cc79fc1 test: verify prompt-identity contract with committed Grokbot overlays` 后 PR #55 合入，main CI 恢复绿。
- confidence:medium（单项目；机理对所有「机器人定时提交 + skip ci」的仓库普适）
- claim:允许状态提交 `[skip ci]` 的仓库，monitoring 必须另有一路读「最新非 skip 提交」的 CI 结论，并把它写进状态源；测试契约随设计改，数据文件里有真实条目是常态而不是异常，不许断言它为空。
- suggested_landing:playbook 运行态 / Error Signal 一节；kit/progress-template 的 monitoring_summary 说明行加「最新非 skip CI 结论」；bench 短案例
- rollback:revert 落地 commit
- trace:loop-notes.md §2026-09-03「`[skip ci]` 让红灯隐身」
- 状态: landed@同批（playbook/truth-hierarchy.md 五⅞.3 + kit/progress-template.yaml monitoring_summary 注释）

### state-source-must-move-with-production-cutovers（改变「每天实际发生什么」的合流必须同批更新状态源与 product map）
- source:inbox-radar @ 946e754 | pin v0.5.1
- evidence:`docs/state/progress.yaml` 停在 2026-07-21 写 launchd unloaded、生产 blocked，而 `launchctl print gui/501/com.chao.inboxradar` 显示已加载 runs=10，runtime `logs/summary.log` 自 07-31 起每日 exit=0；同期 README 写 Lark 卡片 live、HANDOFF 写 email，三份文档三种说法；`946e754 docs: Lark is the display channel, GitHub is truth; sync state layer to reality` 同步。
- confidence:high（同一项目两次实证：07-31 加载 launchd 未更新状态源，08-30 换通道未更新 product map）
- claim:任何改变「每天实际发生什么」的合流（加载调度器、切换通道、启停发送）必须同批更新状态源与 README product map；接续握手读文件前先核一次运行实况（调度器状态、最近一次 run 日志），文件与实况打架时以实况为准并当场点明。
- suggested_landing:playbook/truth-hierarchy 或 boot/session-handshake 读取表增「运行实况核对」行；kit/progress-template 顶部注释加「生产切换同批更新」
- rollback:revert 落地 commit
- trace:loop-notes.md §2026-09-03「状态源六周没动」
- 状态: landed@35b6952 + 同批（truth-hierarchy 五⅞.2 + progress-template.yaml 顶部注释 @35b6952；boot/session-handshake.md 读取表新增「状态源与现实打架吗」行 @同批）

### worktree-guard-claim-before-first-commit（新会话先登记 lane；陈年树先建分支再 park；带独有提交的树 base 重钉自身 HEAD）
- source:inbox-radar @ 671e1ec | pin v0.5.1
- evidence:PreToolUse `agent-on guard` 输出「UNREGISTERED ×5 → RESULT: FAIL」拦下首次提交；`agent-on worktree claim --id lark-channel-s2 --owns …` 后本轨 PASS 但其余 4 棵未登记树仍连坐；三棵 detached 树 `claim --parked` 报「detached HEAD cannot claim a lane; create a branch first」，`checkout -b stale/*` 打标签后 park 成功；带 12/72 个独有提交的树用 `--base <自身 HEAD>` park 后 `check` PASS；auto-mode 分类器拦下把 4 个 `worktree remove --force` 与 `branch -d` 合在一条的命令，拆成一树一命令后逐条通过；guard 在命令执行前评估当前树，含提交动作的修复命令本身被拦，先跑不含提交的修复再提交。
- confidence:high（agent-on 自家工具的可复现行为；与 snapshot/2026-08-20-gate-exit-reachability 的死锁分析同族）
- claim:多 worktree 项目的新会话在第一次会写入之前先 `agent-on worktree claim` 并 `check`；陈年 detached 树先打分支标签再 `claim --cwd … --parked`，带独有提交的树用 `--base <自身 HEAD>` 让 park 成为完解；删除永远留给用户，获批后一树一条命令；含提交动作的修复命令会被 guard 在执行前拦下，修复与提交分两条命令。
- suggested_landing:kit/worktree-control-plane.md 死锁三解节（重写时并入「detached 需分支」「base 重钉」两条实证）；boot/session-handshake 执行轨行；bench 案例
- rollback:revert 落地 commit
- trace:loop-notes.md §2026-09-03「guard 连坐要先登记再提交」+ §2026-09-04「分类器与 guard 都在命令执行前评估」
- 状态: landed@d83fd19（kit/worktree-control-plane.md「陈年树与带独有提交的树」三条：detached 先建分支 / base 重钉自身 HEAD / 一树一条命令；「修复与提交分两条」本仓 v0.20.0 已固化）。这段随 v0.22.0 的闸改动 commit 一起发出，CHANGELOG 当时漏记，本批补记；同批加注：第 2 条在 v0.20.0 之后的闸下已无用处

### compound-shell-chain-breaks-after-heredoc（agent 一条 shell 命令只做一件有状态的事，heredoc 之后的语句不受前面失败保护）
- source:inbox-radar @ a92eaee | pin v0.5.1
- evidence:一条 Bash 里 `switch -c … && cherry-pick -q d9eec5d && python3 - <<'EOF' … EOF` 之后另起一行 `add … && commit --amend --no-edit && … 推送`；`-q` 不是 cherry-pick 的合法参数，前半段失败，但 heredoc 结束后的 amend 照跑，把 main 顶上的 merge commit `59ed22d` 复制成 `984ce07` 并推成了 PR #56 的唯一提交（空 diff）；修复：`reset --hard origin/main` → 正确 cherry-pick → `--force-with-lease` 覆盖自己一分钟前推的分支 → `a92eaee`。
- confidence:medium（单次事故；机理对所有用 heredoc 混排多步版本控制操作的 agent 命令普适）
- claim:一条 shell 命令只做一件有状态的事，或整条用 `&&` 串到底；heredoc 结束后另起的语句不在链里，失败不会阻断它；改写历史前先 `log origin/main..HEAD` 核对分支内容；强推只用 `--force-with-lease`，且只对自己刚推、无人碰过的分支。
- suggested_landing:playbook 工具行为 / anti-hallucination「命令链」一节；kit 的 merge-checklist 或 sop「多步版本控制操作串 &&，先看 log 再改历史」行；bench 案例
- rollback:revert 落地 commit
- trace:loop-notes.md §2026-09-04「复合命令里 heredoc 之后的语句不在 && 链里」
- 状态: landed@同批（playbook/anti-hallucination.md C 附6 第 12 条）

### receipt-contract-tests-must-reach-real-consumer（改回执/契约字段时，测试必须穿到真实消费者，不能停在生产者断言）
- source:inbox-radar @ 51006f6 | pin v0.5.1
- evidence:首版把可选邮件失败记为 `required=False, ok=False` attempt，单测断言「邮件失败不判定当天失败」通过；/ship 的 adversarial、coverage、plan-audit 三路独立指出 `scripts/daily-run-recovery.py:448-462` 只认 `status == "succeeded"`，实际会 exit 42、次日 exit 17 并在 replay 时重发全部卡片；修复把失败改记 `DeliverySkip(email_send_failed)`，并新增 `test_deliver_trusted_run_email_failure_receipt_passes_recovery_and_replays_without_resend` 直接调用 recovery 的 `delivery_complete` 与二次 `deliver_trusted_run`（`51006f6`，pytest 1637 passed）。
- confidence:high（三路审查独立收敛；TDD 通用教训）
- claim:凡改动回执、状态码或契约字段，验收测试必须走到真实消费者（recovery、replay、下游校验器），在生产者层通过的断言不算证据；多路审查对同一处收敛时优先修它。
- suggested_landing:playbook 测试纪律（TDD 与证据）「契约测试要到消费者」一段；kit/phase-card-template 验收标准示例行；bench 案例
- rollback:revert 落地 commit
- trace:loop-notes.md §2026-09-04「回执契约要穿到真实消费者」
- 状态: landed@同批（anti-hallucination C 附6 第 2 条 + kit/phase-card-template.md 验收区「回执/契约字段」条目）
