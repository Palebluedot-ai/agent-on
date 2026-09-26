#!/usr/bin/env python3
"""merge-audit 的测试。全部离线（fixture + tempfile），不连 GitHub。

跑法：
    python3 -m unittest discover -s tools/merge-audit -v
"""

import json
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import merge_audit as ma  # noqa: E402

POLICY = json.loads((Path(__file__).resolve().parent / "policy.json").read_text(encoding="utf-8"))


def pr(number=1, title="t", author="Palebluedot-ai", files=(), body="",
       labels=(), review="", checks=(), merged_at="2026-08-20T00:00:00Z", oid="abc123",
       draft=False, mergeable="MERGEABLE", changed_files=None):
    d = {
        "number": number, "title": title, "body": body,
        "author": {"login": author},
        "files": [{"path": p} for p in files],
        "labels": [{"name": n} for n in labels],
        "reviewDecision": review,
        "statusCheckRollup": [{"name": n, "conclusion": c} for n, c in checks],
        "mergedAt": merged_at, "mergeCommit": {"oid": oid},
        "isDraft": draft, "mergeable": mergeable,
    }
    if changed_files is not None:
        d["changedFiles"] = changed_files
    return d


def claim(pr_no, claimed, note=""):
    return {"kind": "claim", "action": "merged", "pr": pr_no, "claimed": claimed, "note": note}


class TestGlob(unittest.TestCase):
    def test_star_does_not_cross_slash(self):
        rx = ma.glob_to_regex("hooks/*")
        self.assertTrue(rx.match("hooks/a.json"))
        self.assertFalse(rx.match("hooks/deep/a.json"))

    def test_doublestar_crosses_slash(self):
        rx = ma.glob_to_regex("hooks/**")
        self.assertTrue(rx.match("hooks/deep/a.json"))

    def test_leading_doublestar_matches_root_and_nested(self):
        rx = ma.glob_to_regex("**/.env")
        self.assertTrue(rx.match(".env"))
        self.assertTrue(rx.match("app/config/.env"))
        self.assertFalse(rx.match("env"))

    def test_no_partial_match(self):
        rx = ma.glob_to_regex("AGENTS.md")
        self.assertFalse(rx.match("kit/AGENTS.md"))
        self.assertFalse(rx.match("AGENTS.md.bak"))


class TestClassify(unittest.TestCase):
    def test_plain_docs_is_auto(self):
        v = ma.classify(pr(files=["snapshot/x.md", "bench/cases/9.md"]), POLICY)
        self.assertEqual(v["decision"], "AUTO")

    def test_touching_own_permission_config_is_hard_stop(self):
        v = ma.classify(pr(files=[".claude/settings.local.json"]), POLICY)
        self.assertEqual(v["decision"], "HARD_STOP")
        self.assertEqual([r["id"] for r in v["hard_stop"]], ["gate-and-permissions"])

    def test_touching_the_auditor_itself_is_hard_stop(self):
        """被监控者改监控自己的工具——这是本清单存在的头号理由。"""
        v = ma.classify(pr(files=["tools/merge-audit/policy.json"]), POLICY)
        self.assertEqual(v["decision"], "HARD_STOP")
        self.assertIn("gate-and-permissions", [r["id"] for r in v["hard_stop"]])

    def test_touching_route_gate_source_is_hard_stop(self):
        v = ma.classify(pr(files=["cli/src/guard.rs"]), POLICY)
        self.assertEqual(v["decision"], "HARD_STOP")

    def test_untrusted_author_is_hard_stop_even_for_a_typo(self):
        v = ma.classify(pr(author="drive-by", files=["README.md"]), POLICY)
        self.assertEqual(v["decision"], "HARD_STOP")
        self.assertIn("untrusted-author", [r["id"] for r in v["hard_stop"]])

    def test_workflow_change_is_hard_stop(self):
        v = ma.classify(pr(files=[".github/workflows/ci.yml"]), POLICY)
        self.assertEqual(v["decision"], "HARD_STOP")

    def test_whole_github_dir_is_hard_stop_including_templates(self):
        """刻意整括 `.github/**`，连 issue 模板一起停。

        枚举子目录的代价是「下一个 `.github/actions/` 又是同一个洞」；
        整括的代价是 issue/PR 模板改动也要问一次——实测最近 36 条 PR 里只有 1 条碰
        `.github/`（3%），这个代价买断一整类盲区，划算。
        """
        v = ma.classify(pr(files=[".github/ISSUE_TEMPLATE/intake-card.md"]), POLICY)
        self.assertEqual(v["decision"], "HARD_STOP")

    def test_breaking_marker_is_hard_stop(self):
        """用户 2026-08-20 拍板：带 breaking 标注硬停（信号便宜，漏了贵）。"""
        v = ma.classify(pr(files=["playbook/sop.md"], body="正文\n\nBREAKING: 口令改名"), POLICY)
        self.assertEqual(v["decision"], "HARD_STOP")
        self.assertIn("breaking-or-migration", [r["id"] for r in v["hard_stop"]])

    def test_governance_doc_is_notable(self):
        v = ma.classify(pr(files=["AGENTS.md"]), POLICY)
        self.assertEqual(v["decision"], "NOTABLE")

    def test_hard_stop_beats_notable(self):
        v = ma.classify(pr(files=["AGENTS.md", "hooks/hooks.json"]), POLICY)
        self.assertEqual(v["decision"], "HARD_STOP")

    def test_migration_dir_is_hard_stop(self):
        v = ma.classify(pr(files=["db/migrations/001_init.sql"]), POLICY)
        self.assertEqual(v["decision"], "HARD_STOP")

    def test_evidence_names_the_actual_file(self):
        v = ma.classify(pr(files=["hooks/hooks.json"]), POLICY)
        self.assertIn("文件 hooks/hooks.json", v["hard_stop"][0]["evidence"])


