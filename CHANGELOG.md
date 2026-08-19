# CHANGELOG — agent-on 发布账本

> 职责边界:人读的版本账本;版本真相 = git annotated tag(不设 VERSION 文件)。semver 判据:**major = 不动手会坏 / minor = 不动手不坏 / patch = 不用知道**;major 条目必附迁移注记,否则不许打 tag。L3 规则改动必须成对列出 playbook + kit 双落点。

## [未发布]（自 v0.18.0 起攒）

> 语义预判 **minor**（版本号与档位归用户拍板）：不动手不坏——两条都不改任何既有文件的语义骨架，照抄旧模板的项目继续跑不会坏。但第二条**放松**了一条硬约束（push 自己的分支与开 PR 不再算外向硬门），下游项目照抄 AGENTS 模板时 agent 行为会变，所以不是 patch。原第一条（#20 禁止裸 §编号）单独看仍是 patch。

- **`kit/output-contract.md` 新增「引用文档一律说人话（禁止裸 §编号）」（#20）**——契约原本只禁机器**类别名**（`NOW` / `STALE` / 「悬点」），漏了同族的另一半：**文档小节编号**。于是会话守着规矩照样能对用户说「按 §四，AGENTS.md §13 电网表加一行」，把「先去开那份文件」的成本原样转给用户。本次补硬规矩：给用户看的正文**不许出现单独的 `§N`**，引用必须写成「哪份文件里管什么的那一节」，编号只准跟在人话后面当索引，或藏进可点链接（`文件:行号` 形式的 markdown 链接在会话窗口可直接跳转）。配四行 ❌/✅ 对照表与整句改写示范。**唯二例外**：①文档内部自引用 ②会话之间的机器消息（转投模板 / 交单 / 派工词，收件人是 AI）——分界线只有一条：**这句话会不会出现在用户眼前**。自查清单加一条。零下游改动（七处引用本页，不各自抄）。
- 源流：2026-08-19 用户实测原话「什么 4、13，这都是什么？我都不知道」。**值守本班的每一份报告都在犯这条**（裸写 `§2.6` / `§3` / `§7`），本条合入后即时改口。
- 证据：`git diff --numstat` → `41 0 kit/output-contract.md`，删除列为 0；合入 `2026-08-19T12:38:59Z`，merge `2c9b101`
- **外向硬门重划边界：push 自己的分支与开 PR 不在内（#PENDING）**——起因是一条功能会话把活干完、commit 完，停在「要我推上去开 PR 吗？推 GitHub 是外向操作，等你点头」。查证：那句话**既不出自本仓，也不出自项目宪法**（Dartify `AGENTS.md` 全文 175 行零次出现「外向」），是宿主 Claude Code 每个会话自带的通用安全句被当成了项目制度；而该项目宪法的 worktree 生命周期一节恰恰**要求**本地独有工作 48h 内上远端。**本仓自己也给了误读的抓手**：`kit/AGENTS-lite.md` / `kit/AGENTS-skeleton.md` 字面把 `push` 列进「首次须用户确认」，而 `kit/babysit/ROUTING.md` 路由表把「开 PR」划给功能会话自己做、`SETUP.md` 实测放行清单写着「普通 `git push` 一般无需放行」——同一套方法论两处口径拉反方向。本次按**一条判据**统一：改的是**本轨内部状态**（提交 / 推自己的分支 / 开 PR / 跑测试 → 自己做不问）还是**全场共享状态 / 外部世界**（merge · tag · release · 直推受保护分支 · force-push · 删远端分支 · 关别人的 PR · 对外发言 · 部署 · 建资源 · 改共享云配置 · 数据库迁移 · 花钱 → 用户点头）。配**授权幂等**：同一项目内同类动作一次点头长期有效，不逐次追问（逐次问 = 噪音淹掉真正该问的那次），跨到没点过头的新类别才重新问。并写进一条会话自查：**答不出文件名的规矩不是制度**，宿主通用句粒度必然比项目粗，项目宪法更精确时以项目宪法为准。
- 双落点（L3）：**playbook** = `playbook/multi-contributor-protocol.md` 新增「外向硬门的边界（push 与开 PR 不在内）」一节（判据表 + 三条配套）；**kit** = `kit/AGENTS-skeleton.md` 硬约束表「外向操作」行、`kit/AGENTS-lite.md` 铁律 3 与 §0 口径、`kit/worktree-gc-pattern.md` 孤本抢救第 1 步（删掉「获得外向操作授权后」——孤本抢救要等授权，正好把这条机制的目的抵消掉）。口径面同步：`README.md` 三条底线、`boot/adopt.md` 三不变量、`AGENTS.md`「不做的事」（原句「不 push / 不建远程仓 / 不动前身仓」与自举纪律 6「交付轮次必须 push + 打 tag」直接打架，本次改正：不建远程仓与不动前身仓保留为硬门，push 自己的分支与开 PR 摘出）。
- 入册：`bench/cases/39-harness-boilerplate-as-project-rule.md`（L2：宿主通用安全句被当成项目制度）。决策快照：`snapshot/2026-08-19-outward-gate-boundary.md`。
- 诚实边界：**本条没有机械闸**，靠纪律与措辞；宿主通用句仍会每个会话塞进来。真要机械化得由宿主配置或项目 hook 承载，不在本仓范围。

## v0.18.0（2026-08-19）——跨窗口指令路由：三权唯一 + 误投转投 + 机械闸

> **minor**：不动手不坏——不跑 `agent-on oncall claim` 则路由闸整条 fail-open，所有既有行为一字未变；跑了才多一层退出码。无 breaking，不需要迁移注记。既有六段输出契约、合入授权两张清单、四条值守不变量均未改。但新增了三条唯一权的硬要求（合并 / 对外通信 / 跨窗口中转），下游项目照抄治理条款时行为会变，所以不是 patch。

### 用户可见主线

- **`kit/babysit/ROUTING.md`（新）——「谁执行」的唯一真相**：①**三权唯一**（此前只有合并权唯一）——合并权 / **对外通信权**（PR·Issue 评论、Teams/Slack/邮件/webhook）/ **跨窗口中转权**（窗口之间传话经值守）在值守在班期间统一归值守，功能窗口唯一出站通道 = 给值守交单·回执；②**路由表**（八类指令 → 归谁 → 收到的窗口怎么办），含反向误投（功能活派到值守 → 转回作者轨，值守零代修不变）；③**转投四步**（不执行 → 判归属 → 【转投】模板 SendMessage → 给用户一行「已转投、球在值守那」，格式与 output-contract 面板四字段同构）；④边界情形一条判据：改的是**本轨内部状态**还是**全场共享状态**；⑤拿不准 = 当作值守的（fail-closed 分诊）。
- **`agent-on oncall` 五命令（新，`cli/src/oncall.rs`）**：`claim / status / whoami / route / release`。在班登记落 **common git dir 的 `agent-on/oncall.json`**，与 lane 台账同处，因此**每棵 worktree 读到同一份**——`docs/babysit.md` 的「在班值守地址」行是每树一份的文件副本，功能窗口在自己分支上读到的可能是任意旧版本，机器寻址从此以登记为准（人读的交接快照照旧写）。`status --json` 给值守/脚本，`whoami` 回答「本窗口是不是值守」。
- **`oncall route --path <文件>`——转投的第二跳**：功能窗口只需知道值守地址，「这文件归哪条轨」是值守的活（ROUTING §5）。本命令把它从肉眼扫 lane 表变成一条命令，并**按生命周期分组**：只把 live（active/blocked/ready）轨当派工对象，landed/parked 的命中折叠显示——那些轨背后多半已无窗口，派过去等于把活扔进关掉的终端；一条 live 都没有时直说「别直接派」，让值守回到用户那里。本仓实测：`docs/babysit.md` 命中三条轨、`cli/src/landing.rs` 命中三条，**全场无一条 live**（正是下面「顺手发现」那个死角的直接后果）。
- **PreToolUse 路由闸（`cli/src/guard.rs` + `hooks/hooks.json`）**：`Bash` 与新增的 `SendMessage` 两个 matcher 共用同一个 guard。三态——**无人在班 fail-open** / 值守窗口放行 / 功能窗口 `exit 2` 且 stderr 给出：类别 + 在班地址 + 填空版【转投】模板 + 两个逃生门（`oncall release --force` 让值守下班、`oncall claim --force` 本窗口接班，**都改在班登记因而留痕**）。拦的形状：`gh pr merge`·`gh api -X PUT …/pulls/…`·push tag / push main·`gh pr close`·`gh release create`·`gh pr comment`·`gh issue create`·chat webhook（Slack/Teams/Discord/Telegram/Google Chat）·`sendmail` 等；放行的形状：`gh pr create`·一切 `gh` 只读·功能分支 push·交单/回执 SendMessage。
- **SendMessage 闸判据收窄（#17 合入后由值守 review 发现，同批修正）**：原逻辑是「功能窗口发给任何非值守地址一律 block」，把**会话内部通信**（lead ↔ 自己的子代理、background 子代理回 `main`）一并扫了进去——三权管的是**窗口之间**，从来不管一条会话内部怎么传话。判据因此反向：从「拦一切非值守地址」改成**「拦已知是别的窗口的地址」**，依据取自 lane 台账——窗口会话名由其 worktree 目录派生（`worktree-output-clarity-e02325` → `…-02`），前缀匹配到**别的** lane 的 worktree 目录名才算真窗口；`main`、`researcher` 等匹配不到的一律放行。修正不是加豁免名单（名单永远列不全），误伤面从「所有内部通信」缩到零，横向串联照样拦得住。
- **治理与自举同步**：`AGENTS.md` 新增自举纪律 9（本仓自己守）· `kit/babysit/CONTRIBUTING-CLAUSE.md` 第 1 条扩成三权唯一、新增第 7 条转投条款 · `BABYSIT-TEMPLATE.md` §1 上岗加登记 / §3 补三权 / §7 下班四件加 `release` · `SETUP.md` 第 3 步加登记、角色表补通信权、换班补残留清理 · `babysit/README.md` 加第 8 条设计不变量 · `hooks/README.md` 与 `kit/guard/README.md` 记两个 matcher 与三态实测命令。
- **`snapshot/2026-08-19-cross-window-command-routing.md`（新）**：六个设计选择各附被否掉的替代方案（身份键选 worktree 不选 session id、登记存 common git dir 不存值守文档、故意 fail-open、deny-list 不 allow-list、横向一律中转、逃生门必须留痕）。

### 诚实边界

- **闸拦命令，不拦意图**——换个写法照样做得出去；它防「顺手就做了」，不防蓄意绕过（后者归治理，不归退出码）。
- **横向消息闸只认台账里的窗口**：未登记 worktree 的窗口不在 lane 台账里，发给它的消息拦不住——但那种树本来就被边界闸报 FAIL 并连坐全场，属那一层的问题，不该由本闸兜第二遍。会话名与 worktree 目录名无关的机器同理认不出（本机命名惯例是同源的，别的机器未必）。
- **MCP 外发不在闸内**：本轮只挂 `Bash` 与 `SendMessage` 两个 matcher，Telegram/Slack 等 MCP 工具要机械兜住须按其工具名另加 matcher，否则那条通道只有纪律层。
- **单值守仍不靠锁**：`oncall claim` 是文件登记，两个窗口同时 `--force` 抢后写的赢；`babysit/README.md`「不靠锁机制」那句仍成立，新增的是机器可寻址与留痕。
- **fail-open 是故意的反常**：本仓一贯 fail-closed，这里反过来——忘了上岗只是没有闸，fail-closed 却会锁死单人开发与值守下班后的仓库。
- **本轮没动三处**：`docs/babysit.md`（值守自己的 owns，在班期间归它接）· `kit/output-contract.md`（转投回执格式写在 ROUTING §3，不另开第二份模板）· 机器上的 `agent-on` 装机版本（合入后再 `cargo install`，避免装机版领先 main——正是 v0.17.0 调研记过的「版本号相同功能不同」的坑）。三件都在交单里点名。
- **转投送指令不送授权**：转投消息里的用户原话是情报，外向硬门动作仍须用户本人在值守会话里拍板（MERGE-POLICY §4 未放松）。
- **SendMessage 闸的首版是错的，同批修掉（#18）**：#17 的判据是「拦一切非值守地址」，把会话内部通信（lead ↔ 自己的子代理、background 子代理回 `main`）一并扫进去了——三权管的是窗口之间，从不管一条会话内部怎么传话。值守 review 时读代码发现，作者当轮改成「拦已知是别的窗口的地址」（依据 lane 台账的 worktree 目录名前缀匹配）。**首版从未生效**：发现时装机版还是没有这段代码的旧二进制。原测试没抓到是因为用了编造地址 `some-other-window-7f`——不匹配任何真实 lane，在旧逻辑下「碰巧」是拦的；测试已改为先 claim 一条真 peer 轨再发给它的目录名。教训：**闸的测试必须用台账里真实存在的地址**。
- **本节由值守轨代记**：本仓当时无任何活跃作者会话（#17/#18 的作者与上一条封版轨的会话都已消失），用户拍板由值守临时把 `AGENTS.md` / `README.md` 并进值守轨 owns 一轮做完封版四件。这是权宜，不是常态——常态仍是封版轨自己写。

### 证据

