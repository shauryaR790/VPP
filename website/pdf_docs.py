"""Parse vpp_docs_page_master_content.pdf into structured docs page content."""

from __future__ import annotations

import json
import re
from dataclasses import asdict, dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
WEBSITE = Path(__file__).resolve().parent
PDF_CANDIDATES = (
    ROOT / "vpp_docs_page_master_content.pdf",
)
JSON_PATH = WEBSITE / "docs.json"

HEADER_RE = re.compile(r"v\+\+ Documentation Master Content\s+\d+\s*", re.I)
SECTION_SPLIT_RE = re.compile(r"(?=^\d+\.\s+)", re.M)
SECTION_HEAD_RE = re.compile(r"^(\d+)\.\s+(.+)$", re.M)

CODE_STARTERS = (
    "let ",
    "fn ",
    "struct ",
    "enum ",
    "trait ",
    "impl ",
    "import ",
    "print(",
    "return ",
    "if ",
    "while ",
    "for ",
    "match ",
    "vpp ",
    "git ",
    "cargo ",
    "cd ",
    ".\\",
    "Active",
    "Inactive",
    "Ok(",
    "Some(",
    "user.",
    "total ",
    ".vpp ",
    "len(",
    "assert",
    "read_file",
    "write_file",
    "file_exists",
    "json_parse",
    "json_stringify",
    "process_run",
    "count ",
    "score ",
)


def _looks_like_field_line(line: str) -> bool:
    return bool(re.match(r"^[A-Za-z_]\w*:\s+\S+", line.strip()))


def _looks_like_code_fragment(line: str) -> bool:
    stripped = line.strip()
    if not stripped:
        return False
    if stripped.startswith("}"):
        return True
    if "=>" in stripped:
        return True
    if stripped.startswith("else"):
        return True
    if re.match(r"^\w+\s*=", stripped):
        return True
    return False


@dataclass
class ProjectTableRow:
    num: str
    project: str
    teaches: str


@dataclass
class FaqItem:
    question: str
    answer: str


@dataclass
class VersionEntry:
    version: str
    title: str
    description: str


@dataclass
class DocsBlock:
    kind: str
    text: str = ""
    label: str = ""
    rows: list[ProjectTableRow] = field(default_factory=list)
    question: str = ""
    answer: str = ""
    version: str = ""
    title: str = ""


@dataclass
class DocsSection:
    num: int
    title: str
    slug: str
    blocks: list[DocsBlock]


@dataclass
class DocsDocument:
    title: str
    subtitle: str
    meta_line: str
    sections: list[DocsSection]


def _find_pdf() -> Path | None:
    for path in PDF_CANDIDATES:
        if path.exists():
            return path
    return None


def _slugify(title: str) -> str:
    normalized = title.replace("v++", "vpp").replace("V++", "Vpp")
    return re.sub(r"[^a-z0-9]+", "-", normalized.lower()).strip("-")


def _pdf_text() -> str:
    from pypdf import PdfReader

    pdf_path = _find_pdf()
    if pdf_path is None:
        raise FileNotFoundError("No docs PDF found (vpp_docs_page_master_content.pdf).")
    reader = PdfReader(str(pdf_path))
    raw = "\n".join((page.extract_text() or "") for page in reader.pages)
    raw = HEADER_RE.sub("", raw)
    raw = re.sub(r"\n{3,}", "\n\n", raw)
    return raw.strip()


def _is_code_line(line: str) -> bool:
    stripped = line.strip()
    if not stripped:
        return False
    if stripped in ("{", "}", "|", "v", "Lexer", "Parser -> AST", "Module loader", "Type checker -> TypedProgram"):
        return True
    if stripped.startswith("|"):
        return True
    if re.match(r"^v\d+\.\d+", stripped):
        return False
    if _looks_like_field_line(stripped):
        return True
    if _looks_like_code_fragment(stripped):
        return True
    return stripped.startswith(CODE_STARTERS)