class TestSecretsInDiff(unittest.TestCase):
    def test_added_private_key_is_hard_stop(self):
        diff = "+++ b/a.txt\n+-----BEGIN RSA PRIVATE KEY-----\n"
        v = ma.classify(pr(files=["a.txt"]), POLICY, diff)
        self.assertEqual(v["decision"], "HARD_STOP")
        self.assertIn("secrets-and-credentials", [r["id"] for r in v["hard_stop"]])

    def test_removed_secret_is_not_flagged(self):
        """删掉一个密钥不是引入一个密钥。只扫新增行。"""
        diff = "--- a/a.txt\n--------BEGIN RSA PRIVATE KEY-----\n"
        v = ma.classify(pr(files=["a.txt"]), POLICY, diff)
        self.assertEqual(v["decision"], "AUTO")

    def test_aws_key_shape_in_added_line(self):
        diff = "+++ b/a.py\n+AWS_KEY = 'AKIAIOSFODNN7EXAMPLE'\n"
        v = ma.classify(pr(files=["a.py"]), POLICY, diff)
        self.assertEqual(v["decision"], "HARD_STOP")

    def test_deep_flag_recorded(self):
        self.assertFalse(ma.classify(pr(files=["a.md"]), POLICY)["deep"])
        self.assertTrue(ma.classify(pr(files=["a.md"]), POLICY, "")["deep"])


class TestHealth(unittest.TestCase):
    def test_failing_check_is_red(self):
        v = ma.classify(pr(files=["a.md"], checks=[("GitGuardian", "FAILURE")]), POLICY)
        self.assertTrue(v["health"]["red"])

    def test_success_check_is_green(self):
        v = ma.classify(pr(files=["a.md"], checks=[("GitGuardian", "SUCCESS")]), POLICY)
        self.assertFalse(v["health"]["red"])

    def test_changes_requested_is_red(self):
        v = ma.classify(pr(files=["a.md"], review="CHANGES_REQUESTED"), POLICY)
        self.assertTrue(v["health"]["red"])


class TestLedgerChain(unittest.TestCase):
    def setUp(self):
        self.dir = tempfile.TemporaryDirectory()
        self.path = Path(self.dir.name) / "merge-audit.jsonl"

    def tearDown(self):
        self.dir.cleanup()

    def test_chain_holds_across_appends(self):
        for n in (1, 2, 3):
            ma.append_record(self.path, {"kind": "claim", "pr": n})
        _, raw = ma.read_ledger(self.path)
        self.assertEqual(len(raw), 3)
        self.assertIsNone(ma.verify_chain(raw))

    def test_first_record_is_genesis(self):
        rec = ma.append_record(self.path, {"kind": "claim", "pr": 1})
        self.assertEqual(rec["prev"], ma.GENESIS)

    def test_rewriting_history_breaks_the_chain(self):
        for n in (1, 2, 3):
            ma.append_record(self.path, {"kind": "claim", "pr": n})
        _, raw = ma.read_ledger(self.path)
        rec = json.loads(raw[1])
        rec["pr"] = 999
        raw[1] = json.dumps(rec, ensure_ascii=False, sort_keys=True)
        self.path.write_text("\n".join(raw) + "\n", encoding="utf-8")
        _, raw2 = ma.read_ledger(self.path)
        self.assertEqual(ma.verify_chain(raw2), 3)

    def test_deleting_a_record_breaks_the_chain(self):
        for n in (1, 2, 3):
            ma.append_record(self.path, {"kind": "claim", "pr": n})
        _, raw = ma.read_ledger(self.path)
        del raw[1]
        self.path.write_text("\n".join(raw) + "\n", encoding="utf-8")
        _, raw2 = ma.read_ledger(self.path)
        self.assertEqual(ma.verify_chain(raw2), 2)


