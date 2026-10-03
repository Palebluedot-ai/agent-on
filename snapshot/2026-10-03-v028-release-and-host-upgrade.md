# 2026-10-03 · v0.28.0 发版与本机执行面收口

> 职责边界：记录本开发窗口的用户批准、版本批、复核和本机升级范围；不是值守上岗、开启巡逻/清道夫、云端发布或删除 worktree 的授权。具体接续接口见 kit/session-control-plane.md。

## 批准与交接

用户先确认两个窗口共用项目并要求以「设计多 worktree 管理机制」为主；本窗口读取「拉取最新状态」的最新交付和实际文件，后者空闲。双方共用 /Users/chao/Projects/Agent-On，文件/Git 状态共享，聊天历史不自动同步。随后用户明确回复「批准，请继续」，承接上文“新一轮发版和本机 CLI/插件升级仍需单独批准”。本批据此发布并统一升级实际执行器，不把该批准扩张为云端功能或仓库清理。

接手 HEAD/main/origin/main 为 v0.27.0 / ab7bb283f306c8a3dc8e283bedfddfbcd8196d8e。fetch 后远端未推进；本仓无值守登记。本批复用已有未提交实际会话身份/状态接续代码，无新 worktree、无重复实现。

## 独立复核与补漏

- 三个只读子代理复核覆盖/计划、安全、维护性/性能；追加红队复核。未声称跨模型评审。
- 找到损坏登记仅拦 shell 合并却放行已绑定 peer 消息；whoami 同根因冒称无人。两项回归先红（预期 2/1，实际 0），再修明确不可用分支；真正内部消息保留。
- 插件此前仅 Bash 命令 matcher，补原生 Codex exec_command 接线及形状测试；安装后仍需实机证明宿主派发，测试 JSON 不能代替。
- 补非法 host/session ID、任务摘要上限五项与已释放过滤、损坏任务、Claude 环境文件写失败。无新增依赖或数据库。
- 专项 30 次只读性能样本：普通无值守 guard 中位 9.5ms → 7.1ms；SessionStart 16.6ms → 36.5ms（最大 40.9ms、约 260 字符、巡逻关闭未落 recorder）。这是本机小样本，不等于真实开发总成本/协作收益对照。
- 子代理首次全量测试撞到共享 debug 二进制替换；目标测试重跑通过。最终全量只由主窗口顺序构建，避免把测试基础设施竞态当代码绿。

## 安装计划与回退

先审计实际 executor、版本/源码身份、插件缓存和宿主配置。现有全局 CLI/Claude 缓存为 v0.26.0，仓内 release 为 v0.27.0；Codex 个人仅 PreToolUse 与 Stop，不能提供 SessionStart。

备份旧全局/仓内执行器、Cargo 安装记录、Claude 插件登记/设置与 Codex 配置/hooks 到 /private/tmp/agent-on-v028-upgrade.IrzcMJ（目录 0700）。已在动手前给用户单行回退：`bash /private/tmp/agent-on-v028-upgrade.IrzcMJ/rollback.sh`。恢复仅限精确文件；保留旧插件缓存和所有 worktree/分支，不改 shell RC，不动认证/凭据。回退应紧接本次升级执行，不能覆盖后来的用户设置变更。

发布使用本仓三段 semver 与 tag-release 原子 main+annotated tag，不另开 PR；发布后构建可核 tag 的执行器，再更新插件缓存，按宿主正常信任路径验收。不得直接写 trust hash、绕过 hook 信任或把 debug 测试冒充安装成功。

## 验证记录

本节在发布前记录最终测试/构建证据；实际安装/宿主验证日志另保存在本机备份目录。未记录的现场验收不算完成。

- 原批 full 回归本窗口 340 passed / 0 failed；新增复核回归后专项 19 passed / 0 failed。
- 最终全量 Rust **345 passed / 0 failed**（15 组，日志 final-tests.log）；Clippy（warnings 视错）/fmt/diff check exit 0；独立审计 **98 tests / OK**；隔离 candidate release 构建 offline/locked exit 0。文档预检仅报待创建的 v0.28.0 tag，其他规则通过；新 Markdown 会在入 index 后重新验。实际 tag/装机状态仍由后续现场输出确认。
- intake/2026-09-29-dartify.md 与两个已有未跟踪快照不纳入本批，素材 SHA256 保持 85273f542f18f54cfee06cecd836b5d971011a20966f46da8fb4ffd46955321d。

## 未扩大范围

真正云端独立 clone 的幂等回执运输、跨机认领、Hermes 自动 hook、真实项目收益回访仍待做。当前继续使用原窗口，旧 goal 中已被后续用户取消的强制 lane/主树禁业务提交条款不恢复，也不能按旧文本冒报全目标完成。
