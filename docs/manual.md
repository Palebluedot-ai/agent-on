# agent-on 使用手册（参考篇）

> 职责边界：本文件收 README 放不下的**参考细节**——多会话 / worktree 控制面怎么用、路径与换机、按版本的演进记录。产品是什么、为什么用、怎么装、日常口令，看 [README](../README.md)；判据与机制的权威仍在各自的 kit / playbook 正文，本页与它们冲突时以正文为准。

## 一、多会话并行：worktree 控制面与合流

同时开了多个写代码会话 / worktree 时，先让每条执行轨登记互斥文件域，再看全场：

```bash
# 在各 feature worktree 内登记一次
agent-on worktree claim --id auth-api --goal "登录 API" --base origin/main --owns api/auth --owns tests/auth

# 任意 worktree 查看全场；提交/合流前用严格闸
agent-on worktree status
agent-on worktree check

# 并行模式每个仓只装一次；shared hooks 覆盖全部 linked worktree
agent-on worktree hooks install
agent-on worktree hooks status

# 可选：同时安装每日 03:30 的只读回收报告
agent-on worktree hooks install --daily-gc

# 手工加跑只读回收盘点
agent-on worktree gc --dry-run

# 合流协调面：refresh 一次批量取证（唯一联网命令），之后离线看队列与波次
agent-on landing refresh
agent-on landing status
agent-on landing plan
```

一次 `hooks install` 把 `pre-commit` / `pre-push` 放进 common git dir、设成仓库级 shared `core.hooksPath`，primary 与所有 linked worktree 同时生效。两个 hook 都只判**本树**，会拦提交的只有一条：**本 worktree 的某个未提交文件（staged / unstaged / untracked），在另一棵 worktree 里也是未提交的，而且那一份在 7 天内被人碰过**——人读输出一行 `blocked: <路径> is also uncommitted in <另一棵树>`；本树审计跑不起来也拦（`error`）；没撞上，hook 静默，`status` / `check` 打 `ok`。pre-push 另判**推上去的是什么**：把 `origin/<default>` 并进来的本地 merge commit（committer ≠ GitHub、merge 干净）推向开着同仓、进默认分支的 PR 的分支，拦下并给出完整的 `gh api -X PUT …/pulls/<N>/update-branch`；有冲突的 merge、没有这样 PR 的分支都放行（playbook `multi-contributor-protocol.md` §三½.8）。lane 登记只给 `claim` / `edit` 用，commit / push 不读它：`UNREGISTERED`、`OVERLAP`、`OUT-OF-BOUNDS`、`MISSING` 只留在 `--json` 里，不挡 commit；主 worktree 与没登记的树都和别人一样按这一条判。merge / squash-merge / cherry-pick / revert / rebase 控制态自动放行；clean `git merge --no-ff` 走 `pre-merge-commit`，不调用这两个 hook，所以 clean merge 本身仍须走控制轨合流清单。Claude/Codex plugin 的 PreToolUse guard 在 Agent 发出 `commit/push` 前跑同一条判据（本地 merge 那条只在 Git pre-push 里：它要 git 给的待推范围）。可选调度只执行 `gc --dry-run --json`，输出动态 `candidates`，**不自动删**。判据与三条设计约束见 [kit/worktree-control-plane.md](../kit/worktree-control-plane.md)「闸只拦真冲突」。

多 PR 并行时，`landing` 三条命令是合流协调面：所有检查结果绑定 `(PR head SHA, base SHA)`，两者未变直接 SKIP 复用；main 每合入一条只重查有依赖边或文件重叠的 PR。`status` 首页只给五个数（现在做 / 下一批 / 等待中 / 需抢救 / 可回收），全部 worktree 自动落进 ACTIVE/WAITING/PARKED/RESCUE/REAPABLE 五类之一；活跃轨有上限（默认 3，`--parked` 排队不占额）。v1 严格只读：不驻后台、不自动 merge、不自动删树。完整数据模型与分类规则见 [kit/landing-control-plane.md](../kit/landing-control-plane.md)。

## 二、路径、远程安装与换机

**Q：换电脑 / 别人路径不同 / Windows 怎么办？**

**不要求**固定文件夹名（`Projects`、`Agent-On` 都不是产品默认）。分两面：

| 面 | 是什么 | 怎么来 |
|---|---|---|
| **A 运行包** | 读模板 / skill / hook 脚本 | `plugin install` 即可；路径由 Claude 注入 `CLAUDE_PLUGIN_ROOT` |
| **B 工作仓** | 结账写 intake、消化改方法论 | `git clone` 到**任意路径**后登记：`AGENT_ON_ROOT` 或 `~/.config/agent-on/config.json` 的 `work_root` 或项目 lock「本地路径」 |

只 init/adopt 的用户可以**只有 A、没有 B**。settle/digest 前必须有 B，否则 skill 拒绝并提示登记。自检：`/agent-on doctor` 或 `agent-on doctor`。

**Q：远程（非本机 path）怎么装 Claude？**

