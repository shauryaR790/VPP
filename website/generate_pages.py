#!/usr/bin/env python3
"""Generate documentation HTML pages for the V++ website."""

from __future__ import annotations

import html
import json
import re
from pathlib import Path

from course_catalog import (
    CourseProject,
    code_block_html,
    discover_course_projects,
)
from pdf_docs import DocsBlock, DocsDocument, DocsSection, load_docs

ROOT = Path(__file__).resolve().parent.parent
WEBSITE = ROOT / "website"
DOCS = ROOT / "docs"

NAV = [
    ("learn.html", "Learn"),
    ("about.html", "About"),
    ("download.html", "Download"),
    ("blog.html", "Blog"),
    ("docs.html", "Docs"),
    ("contribute.html", "Contribute"),
    ("courses.html", "Courses"),
]

ASSET_PREFIX = "/VPP/"

GITHUB_REPO = "shauryaR790/VPP"

BRAND = "V++"

GITHUB_SVG = (
    '<svg width="20" height="20" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">'
    '<path fill-rule="evenodd" d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38'
    ' 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53'
    ' .63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95'
    ' 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.18.82.63-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27'
    ' 1.51-1.04 2.18-.82 2.18-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48'
    ' 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z"/>'
    '</svg>'
)

TREE_CHEVRON = (
    '<svg class="tree-chevron" width="12" height="12" viewBox="0 0 16 16" aria-hidden="true">'
    '<path fill="currentColor" d="M6.22 3.22a.75.75 0 0 1 1.06 0l4.25 4.25a.75.75 0 0 1 0 1.06l-4.25 4.25a.751.751 0 0 1-1.042-.018.751.751 0 0 1-.018-1.042L9.94 8 6.22 4.28a.75.75 0 0 1 0-1.06Z"/>'
    '</svg>'
)

TREE_FOLDER = (
    '<svg class="tree-icon tree-icon-folder" width="16" height="16" viewBox="0 0 16 16" aria-hidden="true">'
    '<path fill="currentColor" d="M1.75 1A1.75 1.75 0 0 0 0 2.75v10.5C0 14.216.784 15 1.75 15h12.5A1.75 1.75 0 0 0 16 13.25V6.75A1.75 1.75 0 0 0 14.25 5H8.06l-.72-1.44A1.75 1.75 0 0 0 5.68 2H1.75Z"/>'
    '</svg>'
)

TREE_SEARCH = (
    '<svg class="tree-search-icon" width="14" height="14" viewBox="0 0 16 16" aria-hidden="true">'
    '<path fill="currentColor" d="M10.68 11.74a6 6 0 0 1-7.922-8.982 6 6 0 0 1 8.982 7.922l3.04 3.04a.749.749 0 0 1-.326 1.275.749.749 0 0 1-.734-.215ZM11.5 7a4.499 4.499 0 1 0-8.997 0A4.499 4.499 0 0 0 11.5 7Z"/>'
    '</svg>'
)

TREE_FILE = (
    '<svg class="tree-icon tree-icon-file" width="16" height="16" viewBox="0 0 16 16" aria-hidden="true">'
    '<path fill="currentColor" d="M2 1.75C2 .784 2.784 0 3.75 0h5.086c.464 0 .909.184 1.237.513l3.414 3.414c.329.328.513.773.513 1.237v9.586A1.75 1.75 0 0 1 12.25 16h-8.5A1.75 1.75 0 0 1 2 14.25Zm1.75-.25a.25.25 0 0 0-.25.25v12.5c0 .138.112.25.25.25h8.5a.25.25 0 0 0 .25-.25V6h-2.75A1.75 1.75 0 0 1 8 4.25V1.5Z"/>'
    '</svg>'
)

FAQ_CHEVRON = (
    '<svg class="faq-chevron" width="14" height="14" viewBox="0 0 16 16" aria-hidden="true">'
    '<path fill="currentColor" d="M6.22 3.22a.75.75 0 0 1 1.06 0l4.25 4.25a.75.75 0 0 1 0 1.06l-4.25 4.25a.751.751 0 0 1-1.042-.018.751.751 0 0 1-.018-1.042L9.94 8 6.22 4.28a.75.75 0 0 1 0-1.06Z"/>'
    '</svg>'
)

EXTRA_MD = [
    ROOT / "SPEC.md",
    ROOT / "ARCHITECTURE.md",
    ROOT / "MEMORY_MODEL.md",
    ROOT / "CHANGELOG.md",
    ROOT / "CONTRIBUTING.md",
    ROOT / "README.md",
]


MIN_CODE_LINES = 5

COMMENT_LINES: dict[str, list[str]] = {
    "bash": [
        "# Verify install: vpp doctor",
        "# Restart the terminal after PATH changes",
        "# Type-check: vpp check main.vpp",
        "# Format source: vpp fmt main.vpp",
        "# Native build: vpp build src/main.vpp -o app.exe",
    ],
    "vpp": [
        "// Run: vpp run main.vpp",
        "// Type-check only: vpp check main.vpp",
        "// Format: vpp fmt main.vpp",
        "// Tests: vpp test",
        "// vpp calls main() when defined",
    ],
    "toml": [
        "# Package manifest for vpp run / vpp test / vpp build",
        "# Add dependencies with: vpp add <name> --version <ver>",
        "# entry points to src/main.vpp by default",
    ],
    "rust": [
        "// V++ compiler source (Rust)",
        "// See ARCHITECTURE.md for the pipeline overview",
    ],
    "text": [
        "# ...",
        "# See docs for full examples",
    ],
}


def comment_pad(plang: str, index: int) -> str:
    pool = COMMENT_LINES.get(plang, COMMENT_LINES["text"])
    return pool[index % len(pool)]


