# 多会话 Worktree 控制面

> 职责边界：本模式管一台机器上同时存在多个 Claude / Codex 会话与 git worktree 时，如何登记轨道、划文件边界、排依赖、顺序合流和保守回收。它不替你编排任务，不自动 merge，也不自动删除 worktree。

## 一句话

**单写会话可以留在主 worktree；一旦同时有两条及以上写会话，每条写会话（包括原主会话）都必须进入独立 worktree，并登记一份轨道合同。**

“一个会话一个 worktree”只解决环境互踩，没解决工作撞题、文件越界、依赖倒序和遗忘回收。轨道合同补齐五个事实：

| 字段 | 回答什么 |
|---|---|
| `id` / `goal` | 这条轨只交付哪一个闭环目标？ |
| `owns` | 唯一允许修改的路径前缀是什么？ |
| `depends_on` | 哪条轨没 landed 前，本轨不能进入 ready？ |
| `base` | 从哪条合流线分出、当前落后多少？ |
| `status` | `active → blocked/ready → landed`，或 `parked` 释放文件域 |

运行态由 `agent-on` 写入仓库 common git dir 的 `agent-on/lanes/*.json`。同一仓的所有本机 worktree 可见，但不会进入 commit、不会让功能分支争抢 `progress.yaml`。项目的长期真相仍在 phase 卡 / progress / git 历史；这里是本机交通控制面，不是第二份产品真相。

## 角色与边界

### 控制轨（default branch 的主 worktree）

- 单一合流权威；独占 `docs/state/progress.yaml`、契约、共享设计稿和顺序编号。
- 创建 worktree、登记合同、裁决依赖、逐条合流、合流后记账。
- 只有一条写会话时，可以直接在主 worktree 做单线工作；准备启动第二条写会话前，先让主树 clean，再把**所有**写会话迁进独立执行轨。并行期间主树只做只读盘点与逐条合流，不改业务文件。

### 执行轨（feature worktree）

- 一个合同只做一个可验收目标；“顺手衍生功能”不得扩进原轨。
- 只改 `owns`；想碰共享文件时提交悬点，由控制轨收口。
- 依赖没 landed 可以继续做不依赖部分，但不能标 `ready`。

### 只读会话

调研、审查、解释代码且不写文件时不需要新 worktree，也不 claim。只读会话一旦要落修改，先转成执行轨。

## 并发门：第二个写者出现时切换姿势

只读调研、审查和值守不计入写会话；一旦它要落文件，就算写者。切到并行模式时：

1. 主树先提交、转存或明确分类现有改动；主树 dirty 时不得再开第二个写者；
2. 每个写者各建 worktree + branch，并分别 claim lane；
3. 主树退回控制轨，直到并行写者归零。

这比“所有项目从第一分钟就必须开 worktree”轻，也比“主会话可以边合边写”清楚。阈值是**同时写的人数**，不是打开了多少聊天窗口。

## 开轨：fresh base、稳定命名、再登记边界

优先用宿主的原生 worktree 工具；Claude Code 常落在 `.claude/worktrees/<lane-id>`，其他工具生成的合法路径同样接受。手工创建时沿用项目在 AGENTS / lock 中声明的 worktree root；未声明才建议 `.worktrees/<lane-id>`，不要把某一家工具目录强制成跨工具标准。

分支统一用 `<type>/<issue-or-lane>-<slug>`，例如 `feat/142-auth-api` 或 `docs/truth-page-history`。**每个新目标都从 fresh `origin/<default>` 长，不从上一任务的 HEAD 长**；squash merge 会换 hash，从旧 HEAD 续开会把已经 landed 的改动重新背进新 PR。

```bash
git fetch origin
git worktree add -b feat/142-auth-api .worktrees/auth-api origin/main
```

若项目声明的 default branch 不是 `main`，替换为实际名字。无法 fetch 时不能声称 base 是 fresh；先报告离线状态，再由人决定是否接受旧 base。

创建后进入**实际路径**登记；registry 不依赖目录名猜 branch 或 lane：

```bash
agent-on worktree claim \
  --id auth-api \
  --goal "完成登录 API 与测试，不改 UI" \
  --base origin/main \
  --owns api/auth \
  --owns tests/auth
```

有前置轨时加 `--depends-on contract-v2`。claim 会拒绝：

- 同一个 worktree 已有合同；
- 与任何 `active|blocked|ready` 轨道的文件域重叠；
- 依赖 ID 不存在；
- `owns` 是仓库根、绝对路径或含 `..` 的模糊边界。

**边界按路径段匹配**：`app` 包含 `app/pages/a.ts`，但不包含 `apple/a.ts`。能拆到文件就别只写大目录；两个目标必须改同一共享文件时，不并行，排队给同一 owner。

## 机械执行层：并行模式一次安装

轨道合同建立后，在仓内任一 worktree 跑一次：

```bash
agent-on worktree hooks install
agent-on worktree hooks status
```

安装器把 `pre-commit` / `pre-push` 放在 common git dir 的 Agent-On 专属目录，并设置仓库级 shared `core.hooksPath`，所以 primary 与所有 linked worktree 同时生效：

