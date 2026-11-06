# v0.30.0 迁移：自建派工、活动采集和面板退场

> 职责边界：从 v0.29.x 及更早版本升级的 breaking 变更与安全退出步骤。2026-11-06 用户明确批准保留 v0.x，作为本次 breaking 特例；不宣称 v1.0 外部用户验证已完成。

## 变化与替代

| 旧入口 | v0.30.0 |
|---|---|
| dashboard、status --serve/--system、agent report | 已删除；用当前对话和文字 status |
| dispatch / agent-os、patrol、阶段交接、task | 已删除；目标和证据沿项目已有记录，用户按需直接用宿主能力 |
| activity、逐工具 capture | 已删除；不再维护窗口活动状态 |
| model-check | 已删除；模型设置与诊断由宿主提供 |
| janitor | 已删除；保留 checkout、分支、恢复 ref 和历史记录，回收由明确目标授权控制 |
| status --json | schema_version=2；当前 checkout 的 repo/head/branch/dirty、project_records、oncall、landing、landing_age_seconds、next_action；goal/progress 为 null，不能据 Git 推断任务状态 |

旧 JSON 的 config/patrol/activity/worktrees/daily_gc 等面板字段不再提供。linked worktree 的 status 显示当前目录，合入与值守证据仍共读 common git dir。旧采集记录不被当作当前工作进度。Git 冲突保护、跨仓边界、会话身份、master/landing/oncall、独立审计、setup/doctor 和 tag-release 保留。

## 升级顺序

1. 记录 `agent-on doctor`、`agent-on build-info --json` 和 Git hooks 实际指向；备份将被替换的二进制和 Agent-On 插件目录，记录绝对路径。只备份对应插件，不包含凭据或整个宿主配置。
2. 停止明确属于旧 Agent-On dashboard 服务／patrol serve 的进程和专属宿主定时任务。先核 PID、可执行路径及参数，不按模型名称批量杀进程；用户的 Claude/Codex 会话继续保留。没有运行就不做此步骤。
3. 在每个实际启用过旧功能的项目运行新版 `agent-on retire-legacy` 预览。确认项目范围后 `agent-on retire-legacy --apply`：关闭既有 config 中 enabled/janitor_enabled 及 activity-config 的 enabled，首次备份为 `.json.pre-v030`；卸载内容与归属经过校验的旧 janitor 定时项。损坏／软链配置拒绝修改，计时器被手改则报告待人工处理。重复执行安全。这个命令不处理其他项目、不删除历史任务／工作目录／分支／恢复 ref。
4. 构建并安装发布 tag 的 CLI，更新实际 Agent-On 插件副本及声明，按宿主正常方式重载。插件只保留 SessionStart 与必要 PreToolUse guard；只清理 Agent-On 专属旧采集注册，保留其他工具 hooks。个人级注册也须核对，不能只更新仓内 hooks.json。
5. 核实际全局 CLI、Git hook 路径、已加载插件内执行器的 `build-info`。`status` 可用，旧命令不能启动执行，普通 Git 提交仍受原有保护。没有新建活动记录才算采集退出；仓库测试不能替代宿主重载证据。

`retire-legacy` 只解除旧功能策略和有归属证据的清道夫调度。Git 的 `worktree gc --dry-run` / daily-gc 仍只报告，不自动删除 checkout。

## 历史资产与恢复

旧 common-dir 下 `agent-on/control/tasks`、`results`、`sessions`、`activity`、checkouts 及恢复 ref 保留，供人工回查；不进入新文字状态的任务判断。需要恢复历史 checkout 时先核旧回执中保存的 ref 和路径，再按用户授权使用 Git 原生命令，不能覆盖已有路径。

回滚前停止新安装的执行器调用，恢复步骤 1 备份的二进制与插件文件，再按宿主方式重载。若明确要重新启用旧功能，单独审阅 `.json.pre-v030`；不要批量恢复活动采集／清道夫授权或重新安装计时器。代码回滚不自动扩大权限。

旧操作说明已归档：[面板历史](../legacy/retired-v0.30/live-control-plane.md)、[巡逻派工历史](../legacy/retired-v0.30/patrol-control-plane.md)。它们不再是当前使用入口。

## CI 与运行时间

PR 继续按改动范围运行，两个必需检查名不变。Rust 工具链固定，下载与构建缓存分开；构建缓存按平台／工具链／锁文件／profile／源码区分，并恢复兼容前缀。夜间计划香港 01:23 全量，不保证 GitHub 准点创建；查看真实运行完成时间。CI 提速不等于解决 schedule 服务延迟。