def enrich_code_lines(lines: list[str], plang: str) -> list[str]:
    """Expand very short shell snippets with real related commands."""
    non_empty = [ln for ln in lines if ln.strip()]
    if len(non_empty) >= MIN_CODE_LINES:
        return non_empty

    if not non_empty:
        return lines

    first = non_empty[0].strip()

    if plang == "bash":
        if first.startswith("vpp run "):
            target = first[8:].strip()
            return [
                first,
                f"vpp check {target}",
                "vpp --version",
                "# Verify your install",
                "vpp doctor",
            ]
        if first.startswith("vpp check "):
            target = first[10:].strip()
            return [
                first,
                f"vpp run {target}",
                f"vpp fmt {target}",
                "vpp test",
                "vpp build src/main.vpp -o app.exe",
            ]
        if first in ("vpp --version", "vpp doctor"):
            return [
                "vpp --version",
                "vpp doctor",
                "vpp run examples/hello.vpp",
                "vpp check examples/hello.vpp",
                "vpp fmt examples/hello.vpp",
            ]
        if first.startswith("vpp build"):
            merged = list(non_empty)
            for extra in ("./app.exe", "vpp run app.vpp", "vpp test", "# Release binary"):
                if len(merged) >= MIN_CODE_LINES:
                    break
                if extra not in merged:
                    merged.append(extra)
            return merged
        if first.startswith("vpp new"):
            return [
                first,
                "cd myapp",
                "vpp run",
                "vpp test",
                "vpp build src/main.vpp -o myapp.exe",
            ]
        if first.startswith("cd ") and any("vpp run" in ln for ln in non_empty):
            merged = list(non_empty)
            for extra in ("vpp check main.vpp", "vpp test", "vpp --version", "# Module project"):
                if len(merged) >= MIN_CODE_LINES:
                    break
                if extra not in merged:
                    merged.append(extra)
            return merged
        if first.startswith("$") or "$env:" in first or "[Environment]" in first:
            merged = list(non_empty)
            for extra in (
                "# Restart terminal after updating PATH",
                "vpp --version",
                "vpp doctor",
                "vpp run examples/hello.vpp",
            ):
                if len(merged) >= MIN_CODE_LINES:
                    break
                if extra not in merged:
                    merged.append(extra)
            return merged
        if first.startswith("vpp add"):
            merged = list(non_empty)
            for extra in ("vpp install", "vpp run", "vpp test", "# Resolves deps from vpp.toml"):
                if len(merged) >= MIN_CODE_LINES:
                    break
                if extra not in merged:
                    merged.append(extra)
            return merged
        if first.startswith("vpp test"):
            merged = list(non_empty)
            for extra in ("vpp run", "vpp check src/main.vpp", "vpp --version", "# Inline test blocks supported"):
                if len(merged) >= MIN_CODE_LINES:
                    break
                if extra not in merged:
                    merged.append(extra)
            return merged
        if first.startswith("vpp fmt"):
            merged = list(non_empty)
            for extra in ("vpp check main.vpp", "vpp run main.vpp", "# Writes formatted source in place"):
                if len(merged) >= MIN_CODE_LINES:
                    break
                if extra not in merged:
                    merged.append(extra)
            return merged
        if any("vpp" in ln for ln in non_empty):
            merged = list(non_empty)
            for extra in (
                "vpp --version",
                "vpp doctor",
                "vpp run examples/hello.vpp",
                "vpp check examples/hello.vpp",
            ):
                if len(merged) >= MIN_CODE_LINES:
                    break
                if extra not in merged:
                    merged.append(extra)
            return merged

    if plang == "vpp" and len(non_empty) < MIN_CODE_LINES:
        merged = list(non_empty)
        for extra in (
            "fn main() -> int {",
            "    return 0",
            "}",
        ):
            if len(merged) >= MIN_CODE_LINES:
                break
            if extra not in merged:
                merged.append(extra)
        if len(merged) < MIN_CODE_LINES:
            i = 0
            while len(merged) < MIN_CODE_LINES:
                merged.append(comment_pad("vpp", i))
                i += 1
        return merged

    return non_empty


def finalize_code_lines(lines: list[str], plang: str) -> list[str]:
    """No blank lines  -  pad short blocks with comments to MIN_CODE_LINES."""
    cleaned = [ln for ln in lines if ln.strip()]
    if not cleaned:
        return [comment_pad(plang, i) for i in range(MIN_CODE_LINES)]

    result = enrich_code_lines(cleaned, plang)
    result = [ln for ln in result if ln.strip()]

    i = 0
    while len(result) < MIN_CODE_LINES:
        result.append(comment_pad(plang, i))
        i += 1
    return result


def code_filename(display: str, plang: str, lines: list[str]) -> str:
    if plang == "vpp":
        return "main.vpp"
    if plang == "toml":
        return "vpp.toml"
    if plang == "rust":
        return "lib.rs"
    if plang == "bash":
        for ln in lines:
            s = ln.strip()
            if s.startswith("vpp run "):
                path = s[8:].strip().replace("\\", "/")
                name = path.split("/")[-1]
                return name if name else "terminal"
        return "terminal"
    return display.lower()


def slug(text: str) -> str:
    s = re.sub(r"[^a-zA-Z0-9]+", "-", text.lower()).strip("-")
    return s or "section"


def clean_prose(text: str) -> str:
    """Remove dashes and hyphens from user-visible prose (not inline code or URLs)."""
    placeholders: list[str] = []

    def stash(match: re.Match[str]) -> str:
        placeholders.append(match.group(0))
        return f"\x00{len(placeholders) - 1}\x00"

    protected = text
    protected = re.sub(r"`[^`]*`", stash, protected)
    protected = re.sub(r"\[[^\]]*\]\([^)]*\)", stash, protected)
    protected = re.sub(r"https?://[^\s)]+", stash, protected)
    protected = protected.replace(" - ", " ")
    protected = protected.replace("–", " ")
    protected = protected.replace("--", " ")
    protected = re.sub(r"-", " ", protected)
    protected = re.sub(r" +", " ", protected)
    for i, original in enumerate(placeholders):
        protected = protected.replace(f"\x00{i}\x00", original)
    return protected.strip()


class SlugRegistry:
    """Assign unique HTML id slugs across a full generated page."""

    def __init__(self) -> None:
        self._used: set[str] = set()

    def unique(self, text: str) -> str:
        base = slug(text)
        if base not in self._used:
            self._used.add(base)
            return base
        n = 2
        while f"{base}-{n}" in self._used:
            n += 1
        candidate = f"{base}-{n}"
        self._used.add(candidate)
        return candidate