def _extract_code_block(lines: list[str], start: int) -> tuple[str, int]:
    collected: list[str] = []
    i = start
    in_string = False
    quote_char = ""

    def update_string_state(text: str) -> None:
        nonlocal in_string, quote_char
        for ch in text:
            if in_string:
                if ch == quote_char:
                    in_string = False
                    quote_char = ""
            elif ch in ('"', "'"):
                in_string = True
                quote_char = ch

    while i < len(lines):
        line = lines[i]
        stripped = line.strip()
        if not stripped:
            if collected:
                if in_string:
                    collected.append("")
                    i += 1
                    continue
                break
            i += 1
            continue

        if collected:
            continuing_string = in_string
            update_string_state(line)
            if (
                continuing_string
                or in_string
                or _is_code_line(line)
                or _looks_like_field_line(line)
                or _looks_like_code_fragment(line)
                or line.startswith(("    ", "\t"))
                or stripped in ("{", "}", "|")
                or stripped.startswith("|")
                or (collected and stripped.endswith("-> AST"))
            ):
                collected.append(line.rstrip())
                i += 1
                continue
            break

        if _is_code_line(line) or stripped.endswith("-> AST") or stripped == ".vpp source":
            collected.append(line.rstrip())
            update_string_state(line)
            i += 1
            continue

        i += 1

    return "\n".join(collected).strip(), i


def _merge_paragraph(previous: str, nxt: str) -> bool:
    if not previous or not nxt:
        return False
    if nxt.endswith("?"):
        return False
    if re.match(r"^v0\.\d", nxt):
        return False
    if re.match(r"^\d+\.\s", nxt):
        return False
    if previous.endswith((".", "!", "?", ":", ";")):
        return False
    return True


def _is_subheading(line: str, nxt: str) -> bool:
    if not line or line.endswith("?"):
        return False
    if _is_code_line(line) or _looks_like_code_fragment(line):
        return False
    if re.match(r"^v0\.\d", line):
        return False
    if re.match(r"^\d+\.\s", line):
        return False
    if line in ("#", "Project", "Teaches"):
        return False
    if len(line) > 80:
        return False
    if ":" in line and len(line.split(":", 1)[0]) < 40:
        return False
    if line.endswith("."):
        return False
    if "{" in line or "}" in line or "=>" in line:
        return False
    if not nxt:
        return True
    return _is_code_line(nxt) or nxt.startswith("The ") or nxt.startswith("When ") or nxt.startswith("Both.")


def _parse_project_table(lines: list[str], start: int) -> tuple[list[ProjectTableRow], int]:
    rows: list[ProjectTableRow] = []
    i = start
    while i < len(lines) and lines[i].strip() in ("#", "Project", "Teaches"):
        i += 1
    while i + 2 < len(lines):
        num = lines[i].strip()
        if not re.fullmatch(r"\d{2}", num):
            break
        project = lines[i + 1].strip()
        teaches = lines[i + 2].strip()
        if not project:
            break
        rows.append(ProjectTableRow(num=num, project=project, teaches=teaches))
        i += 3
    return rows, i


def _parse_version_entry(line: str) -> VersionEntry | None:
    match = re.match(r"^(v0\.\d+\.\d+)\s+[– - \-]\s*(.+)$", line)
    if not match:
        match = re.match(r"^(v0\.\d+\.\d+)\s+(.+)$", line)
    if not match:
        return None
    version = match.group(1)
    rest = match.group(2).strip()
    return VersionEntry(version=version, title=rest, description="")


def _parse_design_goals(body: str) -> list[DocsBlock]:
    labels = (
        "Readable by default.",
        "Statically typed.",
        "Native compilation.",
        "Two execution paths.",
        "Tooling included.",
    )
    pattern = "(" + "|".join(re.escape(label) for label in labels) + ")"
    parts = re.split(pattern, body.replace("\n", " ").strip())
    blocks: list[DocsBlock] = []
    i = 1
    while i + 1 < len(parts):
        label = parts[i].strip().rstrip(".")
        text = parts[i + 1].strip()
        blocks.append(DocsBlock(kind="labeled", label=label, text=text))
        i += 2
    if not blocks:
        blocks.append(DocsBlock(kind="paragraph", text=body.replace("\n", " ").strip()))
    return blocks


