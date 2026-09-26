# intake:CryptoQuant 2026-09-06 结账（首次，13 卡）

> 范围：首次结账（lock `last_settlement` 为空 → 全量扫描），来源 = `loop-notes.md`（2026-09-05 T0 起）候选层标 `agent-on` 的散文条目 + `agent-on.lock.md` local_deviations 三行，扫描截至 main `b51524e`。项目域条目（ccxt defaultType 安全网、虚拟时钟 ensure_future 用法、Bybit 整点 10006 的时间分布、Bitget endTime 不含的语义本身、commands 并发守卫、ruff 断行）留项目 `loop-notes.md`，不出仓。项目 pin v0.19.0，模型 claude-fable-5-1。

### agents-skeleton-needs-one-liner-slot（M 档 AGENTS 骨架缺「项目一句话」槽位，握手第一步却要从 AGENTS 读总目标）
- source:CryptoQuant @ 177f868 | pin v0.19.0
- evidence:`177f868 chore(scaffold): Agent-On M 档骨架初始化` 按 kit/AGENTS-skeleton.md 实例化后，boot/session-handshake.md 第一步「从 AGENTS.md 读总目标」无处可读；项目自加 `AGENTS.md` §0.5「项目主线」（一句话 / 北极星指针 / runtime≠product 分栏）；agent-on.lock.md local_deviations 第 1 行
- confidence:low（单项目一次；但握手执行书与骨架模板的缺口是可读文件核对的）
- claim:kit/AGENTS-skeleton.md 顶部给一个固定的「项目一句话 + 北极星指针」槽位，让 session-handshake 第一步有确定的读取位置，而不是各项目自加节。
- suggested_landing:kit/AGENTS-skeleton.md 顶部加 §0.5 槽位；boot/session-handshake.md 读取表把「总目标」指向该槽位
- rollback:revert 落地 commit
- trace:loop-notes.md 2026-09-05「M 档 AGENTS 骨架无『项目一句话』槽位」；agent-on.lock.md local_deviations 第 1 行
- 状态: landed@同批（kit/AGENTS-skeleton.md §0.5 项目一句话+北极星指针槽位 + boot/session-handshake.md 读取表指向该槽位）

### guard-blocks-path-literal-not-target（git guard 按命令文本匹配 Agent-On 路径字面 + 写动词，把只在文本里引用路径的项目仓命令也拦了）
- source:CryptoQuant @ 177f868 | pin v0.19.0
- evidence:三次实证。①T0 在项目仓提交时 message 引用了 BOOTSTRAP.md 的绝对路径，PreToolUse guard（`~/.claude/plugins/cache/agent-on/agent-on/0.5.0/kit/guard/agent-on-git-guard.sh`）按「命令文本含 Agent-On 路径 + git 写动词」拦下，实际对 agent-on 仓零 git 操作，改写 message 去掉路径字面后 `177f868` 落地。②本次结账读 agent-on 仓的 `rev-list --count v0.19.0..HEAD`（只读）要先确认不含写动词才敢跑。③本次结账用 Bash heredoc 往 `intake/` 落盘本文件被拦——因为卡片正文里出现「revert 落地 commit」「不 add / 不 commit / 不 push」等字样；改用编辑器工具写文件才过。与 bench 案 39/40「闸拒字面≠禁目标」同型
- confidence:medium（同型案例已入册两例，本项目三次复现）
- claim:跨仓 git guard 应判「git 命令的目标仓（cwd / -C / GIT_DIR / 命令的仓库参数）是不是 Agent-On」，而不是判命令文本里有没有 Agent-On 路径字面；commit message、heredoc 正文、echo/cat 内容里出现路径或动词不构成越界。
- suggested_landing:kit/guard/agent-on-git-guard.sh（或 cli `agent-on guard`）改判据为目标仓，并排除 heredoc 正文；bench/cases 追加本例到 39/40 同族
- rollback:revert 落地 commit（guard 回到字面匹配，只是更保守）
- trace:loop-notes.md 2026-09-05「agent-on-git-guard 误拦」；2026-09-06 结账当场再拦（本卡 evidence ③）
- 状态: landed@同批（入册 bench/cases/46 + 规则落 multi-contributor §三½.5 判据面 / 执行面 + kit/guard/README「执行面自检」）。**根因改判**：09-21 草稿写的「闸体修复 deferred」是误诊——现行 Rust 闸（cli/src/guard.rs）早已按目标仓判，干跑对照老闸 exit 2 / Rust 闸 exit 0；误拦出自插件缓存 0.5.0 里的老 Python 闸，属执行面陈旧（案 47），本机修法 `claude plugin update agent-on@agent-on`

