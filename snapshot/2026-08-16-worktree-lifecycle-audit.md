# 2026-08-16 Worktree 生命周期实盘与产品裁决

> 职责边界：把 Dartify 的真实多会话 / babysit 经验、Agent-On 当日全部 worktree 实盘和本轮产品裁决钉在一起。它是日期快照，不替代 `kit/worktree-control-plane.md` 的长期规则，也不是一份可长期手填的回收名单。

## 结论先行

Agent-On 需要的不是“自动删 worktree”，而是一张**每次重算、拿不准就停**的本机交通报告：单写会话可在主树；第二个写者出现后，所有写者独立 worktree + lane；回收必须同时证明“已落地、无孤本、clean”，再叠加 unlocked / 非活跃 / 静默窗口。CLI 只做 `gc --dry-run` 报告，任何删除仍由人明确授权。

2026-08-16 的审计窗口里没有一棵非主树可安全回收：两棵旧树起初各有本地 unique commit、无远端分支；审计期间新出现的一棵树随后写入未跟踪文件。01:35 read-back 时，另一会话已把其中两笔孤本 push 到远端并拆掉对应 worktree；这是“先保存、再拆树”的现场演示，不是本审计会话执行的删除。最新动态 `candidates` 仍为空。

## 1. Dartify 原始事实

### 1.1 CLAUDE.md 的创建、隔离与三判据

权威源：`/Users/chao/Projects/Dartify/CLAUDE.md`。

- `:27`：“多会话并行是默认姿势”，且“每条会话开工第一件事：给自己开一个 worktree”。
- `:31`：“一条会话一个 worktree”；这是 Dartify 的项目级严格策略。
- `:39–43`：原生工具落 `.claude/worktrees/`；手工例子是 `git worktree add -b feat/<issue#>-<slug> .claude/worktrees/<slug> origin/main`。
- `:45–50`：下一件事必须从 `origin/main` 开新分支，不能从当前 HEAD 续长；原因是 squash 后旧 commit hash 不是 main 的祖先。
- `:69–75`：在自己未碰文件上看到并发半成品，说明有人在主目录裸跑；提交只 add 明确文件，禁止整仓 stash / checkout。
- `:77` 原句：“三条判据全中才删，缺一条就留着”。三条命令在 `:80–82`：

  ```bash
  git merge-base --is-ancestor <分支> origin/main   # ① 已并入 main（无输出=是）
  git rev-list --count origin/<分支>..<分支>         # ② 无未推送提交（=0；分支已合并、远端已删则跳过此条）
  git -C <worktree路径> status --porcelain          # ③ 未提交改动无价值（见下「假脏」）
  ```

- `:85–89` 明确修正：squash 会让判据①假阴性；PR 显示 MERGED 时以 PR 状态为准。
- `:93–96` 列出的 `DEVELOPMENT_TEAM`、Flutter build、旧包名是假脏的 **Dartify 项目事实**，不能复制为通用白名单。
- `:97–103`：当时的人工回收只删 worktree、保留 branch；open PR 的 worktree 不删。

补充生命周期源：`/Users/chao/Projects/Dartify/AGENTS.md:124–130`。

- `:126` 一个主题 = 一条分支 = 一个 worktree，不跨主题复用；
- `:127` 开工前查声明与 `git worktree list`；
- `:128–129` 合并后回收，本地独有工作须尽快上远端；
- `:130` 当时要求每周巡检。babysit 实战已经把频率收紧到每天一次。

其中 `:126–130` 的 lifecycle 原文是：

> 1. **一个主题 = 一条分支 = 一个 worktree**；禁止长命分支跨主题复用（github-status-sync 分支被三个不相干主题复用的教训）。  
> 2. **开工先声明主题**（不是申请锁，见 §11.2）：动工前查 dashboard.html 分工行 + `git worktree list`；发现同主题已有活跃轨 = 先协调再动工。同目录不同主题照开，各自 worktree。  
> 3. **合并即回收**：PR 合并后 24h 内删远端分支、`git worktree remove` 对应 worktree；关闭的 PR（如 #15）连分支一起废，**禁止把关闭 PR 的 diff 从直推后门塞回 main**。  
> 4. **本地独有工作必须 48h 内上远端**（开 draft PR 也行）——2b1f923 那种「main 真实缺陷的现成修复只活在本地 worktree」= 机器一坏就丢。  
> 5. **每周一次 worktree 巡检**（maintainer）：`git worktree list` + `git branch --merged main`，已合并的清掉，错配的（目录名≠分支名）改掉或注明。

### 1.2 babysit 真文

