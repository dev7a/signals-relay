#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import posixpath
import re
import shutil
from dataclasses import dataclass
from pathlib import Path, PurePosixPath

NAV_PATH = PurePosixPath("docs/nav.json")
ASSET_ROOTS = (
    (PurePosixPath("docs/assets"), PurePosixPath("docs-assets/docs/assets")),
    (PurePosixPath("svgs"), PurePosixPath("docs-assets/svgs")),
)
SITE_ROUTE_OVERRIDES = {
    PurePosixPath("README.md"): "",
}
SOURCE_LINK_OVERRIDES = {
    PurePosixPath("site/README.md"): "https://github.com/dev7a/signals-relay/blob/main/site/README.md",
}

FENCE_RE = re.compile(r"(?ms)^[ \t]*```.*?^[ \t]*```[ \t]*\n?")
MARKDOWN_IMAGE_RE = re.compile(r"!\[([^\]]*)\]\(([^)\s]+)(?:\s+\"([^\"]*)\")?\)")
MARKDOWN_LINK_RE = re.compile(r"(?<!!)\[([^\]]+)\]\(([^)\s]+)(?:\s+\"([^\"]*)\")?\)")
HTML_IMAGE_RE = re.compile(r"(?is)<img\b([^>]*?)\/?>")
HTML_SOURCE_RE = re.compile(r"(?is)<source\b([^>]*?)\/?>")
ATTR_RE = re.compile(r'([A-Za-z_:][-A-Za-z0-9_:.]*)\s*=\s*"([^"]*)"')
GITHUB_ALERT_RE = re.compile(
    r"^>\s*\[!(NOTE|TIP|IMPORTANT|WARNING|CAUTION)\]\s*(.*?)[ \t]*(?:\r?\n)?$",
    re.IGNORECASE,
)
GITHUB_ALERT_TYPES = {
    "NOTE": ("info", "Note"),
    "TIP": ("info", "Tip"),
    "IMPORTANT": ("warning", "Important"),
    "WARNING": ("warning", "Warning"),
    "CAUTION": ("error", "Caution"),
}


@dataclass(frozen=True)
class SectionRecord:
    id: str
    title: str


@dataclass(frozen=True)
class PageRecord:
    source_rel: PurePosixPath
    target_rel: PurePosixPath
    nav_label: str
    title: str
    description: str
    section: str


@dataclass(frozen=True)
class NavConfig:
    title: str
    sections: dict[str, SectionRecord]
    pages: list[PageRecord]


def normalize_repo_rel(path: PurePosixPath) -> PurePosixPath:
    normalized = posixpath.normpath(path.as_posix())
    if normalized == ".":
        return PurePosixPath(".")
    return PurePosixPath(normalized)


def resolve_repo_rel(base_dir: PurePosixPath, href: str) -> PurePosixPath | None:
    if not href or href.startswith(("#", "http://", "https://", "mailto:", "data:")):
        return None
    if href.startswith("/"):
        return normalize_repo_rel(PurePosixPath(href.lstrip("/")))
    resolved = normalize_repo_rel(base_dir / href)
    if resolved.parts and resolved.parts[0] == "..":
        return None
    return resolved


def replace_outside_code_fences(text: str, replacer) -> str:
    pieces: list[str] = []
    last = 0
    for match in FENCE_RE.finditer(text):
        pieces.append(replacer(text[last : match.start()]))
        pieces.append(match.group(0))
        last = match.end()
    pieces.append(replacer(text[last:]))
    return "".join(pieces)


def strip_leading_h1(text: str) -> str:
    lines = text.splitlines()
    index = 0
    while index < len(lines) and not lines[index].strip():
        index += 1
    if index < len(lines) and lines[index].startswith("# "):
        index += 1
        while index < len(lines) and not lines[index].strip():
            index += 1
        return "\n".join(lines[index:]).lstrip("\n")
    return text


def build_frontmatter(title: str, description: str) -> str:
    lines = ["---", f"title: {json.dumps(title)}"]
    lines.append(f"description: {json.dumps(description)}")
    lines.extend(("---", ""))
    return "\n".join(lines)


def route_path_for_generated_page(target_rel: PurePosixPath) -> str:
    if target_rel == PurePosixPath("index.mdx"):
        return ""
    path = target_rel.with_suffix("").as_posix()
    if path.endswith("/index"):
        return path[: -len("/index")]
    return path


def site_route_path_for_generated_page(target_rel: PurePosixPath) -> str:
    route = route_path_for_generated_page(target_rel)
    return "docs" if not route else f"docs/{route}"


def asset_site_path_for(repo_rel: PurePosixPath) -> PurePosixPath | None:
    for source_root, target_root in ASSET_ROOTS:
        try:
            suffix = repo_rel.relative_to(source_root)
        except ValueError:
            continue
        return target_root / suffix
    return None


