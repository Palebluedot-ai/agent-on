# Codex 兼容入口

> 职责边界：旧 Codex prompt 的兼容路由；当前入口是 `$agent-on`，唯一内核在 `skill/SKILL.md`。

通过已登记 work_root 或已安装插件定位 `skill/SKILL.md`，按 deliver / status / master / init / adopt / handshake / upgrade / doctor 执行。默认当前对话推进；自建派工、巡逻、活动采集和面板已退场，旧用户先读 `docs/migration-v0.30.md`。不要猜测固定本机目录。
