# 跨窗口指令路由：三权唯一 + 误投转投 + 机械闸（2026-08-19 决策快照）

> 职责边界：本页记**这次拍了什么、为什么这么拍、拿什么证据**。机制正文在 `kit/babysit/ROUTING.md`（唯一真相），治理投影在 `kit/babysit/CONTRIBUTING-CLAUSE.md` 与 `AGENTS.md` 自举纪律 9，实现在 `cli/src/oncall.rs` + `cli/src/guard.rs` + `hooks/hooks.json`。本页不重复展开路由表与模板。
> 触发：用户 2026-08-19 原话——「多并发的这个窗口里边，定一个规则、一个限制：只由一个值守的人负责。只能由他去和其他人、其他窗口沟通，不能合并、不能 merge PR。我有可能在不同的窗口发错指令，比如在某一个功能窗口让它去 merge、或者让它自己用 Teams 沟通——这类直接转到值守那个窗口里。有些功能明显不该在那个窗口操作，要换到适合的窗口。」
> 前置：2026-08-19 [跨窗口值守调研](2026-08-19-babysit-cross-window-research.md)（能力实测与缺口）。本页是那份调研的第一次机制落地。

## 一句话

**一条指令的执行窗口，由指令的类别决定，不由用户碰巧在哪个窗口打字决定。** 合并 / 对外通信 / 跨窗口传话三权唯一归值守；发错窗口的指令不执行、原样转投；纪律之外补一层退出码，无人在班时整条沉默。

## §1 拍了什么（三条）

1. **三权唯一**（此前只有合并权是唯一的）：新增**对外通信权**（PR/Issue 评论、Teams/Slack/邮件/webhook）与**跨窗口中转权**（窗口之间传话经值守）。功能窗口唯一出站通道 = 给值守交单 / 回执。
2. **误投转投**：功能窗口收到不属于自己的指令 → 不执行 → 判类别 → `SendMessage` 转投值守（【转投】模板）→ 给用户一行「已转投、球在值守那」。反向亦然：值守收到功能活转投回作者轨（值守零代修的既有铁律没变）。
3. **机械闸**：`agent-on oncall` 在班登记 + PreToolUse 路由闸（`Bash` 与 `SendMessage` 两个 matcher 共用一个 guard）。三态：无人在班 fail-open / 值守窗口放行 / 功能窗口 exit 2 并给出转投模板与两个逃生门。

## §2 六个设计选择（每条都有被否掉的替代方案）

| 选择 | 选了什么 | 否掉了什么 | 为什么 |
|---|---|---|---|
| 身份键 | **worktree 路径** | session id | 会话是班次、树是资产；同树换会话续跑应当自动继承值守身份，且与 lane 体系同构（值守本来就有自己的 worktree + lane） |
| 登记存哪 | **common git dir 的 `agent-on/oncall.json`** | `docs/babysit.md` 的「在班值守地址」行 | 那一行是**每棵 worktree 各一份的文件副本**——功能窗口在自己分支上读到的可能是任意旧版本。快照仍写给人看，机器寻址以登记为准 |
| 闸的默认 | **fail-open**（无人在班全放行） | fail-closed | 本仓一贯 fail-closed，这次故意反过来：忘了上岗只是没有闸，fail-closed 却会把单人开发和值守下班后的仓库锁死。代价写进诚实边界 |
| 匹配方式 | **deny-list**（认识已知形状的命令） | allow-list | allow-list 会拦死一切正常开发。所以**闸是黑名单、纪律是白名单**：闸兜住顺手就做的，蓄意绕过归治理 |
| 横向沟通 | **一律经值守中转** | 功能窗口之间直发 | 直发快一跳，代价是调度信息散回各窗口聊天记录，接班的人一条看不到。中转延迟有限，信息散失不可逆 |
| 逃生门 | **两个，都改在班登记** | 无逃生门 / 静默绕过 | 用户是老板，但绕闸不能靠改权限或换等价命令。`oncall release --force`（值守下班）与 `oncall claim --force`（本窗口接班）都留痕 |

## §3 核验留痕（本地实测九项，全中）

临时仓（`git init` + 一棵 feature worktree）跑本轨 debug 二进制，**未触碰本仓在班登记**（真值守 `worktree-output-clarity-e02325-*` 当时在班）：