原始记录：`/Users/chao/.claude/projects/-Users-chao-Projects-Dartify--claude-worktrees-dartify-pr-babysit-d6c34c/07973d4b-60e0-4fc4-a2e0-74b6ca285e32.jsonl:1`。该 JSONL 记录的 `.content` 解码后，相关内行如下（可用 `sed -n '1p' … | jq -r '.content' | nl -ba` 复现）：

- 内行 `5–6`：babysit 自己也从 `origin/main` 开 `chore/babysit-0816`，路径 `.claude/worktrees/babysit`；
- 内行 `23`：worktree 盘点按三判据，每天一次，同时看 `df -h`；
- 内行 `26`：不进入别的 worktree add/commit；删远端分支必须先问；
- 内行 `33`：祖先判据与 MERGED 冲突时，以 PR 状态为准；
- 内行 `34`：别树 dirty 可能是历史敏感信息清洗的反向，代提交会把敏感信息灌回去；
- 内行 `38`：已完成使命的远端旧分支，删除仍须用户点头；
- 内行 `39`：当时点名 `frosty-sutherland-2675f2` / `blissful-dirac-1322ef` 可回收；
- 内行 `43`：“拿不准 = 报告而不是猜”。

这些 worktree 约束的解码原文如下（左列是内行号）：

```text
 5  1. 给自己开 worktree（CLAUDE.md 铁律，值守也不例外）：在主目录 ~/Projects/Dartify 跑
 6  `git worktree add -b chore/babysit-0816 .claude/worktrees/babysit origin/main`  值守大多是 gh/git 读操作，flutter 那套 bootstrap 不用做；要补账时在这个 worktree 里开分支。
23  4. 低频（每天一次即可）：worktree 盘点按 CLAUDE.md 三判据（squash 误判见 §4 末条）；`df -h /System/Volumes/Data` 余量。
26  ════════════════════════════════════════ §3 权限边界（硬，越界前先问） ════════════════════════════════════════ 可自主：只读检查 · 本地 worktree 操作 · 补账/值守自己的 state PR 全流程（推分支、开 PR、CI 绿后合并）       · 合并纯 chore(state) 记账 PR（不带代码/脚本改动的） 必须先问用户：合并功能 / scripts/ / AGENTS.md / .github/ 类 PR（#150 #151 #152 #153 全在此列）       · 删远端分支 · 回滚或推数据库迁移 · 关闭别人的 PR · 清单外的一切外向操作 永远不做：进别的 worktree 目录 add/commit · `git add -A` · 直推 main（分支保护拦着，#151 还要把纸面豁免废掉）
33  * `merge-base --is-ancestor` 判「未并入」但 PR 显示 MERGED → squash 换 hash 的已知误判，以 `gh pr list --state merged` 为准。
34  * 别的 worktree 里 `git status` 一堆「改动」→ 可能是历史清洗（#124/#128）的反向，提交回去=敏感信息回灌，绝不代提交。
38  * origin 旧分支 `claude/duplicate-name-handling-a951fc` 已完成使命（内容经 #147 落地），删除需用户点头。
39  * worktree `frosty-sutherland-2675f2` / `blissful-dirac-1322ef` 按三判据可回收。
43  ════════════════════════════════════════ §6 汇报纪律 ════════════════════════════════════════ 中文书面口语，不要翻译腔套话；有动作才出声，全绿安静（noop）； 任何「已合 / 已修 / 完成」必须贴命令实际输出；拿不准 = 报告而不是猜。
```

内行 39 只是 **Dartify 当时的交接快照**。它证明值守需要一张 known reclaim list，但不能成为 Agent-On 的静态名单或项目特有白名单；下次运行必须从当前 git / PR / lane 状态重算。

### 1.3 既有自动脚本为什么不原样移植

`/Users/chao/Projects/Dartify/scripts/worktree-gc.sh:1–136` 把项目经验做成每日脚本，支持 `--dry-run`；但 `:118–123` 在非 dry-run 下会直接 `worktree remove --force`，且没有显式 locked 判据。它适合作为来源项目证据，不适合作为 Agent-On 的通用权限默认。

产品保留三判据、PR 优先、静默窗口与每日盘点；删除动作、项目假脏白名单和特定路径不移植。

## 2. Agent-On 当前实盘

审计基线：`main @ 82a57b1849b08d9e9722f36ec1e71268d906249e`（tag `v0.11.0`）。命令：

```bash
git worktree list --porcelain
git -C <path> status --short
git -C <path> rev-list --left-right --count HEAD...origin/main
git for-each-ref --format='%(refname:short) %(upstream:short) %(upstream:track)' <branch>
git show-ref --verify refs/remotes/origin/<branch>
du -sh <每个精确路径>
df -h /System/Volumes/Data
```