class TestFindings(unittest.TestCase):
    def test_hard_stop_auto_merged_is_a_violation(self):
        """硬停单被当成 AUTO 合了才是 VIOLATION——这才是「没问就合」。"""
        v = ma.classify(pr(1, files=["hooks/hooks.json"]), POLICY)
        f = ma.collect_findings([v], {1: [claim(1, "AUTO")]}, None)
        self.assertIn("VIOLATION", [x["level"] for x in f])

    def test_approved_hard_stop_with_verifiable_pointer_is_not_a_violation(self):
        """硬停单，claimed=HARD_STOP 且指针 git 可核 = 正确流程，不是越界。
        否则每条经用户批准的 canonical 合并都常亮 VIOLATION（本工具自己上线首跑就中）。"""
        v = ma.classify(pr(1, files=["hooks/hooks.json"]), POLICY)
        f = ma.collect_findings([v], {1: [claim(1, "HARD_STOP", "snapshot/x.md")]}, None,
                                pointer_ok=lambda _: True)
        levels = [x["level"] for x in f]
        self.assertNotIn("VIOLATION", levels)
        self.assertIn("APPROVED_HARDSTOP", levels)

    def test_approved_claim_without_verifiable_pointer_is_unverified(self):
        """claimed=HARD_STOP 但指针核不到 → 不认 APPROVED，降为 UNVERIFIED。
        自称批准可以伪造；要求指针指向 git 可核对象，把「打一个字」抬成「指向可审的东西」。"""
        v = ma.classify(pr(1, files=["hooks/hooks.json"]), POLICY)
        f = ma.collect_findings([v], {1: [claim(1, "HARD_STOP", "")]}, None,
                                pointer_ok=lambda _: False)
        levels = [x["level"] for x in f]
        self.assertIn("UNVERIFIED_HARDSTOP", levels)
        self.assertNotIn("APPROVED_HARDSTOP", levels)

    def test_hard_stop_merged_with_no_record_is_unverified_not_violation(self):
        v = ma.classify(pr(1, files=["hooks/hooks.json"]), POLICY)
        f = ma.collect_findings([v], {}, None)
        levels = [x["level"] for x in f]
        self.assertIn("UNVERIFIED_HARDSTOP", levels)
        self.assertNotIn("VIOLATION", levels)

    def test_merged_without_a_record_is_unrecorded(self):
        v = ma.classify(pr(2, files=["a.md"]), POLICY)
        f = ma.collect_findings([v], {}, None)
        self.assertEqual([x["level"] for x in f], ["UNRECORDED"])

    def test_claim_disagreeing_with_verdict_is_mismatch(self):
        v = ma.classify(pr(3, files=["AGENTS.md"]), POLICY)  # 真相 NOTABLE
        f = ma.collect_findings([v], {3: [claim(3, "AUTO")]}, None)
        levels = [x["level"] for x in f]
        self.assertIn("MISMATCH", levels)

    def test_clean_auto_merge_with_a_record_is_silent(self):
        v = ma.classify(pr(4, files=["snapshot/x.md"]), POLICY)
        f = ma.collect_findings([v], {4: [claim(4, "AUTO")]}, None)
        self.assertEqual(f, [])

    def test_red_merge_is_flagged(self):
        v = ma.classify(pr(5, files=["a.md"], checks=[("CI", "FAILURE")]), POLICY)
        f = ma.collect_findings([v], {5: [claim(5, "AUTO")]}, None)
        self.assertIn("MERGED_RED", [x["level"] for x in f])

    def test_broken_chain_surfaces_even_with_no_pr_problems(self):
        v = ma.classify(pr(6, files=["a.md"]), POLICY)
        f = ma.collect_findings([v], {6: [claim(6, "AUTO")]}, 2)
        self.assertEqual([x["level"] for x in f], ["LEDGER_BROKEN"])

    def test_findings_sorted_worst_first(self):
        v1 = ma.classify(pr(7, files=["AGENTS.md"]), POLICY)
        v2 = ma.classify(pr(8, files=["hooks/hooks.json"]), POLICY)
        f = ma.collect_findings([v1, v2], {7: [claim(7, "NOTABLE")], 8: [claim(8, "AUTO")]}, None)
        self.assertEqual(f[0]["level"], "VIOLATION")


