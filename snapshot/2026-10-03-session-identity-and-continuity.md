# 2026-10-03 · 主窗口续开发：实际身份与状态接续

> 职责边界：记录本轮开发范围、复现、验证与实际安装边界。不是新版本发布、全局安装或角色接班授权；执行说明见 kit/session-control-plane.md。

用户要求以本窗口作为主入口继续开发，并核对上次问题是否已修复。读取共用 AGENT.md、本仓纪律、宪章、上轮清单及实际工作区后继续；复用已发布面板/派工实现，没有重复开树或新建任务窗口。

## 接手基线与实际安装

- 本轮开始时 tracked 工作区干净，HEAD 是 annotated v0.27.0。只读 `git ls-remote` 确认远端 main 与 tag peeled commit 均为 `ab7bb283f306c8a3dc8e283bedfddfbcd8196d8e`，tag object 为 `1be88c26148fba04859dd190a5f5c3c18b8757b0`。上轮最终测试日志实际合计 326 passed / 0 failed。
- 上轮 setup、PR 动态证据、command workdir、损坏 payload、每日报告失败检测均已进入 v0.27.0，详细历史保留原快照。
- 本机是混装：全局 `~/.cargo/bin/agent-on` 与 Claude 0.26 插件缓存的 build-info 都为 v0.26.0 / 0c6e5dc；仓库 `cli/target/release/agent-on` 已为 v0.27.0 / ab7bb28。指向仓库的个人 shim 可执行后者；插件及 Git hooks 仍可能调用前者。doctor 实际给出不同执行器和缓存证据。不能从 tag 推出用户所有入口已升级。
- Codex 个人注册仍缺 SessionStart 捕获；Claude 已退出。本轮没有重开它。开始及结束现场均无值守在班，巡逻/清道夫未启用；主窗口授权不被冒报成后台角色已经运行。

## 本轮修复与实现

1. 同目录多个窗口曾都被判值守；改为 actual host + session ID + worktree 共同匹配。交单地址不是身份，未知身份不能上岗；旧 v1 无 identity 标 legacy-unverified，不按目录赋权，原 expiry 仍在。
2. 认领、释放、心跳复用仓库级 OS 锁和私有原子文件写；读取所有者后在锁内重读。并发两个同目录窗口只一个成功，旧窗口迟到不会续新持有者的心跳。无登记的普通 guard 不创建心跳锁。
3. 派工同时核实际会话和调用者工作区，repo/workdir 参数不授予值守权。CLI 新模型进程清父宿主身份及 hook 环境，保留派工回执，等待目标宿主实际接入。
4. 新建/恢复/compact 沿现有 SessionStart capture 入口注入共读状态，最多五项任务摘要；巡逻关闭时不创建 recorder、不捕获 prompt。Claude 仅追加会话变量到宿主环境文件，保护字面引用和其他 hooks；嵌套 Codex 使用自己的 native thread ID。
5. 同目录已绑定原生窗口按实际 ID 识别横向消息；新增 send_message_to_thread 目标解析与 matcher。值守的实际 native ID 可收交单，内部 parent/main/子代理及旧 lane 判据保持。未知窗口/别名不假装已覆盖。
6. 损坏值守登记显示不可用并不授合入/派工权；损坏捕获配置仍注入不可用状态，回执失败不把整个 SessionStart 提示吞掉。接续说明落 kit，并对齐 boot、patrol、guard 与 CHANGELOG 未发布段。

## 实际验证

- 新增第一组 6 项黑盒回归在旧实现全红（/private/tmp/agent-on-session-red.log），随后修实现全部转绿；后续补至 14 项，覆盖身份、并发、迟到、派工、环境传递、消息路由、损坏记录及 primary/linked 共读。
- 最终 `env -u CODEX_THREAD_ID -u AGENT_ON_HOST -u AGENT_ON_SESSION_ID -u AGENT_ON_PARENT_CODEX_THREAD_ID cargo test --offline --quiet`：**340 passed / 0 failed**，15 组，日志 `/private/tmp/agent-on-continuity-final-tests.log`。测试去掉当前窗口环境，不靠本会话实际身份才能通过。Hermes 替身执行额外核六项父身份/hook 环境均未继承，不等于认证模型实跑。
- `cargo clippy --offline --all-targets -- -D warnings`、`cargo fmt --check`、`git diff --check` 均 exit 0。文档闸 226 tracked + 新接续执行书共 227 份，职责边界、链接和推荐 pin 无错误。
- release 候选在 `/private/tmp/agent-on-continuity-release-check` 单独构建，offline/locked 成功；没有覆盖 shim 实际调用的仓库 release 程序。候选 build-info 明示 source_dirty=true，未冒称是新发布版本。
- 候选对本仓实际只读 dashboard 及当前真实 Codex ID 的 SessionStart compact payload 成功，回传既有 task/值守/未启用状态。这里只证明真实文件投影和 wire contract；未宣称宿主自动调用/信任已经验收。
- 原有未跟踪 intake 未动，受保护素材 SHA256 仍 `85273f542f18f54cfee06cecd836b5d971011a20966f46da8fb4ffd46955321d`。本轮未提交、tag、push 或安装；推荐版本仍 v0.27.0。

## 尚未完成与下一步

| 项目 | 状态与下一步 |
|---|---|
| 本批合入/发布/统一实际执行器 | 本地已实现并验证，需对这批 cli/src 和 hooks 的权限判定变更作最终批准，再按本仓发布纪律分层提交、封版和原子推送。旧在班登记若存在，需要原值守明确迁移；不能把未知会话自动接成值守。 |
| Claude/Codex 实际宿主自动接续 | 需要更新真实执行器/缓存并补 Codex SessionStart 接线，按宿主正常信任流程验收。不可拿本次手工 payload 当自动接入成功。 |
| 真正云端/远程 | 独立 clone 没有本机 common dir；便携配置、任务 ID、指定合入入口与幂等回执运输仍待开发及真实云任务验证。 |
| 自我增长与协作收益 | 继续用真实项目回访证明冲突、重复 CI、确认次数减少；消化可删除/缩小规则。没有收益对照，不宣布 RSI 或成长闭环效果验收。 |

下一轮优先把这批接续能力在实际宿主跑通，再接云端回执与真实项目效果回访。持续以原窗口为用户入口；compact 默认留在原会话。