`rev-list` 表中写成 `unique / behind`。`git worktree list --porcelain` 没有任何 `locked` / `prunable` 行；这只能证明 git metadata 未标锁，不能证明没有活会话。主审计会话对两条旧分支运行 `gh pr list --repo Palebluedot-ai/agent-on --state all --head <branch> …`，输出均为 `[]`。下表是四棵树同时存在时的 01:30 审计面：

| worktree / branch | clean / lock | unique / behind | 远端与 PR | 大小 | 三判据与裁决 |
|---|---:|---:|---|---:|---|
| `/Users/chao/Projects/Agent-On` · `main` | 首轮 clean；实现中 dirty / 未标锁 | `0 / 0` | 跟踪 `origin/main` | 主审计约 `700M`；构建中继续增长 | `primary`，永不回收 |
| `.claude/worktrees/agent-orchestration-loop-e4fe70` · `claude/agent-orchestration-loop-e4fe70` | `?? orchestrator-state.md` / 未标锁 | `0 / 0` | 无 origin ref；01:25 新建 | `1.3M` | ①②通过；③ dirty 失败，且 live/recent → `rescue` |
| `.claude/worktrees/cranky-booth-aad846` · `claude/truth-page-dev-timeline-7ed180` | clean / 未标锁 | `2 / 86` | 无 origin ref；PR `[]` | `896K` | ①无 landed 权威；②两笔本地孤本失败；③通过 → `rescue` |
| `.claude/worktrees/truth-page-dev-timeline-7ed180` · `claude/framework-claude-codex-sync-cdebcc` | clean / 未标锁 | `1 / 137` | 无 origin ref；PR `[]` | `692K` | ①无 landed 权威；②一笔本地孤本失败；③通过 → `rescue` |

磁盘证据：`/System/Volumes/Data` 总量 `1.8Ti`、已用 `478Gi`、可用 `1.3Ti`、使用率 `27%`。当前没有磁盘紧急性，更没有拿空间压力放宽判据的理由。

审计开始时只有前三棵（主树 + 两棵旧树）；01:25 新 worktree 出现，随后从 clean 变成 `?? orchestrator-state.md`。这说明“维护一张手写名单”甚至在一个会话内就会过期：动态报告先会因 quiet/live 判 `keep`，下一次已经因 dirty 改判 `rescue`。

01:35 再读 `git worktree list --porcelain` 时，`cranky-booth-aad846` 已被另一个并发会话拆掉，同时 `refs/remotes/origin/claude/truth-page-dev-timeline-7ed180` 出现：两笔 commit 已从“仅本机孤本”升级为“远端已保存、尚未进入 main”，本地 branch 仍保留。此时三棵 worktree 是：主树、dirty/live 的 `agent-orchestration-loop-e4fe70`、含一笔孤本的 `truth-page-dev-timeline-7ed180`。

01:43 又一次 read-back 时，只剩主树与 `agent-orchestration-loop-e4fe70` 两棵；后者已登记 `lane=active`，owns 仅 `orchestrator-state.md`，仍 dirty/recent，故 `rescue`。远端 rescue 分支已整理为 `origin/claude/truth-page-dev-timeline-7ed180 @ 44d3672`（两笔提交仍在）；`framework-claude-codex-sync` 的 branch/worktree 已由并发会话移除。对 `eff1cef..origin/main` 的逐文件 diff 证明 main 已有更强的 landed 文本，但本审计会话没有执行这次删树/删枝。

三次真实 CLI read-back 都是 fail-closed：沙箱内四棵树运行时 `github_pr_query.status=unknown`，决策为 `primary / rescue / rescue / rescue`；第一次只读联网时 `gh: ok`，三棵现存树为 `primary / rescue / rescue`；最终只读联网时 `gh: ok`，两棵现存树为 `primary / rescue`。每次末行都是 `CANDIDATES (0)` 与 `READ-ONLY: no worktree, branch, lane, or report file was changed.`。动态列表随并发事实变化，但安全结论没有摇摆。

最终当前表（01:43）：

| worktree / branch | lane / dirty | base / 保存证据 | 大小 | 决策 |
|---|---|---|---:|---|
| `/Users/chao/Projects/Agent-On` · `main` | primary；本轮实现改动 | `origin/main`；unpushed=0（尚未提交的本轮文件另列 dirty） | 构建后约 `990MiB` | `primary` |
| `.claude/worktrees/agent-orchestration-loop-e4fe70` · 同名 branch | `orchestrator-loop=active`；`?? orchestrator-state.md` | HEAD 是 base 祖先、unique=0；无 upstream/PR | `1.3MiB` | `rescue`（dirty + active + recent） |

