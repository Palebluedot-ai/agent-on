# 案例 40:出口在权限之外的闸 = 死锁(连坐 + 互锁 FAIL 对)

> 层级:L2 | 来源:某功能窗口 2026-08-20 现场 + agent-on 仓内复现 | 入册:2026-08-20

## 症状

worktree 边界闸全场 FAIL。现场五条 lane 各带约 90 个脏文件,另有两棵陈年树(落后 165 / 落后 124)死了很久没人清。会话按 `kit/worktree-control-plane.md`「死锁三解」照做,把 OUT-OF-BOUNDS 清单回填进 owns——回填立刻造出大量 OVERLAP,而 OVERLAP 同样是 FAIL 条件。

报错文案本身是合格工单(写清了哪些文件出界、下一条命令是什么),会话照单开工,然后撞进另一条 FAIL。最后判断真解是删陈年树——**破坏性动作,auto-mode 硬墙拦,必须用户拍板**。闸把所有人拦住了,而唯一的出口不在被拦者的权限里。

## 根因

**互锁 FAIL 对 + 出口全在权限之外。** 仓内实测(scratch 仓两棵陈年脏树,同碰一个共享文件):

| 想走的出口 | 结果 |
|---|---|
| 占位 park(文档解 ③) | 脏树 park 后仍被判 still-writing,边界不释放 → `OUT-OF-BOUNDS` |
| 回填 owns 走正门 `worktree edit` | 第二条轨被入口闸拒:`overlaps still-writing lane a` |
| 回填 owns 走文档明说可用的 JSON 直改 | 绕过入口闸 → `OVERLAP` |
| 缩回 owns 解 OVERLAP | 打回 `OUT-OF-BOUNDS` |
| `set-status parked → ready` / `→ landed` | 两条都 `invalid lane transition`(转移图里根本没有 `parked→ready`,与文档写的合法链矛盾) |
| `edit --status ready`(明说绕转移图) | `ready requires a clean worktree`——守卫另拦一道 |
| `edit --status landed`(同上) | **记录写成功且没有任何干净树守卫**,但边界照样不释放:check 仍报 `OUT-OF-BOUNDS`/FAIL。等于只多了一条假账 |
| 清空脏文件 / 删 worktree | 唯二真出口,**全是破坏性,全在 Agent 权限之外** |

两个 FAIL 条件互为对方的唯一解,可行域为空。叠加三件事才炸:①连坐(一棵坏树 → 全场 FAIL)②文档把「对干净树成立」的解法当通解写给了脏树(`park` 只是登记,互斥闸判的是事实——脏或有独有 commit 会把 parked 轨拉回互斥集)③陈年树只进不出(`gc` 永远 report-only,删除永远人工,没有常设清账议程)。

## 修法

现场:先别按文档回填——**回填是把一条 FAIL 换成另一条**。把陈年树按债务处理(登记 + 挂进值守议程),清理走用户拍板;自己的活换非阻塞路径推进。

制度:见「可复用规则」。根治在机制层——让化石轨(出了窗口、没人在写)退出互斥闸,闸才恢复信号。

## 可复用规则

**闸的出口必须走得通,不只说得清。** 每个 FAIL 条件至少要有一条出口,落在被拦者当下的权限内且非破坏性;出口全是破坏性动作或需要更高权限 = 不是闸是死锁。写闸自检三问:①谁被拦、他有什么权限 ②照报错做完能不能到绿(**实际跑一遍**)③两个 FAIL 条件会不会互为对方的解(互锁 FAIL 对必须设计期消除)。**连坐闸另配一条:清理成本必须落在有权限的常设角色(值守/控制轨)身上**,不能随机砸给下一个撞上它的会话。

## 已固化到哪

multi-contributor §三½.5 出口面(4A 自解释 + 4B 可达性 + 自检三问 + 连坐成本归属);worktree-gc-pattern「陈年树是债务」;merge-checklist 0b;babysit 模板每轮盘点与分诊手册;snapshot/2026-08-20-gate-exit-reachability.md。
