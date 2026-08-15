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
>
> 2. **开工先声明主题**（不是申请锁，见 §11.2）：动工前查 dashboard.html 分工行 + `git worktree list`；发现同主题已有活跃轨 = 先协调再动工。同目录不同主题照开，各自 worktree。
>
> 3. **合并即回收**：PR 合并后 24h 内删远端分支、`git worktree remove` 对应 worktree；关闭的 PR（如 #15）连分支一起废，**禁止把关闭 PR 的 diff 从直推后门塞回 main**。
>
> 4. **本地独有工作必须 48h 内上远端**（开 draft PR 也行）——2b1f923 那种「main 真实缺陷的现成修复只活在本地 worktree」= 机器一坏就丢。
>
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

01:43 当时表（保留时序证据；不是常青名单）：

| worktree / branch | lane / dirty | base / 保存证据 | 大小 | 决策 |
|---|---|---|---:|---|
| `/Users/chao/Projects/Agent-On` · `main` | primary；本轮实现改动 | `origin/main`；unpushed=0（尚未提交的本轮文件另列 dirty） | 构建后约 `990MiB` | `primary` |
| `.claude/worktrees/agent-orchestration-loop-e4fe70` · 同名 branch | `orchestrator-loop=active`；`?? orchestrator-state.md` | HEAD 是 base 祖先、unique=0；无 upstream/PR | `1.3MiB` | `rescue`（dirty + active + recent） |

运行面也在本轮闭环：开场 PATH 上 `agent-on --version` 是 `0.7.0`、`agent-on worktree status` 报 unknown subcommand；安装后 `/Users/chao/.cargo/bin/agent-on --version` 为 `0.12.0`，`worktree gc --help` 显示 dry-run-only 接口，PATH 实跑得到 `gh: ok / CANDIDATES (0) / READ-ONLY`。

### 2.1 发布前最终 read-back（01:49 CST）

本轮提交与本地 tag 封好后重新从权威状态读取，不沿用 01:43 表里的“尚未提交”描述：

| worktree / branch | clean / lock | 三判据证据 | 大小 | 当前裁决 |
|---|---|---|---:|---|
| `/Users/chao/Projects/Agent-On` · `main` | clean / 未标锁 | 相对 `origin/main` 为 `0 / 3`（behind / unique），upstream 尚有 3 个未推提交；主树不进入回收判断 | `993M` | `primary`，永不回收 |
| `.claude/worktrees/agent-orchestration-loop-e4fe70` · 同名 branch | `?? orchestrator-state.md` / 未标锁 | ① HEAD 是 `origin/main` 祖先；② unique=0，但无 upstream/PR；③ dirty 明确失败；另有 `lane=active` | `1.3M` | `rescue`，绝不回收 |

`df -h /System/Volumes/Data` 仍为总量 `1.8Ti`、已用 `478Gi`、可用 `1.3Ti`、使用率 `27%`。联网运行 `agent-on worktree gc --dry-run --repo /Users/chao/Projects/Agent-On --base origin/main --quiet-hours 24` 得到 `gh: ok`、`primary / rescue`、`CANDIDATES (0)` 与只读回执。`gh pr list --state all` 此时出现 PR #1（`claude/truth-page-dev-timeline-7ed180 → main`，OPEN）；它保存的是已拆 worktree 的 rescue 分支，不会把现存 dirty 执行轨变成候选。

远端发布状态也单独 read-back：`git ls-remote` 显示 `origin/main @ 82a57b1`，`v0.10.0` 与 `v0.10.1` 已存在，`v0.12.0` 尚未出现。因此这里证明的是**本地实现已封版、远端仍待明确授权发布**，不把本地 tag 冒充远端可升级版本。

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

## 附录 A：Dartify `CLAUDE.md:27–103` 完整逐行原文

下列文本来自 `/Users/chao/Projects/Dartify/CLAUDE.md`，命令为 `nl -ba /Users/chao/Projects/Dartify/CLAUDE.md | sed -n '27,103p'`。行号是原文件行号；除加左侧行号外，不作摘要或改写。