class TestCliOffline(unittest.TestCase):
    """端到端跑 main()，用 --from-file 走离线路径。"""

    def setUp(self):
        self.dir = tempfile.TemporaryDirectory()
        self.d = Path(self.dir.name)
        self.prs = self.d / "prs.json"
        self.ledger = self.d / "l.jsonl"
        # 用一份去掉 ledger_starts_at 的 policy 副本：这一组测的是 report 的管道，
        # 不该被线上 policy.json 里那个日期牵着走（改了日期不该让管道测试变红）。
        pol = dict(POLICY)
        pol.pop("ledger_starts_at", None)
        self.policy = self.d / "policy.json"
        self.policy.write_text(json.dumps(pol, ensure_ascii=False), encoding="utf-8")

    def tearDown(self):
        self.dir.cleanup()

    def _write(self, prs):
        self.prs.write_text(json.dumps(prs, ensure_ascii=False), encoding="utf-8")

    def _base(self):
        return ["--ledger", str(self.ledger), "--policy", str(self.policy)]

    def test_precheck_exit_codes(self):
        self._write([pr(1, files=["a.md"]),
                     pr(2, files=["AGENTS.md"]),
                     pr(3, files=["hooks/h.json"])])
        base = self._base()
        self.assertEqual(ma.main(base + ["precheck", "--pr", "1", "--from-file", str(self.prs), "--json"]), ma.EXIT_AUTO)
        self.assertEqual(ma.main(base + ["precheck", "--pr", "2", "--from-file", str(self.prs), "--json"]), ma.EXIT_NOTABLE)
        self.assertEqual(ma.main(base + ["precheck", "--pr", "3", "--from-file", str(self.prs), "--json"]), ma.EXIT_HARD_STOP)

    def test_report_fails_on_unrecorded_then_passes_after_record(self):
        self._write([pr(1, files=["a.md"])])
        base = self._base()
        rc = ma.main(base + ["report", "--from-file", str(self.prs), "--json"])
        self.assertEqual(rc, 1, "没记账就该红")
        ma.main(base + ["record", "--pr", "1", "--action", "merged", "--claimed", "AUTO"])
        rc = ma.main(base + ["report", "--from-file", str(self.prs), "--json"])
        self.assertEqual(rc, 0, "记完账就该绿")

    def test_report_notable_alone_does_not_fail_by_default(self):
        self._write([pr(1, files=["AGENTS.md"])])
        base = self._base()
        ma.main(base + ["record", "--pr", "1", "--action", "merged", "--claimed", "NOTABLE"])
        self.assertEqual(ma.main(base + ["report", "--from-file", str(self.prs), "--json"]), 0)
        self.assertEqual(
            ma.main(base + ["report", "--from-file", str(self.prs), "--json", "--fail-on", "notable"]), 1)

    def test_scan_write_then_report_reads_same_ledger(self):
        self._write([pr(1, files=["a.md"])])
        base = self._base()
        ma.main(base + ["scan", "--from-file", str(self.prs), "--write", "--json"])
        recs, raw = ma.read_ledger(self.ledger)
        self.assertEqual([r["kind"] for r in recs], ["verdict"])
        self.assertIsNone(ma.verify_chain(raw))




class TestNoFalsePositivesOnProse(unittest.TestCase):
    """2026-08-20 首次对着本仓真实历史跑时抓到的回归：方法论仓的 PR 正文天天
    讨论「删远端分支」「BREAKING」，自由文本搜索把 #35 / #36 全判成硬停。"""

    def test_prose_discussing_irreversible_actions_is_not_hard_stop(self):
        body = ("必须先问档包括：删远端分支、关闭别人的 PR、数据库迁移、跨仓外向操作。"
                "本 PR 只改文档。")
        v = ma.classify(pr(files=["kit/babysit/MERGE-POLICY.md"], body=body), POLICY)
        self.assertNotEqual(v["decision"], "HARD_STOP")

    def test_prose_mentioning_breaking_is_not_notable_by_itself(self):
        v = ma.classify(pr(files=["snapshot/x.md"], body="清单里有一条是「带 BREAKING 标注的 PR」"), POLICY)
        self.assertEqual(v["decision"], "AUTO")

    def test_structured_breaking_footer_is_hard_stop(self):
        v = ma.classify(pr(files=["snapshot/x.md"], body="正文\n\nBREAKING: 口令改名了\n"), POLICY)
        self.assertEqual(v["decision"], "HARD_STOP")

    def test_structured_migration_footer_is_hard_stop(self):
        v = ma.classify(pr(files=["snapshot/x.md"], body="迁移注记：先跑 setup 再升级"), POLICY)
        self.assertEqual(v["decision"], "HARD_STOP")

    def test_breaking_label_is_hard_stop(self):
        v = ma.classify(pr(files=["snapshot/x.md"], labels=["breaking"]), POLICY)
        self.assertEqual(v["decision"], "HARD_STOP")

    def test_prose_mentioning_breaking_is_still_not_flagged(self):
        """正文讨论这个词不算——只认整行前缀标记。方法论仓天天讨论 breaking。"""
        v = ma.classify(pr(files=["snapshot/x.md"], body="清单里有一条是「带 BREAKING 标注的 PR」"), POLICY)
        self.assertEqual(v["decision"], "AUTO")