运行面也在本轮闭环：开场 PATH 上 `agent-on --version` 是 `0.7.0`、`agent-on worktree status` 报 unknown subcommand；安装后 `/Users/chao/.cargo/bin/agent-on --version` 为 `0.12.0`，`worktree gc --help` 显示 dry-run-only 接口，PATH 实跑得到 `gh: ok / CANDIDATES (0) / READ-ONLY`。

两棵旧树在初审时虽疑似已被 main 上更强版本替代，机械证据仍不能叫 `safe`：

- `truth-page-dev-timeline` 分支独有 `5e10de8`、`346727d`，涉及 `kit/dashboard-template.html` 与 `CHANGELOG.md`；
- `framework-claude-codex-sync` 分支独有 `eff1cef`，涉及 `intake/2026-07-12-Euan-Flutter-2.md`。

“看起来过时”不是删除判据。先比较内容、决定抢救或明确废弃，再重新盘点；本轮不删。

## 3. v0.11.0 前的缺口

| 面 | 既有状态 | 缺口 |
|---|---|---|
| 并发阈值 | 文档写成每个写会话必开 worktree | 单线小任务也背开树成本；没写第二写者出现时主树如何退场 |
| 创建 | 只说“先用普通 git 建” | 没有跨工具 root、branch 命名和 fresh origin/default 约定 |
| 边界 | lane claim/check 已能抓 owns 重叠与越界 | 本轮 `find .git/agent-on …` 为空，盘点到的 worktree 都没有 lane registry；运行事实未纳入控制面 |
| 回收 | `status` 有 `safe|review|rescue`，另篇建议项目自持脚本 | 没有统一可运行的 `gc --dry-run`、du、quiet、locked、PR/squash 聚合报告 |
| known reclaim | 容易留在 babysit / TODO prose | 名单会随新 worktree、PR、dirty 状态立刻失效 |
| 频率 | 写前/提交前/合流前有 check | 没钉“握手 + 每日 + 每次合流后”三种触发 |
| 权限 | CLI 不提供 delete，是安全的 | 文档未把 branch 删除、`--force`、跨树 commit、locked/dirty/unknown 写成一组明确红线 |
| 安装态 | 本机全局 `agent-on worktree status --json` 返回 `unrecognized subcommand 'worktree'` | 源码能力发布后仍需升级本机 CLI；不能把源码存在误报成机器已可用 |

## 4. 产品裁决

1. **阈值按写者数，不按聊天数**：单写会话可主树；≥2 写者时，每个写者独立 worktree + lane，主树只控制/合流。只读会话要写时再转执行轨。
2. **路径跨工具兼容**：优先原生工具路径；Claude 的 `.claude/worktrees/<lane-id>` 合法。手工创建尊重项目声明的 root，未声明建议 `.worktrees/<lane-id>`，不把 `.claude` 变成跨工具唯一标准。
3. **命名与起点稳定**：branch 用 `<type>/<issue-or-lane>-<slug>`；每个新目标先 fetch，从 fresh `origin/<default>` 长，禁止继承上一任务 HEAD。
4. **回收仍是三判据**：landed 权威 + 无孤本 + clean；PR MERGED 可修正 squash 的祖先假阴性，但必须核对它合入的就是目标 base 且 `headRefOid` 覆盖当前 HEAD；MERGED 后本地又长出的 commit 仍要 rescue。
5. **CLI report-only**：`agent-on worktree gc --dry-run [--json] [--repo PATH] [--base REF] [--quiet-hours N]`；缺 `--dry-run` 在访问 repo/gh 前拒绝，不存在 apply/delete 模式。
6. **known reclaim 是派生状态**：JSON `candidates` 每次重算，保留 `errors/reasons/criteria`；如需归档，只写本机 common git dir 或本机日志，不提交机器清单。
7. **通用层不认假脏白名单**：任何 dirty 默认 rescue/review；项目可以提供可审计规则辅助人判断，但不能让 Agent-On 静默丢改动。
8. **权限 fail-closed**：自动化只检查并写本机报告；删除目录/分支、`--force`、跨树 add/commit 一律人工且目标明确授权。locked、dirty、unknown 不删。
9. **盘点三触发**：会话握手、每天一次、每次合流 read-back 后；quiet 同时看 worktree 文件与 per-worktree git admin 活动，但不把“24h 无 mtime”冒充“会话一定已关闭”。磁盘告急只增加频率，不降低标准。

长期规则落在 `kit/worktree-control-plane.md`、`kit/worktree-gc-pattern.md` 与 `kit/AGENTS-skeleton.md`；本仓自己的自举硬句落在根 `AGENTS.md`。不新增根 `CLAUDE.md`，避免双工具真相分叉。