- `cargo test`：**172 passed / 0 failed**（12 条 oncall 单元测试 + 7 条 `cli/tests/oncall_routing.rs` 端到端，端到端走真实二进制与真实 PreToolUse stdin 契约）；`cargo clippy --all-targets` 零 warning
- 临时仓九项实测（未触碰本仓在班登记，真值守当时在班）：无人在班→0 · 登记后功能窗口 merge→2 带模板 · 值守同命令→0 · Teams webhook/PR 评论/push tag→2 · `gh pr create`/只读→0 · SendMessage 给值守→0（前缀匹配）/横向→2 · `release` 后→0。逐项输出见 snapshot §3
- 封版时坐标（值守实跑核对）：`origin/main = d4e9536`、上一个 tag `v0.17.0 → f2b2e90`、`v0.17.0..origin/main` = 7 笔（5 笔内容：`3b2a2ea` / `e1d1144` / `84960d0` / `8e514ff` / `5811d1e`，加 2 个 merge：`059d70e` / `d4e9536`）、`gh pr list --state open` 空
- **本版的机制已在值守窗口实跑**：`cargo install` 自值守树（`d4e9536`）装机后 `agent-on oncall --help` 列出五命令；`oncall claim --session worktree-output-clarity-e02325` 登记成功，`oncall whoami` → 「本窗口是值守」。装机前 `oncall status` 为「无人在班」，闸全程 fail-open——**本版内闸从未拦过任何在跑窗口**
- 顺手发现不代修（snapshot §5）：landed 轨的 worktree 换题目复用时，`set-status active` 报 `invalid lane transition`、`forget` 拒绝（worktree 仍在）、lane id 不能改名——三条路全堵，只能 `edit` 改 goal/owns 而 status 卡在 landed；副作用是 landed 不算 live，`owns` 重叠闸对该轨不设防

## v0.17.0（2026-08-19）——跨窗口值守调研 + 输出契约四处增补

> **minor**：不动手不坏——`kit/output-contract.md` 是**纯新增子节**，既有六段的顺序与语义一字未改，不接入就完全无感。但它加了三条新硬要求（默认值 = 建议值 / Summary 单尾 / 跨窗口编号），下游项目照抄契约时行为会变，所以不是 patch。无 breaking，不需要迁移注记。CLI / playbook / bench / boot / babysit 组件零改动。

### 用户可见主线

- **`kit/output-contract.md` 四处新增（#15）**：
  1. **「表格是允许的渲染形式，不是第二份模板」**——段的顺序与语义是硬的、排版不是；给出**唯一一张**允许的映射表（「需要我拍板的」→ §3 且默认值列不许省 / 「你做的事情」→ §4 已验证格且第三列必须是证据指针 / 「交接给值守」→ 非值守窗口专用，插在 §4 与 §5 之间）。明确点名非值守窗口最容易漏掉撤销面与球在谁那。
  2. **末尾 Summary 块**（长轮次可选，**五行固定顺序**：已完成 / 待拍板 / 交接值守 / 下一步建议 / 球在谁那）+ 两条硬要求：Summary 不许引入新信息（正文找不到的东西 = 违约）、**「球在谁那」并进 Summary 末行不另起——两个尾巴 = 违约**。短轮次不许加。
  3. **跨窗口引用编号 = `<会话名>#<任务 id>`**——不发明新编号，直接拼 Claude Code 两层现成命名空间（会话名来自 `~/.claude/sessions/<pid>.json`，任务 id 来自 `~/.claude/tasks/<sessionId>/<n>.json`），因此**天然可寻址**：会话名同时是 SendMessage 收件地址与 prompt 里 `@` 点名的 typeahead 键。前置纪律：功能会话必须把工作拆进 todo，不拆就没有 `#N`。
  4. **「默认值默认等于建议值」**——第 4 条的默认值默认就写第 3 条的建议值；只有**不可逆**（删除 / force-push / 外向发布 / 花钱 / 动别人的东西）与**超出已授权范围**两种情况允许降级，且必须括号写明降级理由。配套：§6 收尾合并规则一行、§8 自查清单加一行。
- **`snapshot/2026-08-19-babysit-cross-window-research.md`（新，#15）**——跨窗口值守调研快照：一句话结论「**缺口不在设计，在强制点与状态可读性**」；实测推翻需求方四条前提（契约模板已是需求超集、缺的是机械强制点；interactive 会话没有 `status`/`state` 字段，状态只存在于 background 会话；真相之页是按需只读快照不驻后台；隔离已强、缺的是调度）；官方四种并行模式对照与两条与本仓控制面直接冲突的实况；社区 babysit-pr 生态对照与复用判定（不同物种）；四件改进按投入产出排序；终极目标的诚实天花板。

### 诚实边界

- **「默认值 = 建议值」是用户实测催生的**，原话「为什么我不回按照不是建议的来，看起来很奇怪」。此前值守每轮写「建议 A / 不回按 B 走」，系统性让沉默等于较差结果，正好抵消拍板前移的收益。
- **本版只落了调研四件里的一件**：`<会话名>#<任务 id>` 编号约定进了契约；Stop hook + 退出码 2 的机械强制、merge conflict 的事前/事中两层、执行轨 background 化（唯一有真实代价的一条）**都没做**。
- **待同步项未随本版落地**：`kit/babysit/BABYSIT-TEMPLATE.md` §6 与 `docs/babysit.md` §6 都引用 output-contract，#15 未动它们（不在其轨 `owns` 内），归后续消化会话按 #15 的 §9 索引对表。本版不假装它已收口。
- **调研快照是快照，不是机制**：机制真相仍在 `docs/babysit.md`（循环体）/ `kit/babysit/MERGE-POLICY.md`（授权与时延）/ `kit/output-contract.md`（一轮怎么说话）/ 两份控制面。
- 本节由 v0.17.0 封版轨代记（tag 债务补记的常态：功能笔已合入，封版笔另起）；条目照 `v0.16.1..origin/main` 的实际 diff 写。

### 证据

- 版内 diff：`kit/output-contract.md` +58 / `snapshot/2026-08-19-babysit-cross-window-research.md` +205，两文件共 **+263 −0**，无删除、无既有行改写
- 合入：PR #15「docs(babysit): 跨窗口值守调研快照 + output-contract 三处增补」`2026-08-19T07:38:05Z`，merge `12b2e35`；`v0.16.1..origin/main` = 5 笔（4 功能笔 + 1 merge）
- 封版时坐标：`origin/main = 12b2e35`、最新 tag `v0.16.1 → 06106d9`、`gh pr list --state open` 空

## v0.16.1（2026-08-19）——值守文档接契约 + 推荐 pin 补更（tag 债务补记）

> **patch**：不用知道——只动 agent-on **本仓自己**的值守实例化文件 `docs/babysit.md` 与三处推荐 pin 文案，下游项目不接入则完全无感；kit / playbook / bench / CLI 零改动。本节是**补记**：#13 已于 2026-08-19 02:30 合入 main，按自举纪律 6「push 结束 tag 必须钉 HEAD」补打 tag。

### 用户可见主线

- **本仓值守文档升到新契约（#13，`docs/babysit.md`）**——该文件实例化自 v0.15.0 模板，早于 `kit/babysit/MERGE-POLICY.md` 与 `kit/output-contract.md` 落地，照原样跑会「每条 canonical PR 都来问一次」，「少让我拍板」那半个目标不生效。本次补齐五处：§2.1 **门铃即起跑**（交单消息送达即唤醒，当轮就跑收单 + 追平，不等定时唤醒；门铃丢了最多晚一个心跳，队列仍从 `gh pr list` 完整重建）· §2.2 内容分类**按实际 diff 判不按标题判** · §2.6 **「值守加速」一个口令**切 3–5 分钟 + 连续 3 轮 noop 自动回落，§2.7 **时延目标**默认合入档中位 ≤ 5 分钟（本仓无 CI，`X = CI 中位 + 5 分钟` 的 CI 项为 0），单条超时记 §5、班次中位超时算事故进 §7 交接 · §3 授权分级改成**两张显式清单**（默认合入档 5 类 + 四条前置条件 + 必须先问档全列，并写死「用户没明确授权过则默认合入档不生效」） · §6 汇报纪律统一走 `kit/output-contract.md` + 值守特有四条。
- **推荐 pin 补更（本批）**——`AGENTS.md` 从 `v0.15.0`、`README.md` 头部与速览表从 `v0.12.1` 一并抬到 **`v0.16.1`**；README 路线图补 v0.16 一行。此前三处各自停在不同版本，是 v0.13–v0.16 四次发版累积的文案欠账，本批一次清掉。

### 诚实边界

- 本版**没有新能力**：#13 只改本仓自己的值守手册（实例化文件，非 kit canonical），pin 是文案。要看 v0.16 的实际交付看 v0.16.0 节。
- **值守实跑合并首次发生在本版**：#12（merge `1459c39`，`2026-08-19T02:30:02Z`）与 #13（merge `813c87a`，`02:30:46Z`）均由值守会话执行 `gh pr merge`，用户只在值守会话内各拍一次板；`v0.16.0` tag 亦由值守代打。v0.16.0 节「本版内值守实跑合并零次」只对 v0.16.0 的版内容成立，**到本版为止已被推翻**。（两笔 merge 的 GitHub actor 都是仓库账号本身，API 分不出「人手合」与「值守合」；此处以值守当班记录 + 两笔相隔 44 秒的串行特征为准。）
- 本节由 v0.16.1 封版轨代记（tag 债务补记的常态：功能笔已合入，封版笔另起）；条目照 `b74fd64` 的实际 diff 写。

### 证据

- 债务坐标：`git log --oneline v0.16.0..origin/main` → 2 笔（`b74fd64` 功能笔 + `813c87a` merge #13），tag 前非空即债务；#13 合入时刻 `2026-08-19T02:30:46Z`，文件清单 `docs/babysit.md` 一份（+33 −5）
- 封版时坐标：`origin/main = 813c87a`、`gh pr list --state open` 空

## v0.16.0（2026-08-19）——契约层收口 + CLI 两件 + 真相之页开发史

> 六笔已合入 main：#9 / #10 / #8（kit 契约与文档）+ #7 / #6（CLI 两件）+ #11（真相之页开发史）。**minor**：不动手不坏——kit 纯增补 + 既有模板改为引用；CLI 新增一个子命令与一处静默缺陷修复,既有命令行为不变；dashboard 模板新增一个 tab,既有五 tab 不变。

### 每轮输出契约 + 值守合入授权（#9,kit)

- **`kit/output-contract.md`（新）**——每轮输出契约的**唯一真相**,所有会话与子代理同读一份。硬顺序:状态面板 → 拍板 → 结论三格 → 撤销两栏 → 球在谁那 → **之后**才是过程叙述。面板固定四字段 `轨名 │ 一句话状态 │ 我要不要动 │ 下一动作归谁`,类别一律中文人话(「可以合了」「等 CI」「别删,有孤本」),`NOW`/`STALE`/`REAPABLE` 这类机器名**只准放括号里当索引**。拍板收成一节:编号、每条 ≤3 行、**必带「你不回我就按 X 走」**、写清不拍卡住谁;**一轮最多 3 条**,超出自己排序只问最阻塞的,其余写默认值先走——不把裁决成本整体转给用户。
- **「契约悬点」正式改名**为「我按这个假设做了,你不否就当成立」,每条必写**否掉要重做什么**;机制不变(假设必须显式交出来),只换成用户读得懂的说法。结论分三格(已验证附证据指针 / 未验证假设 / 已推翻),同轮自我反复不进正文,只留最终结论 + 一行「此前 X 的说法已作废」。撤销面固定两栏(可以删附一条待执行命令 / 不能删附原因与抢救动作),**`unknown` 一律进「不能删」**。具名角色表(值守 / 作者会话 / 控制轨 / 用户),禁止无主语的「应该有人去…」。
- **`kit/babysit/MERGE-POLICY.md`（新）**——合入授权与时延的**唯一真相**:**门铃即起跑**(交单消息送达即唤醒,当轮就跑追平 + 挂 CI 链,不等定时 tick;队列真相源仍是 open PR 列表,门铃丢了最多晚一个心跳、不漏单);**「值守加速」一个口令**切 3–5 分钟、连续 3 轮 noop 自动回落,用户不改任何配置文件;**默认合入档 5 类**与**必须先问档**两张清单都显式写死(按**实际 diff** 判不按标题判,清单外 fail-closed);**时延目标 `X = CI 中位时长 + 5 分钟`**(无 CI 仓 5 分钟),逐项给依据,单条超时记遗留、班次中位超时**算事故进下班交接**;打回作者时必须同时写「这单已不占你注意力」。
- **八处接线到同一份契约,不各自展开**:`track-prompt-template`(完成报告段 + 铁律 6 + 旋钮表)/ `explore-prompt-template`(完成报告段)/ `review-prompt-template`(裁决输出改三格)/ `AGENTS-skeleton` §10.5「报告即数据」/ `landing-control-plane`(新节「给人看的时候换中文人话」)/ `babysit/README` / `babysit/BABYSIT-TEMPLATE`(§2.1 门铃、§2.6 口令、§3 授权分级改引用、§6 汇报纪律改走契约、§4 最后一处「悬点」改名)/ `CONTRIBUTING-CLAUSE` 第 5 条②(授权分级改成两张清单显式写死 + 时延目标 + 未授权则不生效)。
- 两份新文件各带**「搬到别的项目」自包含接入清单**——不依赖 kit 索引与模板接线,没装 agent-on CLI 也能用。

