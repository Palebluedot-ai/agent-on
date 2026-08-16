# kit/babysit/ 放置草稿（交钥匙件——agent-on 会话审阅后直接落位）

> 配套阅读：`intake/2026-08-16-babysit-merge-dispatcher.md`（提案与实测依据）。
> 本文件 = 两份可直接放置的产物草稿：①通用值守模板 ②项目接入仪式。
> 通用化处理：Dartify 专有内容全部换成 `<占位符>` 或「项目补充区」。

---

## 产物一：`kit/babysit/BABYSIT-TEMPLATE.md`

（项目接入时复制到项目仓 `docs/babysit.md` 并填占位符；用 `/loop <本文档全文>` 启动）

```markdown
# 值守文档（babysit loop）——<项目名>

读本文档：§1 只在首轮做，§2 是循环体。

## §0 GOAL（一句话）
看住 <owner>/<repo> 的 PR 队列、CI、<状态账本机制名>：红了分诊、欠账补账、
授权范围内（§3）的 PR 及时合、拿不准的报告等拍板。
多会话并行是本仓常态——值守只看场子，不抢功能会话的活。

## §1 首轮启动（只做一次，后续轮跳过）
1. 给自己开 worktree（一会话一 worktree 铁律，值守也不例外）。
2. 权限自检：跑一次 `gh pr merge --help` 级别的无害探测确认 allow 规则已配；
   没配则把接入仪式里的 settings 命令贴给用户手跑（agent 改不了自己的权限，机制红线）。
3. 核背景坐标（别信交接文档，自己跑）：main SHA、open PR 列表、上一班声称已完成的关键事实逐条验证。
4. 读规矩原文：<项目治理文档清单，如 CONTRIBUTING/AGENTS 对应章节>。

## §2 每轮检查单（循环体）
1. `git fetch origin -q && gh pr list --repo <owner>/<repo> --state open`
2. 逐个 open PR 跑 `gh pr checks <N>`：
   - 全绿 + §3 可自主类 → 合（追平走服务端 update-branch API，见 §4 首条）
   - 全绿 + 需拍板类 → 报告等用户，别合
   - 红 → 按 §4 分诊；基建红标注即可；非值守造成的红 → 报告不代修
3. 账本巡检：<项目的记账/状态同步机制及其宽限窗；临期由值守补账的操作步骤>
4. 低频（每天一次）：worktree 盘点回收 + `agent-on worktree check` 控制面卫生 + 磁盘余量
5. 节奏：全绿无账可记 = noop tick 放 20–30 分钟；盯活跃 CI 按其时长定唤醒；
   后台 watch+merge 链挂 run_in_background，通知为主信号、唤醒只做兜底

## §3 权限边界（硬，越界前先问）
可自主：只读检查 · 本地 worktree 操作 · 值守自己的状态同步 PR 全流程
       · <项目定义的可自主合并类，如纯状态记账 PR>
       · 值守合并调度下经用户拍板的 PR 全流程（追平→CI→合）
必须先问用户：合并功能/脚本/治理/CI 配置类 PR · 删远端分支 · 数据库迁移
       · 关闭别人的 PR · 清单外一切外向操作
永远不做：进别的 worktree add/commit · `git add -A` · 直推 main
       · 代解语义冲突（真 conflict 打回 PR 作者）· 改自己的权限配置

## §4 分诊手册（先查手册再发明新解释；项目自己的坑往下续）
- merge 报 head 与 base 不同步 → 服务端追平：
  `gh api -X PUT repos/<owner>/<repo>/pulls/<n>/update-branch`，CI 完立刻合
- CI 全 job 数秒死、step 零执行、日志不存在 → org 级 Actions billing 问题；
  查 job annotation 取证，推通知等用户修，每轮最小探针测恢复
- 状态闸脚本拉 GitHub API 抖动（RemoteDisconnected/连败）→ Re-run 即可，非业务违规
- `gh pr checks --watch` 在 push 后数秒视图滞后 → 改盯 `gh run watch <run-id> --exit-status`
- lane 控制面死锁 → claim 拒绝重划就直改 `.git/agent-on/lanes/<id>.json` 再 check 验证；
  OUT-OF-BOUNDS 回填进 owns（check 容忍 parked 轨重叠）；生命周期合法链 parked→ready→landed
- squash 后 `merge-base --is-ancestor` 误判「未并入」→ 以 `gh pr list --state merged` 为准
- <项目补充区：本仓实测过的坑>

## §5 已知遗留（提醒用，值守不抢活）
<交接时的在途事项清单>

## §6 汇报纪律
跟随用户语言；有动作才出声，全绿安静（noop）；
任何「已合/已修/完成」必须贴命令实际输出；拿不准 = 报告而不是猜。
```

---

## 产物二：`kit/babysit/SETUP.md`（项目接入仪式，三步）

```markdown
# 值守接入（每项目一次）

1. **权限**（用户手跑，agent 不能替）：
   ```bash
   mkdir -p <repo>/.claude && cat > <repo>/.claude/settings.local.json <<'EOF'
   {"permissions": {"allow": ["Bash(gh pr merge:*)"]}}
   EOF
   ```
   已有 settings.local.json 则把规则并进 allow 数组。

2. **规矩**：项目治理文档（CONTRIBUTING 或等价物）加「值守合并调度」条款
   （范本 = Dartify PR #163：值守在班时合并统一调度，功能会话开 PR 即交付，
   不自己合；真冲突不代解/授权分级照旧/记账随合并权走；不在班退回原规则）。

3. **启动**：复制 BABYSIT-TEMPLATE.md → 项目 `docs/babysit.md`，填占位符，
   开一条新会话跑 `/loop <docs/babysit.md 全文>`。
   交接换班 = 上一班更新 §1 坐标快照 + §5 遗留清单，下一班核对而非信任。
```

---

## 落位清单（agent-on 会话执行）
1. 上面两份产物分别落 `kit/babysit/BABYSIT-TEMPLATE.md` 与 `kit/babysit/SETUP.md`
2. `boot/adopt.md` / `BOOTSTRAP.md` 各加一句：多会话并行起量时按 kit/babysit/SETUP.md 开值守
3. guard 文档补「值守与 lane 分工」一节（素材在提案文件 §二.2/§二.5）
4. 消化完按惯例处理本 intake 两件（归档/删除随 canonical 惯例）
```