class TestLedgerStartPoint(unittest.TestCase):
    """机制上线之前的合并没有账可对，不该被永久报成 UNRECORDED——
    但「合了不该合的」不受时间限制。"""

    START = "2026-08-20T00:00:00Z"

    def test_old_merge_without_record_is_not_unrecorded(self):
        v = ma.classify(pr(1, files=["a.md"], merged_at="2026-08-01T00:00:00Z"), POLICY)
        f = ma.collect_findings([v], {}, None, self.START)
        self.assertEqual(f, [])

    def test_new_merge_without_record_is_unrecorded(self):
        v = ma.classify(pr(2, files=["a.md"], merged_at="2026-08-21T00:00:00Z"), POLICY)
        f = ma.collect_findings([v], {}, None, self.START)
        self.assertEqual([x["level"] for x in f], ["UNRECORDED"])

    def test_old_hard_stop_merge_is_reported_as_pre_existing_not_violation(self):
        """存量越界仍要列出来，但不该每轮把退出码钉死在 1——那会训练人忽略报告。"""
        v = ma.classify(pr(3, files=["hooks/h.json"], merged_at="2026-08-01T00:00:00Z"), POLICY)
        f = ma.collect_findings([v], {}, None, self.START)
        levels = [x["level"] for x in f]
        self.assertIn("PRE_EXISTING", levels)
        self.assertNotIn("VIOLATION", levels)

    def test_old_red_merge_is_pre_existing(self):
        v = ma.classify(pr(4, files=["a.md"], checks=[("CI", "FAILURE")],
                           merged_at="2026-08-01T00:00:00Z"), POLICY)
        f = ma.collect_findings([v], {}, None, self.START)
        self.assertIn("PRE_EXISTING", [x["level"] for x in f])

    def test_new_hard_stop_auto_merged_is_a_real_violation(self):
        v = ma.classify(pr(5, files=["hooks/h.json"], merged_at="2026-08-21T00:00:00Z"), POLICY)
        f = ma.collect_findings([v], {5: [claim(5, "AUTO")]}, None, self.START)
        self.assertIn("VIOLATION", [x["level"] for x in f])

    def test_new_hard_stop_no_record_is_unverified(self):
        v = ma.classify(pr(5, files=["hooks/h.json"], merged_at="2026-08-21T00:00:00Z"), POLICY)
        f = ma.collect_findings([v], {}, None, self.START)
        self.assertIn("UNVERIFIED_HARDSTOP", [x["level"] for x in f])


class TestDiffAttribution(unittest.TestCase):
    """密钥证据要说清在哪个文件；审计工具自己的规则定义与夹具不该被自己的规则误报。"""

    DIFF = (
        "diff --git a/app/config.py b/app/config.py\n"
        "--- a/app/config.py\n+++ b/app/config.py\n"
        "+KEY = 'AKIAIOSFODNN7EXAMPLE'\n"
        "diff --git a/tools/merge-audit/policy.json b/tools/merge-audit/policy.json\n"
        "--- a/tools/merge-audit/policy.json\n+++ b/tools/merge-audit/policy.json\n"
        "+    \"AKIA[0-9A-Z]{16}\",\n"
    )

    def test_split_attributes_added_lines_to_files(self):
        per_file = ma.split_diff_by_file(self.DIFF)
        self.assertEqual(sorted(per_file), ["app/config.py", "tools/merge-audit/policy.json"])
        self.assertIn("AKIAIOSFODNN7EXAMPLE", per_file["app/config.py"])

    def test_evidence_names_the_file(self):
        hits = ma.diff_patterns_hit(self.DIFF, [r"AKIA[0-9A-Z]{16}"])
        self.assertTrue(any(h.startswith("app/config.py ") for h in hits))

    def test_auditors_own_files_are_excluded_from_secret_scan(self):
        hits = ma.diff_patterns_hit(self.DIFF, [r"AKIA[0-9A-Z]{16}"],
                                    ["tools/merge-audit/**"])
        self.assertEqual(len(hits), 1)
        self.assertTrue(hits[0].startswith("app/config.py "))

    def test_business_secret_still_caught_with_exclusion_on(self):
        v = ma.classify(pr(files=["app/config.py"]), POLICY, self.DIFF)
        self.assertEqual(v["decision"], "HARD_STOP")
        ids = [r["id"] for r in v["hard_stop"]]
        self.assertIn("secrets-and-credentials", ids)

    def test_headerless_diff_falls_back_to_whole_scan(self):
        hits = ma.diff_patterns_hit("+AKIAIOSFODNN7EXAMPLE\n", [r"AKIA[0-9A-Z]{16}"])
        self.assertEqual(len(hits), 1)