- 两个 hook 都跑严格 `worktree check`；未登记、边界重叠、实际 diff 越出 `owns` 或审计 unknown 都 fail-closed；
- `pre-commit` 额外阻断“仍有 `active|blocked|ready` 执行轨时，primary 主树的普通提交”；当 Git 实际调用 `pre-commit` 时，merge / squash-merge / cherry-pick / revert / rebase 控制态通过 git-admin marker 自动放行；
- 成功时静默；失败时打印原因与下一条修复命令；
- 已存在真实 hook 或任何 `core.hooksPath` 时拒绝接管，不覆盖、不绕开；先人工组合后再安装；
- `status` 会验配置与内容漂移；`uninstall` 只移除仍与安装指纹一致的 Agent-On 资产，漂移时整组不动。

Git 自带的人工 `--no-verify` 仍可绕过 Git hook，产品不伪称不可绕过。Claude/Codex plugin 的共用 PreToolUse guard 会在 Agent 发出 `commit/push` 前再跑同一 lane/owns 审计；非 git 与 git 读命令立即放行。Codex 非 managed hook 首次需在 `/hooks` 检查并信任，Agent-On 不改用户 home。

**Git hook 的边界也要诚实**：clean `git merge --no-ff` 走 `pre-merge-commit`，不会调用本版安装的 `pre-commit/pre-push`；所以 clean merge 本身仍须走控制轨合流清单，随后 push 会再过严格闸。若人工用 `--no-verify` 把越界提交写进执行轨，且 lane 的 `base` 错填成会随 merge 移动的本地 `main`，后续审计可能失去稳定对照。`base_sha_at_claim` 是留证，v0.12.1 的边界 diff 仍跟随 `base` ref；合同必须使用 fresh、稳定的 `origin/<default>`，不把逃生口当工作流。

无法安装 shared hook 时，`agent-on worktree check` 仍是手工 fallback；必须保留在提交/合流清单里。回滚：

```bash
agent-on worktree hooks uninstall
```

## 盘点节奏：握手、每日、合流三次都要看

```bash
agent-on worktree status
agent-on worktree status --json
```

它同时报告：未登记 worktree、活跃边界重叠、实际改动越界、依赖未 landed、相对 base 落后、独有 commit、工作区 clean 与回收分类。

- **会话握手 / 转写前**：跑 `status`，确认 cwd、branch、lane 与写者数量；第二个写者出现就触发上面的并发门。
- **每天一次**：手工跑 `agent-on worktree gc --dry-run`；需要低摩擦定时报告时显式执行 `agent-on worktree hooks install --daily-gc`。这是动态盘点，不是删除任务。
- **每次合流后**：远端 read-back，标记 `landed`，再跑一次 `status` + `gc --dry-run`，马上暴露可回收与待抢救项。

写代码前、提交前、合流前跑严格闸：

```bash
agent-on worktree check
```

以下任一成立即非零退出：

- 非主 worktree 未登记；
- 活跃轨道边界重叠；
- 某轨实际变更落在 `owns` 外；
- 活跃记录指向的 worktree 已消失或审计无法完成。

`check` 是 Git hook 与 PreToolUse 共用的底层审计，也可独立运行做诊断。安装器不擅自覆盖用户 hook；冲突未组合前，AGENTS 与派工词必须把手工 `check` 列为提交前命令。

## 衍生需求：分流，不膨胀

执行中发现新功能时只允许三种处理：

1. 与当前验收不可分：回控制轨重切合同，确认旧轨无边界冲突后再继续；
2. 可独立交付：建新 phase / 新 worktree / 新 lane，并用 `depends_on` 表达顺序；
3. 现在不做：记进想法箱或暂停项，不改当前轨。

禁止在一个长寿 worktree 里不断追加第二、第三个目标。**worktree 的寿命应跟一个可合流目标相同，不跟一段聊天或一个大主题相同。**

## 收口状态机

```text
active ──→ blocked ──→ active
   └───────────────→ ready ──→ landed
   └───────────────→ parked
```

- `blocked`：仍占有文件域，防止别人趁停滞抢写。
- `ready`：实现已提交、工作区 clean、边界闸通过，等待控制轨合流。
- `landed`：控制轨已从远端/本地权威确认进入 base；不是执行者说“应该合了”。
- `parked`：停止当前目标并释放文件域，但不代表内容可删；有脏文件或独有 commit 仍是 `rescue`。

更新状态：

```bash
agent-on worktree set-status ready
agent-on worktree set-status blocked
agent-on worktree set-status landed --id auth-api
```

## 合流：一次只进一条

控制轨按下面顺序消费 ready 队列：

1. 先合契约 / 共享文件 owner；
2. 再按 `depends_on` 拓扑顺序，一次只合一条执行轨；
3. 每合一条立即跑合并态全量验证，再处理下一条；
4. 远端 read-back 确认 landed，更新单一状态面；
5. 将 lane 标成 `landed`，再看回收分类。