```bash
claude plugin marketplace add Palebluedot-ai/agent-on
claude plugin install agent-on@agent-on
# 若要结账/消化：
git clone git@github.com:Palebluedot-ai/agent-on.git /anywhere/you/like
echo '{"work_root":"/anywhere/you/like"}' > ~/.config/agent-on/config.json
```
Windows 同样：clone 到如 `D:\dev\agent-on`，`work_root` 填该绝对路径。不是 npm。

**Q：这台电脑坏了，怎么恢复？**

| 路 | 步骤 |
|---|---|
| **A · Plugin（推荐）** | ① GitHub marketplace add + install ② 需要 B 则 clone 任意路径并写 config ③ 全局口令路由随 agent-memory（个人） |
| **B · symlink（兼容）** | clone 任意路径 → `ln -s <仓>/skill ~/.claude/skills/agent-on`（及 Codex `~/.agents/skills`） |

Claude / Codex guard 都随 plugin 挂并共用一份 hook；Codex 非 managed hook 首次在 `/hooks` 检查并信任。项目 lock / loop-notes 在项目仓里，跟项目走。

## 三、版本演进

逐版细节与证据以 [CHANGELOG.md](../CHANGELOG.md) 为准（git tag 即版本）；下面是按里程碑的一行摘要。

- **v0.2**：五块骨架、三代资产合流、Bench 案例集、迭代闭环六站机制、S/M/L 档位路由、存量项目接入书
- **v0.3 ✅ 达成（2026-07-09）**：Euan 倒仓首次结账 + 首次消化跑通，闭环真转过一圈，已封 `v0.3.0`（冻结令随之解除）
- **v0.4 ✅ 达成（2026-07-15）**：AInvestment 完成 BOOTSTRAP 全流程 dogfood + 两默认件验证，已封 `v0.4.0`。超预期交付：规划链 §1.5（MRD→PRD→phase 卡路由）、强制约束层（agent-on-git-guard，双工具 PreToolUse 机械拦截跨仓越界）、项目端零 git 边界、防幻觉第六型
- **v0.5 ✅**：`v0.5.0` Plugin/路径/贡献；**`v0.5.1`** 默认目录 `setup.py` + 三工具装机文档（patch）。诚实边界：Codex plugin hook 未接线（#16430）。
- **v0.6 ✅**：**`v0.6.0`–`v0.6.3`** 攒批/tag 硬门/轻主路径/降档/Superpowers 退出默认。  
- **v0.7 ✅**：可执行面 Rust 化（`agent-on` CLI；主树无 Python 交付脚本）。
- **v0.8 ✅**：记账棘轮 / worktree 回收模式（`v0.8.0`–`v0.8.3`）。
- **v0.9 ✅**：**`v0.9.1`** 交付前 worktree 对表 + 交付链先于环境 + 闸拒命令字面（Dartify 真机）。
- **v0.10 ✅**：**`v0.10.1`** 多会话 worktree 控制面（轨道合同 + 文件边界/依赖/合流/保守回收 + 口令/adopt）。
- **v0.11 ✅**：**`v0.11.0`** 第二十二次消化：闸的三张面 + 运行面验收 + 口令/斜杠调用面。
- **v0.12 ✅**：**`v0.12.1`** worktree 生命周期与执行强制层（只读 GC + shared Git hooks + Claude/Codex PreToolUse）。
- **v0.13 ✅**：**`v0.13.0`** Landing 控制面 v1（合流协调三命令 + SHA 绑定证据缓存 + 六类合流表 + 五类生命周期 + 活跃轨上限）。
- **v0.14 ✅**：**`v0.14.0`** 值守合并调度 babysit（kit/babysit 四件：模板 §0–§7 / 三步接入 / 治理条款范本；landing 的执行半场）。
- **v0.15 ✅**：**`v0.15.0`** 值守两批消化收口（协作篇 §三½.6 值守段 + 记账字面匹配盲区 + 闸四张面 + worktree 重划死锁三解 + bench 37/38 + anti-hallucination #17/#19）+ Deep Research 派工模板 + babysit 交单三型协议 + 本仓值守自举（docs/babysit.md + AGENTS 第 8 条）。
- **v0.16 ✅**：**`v0.16.1`** 契约层收口（`kit/output-contract.md` 每轮输出契约 + `kit/babysit/MERGE-POLICY.md` 合入授权/门铃即起跑/时延目标）+ CLI 两件（`worktree edit` 重划 lane、`claim --owns` 逗号串修复）+ 真相之页「开发史」tab；`v0.16.1` 另补本仓值守文档接契约与推荐 pin 文案（patch）。
- **v0.17 ✅**：**`v0.17.0`** 跨窗口值守调研（interactive 会话没有状态字段 / 缺口在强制点与状态可读性）+ 输出契约四处增补（表格渲染映射 / 末尾 Summary 块 / 跨窗口编号 `<会话名>#<任务 id>` / 默认值默认等于建议值）。
- **v0.18 ✅**：**`v0.18.0`** 跨窗口指令路由（三权唯一：合并 / 对外通信 / 跨窗口中转；`agent-on oncall` 五命令在班登记 + PreToolUse 路由闸，无人在班 fail-open；误投一律【转投】不执行）+ `kit/babysit/ROUTING.md` 与 AGENTS 自举纪律 9。
- **v0.19 ✅**：**`v0.19.0`** 输出契约三轮加固（不许有第三个筐 / 拍板六件含「在哪拍」/ 一句话全批 / 指路报窗口标题）+ Gen-1 角色体系归档进 `legacy/` 与元原则第七条「角色不是架构原语」+ 互斥 owns 闸按事实判（复用旧树的 landed 轨不再关掉边界检查）+ 外向硬门重划（push 自己的分支与开 PR 不在内）。
- **v0.20 ✅**：**`v0.20.0`** 边界闸只拦一件事（本树未提交改动进别人活轨 owns 才拦，不连坐；UNREGISTERED / OUT-OF-BOUNDS / MISSING 降为提示）+ 值守登记带心跳（90 分钟没心跳自动失效，窗口关了不锁全场）+ 边界闸三档分层 + 出口面可达性 + 常驻预授权 + 值守全自动合并与独立审计（`tools/merge-audit/`）。
- **v0.21 ✅**：**`v0.21.0`** 投影漂移：「原件 / 投影」立成真相源治理的第一类区分 + `agent-on drift` 一条命令对账台账与文档两条介质上的投影（默认只报不拦）+ bench 案 43。
- **v0.22 ✅**：**`v0.22.0`** commit 闸不再读 lane 登记：只在另一棵工作树 7 天内也改过同一个未提交文件时拦截；`worktree status` / `check` 平时一行 `ok`。
- **v0.23 ✅**：**`v0.23.0`** 消化 5 份 intake / 38 卡：收编 09-21 那场没提交的消化（23 个预写的 `landed@v0.22.0` 逐卡改正）；闸误拦族升 L3（判据面 + 执行面——跨仓误拦实出自插件缓存旧闸）；消化开场四检（主树自证）+ 去向标注禁止预写版本号；CLI：值守路由闸只认命令位置、intake-lint 认不出卡不再报通过、tag-release 拒绝预写版本号；仪表盘求值闸 `kit/dashboard-check.mjs`；案例 46–50。**`v0.23.1`** 发版推送改一条原子推送（v0.23.0 分两次推，CI 推荐 pin 闸在 tag 到达前 checkout 红了一次）。**`v0.23.2`** hooks 不再把与共享路径逐字节相同的 worktree-scope core.hooksPath 误判为绕闸漂移（宿主每开 session worktree 复制一份，曾挡住 `install --daily-gc`）；install/uninstall 顺手归一化，值不同仍 fail-closed。**`v0.23.3`** README 多会话段改掉连坐旧说法：`hooks install` 那段照 kit「闸只拦真冲突」重写（会拦提交的只有一条，未登记 / 越界 / 主 worktree 不再被挡）。
- **v0.24 ✅**：**`v0.24.0`** CLI 轨收掉 09-26 消化的 deferred：`agent-on doctor` 报 hook 执行面（插件版本、hooks.json、脚本字节对 READ_ROOT，再追到 shim 最终跑的二进制——本机实测插件缓存连 `target/` 一起拷，闸修复没到执行面）与「当前在 linked worktree」；pre-push 拦本地 merge `origin/<default>` 进已开 PR 的分支（有冲突 / 没 PR 放行，文案给完整 update-branch 命令）；同文件闸报对方 rebase 进度；`tag-release --push` 一条原子推送。 **`v0.24.1`** kit 控制面页改掉同页五处旧闸说法（PreToolUse、clean merge 之后的 push、人读 `status`、`check` 非零条件、`RESCUE-DEBT`），照「闸只拦真冲突」重写，判据与代码不动。**`v0.24.2`** CI 文档闸的推荐 pin 判据只认声明写法（版本号紧跟标签、每个文件的声明处数钉死），散文里提到推荐 pin 不再被读成第二个 pin——v0.23.3 发版因此红过、当时靠改散文绕开，本版把那句改回原话。**`v0.24.3`** 一条 lane 的 `base` 解析不了时，git 的 `fatal:` 不再印到每棵树的 `status` / `check`、hook 与 guard 输出上（祖先判定改静默，布尔语义不变；那条 lane 自己的树照旧报 `error:`）。
- **v0.25 ✅**：**`v0.25.0`** `agent-on tag-release --push` 从任何分支、任何 worktree 都推 origin 的默认分支（认 `origin/HEAD`，失效时退回 main / master），不再把分支按原名推上去、让 tag 落在 main 之外；推送被拒撤掉刚打的本地 tag（tag 跨 worktree 共享，留着别的会话会从它往下数版本号）；值守路由闸把它归「合并」类。 **`v0.25.1`** README 重写成面向新用户的中文首页，参考细节移入本页（docs/manual.md）。**`v0.25.2`** 补 MIT LICENSE 文件。
- **v1.0 定义已入 snapshot**，未达标：见 [snapshot/2026-07-16-v10-and-setup.md](../snapshot/2026-07-16-v10-and-setup.md)
- **v1.0（未达标）**：≥2 项目有外人装机开工 + ≥1 次结账进官方 intake 并经消化落地（详见上列 snapshot）——不是「感觉上很多人用」