def md_to_html(text: str, slugs: SlugRegistry | None = None) -> str:
    """Minimal markdown to HTML  -  enough for our docs."""
    registry = slugs or SlugRegistry()
    lines = text.splitlines()
    out: list[str] = []
    in_code = False
    in_table = False
    code_lang = ""
    code_lines: list[str] = []
    list_open = False
    release_prefix = ""

    def close_list():
        nonlocal list_open
        if list_open:
            out.append("</ul>")
            list_open = False

    def close_code_block():
        nonlocal in_code, code_lines, code_lang
        if not in_code:
            return
        label = code_lang or "text"
        if label in ("powershell", "shell", "bash", "sh"):
            display = "Shell"
        elif label in ("vpp", "v++"):
            display = BRAND
        elif label == "toml":
            display = "TOML"
        else:
            display = label if label else "text"
        plang = {
            "powershell": "bash", "shell": "bash", "sh": "bash",
            "vpp": "vpp", "v++": "vpp",
            "toml": "toml", "bash": "bash", "rust": "rust",
        }.get(label, label or "text")
        filename = code_filename(display, plang, code_lines)
        code_lines = finalize_code_lines(code_lines, plang)
        code_text = html.escape("\n".join(code_lines))
        out.append('<div class="code-block-wrap">')
        out.append(
            f'<div class="code-block-header">'
            f'<span class="code-block-filename">{html.escape(filename)}</span>'
            f"</div>"
        )
        out.append(
            f'<pre class="language-{plang}">'
            f'<code class="language-{plang}">{code_text}</code></pre>'
        )
        out.append("</div>")
        in_code = False
        code_lines = []
        code_lang = ""

    i = 0
    while i < len(lines):
        line = lines[i]

        if line.strip().startswith("```"):
            close_list()
            if in_code:
                close_code_block()
            else:
                code_lang = line.strip()[3:].strip()
                code_lines = []
                in_code = True
            i += 1
            continue

        if in_code:
            code_lines.append(line)
            i += 1
            continue

        if line.strip().startswith("|") and "|" in line.strip()[1:]:
            close_list()
            if not in_table:
                out.append('<div class="table-wrap"><table>')
                in_table = True
            cells = [c.strip() for c in line.strip().strip("|").split("|")]
            if all(re.match(r"^:?-+:?$", c.replace(" ", "")) for c in cells if c):
                i += 1
                continue
            tag = "th" if not any("<td>" in x for x in out[-3:]) and "<table>" in out[-1] else "td"
            if in_table and i > 0 and lines[i - 1].strip().startswith("|") and "<thead>" not in "".join(out[-5:]):
                # first row as header
                if tag == "td" and out[-1] == "<div class=\"table-wrap\"><table>":
                    out.append("<thead><tr>")
                    for c in cells:
                        out.append(f"<th>{inline_md(c)}</th>")
                    out.append("</tr></thead><tbody>")
                    i += 1
                    continue
            out.append("<tr>")
            for c in cells:
                out.append(f"<td>{inline_md(c)}</td>")
            out.append("</tr>")
            i += 1
            continue
        elif in_table:
            out.append("</tbody></table></div>")
            in_table = False

        if not line.strip():
            close_list()
            i += 1
            continue

        if re.match(r"^-{3,}\s*$", line.strip()):
            close_list()
            out.append("<hr>")
            i += 1
            continue

        if line.strip().startswith("<!--") or line.strip().startswith("&lt;!--"):
            i += 1
            continue

        if line.startswith("#### "):
            close_list()
            out.append(f'<h4 id="{registry.unique(line[5:])}">{inline_md(line[5:])}</h4>')
        elif line.startswith("### "):
            close_list()
            title = line[4:]
            if release_prefix:
                hid = registry.unique(f"{release_prefix}-{title}")
            else:
                hid = registry.unique(title)
            out.append(f'<h3 id="{hid}">{inline_md(title)}</h3>')
        elif line.startswith("## "):
            close_list()
            title = line[3:]
            version = re.search(r"\[([\d.]+)\]", title)
            release_prefix = slug(version.group(1)) if version else ""
            out.append(f'<h2 id="{registry.unique(title)}">{inline_md(title)}</h2>')
        elif line.startswith("# "):
            close_list()
            release_prefix = ""
            out.append(f'<h1 id="{registry.unique(line[2:])}">{inline_md(line[2:])}</h1>')
        elif line.startswith("- ") or line.startswith("* "):
            if not list_open:
                out.append("<ul>")
                list_open = True
            out.append(f"<li>{inline_md(line[2:])}</li>")
        elif re.match(r"^\d+\.\s", line):
            close_list()
            out.append(f"<p>{inline_md(line)}</p>")
        else:
            close_list()
            out.append(f"<p>{inline_md(line)}</p>")
        i += 1

    close_list()
    if in_code:
        close_code_block()
    if in_table:
        out.append("</tbody></table></div>")
    return "\n".join(out)


def faq_md_to_html(text: str, slugs: SlugRegistry | None = None) -> str:
    """Render FAQ markdown as a collapsible accordion."""
    registry = slugs or SlugRegistry()
    lines = text.splitlines()
    out: list[str] = []
    i = 0

    if i < len(lines) and lines[i].startswith("# "):
        title = lines[i][2:].strip()
        out.append(f'<h1 id="{registry.unique(title)}">{inline_md(title)}</h1>')
        i += 1

    out.append('<div class="faq-list">')
    question: str | None = None
    section_lines: list[str] = []
    first_item = True

    def flush_section() -> None:
        nonlocal question, section_lines, first_item
        if question is None:
            return
        body = "\n".join(section_lines).strip()
        body_html = md_to_html(body, registry) if body else ""
        qid = registry.unique(question)
        open_attr = " open" if first_item else ""
        first_item = False
        out.append(
            f'<details class="faq-item" id="{qid}"{open_attr}>'
            f'<summary class="faq-question">{FAQ_CHEVRON}'
            f'<span class="faq-question-text">{inline_md(question)}</span></summary>'
            f'<div class="faq-answer">{body_html}</div>'
            f"</details>"
        )
        question = None
        section_lines = []

    while i < len(lines):
        line = lines[i]
        if line.startswith("## "):
            flush_section()
            question = line[3:].strip()
            section_lines = []
        elif question is not None:
            section_lines.append(line)
        i += 1

    flush_section()
    out.append("</div>")
    return "\n".join(out)


def render_paths(
    paths: list[Path],
    use_sources: bool = False,
    strip_h1_from: set[str] | None = None,
    slugs: SlugRegistry | None = None,
) -> str:
    registry = slugs or SlugRegistry()
    parts: list[str] = []
    strip = strip_h1_from or set()
    for p in paths:
        if not p.exists():
            continue
        if p.name == "faq.md":
            parts.append(faq_md_to_html(p.read_text(encoding="utf-8"), registry))
            continue
        if use_sources and p.suffix in (".vpp", ".toml", ".rs"):
            parts.append(md_to_html(source_to_md(p), registry))
            continue
        text = p.read_text(encoding="utf-8")
        if p.name in strip:
            lines = text.splitlines()
            if lines and lines[0].startswith("# "):
                text = "\n".join(lines[1:]).lstrip("\n")
        parts.append(md_to_html(text, registry))
    return "\n\n".join(parts)


def brand_text(s: str) -> str:
    return re.sub(r"v\+\+", BRAND, s)


def json_for_script(data: object) -> str:
    """Embed JSON in a script tag without HTML entity breakage."""
    return json.dumps(data, ensure_ascii=False).replace("</", "<\\/")


def course_inline_md(s: str) -> str:
    """Markdown subset for course lessons (preserves -> in signatures)."""
    s = html.escape(s)
    s = re.sub(r"`([^`]+)`", r"<code>\1</code>", s)
    s = re.sub(r"\*\*([^*]+)\*\*", r"<strong>\1</strong>", s)
    return brand_text(s)


def inline_md(s: str) -> str:
    s = clean_prose(s)
    s = html.escape(s)

    def link_repl(match: re.Match[str]) -> str:
        label, url = match.group(1), match.group(2)
        if url.startswith(("http://", "https://", "mailto:", "#", "/")):
            href = url
        elif url.endswith(".html"):
            href = f"{ASSET_PREFIX}{url}"
        else:
            href = url
        return f'<a href="{href}">{label}</a>'

    s = re.sub(r"`([^`]+)`", r"<code>\1</code>", s)
    s = re.sub(r"\*\*([^*]+)\*\*", r"<strong>\1</strong>", s)
    s = re.sub(r"\[([^\]]+)\]\(([^)]+)\)", link_repl, s)
    return brand_text(s)


def collect_md(paths: list[Path]) -> str:
    parts = []
    for p in paths:
        if p.exists():
            parts.append(p.read_text(encoding="utf-8"))
    return "\n\n".join(parts)