def _parse_section_body(section_num: int, body: str) -> list[DocsBlock]:
    if section_num == 2:
        return _parse_design_goals(body)

    lines = [line.rstrip() for line in body.splitlines()]
    blocks: list[DocsBlock] = []
    i = 0
    pending_para: list[str] = []

    def flush_para() -> None:
        nonlocal pending_para
        if pending_para:
            blocks.append(DocsBlock(kind="paragraph", text=" ".join(pending_para).strip()))
            pending_para = []

    if section_num == 21:
        rows, end = _parse_project_table(lines, 0)
        if rows:
            blocks.append(DocsBlock(kind="project_table", rows=rows))
            i = end

    if section_num == 24:
        while i < len(lines):
            line = lines[i].strip()
            if not line:
                i += 1
                continue
            if line.endswith("?"):
                question = line
                answer_parts: list[str] = []
                i += 1
                while i < len(lines):
                    nxt = lines[i].strip()
                    if not nxt:
                        i += 1
                        break
                    if nxt.endswith("?"):
                        break
                    answer_parts.append(nxt)
                    i += 1
                blocks.append(
                    DocsBlock(kind="faq", question=question, answer=" ".join(answer_parts).strip())
                )
                continue
            pending_para.append(line)
            i += 1
        flush_para()
        return blocks

    while i < len(lines):
        line = lines[i].strip()
        if not line:
            flush_para()
            i += 1
            continue

        if section_num == 16:
            entry = _parse_version_entry(line)
            if entry:
                flush_para()
                desc_parts: list[str] = []
                i += 1
                while i < len(lines):
                    nxt = lines[i].strip()
                    if not nxt:
                        i += 1
                        break
                    nxt_entry = _parse_version_entry(nxt)
                    if nxt_entry:
                        break
                    desc_parts.append(nxt)
                    i += 1
                entry.description = " ".join(desc_parts).strip()
                blocks.append(
                    DocsBlock(
                        kind="version",
                        version=entry.version,
                        title=entry.title,
                        text=entry.description,
                    )
                )
                continue

        if section_num == 18 and re.match(r"^v0\.\d", line):
            flush_para()
            match = re.match(r"^(v0\.\d+)\s+[– - \-]\s*(.+)$", line)
            if match:
                title = match.group(2).strip()
                version = match.group(1)
            else:
                parts = line.split(None, 1)
                version = parts[0]
                title = parts[1] if len(parts) > 1 else ""
            desc_parts: list[str] = []
            i += 1
            while i < len(lines):
                nxt = lines[i].strip()
                if not nxt:
                    i += 1
                    break
                if re.match(r"^v0\.\d", nxt) or nxt.startswith("The definition"):
                    break
                desc_parts.append(nxt)
                i += 1
            blocks.append(
                DocsBlock(
                    kind="version",
                    version=version,
                    title=title,
                    text=" ".join(desc_parts).strip(),
                )
            )
            if i < len(lines) and lines[i].strip().startswith("The definition"):
                pending_para.append(lines[i].strip())
                i += 1
            continue

        nxt = lines[i + 1].strip() if i + 1 < len(lines) else ""
        if _is_subheading(line, nxt):
            flush_para()
            blocks.append(DocsBlock(kind="subheading", text=line))
            i += 1
            continue

        label_match = re.match(r"^([A-Z][A-Za-z /]+):\s*(.+)$", line)
        if label_match and section_num in (2, 19, 20, 26):
            flush_para()
            blocks.append(
                DocsBlock(
                    kind="labeled",
                    label=label_match.group(1).strip(),
                    text=label_match.group(2).strip(),
                )
            )
            i += 1
            continue

        if _is_code_line(line) or line == ".vpp source":
            flush_para()
            code, i = _extract_code_block(lines, i)
            if code:
                blocks.append(DocsBlock(kind="code", text=code))
            continue

        if pending_para and _merge_paragraph(pending_para[-1], line):
            pending_para[-1] = pending_para[-1] + " " + line
        else:
            pending_para.append(line)
        i += 1

    flush_para()
    return blocks


