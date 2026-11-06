# 项目接续与实际会话身份

> 职责边界：Claude/Codex 启动、恢复、compact 后的身份与简短接续提示。保留实际值守判断，不采集任务／活动、不创建窗口、不管理宿主压缩。

已接入项目的 `SessionStart` 仅读取实际值守，提示沿当前对话和项目已有记录继续。项目识别依据为当前 checkout 的 lock、Agent-On 自身 CHARTER + BOOTSTRAP，或 common git dir 既有 agent-on 状态。未接入仓静默；损坏值守登记明确报不可用，不冒称无人在班。

## 身份与权限

值守工作区、宿主和真实 session ID 必须同时匹配。`oncall claim --session` 是交单地址，不替代实际 ID；同目录不同窗口不能互相续心跳、释放或抢占。认领、心跳与释放在同一锁内原子写入，迟到旧事件不覆盖新值守。90 分钟失效与旧 v1 身份未确认语义不变。

Codex 使用 `CODEX_THREAD_ID`；Claude 优先 `CLAUDE_CODE_SESSION_ID`。hook payload 的 `session_id` 优先于继承环境，workdir 不授予身份。Claude SessionStart 只追加适配器变量到宿主给出的 `CLAUDE_ENV_FILE`，字面值安全引用并保留其他 hooks；不改 shell 配置。嵌套宿主保留父／子身份区分。其他宿主须提供实际 `AGENT_ON_HOST` / `AGENT_ON_SESSION_ID`，缺身份不授值守权。

路由仍核 lane 和历史已绑定真实 ID，历史绑定只用于保留已有路由限制，不表示窗口仍在执行。未知窗口不靠猜测判断。Agent-On 不再新增窗口注册或活动采集；跨宿主启动交给用户选择的原生工具。

## 安装与验证

插件只注册 SessionStart 与必要 PreToolUse guard；逐工具 capture、Stop、SessionEnd 采集已删除。隐藏兼容入口 `capture` 对非 SessionStart 事件静默无操作，防止尚未重载的旧 hook 重新开启记录。

先升级实际 CLI 与插件，再按宿主方式重载／信任。`build-info` 核实际执行版本；仓库源码通过测试不等于已加载宿主换版。身份错配诊断只显示必要身份字段，不记录任意环境或完整工具输入。

回归保留同目录冒认、跨宿主相同 ID、并发认领、未知／旧／损坏登记、迟到心跳、payload 优先级、Claude 环境文件引用和写失败、历史绑定路由。真实宿主重载须另验；远程 clone 不共享本机 common dir，不能从本机“无人在班”推断远端负责人。