def source_to_md(path: Path) -> str:
    """Wrap source files as markdown sections for the reference page."""
    rel = path.relative_to(ROOT).as_posix()
    text = path.read_text(encoding="utf-8")
    if path.suffix == ".vpp":
        fence = "vpp"
    elif path.suffix == ".toml":
        fence = "toml"
    elif path.suffix == ".rs":
        fence = "rust"
    else:
        return f"\n\n<!-- {rel} -->\n\n{text}"
    if "projects/" in rel and path.name == "main.vpp":
        return f"\n\n```{fence}\n{text}\n```\n"
    heading = f"### `{rel}`"
    return f"\n\n{heading}\n\n```{fence}\n{text}\n```\n"


def collect_sources(paths: list[Path]) -> str:
    parts: list[str] = []
    for p in paths:
        if not p.exists():
            continue
        if p.suffix in (".vpp", ".toml", ".rs"):
            parts.append(source_to_md(p))
        else:
            rel = p.relative_to(ROOT).as_posix()
            parts.append(f"\n\n<!-- file: {rel} -->\n\n{p.read_text(encoding='utf-8')}")
    return "\n".join(parts)


def dedupe_paths(paths: list[Path]) -> list[Path]:
    seen: set[Path] = set()
    out: list[Path] = []
    for p in paths:
        key = p.resolve()
        if key in seen or not p.exists():
            continue
        seen.add(key)
        out.append(p)
    return out


def theory_doc_sources() -> list[Path]:
    """Prose documentation for docs.html: spec, architecture, language reference."""
    paths: list[Path] = []
    for rel in (
        "SPEC.md",
        "ARCHITECTURE.md",
        "MEMORY_MODEL.md",
        "CHANGELOG.md",
    ):
        paths.append(ROOT / rel)
    for rel in (
        "docs/README.md",
        "docs/project/roadmap.md",
    ):
        paths.append(ROOT / rel)
    for section in ("language", "guides", "stdlib"):
        folder = DOCS / section
        if folder.exists():
            paths.extend(sorted(folder.rglob("*.md")))
    return dedupe_paths(paths)


def headings_from_html(content: str) -> list[tuple[str, str, str]]:
    toc = []
    for m in re.finditer(r'<h([1234]) id="([^"]+)">([^<]+)</h\1>', content):
        level, hid, title = m.group(1), m.group(2), m.group(3)
        toc.append((level, hid, title))
    return toc


LEGAL_DIR = DOCS / "legal"  # deprecated; kept for reference only

LATEST_VERSION = "1.0.4"
LATEST_TAG = "v1.0.4"
EXTENSION_VERSION = "1.2.1"
RELEASE_VERSIONS = [
    ("1.0.4", "v1.0.4", True),
    ("1.0.3", "v1.0.3", False),
    ("1.0.2", "v1.0.2", False),
    ("0.7.0", "v0.7.0", False),
    ("0.6.2", "v0.6.2", False),
    ("0.5.0", "v0.5.0", False),
    ("0.4.4", "v0.4.4", False),
]

GITHUB_RELEASE = f"https://github.com/{GITHUB_REPO}/releases/download"


def build_download_page_body() -> str:
    version_opts = "\n".join(
        f'<option value="{ver}"{" selected" if ver == LATEST_VERSION else ""}>'
        f'v{ver}{" (latest)" if latest else ""}</option>'
        for ver, _tag, latest in RELEASE_VERSIONS
    )
    releases_table = "\n".join(
        f'<tr><td>{tag}</td>'
        f'<td><a href="https://github.com/{GITHUB_REPO}/releases/download/{tag}/vpp-{ver}-setup.exe">'
        f'vpp-{ver}-setup.exe</a></td>'
        f'<td><a href="https://github.com/{GITHUB_REPO}/releases/download/{tag}/'
        f'vpp-v{ver}-windows-x64.zip">zip</a></td>'
        f'<td><a href="https://github.com/{GITHUB_REPO}/releases/tag/{tag}">Release page</a></td></tr>'
        for ver, tag, _ in RELEASE_VERSIONS
    )
    return f"""<div id="top"></div>
<div class="download-hub" id="download-hub">
  <h1>Download {BRAND}</h1>

  <p class="download-picker-line">
    Get
    <span class="dl-select-wrap">
      <span class="dl-select-icon" id="dl-version-icon" aria-hidden="true"></span>
      <select id="dl-version" class="dl-select dl-select-iconed" aria-label="Version">{version_opts}</select>
    </span>
    for
    <span class="dl-select-wrap">
      <span class="dl-select-icon" id="dl-os-icon" aria-hidden="true"></span>
      <select id="dl-os" class="dl-select dl-select-iconed" aria-label="Operating system">
        <option value="windows">Windows</option>
        <option value="linux">Linux</option>
        <option value="macos">macOS</option>
      </select>
    </span>
    using
    <span class="dl-select-wrap">
      <span class="dl-select-icon" id="dl-format-icon" aria-hidden="true"></span>
      <select id="dl-format" class="dl-select dl-select-iconed" aria-label="Package type"></select>
    </span>
  </p>

  <div class="download-info" id="dl-info" role="status"></div>

  <div class="code-block-wrap" id="dl-code-wrap">
    <div class="code-block-header">
      <span class="code-block-filename" id="dl-code-filename">terminal</span>
    </div>
    <pre class="language-powershell" id="dl-code-pre"><code class="language-powershell" id="dl-code"># Loading…</code></pre>
  </div>
  <p class="download-code-note" id="dl-code-note" hidden></p>

  <div class="code-block-wrap" id="dl-path-wrap" hidden>
    <div class="code-block-header">
      <span class="code-block-filename">terminal</span>
    </div>
    <pre class="language-powershell"><code class="language-powershell">$dir = "$env:LOCALAPPDATA\\Programs\\vpp"
[Environment]::SetEnvironmentVariable("Path", $env:Path + ";$dir;$dir\\llvm\\bin", "User")
# Restart terminal after updating PATH
vpp --version
vpp doctor</code></pre>
  </div>

  <div class="download-actions">
    <a id="dl-primary" class="btn-dl-primary" href="{GITHUB_RELEASE}/{LATEST_TAG}/vpp-{LATEST_VERSION}-setup.exe" target="_blank" rel="noopener">
      <span class="btn-dl-icon" id="dl-primary-icon" aria-hidden="true"></span>
      <span id="dl-primary-label">Download vpp-{LATEST_VERSION}-setup.exe</span>
    </a>
    <a id="dl-secondary" class="btn-dl-secondary" href="{GITHUB_RELEASE}/{LATEST_TAG}/vpp-v{LATEST_VERSION}-windows-x64.zip" target="_blank" rel="noopener">
      <span class="btn-dl-icon" id="dl-secondary-icon" aria-hidden="true"></span>
      <span id="dl-secondary-label">Portable (.zip)</span>
    </a>
  </div>

  <p class="download-detect" id="dl-detect"></p>

  <div class="download-footlinks">
    <p>Read the <a href="{ASSET_PREFIX}blog.html">changelog</a> for release notes.</p>
    <p>Learn more about <a href="https://github.com/{GITHUB_REPO}/releases" target="_blank" rel="noopener">all releases</a> on GitHub.</p>
    <p>Need to hack on the compiler? See <a href="{ASSET_PREFIX}contribute.html">building from source</a>.</p>
  </div>

  <h2 id="all-releases">All releases</h2>
  <div class="table-wrap"><table>
  <thead><tr><th>Version</th><th>Windows installer</th><th>Portable zip</th><th>Notes</th></tr></thead>
  <tbody>
  {releases_table}
  <tr><td>v0.3.0</td><td colspan="3"><a href="https://github.com/{GITHUB_REPO}/releases/tag/v0.3.0">Modules, package manager, stdlib</a></td></tr>
  <tr><td>v0.2.0</td><td colspan="3"><a href="https://github.com/{GITHUB_REPO}/releases/tag/v0.2.0">Native IR + LLVM</a></td></tr>
  <tr><td>v0.1.0</td><td colspan="3"><a href="https://github.com/{GITHUB_REPO}/releases/tag/v0.1.0">Initial release</a></td></tr>
  </tbody></table></div>

  <h2 id="vscode">VS Code extension</h2>
  <p>Search <strong>v++ Language</strong> in VS Code Extensions (publisher: <strong>vpp-lang</strong>, version <strong>{EXTENSION_VERSION}</strong>) or install from the
  <a href="https://marketplace.visualstudio.com/items?itemName=vpp-lang.vplusplus" target="_blank" rel="noopener">Marketplace</a>.</p>
  <p>Pair with compiler <strong>v{LATEST_VERSION}</strong> from GitHub Releases for debug (F5), Test Explorer, and format-on-save.</p>
</div>"""