def _parse_preamble(text: str) -> tuple[str, str, str]:
    match = SECTION_HEAD_RE.search(text)
    preamble = text[: match.start()].strip() if match else text.strip()
    lines = [line.strip() for line in preamble.splitlines() if line.strip()]
    title = "v++ Documentation Master Content"
    subtitle = ""
    meta_line = ""
    if lines:
        if lines[0].lower().startswith("v++"):
            title = lines[0]
            lines = lines[1:]
        if lines and "complete source content" in lines[0].lower():
            subtitle = lines[0]
            lines = lines[1:]
        if lines and "current release" in lines[0].lower():
            meta_line = lines[0]
            lines = lines[1:]
    return title, subtitle, meta_line


def load_docs_from_pdf() -> DocsDocument:
    text = _pdf_text()
    title, subtitle, meta_line = _parse_preamble(text)
    sections: list[DocsSection] = []
    chunks = SECTION_SPLIT_RE.split(text)
    for chunk in chunks:
        chunk = chunk.strip()
        head = SECTION_HEAD_RE.match(chunk)
        if not head:
            continue
        num = int(head.group(1))
        section_title = head.group(2).strip()
        body = chunk[head.end() :].strip()
        sections.append(
            DocsSection(
                num=num,
                title=section_title,
                slug=_slugify(section_title),
                blocks=_parse_section_body(num, body),
            )
        )
    sections.sort(key=lambda s: s.num)
    return DocsDocument(title=title, subtitle=subtitle, meta_line=meta_line, sections=sections)


def _block_to_dict(block: DocsBlock) -> dict:
    data = {"kind": block.kind}
    if block.text:
        data["text"] = block.text
    if block.label:
        data["label"] = block.label
    if block.question:
        data["question"] = block.question
    if block.answer:
        data["answer"] = block.answer
    if block.version:
        data["version"] = block.version
    if block.title:
        data["title"] = block.title
    if block.rows:
        data["rows"] = [asdict(row) for row in block.rows]
    return data


def _block_from_dict(data: dict) -> DocsBlock:
    rows = [ProjectTableRow(**row) for row in data.get("rows", [])]
    return DocsBlock(
        kind=data["kind"],
        text=data.get("text", ""),
        label=data.get("label", ""),
        rows=rows,
        question=data.get("question", ""),
        answer=data.get("answer", ""),
        version=data.get("version", ""),
        title=data.get("title", ""),
    )


def export_docs_json(path: Path = JSON_PATH) -> None:
    doc = load_docs_from_pdf()
    payload = {
        "title": doc.title,
        "subtitle": doc.subtitle,
        "meta_line": doc.meta_line,
        "sections": [
            {
                "num": section.num,
                "title": section.title,
                "slug": section.slug,
                "blocks": [_block_to_dict(block) for block in section.blocks],
            }
            for section in doc.sections
        ],
    }
    path.write_text(json.dumps(payload, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def load_docs_from_json(path: Path = JSON_PATH) -> DocsDocument:
    raw = json.loads(path.read_text(encoding="utf-8"))
    sections = [
        DocsSection(
            num=item["num"],
            title=item["title"],
            slug=item["slug"],
            blocks=[_block_from_dict(block) for block in item["blocks"]],
        )
        for item in raw["sections"]
    ]
    return DocsDocument(
        title=raw["title"],
        subtitle=raw.get("subtitle", ""),
        meta_line=raw.get("meta_line", ""),
        sections=sections,
    )


def load_docs() -> DocsDocument:
    """Load docs page content. Prefer docs.json when present (PDF is for one-time import)."""
    if JSON_PATH.exists():
        return load_docs_from_json()
    if _find_pdf() is not None:
        doc = load_docs_from_pdf()
        export_docs_json()
        return doc
    raise FileNotFoundError(
        f"Missing docs data. Add {JSON_PATH.name} or vpp_docs_page_master_content.pdf."
    )


if __name__ == "__main__":
    export_docs_json()
    print(f"Wrote {JSON_PATH}")
