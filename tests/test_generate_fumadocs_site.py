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


def _write_nav(repo_root: Path, pages: list[dict[str, str]] | None = None) -> None:
    _write(
        repo_root / "docs" / "nav.json",
        json.dumps(
            {
                "title": "Signals Relay docs",
                "sections": [
                    {"id": "primary", "title": "Docs"},
                    {"id": "maintainers", "title": "Maintainers"},
                    {"id": "design-history", "title": "Design history"},
                ],
                "pages": pages
                or [
                    {
                        "source": "docs/README.md",
                        "target": "index.mdx",
                        "navLabel": "Docs",
                        "title": "Signals Relay docs",
                        "description": "Get started.",
                        "section": "primary",
                    },
                    {
                        "source": "docs/deploy.md",
                        "target": "deploy.mdx",
                        "navLabel": "Deploy",
                        "title": "Deploy",
                        "description": "Deploy the stack.",
                        "section": "primary",
                    },
                    {
                        "source": "docs/operate.md",
                        "target": "operate.mdx",
                        "navLabel": "Operate",
                        "title": "Operate",
                        "description": "Operate the stack.",
                        "section": "primary",
                    },
                    {
                        "source": "docs/architecture.md",
                        "target": "architecture.mdx",
                        "navLabel": "Architecture",
                        "title": "Architecture",
                        "description": "Understand the architecture.",
                        "section": "primary",
                    },
                    {
                        "source": "docs/reference.md",
                        "target": "reference.mdx",
                        "navLabel": "Reference",
                        "title": "Reference",
                        "description": "Look up contracts.",
                        "section": "primary",
                    },
                    {
                        "source": "docs/maintainers/release.md",
                        "target": "maintainers/release.mdx",
                        "navLabel": "Release guide",
                        "title": "Release guide",
                        "description": "Publish a release.",
                        "section": "maintainers",
                    },
                    {
                        "source": "docs/design-history/cloudwatch-kinesis-lambda-relay.md",
                        "target": "design-history/cloudwatch-kinesis-lambda-relay.mdx",
                        "navLabel": "CloudWatch to Kinesis relay",
                        "title": "CloudWatch Logs to Kinesis to Lambda relay",
                        "description": "Historical Kinesis relay design.",
                        "section": "design-history",
                    },
                    {
                        "source": "docs/design-history/long-poller-sqs-delayed-task.md",
                        "target": "design-history/long-poller-sqs-delayed-task.mdx",
                        "navLabel": "Long poller with SQS",
                        "title": "Long poller with SQS for delayed work",
                        "description": "Historical long-poller design.",
                        "section": "design-history",
                    },
                ],
            },
            indent=2,
        )
        + "\n",
    )


def _run_generator(repo_root: Path, content_root: Path, public_root: Path) -> subprocess.CompletedProcess[str]:
    script = Path(__file__).resolve().parents[1] / "scripts" / "generate_fumadocs_site.py"
    return subprocess.run(
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
        check=False,
        text=True,
        capture_output=True,
    )