class TestTruncatedFileList(unittest.TestCase):
    """红队最重的一条：gh --json files 封顶 100 条且按字典序，
    超出部分静默丢失 → 路径类硬停整体失效，而工具自己察觉不到。"""

    def test_detects_truncation_via_changed_files(self):
        self.assertTrue(ma.files_truncated(pr(files=["a.md"] , changed_files=134)))
        self.assertFalse(ma.files_truncated(pr(files=["a.md"], changed_files=1)))

    def test_missing_changed_files_is_not_treated_as_truncation(self):
        """离线夹具没有 changedFiles，不能据此把所有夹具判成硬停。"""
        self.assertFalse(ma.files_truncated(pr(files=["a.md"])))

    def test_unrepairable_truncation_is_hard_stop(self):
        d = pr(files=["bench/cases/pad.md"], changed_files=134)
        d["_files_truncated"] = True
        v = ma.classify(d, POLICY)
        self.assertEqual(v["decision"], "HARD_STOP")
        self.assertIn("unjudgeable-truncated-files", [r["id"] for r in v["hard_stop"]])

    def test_the_exact_evasion_scenario(self):
        """100 个字典序靠前的 pad 文件 + 一个排在后面的危险文件 = 危险文件被砍掉。"""
        danger = "tools/merge-audit/policy.json"
        full = sorted([f"bench/cases/pad-{i:03}.md" for i in range(100)] + [danger])
        self.assertEqual(ma.classify(pr(files=full), POLICY)["decision"], "HARD_STOP")
        truncated = full[:100]
        self.assertNotIn(danger, truncated)
        d = pr(files=truncated, changed_files=len(full))
        d["_files_truncated"] = True
        self.assertEqual(ma.classify(d, POLICY)["decision"], "HARD_STOP",
                         "截断必须硬停——否则这正是绕过路径")


class TestDeepScanFailsClosed(unittest.TestCase):
    """fetch_diff 失败原本返回 ""，classify 会把 deep 标成 True：
    「扫描失败」被记成「扫过了且干净」，失败方向是放行。"""

    def test_none_means_not_scanned(self):
        self.assertFalse(ma.classify(pr(files=["a.md"]), POLICY, None)["deep"])

    def test_empty_string_means_scanned_and_clean(self):
        self.assertTrue(ma.classify(pr(files=["a.md"]), POLICY, "")["deep"])


class TestHealthCompleteness(unittest.TestCase):
    """MERGE-POLICY §3 健康度写了 draft 与 mergeable，代码原来零实现。"""

    def test_draft_is_red_before_merge(self):
        d = pr(files=["a.md"], draft=True); d.pop("mergedAt")
        self.assertTrue(ma.classify(d, POLICY)["health"]["red"])

    def test_conflicting_is_red_before_merge(self):
        d = pr(files=["a.md"], mergeable="CONFLICTING"); d.pop("mergedAt")
        self.assertTrue(ma.classify(d, POLICY)["health"]["red"])

    def test_unknown_mergeable_before_merge_means_wait(self):
        d = pr(files=["a.md"], mergeable="UNKNOWN"); d.pop("mergedAt")
        self.assertTrue(ma.classify(d, POLICY)["health"]["red"])

    def test_merged_pr_is_not_judged_by_premerge_conditions(self):
        """已合并的 PR，GitHub 把 mergeable 报成 UNKNOWN——事后拿它判会全场误报。"""
        d = pr(files=["a.md"], mergeable="UNKNOWN", draft=True,
               merged_at="2026-08-20T00:00:00Z")
        self.assertFalse(ma.classify(d, POLICY)["health"]["red"])

    def test_error_conclusion_is_red(self):
        self.assertTrue(ma.classify(pr(files=["a.md"], checks=[("x", "ERROR")]), POLICY)["health"]["red"])

    def test_clean_pr_is_green(self):
        self.assertFalse(ma.classify(pr(files=["a.md"], checks=[("x", "SUCCESS")]), POLICY)["health"]["red"])


class TestGlobCaseAndEnv(unittest.TestCase):
    def test_uppercase_env_is_caught(self):
        for path in [".ENV", "config/PROD.env", "app/Secrets.yaml", "keys/ID_RSA"]:
            self.assertEqual(ma.classify(pr(files=[path]), POLICY)["decision"], "HARD_STOP",
                             f"{path} 应命中凭据类")

    def test_dot_env_suffix_form(self):
        self.assertEqual(ma.classify(pr(files=["deploy/prod.env"]), POLICY)["decision"], "HARD_STOP")

    def test_ordinary_doc_still_auto(self):
        self.assertEqual(ma.classify(pr(files=["playbook/environment.md"]), POLICY)["decision"], "AUTO")