### plan-revision-must-rewrite-self-contained-cards（计划正文改版后，自包含的 phase 卡与 config 必须同批重写，并用机器断言守门）
- source:CryptoQuant @ 12f1b51 | pin v0.19.0
- evidence:/autoplan 三阶段审查（`a544484`）六个审稿声音（CEO / Eng ×2 / DX ×2 / Design）独立命中同一条：计划正文已改到 v4 节奏，而自包含的 11 张 phase 卡与 config 仍是 v3；`docs/plans/p0-plan-review.md` 跨阶段主题第 1 条；修补 `12f1b51 docs+config(p0.0b)`：正文 v5、11 张卡重写、config 三份按 `docs/plans/p0-contract.yaml`、新增 `tests/test_contract.py`（6 条）把常量钉住
- confidence:medium（单项目，但六路独立审查收敛 + 后续 5 张卡执行期零节奏漂移）
- claim:计划一旦修订，同一 commit 里必须重写所有「自包含」派生物（phase 卡、config、contract），并写一组 contract 测试让「卡 / 配置 / 计划正文」的共享常量只有一个真相源；自包含 ≠ 可以各自过期。
- suggested_landing:playbook/phase-gates.md（或 iteration-loop.md 规划节）加「修订同批重写派生物 + contract 测试」一段；kit/phase-card-template.md 头部 required_context 示例加 contract 指针
- rollback:revert 落地 commit
- trace:loop-notes.md 2026-09-05「/autoplan 六个审稿声音都命中同一条」
- 状态: deferred（需与 playbook/phase-gates.md 规划节、iteration-loop.md 规划节同批重写，避免与既有规划链措辞打架；单点项目证据）

### verify-external-facts-in-planning-with-probes（规划期外部 API/限额事实一律实测，不凭记忆写）
- source:CryptoQuant @ 712c403 | pin v0.19.0
- evidence:规划草稿 6 处外部事实凭记忆写错（宇宙 2,400→实测 3,500 合约；Bybit 限速「按端点」→ 600/5s 每 IP；CoinGecko Demo 30/min→100/min；Bitget/Gate 有多空比端点；OKX 爆仓 REST 已下架；ccxt 是 aiohttp 非 httpx），被 Plan 子代理实测推翻后重写进 `docs/plans/p0-plan.md`「已核实的交易所事实」表（`712c403`）；因此 plan mode 产出细粒度计划（lock local_deviations 第 3 行），粗 plan 会把错误带进实现
- confidence:medium（6/6 错误全在外部事实上，零在设计判断上；单项目）
- claim:规划里凡是外部系统的事实（限额、端点、字段、库的运行时）必须带「实测于 <日期>」的证据行，没实测的写「待核」而不是写一个数；用子代理跑探针比事后返工便宜一个数量级。
- suggested_landing:playbook/anti-hallucination.md 加「外部事实必带实测戳」一条；BOOTSTRAP §1.5 第 5 步 rough plan 注明「外部事实表允许细」；bench 案例
- rollback:revert 落地 commit
- trace:loop-notes.md 2026-09-05「规划草稿里 6 处外部 API 事实凭记忆写错」；agent-on.lock.md local_deviations 第 3 行
- 状态: landed@同批（anti-hallucination C 附6 第 11 条「外部事实必带实测戳」）

