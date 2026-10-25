# 2026-10-03 · 用户确认后的单次阶段交接

> 职责边界：记录本轮交接开发的决策、证据与续接边界，不另建任务台账或工作流规则。执行真相在 `kit/patrol-control-plane.md`，方法论在 `playbook/model-playbook.md`；现场任务为 `stage-handoff-20261003`。

## 会话摘要

用户问自动阶段交接是什么意思、能否继续开发。本轮先解释规划→确认→指定执行者，再落实用户明确交接指令时自动携带背景与派工的一个步骤。不是规划模型自述完成就自行开执行窗口，也不是通用无人值守流程引擎。

本地验证收口：阶段交接专项 15 passed / 0 failed，最终完整回归 401 passed / 0 failed（224 unit +177 integration），exit 0。clippy all-targets -D warnings、fmt --check、diff --check exit 0；文档闸 PASS（229 已跟踪 Markdown）。未安装、未发版、未调用真实账户模型；本地代码完成不冒充实际平台端到端验收。

## 关键决定

- 模型、工具和档位仍由用户决定；没给型号沿用宿主，不能把“继续开发”扩大为启动举例的真实模型、安装或发布。
- 用户确认只读规划的那版结果并指派后，既有 dispatch 接手；四项 handoff 参数一起给出，结果键与审阅时间精确绑定。
- Stop/报告只作数据；确认依据只是调用者记录，不是机器鉴定的人类签名或授权 token。既有值守/宿主审批与调用者真实用户指令要求不变。
- 上游写成果不能通过 handoff 绕过 release/integrate/depends-on；交接不验证、释放或合并上游，不扩张下游明确范围。

## 发现

已有 Task/Session、结果回执、shared writer lock、dispatch 与 runner 足够支持单次交接，无需新服务/数据库/代理调度循环。结果文件会按任务或会话键更新，单纯选择“最新报告”会替换用户已审阅版本，因此新增 handoff-at 检查；创建后把白名单摘要固定在下一任务。摘要限长并提供源文件/树/HEAD 线索，不转移原生私聊全文或隐藏上下文。

## 阻塞与未完成

交接源码、15 项专项与最终 401 项完整回归均通过，静态/文档闸通过。没有真实 Claude/Grok/Hermes 账户端到端交接、全局/插件安装、专用阶段 UI、持久自动触发或无人值守多阶段链。项目 JSON 与系统的项目详情 JSON 保存关系，跨项目任务摘要不投影 handoff；没有 HTML 阶段链验收。旧执行器回写可能丢掉 handoff 字段，跨工具实用需后续升级与真实宿主证据。

## 本轮改动与证据

- `cli/src/coordination.rs`：dispatch 四项 handoff 参数；只读来源/精确结果/版本/大小/来源会话校验；下一任务可选 frozen handoff 与 prompt 的数据/权限边界；task result 回读结果编号。
- `cli/tests/stage_handoff.rs`：真实 Git/共享台账与替身宿主进程测试；覆盖无结果/未知来源/坏 JSON/越界键/阻塞/写来源保护、模型选择保留、重复与并发、版本更新、Stop、失败保留、不自动重放、白名单与凭据遮蔽。
- 方法论与接入件成对修改，skill、看板 JSON 说明、CHANGELOG、AGENTS 同步，不修改 CHARTER。
- 初次显式交接用例实际因旧 CLI 不认 handoff-from 而红，补实现后绿。专项复跑过程中发现测试把 oncall claim 的人读输出误当 JSON、并发 Child 所有权错误，修正测试而不撤掉原断言后重跑。
- 复核发现结果读取可能跟随软链/特殊文件；新增软链反例真实红，补普通结果文件/目录检查后绿。此轮不读取任意结果路径，也不声称对绕过协议的同用户恶意竞态有零风险保证。
- `cargo test --manifest-path cli/Cargo.toml --test stage_handoff` 最终 15 passed / 0 failed（2.14 秒）；`cargo test --manifest-path cli/Cargo.toml --quiet` 最终全部 401 项 passed / 0 failed / exit 0，专用 15 项也包含其中。
- `cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings` exit 0（Finished dev profile，3.10 秒）；fmt/diff exit 0；`python3 .github/scripts/check_docs.py` PASS（229 已跟踪 Markdown，不含本页等未跟踪新稿）。没有通过猜型号、安装依赖、改宿主信任或取消原门禁来过关。

## 续接顺序

1. 同一主入口继续，共享文件顺序修改；已存在的系统看板/模型修正仍未发布，不重复初始化或另造工作树。
2. 本地源码已可显式准备 handoff；实际启动谁仍需用户明确工具/任务/范围，不替用户挑模型。真实闭环先核实际 executor 版本与插件信任，不能用当前源码代替装机证据。
3. 下一步可按用户要求补阶段关系展示或明确授权的宿主调度触发，但不得假装本轮已具备；升级/发布遵本仓 annotated tag 与原子 main/tag 纪律。

## 未决问题

本轮只推进确认后的单次交接；用户尚未指定一次真实规划/执行组合、下一阶段的自动触发条件或无人值守授权。不是要求用户选固定型号才让当前开发继续。
