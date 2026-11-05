"""全链冒烟测试：在临时 QTLEGAL_ROOT 下跑 spec → … → deliver，零依赖，python3 直接运行。"""

import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

PROJECT = Path(__file__).resolve().parent.parent


class PipelineTest(unittest.TestCase):
    def test_full_chain(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp) / "quanttide_legal"
            (root / "data").mkdir(parents=True)
            shutil.copytree(PROJECT / "data" / "fixtures", root / "data" / "fixtures")
            env = dict(os.environ, QTLEGAL_ROOT=str(root),
                       PYTHONPATH=str(PROJECT / "src"))
            for step in ("spec", "collect", "generate", "judge", "verify", "deliver"):
                with self.subTest(step=step):
                    result = subprocess.run(
                        [sys.executable, "-m", "qtlegal", step],
                        env=env, capture_output=True, text=True)
                    self.assertEqual(result.returncode, 0,
                                     f"{step} 失败：\n{result.stdout}\n{result.stderr}")
            deliver = root / "data" / "work" / "deliver"
            batches = list(deliver.iterdir()) if deliver.exists() else []
            self.assertEqual(len(batches), 1, "应有一个交付批次")
            batch = batches[0]
            for artifact in ("questions.jsonl", "keywords.jsonl",
                             "indicators.json", "report.md", "复算记录.txt"):
                self.assertTrue((batch / artifact).exists(), f"缺少交付物 {artifact}")
            indicators = json.loads((batch / "indicators.json").read_text())
            self.assertEqual(indicators["coverage_gap"], [], "覆盖差集应为空")

    def test_delivery_gate_blocks_on_quota(self):
        """量纲抬高后 C05 只有一道 verified 题，deliver 应拒绝打包。"""
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp) / "quanttide_legal"
            (root / "data").mkdir(parents=True)
            shutil.copytree(PROJECT / "data" / "fixtures", root / "data" / "fixtures")
            spec_path = root / "data" / "fixtures" / "regulation" / "spec.sample.json"
            spec = json.loads(spec_path.read_text())
            spec["quota"]["per_category_min_questions"] = 2
            spec_path.write_text(json.dumps(spec, ensure_ascii=False, indent=2))
            env = dict(os.environ, QTLEGAL_ROOT=str(root),
                       PYTHONPATH=str(PROJECT / "src"))
            for step in ("spec", "collect", "generate", "judge", "verify"):
                subprocess.run([sys.executable, "-m", "qtlegal", step],
                               env=env, capture_output=True, text=True)
            deliver = subprocess.run([sys.executable, "-m", "qtlegal", "deliver"],
                                     env=env, capture_output=True, text=True)
            self.assertNotEqual(deliver.returncode, 0, "量纲不达标时 deliver 应拒绝打包")
            self.assertIn("量纲", deliver.stdout)


if __name__ == "__main__":
    unittest.main()
