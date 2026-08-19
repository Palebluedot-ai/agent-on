# 值守接入（每项目一次，三步）

> 什么时候需要：多写会话并行成常态、PR 排队等合、或分支保护开了 up-to-date 硬门——「合并」已经变成全局串行资源。单会话仓不需要值守，别提前装。

## 第 1 步：权限（用户手跑，agent 不能替）

```bash
mkdir -p <项目路径>/.claude && cat > <项目路径>/.claude/settings.local.json <<'EOF'
{"permissions": {"allow": [
  "Bash(gh pr merge:*)",
  "Bash(gh api -X PUT repos/*)"
]}}
EOF
```

- 已有 `settings.local.json` 则把两条规则并进 allow 数组，别整文件覆盖。
- 这两条是实测「时好时坏被拦」的最小集（merge 与服务端 update-branch）；`gh pr view/checks/list`、普通 `git push` 一般无需放行。
- 原则：**被拦哪条补哪条，只放具体动词短语，不放 `gh:*` 全放行**。
- 机制红线：agent 改不了自己的权限配置——Skill（update-config）、Bash 读写、Write 直写三种模态实测全被 auto-mode 分类器拦（拒绝语一致），必须用户手跑。手跑完成后 `gh pr merge` 立即放行（实测输出 "=== #153 MERGED ==="）。

## 第 2 步：规矩（治理条款）

项目治理文档（CONTRIBUTING 或等价物）加「值守合并调度」条款：照抄 [CONTRIBUTING-CLAUSE.md](CONTRIBUTING-CLAUSE.md) 填空。核心一句：**值守在班时合并统一调度；功能会话开完 PR、首轮 CI 触发、描述写全、交单 = 交付完成，不自己合**。

## 第 3 步：启动

1. 复制 [BABYSIT-TEMPLATE.md](BABYSIT-TEMPLATE.md) → 项目仓 `docs/babysit.md`，填掉全部 `<占位符>`（repo 坐标、账本机制、§3 授权分级、治理文档清单、CI 工作流名、合并方式）。
2. 新开一条干净会话：`/loop 读 docs/babysit.md 全文并执行本轮值守`（固定节奏就 `/loop 5m …`；高活跃 5 分钟，常态 10 分钟或动态自定）。
3. 上岗登记（装了 agent-on CLI 的仓）：`agent-on oncall claim --session <本窗口会话名>`——功能窗口靠它拿交单地址，路由闸靠它判「谁不是值守」。下班 `agent-on oncall release`。**不登记 = 路由闸整条 fail-open**（[ROUTING §4](ROUTING.md)）。
4. 保持窗口开着 = 在班。会话自己会走模板 §1 自检坐标，然后进循环。

## 角色分工（速查）

| 角色 | 干什么 | 不干什么 |
|---|---|---|
| 值守会话（唯一） | 巡检队列 · 分诊 CI · 服务端追平 · 请拍板 · 执行 merge · 记账 · 回执 · **一切对外通信** · **窗口之间传话与派工** | 改功能分支本地文件 · 代解冲突 · 代修缺陷 · 改自己权限 |
| 功能会话（可多个） | 开发 · 提交 · 开 PR · SendMessage 交单（模板见治理条款；收件地址跑 `agent-on oncall status`，没装 CLI 才读 `docs/babysit.md` 交接快照「在班值守地址」）· 把发错窗口的指令转投值守（[ROUTING §3](ROUTING.md)）· 收回执后清理本地 | merge · 对外通信（PR 评论 / Teams / 邮件 / webhook）· 横向找别的功能窗口说话 · 为追平 rebase main 后强推 · 直推受保护分支 |
| 用户 | 拍板（值守给足材料：PR 号/CI/冲突/影响面）· 修 billing 类只有管理员能修的事故 | — |

## 换班

- **下班**：旧班按模板 §7 做完三件（交接快照 / 遗留清单 / 新坑入 §4，commit 文档）→ `agent-on oncall release` → 关窗口。
- **接班**：新开会话，`agent-on oncall claim --session <新会话名>`（旧班没下班就 `--force` 接管），重新 `/loop` 同一份 `docs/babysit.md`。新班核对坐标而非信任交接。
- 同一时间至多一个值守窗口；文档是唯一持久资产，窗口只是班次。
- **忘了 release 就关窗口**：登记的 worktree 还在则功能窗口会被继续挡——任何窗口可跑 `agent-on oncall release --force` 清理（留痕，不是绕闸）；worktree 也没了则闸自动按「无人在班」处理。