def shell(
    active: str,
    title: str,
    body: str,
    sidebar_html: str,
    toc_html: str,
    desc: str = "",
    extra_scripts: list[str] | None = None,
    body_class: str = "page-docs",
) -> str:
    nav_items = "\n".join(
        f'<a href="{ASSET_PREFIX}{href}" class="nav-link{" active" if href == active else ""}">{label}</a>'
        for href, label in NAV
    )
    meta = f'<meta name="description" content="{html.escape(clean_prose(desc))}">' if desc else ""
    return f"""<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{html.escape(clean_prose(title))} | {BRAND}</title>
  {meta}
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?family=IBM+Plex+Sans:wght@400;500;600;700&family=IBM+Plex+Mono:wght@400;500&family=Shrikhand&display=swap" rel="stylesheet">
  <link rel="stylesheet" href="{ASSET_PREFIX}css/style.css">
  <link rel="stylesheet" href="{ASSET_PREFIX}css/prism-vpp.css">
  <link rel="icon" href="{ASSET_PREFIX}assets/favicon.png">
  <link rel="apple-touch-icon" href="{ASSET_PREFIX}assets/logo-header.png">
</head>
<body class="{body_class}">
  <header class="site-header">
    <div class="header-inner">
      <a href="{ASSET_PREFIX}index.html" class="brand"><img src="{ASSET_PREFIX}assets/logo-header.png" alt="{BRAND}" class="brand-logo"></a>
      <nav class="top-nav">{nav_items}</nav>
      <div class="header-actions">
        <a href="https://github.com/{GITHUB_REPO}" class="icon-btn" aria-label="GitHub" target="_blank" rel="noopener">
          {GITHUB_SVG}
        </a>
      </div>
      <button class="nav-toggle" aria-label="Menu">☰</button>
    </div>
  </header>
  <div class="docs-layout">
    <aside class="docs-sidebar">{sidebar_html}</aside>
    <main class="docs-main">
      <article class="docs-article">{body}</article>
    </main>
    <aside class="docs-toc">
      <p class="toc-label">On this page</p>
      <nav class="toc-tree" aria-label="On this page">{toc_html}</nav>
    </aside>
  </div>
  <script src="https://cdn.jsdelivr.net/npm/prismjs@1.29.0/prism.min.js"></script>
  <script src="https://cdn.jsdelivr.net/npm/prismjs@1.29.0/components/prism-clike.min.js"></script>
  <script src="https://cdn.jsdelivr.net/npm/prismjs@1.29.0/components/prism-javascript.min.js"></script>
  <script src="https://cdn.jsdelivr.net/npm/prismjs@1.29.0/components/prism-bash.min.js"></script>
  <script src="https://cdn.jsdelivr.net/npm/prismjs@1.29.0/components/prism-toml.min.js"></script>
  <script src="https://cdn.jsdelivr.net/npm/prismjs@1.29.0/components/prism-rust.min.js"></script>
  <script src="{ASSET_PREFIX}js/prism-vpp.js"></script>
  <script src="{ASSET_PREFIX}js/main.js"></script>
{"".join(f'  <script src="{ASSET_PREFIX}js/{s}"></script>\n' for s in (extra_scripts or []))}</body>
</html>"""


def doc_href(href: str) -> str:
    if href.startswith("http") or href.startswith("/"):
        return href
    if "#" in href:
        file, frag = href.split("#", 1)
        return f"{ASSET_PREFIX}{file}#{frag}"
    return f"{ASSET_PREFIX}{href}"


def build_sidebar(groups: dict[str, list[tuple[str, str]]], active_href: str) -> str:
    parts = [
        '<div class="tree-search-wrap">',
        '<div class="tree-search-field">',
        TREE_SEARCH,
        '<input type="search" class="tree-search" placeholder="Go to page" aria-label="Filter navigation">',
        "</div></div>",
        '<nav class="sidebar-tree" aria-label="Documentation">',
    ]
    for group, links in groups.items():
        parts.append('<details class="tree-folder" open>')
        parts.append(
            f'<summary class="tree-folder-label">{TREE_CHEVRON}{TREE_FOLDER}'
            f"{html.escape(group)}</summary>"
        )
        parts.append('<ul class="tree-list">')
        for href, label in links:
            parts.append(
                f'<li><a href="{doc_href(href)}" class="tree-link">'
                f"{TREE_FILE}{html.escape(label)}</a></li>"
            )
        parts.append("</ul></details>")
    parts.append("</nav>")
    return "\n".join(parts)


def build_toc(headings: list[tuple[str, str, str]]) -> str:
    if not headings:
        return f'<ul><li><a href="#top" class="tree-link">{TREE_FILE}Top</a></li></ul>'
    parts = ["<ul>"]
    for level, hid, title in headings[:120]:
        if level == "4":
            cls = "toc-h4"
        elif level == "3":
            cls = "toc-h3"
        elif level == "2":
            cls = "toc-h2"
        else:
            cls = ""
        title_clean = html.unescape(title)
        parts.append(
            f'<li class="{cls}"><a href="#{hid}" class="tree-link">{TREE_FILE}{title_clean}</a></li>'
        )
    parts.append("</ul>")
    return "\n".join(parts)