```text
 27  - **多会话并行是默认姿势**：要开几条会话就开几条，不需要跟谁报备。隔离靠机制不靠人记——**每条会话开工第一件事：给自己开一个 worktree**（见下节）。
 28
 29  ## 多会话并行（worktree）
 30
 31  **几条会话都行，代价是磁盘不是纪律。** 一条会话一个 worktree，git 层面天然互不干扰；同一个目录（`app/`）也能多条会话同时改，冲突留给 rebase 解，不靠排队。分支命名与 PR 硬门（§三）、轨道与独占区（§一）、progress.yaml 记账（§五）以 [CONTRIBUTING.md](CONTRIBUTING.md) 为准，本节只讲「怎么隔离」。
 32
 33  - **能开几条看余量**：每个 worktree 各自编译，一份 `app/.dart_tool` + `app/build` ≈ 2.6G，不与主目录共享。按 `余量 ÷ 3G` 估能开几条，`< 5G` 就先去主目录 `flutter clean` 腾地方：
 34
 35    ```bash
 36    df -h /System/Volumes/Data
 37    ```
 38
 39  - **怎么开**：有原生 worktree 工具（`EnterWorktree` / `/worktree`）就用它——落在 `.claude/worktrees/`（已 gitignore），默认从 `origin/main` 切，会话退出时提示回收。创建后把分支名改成仓内规范 `feat|fix|docs/<issue#>-<slug>`。没有原生工具再手敲（**在主目录 `~/Projects/Dartify` 跑**）：
 40
 41    ```bash
 42    git worktree add -b feat/<issue#>-<slug> .claude/worktrees/<slug> origin/main
 43    ```
 44
 45  - **接着干下一件事时，新分支必须从 `origin/main` 长，不能从当前 HEAD 长**。同一个 worktree 里连着做两件事很自然——上一条 PR 刚合，随手 `git switch -c 下一条` 就接着写。**只要上一条是 squash 合的，这一步就埋雷**：squash 在 main 上生成的是**新 hash**，你脚下这条分支的原始提交并不是 main 的祖先，于是新分支把「已经进了 main 的改动」又背了一遍。症状很唬人——PR 一开就是 `DIRTY`（与 base 冲突），**而且仓内 CI 一个 job 都不触发**（只有不依赖 checkout 的 GitGuardian 会跑），看上去像 CI 坏了，其实是 PR 压根没进到能 checkout 的状态：
 46
 47    ```bash
 48    git fetch origin && git switch -c <新分支> origin/main   # 对
 49    git switch -c <新分支>                                    # 错:从刚合并的分支头上长
 50    ```
 51
 52    已经长歪了也不用重来，把那条已合并的摘掉即可（`<已合并提交>` = 上一条 PR 在本地的原始提交）：
 53
 54    ```bash
 55    git rebase --onto origin/main <已合并提交> <新分支>
 56    ```
 57
 58    （2026-08-03 实测：PR #71 squash 合并后接着开 #72，正是这样卡住的。这与下面「怎么收」里判据①的误判是同一个病根——squash 换 hash——只是换了副面孔：那边表现为「该收的 worktree 判成不该收」，这边表现为「PR DIRTY + CI 不跑」。）
 59
 60  - **新 worktree 起步**：`app/config.json` 与 `app/ios/Flutter/Local.xcconfig`（签名团队）都是 gitignored 的，不会跟着 checkout 过来，**漏了这步真机/build 直接起不来**：
 61
 62    ```bash
 63    cp ~/Projects/Dartify/app/config.json app/config.json && cp ~/Projects/Dartify/app/ios/Flutter/Local.xcconfig app/ios/Flutter/Local.xcconfig && cd app && flutter pub get && dart run tool/gen_tokens.dart && dart run build_runner build --delete-conflicting-outputs
 64    ```
 65
 66    签名换团队只改 `Local.xcconfig`，别在 Xcode 界面里选 team——界面选择会把 ID 写回 `project.pbxproj` 带进仓（2026-07-29 就是这么混进过一个别的机器的 ID）。
 67
 68  - **同目录并行**：两条会话都改 `app/` 完全允许，不必拆子目录、不必排队。代价是后合的那个 PR 要 `git rebase origin/main` 自己解干净（CONTRIBUTING §四 已定：冲突不带进 main）。降低代价的办法是 PR 小、活短（≤2 天）、合得快，不是少开会话。
 69  - **撞车识别**：在自己的 worktree 里干活基本不会再撞。**万一编译/analyze 报错在自己完全没碰过的文件上，那说明有会话在主目录裸跑**——查时间戳确认（几十秒内=对方正在写半成品），自己这份照旧继续，让那条会话去开自己的 worktree：
 70
 71    ```bash
 72    git status --short && ls -lT <可疑文件>
 73    ```
 74
 75  - **提交纪律**：一律 `git add <明确文件列表>`，禁止 `-a` / `.`（会把别人的半成品裹进自己的提交）；**绝不 `git stash` / `git checkout --` 整个仓库**，那会扫掉另一条线正在编辑的东西，且不可恢复。
 76  - **纯文档改动**也走 worktree：不要把文档提交混进正在开 PR 的功能分支。
 77  - **怎么收**：本节此前只教了开、没教收，于是攒成灾——**2026-08-01 实测：29 个 worktree 占 9.3G。按下面的判据核了一遍，20 个可回收、9 个有活在跑——收完 1.9G，释放 7.4G**。CONTRIBUTING §三 的「生命周期 ≤2 天」不会自己实现，得有人按判据收。**三条判据全中才删，缺一条就留着**：
 78
 79    ```bash
 80    git merge-base --is-ancestor <分支> origin/main   # ① 已并入 main（无输出=是）
 81    git rev-list --count origin/<分支>..<分支>         # ② 无未推送提交（=0；分支已合并、远端已删则跳过此条）
 82    git -C <worktree路径> status --porcelain          # ③ 未提交改动无价值（见下「假脏」）
 83    ```
 84
 85    ⚠️ **判据①遇上 squash 合并会误判**。CONTRIBUTING §四 允许单主题 PR squash，而 squash 把整条分支压成一个**新 hash**，`--is-ancestor` 于是判「否」，内容其实早已在 main 里。症状是「PR 明明显示 MERGED，判据①却说没并入，`origin/main..分支` 里原样躺着全部提交」——**这时以 PR 状态为准，别信 ①**：
 86
 87    ```bash
 88    gh pr list --repo Palebluedot-ai/Dartify --head <分支> --state merged
 89    ```
 90
 91    （2026-08-01 实测：PR #44 squash 合并后，拿它的分支去 rebase 会在自己改过的行上跟「自己」冲突——因为在重放已经进了 main 的改动。碰到这种冲突先查 PR 状态，八成是分支已经完成使命了。）
 92
 93  - **判据③的「假脏」三种**：看着有改动，其实全是垃圾，别为它们留着 2.6G——
 94    - `M app/ios/Runner.xcodeproj/project.pbxproj` 里只有 `DEVELOPMENT_TEAM` 变化 → Xcode 界面选 team 写回的。PR #40 之后团队 ID 归 gitignored 的 `Local.xcconfig`，这改动留着只会再污染一次（2026-08-01 一次抓到**四个** worktree 都躺着它，占 3.5G）
 95    - `?? api/node_modules`、`?? app/build` → 未追踪的产物
 96    - 改动里引用旧包名 `package:euan/...` → D24 已全改 `package:app/`，这种代码现在根本编译不过，是死的
 97  - **回收命令**（只删工作目录与编译产物，**分支保留**——要用时一条 `worktree add` 就能重建）：
 98
 99    ```bash
100    git worktree remove --force .claude/worktrees/<name>
101    ```
102
103    ⚠️ **别删还开着 PR 的 worktree**：review 要改就得重建，重跑 `pub get` + `build_runner` 好几分钟，省下的那点空间不值。
```
