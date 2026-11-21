# intake:SalesDashboard 2026-08-15 结账（首次·3 卡）

> 背景一句话：OTC 销售看板接上 Graph 日更与 Vite 客户端后，三处「看起来已经有了」翻车——日历任务开火当成功、看见额外列表头当错格式、客户端 barrel 把 node:fs 打进浏览器。三卡都是协作/验收顺序，不含销售口径。

### calendar-agent-fired-is-not-success（日历任务开火 ≠ 无人值守成功）
- source:SalesDashboard @ 7691c2a | pin v0.9.1
- evidence:launchd `com.chao.salesdashboard.sync` 已装，`StartCalendarInterval` Hour=10 Minute=15，`TZ=Asia/Hong_Kong`。2026-08-15 `launchctl print`：`runs = 1`、`last exit code = 1`；`SalesDashboard/logs/sync-stderr.log` 为 Graph token POST 网络失败（`nodename nor servname provided, or not known`）。stdout 空。对照：同一机器交互壳里 `npm run data:sync -- --xlsx …` 能 replace 成功（9912 行）。
- confidence:medium（单项目一次真开火；与「plist 在 ≠ cron 绿」「权限勾了 ≠ 事件到」同构）
- claim:验收定时任务时，**安装成功与日历开火都不能当完成**。完成条件固定三件：`launchctl`/`cron` 的 runs+exit、任务自己的 stdout/stderr、以及下游产物（文件/行数）——缺日志或 exit≠0 必须标失败。禁止用「agent 已 load / 到点了」推断拉取已通。
- suggested_landing:playbook 外部依赖/运行时验收；bench 短案例；kit 交付 checklist 一行「定时：runs+exit+产物」
- rollback:revert 落地 commit
- trace:loop-notes.md L9；phase-s1.2-launchd-graph-network.md；install commit 7691c2a
- 状态:landed@同批（C 附3 + sop 12 + merge 5b + bench 33）

### browser-entry-must-not-reexport-node-fs（浏览器入口禁止再导出 Node 写盘）
- source:SalesDashboard @ 974d843 | pin v0.9.1
- evidence:用户打开看板报 `Module "node:fs" has been externalized… Cannot access "node:fs.existsSync" in client code`。当时 `src/sales/index.ts` 从同一 barrel 导出 `useSalesRecords` 与 `import-local`/`store`（`node:fs`）。修复：barrel 只留读数；API 改为 `~/sales/ingest`、`~/sales/store`。`npm test` 40 pass，含 `client sales barrel does not pull node:fs ingest/store`。
- confidence:high（Vite 官方明确 externalize node builtins；任何「一个 index 打给 SSR+CSR」的 TS 仓都会炸）
- claim:给浏览器/CSR 的公共入口**不得 re-export** 引用 `node:fs`/`path`/`child_process` 的模块。读数与写盘分成两个入口；dashboard 只 import 读数。禁止用「反正 tree-shake」赌打包器不会把 store 打进客户端。
- suggested_landing:playbook 前端/bundler 边界；kit AGENTS 或 review checklist 一行；bench 案例
- rollback:revert 974d843 的 barrel 拆分（会把看板打回红）
- trace:loop-notes.md L11；commit 974d843
- 状态:landed@同批（AGENTS-skeleton + review-prompt + bench 36）

### map-required-fields-before-header-reject（多一列表头先映射，禁止整表当错格式）
- source:SalesDashboard @ 974d843 | pin v0.9.1
- evidence:旧 `assertDailyFormat` 看见「销售分组 / KYC日期 / opt手续费」即抛「这是 OTC 组明细格式」。对两份真实 xlsx：重叠 5371 键总收入全等，且 `OTC交易=opt手续费`、`OrderBook=现货手续费` 全等。拒收挡住日更源。修后按列映射 + 空名零金额行跳过；`npm test` 含 `maps mail 组明细 headers` / `mail 组明细 xlsx replace-imports`。
- confidence:medium（一次产品线，但「用标记列杀整表」是通用 ingest 幻觉）
- claim:ingest/导入遇到**多出来的列**时，先核必填列能否映射、抽几行对账；**禁止**「看见某标记列 → 整文件错误格式」。真缺必填列再拒。空名且金额为 0 的行可跳过，不要为此失败整批。
- suggested_landing:playbook anti-hallucination / 数据导入验收；kit ingest 或 review 一行；bench 短案例
- rollback:revert 974d843 的 ingest 映射（日更会再次整表拒收）
- trace:loop-notes.md L10；commit 974d843
- 状态:rejected(项目域 ingest 口径,用户拍板不出仓)
