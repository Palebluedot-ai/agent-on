# 项目接续与实际会话身份

> 职责边界：同一 Git common dir 内的 Claude/Codex 启动、恢复、压缩续跑状态提示与值守身份接线。不初始化项目、不认领角色、不启用巡逻、不创建窗口；真正云端的独立 clone 不共享这份运行台账。

已接入项目的用户继续在原窗口说目标。安装并正常信任当前插件 hooks 后，`SessionStart` 读取已有项目记录；startup、resume、compact 都返回同一份简短状态：实际值守、巡逻/捕获是否启用、未释放任务与台账地址。复用 `hooks/hooks.json` 的现有入口，不另存一份任务数据库。这里只补注入已有事实，不自动生成完整任务摘要或接管宿主压缩。

项目识别依据是 primary 根的 `agent-on.lock.md`，Agent-On 自身的 `CHARTER.md + BOOTSTRAP.md`，或已存在的 control 台账。未接入仓静默。巡逻关闭时接续只读已有台账，不建 control 目录、不记录 prompt；启用巡逻后的需求捕获仍走现有 capture。状态损坏明确报不可用，不冒称无人在班。

## 值守身份

值守的工作区、宿主与实际会话 ID 必须同时匹配。`oncall claim --session <交单地址>` 中的 session 只是可读交单地址，不能代替实际 ID。同目录不同会话不能互相续心跳、release 或抢占；明确交接仍用原有 force 入口，权限与用户授权不变。认领、释放、续心跳用同一仓库锁并原子落盘；迟到的旧会话事件不能覆盖新值守。

已捕获并绑定实际 ID 的原生聊天即使共用目录也算不同窗口。guard 识别 SendMessage 与 send_message_to_thread 的实际目标，交单给值守可用该宿主的真实 ID；其他已确认窗口的横向消息仍转投值守。未捕获窗口和未知别名不能凭猜测识别，不能宣称覆盖所有平台消息。CLI 新派出的模型进程先清掉父宿主身份和 hook 环境，保留派工回执，再由目标宿主建立自己的身份。

- Codex 工具进程使用原生 `CODEX_THREAD_ID`。hooks 使用实际 payload 的 session_id；该字段优先于继承的环境 ID，workdir 不授予身份。Codex app 与 CLI 同属 Codex 身份域。
- Claude 的 SessionStart 仅追加自己的身份变量到宿主给出的 `CLAUDE_ENV_FILE`，后续 Bash 命令继承。保留其他 hooks 的 export，字面值经 shell 引号处理，不改用户 shell 配置。从 Codex 启动的 Claude 标记父 thread ID；随后新 Codex 子会话使用自己的原生 ID。
- 其他执行器的适配需要实际设置 `AGENT_ON_HOST` 与 `AGENT_ON_SESSION_ID`，不能把路由名、目录名或准备回执冒充已确认身份。未确认身份不能上岗或获得在班值守权；普通无值守工作仍按原规则。

旧 v1 登记仍可读取，但没有 identity 时标 `legacy-unverified`，不会按目录给任何窗口值守权，也不由任意窗口代续心跳。正常 90 分钟失效规则保留；若仍有值守需继续工作，由用户明确确认原值守重新接班后再执行 `oncall claim --session <地址> --force`。不为迁移静默接班或清空登记。

## 安装与验证边界

修改仓库源码不会升级全局 CLI、插件缓存或已有会话。先核实际 executor，再更新当前插件并按宿主正常流程信任 hooks；旧个人注册仅有 PreToolUse 时不能完成 SessionStart 注入。不得以未发布 debug 二进制通过测试冒充用户已安装。

发布前回归包含：同目录冒认、不同宿主同 ID、并发认领、未知/旧登记、旧心跳迟到、工作目录冒认派工、Claude 环境字面引用与嵌套 Codex、linked worktree 共读、损坏台账。认证模型实跑和宿主信任是否生效另验，不能用替身 payload 测试替代。

同一 clone 的 worktree 共读 common dir；换平台不重新初始化项目。远程 clone/临时云容器必须携带项目规则、pin 与指定合入入口，并实际回传任务证据；本机这里的“无人在班”不证明远程项目全局无人负责。本版没有分布式认领、云环境发布或跨机实时同步。

依据：[Codex hooks](https://learn.chatgpt.com/docs/hooks) 的 SessionStart source、session_id、额外上下文和 compact 后注入；[Claude hooks](https://code.claude.com/docs/en/hooks) 的 SessionStart 与 per-session 环境文件。接口可用性仍以实际宿主版本与信任状态为准。