### CLI 两件（#7 / #6）

- **`agent-on worktree edit`（新,#7）**——lane 就地重划 goal / owns / branch / base,免得每次重划都去手改 `.git/agent-on/lanes/*.json`;owns 仍过活跃轨重叠闸。(`cli/src/{main,worktree}.rs` + `cli/tests/worktree_edit.rs`,+458 行)
- **`worktree claim --owns` 逗号列表修复（#6）**——此前传逗号分隔列表会被**整串存成单条边界**(静默缺陷:边界看似登记成功,实际一条也没生效);入口自动分割 + 回归测试。(`cli/src/main.rs` + `cli/tests/worktree_hooks.rs`)

### kit 文档收口（#10 / #8）

- **kit 索引两行（#10）**——`kit/README.md` 补 `output-contract.md` 与 `babysit/MERGE-POLICY.md` 两行索引,外加「四条不许省的纪律」后的第五条汇报纪律(每轮输出走 output-contract)。索引行与文件必须同批落地,此前被 #5 占用 CHANGELOG 故延到本批;#10 同时写下本节的初版(自 v0.15.0 起攒的三笔)。
- **重划节改指 `worktree edit`（#8）**——`kit/worktree-control-plane.md`「重划与死锁三解」的重划入口由「JSON 直改」改为 `agent-on worktree edit`,JSON 直改降级为 fallback(无 CLI 或被活跃轨重叠闸拦住时);死锁第 2 条注明 `edit` 与 claim 同闸、该条仍只能 JSON 直改;「已知雷」的逗号串改成「0.12.x 装机版」历史注记 + 指向 #6/#7 的修复与 `edit --owns` 改错路径。文档随 CLI 能力同批更新,不留悬空说明。

### 真相之页「开发史」（#11,kit）

- **`kit/dashboard-template.html` 新增第六 tab「开发史」**——日历热力图(每格一天,颜色=当日主力平台,深浅=提交量,点格子锚点跳当天明细)+ 倒序逐日时间线(类型徽章 功能/DEBUG/拍板/文档/收件 + 平台徽章 Claude Code/Codex/Grok/未标注)。与「里程碑」分工明确:里程碑人挑大事记,开发史机器转录全量流水。
- 真相源 = `git log` 机械转录,模板内附重绘命令与转录规则(类型读 commit 前缀,平台读 `Co-Authored-By` 署名,读不到标「未标注」);顶部数据源清单与页脚补「开发史 ← git log」行;配色全取既有变量;`instantiated-from` 版本戳由硬编码 `v0.4` 改为占位符。
- **这是 PR #1 的救回**:原分支从 v0.12.0 时代长出(落后 35),其 CHANGELOG 条目写进了此后已随四次发版定稿的 [未发布] 段而逐字冲突死锁。#11 把唯一的功能文件原样落到 fresh `origin/main` 上(逐字取自原提交 `5720a40`,无二次编辑),不 force-push、不改写原分支历史;PR #1 已于 `2026-08-19T02:01:04Z` 关闭。

### 诚实边界

- **默认合入档不是无人自动合并**:它是用户显式预授权的清单,拍板前移了一次,仍然是人拍的;**用户没明确授权过则不生效**,全部按「必须先问」办。
- #9 只改契约与调度参数(模板 / 治理条款 / 派工词 / 汇报纪律),**kit 侧 CLI 零改动**;把面板渲染做进 `landing status --human` 是未挂的后续,需单独拍板。
- `.claude/settings.local.json`(SETUP §1 允许集)在 #9 写下本节时尚未建,故本批六条 PR **全部由用户手合**,值守未参与任何一次合并;该文件已于封版前建好(两条 allow 规则与 SETUP §1 逐字一致),值守首班同日上岗,但「值守实跑合并」在本版内仍是零次——下一版才有实测数据。
- 本节 CLI 两件由 #9 作者代记以便封版(封版必须描述版内全部改动),条目照两笔 commit 的实际 diff 写;**各作者封版前核对自己那笔**。

### 证据

- 合入时刻:#9 `2026-08-19T01:26:54Z`、#7 `01:27:07Z`、#6 `01:27:22Z`、#11 `01:59:58Z`、#10 `02:00:10Z`、#8 `02:00:22Z`;每笔过 `agent-on worktree check` RESULT: PASS
- 封版时坐标(值守首班实跑核对):`origin/main = cf94ae9`、`gh pr list --state open` 空、`git log --oneline v0.15.0..origin/main | wc -l` → 15、`intake/` 41 卡全部已标去向
- 契约自查器对交付轮次输出跑 **10/10 PASS**;面板渲染器直接吃 `agent-on worktree gc --dry-run --json` + `gh pr list --json` 机械生成四字段面板,`unknown` 确实落进「不能删」栏
- 跨轨顺序留痕:`#4` lane 核远端后 ready → landed 释放 babysit 四文件 → #9 追平 origin/main 并扩 owns 后才接线;`kit/README.md` 索引两行等 `#5` 落地后同批补(索引行与文件同 PR,先加会造成悬空引用)

## v0.15.0（2026-08-17）——值守消化批 + 交单协议 + 本仓值守自举

> **minor**：不动手不坏。playbook / bench / kit 纯增补与措辞升级，新 kit 模板一份；agent-on 自身接入值守（docs/babysit.md + AGENTS 第 8 条）不影响下游项目；CLI 零改动。

### 用户可见主线

- **消化批（一卡一 commit，11 卡全收口）**：协作篇 §三½.6 值守合并调度（排队经济学 O(N²)→O(N) / 批准来源转述≠批准 / 调度员打回边界）+ §三½.1 字面匹配盲区（提及≠记账）+ §三½.5 闸三张面升**四张面**（新增出口面：报错即工单）；worktree-control-plane 新节「重划与死锁三解」（lane JSON 直改 = 重划机制、占位 park = 连坐逃生门【2026-08-17 拍板：维持连坐】、`--owns` 逗号雷 workaround）；bench 案 37（等 CI 的信号源）/ 案 38（全 job 秒死 = 账单层）+ 案 34 同步升四面；anti-hallucination #17 扩句（权限自改是硬墙，三模态全拦是防自我解锁设计）+ #19 新条（数字纪律：禁为数字编造解释 / 截断输出须确认全集 / 内容农场污染剔除）
- **kit/deep-research-prompt-template.md（新）**：调研域派工 prompt——v1 骨架 + v2 四条执行纪律（仓内审计先行 / 授权推翻前提 / 数字纪律 / 对抗自核验）+ 失效对照表；workflow-orchestration-checklist 调研派工行与 kit/README 索引接线
- **babysit 交单协议补强**（PR #4，kit/babysit 三件）：交单消息三型——【交单】外新增【撤单/HOLD】与【READY】，值守以最新一条为准、不凭旧交单行动；在班值守地址写进交接快照，交单方读文档不猜 ListAgents 名字
- **本仓值守自举**：`docs/babysit.md`（agent-on 实例化——无 CI 仓核对面三查、三面账本巡检【发版硬门 / intake 积压 / lane 卫生】、本仓四条实测分诊）+ AGENTS 自举纪律第 8 条（值守在班合并权唯一，不在班回退维护者自合 + 必 tag）；推荐 pin v0.12.1→v0.15.0（补 v0.13 / v0.14 两批漏更欠账）；README 对表（38 卡 / 调研模板 / 路线 v0.13–v0.15 行）

### 诚实边界

- 连坐策略维持、CLI 零改动；「逃生门」是文档化姿势不是新机制。CLI 两欠账（`--owns` 逗号分割、`worktree edit`）已立后台任务卡，不在本批
- 本仓无 CI workflows：docs/babysit.md 的合并核对面为 mergeable / GitGuardian / guard 三查；模板中的 CI watch 链在本仓标注不适用
- `.claude/settings.local.json`（SETUP §1 允许集）截至本批仍未建——值守首班上岗前须用户手跑；本批合并两次撞分类器即为此因（anti-hallucination #17「两步即停」实录）
- 消化预算线内全收口：承接层五份值守相关文件零 pending；更早批次此前已收口

### 证据

- 消化九笔 + 自举一笔 + #4 四文件，每笔 commit 过 pre-commit + PreToolUse 双闸（`agent-on worktree check` RESULT: PASS）
- 分诊选择题四题用户拍板：正文四处全采纳 / kit 新件全采纳 / 连坐维持+写清逃生门 / CLI 立后台任务卡
- 消化中连坐两次实测（当日累计四棵未登记树全靠占位 park 解开），姿势即本批「重划与死锁三解」第 3 条；PreToolUse 先评估整条命令（claim 与 commit 必须拆条）为当日新发现，已入该节
- 消化来源：intake `2026-08-16-dartify.md`（6 卡）/ `2026-08-17-dartify.md`（4 卡）/ `2026-08-16-dartify-worktree-guard-field-report.md` / `2026-08-17-dartify-deep-research-prompt.md`（1 卡）——原地标注全部 landed@同批；交单协议证据：Dartify PR #176 实战（HOLD/READY 临场发明 + 收件人错投靠代转补救）

## v0.14.0（2026-08-17）——值守合并调度（babysit merge dispatcher）

> **minor**：不动手不坏。纯新增 kit 组件与文档接线，存量项目不接入则行为不变；CLI 零改动。

### 用户可见主线

- **kit/babysit/ 四件**：`BABYSIT-TEMPLATE.md`（值守文档模板 §0–§7：GOAL / 首轮启动 / 每轮检查单 / 权限三档 / 分诊手册 / 已知遗留 / 汇报纪律 / 交接下班）+ `SETUP.md`（三步接入：权限用户手跑 → 治理条款 → 复制模板启动，含角色分工与换班 SOP）+ `CONTRIBUTING-CLAUSE.md`（治理条款范本，含交单模板）+ `README.md`（定位、排队经济学、五条设计不变量）
- **定位**：多会话并行下远端公共态（main / PR 队列 / CI / 账本）的值班经理。排队经济学：up-to-date 硬门下 N 会话各自追平自合 = O(N²) 次 rebase，值守串行调度 = O(N)——合并权中央化不是偏好，是硬门下的最优解
- **landing 的执行半场**：landing v1 只读出 NOW / 波次当排序输入，值守做「追平 → CI → 拍板 → merge → 记账 → 回执」串行执行；auto-merge 挂点的有人拍板实现，无人自动合并仍然不做
- **五条设计不变量**：会话是班次、文档是资产 / 队列真相源 = open PR 列表（SendMessage 交单只是门铃）/ 追平只走服务端 update-branch / 串行 + 连锁追平 / 批准只认本会话用户输入（同行转述 ≠ 批准）
- **接线**：`landing-control-plane.md` 补「执行半场」节、`worktree-control-plane.md` 补「值守与 lane 的分工」节、BOOTSTRAP §5 与 `boot/adopt.md` 增量接管段各加一句接入指引；README 补 v0.13 路线行欠账

### 诚实边界

- 不做无人自动合并：需拍板类永远等用户；单值守互斥靠治理条款与接班仪式，不靠锁（在班心跳 / 队列标签化 = v2 挂点，零真实需求前不建）
- 值守零代修：真缺陷打回作者四件套（证据指针 + 缺陷定位 + 修复选项 + SendMessage），billing 类事故推通知等管理员
- 权限最小集只两条 allow（merge + 服务端 update-branch），被拦哪条补哪条，不放 `gh:*`；agent 改不了自己的权限配置（三模态实测全拦），SETUP 把它做成用户手跑步
- 本批只消化 babysit 落点：08-16 六卡 / 08-17 四卡的 playbook、bench、anti-hallucination 落点仍 pending（各卡已原地标注）

### 证据

- Dartify 值守夜班（08-16）：单 /loop 会话 9 连合（#150–#153 / #155 / #158 / #160–#162），每条走追平→CI→合；#150 两轮追平（76ec1df→75a0ca8）实证连锁追平；途中处置 org 级 Actions billing 瘫痪 ~6.5h（job annotation 取证 + 推通知 + 每轮探针）；治理条款入 Dartify CONTRIBUTING §四（PR #163）
- Dartify 三单实战（08-17，#164 / #165 / #169）：run id 按 workflowName 过滤修正抓错（31956970664→31956970653）；#169 真红打回作者四件套后 15 分钟修绿（3e41feb），值守零代修；转述指令仍向本人复核后执行，作者回执「你另行向用户核拍板是对的——该省的从来不是这步」
- 跨 lane 追平边界实测：本地推别人分支被 worktree guard 正确拦下，`gh api -X PUT …/update-branch` 输出 "Updating pull request branch." 干净通过
- 消化来源：intake `2026-08-16-babysit-merge-dispatcher.md` 与 `2026-08-16-babysit-kit-template-draft.md`（两专题件全部落位）+ `2026-08-16-dartify.md` 六卡 / `2026-08-17-dartify.md` 四卡的 babysit 落点

## v0.13.0（2026-08-16）——Landing 控制面 v1：合流协调 + 生命周期分类

> **minor**：不动手不坏。存量项目 lane/hook/gc 行为不变；要多 PR 排队与五类生命周期汇总时升级 CLI。新增 `worktree claim --parked` 与活跃轨上限（默认 3）只影响新 claim / 激活动作，存量 lane 记录不迁移。

### 用户可见主线