class GenerateFumadocsSiteTests(unittest.TestCase):
    def test_generator_uses_nav_tree_links_metadata_and_assets(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            repo_root = Path(tmp) / "repo"
            content_root = repo_root / "site" / "content" / "docs"
            public_root = repo_root / "site" / "public"

            _write_nav(repo_root)
            _write(
                repo_root / "docs" / "README.md",
                """# Documentation

- [Deploy](./deploy.md)
- [Architecture](./architecture.md)
- [Reference](./reference.md)
- [Release](./maintainers/release.md)
- [Design](./design-history/cloudwatch-kinesis-lambda-relay.md#questions)
- [Delayed work](./design-history/long-poller-sqs-delayed-task.md)
- [Root README](../README.md)
- [External](https://example.com)
- [Email](mailto:hello@example.com)
""",
            )
            _write(
                repo_root / "docs" / "deploy.md",
                """# Deploy

Deploy the stack.

> [!NOTE]
> Keep this source compatible with GitHub alerts.

See [release](./maintainers/release.md).

![Launch badge](./assets/cloudformation-launch-badge.svg)

```md
> [!NOTE]
> Do not rewrite alerts inside code fences.
[Do not rewrite](./maintainers/release.md)
```

1. Preserve indented code fences:

   ```md
   > [!NOTE]
   > Do not rewrite indented alerts inside code fences.
   [Do not rewrite indented](./maintainers/release.md)
   ![Do not rewrite indented image](./assets/cloudformation-launch-badge.svg)
   ```
""",
            )
            _write(
                repo_root / "docs" / "architecture.md",
                """# Architecture

Understand the runtime path.

See [alternate](./design-history/cloudwatch-kinesis-lambda-relay.md) and [delayed](./design-history/long-poller-sqs-delayed-task.md).

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../svgs/architecture-flow-dark.svg">
  <img src="../svgs/architecture-flow-light.svg" alt="Architecture">
</picture>
""",
            )
            _write(repo_root / "docs" / "operate.md", "# Operate\n\nOperate.\n")
            _write(repo_root / "docs" / "reference.md", "# Reference\n\nReference.\n")
            _write(
                repo_root / "docs" / "maintainers" / "release.md",
                """# Release Guide

Maintainer workflow.

See [deploy](../deploy.md) and [local](#manual-sar-publish).
See [site](../../site/README.md).
""",
            )
            _write(
                repo_root / "docs" / "design-history" / "cloudwatch-kinesis-lambda-relay.md",
                """# CloudWatch Logs To Kinesis To Lambda Relay

Alternative design.

## Questions

See [architecture](../architecture.md).
""",
            )
            _write(
                repo_root / "docs" / "design-history" / "long-poller-sqs-delayed-task.md",
                """# Long Poller With SQS For Delayed Work

Historical option.
""",
            )
            _write(repo_root / "docs" / "assets" / "cloudformation-launch-badge.svg", "<svg />\n")
            _write(repo_root / "svgs" / "architecture-flow-dark.svg", "<svg />\n")
            _write(repo_root / "svgs" / "architecture-flow-light.svg", "<svg />\n")

            result = _run_generator(repo_root, content_root, public_root)
            self.assertEqual(result.returncode, 0, result.stderr)

            root_page = (content_root / "index.mdx").read_text(encoding="utf-8")
            deploy_page = (content_root / "deploy.mdx").read_text(encoding="utf-8")
            architecture_page = (content_root / "architecture.mdx").read_text(encoding="utf-8")
            release_page = (content_root / "maintainers" / "release.mdx").read_text(
                encoding="utf-8"
            )
            design_page = (
                content_root / "design-history" / "cloudwatch-kinesis-lambda-relay.mdx"
            ).read_text(encoding="utf-8")

            self.assertIn('title: "Signals Relay docs"', root_page)
            self.assertIn('description: "Get started."', root_page)
            self.assertIn("./deploy/", root_page)
            self.assertIn("./architecture/", root_page)
            self.assertIn("./reference/", root_page)
            self.assertIn("./maintainers/release/", root_page)
            self.assertIn("./design-history/cloudwatch-kinesis-lambda-relay/#questions", root_page)
            self.assertIn("./design-history/long-poller-sqs-delayed-task/", root_page)
            self.assertIn("../", root_page)
            self.assertIn("https://example.com", root_page)
            self.assertIn("mailto:hello@example.com", root_page)
            self.assertNotIn(".mdx", root_page)
            self.assertIn("./maintainers/release/", deploy_page)
            self.assertIn('<Callout type="info" title="Note">', deploy_page)
            self.assertIn("Keep this source compatible with GitHub alerts.", deploy_page)
            self.assertIn("> [!NOTE]", deploy_page)
            self.assertIn("[Do not rewrite](./maintainers/release.md)", deploy_page)
            self.assertIn("[Do not rewrite indented](./maintainers/release.md)", deploy_page)
            self.assertIn(
                "![Do not rewrite indented image](./assets/cloudformation-launch-badge.svg)",
                deploy_page,
            )
            self.assertIn(
                "![Launch badge](../../docs-assets/docs/assets/cloudformation-launch-badge.svg)",
                deploy_page,
            )
            self.assertIn('title: "Architecture"', architecture_page)
            self.assertIn(
                '<source media="(prefers-color-scheme: dark)" srcSet="../../docs-assets/svgs/architecture-flow-dark.svg" />',
                architecture_page,
            )
            self.assertNotIn("srcset=", architecture_page)
            self.assertIn("../../docs-assets/svgs/architecture-flow-light.svg", architecture_page)
            self.assertIn("./design-history/cloudwatch-kinesis-lambda-relay/", architecture_page)
            self.assertIn("./design-history/long-poller-sqs-delayed-task/", architecture_page)
            self.assertIn("../deploy/", release_page)
            self.assertIn("#manual-sar-publish", release_page)
            self.assertIn(
                "https://github.com/dev7a/signals-relay/blob/main/site/README.md",
                release_page,
            )
            self.assertIn("../architecture/", design_page)

            root_meta = json.loads((content_root / "meta.json").read_text(encoding="utf-8"))
            self.assertEqual(
                root_meta,
                {
                    "title": "Signals Relay docs",
                    "pages": [
                        "index",
                        "deploy",
                        "operate",
                        "architecture",
                        "reference",
                        "maintainers",
                        "design-history",
                    ],
                },
            )
            maintainers_meta = json.loads(
                (content_root / "maintainers" / "meta.json").read_text(encoding="utf-8")
            )
            self.assertEqual(
                maintainers_meta,
                {
                    "title": "Maintainers",
                    "pages": ["release"],
                },
            )
            design_meta = json.loads(
                (content_root / "design-history" / "meta.json").read_text(encoding="utf-8")
            )
            self.assertEqual(
                design_meta,
                {
                    "title": "Design history",
                    "pages": [
                        "cloudwatch-kinesis-lambda-relay",
                        "long-poller-sqs-delayed-task",
                    ],
                },
            )
            generated_nav = json.loads((content_root / "nav.json").read_text(encoding="utf-8"))
            self.assertEqual([item["label"] for item in generated_nav["primary"]], [
                "Docs",
                "Deploy",
                "Operate",
                "Architecture",
                "Reference",
            ])
            self.assertEqual([item["label"] for item in generated_nav["index"]], [
                "Deploy",
                "Operate",
                "Architecture",
                "Reference",
            ])
            self.assertEqual(
                [section["title"] for section in generated_nav["secondary"]],
                ["Maintainers", "Design history"],
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

    def test_generator_requires_page_metadata(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            repo_root = Path(tmp) / "repo"
            content_root = repo_root / "site" / "content" / "docs"
            public_root = repo_root / "site" / "public"
            _write(repo_root / "docs" / "README.md", "# Documentation\n\nFront door.\n")
            _write_nav(
                repo_root,
                pages=[
                    {
                        "source": "docs/README.md",
                        "target": "index.mdx",
                        "navLabel": "Docs",
                        "title": "Signals Relay docs",
                        "section": "primary",
                    }
                ],
            )

            result = _run_generator(repo_root, content_root, public_root)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("description", result.stderr)

    def test_generator_is_deterministic(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            repo_root = Path(tmp) / "repo"
            content_root = repo_root / "site" / "content" / "docs"
            public_root = repo_root / "site" / "public"

            _write_nav(repo_root)
            _write(repo_root / "docs" / "README.md", "# Documentation\n\nFront door.\n")
            _write(repo_root / "docs" / "deploy.md", "# Deploy\n\nDeploy.\n")
            _write(repo_root / "docs" / "architecture.md", "# Architecture\n\nArchitecture.\n")
            _write(repo_root / "docs" / "operate.md", "# Operate\n\nOperate.\n")
            _write(repo_root / "docs" / "reference.md", "# Reference\n\nReference.\n")
            _write(repo_root / "docs" / "maintainers" / "release.md", "# Release\n\nRelease.\n")
            _write(
                repo_root / "docs" / "design-history" / "cloudwatch-kinesis-lambda-relay.md",
                "# CloudWatch\n\nDesign.\n",
            )
            _write(
                repo_root / "docs" / "design-history" / "long-poller-sqs-delayed-task.md",
                "# Long poller\n\nDesign.\n",
            )

            first = _run_generator(repo_root, content_root, public_root)
            self.assertEqual(first.returncode, 0, first.stderr)
            snapshot = {
                path.relative_to(content_root).as_posix(): path.read_text(encoding="utf-8")
                for path in sorted(content_root.rglob("*"))
                if path.is_file()
            }

            second = _run_generator(repo_root, content_root, public_root)
            self.assertEqual(second.returncode, 0, second.stderr)
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
