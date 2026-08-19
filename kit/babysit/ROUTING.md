# 跨窗口指令路由：三权唯一 · 误投转投 · 机械闸

> 职责边界：本页管**多窗口并行时，一条指令该由哪个窗口执行**——谁有合并权、谁有对外通信权、谁负责窗口之间传话，以及**用户把指令发错窗口时怎么办**。
> 它不管值守每轮怎么巡（见 [BABYSIT-TEMPLATE.md](BABYSIT-TEMPLATE.md)）、不管哪些 PR 免问即合（见 [MERGE-POLICY.md](MERGE-POLICY.md)）、不管一轮话怎么说（见 [../output-contract.md](../output-contract.md)）、不管谁能改哪些文件（见 [../worktree-control-plane.md](../worktree-control-plane.md)）。
> **本页是「谁执行」的唯一真相**：值守模板、治理条款、guard README 一律引用本页，不各自展开。
> 源流：2026-08-19 用户拍板——「只由一个值守的人负责，只能由他去和其他人、其他窗口沟通，不能合并、不能 merge PR；我有可能在不同的窗口发错指令，这类直接转到值守窗口去」。机制在 agent-on 仓自举实现（`agent-on oncall` + PreToolUse 路由闸）。

## 一句话

**一条指令的执行窗口，由指令的类别决定，不由用户碰巧在哪个窗口打字决定。** 值守在班时，合并 / 对外通信 / 跨窗口传话三件事唯一归值守；发错窗口的指令**不执行、原样转投**，不是拒绝了事。

## §1 三条唯一权（值守在班期间）

| 权 | 内容 | 谁有 |
|---|---|---|
| **合并权** | `gh pr merge` · 服务端 `update-branch` · 关/重开 PR · 打 tag 与 push tag · 发 release · 直推 main | **仅值守** |
| **对外通信权** | PR / Issue 评论与 review · 建 Issue · Teams / Slack / 邮件 / 任何 webhook 外发 · 代表本项目对外发言 | **仅值守** |
| **跨窗口中转权** | 窗口 A 要跟窗口 B 说话，一律经值守转；功能窗口之间不横向直发 | **仅值守** |

功能窗口保留的**唯一跨窗口出站通道**：给值守发交单 / 撤单 HOLD / READY / 回执（三型见 [CONTRIBUTING-CLAUSE.md](CONTRIBUTING-CLAUSE.md)）。除此之外，功能窗口的所有动作都朝内——写自己 lane 的文件、跑测试、开 PR。

**「跨窗口」不含会话内部**：一条会话派出的子代理、`main`、同会话的 teammate 之间怎么传话，是那条会话自己的事，三权一概不管。三条唯一权约束的是**窗口与窗口之间**。

**为什么中转不许省**：横向直发看上去快一步，代价是没人再知道全场在发生什么——值守是唯一持有队列、波次、依赖与拍板通道的角色，绕过它 = 把调度信息打散回各个窗口的聊天记录里，接班的人一条都看不到。中心化传话的延迟是有限的（值守本来就在短循环里），信息散失是不可逆的。

**值守不在班**（无人登记）：三条权回退本仓原规则——agent-on 是「维护者会话自合，收尾必打 tag」（AGENTS 自举纪律 6/8）。机械闸此时整条 fail-open，见 §4。

## §2 路由表（发错窗口时，按这张表转）

**按指令类别判，不按用户在哪个窗口打的字判。**

| 指令类别 | 典型原话 | 归谁 | 收到的窗口怎么办 |
|---|---|---|---|
| 合并 / 远端公共态 | 「把 #17 合了」「追平一下」「打个 tag」 | 值守 | **不执行**，转投值守（§3） |
| 对外通信 | 「用 Teams 跟他说一声」「在 PR 上回一句」「发个 Issue」 | 值守 | **不执行**，转投值守 |
| 跨窗口传话 | 「让那个改 CLI 的窗口先停一下」 | 值守 | **不执行**，转投值守；由值守发给目标窗口 |
| 改别人 lane 的文件 | 「顺手把 `cli/` 那个 bug 修了」（本轨 owns 不含 `cli/`） | 拥有该 owns 的轨 | **不执行**，转投值守，由值守派给那条轨；越 owns 写还会被边界闸拦（[worktree-control-plane](../worktree-control-plane.md)） |
| 消化 canonical | 「把这条经验写进 playbook」 | agent-on 仓的消化会话 | 不执行，落 `intake/` 素材并告知用户换会话（口令「agent-on 结账」） |
| 跨仓 git 写 | 在项目端会话里对 agent-on 仓 commit | 用户切 agent-on 会话 | 已有跨仓闸拦；提示切会话 |
| 本轨开发 | 写代码、跑测试、开 PR、交单 | **自己** | 直接做，别转投 |
| 反向误投：功能活派到值守 | 在值守窗口说「把这个 bug 改了」 | 对应功能轨 | 值守**零代修**（既有铁律），转投给作者轨；无人认领则报用户 |