- **三条只读命令**：`agent-on landing refresh|status|plan [--json] [--repo] [--quiet-hours]`；`refresh` 是唯一联网命令（`gh` 批量探针 + `git ls-remote`），`status`/`plan` 离线读快照
- **SHA 绑定证据缓存**：每条轨绑定 `(PR head SHA, base SHA)`，两者未变直接 SKIP 复用，绝不重复取证；base 移动时只重查有依赖边或文件重叠的 PR，无重叠的轨证据仍有效（键的 base 半边直接推进）
- **六类合流表**（格式固定）：NOW / NEXT / PARALLEL / FIX / STALE / SKIP，判定优先级 FIX > STALE > NOW > NEXT > PARALLEL > SKIP；NOW 每轮只选一条（合并严格串行），按下游依赖数 + PR 号排序
- **五类生命周期**：全部 worktree / 功能轨自动落进 ACTIVE / WAITING / PARKED / RESCUE / REAPABLE 恰好一类；首页只给五个数（现在做 / 下一批 / 等待中 / 需抢救 / 可回收）；REAPABLE 需要合流权威证据 + clean + 静默期，主树永不 REAPABLE
- **活跃轨上限**：`agent-on/config.json` 的 `active_cap`（默认 3）；`claim` 超限拒绝、`--parked` 排队不占额、`set-status active` 激活同样过闸
- **波次规划**：`plan` 按依赖拓扑排 WAVE 1/2/…，并行准备轨与前置修复（FIX/STALE）分列；波次只是建议，实际合流仍走控制轨合流清单 + 远端 read-back

### 诚实边界

- v1 严格只读 + 按需运行：不驻后台、不自动 merge、不自动删 worktree、不代写 lane 状态（PR 合流后只提示 `set-status landed`）
- 快照是本机缓存（common git dir `agent-on/landing/snapshot.json`），不是第二套 canonical 真相；PR 权威在托管平台，丢了重新 refresh 即可
- 「无文件重叠 ⇒ 证据仍有效」是文件粒度近似；gh 每 PR 100 个文件、compare 300 个文件的截断都如实标注并保守处理（截断 → 视为重叠）
- 离线时 base SHA 降级本地 `origin/<default>`（标注 `local`，可能过期）；gh 探针失败则 refresh 整体报错，不出半份快照

### 证据

- Rust：134 测试全过（`cargo test`：125 unit/integration + 9 hooks integration），`cargo clippy --all-targets` 0 警告；含 8 个 FakeGh + 真实 git fixture 的端到端用例（SKIP 复用、base 移动重叠→STALE 不重查、无重叠→reused-valid、PR 合流→REAPABLE、离线 status、依赖波次）
- 真机 dogfood（本仓）：首次 refresh 取证 1 条、第二次 `取证 0 条 | SKIP 复用 1 条`；同屏抓出 5 条 RESCUE（primary 未推送、两条 landed 轨未推送、合流残留、未登记脏树）
- 设计契约：`kit/landing-control-plane.md`（数据模型、缓存键、失效规则、六类/五类判定、auto-merge 挂点）；决策快照 `snapshot/2026-08-16-landing-control-plane.md`

## v0.12.1（2026-08-16）——Worktree 执行强制层

> **patch**：存量项目不安装 hooks 时行为不变；启用多写会话的项目可用一次安装把 v0.12.0 的人工检查接到真实 commit/push/PreToolUse。lane、回收判据与“永不自动删除”语义不变。

### 用户可见主线

- **一次安装，所有 worktree 生效**：新增 `agent-on worktree hooks install|status|uninstall [--repo PATH]`；仓库 common git dir 中的 shared `pre-commit/pre-push` 自动运行严格 `worktree check`，成功静默、失败给出修复命令
- **主树控制轨真闸**：仍有 `active|blocked|ready` 执行轨时，primary 的普通 commit 被拦；当 Git 实际触发 `pre-commit` 时，merge、squash-merge、cherry-pick、revert、rebase 等控制态 marker 自动放行（clean merge 见下方诚实边界）
- **不吞用户 hooks**：已有真实 hook 或 `core.hooksPath` 时 install fail-closed；status 校验内容/可执行文件/配置漂移；uninstall 只删指纹匹配的 Agent-On 资产，hook 与 scheduler 任一漂移时整组不动
- **Claude/Codex PreToolUse 接线**：两家 plugin 共用 canonical `hooks/hooks.json`；只有 `git commit/push` 才支付完整 lane/owns audit 成本，非 git 与 git 读命令立即放行。Codex manifest 正式接线，删除旧 `hooks-codex.json` 双头
- **旧 Codex 注册止血**：`agent-on-git-guard.sh` 成为 Bash/Python polyglot 兼容入口，旧 `python3 …guard.sh` 不再 SyntaxError；status 只读提示 legacy 个人 hook，不静默改用户 home
- **可选每日报告**：`agent-on worktree hooks install --daily-gc` 在 macOS 安装用户 LaunchAgent、Linux 安装 systemd user timer；每日 03:30 固定运行 `gc --dry-run --json --repo <primary>`，从 linked tree 安装也归一到稳定 primary。无 daemon、无 delete，卸载保留历史报告

### 诚实边界

- Git 原生 `--no-verify` 仍是人工逃生口；Agent 发起同一命令时，PreToolUse 在 Git 之前再拦一次
- clean `git merge --no-ff` 不触发本版两类 hook（Git 使用 `pre-merge-commit`）；控制轨仍须走合流清单，随后 push 再过严格闸。lane 的 `base` 必须用稳定 `origin/<default>`，不能用会随本地 merge 移动的 `main`
- Codex 非 managed hook 首次信任仍由宿主 `/hooks` 管理；Agent-On 不替用户点击或改写 home
- daily GC 默认不安装；即使安装也只产动态候选报告，不执行 `git worktree remove` 或删分支

### 证据

- Rust：73 unit + 9 真实 Git integration 全过；integration 覆盖 primary/未登记/越界 commit 阻断、linked lane 修复后 commit+push、squash 合流放行、bare remote 未前进、per-worktree hook override、安装/状态/卸载幂等、既有 hook 保护与双向 drift 全不动
- 调度：19 项定向测试，含 linked→primary identity、persisted exact state、PATH/可执行路径变化、仓库 move/delete、foreign/drift 安全边界与 macOS 原生 `plutil -lint`；真机 LaunchAgent kickstart 产出 dry-run JSON 后完整卸载
- 设计与完整验收矩阵：`snapshot/2026-08-16-worktree-enforcement.md`

## v0.12.0（2026-08-16）——Worktree 生命周期只读回收审计

> **minor**：不动手不坏。存量项目可继续用 `worktree status/check`；需要动态回收候选与每日盘点时升级 CLI。

### 用户可见主线

- **report-only GC**：新增 `agent-on worktree gc --dry-run [--json] [--repo PATH] [--base REF] [--quiet-hours N]`；缺 `--dry-run` 在读取 repo/PR 前拒绝，CLI 没有 apply/delete 模式
- **三判据同屏**：逐树输出目标 base landed 证据、upstream/unpushed/unique 保存证据、raw dirty；再叠加 primary/locked/prunable、lane、open PR、24h 文件+git-admin 静默与磁盘大小
- **squash-safe 且 base-safe**：MERGED PR 只有在 `baseRefName` 等于目标 base、`headRefOid` 覆盖当前 HEAD 时才能修正祖先假阴性；合后新增本地提交仍是 rescue，合进父 feature/develop 的 PR 不能冒充已进 main
- **动态 known reclaim**：JSON `candidates` 每次从 git/PR/lane 事实重算；握手、每日一次、每次合流 read-back 后盘点，禁止手填常青清单
- **边界收紧**：通用层不自动认“假脏”；locked/dirty/unknown/open PR/无 PR 孤本不进入候选。旧 `worktree status` 也补上 registered locked/prunable 保护

### L3 双落点

- **playbook**：`playbook/multi-contributor-protocol.md` 固化第二写者并发门、fresh base、三判据派生名单与 report-only 权限
- **kit**：`kit/worktree-control-plane.md` + `kit/worktree-gc-pattern.md` + AGENTS 骨架/merge checklist 落成创建、盘点、回收和权限执行面

### 证据

- Dartify 原文：`CLAUDE.md:27–103` 三判据/squash/假脏 + `AGENTS.md:124–130` lifecycle；babysit JSONL 解码内行 5–6/23/26/33–34/38–39/43；完整来源与逐字锚见 `snapshot/2026-08-16-worktree-lifecycle-audit.md`
- Agent-On dogfood：审计窗口从 3 棵变 4 棵、再回到 3 棵；一棵新树从 clean 变 dirty，另一棵 2-commit 孤本被并发会话先 push 再拆。动态候选始终 0，证明静态名单会在同一会话内过期
- 机器：`cargo test --no-fail-fast` 38/38；`cargo clippy --all-targets -- -D warnings` 通过；真实联网 dry-run `gh: ok`、`CANDIDATES (0)`、read-only 回执；新增三份 plugin manifest 与 CLI 版本一致性测试
- 独立 gstack review 抓出并已修：PR 合错 base 假阳性、`git status` optional-lock 写 index、quiet 漏看 linked git-dir 活动

## v0.11.0（2026-08-16）——第二十二次消化：闸的失效面 + 运行面验收

> **minor**：不动手不坏。存量项目继续沿用旧协议；新闸/验收/口令契约见本版 playbook/kit。

### 第二十二次消化（14 卡：13 landed / 1 rejected）
来源：`intake/2026-08-15-{dartify,inbox-radar,SalesDashboard,onboard-bot-lark}.md`

### L3 双落点
- **command-phrase**（纠偏）：`skill/SKILL.md` 末行 + README 三路 + `codex/AGENTS-global-snippet.md`——口令=读执行书照做，与斜杠结果等价、调用面不等价；禁止把 Skill 拒调读成口令失灵
- **config-surface-green-is-not-runtime**（三卡并）：anti-hallucination C 附3 + sop 集成清单 12–13 + merge 5b / phase-card「运行面」+ bench 33
- **gate-three-faces**（三卡并）：multi-contributor §三½.5 + merge 2c/7d/7g/7h + worktree-gc DIRTY 双成因 + bench 34
- **ledger-self-coverage**：§三½.1 + kit/ledger-ratchet-pattern「元动作自涵盖」+ merge 7b + bench 30 续
- **discriminating-probe**：anti-hallucination 第六型 18 + review/phase-card + bench 35

### 其余 landed
- **llm-fallback-empty**：C 附4 + phase-card「输出校验」
- **no-invented-directory**：C 附5 + AGENTS-skeleton「不发明花名册」
- **feature-delete-shared-terms**：sop「删功能先划标识边界」+ merge 2d
- **browser-entry-no-node-fs**：AGENTS-skeleton + review + bench 36（不升 playbook 长节）

### rejected
- **map-required-fields-before-header-reject**：项目域 ingest 口径

### 附记
- 收件 ec6fff3 / d46f03b / a20cc41 / 0d2a17f；bench 32→36
- 开场补推卡在本机的 `v0.10.0`/`v0.10.1`（tag 已打未 push）；两棵已知 rescue worktree 未动
- C 附4/5 正文曾与判别式探针同文件落入 66c88a7，kit 双落点仍按卡拆 commit

## v0.10.1（2026-08-16）——Worktree 入口与存量接管补齐

> **patch**：不用知道也能用；补齐 v0.10.0 控制面的发现与 adopt 路径。

- `/agent-on worktree` / `$agent-on worktree` 进入同一控制面内核；空参数先跑只读 status，有后缀才映射 CLI，禁止偷换成删除
- `boot/adopt.md` 考古阶段强制摊开既有多 worktree；dirty/unique 先标 rescue，仍写代码的旧轨增量 claim，不回填历史、不先删
- README 指令速查增加多会话控制面；推荐 pin、CLI/setup 与 plugin manifest 对齐 `v0.10.1`

## v0.10.0（2026-08-16）——多会话 Worktree 控制面

> **minor**：不动手不坏。存量项目继续沿用旧并行协议；需要多 Claude/Codex worktree 控制面时升级 CLI 并按新模式 claim。

### 用户可见主线
- **轨道合同**：一个写会话 = 一个 worktree = 一个单目标合同（goal / owns / depends_on / base / status）；衍生功能分流为新 lane，不再让长寿 worktree 无限扩 scope
- **机械边界闸**：Rust CLI 新增 `agent-on worktree claim|set-status|status|check|forget`；claim 拒绝活跃文件域重叠，check 拦未登记/实际越界/失联记录，ready 要求 clean + 依赖 landed
- **保守回收**：全场输出 `primary|safe|review|rescue|metadata`；主 worktree 永不回收，squash/无 PR/孤本拿不准只报告，CLI 不自动删目录或分支
- **会话/合流闭环**：handshake 复述 lane 合同；派工前 claim、提交前 check、远端确认后 landed，再按分类人工回收
- **交付入口对齐**：CLI/package/setup 默认 pin、Claude plugin/marketplace manifest、README/Codex 安装文案统一到 `v0.10.0`；新增测试锁住 CLI 版本与 setup 默认 pin

### L3 双落点
- **playbook**：`playbook/multi-contributor-protocol.md` 固化轨道合同、即时控制面与保守回收原则
- **kit**：新增 `kit/worktree-control-plane.md`，并对表 AGENTS 骨架 / track prompt / merge checklist / worktree GC pattern；BOOTSTRAP 第二写轨启用