def write_doc_page(
    filename: str,
    active: str,
    title: str,
    md_paths: list[Path],
    sidebar: dict[str, list[tuple[str, str]]],
    sidebar_active: str,
    desc: str,
    use_sources: bool = False,
    page_h1: str | None = None,
    strip_h1_from: set[str] | None = None,
) -> None:
    slug_registry = SlugRegistry()
    body = render_paths(
        md_paths,
        use_sources=use_sources,
        strip_h1_from=strip_h1_from,
        slugs=slug_registry,
    )
    if page_h1:
        body = (
            f'<div id="top"></div>\n'
            f'<h1 id="{slug_registry.unique(page_h1)}">{html.escape(clean_prose(page_h1))}</h1>\n{body}'
        )
    else:
        body = f'<div id="top"></div>\n' + body
    headings = headings_from_html(body)
    toc = build_toc(headings)
    page = shell(active, title, body, build_sidebar(sidebar, sidebar_active), toc, desc)
    (WEBSITE / filename).write_text(page, encoding="utf-8")
    lines = page.count("\n")
    print(f"Wrote {filename}: {lines} lines")


def courses_hub_shell(
    active: str,
    title: str,
    body: str,
    desc: str = "",
    extra_scripts: list[str] | None = None,
) -> str:
    nav_items = "\n".join(
        f'<a href="{ASSET_PREFIX}{href}" class="nav-link{" active" if href == active else ""}">{label}</a>'
        for href, label in NAV
    )
    meta = f'<meta name="description" content="{html.escape(clean_prose(desc))}">' if desc else ""
    scripts = "".join(
        f'  <script src="{ASSET_PREFIX}js/{s}"></script>\n' for s in (extra_scripts or [])
    )
    return f"""<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{html.escape(clean_prose(title))} | {BRAND}</title>
  {meta}
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?family=IBM+Plex+Sans:wght@400;500;600;700&family=IBM+Plex+Mono:wght@400;500&family=Shrikhand&display=swap" rel="stylesheet">
  <link rel="stylesheet" href="{ASSET_PREFIX}css/style.css">
  <link rel="stylesheet" href="{ASSET_PREFIX}css/prism-vpp.css">
  <link rel="icon" href="{ASSET_PREFIX}assets/favicon.png">
  <link rel="apple-touch-icon" href="{ASSET_PREFIX}assets/logo-header.png">
</head>
<body class="page-courses-hub">
  <header class="site-header">
    <div class="header-inner">
      <a href="{ASSET_PREFIX}index.html" class="brand"><img src="{ASSET_PREFIX}assets/logo-header.png" alt="{BRAND}" class="brand-logo"></a>
      <nav class="top-nav">{nav_items}</nav>
      <div class="header-actions">
        <a href="https://github.com/{GITHUB_REPO}" class="icon-btn" aria-label="GitHub" target="_blank" rel="noopener">
          {GITHUB_SVG}
        </a>
      </div>
      <button class="nav-toggle" aria-label="Menu">☰</button>
    </div>
  </header>
  <main class="courses-hub-main">{body}</main>
  <script src="https://cdn.jsdelivr.net/npm/prismjs@1.29.0/prism.min.js"></script>
  <script src="https://cdn.jsdelivr.net/npm/prismjs@1.29.0/components/prism-clike.min.js"></script>
  <script src="https://cdn.jsdelivr.net/npm/prismjs@1.29.0/components/prism-javascript.min.js"></script>
  <script src="https://cdn.jsdelivr.net/npm/prismjs@1.29.0/components/prism-bash.min.js"></script>
  <script src="{ASSET_PREFIX}js/prism-vpp.js"></script>
  <script src="{ASSET_PREFIX}js/main.js"></script>
{scripts}</body>
</html>"""


def build_course_card(project: CourseProject, card_id: str = "") -> str:
    id_attr = f' id="{html.escape(card_id)}"' if card_id else ""
    return f"""<a href="{ASSET_PREFIX}{project.page_name}" class="course-card"{id_attr} data-level="{html.escape(project.level_key)}">
  <div class="course-card-thumb" aria-hidden="true">
    <div class="course-card-grid"></div>
    <div class="course-card-cover">
      <img src="{ASSET_PREFIX}assets/logo-white.png" alt="" class="course-card-logo">
      <span class="course-card-thumb-label">{html.escape(project.title)}</span>
    </div>
  </div>
  <div class="course-card-body">
    <h2 class="course-card-title">{html.escape(project.title)}</h2>
    <p class="course-card-summary">{html.escape(clean_prose(project.summary))}</p>
    <div class="course-card-meta">
      <span class="course-card-avatar" aria-hidden="true">V++</span>
      <div>
        <span class="course-card-author">Course Curriculum</span>
        <span class="course-card-date">{html.escape(project.published_label)}</span>
      </div>
    </div>
  </div>
</a>"""


def build_courses_hub_cards(projects: list[CourseProject]) -> str:
    seen_levels: set[str] = set()
    cards: list[str] = []
    for project in projects:
        anchor_id = ""
        if project.level_key not in seen_levels:
            anchor_id = f"courses-{project.level_key}"
            seen_levels.add(project.level_key)
        cards.append(build_course_card(project, anchor_id))
    return "\n".join(cards)


def build_courses_hub_toc(projects: list[CourseProject]) -> str:
    parts = [
        "<ul>",
        f'<li><a href="#top" class="tree-link">{TREE_FILE}Top</a></li>',
        f'<li class="toc-h2"><a href="#courses-overview" class="tree-link">{TREE_FILE}Overview</a></li>',
        f'<li class="toc-h2"><a href="#courses-filter" class="tree-link">{TREE_FILE}Browse by level</a></li>',
    ]
    level_labels = [
        ("beginner", "Beginner"),
        ("intermediate", "Intermediate"),
        ("advanced", "Advanced"),
    ]
    for level_key, label in level_labels:
        level_projects = [p for p in projects if p.level_key == level_key]
        if not level_projects:
            continue
        parts.append(
            f'<li class="toc-h2"><a href="#courses-{level_key}" class="tree-link">'
            f"{TREE_FILE}{html.escape(label)}</a></li>"
        )
        for project in level_projects:
            parts.append(
                f'<li class="toc-h3"><a href="{ASSET_PREFIX}{project.page_name}" class="tree-link">'
                f"{TREE_FILE}{project.num:02d}. {html.escape(project.title)}</a></li>"
            )
    parts.append("</ul>")
    return "\n".join(parts)


def build_courses_hub_body(projects: list[CourseProject]) -> str:
    cards = build_courses_hub_cards(projects)
    return f"""<div class="courses-hub-inner" id="top">
  <header class="courses-hub-header" id="courses-overview">
    <h1 id="courses-title">Courses &amp; Projects</h1>
    <p>Twenty guided builds from first print to JSON configs. Open a project for step by step theory, incremental code, and a runnable playground.</p>
  </header>
  <nav class="courses-filter" id="courses-filter" aria-label="Filter projects">
    <button type="button" class="courses-filter-btn active" data-filter="all">Everything</button>
    <button type="button" class="courses-filter-btn" data-filter="beginner">Beginner</button>
    <button type="button" class="courses-filter-btn" data-filter="intermediate">Intermediate</button>
    <button type="button" class="courses-filter-btn" data-filter="advanced">Advanced</button>
  </nav>
  <div class="course-grid">{cards}</div>
</div>"""


