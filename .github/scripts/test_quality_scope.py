"""Scope contract for Agent-On v2."""
import unittest
from quality_scope import clippy_command, plan, test_command


class QualityScopeTest(unittest.TestCase):
    def test_docs_skills_and_manifests_do_not_compile_rust(self):
        for paths in (
            ["README.md", "skills/prove-it/SKILL.md", "rules/core-guardrails.md"],
            ["plugin.json", ".claude-plugin/plugin.json", ".codex-plugin/plugin.json"],
            [".github/scripts/check_docs.py", ".github/scripts/doc-boundary-baseline.txt"],
            ["cli/tests/fixture.txt"],
        ):
            with self.subTest(paths=paths):
                result = plan(paths)
                self.assertFalse(result["rust"])
                self.assertIsNone(test_command(result))
                self.assertIsNone(clippy_command(result))

    def test_leaf_modules_map_to_matching_integration_suite(self):
        wt = plan(["cli/src/wt.rs"])
        self.assertTrue(wt["rust"])
        self.assertFalse(wt["full"])
        self.assertEqual(wt["tests"], ["wt_radar"])
        self.assertIn("wt_radar", test_command(wt))

        mc = plan(["cli/src/merge_check.rs"])
        self.assertEqual(mc["tests"], ["merge_rehearsal"])

        inst = plan(["cli/src/install.rs", "cli/src/doctor.rs"])
        self.assertEqual(inst["tests"], ["install_and_doctor"])

    def test_shared_or_unknown_cli_paths_escalate_to_full(self):
        for path in ("cli/src/main.rs", "cli/src/git_util.rs", "cli/src/unknown.rs", "cli/Cargo.toml"):
            with self.subTest(path=path):
                result = plan([path])
                self.assertTrue(result["full"])
                self.assertEqual(result["tests"], [])

    def test_nightly_and_runner_changes_lint_all_targets(self):
        nightly = plan([], full=True)
        self.assertTrue(nightly["full"])
        self.assertTrue(nightly["lint_all_targets"])
        self.assertIn("--all-targets", clippy_command(nightly))

        runner = plan([".github/workflows/gate.yml"])
        self.assertTrue(runner["full"])
        self.assertTrue(runner["lint_all_targets"])


if __name__ == "__main__":
    unittest.main()