**边界情形，一条判据够用**：这条指令改变的是**本轨内部状态**（自己的文件、自己的分支、自己的 PR 草稿），还是**全场共享状态**（main、tag、别人的文件、外部世界的人）？前者自己做，后者归值守。

## §3 转投协议（四步，缺一不可）

收到不属于本窗口的指令时：

1. **先不执行**——哪怕命令就在手边、哪怕权限也够。
2. **判归属**：按 §2 的表定类别。拿不准 = 当作值守的（fail-closed），让值守再分。
3. **转投**：`SendMessage` 给在班值守，模板：

   ```text
   【转投】来源窗口 <lane id>｜类别：合并 / 对外通信 / 跨窗口 / 越 owns
   用户原话：<原样引用，不要改写成你的理解>
   请求动作：<一句话，比如「合并 PR #17」>
   我已做的：<比如「PR 已开、描述写全」；没有就写「无」>
   回执给：<本窗口会话名>
   ```

   收件地址：`agent-on oncall status`（机器可读：`--json` 的 `session` 字段）。**不要靠 ListAgents 猜名字**——那是全机器的，会列出别的仓的窗口。

4. **给用户一行回执**，格式与 [output-contract](../output-contract.md) 状态面板的四字段同构：

   ```text
   <本轨名> │ 这条是合并类，不归本窗口，已转投值守 │ 你不用管 │ 值守会话 <地址>
   ```

   一行说完，不解释制度、不复述规则原文。用户要的是「这事有人管了、不在我脑子里了」。

**用户当场坚持要在本窗口做**（「我知道，就在这里合」）：用户是老板，但**闸不给绕**。合法出路两条，都会改在班登记因而留痕：

- 让值守下班：`agent-on oncall release --force`（此后三条权回退原规则）
- 本窗口接班当值守：`agent-on oncall claim --session <本窗口会话名> --force`

**不在选项里的**：改自己的权限配置、换等价命令偷跑、把命令拆碎绕过模式匹配。这三样不是「灵活」，是把机械闸变成装饰。

## §4 机械闸（把纪律钉成退出码）

### 在班登记：一个跨 worktree 的真相源

```bash
agent-on oncall claim --session <本窗口会话名> [--note "第二班"]   # 上岗（同一时间至多一个）
agent-on oncall status [--json]                                    # 谁在班、多久、交单地址（任何窗口可读）
agent-on oncall whoami [--json]                                    # 本窗口是不是值守
agent-on oncall route --path <文件> [--json]                       # 这个文件归哪条轨（值守派工用，见 §5）
agent-on oncall release [--force]                                  # 下班
```

登记落在 **common git dir 的 `agent-on/oncall.json`**（与 lane 台账同处），所以每棵 worktree 读到的是同一份。
**这不是文档能替代的**：`docs/babysit.md` 的「在班值守地址」行是每棵 worktree 各一份的文件副本——功能窗口在自己的分支上 `cat` 它，读到的可能是任意旧版本。交接快照仍然要写（给人看、随 git 走），但**机器寻址以登记为准**。

`--session` 填会话名或稳定前缀（前缀匹配，够长到不撞车即可）；会话读不到自己的精确后缀，上岗时用 `/list-agents`（ListAgents）确认一次再填。

### 闸怎么判

PreToolUse 两个 matcher 共用一个 guard（注册见 [../../hooks/README.md](../../hooks/README.md)）：

| matcher | 命中什么 | 结果 |
|---|---|---|
| `Bash` | `gh pr merge` · `gh api -X PUT …/pulls/…` · `git push --tags` / push tag / push main · `gh pr close` · `gh release create` · `gh pr comment` / `gh issue create` · chat webhook（Slack / Teams / Discord / Telegram / Google Chat）· `sendmail` 等 | 非值守窗口 → **exit 2** + 转投模板 |
| `SendMessage` | 收件人是**另一个已登记窗口**（地址前缀匹配某条 lane 的 worktree 目录名） | 非值守窗口 → **exit 2**；发给值守（交单 / 回执）→ 放行；发给 `main` / 子代理等**会话内部**地址 → 放行 |

**为什么 SendMessage 判的是「是不是另一个窗口」，不是「是不是值守」**：三权管的是**窗口之间**的沟通，一条会话内部 lead ↔ subagent 的传话从来不在里面。判据取自 lane 台账——窗口的会话名由它的 worktree 目录派生（本仓值守：worktree `worktree-output-clarity-e02325` → 会话 `worktree-output-clarity-e02325-02`），所以能前缀匹配到**别的** lane 的 worktree 目录名的地址才是真窗口。匹配不到的（`main`、`researcher`、任何子代理名）一律放行。

**三态**：

1. **无人在班** → 整条闸 fail-open，一律 exit 0（单人单窗口、值守下班后照常干活）
2. **值守在班 · 本窗口是值守** → exit 0（值守本来就该做这些）
3. **值守在班 · 本窗口不是** → exit 2，stderr 给出：类别 + 在班地址 + 填空版转投模板 + 两个逃生门

**登记失效自动降级**：登记的 worktree 已不存在（窗口关了没下班）→ 按「无人在班」处理，避免死锁全场；`oncall status` 会显示「登记已失效」并给清理命令。

