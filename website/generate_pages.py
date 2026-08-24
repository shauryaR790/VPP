#!/usr/bin/env python3
"""Generate minimal Wikipedia-style V++ site (docs + download only)."""

from __future__ import annotations

import html
import re
from pathlib import Path

from pdf_docs import DocsBlock, DocsDocument, DocsSection, load_docs

WEBSITE = Path(__file__).resolve().parent
ASSET_PREFIX = "/VPP/"
GITHUB = "https://github.com/shauryaR790/VPP"
RELEASES = f"{GITHUB}/releases"

# Generated pages only — remove everything else on build.
GENERATED_PAGES = {"index.html", "docs.html", "download.html"}

REMOVE_HTML: list[str | Path] = [
    "learn.html",
    "about.html",
    "blog.html",
    "contribute.html",
    "courses.html",
    "license.html",
    "course-*.html",
]


def slug(text: str) -> str:
    normalized = text.replace("v++", "vpp").replace("V++", "Vpp")
    return re.sub(r"[^a-z0-9]+", "-", normalized.lower()).strip("-")


def inline_md(text: str) -> str:
    escaped = html.escape(text)
    escaped = re.sub(
        r"`([^`]+)`",
        r'<code>\1</code>',
        escaped,
    )
    escaped = re.sub(
        r"\[([^\]]+)\]\(([^)]+)\)",
        r'<a href="\2">\1</a>',
        escaped,
    )
    return escaped


def render_code(text: str) -> str:
    return f"<pre><code>{html.escape(text)}</code></pre>"


def render_block(block: DocsBlock, section: DocsSection) -> str:
    if block.kind == "subheading":
        hid = slug(f"{section.title}-{block.text}")
        return f"<h3 id=\"{hid}\">{html.escape(block.text)}</h3>"
    if block.kind == "paragraph":
        return f"<p>{inline_md(block.text)}</p>"
    if block.kind == "labeled":
        return (
            f"<p><strong>{html.escape(block.label)}</strong> "
            f"{inline_md(block.text)}</p>"
        )
    if block.kind == "code":
        return render_code(block.text)
    if block.kind == "version":
        parts = [f"<p><strong>{html.escape(block.version)}</strong> {html.escape(block.title)}</p>"]
        if block.text:
            parts.append(f"<p>{inline_md(block.text)}</p>")
        return "\n".join(parts)
    if block.kind == "faq":
        qid = slug(block.question)
        return (
            f'<h3 id="{qid}">{inline_md(block.question)}</h3>'
            f"<p>{inline_md(block.answer)}</p>"
        )
    if block.kind == "project_table":
        rows = "".join(
            f"<tr><td>{html.escape(row.num)}</td>"
            f"<td>{html.escape(row.project)}</td>"
            f"<td>{inline_md(row.teaches)}</td></tr>"
            for row in block.rows
        )
        return (
            "<table><thead><tr><th>#</th><th>Project</th><th>Teaches</th></tr></thead>"
            f"<tbody>{rows}</tbody></table>"
        )
    return ""


def build_toc(sections: list[DocsSection]) -> str:
    items = "".join(
        f'<li><a href="#{docs_section_id(s)}">{html.escape(s.title)}</a></li>'
        for s in sections
    )
    return f'<nav class="wiki-sidebar"><h2>Contents</h2><ol>{items}</ol></nav>'


def docs_section_id(section: DocsSection) -> str:
    return section.slug


def build_docs_body(doc: DocsDocument) -> str:
    parts = [
        '<div class="infobox">',
        '<div class="infobox-title">V++</div>',
        "<table>",
        "<tr><th>Paradigm</th><td>Multi-paradigm</td></tr>",
        "<tr><th>Typing</th><td>Static, inferred</td></tr>",
        "<tr><th>Implementation</th><td>Rust + LLVM</td></tr>",
        f'<tr><th>Repository</th><td><a href="{GITHUB}">GitHub</a></td></tr>',
        f'<tr><th>License</th><td><a href="{GITHUB}/blob/main/LICENSE">MIT</a></td></tr>',
        "</table>",
        "</div>",
    ]
    if doc.meta_line:
        parts.append(f'<p class="wiki-subtitle">{html.escape(doc.meta_line)}</p>')

    for section in doc.sections:
        sid = docs_section_id(section)
        parts.append(f'<h2 id="{sid}">{section.num}. {html.escape(section.title)}</h2>')
        for block in section.blocks:
            rendered = render_block(block, section)
            if rendered:
                parts.append(rendered)

    return "\n".join(parts)