def build_course_page_body(project: CourseProject) -> str:
    parts = [
        f'<div id="top"></div>',
        f'<nav class="course-breadcrumb"><a href="{ASSET_PREFIX}courses.html">Courses</a>'
        f' <span aria-hidden="true">/</span> '
        f'<span>Project {project.num:02d}</span></nav>',
        f'<header class="course-header">',
        f'<h1 id="course-title">{html.escape(project.title)}</h1>',
        f'<p class="course-lead">{html.escape(clean_prose(project.summary))}</p>',
        "</header>",
    ]

    for idx, section in enumerate(project.sections, start=1):
        parts.append(f'<section class="course-step" id="{html.escape(section.section_id)}">')
        parts.append(f'<div class="course-step-head">')
        parts.append(f'<span class="course-step-num">{idx}</span>')
        parts.append(
            f'<h2 id="{html.escape(section.section_id)}-title">{html.escape(section.title)}</h2>'
        )
        parts.append("</div>")
        parts.append('<div class="course-step-body">')
        for para in section.paragraphs:
            para = para.strip()
            if para:
                parts.append(f"<p>{course_inline_md(para)}</p>")
        if section.list_items:
            parts.append("<ol class=\"course-step-list\">")
            for item in section.list_items:
                parts.append(f"<li>{course_inline_md(item)}</li>")
            parts.append("</ol>")
        if section.section_id == "expected-behavior" and project.output.strip():
            parts.append(
                f'<pre class="course-expected-output">{html.escape(project.output)}</pre>'
            )
        if section.code.strip():
            parts.append(code_block_html(section.code))
        parts.append("</div></section>")

    finish_num = len(project.sections) + 1
    output_json = json_for_script(
        {
            "output": project.output,
            "source": project.source,
            "run_cmd": project.run_cmd,
        }
    )
    parts.append(
        f'<section class="course-finish" id="course-finish">'
        f'<div class="course-step-head course-finish-head">'
        f'<span class="course-step-num">{finish_num}</span>'
        f'<h2 class="course-finish-title" id="course-finish-title">Full program</h2>'
        f"</div>"
        f'<div class="course-source-editor">'
        f'{code_block_html(project.source, wrap_class="course-source-wrap", code_class="course-source-code")}'
        f"</div>"
        f'<script type="application/json" id="course-playground-data">{output_json}</script>'
        f'<div class="course-playground">'
        f'<div class="course-playground-toolbar">'
        f'<button type="button" class="btn btn-primary course-run-btn">Test program</button>'
        f'<button type="button" class="btn btn-outline course-reset-btn">Reset</button>'
        f"</div>"
        f'<div class="course-terminal" aria-live="polite">'
        f'<div class="course-terminal-header">'
        f'<span class="course-terminal-dot"></span>'
        f'<span class="course-terminal-dot"></span>'
        f'<span class="course-terminal-dot"></span>'
        f'<span class="course-terminal-title">vpp</span>'
        f"</div>"
        f'<div class="course-terminal-body">'
        f'<div class="course-terminal-line course-terminal-muted">$ ready. Click Test program.</div>'
        f'<pre class="course-run-output"></pre>'
        f"</div></div></div></section>"
    )
    return "\n".join(parts)


def build_course_sidebar(projects: list[CourseProject], active_page: str) -> str:
    parts = [
        '<div class="tree-search-wrap">',
        '<div class="tree-search-field">',
        TREE_SEARCH,
        '<input type="search" class="tree-search" placeholder="Find project" aria-label="Filter projects">',
        "</div></div>",
        '<nav class="sidebar-tree" aria-label="Courses">',
        '<details class="tree-folder" open>',
        f'<summary class="tree-folder-label">{TREE_CHEVRON}{TREE_FOLDER}Projects</summary>',
        '<ul class="tree-list">',
        f'<li><a href="{ASSET_PREFIX}courses.html" class="tree-link">{TREE_FILE}All projects</a></li>',
    ]
    for p in projects:
        parts.append(
            f'<li><a href="{ASSET_PREFIX}{p.page_name}" class="tree-link">'
            f"{TREE_FILE}{p.num:02d}. {html.escape(p.title)}</a></li>"
        )
    parts.extend(["</ul></details></nav>"])
    return "\n".join(parts)


def write_course_pages(projects: list[CourseProject], sidebar: dict[str, list[tuple[str, str]]]) -> None:
    course_sidebar = build_course_sidebar(projects, "")
    hub_body = build_courses_hub_body(projects)
    hub_toc = build_courses_hub_toc(projects)
    hub_page = shell(
        "courses.html",
        "Courses",
        hub_body,
        course_sidebar,
        hub_toc,
        f"Twenty guided {BRAND} projects with step by step lessons and runnable code.",
        extra_scripts=["courses.js"],
        body_class="page-docs page-courses-hub",
    )
    (WEBSITE / "courses.html").write_text(hub_page, encoding="utf-8")
    print(f"Wrote courses.html: {hub_page.count(chr(10))} lines")

    for project in projects:
        body = build_course_page_body(project)
        headings = headings_from_html(body)
        toc = build_toc(headings)
        page = shell(
            "courses.html",
            f"Project {project.num:02d}: {project.title}",
            body,
            course_sidebar,
            toc,
            f"Step by step {BRAND} course: {project.title}.",
            extra_scripts=["course-runner.js"],
            body_class="page-docs page-course",
        )
        (WEBSITE / project.page_name).write_text(page, encoding="utf-8")
        print(f"Wrote {project.page_name}: {page.count(chr(10))} lines")


def docs_section_id(section: DocsSection) -> str:
    return section.slug


def render_docs_block(block: DocsBlock, section: DocsSection) -> str:
    if block.kind == "subheading":
        hid = slug(f"{section.title}-{block.text}")
        return f'<h3 id="{hid}">{html.escape(clean_prose(block.text))}</h3>'
    if block.kind == "paragraph":
        return f"<p>{course_inline_md(block.text)}</p>"
    if block.kind == "labeled":
        return (
            f"<p><strong>{html.escape(clean_prose(block.label))}:</strong> "
            f"{course_inline_md(block.text)}</p>"
        )
    if block.kind == "code":
        return code_block_html(block.text)
    if block.kind == "version":
        parts = [
            f"<p><strong>{html.escape(block.version)}</strong> "
            f"{html.escape(clean_prose(block.title))}</p>"
        ]
        if block.text:
            parts.append(f"<p>{course_inline_md(block.text)}</p>")
        return "\n".join(parts)
    if block.kind == "faq":
        qid = slug(block.question)
        return (
            f'<details class="faq-item" id="{qid}">'
            f'<summary class="faq-question">{FAQ_CHEVRON}'
            f'<span class="faq-question-text">{course_inline_md(block.question)}</span></summary>'
            f'<div class="faq-answer"><p>{course_inline_md(block.answer)}</p></div>'
            f"</details>"
        )
    if block.kind == "project_table":
        rows_html = "".join(
            f"<tr><td>{html.escape(row.num)}</td>"
            f"<td>{html.escape(row.project)}</td>"
            f"<td>{course_inline_md(row.teaches)}</td></tr>"
            for row in block.rows
        )
        return (
            '<div class="table-wrap"><table>'
            "<thead><tr><th>#</th><th>Project</th><th>Teaches</th></tr></thead>"
            f"<tbody>{rows_html}</tbody></table></div>"
        )
    return ""