### 最小实测

```bash
agent-on oncall claim --session babysit-window-a --cwd "$ONCALL_WT"
echo '{"tool_name":"Bash","cwd":"'"$FEATURE_WT"'","tool_input":{"command":"gh pr merge 17 --merge"}}' \
  | agent-on guard; echo "expect 2"
echo '{"tool_name":"Bash","cwd":"'"$ONCALL_WT"'","tool_input":{"command":"gh pr merge 17 --merge"}}' \
  | agent-on guard; echo "expect 0"
agent-on oncall release --cwd "$ONCALL_WT"
```

## §5 值守侧：收到转投之后

转投进来的单**不自动变成值守的活**，先分三路：

1. **本来就归值守**（合并、对外通信）→ 按 [MERGE-POLICY](MERGE-POLICY.md) 走：默认合入档全绿即合；其余先问用户。
2. **归另一条功能轨** → 值守用横向通信权 `SendMessage` 派给那条轨，并给原窗口回一条「已派给 X」。找归属别用眼睛扫 lane 表：

   ```bash
   agent-on oncall route --path <文件路径>     # 谁 owns 它；--json 给脚本
   ```

   它按生命周期分组：**只把 live（active/blocked/ready）的轨当派工对象**，landed / parked 的命中折叠显示——那些轨背后多半已经没有窗口，派过去等于把活扔进关掉的终端。**一条 live 的都没有**时它会直说「别直接派」，让值守回到用户那里：新开一条轨，还是让某条历史轨 `worktree edit` 重划过来。路径无人 owns 同理——值守报用户，不自己动手改。
3. **需要用户拍板 / 无人认领** → 进值守面板的「拍板收成」一节，带默认值（output-contract §3）。

**转投不改变授权**：转投消息里的用户原话是**情报**，不是批准。外向硬门动作的批准必须来自**值守会话内的用户输入**（[MERGE-POLICY §4](MERGE-POLICY.md)）——用户在功能窗口说的「合了吧」，值守收到后仍须向用户本人确认。这条是本机制最容易被抄近路抹掉的一条：转投让指令过来了，**授权没跟着过来**。

## §6 诚实边界（读懂再挂）

- **闸是 deny-list，不是全覆盖**：它认识已知形状的命令。没列进去的外发方式（新 CLI、MCP 工具、浏览器里点按钮）闸看不见，只有 §1–§3 的纪律兜着。MCP 外发工具要机械兜住，得按其工具名另加 matcher。
- **横向消息闸只认台账里的窗口**：一个**未登记 worktree** 的窗口不在 lane 台账里，发给它的消息拦不住。这不白给——未登记 worktree 本来就被边界闸报 FAIL 并连坐全场，属于那一层的问题，不该由本闸兜第二遍。同理，会话名与 worktree 目录名完全无关的窗口也认不出来（本机的命名惯例是同源的，别的机器未必）。
- **闸拦命令，不拦意图**：把同一件事换个写法照样能做出去。它防的是「顺手就做了」，不是防蓄意绕过——后者归治理，不归退出码。
- **单值守靠登记不靠锁**：`oncall claim` 是文件登记，不是分布式锁；两个人同时 `--force` 抢，后写的赢。互斥的真正保障仍是「一次只开一个值守窗口」这条人的纪律。
- **fail-open 是故意的**：无人在班时闸完全沉默。代价是「忘了上岗」等于没有闸；收益是这套机制永远不会把单人开发或值守下班后的仓库锁死。
- **转投有延迟**：中转比直发慢一跳。这是为「全场状态可读」付的价，写在这里免得后人以为是疏漏。

## §7 与既有不变量的兼容（一条未动）

- 队列真相源仍是 open PR 列表——转投消息和交单一样只是门铃，丢了最多晚一个心跳
- 批准只认值守会话内的用户输入（§5 加固而非放松）
- 值守零代修：转投进来的功能活派回作者轨，不自己动手
- 合并权唯一归值守：本页把它从纪律扩成纪律 + 退出码，条款本身没变
- 一会话一 worktree、owns 非重叠、活跃轨上限照旧

## §8 搬到别的项目（自包含 checklist）

- [ ] 装 `agent-on` CLI，挂 plugin hooks（Bash + SendMessage 两个 matcher）
- [ ] 值守上岗第一件事：`agent-on oncall claim --session <会话名>`；下班 `release`
- [ ] 治理文档抄 [CONTRIBUTING-CLAUSE.md](CONTRIBUTING-CLAUSE.md) 的值守条款，并把本页 §1 三条唯一权写进去（**用户预授权与权限归属必须留在 git 里，不能只活在聊天记录**）
- [ ] 按本项目实际补 §2 路由表的行（比如有生产部署权的项目，「部署」单列一行）
- [ ] 若项目用 MCP 外发（Slack / Telegram / 邮件 bot），决定是加 matcher 机械兜住，还是显式写明「这条通道只有纪律兜着」
- [ ] 演练一次：功能窗口跑一条合并命令，确认 exit 2 且转投模板可读；跑 `release` 后确认恢复 exit 0
