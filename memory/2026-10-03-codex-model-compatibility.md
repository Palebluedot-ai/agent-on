# Codex CLI 默认型号兼容排错记录

> 职责边界：本页记录 2026-10-03 本仓默认型号拒绝的根因、最小修复与本地证据；不是发布/安装回执，不声明其他厂商或完整 Agent-OS 待办已验收。唯一完整排错记录；方向摘要在 snapshot。

**当前结论（同日最后决定）**：规划/执行/验收的工具、型号与推理档位由用户决定；Agent-On 不自动选型或改派。启动与 retry/resume 仅传用户显式型号，未指定沿用宿主当次设置，不调用目录检查或添加档位覆盖。下面「初次修复」与「门槛减法」是历史经过，自动兜底已撤回；最新改动与证据见末节。

## 症状与根因

已有只读任务 `codex-local-dogfood-20261003` 因原生 Codex CLI 返回 HTTP 400 / turn.failed / exit 1 阻塞，报 `gpt-6.1-sol` 不支持 ChatGPT 账户。不是未安装 hook：此前真实 SessionStart 已自动注入项目状态，但未进入模型工具调用。

本机 Codex 0.147.0 已通过 ChatGPT 登录。有效配置为 `gpt-6.1-sol` / `xhigh` / `priority`；没有自定义 provider。原生 `config/read`、`account/read`、`model/list` 的只读回读证实：这台 CLI 的可见目录没有配置型号，唯一 `isDefault` 为 `gpt-5.6-sol`，支持 xhigh。账户/客户端目录与共享设置不匹配，先前薄适配直接继承设置，既没兼容检查，也没留下型号选择回执。

