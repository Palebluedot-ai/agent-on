# 2026-10-03 · v0.12.1 Worktree goal 当前验收审计

> 职责边界：逐项检查旧 goal 与现行源码、实际安装及运行证据；不缩改 goal、不恢复已被用户撤回的强制流程，不授予新版发布或全局配置改动权限。

前一轮为实际进展：新增本机控制面与回归。本轮也有实际进展：重新验证执行入口，补真实漏检，恢复一个已安装的只读报告任务。不是以状态复述代替工作。

## 当前权威与冲突

旧目标要求主树业务提交禁令和 lane/owns 的 PreToolUse 强制检查。现行 AGENTS 第 7 条及 10-02 核心清单要求 lane 可选、Git 仅核实际 index/待推路径，PreToolUse 只保留跨仓和路由授权。本轮保持后来的用户决定，**这不等于旧 goal 原样完成**。旧 v0.12.1 已有 annotated tag（878206b，2026-08-16），当前已发布基线为 v0.26.0/0c6e5dc，不另造第二个 v0.12.1 tag。

## 要求与证据逐项对照

| 原要求 | 当前证据 | 判定 |
|---|---|---|
| 一键 install/status/uninstall，共享 pre-commit/pre-push | 实盘 `core.hooksPath=.git/agent-on/hooks`，两个 hook active；integration 验一次安装覆盖 primary/linked、幂等卸载、配置漂移保护 | 已实现；不为验收卸载真实保护 |
| 共享 hooks 自动 check | 真实 Git 回归读有效 index 和逐提交推送范围，冲突被拦；只读盘点与操作过滤共用审计结果 | 原严格全树 check 后被改为实际操作范围；不冒称旧判据仍在 |
| 有活跃执行轨就禁止主树业务提交 | 当前真实回归明确允许不撞其他写者的主树提交 | 与后续决定相反，未按旧条款达成 |
| Claude/Codex PreToolUse lane/owns 边界 | 当前只管跨仓与值守路由；lane 不决定 commit | 与后续决定相反，未按旧条款达成 |
| Claude/Codex 接线及真实拦截 | Claude v0.26 插件 enabled，另有用户 Bash hook；Codex 用户级 command hook 指向 canonical shim，不是已安装插件。新增黑盒执行同一真实 shim，Claude/Codex 写 payload exit 2，读 exit 0 | 接线存在、入口行为已验；未做本轮认证模型驱动的宿主工具调用/信任验证 |
| 可选每日 report-only GC，有真实调度 | 原 launchd 登记存在但 last exit=78。只重载同一份 plist 后 kickstart，实际 last exit=0；最新 JSON 为 dry-run、6 trees、0 candidates、errors=[] | 既有 macOS 现场已恢复；未启用删除；Linux 实机未验 |
| 失败不 silent fail，状态明确、指引可执行 | 新源码对已登记但失败返回 RunFailed/非零；查询失败也非零；损坏 hook JSON 返回错误指引/exit 2；未知/运行中不冒称已成功；已知运行失败可按匹配内容重载；失败仍可卸载 | 新覆盖的场景已验，尚未全局安装；不是任意外部失败皆已穷举 |
| 可卸载，既有 hook/配置不被覆盖，保留日志 | 漂移预检、旧路径/仓移动、外部 hook、PATH/executor 改变的回归继续通过；新失败用例也能 uninstall | 已实现，真实配置/日志本轮保留 |
| 只在高杠杆节点检查，不逐文件强制审计 | 提交/推送范围闸，普通文件编辑不触发 Git 闸；PreToolUse 与可选捕获分工 | 现行规则成立；旧“其他路径零开销”不能以 allow=0 证明，值守心跳还有后续明确约定 |
| 安装/心智/排障成本显著低于直接管理两家 hooks | 共享安装、诊断与自动恢复有机制证据；但用户级 hook 与插件重复入口仍存在 | 尚无同任务对照和下游使用反馈，不能声称成本验收通过 |
| 不引入运行时 daemon、不每次编辑扫描 | 原生 Git hooks + OS 已有定时器；新面板是用户显式启动的可关闭服务，不是 hooks 的必需 daemon | hooks 主路径成立；后续可选派工/面板属于新需求 |
| 不自动删除任何树 | 本轮未启用 janitor，既有 GC 命令固定 dry-run；所有旧树保留 | 本轮满足；10-01 用户另行授权的可选有限 janitor 不被改回删除型旧 GC |
| 一次安装保护即生效，日常几乎无感 | 临时 Git 实测无需 claim：真实冲突 commit/push 被拦；--only 无关路径提交通过；对方清理后 push 通过 | 可证明范围行为，不能从此推导整体人类使用成本 |

