# 巡逻、派工与清道夫

> 职责边界：本页是可选协调功能的执行书，说明入口、状态与能力边界；值守仍负责合并、发布、对外通信和跨窗口中转。原有 worktree 同文件闸与 `gc --dry-run` 不变。

## 一个人只需一个入口

用户继续在原窗口说目标和反馈。入口先把共享组件、样式 token、导航、路由、锁文件合成一个任务，再把独立部分派出去。十个窗口不等于十个 worktree/PR；共享文件先统一修改、合入，再启动消费者。

| 角色 | 职责 | 默认形态 |
|---|---|---|
| 入口 | 接用户目标、拆任务、决定共用改动归谁 | 原窗口；值守在班时派工经值守 |
| 巡逻 | 记录需求、看任务意图与实际重叠、提供简短列表 | 一个自动创建的独立会话，项目内复用 |
| 执行 | 修改明确范围、交验证证据 | 按需开独立 Claude/Codex/Grok 会话 |
| 值守 | 原有串行合并与审计 | 保留原机制 |
| 清道夫 | 回收已释放、保全、闲置的受管 checkout | 确定性本机检查；无需另开聊天 |

任务列表用 `任务 / 执行窗口 / 状态 / 下一步` 四列。钉住目标、证据与权限；不复述每个工具调用。

主窗口与巡逻窗口各自在原会话 compact，沿用当前任务和回执，不因为压缩重新派发或造树。需求摘要和巡逻台账不是完整任务记忆；接续仍读已有任务记录、相关源文件与实际状态。流程约定见 [会话续接](../boot/session-handshake.md)；当前源码补 SessionStart 的状态注入，安装/信任与实际身份见 [项目接续控制面](session-control-plane.md)，尚无自动压缩前保存完整任务的接线。

## 启用与自动开窗

```bash
agent-on patrol start --host codex
# 也可 --host claude；macOS 默认新 Terminal 窗口
# 已在 tmux 中：--window tmux
agent-on patrol status
agent-on patrol scan
```

`start` 启用本项目记录并创建独立巡逻会话，重复调用复用同一回执。新窗口自行启动已有 host CLI，携带项目路径、角色和台账路径。进程启动后为 `running-unconfirmed`；收到真实 host hook 才为 `ready`。终端打开、登录提示、准备 prompt 都不算 ready。启动失败留下失败回执，不丢任务或 checkout。

首次启用后，插件的 `UserPromptSubmit` 自动记录原窗口需求，并在巡逻不在时尝试重开；`SessionStart/PostToolUse/Stop/SessionEnd` 更新会话状态。**Stop 只表示一轮结束，不表示任务结束。** 只有已安装且获宿主信任的 hooks 能覆盖自动捕获；旧插件、未获信任的 hook、云任务或未接 hook 的 Grok 会显示缺少确认，不能宣称已记录全部窗口。

巡逻的本地观察器每 30 秒更新路径证据，不重复调用模型。重叠或待确认集合变化时才发本机提示（macOS 系统通知；其他系统在巡逻终端提示），同一问题不反复刷屏。独立模型会话先给初始报告，此后按用户查询读取最新台账；不会假装每次文件变化都已得到模型判断。主窗口每次用户提交会收到当前台账位置与重叠数量，可据此调整派工。模型语义判断、视觉一致性仍按具体风险触发。

`patrol disable` 停止巡逻需求捕获和自动开窗，不杀现有会话。未发布源码另有独立 `activity enable/disable`，只观察窗口事件、不派工；关闭巡逻不等于关闭已明确启用的观察，关闭观察也不撤回任务或清道夫配置，见 [面板执行书](live-control-plane.md)。退出 host 后可用 `patrol resume <receipt-id>` 按已记录的精确会话 ID 续接；缺少 ID 时拒绝用 `--last` 猜。启动卡住或原生聊天已关但没有 SessionEnd 时，明确指定 `patrol close <receipt-id> --evidence '实际关闭依据'`；运行中的进程不能被这条命令冒充关闭。

未接到真实 host ID 的失败启动用 `patrol retry <receipt-id>`，复用原任务和 checkout；已接到 ID 就精确 resume。状态不明先确认关闭，不重复创建或新开树。

## Agent OS 是可选派工能力

CLI `agent-os` 是 `dispatch` 的别名；技能入口两种叫法走同一条接线。