本次事实中的型号不是程序默认表。实现不固定任何型号，也不把目录兼容说成推理成功。[原生接口文档](https://learn.chatgpt.com/docs/app-server)说明目录与有效配置查询；[开发者设置](https://learn.chatgpt.com/docs/developer-settings)说明单次参数覆盖，无需写全局配置。

## 初次修复与边界（历史实现，自动兜底已撤回）

- `cli/src/codex_model.rs`：用已安装 Codex 的 stdio metadata RPC，最多 12 秒/16 页、单行最多 1 MiB、有限队列；不创建模型会话，不调用工具。不持久化账户邮箱、配置层或任意原生错误正文，不读取/复制凭据。只结束自己启动的 metadata 子进程。
- 没有显式型号才查询。可见配置型号保留；不在目录时选唯一可见 `isDefault`，不猜第一条。显式型号原样保留并标 not-probed；自定义 provider/API-key 不切登录/provider/计费方式。托管型号或档位不兼容就停止，不覆盖政策。配置档位不支持时仅用目录公布的有效默认档位，留下覆盖记录。
- `launch` 在开终端前检查，网络查询不持有共享台账锁。成功计划写现有 `Session.model_resolution`，原 `Session.model` 仍表示用户显式选择；provider argv 只带单次覆盖。retry/resume 重新检查，沿用任务/checkout 与精确会话 ID，不把旧兜底钉成永久默认。
- 老/外部准备回执在 runner 补检查；只准备的 external/桌面桥接不宣称 CLI 已启动。检查、启动异常写失败会话与 blocked 任务，清 runner PID，保留 checkout；实际宿主非成功退出记录 exited/exit_code 与 host-exit 阻塞，不自动重试。重试只清 runner 自己的 launch-failure/host-exit，不清作者别的阻塞或结果；新失败也不先覆盖作者阻塞再冒充可清。
- `agent-on model-check --cwd <实际目录>` 提供同一 JSON 计划；缺省是实际 cwd，不误用 Agent-On 源码目录。`catalog-only` 不验证真实账户推理资格；登录、网络、额度、权限错误不进入换号/重放工具循环。不改桌面设置、审批、登录、hook 信任或合并/回收政策。

## 验证

先红后绿：最初 default-resolution 回归因缺 model-check 失败；preflight 失败回执回归证明原任务仍为 dispatched 后修正；实际 cwd 回归证明旧入口误用源码目录后修正；非成功模型退出的任务仍为 dispatched、以及新检查失败覆盖作者阻塞，也分别取得真实失败输出后修正。

- `cargo test --manifest-path cli/Cargo.toml --test model_compatibility`：16 passed / 0 failed。覆盖兼容/不兼容默认、显式型号、API-key/自定义 provider、托管型号/档位、分页、未登录、超时与错误脱敏、runner 参数与回执、检查失败不启模型、重试/精确续接重新选型、作者阻塞保留、实际 cwd、非成功模型退出不重放工具。
- `cargo test --manifest-path cli/Cargo.toml --quiet`：最终完整 Rust 回归 364 passed / 0 failed / exit 0（220 单元＋144 集成）；clippy `--all-targets -- -D warnings`、fmt 与 `git diff --check` 均通过。
- `python3 .github/scripts/check_docs.py`：229 份已跟踪 Markdown PASS；不把未跟踪的新记录算进去。

真实原生只读验收：本机开发版 model-check 先取得 `native-default-fallback` 计划，再仅本次传型号给已登录 Codex，沿原 read-only sandbox 与正常 hook 信任运行 `agent-on build-info --json`。没有手工调用 capture，没有权限绕过，也没修改全局配置。实际原生线程 `01a10050-4984-7cd3-9642-f441cd275b69` 的 turn_context 是 `gpt-5.6-sol` / `xhigh`；command_execution exit 0，turn.completed，原生退出 0。实际工具返回已安装 agent-on v0.28.0 / commit `102b4776f50527422f0d3909f7da828d84a05a48` / clean identity。

该线程独立 developer 消息包含自动注入的“Agent-On 项目状态已接续”，并列出当时两个未释放任务；不是事后从 task list/dashboard 反推。允许命令执行成功不单独证明拒绝分支，Git/shim 拦截沿此前已安装 executor 黑盒验证；本次未做真实模型越界写测试或实机 compact/resume。适配器 retry/resume 由替身 host/launcher 真实子进程测试，不冒充 GUI 实机。

本机原始日志（临时目录，可能清理，不随项目分发）：`/private/tmp/agent-on-model-fixed.BWXg3Q/` 中的 model-resolution.json、codex-events.jsonl、codex-answer.txt；自有原生 rollout 在 `/Users/chao/.codex/sessions/2026/10/03/rollout-2026-10-03T13-50-24-01a10050-4984-7cd3-9642-f441cd275b69.jsonl`。只核本次自有探测，不采其他聊天。

## 交付状态

本地修复已实现，未 commit/tag/push，未升级全局 CLI 或插件 executor；旧安装 v0.28.0 仍没有这次检查。与前轮未发布看板共用 coordination.rs，由同一入口顺序修改并统一验证；意图扫描可以提示重叠，但没有启动第二写者、创建项目 worktree 或删除旧树。相关任务回传为 reported/待验收，不自动 release。

未测试：其他账户/套餐、真实 API-key/custom provider、真实 Claude/Hermes 模型调用、Linux 实机终端、桌面模型选择器、动态账户目录变化后的自动恢复。不宣称“所有模型以后都不会被拒绝”。

## 同日调整：模型不作为前置门槛（历史实现，自动兜底随后撤回）

用户要求默认不指定型号，每次允许不同模型，避免把系统自己卡住。核代码发现两处人为约束：正常 native/configured 默认也被 provider argv 写成 `--model`；元数据读取/目录不确定会在实际 host 启动前先标失败。这两处已改：只有 explicit 和确证不兼容的 native-default-fallback 才附型号；元数据诊断失败只记录 `host-default-unverified` / unavailable、提示后原样启动 host，不附型号或档位覆盖。模型不在状态回执的必填字段里；五类 report 不读模型目录。

原生 host 仍执行登录、账户资格、托管限制、sandbox 和审批；不是放宽权限，不变更 provider/登录/计费通道。model-check 作为用户主动调用的诊断仍严格返回不可确认，默认启动不要求先跑它。真实 launch/host-exit 失败仍报 blocked，不因为降级提示宣称成功。正常配置和默认由宿主动态选择，显式用户选择不替换，已证不兼容才用原生目录唯一推荐做单次兼容，不留硬编码型号。

官方依据 [Codex 配置优先级](https://learn.chatgpt.com/docs/config-file/config-basic)：CLI 覆盖高于原有默认，故不把正常 metadata 观察重新写成覆盖；requirements 与 defaults 分开、仍由原生客户端执行。本次使用官方文档核对，不从第三方型号推荐选替代型号。

先红后绿：新增正常/未配置默认不带 --model 的用例真实报出旧 argv；目录为空仍可启动用例真实报出旧检查拦截。改后专用回归 **18 passed / 0 failed**，含原生默认动态 retry/resume、目录不确认不要求型号、实际 host exit 1 仍 blocked、实际坏解释器 spawn 失败及重试只清自己失败、作者阻塞不被覆盖。旧 timeout 测试的错误文案断言一并更新，保留超时与秘密过滤断言后复跑通过。此处全为本机替身 host 的真实子进程，不宣称新一轮实际账户推理已经验证；前节真实只读验收属于初次单次兼容路径。

最新完整 `cargo test --manifest-path cli/Cargo.toml --quiet` 为 **385 passed / 0 failed**（224 unit +161 integration），exit 0；clippy all-targets -D warnings、fmt --check、diff --check exit 0；文档闸 PASS（229 已跟踪 markdown）。18 项型号专用用例同时在这轮完整回归内通过。没有以替身模型测试代替现场认证模型测试。

本轮没有安装全局 CLI/插件，没有 commit/tag/push，没有完成活动自动接线或测试其他宿主模型调用；不把用户的「不用指定型号」扩大为这些动作的授权。

## 同日最后决定：Agent-On 无模型选择权

用户明确「规划 执行 所用的模型 不由 agent-on 决定……agent-on 只是可以按照我明确的指令进行调度和编排」。据此撤回前述 native-default-fallback 和 reasoning effort 替换，包括兼容默认的所谓单次兜底。原生目录不是用户授权，拒绝也不是换号授权。

实现：provider argv 在所有已接 CLI 中只使用用户显式 `session.model`；Codex launch/serve/retry/resume 不再调用 metadata，刷新回执为 explicit 或 host-settings / not-probed，旧自动选择与档位记录不传下去。登录、账户资格、托管要求与 sandbox/审批仍由实际宿主执行；真实失败照旧 blocked、保留任务/checkout、不自动重试或重放。可选 model-check 保留只读、有界、脱敏的诊断；配置或档位不兼容就明确非零，不给替代号。未配置时观察 native-default 也只供诊断，不成为启动选择。

方法论/接入件成对落点为 `playbook/model-playbook.md` / `kit/patrol-control-plane.md`；旧混编建议注明用户指令边界并撤去自动升降档表述，索引、看板说明、CHANGELOG、AGENTS 和同日快照同步对齐。不改 CHARTER，不引入自动阶段引擎，不安装、不发版。

先红后绿：新用例 `legacy_automatic_choices_never_override_user_or_host_settings` 真实失败，旧 argv 出现 `--model old-automatic-model` 与 `-c model_reasoning_effort="low"`；改后同一用例通过，覆盖未指定和用户显式选择两种情形，启动目录查询标记文件均未产生，旧自动参数均未出现。专用回归 **19 passed / 0 failed**（12.84 秒），包含诊断不替换型号/档位、分页、超时脱敏、实际 spawn/host exit 失败、旧回执与重试/精确续接保留。此轮替身宿主真实子进程不产生账户推理请求，不冒充认证模型已实跑。

收口：`cargo test --manifest-path cli/Cargo.toml --quiet` 实际 exit 0，**386 passed / 0 failed**（224 unit +162 integration）；本轮 19 项专用用例也在完整回归中通过。`cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings` exit 0（Finished dev profile，2.84 秒）；fmt --check、diff --check exit 0；文档闸 **PASS（229 已跟踪 Markdown）**。没有重启模型、安装全局/插件、创建/回收项目 worktree、commit/tag/push 或宣称实时活动与阶段流水线已完成。