def page_shell(
    title: str,
    active: str,
    body: str,
    sidebar: str = "",
    description: str = "",
) -> str:
    nav = [
        ("index.html", "Main page"),
        ("docs.html", "Documentation"),
        ("download.html", "Download"),
    ]
    nav_html = "".join(
        f'<a href="{ASSET_PREFIX}{href}" class="{"active" if href == active else ""}">{label}</a>'
        for href, label in nav
    )
    layout = "page-wrap" if sidebar else "page-wrap no-sidebar"
    desc = f'<meta name="description" content="{html.escape(description)}">' if description else ""
    return f"""<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{html.escape(title)}</title>
  {desc}
  <link rel="stylesheet" href="{ASSET_PREFIX}css/style.css">
  <link rel="icon" href="{ASSET_PREFIX}assets/favicon.png">
</head>
<body>
  <header class="site-header">
    <div class="header-inner">
      <a href="{ASSET_PREFIX}index.html" class="brand">V++</a>
      <nav class="top-nav">{nav_html}</nav>
      <div class="header-actions"><a href="{GITHUB}">GitHub</a></div>
    </div>
  </header>
  <div class="{layout}">
    {sidebar}
    <main class="wiki-main">
      <article class="wiki-content">
        {body}
      </article>
    </main>
  </div>
  <footer class="site-footer">
    Text is available under the <a href="{GITHUB}/blob/main/LICENSE">MIT License</a>.
    Documentation from the <a href="{GITHUB}">V++ repository</a>.
  </footer>
  {"<script src=\"" + ASSET_PREFIX + "js/download.js\"></script>" if active == "download.html" else ""}
</body>
</html>
"""


def build_index_page() -> str:
    body = f"""
<h1 class="wiki-title">V++</h1>
<p class="wiki-subtitle">From the V++ project</p>
<p><strong>V++</strong> (vpp) is an open-source programming language and compiler.
It aims for readable syntax, static typing with local inference, and native compilation via LLVM.</p>
<p>The compiler is written in Rust. Programs can run through a tree-walking interpreter or compile to native executables.
The language includes structs, enums, generics, traits, <code>Option</code>/<code>Result</code>, modules, and v1.0.5 automation APIs (typed process I/O, env, directories, logging).</p>
<h2>Getting the compiler</h2>
<p>Prebuilt Windows releases are on GitHub. See the <a href="{ASSET_PREFIX}download.html">Download</a> page.</p>
<h2>Documentation</h2>
<p>Language reference, CLI, architecture, and FAQ: <a href="{ASSET_PREFIX}docs.html">Documentation</a>.</p>
<h2>External links</h2>
<ul>
  <li><a href="{GITHUB}">Source repository</a></li>
  <li><a href="{RELEASES}">Releases</a></li>
  <li><a href="https://marketplace.visualstudio.com/items?itemName=vpp-lang.vplusplus">VS Code extension</a></li>
</ul>
"""
    return page_shell("V++", "index.html", body, description="V++ programming language")


def build_docs_page(doc: DocsDocument) -> str:
    body = f'<h1 class="wiki-title">{html.escape(doc.title)}</h1>\n' + build_docs_body(doc)
    sidebar = build_toc(doc.sections)
    return page_shell(
        f"{doc.title} - V++",
        "docs.html",
        body,
        sidebar,
        "V++ language documentation",
    )


def build_download_page() -> str:
    versions = ["1.0.4", "1.0.3", "1.0.2", "0.7.0", "0.6.2", "0.5.0"]
    version_opts = "".join(
        f'<option value="{v}"{" selected" if v == "1.0.4" else ""}>v{v}</option>'
        for v in versions
    )
    body = f"""
<h1 class="wiki-title">Download V++</h1>
<p class="download-lead">Prebuilt binaries are published on <a href="{RELEASES}">GitHub Releases</a>.
Windows builds are the most complete; Linux and macOS bundles may also be attached to releases.</p>
<div class="download-form" id="download-hub">
  <label for="dl-version">Version</label>
  <select id="dl-version" aria-label="Version">{version_opts}</select>
  <label for="dl-os">Platform</label>
  <select id="dl-os" aria-label="Operating system">
    <option value="windows">Windows</option>
    <option value="linux">Linux</option>
    <option value="macos">macOS</option>
  </select>
  <label for="dl-format">Format</label>
  <select id="dl-format" aria-label="Package type"></select>
</div>
<p class="download-note" id="dl-detect"></p>
<div class="download-info" id="dl-info"></div>
<div class="download-actions">
  <a id="dl-primary" href="{RELEASES}/latest" target="_blank" rel="noopener">Download</a>
  <a id="dl-secondary" href="{RELEASES}/latest" target="_blank" rel="noopener" hidden>Alternate format</a>
</div>
<pre id="dl-code-wrap"><code id="dl-code"># Select a version and platform above.</code></pre>
<p class="download-note">Build from source: clone <a href="{GITHUB}">the repository</a> and run
<code>cargo build --release --features codegen,lsp</code>. See repository docs for LLVM requirements.</p>
<p class="download-note">VS Code: install the
<a href="https://marketplace.visualstudio.com/items?itemName=vpp-lang.vplusplus">v++ Language</a> extension.</p>
"""
    return page_shell("Download - V++", "download.html", body, description="Download V++ compiler")


def cleanup_old_pages() -> None:
    for entry in REMOVE_HTML:
        if isinstance(entry, Path):
            paths = [entry] if entry.is_file() else []
        elif "*" in str(entry):
            paths = list(WEBSITE.glob(str(entry)))
        else:
            paths = [WEBSITE / entry]
        for path in paths:
            if path.is_file() and path.name not in GENERATED_PAGES:
                path.unlink()
                print(f"Removed {path.name}")


def main() -> None:
    doc = load_docs()
    pages = {
        "index.html": build_index_page(),
        "docs.html": build_docs_page(doc),
        "download.html": build_download_page(),
    }
    for name, content in pages.items():
        (WEBSITE / name).write_text(content, encoding="utf-8")
        print(f"Wrote {name}")
    cleanup_old_pages()


if __name__ == "__main__":
    main()
