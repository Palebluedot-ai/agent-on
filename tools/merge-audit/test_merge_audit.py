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
       labels=(), review="", checks=(), merged_at="2026-08-20T00:00:00Z", oid="abc123"):
    return {
        "number": number, "title": title, "body": body,
        "author": {"login": author},
        "files": [{"path": p} for p in files],
        "labels": [{"name": n} for n in labels],
        "reviewDecision": review,
        "statusCheckRollup": [{"name": n, "conclusion": c} for n, c in checks],
        "mergedAt": merged_at, "mergeCommit": {"oid": oid},
    }


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

    def test_issue_template_is_not_a_workflow(self):
        v = ma.classify(pr(files=[".github/ISSUE_TEMPLATE/intake-card.md"]), POLICY)
        self.assertEqual(v["decision"], "AUTO")

    def test_breaking_marker_is_notable_not_hard_stop(self):
        v = ma.classify(pr(files=["playbook/sop.md"], body="正文\n\nBREAKING: 口令改名"), POLICY)
        self.assertEqual(v["decision"], "NOTABLE")
        self.assertIn("breaking-or-migration", [r["id"] for r in v["notable"]])

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
    def test_hard_stop_that_got_merged_is_a_violation(self):
        v = ma.classify(pr(1, files=["hooks/hooks.json"]), POLICY)
        f = ma.collect_findings([v], {1: {"claimed": "HARD_STOP", "pr": 1}}, None)
        self.assertIn("VIOLATION", [x["level"] for x in f])

    def test_merged_without_a_record_is_unrecorded(self):
        v = ma.classify(pr(2, files=["a.md"]), POLICY)
        f = ma.collect_findings([v], {}, None)
        self.assertEqual([x["level"] for x in f], ["UNRECORDED"])

    def test_claim_disagreeing_with_verdict_is_mismatch(self):
        v = ma.classify(pr(3, files=["AGENTS.md"]), POLICY)  # 真相 NOTABLE
        f = ma.collect_findings([v], {3: {"claimed": "AUTO", "pr": 3, "note": ""}}, None)
        levels = [x["level"] for x in f]
        self.assertIn("MISMATCH", levels)

    def test_clean_auto_merge_with_a_record_is_silent(self):
        v = ma.classify(pr(4, files=["snapshot/x.md"]), POLICY)
        f = ma.collect_findings([v], {4: {"claimed": "AUTO", "pr": 4}}, None)
        self.assertEqual(f, [])

    def test_red_merge_is_flagged(self):
        v = ma.classify(pr(5, files=["a.md"], checks=[("CI", "FAILURE")]), POLICY)
        f = ma.collect_findings([v], {5: {"claimed": "AUTO", "pr": 5}}, None)
        self.assertIn("MERGED_RED", [x["level"] for x in f])

    def test_broken_chain_surfaces_even_with_no_pr_problems(self):
        v = ma.classify(pr(6, files=["a.md"]), POLICY)
        f = ma.collect_findings([v], {6: {"claimed": "AUTO", "pr": 6}}, 2)
        self.assertEqual([x["level"] for x in f], ["LEDGER_BROKEN"])

    def test_findings_sorted_worst_first(self):
        v1 = ma.classify(pr(7, files=["AGENTS.md"]), POLICY)
        v2 = ma.classify(pr(8, files=["hooks/hooks.json"]), POLICY)
        f = ma.collect_findings([v1, v2], {7: {"claimed": "NOTABLE", "pr": 7},
                                           8: {"claimed": "HARD_STOP", "pr": 8}}, None)
        self.assertEqual(f[0]["level"], "VIOLATION")