### serialize-live-checks-per-ip（同一出口 IP 上的活体验证要串行，限速压测期间不得再打同一所）
- source:CryptoQuant @ b58a4da | pin v0.19.0
- evidence:`scripts/gate_ramp.py` 以 10 req/s 对 Gate 持续压测的同时跑活体 `cq markets coverage`，coverage 的 Gate contracts（1.3 MB）请求 20 s 超时，差点把「Gate 覆盖率」误判成失败；CLI 因此补「单所失败只记 error 不拖垮整表」（`src/cryptoquant/cli/markets.py`，`48fe8c1` / `b58a4da`）
- confidence:low（单项目一次）
- claim:活体验证共享外部配额（同 IP / 同 key）时按所串行，压测与验收不能并行；活体证据里注明「当时是否有并行外部负载」。
- suggested_landing:playbook/sop.md 活体验证一节加一句；kit/phase-card-template.md 活体验收行示例注「串行」
- rollback:revert 落地 commit
- trace:loop-notes.md 2026-09-06「同一 IP 上并行跑 Gate 压测与活体 coverage」
- 状态: landed@同批（anti-hallucination C 附6 第 9 条 + phase-card 验收区「活体」条目）

### pipeline-masks-test-exit-code（门禁命令接管道会吞掉 pytest 的退出码，红测试照样进库）
- source:CryptoQuant @ 86faffe | pin v0.19.0
- evidence:`set -e` 下 `uv run pytest -q | tail -3 && <提交>` 以 tail 的退出码为准，`9a4a967 feat(p0.4)` 带着 1 个红测试进库；`86faffe` / `c8200aa` 修正；之后门禁一律 `uv run pytest -q || exit 1` 或 `set -o pipefail`，本项目后续 10 个 commit 零复发
- confidence:medium（shell 语义确定；agent 写门禁链时的常见坑）
- claim:「完成 = 贴证据」的证据命令不许接管道；门禁链用 `set -o pipefail` + 显式 `|| exit 1`，并且证据输出要在提交之前被读到而不是被 tail 吃掉。
- suggested_landing:kit/merge-checklist.md 或 sop 的「提交前门禁」行；bench 案例
- rollback:revert 落地 commit
- trace:loop-notes.md 2026-09-06「`set -e` 挡不住管道里的 pytest 失败」
- 状态: landed@同批（anti-hallucination C 附6 第 1 条「证据命令不许接管道」+ kit/merge-checklist.md 2e 门禁链不接管道）

### fixture-absence-must-be-explicit（靠「fixture 目录碰巧不全」制造失败的测试，会在补录 fixture 后悄悄变绿）
- source:CryptoQuant @ 1283aa6 | pin v0.19.0
- evidence:Binance OI fixtures 从 3 个补录到全宇宙 524 个后，`tests/test_oi_funding.py::test_open_interest_job_partial_and_reuse_ticker` 靠「SOLUSDT 没 fixture」造 partial 的断言变成 ok 而红；改为伪造一个不存在的合约 `ZZZNOFIXTUREUSDT` 显式制造缺失（`1283aa6`）
- confidence:low（单项目一次；机理对所有录制回放式测试普适）
- claim:需要「缺数据」场景的测试必须显式构造缺失（假 id / 明确删除），不能依赖真实 fixture 目录当前恰好不含某项；fixture 补全应是纯增量、零测试语义变化。
- suggested_landing:playbook/anti-hallucination.md 或 sop 测试纪律「fixture 与替身」一段；bench 案例
- rollback:revert 落地 commit
- trace:loop-notes.md 2026-09-06「把 Binance OI fixtures 补录到全宇宙 524 个后…」
- 状态: landed@同批（anti-hallucination C 附6 第 3 条）

