import contextlib
import io
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import hygiene


def run_in_repo(files):
    with tempfile.TemporaryDirectory() as tmp:
        subprocess.run(["git", "init", "-q", tmp], check=True)
        for name, content in files.items():
            target = Path(tmp, name)
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(content)
        subprocess.run(["git", "-C", tmp, "add", "-A", "-f"], check=True)
        previous = os.getcwd()
        os.chdir(tmp)
        out = io.StringIO()
        try:
            with contextlib.redirect_stdout(out):
                code = hygiene.main()
        finally:
            os.chdir(previous)
        return code, out.getvalue()


PNG = b"\x89PNG\r\n\x1a\n"


class ImageAllowance(unittest.TestCase):
    def test_png_under_synthetic_passes(self):
        code, out = run_in_repo({"core/testdata/synthetic/frames/frame_00.png": PNG})
        self.assertEqual(code, 0, out)

    def test_png_elsewhere_under_testdata_fails(self):
        for path in (
            "core/testdata/golden/frame.png",
            "core/testdata/frame.png",
            "core/testdata/synthetic.png",
            "core/testdata/synthetic_other/frame.png",
        ):
            with self.subTest(path=path):
                code, out = run_in_repo({path: PNG})
                self.assertEqual(code, 1, out)
                self.assertIn(path, out)

    def test_png_outside_core_fails(self):
        for path in (
            "frame.png",
            "docs/frame.png",
            "host/core/testdata/synthetic/frame.png",
            "Core/testdata/synthetic/frame.png",
        ):
            with self.subTest(path=path):
                code, out = run_in_repo({path: PNG})
                self.assertEqual(code, 1, out)

    def test_other_blocked_kinds_under_synthetic_still_fail(self):
        for name in ("clip.mp4", "battle.fga", "run.log"):
            with self.subTest(name=name):
                code, out = run_in_repo({"core/testdata/synthetic/" + name: b"x"})
                self.assertEqual(code, 1, out)

    def test_uppercase_suffix_is_judged_the_same(self):
        code, _ = run_in_repo({"core/testdata/synthetic/frames/FRAME.PNG": PNG})
        self.assertEqual(code, 0)
        code, _ = run_in_repo({"core/testdata/other/FRAME.PNG": PNG})
        self.assertEqual(code, 1)

    def test_allowance_is_exactly_the_synthetic_prefix(self):
        self.assertEqual(hygiene.ALLOWED_IMAGE_PREFIXES, ("core/testdata/synthetic/",))


class OtherRulesStillApply(unittest.TestCase):
    def test_private_address_in_text_file_under_synthetic_fails(self):
        code, out = run_in_repo({"core/testdata/synthetic/notes.txt": b"host 192.168.1.20\n"})
        self.assertEqual(code, 1, out)
        self.assertIn("private network address", out)


if __name__ == "__main__":
    unittest.main()
