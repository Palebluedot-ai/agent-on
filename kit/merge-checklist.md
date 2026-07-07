# 合流七步 Checklist(orchestrator 每次照抄)

> 双轨模式的第三环:轨道各自绿 ≠ 合流绿 ≠ 边缘出口绿。Run #3(撞车修复)与 Run #7(生产 canary 抓平台行为)的固化。

- [ ] **1. 顺序合流**:契约变更(若有)先进 main;再 merge 各轨 worktree 分支(`--no-ff` 留 run 痕)。
- [ ] **2. 全量双端回归**:不是只跑新增——[后端全量命令] + [前端全量命令] + 类型检查;任何红=先修再继续。
- [ ] **3. 悬点逐条裁决**:两轨报告的「契约悬点」并集,逐条定谁改(契约跟正/实现跟正/记账后置),写进 progress notes。
- [ ] **4. 翻转 Fake→真**:provider/开关切真实现;**grep 所有「进壳/进页」测试确认钉了全量 Fake overrides**(Run #3 教训:只钉 auth 会裸奔)。翻转后再跑一遍全量。
- [ ] **5. 部署 + 生产 canary**(有部署面时):部署后跑边缘出口的 LIVE 探针——平台会剥头/整形/缓存,本地绿不算数(Run #7:Vercel 剥 Server-Timing/304 剥光)。
- [ ] **6. 上机人验**:装真机/模拟器,给用户一句「去点哪里看什么」。
- [ ] **7. 记账五件套**:progress.yaml 状态 + run-ledger 一行(时长/冲突/悬点/返工/成本)+ loop-notes 教训(如有)+ qa 三桶(如有)+ commit&push;清理已合流 worktree。

## DoD 门禁(一代移植)

> 源流:一代 agent-orchestration-playbook/05_playbooks/execution-layer-dod-v1.md,2026-07-07 批二移植。
> 上面七步管「怎么合」;这一节管「什么才算合完」——七步全打勾但下面四条不满足,不许标 done。

一次编排的完成标准不是「能跑」,而是四条同时成立:

1. **可编排**:状态机稳定,任务始终停在某个已定义状态,没有悬空。
2. **可审计**:每次状态变更都有 audit_event,事件轨迹从 queued 到 done 完整不断。
3. **可追责**:每个决策能说清是谁、对着哪张卡、基于什么证据做的(result_card 的 decisions + evidence_links)。
4. **可回放**:失败能凭 run_id + audit 轨迹定位到出错那一步,不靠记忆复盘。

### 最小卡集规则

durable(要进主干、算数)的任务,`task_card` / `result_card` / `memory_card` **缺任一张不得标 done**。Explore 车道的抛弃产物不受此约束。

### 固定状态机

`queued → triaged → planned → running → reviewing → done | failed`

只许走 `kit/schemas/audit-event.schema.json` 定义的合法迁移,跳级(如 queued 直接 running)由 `ledger/audit-lint.py` 拦下。`reviewing=revise` 不得标 done;迁到 failed 必须记 error_type。