### transient-errors-must-not-be-marked-permanent（验收指标按语义不按表面形态；暂时性错误不得被长任务永久标记）
- source:CryptoQuant @ 631990e | pin v0.19.0
- evidence:365 天回填叠上整点在线槽，Bybit 连回 4 个 retCode 10006（HTTP 200）：卡上的验收「429/418 = 0」按表面形态写，没抓到业务码限速；回填把任何异常当「历史不可用」永久标 done+error（SQLite commands #2 result JSON `errors` 4 条）；修复 `631990e`：分类后暂时性错误保水位 + 整所冻结 + 旧行按错误文本前缀重试，`tests/test_backfill.py::test_backfill_transient_error_keeps_watermark_freezes_and_retries` 先红后绿；同批 `docs/requirements.md` D19
- confidence:medium（单项目；「按语义写验收」与「暂时/永久分类」两条都是通用纪律）
- claim:验收条目要写被测语义（「未触发任何限速信号」）而不是某个载体形态（HTTP 429）；长时任务对错误必须先分类，暂时性错误只允许「保进度 + 退避 + 重试」，永久标记只给确认过的永久错误。
- suggested_landing:kit/phase-card-template.md 验收标准示例（按语义写）；playbook/sop.md 或 error-signal 一节「暂时/永久分类」；bench 案例
- rollback:revert 落地 commit
- trace:loop-notes.md 2026-09-06「限速红线被碰」
- 状态: landed@同批（anti-hallucination C 附6 第 7 条）

### resumability-tests-must-cross-key-boundary（「可续跑」的测试必须让时钟跨过一次键值边界再跑第二次）
- source:CryptoQuant @ 73dde38 | pin v0.19.0
- evidence:回填水位键用绝对 (range_start, range_end)，两者每整点各挪一格；单测两次「续跑」都在同一虚拟时刻，全绿；活体 cq run 停一小时后直连重投 Bybit，747 个市场 365 天全部重下（直连输出「4868 请求 · 4409942 根」）；`73dde38` 改为「起点被旧范围覆盖」的最近水位续跑，新测试 `test_backfill_progress_survives_hour_drift` 让虚拟时钟前进 1h / 2h 再跑
- confidence:medium（单项目；机理对任何以时间派生键的可续跑/幂等设计普适）
- claim:凡声称可续跑 / 幂等 / 去重的机制，测试至少包含一次「时间跨过键值的自然边界（整点 / 日界 / 分页边界）后重跑」，否则测的只是同一时刻的重复调用。
- suggested_landing:playbook/sop.md 测试纪律加一条；kit/phase-card-template.md 的「可续跑」类验收示例；bench 案例
- rollback:revert 落地 commit
- trace:loop-notes.md 2026-09-06「回填水位键用了绝对 (range_start, range_end)」
- 状态: landed@同批（anti-hallucination C 附6 第 6 条）

### replay-double-writes-more-than-production（回放替身写回的属性比线上多，单测全绿而线上分类失效）
- source:CryptoQuant @ 4e83d7a | pin v0.19.0
- evidence:`src/cryptoquant/testing/fixtures.py:208-211` 回放时把 `last_json_response` 写回 ccxt 实例；真 ccxt `base/exchange.py:343` 默认 `enableLastJsonResponse = False` 从不写；错误分类层在单测里认得 Bybit 10006，线上判成 `parse`（非暂时性）→ 12 个市场的回填水位被永久标 done。修复 `4e83d7a`：工厂开开关 + handle_errors 包装层记 body；新测试 `tests/test_fixture_replay.py::test_factory_records_parsed_body_for_error_classification` 走 ccxt bybit 的真 handle_errors 路径而非替身
- confidence:high（根因在第三方库源码里逐行核实；replay/live 分叉是录制回放体系的结构性风险）
- claim:替身（replayer / mock）写回被测对象的每一个属性都要回答「线上谁来写」；错误分类等安全关键路径至少要有一条测试穿过真实库的错误处理入口，而不是全靠替身。
- suggested_landing:playbook/anti-hallucination.md「替身比线上慷慨」一条；kit 的 fixtures/README 模板（若有）加「写回属性对照表」；bench 案例
- rollback:revert 落地 commit
- trace:loop-notes.md 2026-09-06「回放替身比线上『更慷慨』」
- 状态: landed@同批（anti-hallucination C 附6 第 4 条）