```bash
agent-on dispatch --id shared-ui --host claude \
  --goal '统一 Card 与页面间距，附预览证据' \
  --path 'src/components/Card.tsx' --path 'src/styles/**' --worktree

agent-on dispatch --id home --host codex \
  --goal '按已合入的 Card 完成首页' --path 'src/pages/home/**' \
  --depends-on shared-ui --worktree

agent-on dispatch --id investigate --host grok \
  --goal '分析交互方案，先给建议' --read-only
```

第二条只有 `shared-ui` 已释放且成果已进主树才会启动。`--worktree` 明确申请隔离；默认不逐页新建树。只读任务复用项目；同时写同一主树的受管执行会话会被拒绝。派发前声明路径重叠会直接返回任务归属提示；普通 `task add` 和 `patrol scan` 只提示，不给旧会话增加提交闸。

**阶段、执行工具、模型与推理档位由用户决定，Agent-On 只按明确指令调度。** 规划/执行/验收不固定绑定型号；不按价格、能力印象、目录推荐、空闲或失败自行替用户选型或改派。默认不指定型号，型号也不是登记或接线的前置条件。用户主动给出 `--model` 才按原值传递；没有给就沿用当次宿主设置，不额外添加模型/档位参数、不查询目录后选替代者。Codex 的 `session.model_resolution` 仅记录 explicit 或 host-settings / not-probed 的指令依据，不证明实际推理型号或账户资格。retry/resume 仍保持用户明确选择和精确会话 ID；旧自动兜底/档位记录也不再成为启动参数。更换需用户新的明确指令，不把失败、结果回执或阶段完成当作更换授权。

`agent-on model-check --cwd <实际项目目录>` 是用户可选、严格的只读诊断，已从启动路径移开：不可确认或配置与目录不匹配就非零说明，不选择替代型号或档位，不改配置、不启动执行、不要求先通过。`verification: catalog-only` 只是目录观察，不表示真实推理成功；未配置时观察到的 native-default 也不是派工指令。启动由原生客户端执行登录、账户、托管政策与权限检查。真正缺 CLI、无法启动或宿主非成功退出仍保留任务/checkout 并明确报失败/blocked，不能把「已交给宿主」说成「已执行成功」。作者已有的其他阻塞说明不被覆盖；重试只清本机 runner 自己记的失败。拒绝后等待用户处理或明确改派，不自动换号、降档、重试或重放已执行工具的任务。显式 `--model` 为 `not-probed`，由宿主执行时验证；自定义 provider/API-key 沿用自身设置，不切登录方式、provider 或计费通道。`--window external` 仅准备回执，不冒充已检测或启动了 host。

