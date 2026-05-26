from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


SCRIPT = Path(__file__).resolve().parents[1] / "scripts" / "render_github_release_notes.py"
APPLICATION_ID = (
    "arn:aws:serverlessrepo:us-east-1:902605180476:applications/signals-relay"
)
APPLICATION_URL = (
    "https://serverlessrepo.aws.amazon.com/applications/"
    "us-east-1/902605180476/signals-relay"
)


class RenderGithubReleaseNotesTests(unittest.TestCase):
    def test_release_notes_include_public_sar_url(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            tmp_path = Path(tmp)
            manifest = tmp_path / "manifest.json"
            output = tmp_path / "notes.md"

            manifest.write_text(
                json.dumps(
                    {
                        "tag": "v0.1.0",
                        "version": "0.1.0",
                        "commit": "359ae398d13dec033febe5aadda1d073b3cd0662",
                        "applicationId": APPLICATION_ID,
                        "semanticVersion": "0.1.0",
                        "launchTemplates": {
                            "noVpc": {
                                "templateUrl": "https://example.com/no-vpc.yaml",
                                "quickCreateUrl": "https://example.com/quick-no-vpc",
                                "s3Uri": "s3://example/releases/0.1.0/launch-no-vpc.yaml",
                            },
                            "vpc": {
                                "templateUrl": "https://example.com/vpc.yaml",
                                "quickCreateUrl": "https://example.com/quick-vpc",
                                "s3Uri": "s3://example/releases/0.1.0/launch-vpc.yaml",
                            },
                        },
                    }
                )
            )

            result = subprocess.run(
                [
                    sys.executable,
                    str(SCRIPT),
                    "--manifest",
                    str(manifest),
                    "--output",
                    str(output),
                ],
                check=False,
                capture_output=True,
                text=True,
            )

            self.assertEqual(result.returncode, 0, result.stderr)
            notes = output.read_text()
            self.assertIn("## SAR Application", notes)
            self.assertIn(f"- SAR application: {APPLICATION_URL}", notes)
            self.assertIn(f"- Application ID: `{APPLICATION_ID}`", notes)


if __name__ == "__main__":
    unittest.main()