### fake-echoes-assumption-live-audit-required（假交易所只是复读我的假设；活体后必须做逐项完整率审计）
- source:CryptoQuant @ bd4c148 | pin v0.19.0
- evidence:Bitget 分页按「endTime 含」实现，假交易所也按「含」写，`test_pagination_no_gap_no_overlap` 全绿；活体 `scripts/klines_completeness.py` 一跑：462/468 市场各缺 3 根，缺口间距 201 h = 每页 200 根边界各丢一根；`bd4c148` 游标改 ts−1 ms（含/不含都不丢不重），测试对 Bitget 同时跑两种语义（`[bitget_inclusive]` 参数），`--force` 重灌后 100%
- confidence:medium（单项目；对任何「按外部语义实现 + 用自写替身验证」的接入层普适）
- claim:外部系统的语义（含/不含、升/降序、单位）只能由实测或录制的真实响应证明，自写替身不算证据；实现允许两种语义都正确时优先选它；活体之后必须跑一次覆盖全体对象的完整率审计，抽样看几根不算验收。
- suggested_landing:playbook/anti-hallucination.md「替身不是证据」；kit/phase-card-template.md 活体验收示例加「全量审计脚本」；bench 案例
- rollback:revert 落地 commit
- trace:loop-notes.md 2026-09-06「Bitget 分页按『endTime 含』写」
- 状态: landed@同批（anti-hallucination C 附6 第 5 条 + phase-card 验收区「活体·全量完整率审计」）

### skip-mrd-when-north-star-doc-complete（用户自带完整北极星文档时，BOOTSTRAP 规划链的 MRD / PRD 模板环节可省，用修订记录承接偏差）
- source:CryptoQuant @ 712c403 | pin v0.19.0
- evidence:用户交付的北极星文档已含范围 / 信号定义 / 验收 / 里程碑 / 决策 / 待确认；BOOTSTRAP §1.5 第 2 步 MRD（/office-hours）与第 4 步 PRD 模板未走，直接存为 `docs/product/prd.md` 并以修订记录承接三处偏差（`712c403`）；后续 /autoplan 三阶段全部批准，未因缺 MRD 返工；agent-on.lock.md local_deviations 第 2 行
- confidence:low（单项目一次）
- claim:BOOTSTRAP 规划链按「输入已具备的要素」跳步：北极星文档已覆盖 MRD/PRD 要素时不重走模板，只做要素对表 + 修订记录；跳步要在 lock 的 local_deviations 登记。
- suggested_landing:BOOTSTRAP §1.5 加「输入已完整可跳步」的判据行；kit/agent-on-lock-template.md local_deviations 示例
- rollback:revert 落地 commit
- trace:agent-on.lock.md local_deviations 第 2 行
- 状态: deferred（单点证据 confidence low；BOOTSTRAP §1.5 规划链表加「跳步判据」需与 boot/new-project-questionnaire.md 联动，单独立轨）

### capability-probe-conflicts-with-settlement-budget（lock 模板要求首次结账前跑 1 小时能力探针，与结账「一句口令 + 零确认」预算冲突）
- source:CryptoQuant @ b51524e | pin v0.19.0
- evidence:`kit/agent-on-lock-template.md` 实例化行「保费档位:甲(按模型层级预设,未跑 bench/capability-probe;换模型或首次结账前补跑并回填)」；`bench/capability-probe.md` 标题「新模型入编考试,~1 小时」；`boot/settlement.md` 负担预算「结账 = 一句口令 + 零确认」且「超预算的步骤,记 loop-notes 并当场简化」——本次首次结账按后者办：不跑探针，档位保持预设，登记 local_deviations
- confidence:medium（两份 canonical 文本可直接核对的冲突）
- claim:能力探针不挂在「首次结账前」，改挂在独立口令或 BOOTSTRAP 定档后的可选步骤；lock 模板那行改成「未跑探针 → 档位为预设，需实测时说『跑能力探针』」。
- suggested_landing:kit/agent-on-lock-template.md 该行措辞；bench/capability-probe.md 头部「何时跑」；boot/settlement.md 不改
- rollback:revert 落地 commit
- trace:agent-on.lock.md local_deviations 第 4 行（本次结账新增）
- 状态: landed@同批（kit/agent-on-lock-template.md 档位行改措辞 + bench/capability-probe.md 头部新增「何时跑」）