def relative_asset_href(target_rel: PurePosixPath, repo_rel: PurePosixPath) -> str | None:
    asset_site_path = asset_site_path_for(repo_rel)
    if asset_site_path is None:
        return None
    current_site_route = site_route_path_for_generated_page(target_rel)
    return posixpath.relpath(asset_site_path.as_posix(), start=current_site_route or ".")


def rewrite_html_images(text: str, source_rel: PurePosixPath, target_rel: PurePosixPath) -> str:
    attr_names = {
        "class": "className",
        "srcset": "srcSet",
    }

    def rewrite_attrs(match: re.Match[str], tag_name: str, attr_name: str) -> str:
        attrs = ATTR_RE.findall(match.group(1))
        rewritten: list[tuple[str, str]] = []
        target_found = False
        for key, value in attrs:
            if key == attr_name:
                resolved = resolve_repo_rel(source_rel.parent, value)
                if resolved:
                    relative = relative_asset_href(target_rel, resolved)
                    if relative:
                        value = relative
                target_found = True
            rewritten.append((attr_names.get(key, key), value))
        if not target_found:
            return match.group(0)
        rendered = " ".join(f'{key}="{value}"' for key, value in rewritten)
        return f"<{tag_name} {rendered} />"

    text = HTML_IMAGE_RE.sub(lambda match: rewrite_attrs(match, "img", "src"), text)
    return HTML_SOURCE_RE.sub(lambda match: rewrite_attrs(match, "source", "srcset"), text)


def rewrite_markdown_images(text: str, source_rel: PurePosixPath, target_rel: PurePosixPath) -> str:
    def replace(match: re.Match[str]) -> str:
        alt, href, title = match.group(1), match.group(2), match.group(3)
        resolved = resolve_repo_rel(source_rel.parent, href)
        if not resolved:
            return match.group(0)
        relative = relative_asset_href(target_rel, resolved)
        if not relative:
            return match.group(0)
        title_suffix = f' "{title}"' if title else ""
        return f"![{alt}]({relative}{title_suffix})"

    return MARKDOWN_IMAGE_RE.sub(replace, text)


def rewrite_markdown_links(
    text: str,
    source_rel: PurePosixPath,
    target_rel: PurePosixPath,
    source_map: dict[PurePosixPath, PurePosixPath],
) -> str:
    current_route = route_path_for_generated_page(target_rel)

    def replace(match: re.Match[str]) -> str:
        label, href, title = match.group(1), match.group(2), match.group(3)
        if href.startswith(("#", "http://", "https://", "mailto:", "data:")):
            return match.group(0)

        raw_path, hash_sep, fragment = href.partition("#")
        resolved = resolve_repo_rel(source_rel.parent, raw_path)
        if not resolved:
            return match.group(0)
        if resolved in SITE_ROUTE_OVERRIDES:
            current_site_route = site_route_path_for_generated_page(target_rel)
            target_site_route = SITE_ROUTE_OVERRIDES[resolved]
            relative = posixpath.relpath(target_site_route or ".", start=current_site_route or ".")
            rewritten = "./" if relative == "." else relative
            if rewritten != "./" and not rewritten.startswith((".", "/")):
                rewritten = f"./{rewritten}"
            if not rewritten.endswith("/"):
                rewritten = f"{rewritten}/"
            if hash_sep:
                rewritten = f"{rewritten}#{fragment}"
            title_suffix = f' "{title}"' if title else ""
            return f"[{label}]({rewritten}{title_suffix})"
        source_link_override = SOURCE_LINK_OVERRIDES.get(resolved)
        if source_link_override:
            rewritten = source_link_override
            if hash_sep:
                rewritten = f"{rewritten}#{fragment}"
            title_suffix = f' "{title}"' if title else ""
            return f"[{label}]({rewritten}{title_suffix})"
        mapped = source_map.get(resolved)
        if mapped is None:
            return match.group(0)
        target_route = route_path_for_generated_page(mapped)
        relative = posixpath.relpath(target_route or ".", start=current_route or ".")
        rewritten = "./" if relative == "." else relative
        if rewritten != "./" and not rewritten.startswith((".", "/")):
            rewritten = f"./{rewritten}"
        if not rewritten.endswith("/"):
            rewritten = f"{rewritten}/"
        if hash_sep:
            rewritten = f"{rewritten}#{fragment}"
        title_suffix = f' "{title}"' if title else ""
        return f"[{label}]({rewritten}{title_suffix})"

    return MARKDOWN_LINK_RE.sub(replace, text)


