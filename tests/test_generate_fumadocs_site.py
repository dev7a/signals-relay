from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


def _write(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")


def _run_generator(repo_root: Path, content_root: Path, public_root: Path) -> None:
    script = Path(__file__).resolve().parents[1] / "scripts" / "generate_fumadocs_site.py"
    subprocess.run(
        [
            sys.executable,
            str(script),
            "--repo-root",
            str(repo_root),
            "--content-root",
            str(content_root),
            "--public-root",
            str(public_root),
        ],
        check=True,
    )


class GenerateFumadocsSiteTests(unittest.TestCase):
    def test_generator_normalizes_docs_tree_links_and_assets(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            repo_root = Path(tmp) / "repo"
            content_root = repo_root / "site" / "content" / "docs"
            public_root = repo_root / "site" / "public"

            _write(
                repo_root / "docs" / "README.md",
                """# Documentation

Choose the document that matches your task.

- [Install](./install.md)
- [Architecture](./current-architecture.md)
- [Release](./release.md)
- [Design](./cloudwatch-kinesis-lambda-relay.md#questions)
- [Delayed work](./long-poller-sqs-delayed-task.md)
- [Root README](../README.md)
- [External](https://example.com)
- [Email](mailto:hello@example.com)
""",
            )
            _write(
                repo_root / "docs" / "install.md",
                """# Install And Deployment

Use this guide when you want to deploy.

> [!NOTE]
> Keep this source compatible with GitHub alerts.

See [release.md](./release.md).

![Launch badge](./assets/cloudformation-launch-badge.svg)

```md
> [!NOTE]
> Do not rewrite alerts inside code fences.
[Do not rewrite](./release.md)
```
""",
            )
            _write(
                repo_root / "docs" / "current-architecture.md",
                """# Current Architecture

Understand the runtime path.

See [alternate](./cloudwatch-kinesis-lambda-relay.md) and [delayed](./long-poller-sqs-delayed-task.md).

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../svgs/architecture-flow-dark.svg">
  <img src="../svgs/architecture-flow-light.svg" alt="Architecture">
</picture>
""",
            )
            _write(
                repo_root / "docs" / "cloudwatch-kinesis-lambda-relay.md",
                """# CloudWatch Logs To Kinesis To Lambda Relay

Alternative design.

## Questions

See [architecture](./current-architecture.md).
""",
            )
            _write(
                repo_root / "docs" / "long-poller-sqs-delayed-task.md",
                """# Long Poller With SQS For Delayed Work

Historical option.
""",
            )
            _write(
                repo_root / "docs" / "release.md",
                """# Release Guide

Maintainer workflow.

See [install](./install.md) and [local](#manual-sar-publish).
""",
            )
            _write(repo_root / "docs" / "assets" / "cloudformation-launch-badge.svg", "<svg />\n")
            _write(repo_root / "svgs" / "architecture-flow-dark.svg", "<svg />\n")
            _write(repo_root / "svgs" / "architecture-flow-light.svg", "<svg />\n")

            _run_generator(repo_root, content_root, public_root)

            root_page = (content_root / "index.mdx").read_text(encoding="utf-8")
            install_page = (content_root / "install.mdx").read_text(encoding="utf-8")
            architecture_page = (content_root / "current-architecture.mdx").read_text(
                encoding="utf-8"
            )
            release_page = (content_root / "release.mdx").read_text(encoding="utf-8")

            self.assertFalse((content_root / "cloudwatch-kinesis-lambda-relay.mdx").exists())
            self.assertFalse((content_root / "long-poller-sqs-delayed-task.mdx").exists())
            self.assertIn("./install/", root_page)
            self.assertIn("./current-architecture/", root_page)
            self.assertIn("./release/", root_page)
            self.assertIn(
                "https://github.com/dev7a/signals-relay/blob/main/docs/cloudwatch-kinesis-lambda-relay.md#questions",
                root_page,
            )
            self.assertIn(
                "https://github.com/dev7a/signals-relay/blob/main/docs/long-poller-sqs-delayed-task.md",
                root_page,
            )
            self.assertIn("../", root_page)
            self.assertIn("https://example.com", root_page)
            self.assertIn("mailto:hello@example.com", root_page)
            self.assertNotIn("./cloudwatch-kinesis-lambda-relay/", root_page)
            self.assertNotIn("./long-poller-sqs-delayed-task/", root_page)
            self.assertNotIn(".mdx", root_page)
            self.assertIn("./release/", install_page)
            self.assertIn('<Callout type="info" title="Note">', install_page)
            self.assertIn("Keep this source compatible with GitHub alerts.", install_page)
            self.assertIn("> [!NOTE]", install_page)
            self.assertIn("[Do not rewrite](./release.md)", install_page)
            self.assertIn("../docs-assets/docs/assets/cloudformation-launch-badge.svg", install_page)
            self.assertIn('title: "Architecture"', architecture_page)
            self.assertIn("../../docs-assets/svgs/architecture-flow-dark.svg", architecture_page)
            self.assertIn("../../docs-assets/svgs/architecture-flow-light.svg", architecture_page)
            self.assertIn(
                "https://github.com/dev7a/signals-relay/blob/main/docs/cloudwatch-kinesis-lambda-relay.md",
                architecture_page,
            )
            self.assertIn(
                "https://github.com/dev7a/signals-relay/blob/main/docs/long-poller-sqs-delayed-task.md",
                architecture_page,
            )
            self.assertIn("./install/", release_page)
            self.assertIn("#manual-sar-publish", release_page)

            meta = json.loads((content_root / "meta.json").read_text(encoding="utf-8"))
            self.assertEqual(
                meta["pages"],
                [
                    "index",
                    "install",
                    "current-architecture",
                    "release",
                ],
            )
            self.assertTrue(
                (
                    public_root
                    / "docs-assets"
                    / "docs"
                    / "assets"
                    / "cloudformation-launch-badge.svg"
                ).exists()
            )
            self.assertTrue((public_root / "docs-assets" / "svgs" / "architecture-flow-dark.svg").exists())

    def test_generator_is_deterministic(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            repo_root = Path(tmp) / "repo"
            content_root = repo_root / "site" / "content" / "docs"
            public_root = repo_root / "site" / "public"

            _write(repo_root / "docs" / "README.md", "# Documentation\n\nFront door.\n")
            _write(repo_root / "docs" / "install.md", "# Install\n\nInstall.\n")
            _write(repo_root / "docs" / "current-architecture.md", "# Architecture\n\nArchitecture.\n")
            _write(
                repo_root / "docs" / "cloudwatch-kinesis-lambda-relay.md",
                "# CloudWatch\n\nDesign.\n",
            )
            _write(
                repo_root / "docs" / "long-poller-sqs-delayed-task.md",
                "# Long poller\n\nDesign.\n",
            )
            _write(repo_root / "docs" / "release.md", "# Release\n\nRelease.\n")

            _run_generator(repo_root, content_root, public_root)
            snapshot = {
                path.relative_to(content_root).as_posix(): path.read_text(encoding="utf-8")
                for path in sorted(content_root.rglob("*"))
                if path.is_file()
            }

            _run_generator(repo_root, content_root, public_root)
            self.assertEqual(
                snapshot,
                {
                    path.relative_to(content_root).as_posix(): path.read_text(encoding="utf-8")
                    for path in sorted(content_root.rglob("*"))
                    if path.is_file()
                },
            )


if __name__ == "__main__":
    unittest.main()