### 证据
- 用户实证：长期多 Claude 对话、多 worktree 即使“一上下文一功能”仍发生 scope 衍生、合流顺序失忆与回收困难
- 本仓 dogfood：首次 `worktree status` 立即检出 2 个 clean 但含孤本的未登记 Claude worktree（behind/unique = 68/2、119/1，均 `rescue`）；未擅自改动或回收
- CLI：全量 `cargo test`（27 tests）+ help/status JSON 实跑；严格闸绿路、边界段匹配、越界、重叠、ready clean、primary 不回收均有单测；跨模块环境变量测试共用锁，消除旧 flaky
- 决策快照：`snapshot/2026-08-16-worktree-control-plane.md`

## v0.9.1（2026-08-08）——README 路线图纠偏

> **patch**。

- 修正 README 版本路线图（v0.7/0.8/0.9 分行）；推荐 pin 钉 `v0.9.1`

## v0.9.0（2026-08-08）——第二十一次消化：交付前对表 worktree

> **minor**：不动手不坏。

### 第二十一次消化（Dartify 真机交付 3 卡全 landed）
- **worktree-snapshot-shipped-as-latest**：kit/worktree-gc-pattern 交付前对表 + multi-contributor + merge §6 + bench 32
- **env-hypothesis-crowds-out-delivery-self-check**：sop 没生效先查交付链 + anti-hallucination #16
- **classifier-denies-command-not-goal**：anti-hallucination #17 + guard README 诊断 + bench 32
- 附记：收件 6bc8f9d；来源 `intake/2026-08-08-dartify.md`；bench 31→32

## v0.8.3（2026-08-06）——推荐 pin 与 HEAD tag 对齐

> **patch**。

- 推荐 pin / checkout 与 `v0.8.3` 同 commit（避免 tag-release 后 pin 文案落后一代）

## v0.8.2（2026-08-06）——docs 对齐

> **patch**。中间 tag；见 git history。

## v0.8.1（2026-08-06）——docs 对齐

> **patch**。中间 tag。

## v0.8.0（2026-08-06）——第二十次消化：记账棘轮 / worktree 回收 / 孤本

> **minor**：不动手不坏。存量 pin v0.7.x 直升；新模式见 kit 两份 pattern 文件。

### 第二十次消化（Dartify 治理机制化 4 卡全 landed）
- **ledger-ratchet-mechanism**：multi-contributor §三½.1 + kit/ledger-ratchet-pattern.md + merge-checklist 7b + bench 30
- **calendar-deadline-needs-executor**：§三½.2 + kit/worktree-gc-pattern.md + bench 31
- **guard-allowlist-author-deadlock**：§三½.3 协作者出口 + bench 30
- **orphan-work-rescue-before-reap**：§三½.4 孤本三步 + worktree-gc 无 PR 档 + bench 31
- 附记：收件 afbf19d；bench 29→31；来源 `intake/2026-08-06-dartify.md`

## v0.7.2（2026-08-03）——guard 路径边界 + setup 测试

> **patch**：不用知道也能用。

- **guard**：`inside_agent_on` 要求路径分隔符边界，拒绝 `B`/`B-evil` 前缀误匹配；单测 sibling 会话不得放行写 B
- **fail-open**：B 未登记/非法 env 时 `guard_decision` 断言 exit 0
- **setup**：`config-only` 正反例单测 + CLI `--config-only` 实跑


## v0.7.1（2026-08-03）——文案纠偏（audit-lint 引用）

> **patch**：不用知道也能用。

- 修正 merge-checklist / schemas / run-card-logging 中 `agent-on audit-lint` 引用笔误


## v0.7.0（2026-08-03）——可执行面 Rust 化（去掉 Python 交付依赖）

> **minor**：不动手不坏（方法论 markdown 不变）；装机/校验/guard/发版助手从 Python 迁到 **Rust CLI**。存量 pin v0.6.x 直升即可；要跑 doctor/setup/lint/tag 需安装 `agent-on` 二进制（`cargo install --path cli`）。无 major。

### 用户可见主线
- **单一 CLI**：`cli/` crate → `agent-on` 子命令：`doctor` · `guard` · `intake-lint` · `audit-lint` · `check routing` · `tag-release` · `setup`
- **删除主树交付用 `.py`**：原 `scripts/*.py`、`kit/guard/agent_on_paths.py`、原 python shebang 的 `agent-on-git-guard.sh`、`ledger/*-lint.py`
- **hooks**：PreToolUse 改为 `bash …/kit/guard/agent-on-git-guard`（shim → release 二进制或 PATH）
- **文档/ skill / settlement / AGENTS**：默认命令改为 `agent-on …`；装机依赖 rustup + git
- **测试**：`cargo test`（paths / guard 矩阵 / intake evidence 硬门 / audit 状态机 / tag-release 临时仓 / routing 缺文案失败）

