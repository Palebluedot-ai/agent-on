# intake:inbox-radar 2026-08-15 结账（自 2026-07-31 增量，3 卡）

> 范围：`last_settlement` 之后 loop-notes 与主线提交里可复用的 **AI 协作过程** 教训。开户球权/GTT/补件清单等 **业务域规则** 留项目 `loop-notes`（仍 `sync_status=local`），不出仓。

### llm-fallback-empty-must-not-erase-extraction（schema 空数组 fallback 不得抹掉已抽出的结构化清单）
- source:inbox-radar @ 3adc3ad | pin v0.5.1
- evidence:`3adc3ad fix: encode 2026-08-13 gold-set rules on the live analysis path` 同时改 `prompt_contract`（仅在正文真无补件要求时才允许空 outstanding）、`find_key_emails`/email_type 升级、以及 `llm_output_validation`/pipeline 在 outstanding 非空时覆盖「无待办」一句话；单测 `tests/test_reconcile_summary_outstanding.py`、`tests/test_prompt_gold_set_outstanding.py`、`tests/test_preprocessor_find_key_emails.py` 锁住回归。
- confidence:medium（单项目金标复现；机理对「可选空数组 + 分类重路由」的 LLM 管道普适）
- claim:当输出 schema 允许 empty-array fallback 时，空数组只代表「结构真的不存在」，不能因分类掉进 fallback 路径而抹掉正文已抽出的清单；分类/重路由之后仍须用确定性校验：结构化 outstanding 非空时，禁止 LLM 摘要把状态写成「无待办」。
- suggested_landing:playbook/anti-hallucination 或 LLM 输出校验一节；kit 里 structured-output / prompt contract 检查行；bench 短案例
- rollback:revert 落地 commit
- trace:loop-notes.md §2026-08-13「fallback 空数组会吞掉正文清单」
- 状态:landed@同批（C 附4 + phase-card 输出校验）

### no-invented-directory-without-human-confirm（目录/花名册条目不得仅凭邮箱出现在 mailbox 就发明）
- source:inbox-radar @ 03fe9d4 | pin v0.5.1
- evidence:`03fe9d4 feat: register JC on the sales roster and assign KING OF ROLLS` 仅在 Chao 确认后把 `jc.jiao@hashkey.com` 写入 `config/team-rules.json` / `team.json`，并 regenerate Worker actor directory（`0817502`）；此前 loop-notes 明确禁止「mailbox 里已有邮箱就发明花名册行」。测试 `tests/test_roster_resolve.py`、`tests/test_preprocessor_detect_owner.py`。
- confidence:medium（单项目；与 anti-hallucination「不发明身份」同构，适用于任何从通讯痕迹反推组织目录的系统）
- claim:从邮件/IM 痕迹推断组织目录时，**出现过的邮箱 ≠ 可写花名册**；只在人类确认后登记身份，再 regenerate 派生 actor/open_id 映射。禁止 agent 为了「分到人」而 invent roster 行。
- suggested_landing:playbook/anti-hallucination 身份/目录纪律；kit/roster 或 actor-directory 生成 checklist 行
- rollback:revert 落地 commit
- trace:loop-notes.md §2026-08-13「花名册只登记 Chao 确认的邮箱」
- 状态:landed@同批（C 附5 + AGENTS-skeleton 不发明花名册）

### feature-delete-must-not-collapse-shared-domain-terms（删功能先划标识边界，勿把共享业务词一并拆掉）
- source:inbox-radar @ 57cc3b6 | pin v0.5.1
- evidence:`57cc3b6 remove onboarding pack send; keep case tracking` 删除 pack 资产/planner/Worker 发开户包链路与 action types，但保留 preprocessor `category=onboarding`、`onboarding-cases.json` 读者、override category；验证：`uv run python -m pytest` 1411 passed、`workers/lark-callback` npm test 131 passed；授权对已删 action type fail-closed。
- confidence:medium（单次大规模删功能；口令原文易被误读成「一切 onboarding」）
- claim:移除「某产品面」时先冻结 **标识边界**（如 `onboarding_pack_*` / 发开户包 vs 业务词 onboarding 的 case 跟踪）；先删叶子模块与资产，再拆授权与入口接线；共享领域词上的 pipeline/dashboard/override **默认保留**，除非证据证明只服务被删面。
- suggested_landing:playbook 变更范围/删功能纪律；kit/merge-checklist 或 plan 模板「shared term 不随 feature 删除」行
- rollback:revert 落地 commit
- trace:2026-08-14 会话口令「发送开户文件 onboarding…可删」+ 明确「不要删除多了」；commit 57cc3b6
- 状态:pending