class TestCliOffline(unittest.TestCase):
    """端到端跑 main()，用 --from-file 走离线路径。"""

    def setUp(self):
        self.dir = tempfile.TemporaryDirectory()
        self.d = Path(self.dir.name)
        self.prs = self.d / "prs.json"
        self.ledger = self.d / "l.jsonl"

    def tearDown(self):
        self.dir.cleanup()

    def _write(self, prs):
        self.prs.write_text(json.dumps(prs, ensure_ascii=False), encoding="utf-8")

    def test_precheck_exit_codes(self):
        self._write([pr(1, files=["a.md"]),
                     pr(2, files=["AGENTS.md"]),
                     pr(3, files=["hooks/h.json"])])
        base = ["--ledger", str(self.ledger)]
        self.assertEqual(ma.main(base + ["precheck", "--pr", "1", "--from-file", str(self.prs), "--json"]), ma.EXIT_AUTO)
        self.assertEqual(ma.main(base + ["precheck", "--pr", "2", "--from-file", str(self.prs), "--json"]), ma.EXIT_NOTABLE)
        self.assertEqual(ma.main(base + ["precheck", "--pr", "3", "--from-file", str(self.prs), "--json"]), ma.EXIT_HARD_STOP)

    def test_report_fails_on_unrecorded_then_passes_after_record(self):
        self._write([pr(1, files=["a.md"])])
        base = ["--ledger", str(self.ledger)]
        rc = ma.main(base + ["report", "--from-file", str(self.prs), "--json"])
        self.assertEqual(rc, 1, "没记账就该红")
        ma.main(base + ["record", "--pr", "1", "--action", "merged", "--claimed", "AUTO"])
        rc = ma.main(base + ["report", "--from-file", str(self.prs), "--json"])
        self.assertEqual(rc, 0, "记完账就该绿")

    def test_report_notable_alone_does_not_fail_by_default(self):
        self._write([pr(1, files=["AGENTS.md"])])
        base = ["--ledger", str(self.ledger)]
        ma.main(base + ["record", "--pr", "1", "--action", "merged", "--claimed", "NOTABLE"])
        self.assertEqual(ma.main(base + ["report", "--from-file", str(self.prs), "--json"]), 0)
        self.assertEqual(
            ma.main(base + ["report", "--from-file", str(self.prs), "--json", "--fail-on", "notable"]), 1)

    def test_scan_write_then_report_reads_same_ledger(self):
        self._write([pr(1, files=["a.md"])])
        base = ["--ledger", str(self.ledger)]
        ma.main(base + ["scan", "--from-file", str(self.prs), "--write", "--json"])
        recs, raw = ma.read_ledger(self.ledger)
        self.assertEqual([r["kind"] for r in recs], ["verdict"])
        self.assertIsNone(ma.verify_chain(raw))


if __name__ == "__main__":
    unittest.main(verbosity=2)


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

    def test_structured_breaking_footer_is_notable(self):
        v = ma.classify(pr(files=["snapshot/x.md"], body="正文\n\nBREAKING: 口令改名了\n"), POLICY)
        self.assertEqual(v["decision"], "NOTABLE")

    def test_structured_migration_footer_is_notable(self):
        v = ma.classify(pr(files=["snapshot/x.md"], body="迁移注记：先跑 setup 再升级"), POLICY)
        self.assertEqual(v["decision"], "NOTABLE")

    def test_breaking_label_is_notable(self):
        v = ma.classify(pr(files=["snapshot/x.md"], labels=["breaking"]), POLICY)
        self.assertEqual(v["decision"], "NOTABLE")


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

    def test_old_hard_stop_merge_is_still_a_violation(self):
        v = ma.classify(pr(3, files=["hooks/h.json"], merged_at="2026-08-01T00:00:00Z"), POLICY)
        f = ma.collect_findings([v], {}, None, self.START)
        self.assertIn("VIOLATION", [x["level"] for x in f])

    def test_old_red_merge_is_still_flagged(self):
        v = ma.classify(pr(4, files=["a.md"], checks=[("CI", "FAILURE")],
                           merged_at="2026-08-01T00:00:00Z"), POLICY)
        f = ma.collect_findings([v], {}, None, self.START)
        self.assertIn("MERGED_RED", [x["level"] for x in f])