### 迁移（给人）
1. 安装 [rustup](https://rustup.rs)
2. 在 B 仓：`cargo install --path cli --force`（或 `cargo build --release --manifest-path cli/Cargo.toml` 供 plugin hooks）
3. 原 `python3 scripts/…` 一律改 `agent-on …`

## v0.6.3（2026-08-03）——本仓对话 commit 必打 tag

> **patch**：不用知道也能用；存量 pin v0.6.2 可直升。无 major。

### 发版硬门扩围
- **协议**：agent-on **本仓直接对话**凡 `git commit` 并交付/push，收尾必封 CHANGELOG + 更新推荐 pin + **annotated tag 钉 HEAD 并 push**；禁止只 commit/push 不发版；goal 写「不要求 tag」无效
- **落点**：AGENTS 自举纪律第 6 条、`boot/settlement.md` 发版硬门、`playbook/iteration-loop.md` 消化条、`scripts/tag-release.py` 适用范围
- **背景**：用户 2026-08-03 拍板「在本库直接对话的所有 commit 都要加 tag」；此前 v0.6.1 仅绑消化收尾，轻主路径两 commit 曾只 push 导致 HEAD 领先 tag

## v0.6.2（2026-08-03）——轻主路径 + 降档协议 + Superpowers 退出默认

> **patch**：不用知道也能用；存量 pin v0.6.1 可直升。无 major。

### 用户可见主线
- **降档协议**：`boot/adopt.md` §三 — 与升档对等；禁止静默降档；须用户显式批准；只删不用的件、不重播；local_deviations 登记；BOOTSTRAP 档错指针改走 §三
- **开箱更轻 / Superpowers 退出默认推荐**：kit AGENTS-lite/skeleton、BOOTSTRAP skill 尾注、README 分工定调（制度在 agent-on；GStack 可选；实现不默认 Superpowers 引擎）
- **空转件可见降权**：jsonl L 四卡 / `kit/schemas` 开箱勿启用；`phase-gates` 每轮锁口令复述死亡名单交叉；ledger 旁路横幅
- **审视清单**：`snapshot/2026-08-02-light-hard-premium-mrd.md`（B1/C1）；`snapshot/2026-08-03-research-residual-audit.md`（deep-research 余量）
- **断言**：`scripts/check-skill-routing.py`（skill 路由 + 降档协议 + schemas/锁口令非默认）
- **附记**：本机 Claude 路由 C1 在 agent-memory 仓（`e399df5`），不在本 tag 树内

## v0.6.1（2026-08-02）——消化收尾必打 tag（发版硬门）

> **patch**：不用知道也能用；存量 pin v0.6.0 可直升。无 major。

### 发版硬门：消化收尾必打 tag
- **协议**：`boot/settlement.md` 收尾由「该打 tag 就打」改为 **必打 annotated tag 并 push**；禁止跨会话「minor 攒批」积压不发版（实证：v0.5.1 后 59 commit 导致下游「无可升级版本」）
- **助手**：`scripts/tag-release.py --level patch|minor|major --title "…" [--push]`
- **双落点**：AGENTS 消化收尾四件、iteration-loop ⑤发布站、skill 硬门半句

## v0.6.0（2026-08-02）——消化攒批：真相面 / 发布门 / Dartify 协作与安全

> **minor**：不动手不坏。存量项目 pin 从 v0.5.x **直升即可**，已实例化文件默认不动（新工件与读 B 仓正文自然用新规则）。无 major、无迁移注记。
>
> 封版依据：v0.5.1 后 59 个 commit 全在 [未发布] 攒批（第 11–19 次消化 + guard 修复等）；下游 pin 钉在 v0.5.1 无法「agent-on 升级」——发版解除空操作。

### 用户可见主线
- **真相面治理**：喂养四元组（漏喂怎么发现）、幽灵 backlog、易变事实禁 self-pin、历史文档隔离
- **发布门 / 运行态**（sop Phase 6½）：机器 preflight、skip 保持红、门包执行入口、exact-SHA 门禁、能力级窄发布、runtime clean checkout
- **并行四种事故**：互踩 / 工作重复 / 撞号 / **主题撞题（提交前再查重）**
- **Dartify 痛点**：docs-only 与生产 CI 解耦；推迟项禁「顺手」；验证全量与夹具可达；边缘 IP / 密码界 / 扫描静默降级
- **其他**：runtime≠product surface；loop cadence≠截止；竞品私有栈不阻塞选型；guard pathspec 收窄

### 消化批次（细节见下列各节原文）
第 11–19 次消化已全部 landed；bench 案例增至 29 张。


### 第十九次消化（2026-08-02，Dartify 结账 11 卡全 landed；定级 minor 攒批）
- **包A 推迟项/真相面/CI 解耦**(痛点):truth-hierarchy 喂养**四元组**+漏喂发现;TODO 禁「顺手」;sop 状态面≠发布;merge 7f docs-only 不触发生产
- **包B 并行**:第四型主题撞题(提交前再查重)+ handoff 不抢轨 + merge 7e 同名分派点死代码;bench 27 扩四型
- **包C 验证**:夹具可达性 / 合并后全量验证 / 扫描静默降级(anti-hallucination #12–13)
- **包D 安全**:边缘 IP 单源 / 密码界共享 schema / 宿主迁移 runbook / 禁 exploit 静态路径(#14–15 + sop #10–11)
- 附记：收件 ca7c1db；1 内容 commit 归并 11 卡；本批无 major
- 来源：`intake/2026-08-02-dartify.md`（pin v0.5.1；用户口述 progress/CI/Cloud 反复失效）

### 第十八次消化（2026-08-01，hk-sfc-licensees 首结 4 卡全 landed；定级 minor 攒批）
- **local-dev-constraint-is-not-product-endstate**：AGENTS-skeleton `runtime ≠ product surface` + BOOTSTRAP 需求六问 Q3
- **loop-cadence-is-not-deadline**：phase-card §2b cadence≠截止 + skill Loop 用户可见半句
- **do-not-block-on-competitor-private-stack**：anti-hallucination 第六型#11 + elicitation/BOOTSTRAP 竞品选型
- **scheduler-stop-must-match-real-product-scope**：phase §2b stop 对齐产品范围 + skill 默认不自删
- 附记：收件 981f270；2 内容 commit（卡1 / 卡2–4）；本批无 major、无 bench 新增
- 来源：`intake/2026-08-01-hk-sfc-licensees.md`（pin v0.5.1）

### 第十七次消化（2026-07-31，inbox-radar 首结 21 卡全 landed；定级 minor 攒批）
- **包1 真相面**(5 卡归并)：truth-hierarchy §五¾ + merge 7b + dashboard ⑥ + phase 收口行
- **包2 发布门**(6 卡归并)：sop Phase 6½ + phase preflight/scope + progress blockers + merge §5/7c
- **包3 远端取证**(2 卡)：sop §6½.8 + merge 7d + **bench 29** ambiguous read-back
- **包4 运行态**：sop runtime clean checkout + merge §5
- **包5 台账**：iteration-loop + run-card-logging 禁 retroactive 伪造
- **包6 边界**(4 卡)：anti-hallucination 第六型 #7–10 + review-prompt 附加检查
- **包7 stale gate**：workflow-orchestration §一.5 时序 + merge 7c
- **包8 可执行文档**：sop §6½.9 + review-prompt executable docs
- 附记：收件 0395b74；21 卡 8 语义包 / 3 内容 commit；README bench 28→29；本批无 major；pin 旧 v0.3.0 摩擦未单开 rejected（规则仍适用）
- 来源：`intake/2026-07-31-inbox-radar.md`

### 第十六次消化（2026-07-30 晚，Euan 二结 6 卡全 landed；定级 minor 攒批）
- **shared-id-namespace-collision**：multi-contributor 第三型撞号 + AGENTS 取号即落盘 + bench 27 扩（并入 two-collision）
- **rename-wave-rebase-derive-the-map**：merge-checklist §1b 改名浪潮三步（映射→残差→新增扫尾）
- **classify-red-check-origin-before-blocking**：merge-checklist §2b 红灯先分来源
- **blocked-dependency-check-actual-requirement**：sop 能力-依赖对照 + phase-card 验收旁注
- **human-console-checklist-must-be-doc-verified**：anti-hallucination 第六型#6 + sop 集成清单第 9 条
- **guard-path-regex-matches-content-not-target**：`agent-on-git-guard.sh` 收窄 pathspec 目标识别 + README（**代码修复**；回归 A/B/C/D/E）
- 附记：收件 82e0692；卡2+3 合 commit（merge-checklist 同文件）；本批无 major
- 来源：`intake/2026-07-30-euan-flutter.md` 追加批（社交登录 + 五连合并；pin v0.5.1）

### 第十五次消化（2026-07-30，Euan harness 审计结账 5 卡全 landed；定级 minor 攒批）
- **paper-mechanism-rots-silently**：multi-contributor §三½ 机制须带闸 + AGENTS-skeleton 自问行（L3）
- **truth-surface-feeding-table**：truth-hierarchy §五½ 喂养表 + dashboard-template 三元组（L3）
- **two-collision-diseases-two-cures**：multi-contributor §二.2 互踩≠撞题 + bench/cases/27（L3）
- **code-first-needs-retroactive-ledger**：AGENTS-skeleton §9 48h 事后追认 + 外围义务检查
- **audit-with-adversarial-verifiers**：review-prompt 合规审计变体 + bench/cases/28
- 附记：收件 408e9a2；一卡一 commit（94b9f14/3e0e963/0cc4eab/4a28ed2/5118235）；README bench 26→28；本批无 major
- 来源：`intake/2026-07-30-euan-flutter.md`（Euan pin v0.3.0；harness v3 重建 / PR #22）

### 第十四次消化（2026-07-29，Euan 结账 5 卡全 landed；定级 minor 攒批）
- **observe-dont-interrogate**：sop Phase 6 排障·观测优先于假设 + AGENTS-skeleton 排障纪律半句
- **ratelimit-masks-real-error**：anti-hallucination 第六型#5 + bench/cases/26（L3 双落点）
- **text-assertion-cannot-prove-structure**：anti-hallucination C 附2「锚必须能判别」+ phase-card 配置锚行（L3 双落点）
- **cwd-not-flag-decides-source**：并入第六型#4 读取位置/cwd 维 + AGENTS-skeleton/merge-checklist 不可逆动作前验证作用域
- **cdn-verify-needs-cachebuster-and-window**：sop 集成清单第 8 条 + merge-checklist §5 静态资产行
- 附记：收件 05c9c72；一卡一 commit（90cea1d/026e91c/5bed77b/ab0724f/652265e）；README bench 25→26；本批无 major
- 来源：`intake/2026-07-29-euan-flutter.md`（Euan pin v0.3.0；API 宿主迁 CF + dartify.dev 发信链路）

### 第十三次消化（2026-07-26，Euan 三结 12 卡全 landed；定级 minor 攒批）
- **demo-anchor-into-repo-before-build**：kit/phase-card-template §0 Demo 锚点四件套前置检查
- **serve-dc-html ∪ react-fiber-logic**（语义归并）：bench/cases/25 + README 索引/时机
- **disjoint-file-ownership-parallel-implementers**：workflow-orchestration §二½ + phase-card 文件域/共享 owner + checklist 行（L3 双落点）
- **chunked-readers-gap-matrix-workflow**：workflow-orchestration §三¼ + checklist 大单文件节
- **wiring-anchor-def-not-integration ∪ guardrail-entity-must-resolve**（语义归并）：anti-hallucination C 附 integration-gap（接线锚+实体解析）+ phase-card 验收两条（L3 双落点）
- **upstream-digest-not-spec**：workflow-orchestration §一.3 re-read-source + checklist 源料行
- **workflow-resume-on-quota-death**：§一.6 熄火续跑操作序 + checklist 行（与上卡同 commit 因同文件）
- **interleaved-shared-file-single-commit**：kit/commit-layering bisectable 边界条
- **staged-residue-breaks-atomic-commit**：anti-hallucination 第六型#4 + commit-layering/AGENTS-skeleton 提交纪律（L3 双落点）
- **dirty-file-not-garbage-triage-before-clean**：sop Phase 7 工作区分诊 + guard README 人侧旁注
- 附记：收件 c5a73cb/afbc2de/7d85336；12 卡 9 内容 commit（3 组语义归并）；README bench 24→25；本批无 major
- 来源：`intake/2026-07-21-euan-flutter.md` + `2026-07-23-euan-flutter.md` + `2026-07-26-euan-flutter.md`（Euan pin v0.3.0）

### 第十二次消化（2026-07-19，Euan 结账 2 卡全 landed；定级 minor 攒批）
- **privacy-feature-tension-hybrid-optin**：elicitation-protocol 新增 §五 隐私×能力冲突（结构化默认 + 证据 opt-in + 禁双全叙事）+ prd-template 第 10 条
- **partial-constitution-unpause-dtable**：AGENTS-skeleton 暂停项局部解禁（D 表划界 + 多面同步）+ §9 一行 + truth-hierarchy §六
- 附记：收件 e5878c1；一卡一 commit（83e8dec 隐私 / 7add1e7 局部解禁）；产品口径 D18 留项目端不出仓（域判据）

### 第十一次消化（2026-07-18，IPONews 四结+五结 5 卡全 landed；定级 minor 攒批）
- **long-task-loop-ledger-on-phase**：kit/phase-card-template §2b 可选 Loop 台账 + anti-hallucination 机制5 半句
- **human-async-not-global-bottleneck**：kit/progress-template current_bottleneck 旁注 + mechanisms/three-stabilizers TOC 口径
- **hook-dev-floor-api-prod-gate**：anti-hallucination 附节 dev floor vs prod API gate + claude-hooks-template 文首
- **session-receipt-before-gated-actions**：kit/guard README L-进场会话回执模式 + AGENTS-skeleton 可选 preflight 行
- **modular-prompt-orphan-must-fail**：sop 外部服务集成清单第 7 条 + anti-hallucination 映射行
- 附记：开场收件 970ccf1/573346f；一卡一 commit（ae9bec8/e0cef89/af03b2f/69c7cc6/0c7b2e7）；来源 intake/2026-07-16-IPONews.md + 2026-07-18-IPONews.md

## v0.5.1（2026-07-16）——默认目录 setup + 三工具装机文档

> **patch**：不用知道也能用；推荐新装走 setup 默认目录。存量 pin v0.5.0 可直升。无 major、无迁移注记需求。

### feat：默认工作目录 + setup 一键装机 + v1.0 定义入 snapshot
- **默认 B**：mac/linux `~/.local/share/agent-on`；Windows `%LOCALAPPDATA%\agent-on`；`agent_on_paths` 解析序末档认默认目录
- **scripts/setup.py**：clone/checkout pin、写 config、可选 plugin/symlink、doctor、intake-lint
- **snapshot/2026-07-16-v10-and-setup.md**：v1.0 四条诚实验收 + 目录/setup 拍板
- **AGENTS.md**：当前阶段改为 v0.5.x / 下一里程碑 v1.0（去掉陈旧 v0.2 表述）
- README 五分钟装机改以 setup 为推荐入口；settlement 结账前提示 intake-lint
- **第十次消化开场（2026-07-16）**：intake 零 pending（60 landed / 4 deferred / 2 rejected）；无收件；本批无卡落地。附带：封本 patch + 清理孤儿 worktree

### docs：三工具装机入口写清下载源
- README「给朋友的 5 分钟装机」扩为 Claude Code / Codex / Grok 分节：唯一源 GitHub + 各工具从哪来/怎么挂
- codex/README 远端 marketplace 与离线 clone 路径对齐 public 仓

## v0.5.0（2026-07-16）——Plugin 装机 + 可移植路径 + 上游贡献

> 封版依据：路线 A 分期发；Claude marketplace install + hook 注册本机实测通过；路径废除 Chao 默认 `~/Projects/Agent-On`；上游贡献 L0–L3 协议 + GitHub 模板。**minor**：存量项目 pin 直升即可，实例化文件不动。装机见 README「给朋友的 5 分钟装机」。
>
> 诚实边界（不挡 minor 封版）：Codex 侧 plugin hook 仍 `hooks:{}`（#16430 未接线）；Claude 交互会话 PreToolUse 真拦未在登录 TUI 补跑（合成拦测 + debug 注册已有证据）。

### 主能力（本版用户可见）
- **Plugin 分发**：`.claude-plugin/*` + `.codex-plugin/*` + `skills/agent-on` 别名 + Claude `hooks/hooks.json`（guard 经 `${CLAUDE_PLUGIN_ROOT}`）
- **路径 A/B**：运行包 = plugin；可写工作仓 B = `AGENT_ON_ROOT` / `~/.config/agent-on/config.json` / lock「本地路径」；`doctor`；无 B 时 settle 拒、guard fail-open
- **上游贡献**：默认只「用」；自愿 intake-only PR / Issue；禁止社区直改 playbook/kit
- **第九次消化**：结账回执 default-branch 硬门 + 待消化 N 读时对账

### 上游贡献形态：intake-only PR / Issue，禁止社区直改 canonical（2026-07-16）
- **协议**：`boot/settlement.md` 新增「上游贡献形态」——L0 私货 / L1 intake / L2 运输(PR|Issue) / L3 仅维护者消化；默认不强制 PR；PR diff 白名单 `intake/**`
- **模板**：`.github/ISSUE_TEMPLATE/intake-card.md`、`.github/PULL_REQUEST_TEMPLATE/intake.md`、默认 `pull_request_template.md`
- **双落点轻量**：intake/README 第六条、iteration-loop §六½、promotion-card 纪律、skill 贡献指引、README FAQ
- 与可移植装机正交：plugin(A)+任意 B；回馈≠人人改 main

### 可移植路径：废除 Chao 默认 `~/Projects/Agent-On`（2026-07-16）
- **协议**：装机面 A（plugin / `CLAUDE_PLUGIN_ROOT`）与可写工作仓 B 分离；B 须显式登记（`AGENT_ON_ROOT` → `~/.config/agent-on/config.json` 的 `work_root` → lock「本地路径」）；任意 OS/任意文件夹名
- **实现**：新增 `kit/guard/agent_on_paths.py`（doctor 报告）；guard 改用其 `resolve_work_root`，**未登记 B 时 fail-open**；示例 `kit/agent-on-user-config.example.json`
- **skill**：`$READ_ROOT` / `$WRITE_ROOT` 双解析；子命令 `doctor`；settle/digest 无 B 拒绝；禁止猜 `Projects`
- **文档**：README 远程装机 + Windows；lock 模板本地路径说明；settlement 消化粘贴令用 `$WRITE_ROOT`；codex 全局片段去硬编码
- **验证**：config 拦/放、无 B fail-open、`AGENT_ON_ROOT` 覆盖、lock 解析

### 第九次消化（2026-07-16，IPONews 三结 2 卡全 landed；定级 minor 攒批）
- **settlement-receipt-on-default-branch**：`boot/settlement.md` 上半场 step5 加 **default-branch 硬门**（回执 commit 须为项目 main 祖先，`git merge-base --is-ancestor` 验收；worktree/feature 上只写 intake，回执须 checkout main 或 cherry-pick 后再报完成）+ `boot/session-handshake.md` 读取表「工作树蔓延」行扩 **结账回执困死枝巡检**（与 worktree-sprawl 同族）。双落点均在 boot/ 执行面（结账写点 + 握手读点）
- **pending-digest-reconcile-on-read**：`boot/session-handshake.md` 读取表「待消化」行扩为 **读时对账、不盲信 N** + `boot/settlement.md` 上半场 step0 顺手对账清已收口积压、step6 写明 **N 生命周期**（结账写/抬高；清 0 在下次打开项目的会话，不在消化端——跨仓边界禁止回写项目）。双落点：handshake 读 + settlement 写两侧交叉引用
- 附记：开场工作区有 v0.5 阶段 2–4 WIP，digest 前 stash 隔离、收尾 pop 还原；一卡一 commit（4dac4c2 / 423645f）；用户原话「用户拍板两道防呆入协议」直落未再出选择题
- 来源：`intake/2026-07-15-IPONews.md`（IPONews pin v0.3.0；实证 acf6e4a 困旁支 + loop-notes 粘「待消化 3」）

### 阶段 2–4：Claude guard 入 plugin + 换机文档 + Codex 备件（2026-07-15）
- **阶段 2**：新增 `hooks/hooks.json`——Claude PreToolUse 调 `python3 "${CLAUDE_PLUGIN_ROOT}/kit/guard/agent-on-git-guard.sh"`（随 plugin 启用自动挂载）
- **阶段 3 备件未接线**：`hooks/hooks-codex.json` 已写；`.codex-plugin/plugin.json` **仍** `hooks:{}`——#16430 未实测前不接线
- **阶段 4 文档**：README 换机 A/B；codex/README；hooks/README + kit/guard 两路注册
- **Claude 闸门实测（2026-07-16）**：marketplace add 本仓 → install agent-on@agent-on；details Hooks(1) PreToolUse；debug `Loading hooks from plugin: agent-on`；cache 路径合成 PreToolUse exit 2

### 阶段 1：Plugin 骨架落地（2026-07-15）
- `.claude-plugin/plugin.json` + `marketplace.json`（自营单仓 `source:"./"`）+ `.codex-plugin/plugin.json` + `skills/agent-on → ../skill`
- 验证：JSON 合法；Codex marketplace add 实测通过

## v0.4.0（2026-07-15）——从文档约定到机械强制

> 封版依据：2026-07-11 拍板「攒批至新项目 dogfood 后随 v0.4.0 一并封版」；AInvestment 完成 BOOTSTRAP 全流程 dogfood（init→规划→结账→消化）+ 两默认件实测。本版主线：规划链 §1.5、强制层 guard（PreToolUse 双工具）、项目端零 git 边界、消化协议三缝、第二~八次消化全部内容。minor 档：存量项目 pin 直升即可，实例化文件不动。

### 第八次消化（2026-07-15，AInvestment 首结 3 卡 + Euan 两日 2 卡：4 landed + 1 rejected；定级 minor 攒批）
- **裁决先行（卡A rejected 立边界）**：delayed-data-as-product-truth 判 rejected——**agent-on 只收 AI 协作过程的教训，项目域知识归项目端**（用户裁决；过程内核「外部数据对账」已被 sop 集成清单第 3 条覆盖，再加=指令膨胀）。此判例是出仓判据的第一次显式适用
- **freeze-deferred-channels-as-ban**：AGENTS-skeleton §1 暂停项行补渠道/触点类举例 + BOOTSTRAP 需求六问 Q6 补半句——MVP 后置渠道必须入禁令表，防实现会话当 soft backlog 偷做
- **interaction-reference-not-asset-clone**：BOOTSTRAP 六问 Q2 参照物扩注「两栏拆法：学什么/不复制什么+能力对等边界」+ elicitation-protocol 参照物回显条同补
- **worktree-when-and-collision-guard**：multi-contributor-protocol §二.2 补「何时开 worktree 判据（单轨不开/≥2 轨且文件域可切才开）+ 三共享面归口（design/·contracts/·progress.yaml）」+ merge-checklist 第 1 步合流前文件域对照
- **tool-detail-confabulation-guard**：anti-hallucination 增补**第六型「取证幻觉」**（完成幻觉的姊妹型：把没看见的细节补成看见了）三条机械防线（复跑取证开 -v / 片段≠日志 / 锚点出口前存在性检查）+ 映射表行 + review-prompt 禁令段补取证条款
- 附记：本批为「项目端零 git」新协议 + guard 上线后的首次消化——AInvestment 结账正确地只留 untracked 文件，消化开场收件 commit（bd75914），跨仓边界闭环首次完整走通；一卡一 commit（cb25d9d/4105d0f/2562bba/4f67722/901536c）
- **追加（用户拍板「项目域都归项目」）**：卡A 判例入协议——settlement 上半场 step 2 证据硬门扩「**域判据**」（出仓候选必须是 AI 协作过程教训，域知识不出仓）+ promotion-card 纪律行同落

### 强制约束层 L-动作（2026-07-13，deep-research wf_9c47f385-3e2 + 用户拍板「按建议进行」；定级 minor 攒批）
- **enforcement-layer-design 最小件**：新增 `kit/guard/agent-on-git-guard.sh`（PreToolUse hook，python3 轻量校验脚本）——「会话根不在 agent-on 仓却对 agent-on 执行 git 写操作」即 exit 2 拦截并回灌原因；读操作与 agent-on 自会话放行。实测矩阵 14/14（事故原型/cd 链/相对路径/-C/--no-verify/tag 创建读取边界/四类合法动作零误伤），kit/guard/README 记录注册片段、失效面与回滚
- 注册现状：Codex `~/.codex/hooks.json` 已挂（两家 hooks schema 相同）；Claude 侧写入被用户 autoMode soft_deny 正确拦截（改 ~/.claude/ 需确认）——**机械护栏不认 agent 身份的活例**，片段待用户确认
- 附带修复：Codex Stop hook 的 auto-sync 路径仍指已改名的 `~/claude-memory` → 改指 `~/agent-memory`（死链导致 Codex 侧会话结束不同步）
- 研究结论要点入卡（intake/2026-07-13-agent-on-self.md）：advisory 天花板官方盖章、去矛盾救不了合规率（p=0.912 null）、目录级 git 禁令必须 hook（deny glob 无 cwd）、Codex rules 不认 cwd 故跨仓模式必须 hook 或仓侧兜底；L-进场/L-收尾/git 原生兜底三件 deferred

### 消化协议第四缝（2026-07-13，Euan 越界事故取证 + 跨仓边界硬规矩执行；定级 minor 攒批）
- **settle-no-git-boundary-align**：settle 上半场 step4 从「commit 后立即 push」改为「**只写文件，不碰 git**」——项目端会话对 agent-on 仓不 add/commit/push，git 动作全归 agent-on 仓会话；消化开场三检放行未跟踪 intake/*.md 并新增「**收件 commit**」步（boot/settlement.md 两处 + intake/README 规则 5）
- 根因是**规则冲突不是（只是）没读**：step4 旧文（07-12 修并发缝）与 AGENT.md 跨仓边界（07-13 用户立）矛盾并存一天——Euan 会话依旧协议 commit 被判越界（5b4ecdd 已撤为 dangling，本批收件 e82f0d1），同日上午 IPONews 同样 commit+push 还被当「新协议实证」表扬。同一行为两份文档下分别「守规/越界」，advisory 层连自洽都难保证——此实证直接输入「强制约束层」研究（进行中）
- 卡在 intake/2026-07-13-agent-on-self.md，直落即标 landed（执行用户既有硬规矩，非新决策）

### 第七次消化（2026-07-13，IPONews 二结 4 卡全 landed，一卡一 commit 首跑；定级 minor 攒批）
- **fanout-probe-gate**：workflow-orchestration §〇 探针闸门（大扇出前 1 探针子代理验工具通路，不过不扇出）+ checklist 探针行 + bench 案例 24（同机正反对照：22 代理 66 万 token 零产出 vs 105 代理 0 错）；README 案例计数 23→24 并修陈旧拆分
- **dom-render-verification + pricing-freshness-gate（语义归并同节落地）**：workflow-orchestration 新增 **§三½ 外部事实直核纪律**两条（价格/榜单证据必须浏览器渲染 DOM、表格核总行数防静默截断；型号与价格当日官方页直核+逐项标查证日期，prompt 预填型号只当检索线索）+ checklist「调研型扇出附加条款」两行
- **quota-fallback-subagent-downgrade**：§一第 6 件断点续跑变体（已完成成果照用/缺口降档模型续跑/合成判断留主会话）+ checklist 撞配额预案行——标「单会话实证」
- 附记：一卡一 commit 新规首次实跑（4 卡 4 commit：102b87f/1205a6a/197b20b/eccb2f3），canonical 中间态窗口缩到单卡分钟级；卡2 落地时曾误吞 §四标题，同 commit 内 grep 结构核验抓回——锚点替换必须把锚补回替换文本

### 消化协议第三缝（2026-07-13，Euan 撞窗实证 + 用户拍板选项 1 直落；定级 minor 攒批）
- **digest-per-card-commit**：消化落地改「一卡一 commit」禁攒批（boot/settlement.md 下半场 step 3）——agent-on 工作区是所有项目会话的服务面，攒批拉长 canonical 中间态窗口，并发读者会读到半截规则。实证：第六次消化 12 文件一批收口，窗口期被 Euan 会话撞见未提交 BOOTSTRAP.md。来源：Euan 反馈（其观察行待其下次结账回流，届时按语义归并标已落）；候选项 2「读者按 pin 读」维持 deferred（触发 = 首个 major）

### 第六次消化（2026-07-13，框架自研究 5 卡 + Euan 尾单 1 卡，5 landed + 1 半落半缓；定级 minor 攒批）
- 频次扫描：「未 commit 的工作对其他会话不存在」坑家族跨项目双实证（AINVESTMENT 零落盘零 commit + Euan worktree-sprawl）→ 置顶升 L3 双落点
- **bootstrap-verifiable-landing（L3）**：BOOTSTRAP §2 S 捷径与 M/L 新增第 9 步 initial commit（落盘未 commit = 初始化未完成，禁报完成）+ §7 验收第 7 条 `git log` 证据——boot 侧；kit/commit-layering「环节收口 = 一个 commit」节——kit 侧
- **gstack-artifact-transcription（L3，与上同族成对）**：BOOTSTRAP §4 新增 **L8 产物入仓 + 收口 commit**（外部 skill 产物转录进 docs/ 才算环节完成；orchestrator 是规划链落盘与 commit 唯一责任人，与 L3 同构）+ §2 步 1 补 docs/{product,requirements,plans}/
- **planning-chain-routing**：BOOTSTRAP 新增 **§1.5 规划链**（调研→MRD→澄清→PRD→方案→审查→拆解路由表，M/L 分档，S 跳过；/office-hours 强制 Startup 当 MRD、/spec 下沉单卡精修、/autoplan per-milestone）+ 模板问卷化协议（AI 草拟+用户勘误、答不上落 99_待确认、禁编填）落 prd/requirement-pack 模板头注 + kit/README 启动步骤第 0 步
- **cross-tool-skill-routing-parity**：BOOTSTRAP §4 尾注「路由含压制」+ AGENTS-skeleton §skill 路由压制条款 + AGENTS-lite §0 一行——压制必须写双工具共读层（实证：AINVESTMENT superpowers 抢跑）；机器侧 AGENT.md 属 agent-memory 仓，提醒用户自补
- **novice-checkpoint-ux（半落半缓）**：最小件「口令动作收口即 commit、commit 时间线即回退时间线」并入 L8 与 commit-layering；「回退口令」deferred（触发 = dogfood 中用户真喊回退）
- **single-human-facing-list（Euan 尾单）**：playbook/truth-hierarchy 新增 §五「单读面：人读状态面一个受众只养一份，第二份直接删」+ kit/dashboard-template 铁律第 ⑤ 条「只此一份」（四条铁律 → 五条）
- 附记：本批 5 卡的拍板依据 = 全部内容已在来源会话与用户逐条对过 + 消化口令（与第三次消化同例）；intake 未收口清零

### 消化协议两缝（2026-07-12，框架自研究会话直落，2 卡 landed；定级 minor 攒批）
- **digest-semantic-grouping**：开场频次扫描分组判据从「slug 字面」扩为「slug 或 claim 语义同类」——slug 是 AI 起的会漂，同坑不同名照样置顶升 L3（boot/settlement.md 下半场 step 1）
- **digest-batch-budget**：三态分诊加预算线（硬）——选择题一场一组 ≤10 题，超线按 intake 文件先旧后新处理到线即收口，剩余留承接层播报「剩 N 份下批」（boot/settlement.md 下半场 step 2）
- 附记：两缝均为机制推演的预防性修补（无实体撞例，confidence=medium 如实标注），卡在 intake/2026-07-12-framework-research.md 追加、直落即标 landed

### 第五次消化（2026-07-12，用户三问：保鲜与升级提示，3 卡全 landed）
- 裁决先行：「定期更新」改造为**事件绑定 + 读时痛感**（时钟触发=快照定稿时已钉死的负空间）
- **readme-freshness**：消化收尾加第四件「README 对表」（agent-on 侧，双实证：两次消化都在 README 抓到过期）+ sop Phase 7 里程碑项目 README 对表（项目侧）
- **dashboard-staleness**：session-handshake 读取表加「仪表盘还新鲜吗」行 + dashboard 模板更新时机第三条「握手新鲜度提醒」
- **upgrade-nudge**：session-handshake 读取表加「pin 落后了吗」行——每次握手顺带对表 agent-on 最新 tag，落后就播报「可说『agent-on 升级』」；只提示不动手，未发布 commit 不算版本不催

### 第四次消化（2026-07-12，首次跨项目：Euan 二结 7 卡 + IPONews 首结 3 卡 + 并发缝自源 1 卡，全 landed）
- **bench 案例 21-23**（Euan 高置信三张）：双假阴性（测试锚真相源不锚生成物）/ 编辑器类 MCP 三层真相（commit 前查 mtime）/ sed 字符类事故（结构化文件禁 shell 字面量替换）+ 索引与使用时机表
- **worktree-sprawl-patrol（L3 双落点，Euan+agent-on 双处实证）**：kit/merge-checklist 第 7 步全量巡检（拆前三查/孤儿先收编）+ boot/session-handshake 读取表「工作树蔓延」行
- **穷尽式提取模式**：playbook/workflow-orchestration.md 新增 §三（分片提取→双路对抗核对，核对者必须独立生成机械对表清单）
- **design-handoff-as-file**：sop Phase 1「设计稿交接=收文件落仓，链接不是交接物」
- **hard-axis-soft-tags（判据先行）**：playbook/architect-lens.md 尾注同族补条（先给一句话判据再端 schema）
- **IPONews 三条 adopt 软化条款**（均标「单项目实证」）：低摩擦合并变体 / probe 延后条款 / 双真相源按维度分工（boot/adopt.md 三处 + kit/dashboard-template 数据源多源注记）
- **并发缝三件（当日双实体证据）**：settle step4 补「commit 后立即 push+被拒 rebase 重推+同日同项目追加同名文件」；消化下半场新增 step0「开场三检」单写者安全门
- 附记：本批为**首次真实多项目并发**——两项目同日结账零 git 冲突（intake 命名空间设计经受实战），暴露的三条协议缝当场修复

### 第三次消化（2026-07-12，1 卡：Codex 一核两适配提案经对抗裁决）
- **single-skill-kernel（landed）**：`skill/SKILL.md` 确立为唯一内核（头注声明 Claude `/agent-on` 与 Codex `$agent-on` 同源，symlink 各挂 `~/.claude/skills` 与 `~/.agents/skills`）+ 补两条路由规则（无 lock 自动判 init/adopt；HEAD 领先 tag 时诚实播报「未发布变化」不伪装成版本，同落 settlement step 0）；`codex/prompts/agent-on.md` 降为迁移壳（v0.4 dogfood 后删）；codex/README 主路改 `$agent-on`。对抗保留：每 pin 路由（触发=首个 major）、能力探测/自动路由（dogfood 前不设计）、Plugin 打包（维持先自用拍板）。附记：审查者初判 `$agent-on`/`~/.agents` 为幻觉，被实证纠正——对抗双向生效

### 第二次消化（2026-07-11，7 张卡：外部记忆系统输入 5 + 用户功能两问 2）

去向：6 landed / 1 半落半缓。定级 minor；按用户拍板**攒批至新项目 dogfood 后随 v0.4.0 一并封版**。

- **no-database-stance（入宪）**：CHARTER 边界节「记忆不建数据库」（markdown 即记忆，DB 永远可选旁挂、坏退 grep、不进必需件）+ 拍板链登记
- **auto-snapshot-triggers（L3 双落点；设计经对抗评审击穿四处后定稿）**：playbook/sop.md Phase 0「拍板即快照」（D 表搭车）+ Phase 7「快照三写点」（决策边界/险段前/交接收口，时钟回合永不触发）——playbook 侧；boot/session-handshake.md staleness 标红 + 消费记录、kit/claude-hooks-template.md 机械地板（PreCompact agent 型 hook，Claude 专属可选）——boot/kit 侧
- **conversation-idea-capture（L3 双落点）**：BOOTSTRAP §6 + kit/AGENTS-skeleton §9 想法类捷径（boot/playbook 侧）+ kit/thoughts-and-ideas-template 头部「AI 也会代笔」（kit 侧）——三保险：只进速记区/保守偏置/升级需求归用户
- **distill-merge-abstraction**：settlement 下半场「同类多条先合并抽象成一条 L2」
- **memory-health-visibility（半落）**：settlement 收尾「顺口报成长」播报（数字从文件数出禁手编）；元仪表盘维持 deferred（触发=服务 2-3 项目）
- **deferred 两张入 Backlog**：cases-retirement-tiering、semantic-retrieval-adapter（见下）

### 新增（v0.4 功能）
- **Codex 适配层**（2026-07-11 用户提——兑现宪章承诺 3「工具无关」还停在纸面的机器侧半边）：`codex/` 三件——AGENTS-global-snippet（并入 `~/.codex/AGENTS.md` 的口令路由，Codex 不读 CLAUDE.md）+ prompts/agent-on.md（Codex 自定义 prompt 约定的斜杠命令，与 skill/SKILL.md 互为镜像、arg 无关设计）+ README（接入两行/谁写谁读/卸载/诚实边界：中文口令永远主路）。项目侧零适配（AGENTS.md 双工具原生）。README FAQ「Codex 能用吗」与换机步骤已更

### 新增（v0.4 功能，2026-07-10 用户提；待新项目 dogfood 验证后随 v0.4 发版）
- **项目仪表盘**（源自 Euan `docs/dashboard.html` 已验证原型）：`kit/dashboard-template.html`——纯静态单文件零依赖，五块（当前阶段 / To-do 分「🫵等你·🤖等我」/ 执行计划 / 决策台账 / 里程碑），内容集中在 DATA 对象、从真相源重绘。**接线**：BOOTSTRAP §2 第 8 步播种（M/L）+ merge-checklist 第 7 步合流必更 + 口令「更新仪表盘」。四铁律：从真相源读了重绘 / 禁手填 / 单写者 / 忠于真相不定义真相
- **想法收集箱**（源自 Euan `docs/idea.md`）：`kit/thoughts-and-ideas-template.md`——两区（📥速记区你随手写 + 🗂已整理 AI 维护带日期），整理时归类成文标去向（待评估 / 升级需求→requirements / 结账→agent-on / 搁置 / 弃）。**接线**：BOOTSTRAP 播种（全档）+ session-handshake 握手提醒速记区非空 + 口令「整理想法」
- 两个项目内口令：BOOTSTRAP §6 + 全局 CLAUDE.md 路由——「整理想法」（全档）、「更新仪表盘」（M/L）

## [v0.3.0] - 2026-07-09 · 首次真实闭环转完一圈

> **里程碑**：CHARTER 定义的 v0.3 门槛达成——Euan 仓内 `agent-on/` 倒仓首次结账（@345bad6）+ 首次消化（本次会话）跑通。semver：minor（新增内容/模板行/案例/机件，无 breaking，存量项目可不动手）。这是「项目 → 结账 → 消化 → 发布」闭环的**第一次真实运转**（v0.2 那次是自建过程内部验证，非下游结账口令回流）。

### 首次消化（2026-07-09，23 张 Promotion Card 分诊落地）

来源：[intake/2026-07-09-agent-on-self-review.md](intake/2026-07-09-agent-on-self-review.md)（17 张四镜头评审卡）+ [intake/2026-07-09-euan-flutter.md](intake/2026-07-09-euan-flutter.md)（6 张 Euan 倒仓 delta）。去向：**21 landed / 1 deferred / 1 rejected**，零残余 pending。

- **低风险直落 7 卡**（@72185bb）：README 诚实化（删「已转过一圈」幻觉、加术语表 5 行、换机三步 FAQ、斜杠 quirk 说明）、settlement 首结账空锚分支 + 幂等注、BOOTSTRAP 档播错不重播、kit/README 限定 M/L 启动步骤 + 删倒仓前旧句
- **中高风险用户拍板 12 卡**（@d42344a）：
  - `digest-friction-paste`：settlement 收尾从「一句问句」改为默认动作（可粘贴消化开场 + 项目 loop-notes 待办位）+ session-handshake 读取表加「待消化」行
  - `prose-first-settle-path`：settlement step1 主路径显式定为「loop-notes 散文 → Promotion Card」，jsonl/audit-lint 降为 L 档旁路并标「尚未实战验证」（settlement + README 结构表 + ledger/run-card-logging 头部）
  - **`cases-delivery-channel`（L3 双落点）**：BOOTSTRAP §4 扫坑指针（playbook 侧）+ kit/phase-card-template 内联要点行（kit 侧）——bench/cases 对下游的送达通道接通
  - **`skill-routing-slot`（L3 双落点）**：kit/AGENTS-skeleton 新增 §skill 路由槽（kit 侧）+ BOOTSTRAP §1 第 5 问采集本机 skill 体系（playbook 侧）
  - `lock-model-premium-field`：kit/agent-on-lock-template 加 model + 保费档位行 + session-handshake 握手对表核模型档位
  - `intake-lint-timing`（争议→现在就做）：新增 `ledger/intake-lint.py`（Promotion 六项完整性校验器，反例测试验证硬门有效）
  - `parallel-live-as-discovery`（Euan 3x 复现 → 升 L3）：kit/merge-checklist step5 加「并行轨各跑 LIVE 当发现器」
  - `readonly-guardrails` / `destructive-api-protection` / `eval-goldset-honesty`（Euan 单次 → bench 案例）：新增 bench/cases/18、19、20 + 索引
  - `architecture-radar`（冻结令期不新开篇）：并入 playbook/architect-lens.md 附节（信号→动作→量级），修复原悬空链
  - `semver-clash`（争议→本次即打 v0.3.0）：里程碑语义与版本号同指 minor bump，累积 minor 工作骑进 v0.3.0
- **deferred 1 卡**：`probe-from-cases-zero`（冻结令期不加探针题，转化时机顺延，Backlog 保留）
- **rejected 1 卡**：`scaffold-not-design`（已在 bench/cases/02 + freedom-vs-discipline §三，结账对照清单漏扫，无新增量）

### 拍板（2026-07-09，四镜头评审后用户裁决）
- **v0.3 门槛砍半**：v0.3 = Euan 倒仓首次结账+消化跑通（单件）；新项目 BOOTSTRAP dogfood 顺延为 v0.4 —— CHARTER 成功标准 + README 路线已更
- **冻结令入宪**：首次真实结账前不新增 playbook 篇目 / kit 模板，修订必须走 intake 消化 —— CHARTER 边界节
- 对外节奏：先自用磨 2-3 个项目再谈开源；首批 16 张评审卡入 [intake/2026-07-09-agent-on-self-review.md](intake/2026-07-09-agent-on-self-review.md) 等首次消化

### 新增
- **迭代闭环六站机制**(2026-07-08,三镜头提案 + 对抗评审合成,ABDC 自举):playbook/iteration-loop.md + boot/settlement.md(结账/消化/升级三口令执行书)+ intake/ 承接层 + kit/promotion-card-template.md + kit/agent-on-lock-template.md
- **Bench 案例集**(批三):bench/cases/ 17 张翻车案例卡(Euan 实战 15 + 一代二代标本 2)
- 前身三仓归档标头(批四):agent-orchestration-playbook / communication-governance-playbook / non-drift-communication-protocol 各自 README 指回本仓

### 变更(L3 双落点成对列出)
- 六类触发采集纪律:BOOTSTRAP §6(playbook 侧)+ kit/merge-checklist.md 第 7 步扫尾行(kit 侧)
- lock 播种步骤:BOOTSTRAP §2 第 7 步(playbook 侧)+ kit/AGENTS-skeleton.md §0 指针(kit 侧)
- CHARTER 新增承诺 4「越用越强」(2026-07-07 用户拍板)
- **外部服务集成清单**:sop.md 新增六条实证清单(playbook 侧)+ BOOTSTRAP L7 精简铁律与 phase-card 集成探针既有(boot/kit 侧)——由批三案例审查倒逼产出:5 张卡引用一个从未存在的清单,消化时真的把它建出来。**这是「案例 → 协议升级」回路的第一次真实运转**(2026-07-08)
- **S/M/L 档位路由 + 存量项目接入**(2026-07-08,用户「高射炮打蚊子?已开工的项目怎么用?」两问触发——「脚手架不合身」信号的即时消化):BOOTSTRAP §1 定档三问 + §2 S 档三件套捷径(boot 侧)+ kit/AGENTS-lite.md 轻装宪法(kit 侧)+ boot/adopt.md 存量接入书(考古→定档→增量补件,含升档协议);README 全面重写为「三入口两口令」产品自述
- **`/agent-on` 斜杠命令**(2026-07-08,用户「怎么更丝滑调用」触发):skill/SKILL.md 六子命令(init/adopt/handshake/settle/digest/upgrade)→ 按表导到 boot/ 对应执行书;skill 源随仓版本化,symlink 挂 ~/.claude/skills/agent-on(不占 claude-memory 同步,换机器一行 symlink);disable-model-invocation(与中文口令 CLAUDE.md 路由分工:斜杠管确定性,口令管自然语言)
- **工作流编排防幻觉七件**(2026-07-08,agent-on 融合工程自身实战的即时消化——脚本控制流/schema 强制/证据派工/对抗 stage/断点续跑/单一合流权):playbook/workflow-orchestration.md(playbook 侧)+ kit/workflow-orchestration-checklist.md 任务书七要素+编排七件(kit 侧);与六步协议按「会不会撞文件」分工,补齐七层防幻觉栈的第 2 层 schema 件与第 5 层整层

## [v0.2.0] - 2026-07-07

- 批一:五块骨架(Boot/Kit/Playbook/Bench/Ledger)+ 三代资产迁入 + CHARTER / README / AGENTS / BOOTSTRAP 四门面
- 批二:前身仓精选移植 21 文件——五篇方法论(真相源治理/阶段闸门/元原则/ABDC/沉淀分层)、四张卡 JSON Schema、audit-lint、ABDC 四模板、DoD 门禁、会话握手、深挖问卷、能力真相表、修正闭环
- 融合裁决全记录:snapshot/2026-07-07-fusion-map.md

## [v0.1.0] - 2026-07-01

- 工具定义 snapshot(agent-on 边界拍板)+ Loop Engineering 机制七篇导入(project-kickoff-os 时期)

## Backlog(deferred,等 dogfood 数据说话)

- ~~intake-lint:Promotion 六项机器校验~~ **✓ v0.3.0 落地**(ledger/intake-lint.py,首结账消化时人眼核卡繁琐 = 工程镜头胜出的实证)
- ~~架构雷达机制移植~~ **✓ v0.3.0 落地**(并入 playbook/architect-lens.md 附节,冻结令期不新开篇)
- changelog-lint:「major 无迁移注记不许打 tag」的机器门
- probe-from-cases-zero:案例 08/09/10/14 转化为新探针题(deferred,冻结令期不加;转化时机顺延——冻结令已解除,下次消化可捡)
- 多协作者结账对接(multi-contributor-protocol 条款)
- cases-retirement-tiering:案例集「活跃扫坑清单 vs 归档库」两级可见性——升成 L3 门禁的案例退出默认扫描(触发信号:案例超 ~40-50 张,或 loop-notes 首现「扫坑捞不到/扫一堆无关」)
- semantic-retrieval-adapter:语义检索可选旁挂(如 gbrain 索引本仓)——三到顶信号(README 漏登记≥2 次 / 同坑异措辞漏合≥3 次 / 时机表单场景 >10 卡退化全扫)任一触发才动;三设计闸:永远 optional / 坏退 grep / 不进 BOOTSTRAP 必需件
- memory-health 元仪表盘(触发=服务 2-3 个项目;结账播报半句已于 2026-07-11 先行落地)
- **v0.4 门槛**:一个新项目 BOOTSTRAP dogfood 全流程(v0.3 已达,门槛顺延)