Claude/Codex/Grok/Hermes 适配器调用用户指定、本机已安装的 CLI，继承它们的登录与审批；不代装工具，不共用登录，不带绕过权限参数。缺 CLI/终端能力时明说原因，不改用另一个工具或拟造会话。原生只读诊断与用户显式单次参数依据 [Codex app-server](https://learn.chatgpt.com/docs/app-server) 与 [开发者设置](https://learn.chatgpt.com/docs/developer-settings)；这是薄 CLI 适配，不是通用模型编排运行时。下节单次交接是未发布源码功能；没有「规划者自述完成就无人值守启动下一阶段」流水线，也不据此宣称所有工具的实机交接已验收。

Hermes 执行任务用 `dispatch --host hermes`，调用 `hermes chat --in <实际树> --query <字面 prompt>`，精确续接用 --resume 与 --no-restore-cwd，防止恢复到另一目录。当前 adapter 不接管 Hermes 私有配置或 host hooks，没有可验证的只读沙箱，因此拒绝 Hermes 巡逻与 --read-only。进程状态由 runner 回传，缺 hook 时保持未确认，结果沿现有 task result；真实认证模型任务另需实测。官方接口见 [Hermes CLI reference](https://hermes-agent.nousresearch.com/docs/reference/cli-commands/)。

日常状态可直接打开 [本机面板](live-control-plane.md)。它统一显示已有回执与旧树，空台账会如实说明覆盖缺口；不为显示状态另开模型窗口。

任务也可只登记：`task add <id> --goal '…' --path '…'`。执行者用 `task progress <id> --state working|blocked|waiting --note '当前事实和下一步'` 显式回传；`task result` 后为 reported，表示已有结果自述、未验收。它们不改变验证标记、不释放任务；已释放任务不能被 progress 复活。执行窗口退出后，由入口/作者以实际证据 `task release <id> --evidence '…'`。发布、merge、update-branch 和权限仍归值守，dispatch 不获得这些权力。

## 确认规划后的单次阶段交接（未发布）

这一步的「自动」是**按用户明确交接指令自动带好上下文并派工**，不是系统自己决定规划完成、选执行者或连续启动下一阶段。自然语言可说「这版规划通过，按刚才的指派交给执行者，范围是……」；入口先核真实结果和原用户指令，再使用已有 dispatch。用户没有指定执行工具时补问工具；没有指定型号则省略模型参数，不为登记强迫选型。当前没有预先授权多阶段无人值守执行或 dashboard 写操作 API。

```bash
# 规划任务必须只读；结果文件是含源文件/验证指针的简短报告，不是整个聊天
agent-on task result plan --file ./plan-summary.md
# 从上条真实回执取得 result_id 和 at；例中的时间须替换为用户审阅的那条 at
agent-on dispatch --id implementation --host grok \
  --goal '按已确认的方案实现登录界面，附测试和预览证据' --path 'src/login/**' \
  --handoff-from plan --handoff-result task-plan \
  --handoff-at '<已审阅结果回执的 at>' \
  --handoff-evidence '用户在原入口确认这版方案并明确指派；只实现 src/login/**'
```

默认 Terminal/tmux 路径复用已有 launcher 与 runner；`--window external` 只准备上下文/回执，桌面仍按原生桥接显式创建并绑定，不冒称已启动。`--model` 仅在用户明确给出时附带。只读任务、实际 worktree、路径重叠、依赖集成、值守派工路由与权限检查全部沿原 dispatch；写代码的上游不能用 handoff 绕过合入，必须 release/integrate 后走 `--depends-on`。

`handoff-from/result/at/evidence` 四项必须一起给出。来源必须是同项目的只读任务，且非 queued/blocked；精确结果键只接受该任务的 `task result` 或已关联会话的 Stop 回执，结果目录/文件必须是普通文件系统对象，不跟随软链或读取特殊文件。Stop 仍只表示一轮返回，**本身不触发派工、不验收、不释放**。源回执最大 64 KiB，交接快照最大 32 KiB；摘要与确认依据限长并遮蔽明显凭据，不承诺发现任意秘密。坏回执、无来源、无摘要、跨仓、错误时间或错误结果键拒绝后不创建任务、会话或 worktree。用户审阅的 at 与当前回执不一致时，先报告方案已更新，不能静默取最新版替代。

下一任务的可选 `handoff` 保存来源任务/回执、结果时间、来源目标摘要/路径/实际树与 HEAD、结果摘要、确认依据和实际调用者；只传白名单字段，报告的 verified 自述始终不当验证通过。创建后的快照不会随着规划报告更新而改写。摘要不是完整上下文，接手者须核真实源文件；上游报告不能扩张下一任务明确的目标、文件范围或权限。

确认依据是调用者对真实用户指令的记录，**不是签名、授权 token 或机器鉴定的人类批准**；CLI 不从文本里解析「批准」来授予权力，宿主审批和值守权限照旧。只许入口在确有对应指令时传入，不准由规划者的结果冒充用户。多个进程同一目标编号并发交接只创建一次；后续重发拒绝重复派工。启动或实际 host 失败保留该任务、交接快照和 checkout，明确 blocked，不自动再开窗或换模型；恢复沿原 retry/resume。

项目 JSON 和系统总览的项目详情 JSON 可读取同一任务里的交接关系，不另存 stage 数据库；系统总览的跨项目任务摘要尚不投影 handoff，浏览器尚无专用阶段流程图或批准按钮。旧 executor 可读任务，但回写可能丢掉新增 handoff 字段，跨工具使用前须升级实际执行器。这批只有本地源码和替身宿主子进程证据；不声称真实 Grok/Claude/Hermes 账户调用或完整自动流水线已跑通。

## Codex 桌面原生桥接

桌面上希望看到原生独立聊天时，入口 skill 使用 `patrol start --host codex-app --window external` 准备回执，然后通过可用的 `create_thread` 创建并启动真实聊天，等 `wait_threads` 返回进展，最后 `patrol bind <receipt-id> --thread-id <实际id>`。有实际 host hook 才 ready；hook 先于 bind 到达时归并已有观察回执，没有 hook 就保持待确认。真实独立终端窗口用 Terminal 适配器；创建桌面聊天本身不保证打开第二个操作系统窗口。

原生执行派工可用 `dispatch --host codex-app --window external`。若带 `--worktree`，**由桌面宿主创建**，不预先另外造一棵 Git 树；绑定时必须加 `--worktree-path <宿主返回的实际路径>`，核对同仓后登记。宿主原生 worktree 的归档走其原生 archive 工具，当前 CLI 清道夫不冒充拥有它。没有桌面创建工具时，用 Terminal/tmux；链接预填 composer、`codex app` 打开目录均不能代替自动接单。

原生聊天无终端观察器，可按已授权的巡逻/周期回收需求，用宿主 heartbeat 定期调用 `patrol scan`，配置启用才 `janitor run --apply`。不变或不可行动时保持安静。记录真实 automation id，不另起自制服务；关闭功能后应停止对应 heartbeat。

## 清道夫的有限授权

```bash
agent-on janitor enable                 # 本项目，闲置 24 小时；安装每天 03:30 的回收检查
agent-on janitor enable --manual        # 只启用策略，由宿主 heartbeat 或人工调用
agent-on janitor run                    # 默认只报告
agent-on janitor run --apply            # 必须已启用才可回收
agent-on janitor restore <task-id>       # 保留的恢复 ref 重建原路径
agent-on janitor disable
```

默认复用已有 launchd/systemd 安装器，创建独立的 janitor 日任务，窗口关闭后仍可运行。每次执行重新读取策略；disable 先撤回回收授权，再卸载属于本项目且未被改写的任务。宿主原生 heartbeat 可选择 `--manual`，避免重复调度。`hooks install --daily-gc` 仍是原有只读报告，权限和任务文件均不升级成删除任务。

自动回收仅限 **dispatch 明确创建在本项目 control/checkouts 下的树**，并同时满足：

- 作者已明确 release，有证据及释放时 HEAD，之后 HEAD 没变；没有活会话、未确认会话、共享树或活依赖。
- HEAD 已合入主树的目标 ref，或当前 PR 已合入且其实际 head 覆盖本地 HEAD；远端查询失败不猜 squash 已完成。目标 HEAD 永远在主树解析。
- 无 staged/unstaged/untracked；无 locked、进行中的 Git 操作、子模块或嵌套仓库；进程/CWD 清单完整且没有仍使用该 checkout 的进程（依赖 `lsof`，不可用就保护）。
- 闲置期完成。ignored 文件默认保护；只有用户明确列出的可再生成缓存目录可丢弃，`.env`、数据库、凭据即使在缓存内也保护。符号链接或无法完整盘点时留给人工。
- 删除前重新核对。先写真实 `refs/agent-on/recovery/<task-id>` 与回执，再 `git worktree remove`；不 force、不跨树 stash/commit、不删除分支。恢复 ref 保留对象，不依赖一串可能被 Git GC 清掉的 hash。

`--regenerable-ignored node_modules` 等目录是明确的可再生成缓存授权，不设默认通配清理。报告中 `protected` 附原因；救援和待确认不会当可删。工作区与进程仍可能在检查后改变，Git 的最终移除检查与受管任务释放协议共同缩小竞态；没有“对任何未知进程零风险”的承诺。

## 状态与验证

回执与私有需求摘要落 `<common-git-dir>/agent-on/control/`，跨树共读，不进项目提交。原始工具输入、工具输出和全量 transcript 不采集；需求摘要限长并遮蔽常见凭据形式，不能保证识别任意秘密。关闭记录后不再采新数据。

巡逻看声明路径、相对共同祖先的已提交 diff、暂存/未暂存和未跟踪文件。缺少声明或读不到树会报告 unknown。它能提早发现同文件风险，不能机械证明 API 语义、视觉或依赖都一致。

验证按改动选：样式看预览与交互，行为改动跑相关回归，共享组件做一次集成验证。仅当 HEAD、base、输入与环境一致才能复用已有证据；required CI 照旧满足，不以旧绿灯冒充新验证。本版不自建 CI 缓存或跳过仓库必需检查。

实现与回归：`cli/src/coordination.rs`、`cli/src/janitor.rs`、`cli/tests/coordination.rs`。构建需 Rust 1.89+（使用标准库文件锁，进程退出自动释放）。不添加运行库或模型服务。
