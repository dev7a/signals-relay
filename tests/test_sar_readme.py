from __future__ import annotations

import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
SAR_README = REPO_ROOT / "docs" / "sar-readme.md"
TEMPLATE = REPO_ROOT / "template.yaml"


class SarReadmeTests(unittest.TestCase):
    def test_template_uses_sar_specific_readme(self) -> None:
        self.assertIn("ReadmeUrl: docs/sar-readme.md", TEMPLATE.read_text())

    def test_sar_readme_avoids_github_only_markup(self) -> None:
        text = SAR_README.read_text()

        self.assertNotIn("[!", text)
        self.assertNotRegex(text, r"</?[a-zA-Z][^>]*>")
        self.assertNotRegex(text, r"\]\(\.?\.?/")

    def test_sar_readme_uses_absolute_links(self) -> None:
        text = SAR_README.read_text()
        links = re.findall(r"\]\(([^)]+)\)", text)

        self.assertTrue(links)
        for link in links:
            self.assertTrue(
                link.startswith("https://"),
                f"expected SAR README link to be absolute: {link}",
            )


if __name__ == "__main__":
    unittest.main()
