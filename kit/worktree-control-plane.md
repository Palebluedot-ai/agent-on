# 多会话 Worktree 控制面

> 职责边界：本模式管一台机器上同时存在多个 Claude / Codex 会话与 git worktree 时，如何登记轨道、划文件边界、排依赖、顺序合流和保守回收。它不替你编排任务，不自动 merge，也不自动删除 worktree。

## 一句话

**一个写代码的会话 = 一个 worktree = 一个轨道合同；没有合同就不开工。**

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
- 默认不与执行轨同时改业务文件；需要亲自实现时，也要像普通执行轨一样 claim 明确文件域。

### 执行轨（feature worktree）

- 一个合同只做一个可验收目标；“顺手衍生功能”不得扩进原轨。
- 只改 `owns`；想碰共享文件时提交悬点，由控制轨收口。
- 依赖没 landed 可以继续做不依赖部分，但不能标 `ready`。

### 只读会话

调研、审查、解释代码且不写文件时不需要新 worktree，也不 claim。只读会话一旦要落修改，先转成执行轨。

## 开轨：先切边界，再开会话

先用普通 git 建 worktree，然后在那个 worktree 中登记：

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

## 日常对表：一条命令看全场

```bash
agent-on worktree status
agent-on worktree status --json
```

它同时报告：未登记 worktree、活跃边界重叠、实际改动越界、依赖未 landed、相对 base 落后、独有 commit、工作区 clean 与回收分类。

写代码前、提交前、合流前跑严格闸：

```bash
agent-on worktree check
```

以下任一成立即非零退出：

- 非主 worktree 未登记；
- 活跃轨道边界重叠；
- 某轨实际变更落在 `owns` 外；
- 活跃记录指向的 worktree 已消失或审计无法完成。

`check` 是本地机械闸，可接入项目已有 pre-commit / pre-push，但模板不擅自覆盖用户 hook。没有接 hook 时，AGENTS 与派工词必须把它列为提交前命令。

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

`status` 的 `reclaim` 只有四类：

| 分类 | 含义 | 动作 |
|---|---|---|
| `safe` | 已标 landed、clean，且 HEAD 是 base 祖先 | 可人工 `git worktree remove <path>`；默认保留分支 |
| `review` | 没有明显孤本，但状态/祖先关系不足以自动判断 | 查 PR / merge 权威后再决定 |
| `rescue` | 脏、越界或有未进入 base 的独有 commit | 先 push / commit / 开 PR，禁止删 |
| `metadata` / `review-missing` | worktree 已不在，只剩合同记录 | landed/parked 可留作本机历史；活跃记录则需调查 |
| `primary` | 这是主 worktree | 永不作为回收目标 |

squash merge 会让“HEAD 是否为 base 祖先”失真，所以 `safe` 判据刻意保守；PR 状态以 `gh pr view` 或托管平台 API 为权威。CLI 本轮**不提供自动删除命令**，避免唯一副本被误收。

人工拆掉 worktree 后，若不想保留本机合同历史，可清理精确 metadata：

```bash
agent-on worktree forget --id auth-api
```

它只删 common git dir 里的该条 lane JSON，并且仅在状态为 `landed|parked`、对应 worktree 已不存在时放行；不会删目录或分支。

## 失控时的恢复顺序

已经堆了很多未登记 worktree 时，不要先删：

1. `agent-on worktree status` 列全场；
2. 每棵树查 `status`、独有 commit、对应 PR；
3. 仍要做的逐个 claim；撞边界的只留一个 active，其余 blocked/parked；
4. 无 PR 的孤本先 push 消单点；
5. 按依赖顺序合流；
6. 只有 `safe` 才拆 worktree，其余保留并写清下一动作。

这套控制面解决的是“谁在改什么、何时能合、能不能删”；任务本身怎么设计、测试和审查，仍由 phase 卡与项目选用的环节 harness 负责。

**机器边界**：lane 控制面在本机 common git dir，不随 clone 同步；换电脑或远程执行机后按现存 worktree 重新 claim。长期决策与验收仍必须进项目仓内的 phase/progress/git，不能只留在 lane JSON。
