"""Scope contract: docs stay fast; shared or unknown execution paths get full coverage."""
import unittest
from quality_scope import clippy_command, plan, test_command


class QualityScope(unittest.TestCase):
    def test_document_change_does_not_compile_rust(self):
        for paths in (
            ["README.md", "playbook/delivery.md"],
            [".github/pull_request_template.md"],
            [".github/PULL_REQUEST_TEMPLATE/intake.md"],
            [".github/scripts/check_docs.py", ".github/scripts/doc-boundary-baseline.txt"],
            ["cli/tests/dashboard_browser.cjs"],
        ):
            with self.subTest(paths=paths):
                result = plan(paths)
                self.assertFalse(result["rust"])
                self.assertFalse(result["audit"])
                self.assertFalse(result["intake"])
                self.assertIsNone(test_command(result))
                self.assertIsNone(clippy_command(result))

    def test_intake_keeps_its_lint_without_all_tests(self):
        result = plan(["intake/case.md"])
        self.assertTrue(result["intake"])
        self.assertFalse(result["rust"])

    def test_linter_change_lints_the_corpus_and_its_consumer(self):
        result = plan(["cli/src/intake_lint.rs"])
        self.assertTrue(result["intake"])
        self.assertTrue(result["rust"])
        self.assertFalse(result["full"])
        self.assertEqual(result["tests"], ["setup_install"])

    def test_brief_covers_public_consumers(self):
        result = plan(["cli/src/project_status.rs"])
        self.assertFalse(result["full"])
        self.assertEqual(result["tests"], ["project_status", "session_continuity", "system_overview"])
        self.assertFalse(result["intake"])
        self.assertEqual(
            clippy_command(result)[:6],
            ["cargo", "clippy", "--manifest-path", "cli/Cargo.toml", "--locked", "--bin"],
        )
        self.assertNotIn("--all-targets", clippy_command(result))

    def test_leaf_and_audit_wrapper_stay_narrow(self):
        master = plan(["cli/src/master.rs"])
        self.assertFalse(master["full"])
        self.assertEqual(master["tests"], ["master"])
        self.assertFalse(master["intake"])
        audit = plan(["tools/merge-audit/merge_audit.py"])
        self.assertFalse(audit["full"])
        self.assertTrue(audit["audit"])
        self.assertTrue(audit["rust"])
        self.assertEqual(audit["tests"], ["master"])
        self.assertEqual(plan(["cli/src/tag_release.rs"])["tests"], [])
        self.assertFalse(plan(["cli/src/tag_release.rs"])["full"])

    def test_shared_or_new_execution_paths_get_all_tests(self):
        for path in ["cli/src/worktree.rs", "cli/src/new.rs", "cli/Cargo.lock",
                     "hooks/hooks.json", "kit/guard/shim",
                     "cli/src/guard.rs", "cli/src/coordination.rs", "cli/src/oncall.rs"]:
            with self.subTest(path=path):
                result = plan([path])
                self.assertTrue(result["full"])
                self.assertEqual(result["tests"], [])
                self.assertFalse(result["intake"])
                self.assertNotIn("--all-targets", clippy_command(result))

    def test_scope_runner_and_nightly_do_not_narrow_coverage(self):
        nightly = plan([], full=True)
        self.assertTrue(nightly["rust"])
        self.assertTrue(nightly["full"])
        self.assertTrue(nightly["intake"])
        self.assertTrue(nightly["audit"])
        self.assertTrue(nightly["lint_all_targets"])
        self.assertIn("--all-targets", clippy_command(nightly))
        self.assertEqual(
            test_command(nightly),
            ["cargo", "test", "--manifest-path", "cli/Cargo.toml", "--locked"],
        )
        runner = plan([".github/workflows/gate.yml", ".github/scripts/quality_scope.py"])
        self.assertTrue(runner["full"])
        self.assertTrue(runner["audit"])
        self.assertTrue(runner["lint_all_targets"])
        self.assertIn("--all-targets", clippy_command(runner))
        mixed = plan(["cli/src/project_status.rs", "cli/src/main.rs"])
        self.assertTrue(mixed["full"])
        self.assertFalse(mixed["lint_all_targets"])

    def test_edited_integration_test_is_still_linted_when_suite_widens(self):
        result = plan(["cli/src/worktree.rs", "cli/tests/worktree_edit.rs"])
        self.assertTrue(result["full"])
        self.assertEqual(result["clippy_tests"], ["worktree_edit"])
        cmd = clippy_command(result)
        self.assertIn("--test", cmd)
        self.assertIn("worktree_edit", cmd)
        self.assertNotIn("--all-targets", cmd)

    def test_missing_scoped_test_falls_back_to_full(self):
        data = plan(["cli/src/master.rs"])
        data["tests"] = ["no_such_suite"]
        data["clippy_tests"] = ["no_such_suite"]
        self.assertNotIn("--test", test_command(data))
        self.assertIn("--all-targets", clippy_command(data))


if __name__ == "__main__":
    unittest.main()
