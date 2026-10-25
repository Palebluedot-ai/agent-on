"""Scope contract: docs stay fast; shared or unknown execution paths get full coverage."""
import unittest
from quality_scope import plan


class QualityScope(unittest.TestCase):
    def test_document_change_does_not_compile_rust(self):
        result = plan(["README.md", "playbook/delivery.md"])
        self.assertFalse(result["rust"])
        self.assertFalse(result["audit"])

    def test_intake_keeps_its_lint_without_all_tests(self):
        result = plan(["intake/case.md"])
        self.assertTrue(result["intake"])
        self.assertFalse(result["rust"])

    def test_brief_covers_public_consumers(self):
        result = plan(["cli/src/project_status.rs"])
        self.assertFalse(result["full"])
        self.assertEqual(result["tests"], ["project_status", "session_continuity", "system_overview"])

    def test_shared_or_new_execution_paths_get_all_tests(self):
        for path in ["cli/src/worktree.rs", "cli/src/new.rs", "cli/Cargo.lock",
                     "hooks/hooks.json", "kit/guard/shim", ".github/workflows/gate.yml",
                     "tools/merge-audit/merge_audit.py"]:
            with self.subTest(path=path):
                self.assertTrue(plan([path])["full"])

    def test_nightly_and_mixed_changes_do_not_narrow_coverage(self):
        self.assertTrue(plan([], full=True)["rust"])
        self.assertTrue(plan(["cli/src/project_status.rs", "cli/src/main.rs"])["full"])


if __name__ == "__main__":
    unittest.main()