def build_docs_page_body(doc: DocsDocument) -> str:
    parts = [
        '<div id="top"></div>',
        f'<h1 id="documentation">{html.escape(clean_prose(doc.title))}</h1>',
    ]
    if doc.subtitle:
        parts.append(f"<p>{course_inline_md(doc.subtitle)}</p>")
    if doc.meta_line:
        parts.append(f"<p>{course_inline_md(doc.meta_line)}</p>")

    for section in doc.sections:
        sid = docs_section_id(section)
        parts.append(
            f'<h2 id="{sid}">{section.num}. {html.escape(clean_prose(section.title))}</h2>'
        )
        if section.num == 24:
            parts.append('<div class="faq-list">')
        for block in section.blocks:
            rendered = render_docs_block(block, section)
            if rendered:
                parts.append(rendered)
        if section.num == 24:
            parts.append("</div>")

    return "\n".join(parts)


def write_docs_page(
    filename: str,
    active: str,
    title: str,
    doc: DocsDocument,
    sidebar: dict[str, list[tuple[str, str]]],
    sidebar_active: str,
    desc: str,
) -> None:
    body = build_docs_page_body(doc)
    headings = headings_from_html(body)
    toc = build_toc(headings)
    page = shell(active, title, body, build_sidebar(sidebar, sidebar_active), toc, desc)
    (WEBSITE / filename).write_text(page, encoding="utf-8")
    print(f"Wrote {filename}: {page.count(chr(10))} lines")


def all_doc_links(course_projects: list[CourseProject] | None = None) -> dict[str, list[tuple[str, str]]]:
    getting = [
        ("learn.html#introduction", "Introduction"),
        ("learn.html#install", "Install"),
        ("learn.html#first-program", "First program"),
        ("learn.html#first-project", "First project"),
        ("courses.html", "Courses & projects"),
    ]
    language = [
        ("docs.html#what-is-vpp", "What is v++"),
        ("docs.html#language-reference", "Language reference"),
        ("docs.html#control-flow", "Control flow"),
        ("docs.html#generics", "Generics"),
        ("docs.html#traits-and-impl", "Traits & impl"),
        ("docs.html#modules-and-packages", "Modules & packages"),
        ("docs.html#standard-library", "Standard library"),
        ("docs.html#builtins", "Builtins"),
    ]
    guides = [
        ("docs.html#quick-start", "Quick start"),
        ("docs.html#cli", "CLI"),
        ("docs.html#compiler-architecture", "Compiler architecture"),
        ("docs.html#runtime-and-memory-model", "Runtime & memory"),
        ("docs.html#diagnostics", "Diagnostics"),
        ("docs.html#testing-and-interpreter-native-parity", "Testing & parity"),
        ("docs.html#faq", "FAQ"),
    ]
    project = [
        ("about.html", f"About {BRAND}"),
        ("about.html#architecture", "Architecture"),
        ("about.html#memory-model", "Memory model"),
        ("blog.html", "Release notes"),
        ("download.html", "Download"),
        ("contribute.html", "Contribute"),
    ]
    courses_links: list[tuple[str, str]] = [("courses.html", "All projects")]
    if course_projects:
        for p in course_projects:
            courses_links.append((p.page_name, f"{p.num:02d}. {p.title}"))
    return {
        "Getting started": getting,
        "Language": language,
        "Guides": guides,
        "Project": project,
        "Courses": courses_links,
    }


def main() -> None:
    course_projects = discover_course_projects()
    sidebar = all_doc_links(course_projects)

    learn_paths = [
        DOCS / "getting-started" / "introduction.md",
        DOCS / "getting-started" / "install.md",
        DOCS / "language" / "README.md",
        DOCS / "project" / "faq.md",
        DOCS / "getting-started" / "hello-world.md",
        DOCS / "getting-started" / "first-project.md",
        DOCS / "getting-started" / "vscode-setup.md",
    ]
    write_doc_page("learn.html", "learn.html", "Learn", learn_paths, sidebar, "learn.html#introduction",
                   f"Learn {BRAND}  -  installation, syntax, and your first programs.")

    docs_doc = load_docs()
    write_docs_page(
        "docs.html",
        "docs.html",
        "Documentation",
        docs_doc,
        sidebar,
        "docs.html",
        f"{BRAND} language reference, toolchain, compiler architecture, and documentation hub.",
    )

    about_paths = [ROOT / "README.md", ROOT / "ARCHITECTURE.md", ROOT / "MEMORY_MODEL.md",
                   ROOT / "SPEC.md", DOCS / "project" / "roadmap.md"]
    write_doc_page(
        "about.html", "about.html", "About The Language", about_paths, sidebar, "about.html",
        "About The Language  -  design, architecture, memory model, and roadmap.",
        page_h1="About The Language",
        strip_h1_from={"README.md"},
    )

    blog_paths = [ROOT / "CHANGELOG.md", DOCS / "project" / "roadmap.md"]
    write_doc_page("blog.html", "blog.html", "Blog", blog_paths, sidebar, "blog.html",
                   f"{BRAND} release notes and development blog.")

    contrib_paths = [ROOT / "CONTRIBUTING.md", DOCS / "contributing" / "building-from-source.md",
                     DOCS / "contributing" / "running-tests.md", ROOT / "SECURITY.md",
                     ROOT / "CODE_OF_CONDUCT.md"]
    write_doc_page("contribute.html", "contribute.html", "Contribute", contrib_paths, sidebar, "contribute.html",
                   f"Contribute to the {BRAND} compiler, docs, and ecosystem.")

    write_course_pages(course_projects, sidebar)

    write_doc_page(
        "license.html", "license.html", "License", [DOCS / "LICENSE.md"],
        sidebar, "license.html", f"MIT License for {BRAND}.",
        page_h1="License",
    )

    # Download page
    download_body = build_download_page_body()
    headings = headings_from_html(download_body)
    page = shell(
        "download.html",
        "Download",
        download_body,
        build_sidebar(sidebar, "download.html"),
        build_toc(headings),
        f"Download {BRAND} prebuilt binaries for Windows, Linux, and macOS.",
        extra_scripts=[f"download.js?v={LATEST_VERSION}"],
        body_class="page-docs page-download",
    )
    (WEBSITE / "download.html").write_text(page, encoding="utf-8")
    print(f"Wrote download.html: {page.count(chr(10))} lines")


if __name__ == "__main__":
    main()