def rewrite_content(
    text: str,
    source_rel: PurePosixPath,
    target_rel: PurePosixPath,
    source_map: dict[PurePosixPath, PurePosixPath],
) -> str:
    def replacer(segment: str) -> str:
        segment = rewrite_html_images(segment, source_rel, target_rel)
        segment = rewrite_markdown_images(segment, source_rel, target_rel)
        return rewrite_markdown_links(segment, source_rel, target_rel, source_map)

    return replace_outside_code_fences(text, replacer)


def transform_github_alerts(text: str) -> str:
    def replacer(segment: str) -> str:
        lines = segment.splitlines(keepends=True)
        output: list[str] = []
        index = 0

        while index < len(lines):
            match = GITHUB_ALERT_RE.match(lines[index])
            if match is None:
                output.append(lines[index])
                index += 1
                continue

            alert_kind = match.group(1).upper()
            first_line = match.group(2).strip()
            callout_type, title = GITHUB_ALERT_TYPES[alert_kind]
            body_lines: list[str] = []
            if first_line:
                body_lines.append(f"{first_line}\n")

            index += 1
            while index < len(lines):
                line = lines[index]
                if not line.startswith(">"):
                    break
                body_lines.append(re.sub(r"^>[ \t]?", "", line, count=1))
                index += 1

            body = "".join(body_lines).strip()
            output.append(f'<Callout type="{callout_type}" title="{title}">\n\n')
            if body:
                output.append(body)
                output.append("\n\n")
            output.append("</Callout>\n")

        return "".join(output)

    return replace_outside_code_fences(text, replacer)


def read_required_string(data: dict[str, object], key: str, context: str) -> str:
    value = data.get(key)
    if not isinstance(value, str) or not value.strip():
        raise ValueError(f"{context} must define a non-empty string field {key!r}")
    return value


def read_repo_path(data: dict[str, object], key: str, context: str) -> PurePosixPath:
    value = read_required_string(data, key, context)
    path = PurePosixPath(value)
    if path.is_absolute() or ".." in path.parts:
        raise ValueError(f"{context} field {key!r} must be a repository-relative path")
    return normalize_repo_rel(path)


def load_nav_config(repo_root: Path) -> NavConfig:
    nav_file = repo_root / NAV_PATH
    data = json.loads(nav_file.read_text(encoding="utf-8"))
    if not isinstance(data, dict):
        raise ValueError(f"{NAV_PATH} must contain a JSON object")

    title = read_required_string(data, "title", NAV_PATH.as_posix())

    raw_sections = data.get("sections")
    if not isinstance(raw_sections, list) or not raw_sections:
        raise ValueError(f"{NAV_PATH} must define a non-empty sections array")

    sections: dict[str, SectionRecord] = {}
    for index, raw_section in enumerate(raw_sections):
        context = f"{NAV_PATH}: sections[{index}]"
        if not isinstance(raw_section, dict):
            raise ValueError(f"{context} must be an object")
        section = SectionRecord(
            id=read_required_string(raw_section, "id", context),
            title=read_required_string(raw_section, "title", context),
        )
        if section.id in sections:
            raise ValueError(f"{context} duplicates section id {section.id!r}")
        sections[section.id] = section

    raw_pages = data.get("pages")
    if not isinstance(raw_pages, list) or not raw_pages:
        raise ValueError(f"{NAV_PATH} must define a non-empty pages array")

    pages: list[PageRecord] = []
    seen_sources: set[PurePosixPath] = set()
    seen_targets: set[PurePosixPath] = set()
    for index, raw_page in enumerate(raw_pages):
        context = f"{NAV_PATH}: pages[{index}]"
        if not isinstance(raw_page, dict):
            raise ValueError(f"{context} must be an object")
        source_rel = read_repo_path(raw_page, "source", context)
        target_rel = read_repo_path(raw_page, "target", context)
        section_id = read_required_string(raw_page, "section", context)
        if section_id not in sections:
            raise ValueError(f"{context} references unknown section {section_id!r}")
        if source_rel in seen_sources:
            raise ValueError(f"{context} duplicates source {source_rel}")
        if target_rel in seen_targets:
            raise ValueError(f"{context} duplicates target {target_rel}")
        if source_rel.suffix != ".md":
            raise ValueError(f"{context} source must be a Markdown file")
        if target_rel.suffix != ".mdx":
            raise ValueError(f"{context} target must be an MDX file")
        if not (repo_root / source_rel).exists():
            raise FileNotFoundError(f"{context} source does not exist: {source_rel}")
        seen_sources.add(source_rel)
        seen_targets.add(target_rel)
        pages.append(
            PageRecord(
                source_rel=source_rel,
                target_rel=target_rel,
                nav_label=read_required_string(raw_page, "navLabel", context),
                title=read_required_string(raw_page, "title", context),
                description=read_required_string(raw_page, "description", context),
                section=section_id,
            )
        )

    return NavConfig(title=title, sections=sections, pages=pages)