两个 ready 轨互相有文件域重叠，说明开轨时已经失败；不要靠“到时手工解冲突”掩盖设计错误。

## 回收：分类，不猜

`status` 的 `reclaim` 使用下列保守分类：

| 分类 | 含义 | 动作 |
|---|---|---|
| `safe` | landed 有权威证据、无未保存孤本、clean 且未 locked | 仅表示“已知回收候选”；仍由人精确执行 `git worktree remove <path>`，默认保留分支 |
| `review` | 没有明显孤本，但状态/祖先关系不足以自动判断 | 查 PR / merge 权威后再决定 |
| `rescue` | 脏、越界或有未进入 base 的独有 commit | 先 push / commit / 开 PR，禁止删 |
| `metadata` / `review-missing` | worktree 已不在，只剩合同记录 | landed/parked 可留作本机历史；活跃记录则需调查 |
| `primary` | 这是主 worktree | 永不作为回收目标 |

三条证据必须全中，缺一条就留：

1. **已 landed**：PR / 托管平台状态是权威；`merge-base --is-ancestor` 的“是”可作正证据，“否”遇到 squash 不能证明未合；
2. **无孤本**：没有未推送、也没有未被目标 base 或 MERGED PR 权威覆盖的提交；远端分支不存在、无 upstream、无 PR 都只能判 unknown/review，不能顺手当作 0；
3. **工作区无价值改动为零**：通用安全值是 clean。项目可以另立“生成物/机器噪音”规则辅助人工分类，但 Agent-On 不内置某个项目的假脏白名单，也不会替人认定 dirty 内容无价值。

`locked`、dirty、PR/open、PR/unknown、审计失败或 detached 且归属不明，一律不得进入 `safe`。squash merge 会让祖先关系失真，所以 PR 状态以 `gh pr view` 或托管平台 API 为权威；MERGED PR 还必须确实以当前目标 base 为 base，并由 `headRefOid` 覆盖现有 HEAD，不能拿“合进另一条父分支”的 PR 冒充已进 main。

```bash
agent-on worktree gc --dry-run
agent-on worktree gc --dry-run --json
```

`gc` 是 **report-only**：`--dry-run` 是显式安全门，命令不删除目录或分支。它把每棵树判为 `primary|keep|review|rescue|candidate`；只有 `candidate` 进入 JSON `candidates`。这份当次结果就是“known reclaim list”，会随 git / PR / registry 事实重算，**不得再手填一份常青名单**。需要留档时只写本机 git common dir 或本机日志，不把机器路径和过期判断提交进仓。

可选的每日调度由同一个安装面管理：

```bash
agent-on worktree hooks install --daily-gc
agent-on worktree hooks status
agent-on worktree hooks uninstall
```

macOS 使用用户 LaunchAgent，Linux 使用 systemd user timer，固定每日 03:30；无常驻 daemon。即使从 linked worktree 安装，key、working directory 与 `--repo` 也归一到稳定的 primary worktree，避免功能树回收后定时任务悬空。命令固定为 `worktree gc --dry-run --json`；日志只进用户 state 目录。`uninstall` 保留历史报告，并与 Git hooks 一起先做漂移预检，任一面拿不准则整组不动。

人工拆掉 worktree 后，若不想保留本机合同历史，可清理精确 metadata：

```bash
agent-on worktree forget --id auth-api
```

它只删 common git dir 里的该条 lane JSON，并且仅在状态为 `landed|parked`、对应 worktree 已不存在时放行；不会删目录或分支。

## 自动化与权限边界

- 自动化只许：在 commit/push/PreToolUse 跑边界检查，读取 git / registry / PR 状态，写本机 JSON/日志报告；lane 状态变更仍由显式命令触发，不能由 hook 或 GC 猜。
- 必须人工或获得目标明确的用户授权：删除 worktree 目录、删除本地/远端分支、`--force`、进入别的 worktree add/commit、代另一轨处理 dirty 内容。
- 即使用户笼统说“清一清”，locked、dirty 或 unknown 也不删；先解除占用、逐项分类或抢救，再重新盘点。
- 禁止从一个 worktree 对另一个 worktree 批量 `checkout` / `restore` / `stash`；这不是清理，是跨轨改写。

## 失控时的恢复顺序

已经堆了很多未登记 worktree 时，不要先删：

1. `agent-on worktree status` 列全场；
2. 每棵树查 `status`、独有 commit、对应 PR；
3. 仍要做的逐个 claim；撞边界的只留一个 active，其余 blocked/parked；
4. 无 PR 的孤本获授权后先 push 消单点；
5. 按依赖顺序合流；
6. 只有 `safe` 才拆 worktree，其余保留并写清下一动作。

这套控制面解决的是“谁在改什么、何时能合、能不能删”；任务本身怎么设计、测试和审查，仍由 phase 卡与项目选用的环节 harness 负责。

**机器边界**：lane 控制面在本机 common git dir，不随 clone 同步；换电脑或远程执行机后按现存 worktree 重新 claim。长期决策与验收仍必须进项目仓内的 phase/progress/git，不能只留在 lane JSON。