## 本轮真实漏检与修正

1. 新增可选外部 executor 黑盒用例：`AGENT_ON_ACCEPTANCE_EXECUTOR=/Users/chao/.cargo/bin/agent-on cargo test --test worktree_hooks blackbox_executor -- --nocapture`。已安装 v0.26.0 的 native commit/push 范围拦截正常，但 Codex `cwd=外部会话 / tool_input.workdir=B / cmd=git commit` 没拦（exit 0）。新源码分离执行位置与会话身份，完整黑盒通过：真实 commit/push block、无关 --only allow、修复后 push allow、两家 shim write=2/read=0。不是测试改期望放过漏洞。
2. 定时注册不等于执行成功。`launchctl print` 最初 runs=9/last exit 78；kickstart 复现 runs=10/exit 78，系统日志明确 `Unable to get updated LWCR ... error 0x3`，GC 尚未启动。只对该已登记且未运行的任务 bootout/bootstrap 原 plist，随后 kickstart 实际 runs=1/exit 0。没有修改 plist、binary、03:30 时间或 GC dry-run 参数，既有日志保留。
3. 新状态检测覆盖 launchd exit/signal、systemd service Result/ExecMainStatus、未知和运行中；安装错误不阻止安全卸载。Linux 属性依据 [systemd 官方接口说明](https://wiki.freedesktop.org/www/Software/systemd/dbus/)，隔离替身/解析测试不等于 Linux 实机调度验收。
4. 损坏 JSON、JSON 数组/标量和 stdin 读取失败不再被 guard 当作 allow，给宿主 wiring 排障指引并 exit 2。空输入仍兼容 shim 的人工探针，不把没有命令的探针伪称写操作；黑盒通过真实 shim 验截断 JSON 被拒。

## 验证与未完成边界

- hook / dormant / owns / exits 定向回归：31 项 passed（追加前）；完整本批回归新增 5 项后：324 passed / 0 failed；clippy -D warnings、fmt check、diff check exit 0。含本批三个新 Markdown 的文档闸 226 份 PASS。
- 可重复黑盒入口：`cargo test --test worktree_hooks blackbox_executor -- --nocapture`。测试只用临时 Git 仓和本地 bare remote，不对 origin/main 推送、不改真实项目 hook 配置。
- 程序源码与现场安装严格分开：候选 debug executor 已含修正，全局 CLI、Claude release/cache 与 Codex 用户 shim 实际调用的 release 仍 v0.26.0。本轮没有将未发布源码灌进全局缓存。
- 发布批准、新版实际执行器生效、认证宿主驱动的工具调用、成本对照仍未完成；旧强制条款与后续规则的冲突也不能靠绿色测试抹平。**goal 继续 active，不调用 complete。**

本轮新增源码与上一轮面板一同保留待审，拟 v0.27.0。按权限硬停等待该批合入/发布批准，不把旧版本 push 授权扩展为新版批准。

## 后续发布批准（2026-10-03）

用户随后明确批准 v0.27.0 发布到 main，并原子推送 main＋annotated tag，解决本批发布授权这一项。以上表格与未发布现场证据保留原时间边界；全局执行器/插件升级、认证宿主工具调用与成本对照仍不在本次批准范围，不能把发版等同旧 goal 全项达成。