```text
1) 无人在班 · 功能窗口 gh pr merge            → exit 0   （闸沉默）
2) oncall claim --session babysit-window-a    → ONCALL CLAIMED
3) 功能窗口 oncall status --json              → present=true, self_is_oncall=false（跨 worktree 读到同一份）
4) 功能窗口 gh pr merge                       → exit 2 + 转投模板 + 在班地址 + 两个逃生门
5) 值守窗口 gh pr merge（同一条命令）          → exit 0
6) 功能窗口 curl https://webhook.office.com/… → exit 2   （Teams 外发）
   功能窗口 gh pr comment 17                  → exit 2
   功能窗口 git push origin v0.18.0           → exit 2   （tag 归值守）
7) 功能窗口 gh pr create --fill               → exit 0   （交付动作照常）
   功能窗口 gh pr view 17 --json mergeable    → exit 0
8) SendMessage → babysit-window-a-02          → exit 0   （交单通道，前缀匹配）
   SendMessage → other-feature-window-9c      → exit 2   （横向被拦）
9) oncall release → 功能窗口 gh pr merge      → exit 0   （恢复沉默）
```

自动化测试：`cargo test` **172 passed / 0 failed**（含 12 条 oncall 单元测试 + 7 条 `tests/oncall_routing.rs` 端到端，端到端用真实二进制与真实 PreToolUse stdin 契约）。`cargo clippy --all-targets` 零 warning。

## §4 诚实边界（不写在这里就会被当成没有）

- **闸拦命令，不拦意图**：换个写法照样做得出去。它防「顺手就做了」，不防蓄意绕过。
- **MCP 外发不在闸内**：Telegram / Slack MCP 等按各自工具名注册，本轮只挂了 `Bash` 与 `SendMessage` 两个 matcher。要机械兜住得自己加 matcher，否则那条通道只有纪律层。
- **单值守靠登记不靠锁**：两个窗口同时 `--force` 抢，后写的赢。互斥的真正保障仍是「一次只开一个值守窗口」这条人的纪律——`README.md` 诚实边界里「不靠锁机制」那句仍然成立，只是现在多了机器可寻址与留痕。
- **本轮没做**：没改 `docs/babysit.md`（值守自己的资产，在班期间归它）；没重装机器上的 `agent-on`（重装会让装机版本领先 main，正是调研 §4 记过的「版本号相同功能不同」的坑）；没动 `kit/output-contract.md`（转投回执格式写在 ROUTING §3，与面板四字段同构）。三件都在交单里点名给值守 / 用户拍。
- **转投有延迟**：中转比直发慢一跳，是为「全场状态可读」付的价，不是疏漏。

## §5 顺手发现，不代修（lane 控制面两个死角）

复用一棵 landed 的 worktree 做新题目时（本轨就是：`landing-control-plane-v1` 这棵树改做跨窗口路由），控制面**三条路全堵**：

1. `set-status active` 报 `invalid lane transition: landed -> active`——landed 是终态，无出口
2. `forget` 拒绝：`refuse to forget … while worktree still exists`
3. lane id 不能改名（id 即文件名，无 rename）

结果：只能 `worktree edit` 改 goal/branch/owns，**status 永远卡在 landed、id 永远显示旧题目**。副作用不止于可读性——landed 不算 live，`owns` 重叠闸对本轨**不设防**（别的轨可以 claim 走本轨正在写的文件）。

这与调研 §9 记的「窗口复用换题目要重划」是同一根问题的两半：那半是提醒人去重划，这半是**重划本身缺一段生命周期**。可能的解：允许 `landed → parked → active`，或给 `edit` 加 `--id-rename`，或让 `forget --relocate` 接受仍在的 worktree。**本轨不代修**——它属于 worktree 控制面，不属于本轨 owns。

## §6 待接（明确交给谁）

| 事项 | 归谁 | 说明 |
|---|---|---|
| 合并本轨 PR + 打 tag | **在班值守** | canonical 改动（AGENTS/kit/cli/hooks），按 MERGE-POLICY 属「必须先问档」，须用户拍板 |
| `docs/babysit.md` 接入（§1 上岗加 `oncall claim`、§3 补三权、§7 下班加 `release`） | **在班值守** | 那是值守自己的 owns，本轨不越界代改 |
| `cargo install --path cli` 重装装机 CLI | **用户 / 值守** | 合入后再装，避免装机版本领先 main |
| lane 生命周期死角（§5） | 后续轨 | 与本轨主题相邻但不同轨 |
