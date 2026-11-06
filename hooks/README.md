# Agent-On 宿主 hooks

> 职责边界：插件的 SessionStart 身份接续与 PreToolUse Git／消息路由保护。具体权限由项目规则决定；不采集窗口活动，不启动模型或后台服务。

`hooks.json` 仅保留两类事件：

- SessionStart：共用 shim 的 `capture` 入口，仅注入值守身份和项目接续提示。
- PreToolUse：Git／命令执行核跨仓与权限，跨窗口消息核既有路由。实际 index 和推送路径的同文件检查由原生 Git hooks 执行。

逐工具 capture、UserPromptSubmit、PostToolUse、Stop、SessionEnd 采集已退场。旧插件若暂未重载，新执行器的 `capture` 对非启动事件无操作；完整升级仍须更新实际插件声明，见 [迁移说明](../docs/migration-v0.30.md)。

SessionStart 只处理宿主提供的真实身份；不会认领值守、创建任务或记录工具输入输出。详细语义见 [会话身份](../kit/session-control-plane.md)。