class TestDiffPathWithSpaces(unittest.TestCase):
    def test_authoritative_path_comes_from_plus_line(self):
        diff = ("diff --git a/my dir/a b.py b/my dir/a b.py\n"
                "--- a/my dir/a b.py\n+++ b/my dir/a b.py\n"
                "+KEY='AKIAIOSFODNN7EXAMPLE'\n")
        per_file = ma.split_diff_by_file(diff)
        self.assertIn("my dir/a b.py", per_file)
        hits = ma.diff_patterns_hit(diff, [r"AKIA[0-9A-Z]{16}"])
        self.assertTrue(hits[0].startswith("my dir/a b.py "))


class TestTimestampComparison(unittest.TestCase):
    """字符串比大小会把同一时刻的不同写法判成差 8 小时。"""

    def test_equivalent_instants_compare_equal(self):
        self.assertTrue(ma.ts_at_or_after("2026-08-20T11:20:00+08:00", "2026-08-20T03:20:00Z"))
        self.assertTrue(ma.ts_at_or_after("2026-08-20T03:20:00Z", "2026-08-20T11:20:00+08:00"))

    def test_ordering_still_works(self):
        self.assertFalse(ma.ts_at_or_after("2026-08-19T00:00:00Z", "2026-08-20T00:00:00Z"))
        self.assertTrue(ma.ts_at_or_after("2026-08-21T00:00:00Z", "2026-08-20T00:00:00Z"))

    def test_unparseable_defaults_to_in_scope(self):
        self.assertTrue(ma.ts_at_or_after("garbage", "2026-08-20T00:00:00Z"))


class TestClaimRevision(unittest.TestCase):
    """被点名之后追加一条「改口」记录就能洗掉 MISMATCH——追加合法、链完整、零痕迹。"""

    def test_first_claim_is_the_one_compared(self):
        v = ma.classify(pr(1, files=["AGENTS.md"]), POLICY)   # 真相 NOTABLE
        f = ma.collect_findings([v], {1: [claim(1, "AUTO"), claim(1, "NOTABLE")]}, None)
        levels = [x["level"] for x in f]
        self.assertIn("MISMATCH", levels, "拿第一条比，改口不该洗掉 MISMATCH")

    def test_revision_itself_is_reported(self):
        v = ma.classify(pr(1, files=["AGENTS.md"]), POLICY)
        f = ma.collect_findings([v], {1: [claim(1, "AUTO"), claim(1, "NOTABLE")]}, None)
        self.assertIn("CLAIM_REVISED", [x["level"] for x in f])

    def test_single_consistent_claim_is_silent(self):
        v = ma.classify(pr(1, files=["snapshot/x.md"]), POLICY)
        self.assertEqual(ma.collect_findings([v], {1: [claim(1, "AUTO")]}, None), [])


class TestReportDiscloseWhatItDidNotSee(unittest.TestCase):
    """报告最危险的失败不是漏判，是把「我没查」说成「没发现」。"""

    def setUp(self):
        self.dir = tempfile.TemporaryDirectory()
        self.d = Path(self.dir.name)
        self.prs = self.d / "prs.json"
        self.ledger = self.d / "l.jsonl"
        pol = dict(POLICY)
        pol.pop("ledger_starts_at", None)
        self.policy = self.d / "policy.json"
        self.policy.write_text(json.dumps(pol, ensure_ascii=False), encoding="utf-8")

    def tearDown(self):
        self.dir.cleanup()

    def _run(self, argv):
        import io, contextlib
        buf = io.StringIO()
        with contextlib.redirect_stdout(buf):
            rc = ma.main(["--ledger", str(self.ledger), "--policy", str(self.policy)] + argv)
        return rc, buf.getvalue()

    def test_json_report_declares_deep_and_window(self):
        self.prs.write_text(json.dumps([pr(1, files=["a.md"])]), encoding="utf-8")
        _, out = self._run(["report", "--from-file", str(self.prs), "--json"])
        d = json.loads(out)
        self.assertFalse(d["deep"])
        self.assertIsNotNone(d["window"])

    def test_text_report_says_it_did_not_scan_diffs(self):
        self.prs.write_text(json.dumps([pr(1, files=["a.md"])]), encoding="utf-8")
        _, out = self._run(["report", "--from-file", str(self.prs)])
        self.assertIn("没有**扫 diff", out.replace("*", "*"))

    def test_zero_findings_never_fails_even_with_fail_on_ok(self):
        self.prs.write_text(json.dumps([pr(1, files=["a.md"])]), encoding="utf-8")
        ma.main(["--ledger", str(self.ledger), "--policy", str(self.policy),
                 "record", "--pr", "1", "--action", "merged", "--claimed", "AUTO"])
        rc, _ = self._run(["report", "--from-file", str(self.prs), "--json", "--fail-on", "ok"])
        self.assertEqual(rc, 0)