def build_source_map(records: list[PageRecord]) -> dict[PurePosixPath, PurePosixPath]:
    return {record.source_rel: record.target_rel for record in records}


def write_json(path: Path, data: dict[str, object]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")


def copy_assets(repo_root: Path, public_root: Path) -> None:
    target_root = public_root / "docs-assets"
    if target_root.exists():
        shutil.rmtree(target_root, ignore_errors=True)
    for source_root_rel, target_root_rel in ASSET_ROOTS:
        source_root = repo_root / source_root_rel
        if not source_root.exists():
            continue
        for source in sorted(source_root.rglob("*")):
            if source.is_dir():
                continue
            relative = source.relative_to(source_root)
            destination = public_root / target_root_rel / relative
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(source, destination)


def write_page(
    repo_root: Path,
    content_root: Path,
    record: PageRecord,
    source_map: dict[PurePosixPath, PurePosixPath],
) -> None:
    source_path = repo_root / record.source_rel
    raw = source_path.read_text(encoding="utf-8")
    body = strip_leading_h1(raw)
    body = transform_github_alerts(body)
    rewritten = rewrite_content(body, record.source_rel, record.target_rel, source_map).rstrip()
    destination = content_root / record.target_rel
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_text(
        build_frontmatter(record.title, record.description) + rewritten + "\n",
        encoding="utf-8",
    )


def page_key_for_meta(target_rel: PurePosixPath) -> str:
    if target_rel.name == "index.mdx":
        return "index"
    return target_rel.stem


def write_meta_files(content_root: Path, nav: NavConfig) -> None:
    root_pages: list[str] = []
    folder_pages: dict[PurePosixPath, list[str]] = {}
    folder_titles: dict[PurePosixPath, str] = {}

    for record in nav.pages:
        parent = record.target_rel.parent
        if parent == PurePosixPath("."):
            root_pages.append(page_key_for_meta(record.target_rel))
            continue

        folder = PurePosixPath(parent.parts[0])
        if folder.as_posix() not in root_pages:
            root_pages.append(folder.as_posix())
        folder_pages.setdefault(parent, []).append(page_key_for_meta(record.target_rel))
        folder_titles.setdefault(parent, nav.sections[record.section].title)

    write_json(
        content_root / "meta.json",
        {
            "title": nav.title,
            "pages": root_pages,
        },
    )

    for folder, pages in folder_pages.items():
        write_json(
            content_root / folder / "meta.json",
            {
                "title": folder_titles.get(folder, folder.name.replace("-", " ").title()),
                "pages": pages,
            },
        )


def docs_href_for_target(target_rel: PurePosixPath) -> str:
    route = route_path_for_generated_page(target_rel)
    return "/docs/" if not route else f"/docs/{route}/"


def nav_item_for_record(record: PageRecord) -> dict[str, str]:
    return {
        "label": record.nav_label,
        "title": record.title,
        "description": record.description,
        "href": docs_href_for_target(record.target_rel),
    }


def write_generated_nav(content_root: Path, nav: NavConfig) -> None:
    primary = [
        nav_item_for_record(record) for record in nav.pages if record.section == "primary"
    ]
    secondary = []
    for section_id, section in nav.sections.items():
        if section_id == "primary":
            continue
        items = [
            nav_item_for_record(record) for record in nav.pages if record.section == section_id
        ]
        if items:
            secondary.append({"id": section_id, "title": section.title, "items": items})

    write_json(
        content_root / "nav.json",
        {
            "title": nav.title,
            "primary": primary,
            "index": [item for item in primary if item["href"] != "/docs/"],
            "secondary": secondary,
        },
    )


def generate_site_content(repo_root: Path, content_root: Path, public_root: Path) -> None:
    nav = load_nav_config(repo_root)
    source_map = build_source_map(nav.pages)

    if content_root.exists():
        shutil.rmtree(content_root)
    content_root.mkdir(parents=True, exist_ok=True)

    for record in nav.pages:
        write_page(repo_root, content_root, record, source_map)
    write_meta_files(content_root, nav)
    write_generated_nav(content_root, nav)
    copy_assets(repo_root, public_root)


def parse_args() -> argparse.Namespace:
    script_path = Path(__file__).resolve()
    repo_root = script_path.parent.parent
    parser = argparse.ArgumentParser(description="Generate Fumadocs content from repository docs.")
    parser.add_argument("--repo-root", type=Path, default=repo_root)
    parser.add_argument(
        "--content-root", type=Path, default=repo_root / "site" / "content" / "docs"
    )
    parser.add_argument("--public-root", type=Path, default=repo_root / "site" / "public")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    generate_site_content(
        repo_root=args.repo_root.resolve(),
        content_root=args.content_root.resolve(),
        public_root=args.public_root.resolve(),
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