class TestGateEntitiesNotJustPointers(unittest.TestCase):
    """红队实测：清单里放的是「配置文件」，而闸的真正行为由它**指向的实体**决定。
    hooks/hooks.json 在清单里，它指向的 kit/guard/agent-on-git-guard 却不在——
    往后者开头加一行 `exit 0` 就能让全仓跨仓 git 闸失效，判定还是 AUTO。"""

    def _decide(self, path):
        return ma.classify(pr(files=[path]), POLICY)["decision"]

    def test_the_guard_script_itself(self):
        self.assertEqual(self._decide("kit/guard/agent-on-git-guard"), "HARD_STOP")

    def test_guard_judgment_sources(self):
        for path in ["cli/src/paths.rs", "cli/src/worktree.rs",
                     "cli/src/worktree_hooks.rs", "cli/src/main.rs"]:
            self.assertEqual(self._decide(path), "HARD_STOP", path)

    def test_a_brand_new_file_in_cli_src_is_covered(self):
        """整目录括住的意义：下一个新文件不用等谁想起来补清单。"""
        self.assertEqual(self._decide("cli/src/some_future_gate.rs"), "HARD_STOP")

    def test_ci_script_and_its_baseline(self):
        for path in [".github/workflows/gate.yml",
                     ".github/scripts/check_docs.py",
                     ".github/scripts/doc-boundary-baseline.txt"]:
            self.assertEqual(self._decide(path), "HARD_STOP", path)

    def test_plugin_manifests_that_register_hooks(self):
        """子代理实测把 .codex-plugin/plugin.json 的 hooks 注册删掉 = Codex 侧闸整条失效。"""
        for path in [".codex-plugin/plugin.json", ".claude-plugin/plugin.json"]:
            self.assertEqual(self._decide(path), "HARD_STOP", path)

    def test_ordinary_docs_are_still_automatic(self):
        """翻面的目的没被这次收紧吃掉：本仓绝大多数 PR 仍然自动合。"""
        for path in ["playbook/sop.md", "bench/cases/9.md", "snapshot/x.md",
                     "intake/2026-07-16-IPONews.md", "ledger/merge-audit.jsonl"]:
            self.assertEqual(self._decide(path), "AUTO", path)


class TestPointerResolver(unittest.TestCase):
    """拍板指针必须指向 git 可核对象，不能是裸 URL 或空串。"""

    def setUp(self):
        self.dir = tempfile.TemporaryDirectory()
        self.root = Path(self.dir.name)
        import subprocess
        subprocess.run(["git", "init", "-q", str(self.root)], check=True)
        (self.root / "snapshot").mkdir()
        (self.root / "snapshot" / "x.md").write_text("决策", encoding="utf-8")
        self.ok = ma.make_pointer_resolver(self.root)

    def tearDown(self):
        self.dir.cleanup()

    def test_existing_repo_file_is_ok(self):
        self.assertTrue(self.ok("snapshot/x.md"))

    def test_path_with_anchor_is_ok(self):
        self.assertTrue(self.ok("snapshot/x.md#拍板"))

    def test_missing_file_is_not_ok(self):
        self.assertFalse(self.ok("snapshot/does-not-exist.md"))

    def test_bare_pr_url_is_not_ok(self):
        """URL 不是 git 对象，核验不了——逼着指向 git 里那份快照/commit。"""
        self.assertFalse(self.ok("https://github.com/x/y/pull/39"))

    def test_empty_is_not_ok(self):
        self.assertFalse(self.ok(""))
        self.assertFalse(self.ok(None))


class TestNoClassesAfterMainBlock(unittest.TestCase):
    """守住红队 #29 的坑：`if __name__` 之后再追加测试类，直接跑文件会静默跳过它们。
    这个坑本轮踩了三次（每次都是 cat >> 追加，落到 __main__ 之后）——见 bench 案例 44。
    给这个文件加测试类时用 Edit 插到本类之前，别 cat >>。"""

    def test_main_guard_is_the_last_code_in_the_file(self):
        import re
        src = Path(__file__).read_text(encoding="utf-8")
        m = list(re.finditer(r"(?m)^if __name__ == .__main__.:", src))
        self.assertEqual(len(m), 1, "应当恰好一个 __main__ 守卫块")
        after = src[m[0].start():]
        self.assertFalse(re.search(r"(?m)^class Test", after),
                         "有测试类排在 if __name__ 之后——直接跑文件会漏掉它们（红队 #29）")


if __name__ == "__main__":
    unittest.main(verbosity=2)
