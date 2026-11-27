#!/usr/bin/env python3
"""Registers kept as one file per entry, so two pull requests that each add an entry never edit the same file.

The fudge register and the build rulings were each one long Markdown file whose tables every pull request
appended to, at the same table ends, so unrelated pull requests conflicted after going green (27 Nov 2026).
Each is now a folder:

    docs/fudges/                 the fudge register        (docs/fudges.md points here)
    handover/decisions/          rulings during the build  (handover/decisions.md points here)

A folder holds its parts in name order. A file is a piece of prose (headings and text). A sub-folder is one
table: each file in it is one row, written as the table's header, its separator and the row, so it reads as a
one-row table on its own. Existing rows are named by a number (0010-....md) or, for rulings, by date; a new row
is named after its id or a short slug (F-S37.md, cm13-wear-grid.md, 2026-11-27-shared-lists.md), which sorts
after the numbered rows and never collides with another pull request's row.

    python3 scripts/registers.py --print fudges        the whole register as one Markdown document
    python3 scripts/registers.py --check               every register is well formed (scripts/check.sh)
    python3 scripts/registers.py --absorb fudges       a branch from before the split: move the rows its old
                                                       one-file docs/fudges.md carries into the folder

The pointer files (docs/fudges.md, handover/decisions.md) are written by this script; --check fails if one
holds table rows again, which is what a branch from before the split looks like after merging main.
"""

import argparse
import hashlib
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

REGISTERS = {
    "fudges": {
        "dir": "docs/fudges",
        "pointer": "docs/fudges.md",
        "title": "Fudge register",
        # A fudge's id in the first column, where the table has one.
        "id": r"F-[A-Za-z0-9.\-]+",
    },
    "decisions": {
        "dir": "handover/decisions",
        "pointer": "handover/decisions.md",
        "title": "Rulings during the build",
        "id": None,
    },
}

POINTER = """# {title}

This register is kept as one file per entry in [`{dir}/`]({rel}/), so two pull requests that each add an entry never
edit the same file (27 Nov 2026). Read it whole on one page at https://v4.starsystemx.com/{name}.html (built from
main at every deploy), or locally with:

    python3 scripts/registers.py --print {name}

**To add an entry:** in the table's folder under `{dir}/`, copy any row file to a new file named after the entry's
id or a short slug (for example `{example}`), and edit its last line. Keep the header and separator lines as they
are. Prose and headings are the plain `.md` files beside the table folders. `scripts/check.sh` runs
`python3 scripts/registers.py --check`.

A branch from before the split that added rows to this file: take main's version of this file, put your copy aside,
and run `python3 scripts/registers.py --absorb {name} --from <your copy>`.

This file is written by `scripts/registers.py`; do not add rows here.
"""

EXAMPLES = {"fudges": "F-S37.md", "decisions": "2026-11-27-shared-lists.md"}


class RegisterError(Exception):
    pass


def path(*p):
    return os.path.join(ROOT, *p)


def is_row(line):
    return line.startswith("|")


def is_sep(line):
    return re.fullmatch(r"\|(\s*:?-{3,}:?\s*\|)+\s*", line) is not None


def cells(line):
    """A row's cells; a | inside a cell is written \\|."""
    body = line.strip()
    body = body[1:] if body.startswith("|") else body
    body = body[:-1] if body.endswith("|") and not body.endswith("\\|") else body
    return [c.strip() for c in re.split(r"(?<!\\)\|", body)]


def split_parts(text):
    """Split a Markdown document into ("text", str) and ("table", header, sep, [rows]) parts, losslessly."""
    lines = text.split("\n")
    parts = []
    buf = []
    i = 0
    while i < len(lines):
        if i + 1 < len(lines) and is_row(lines[i]) and is_sep(lines[i + 1]):
            if buf:
                parts.append(("text", "\n".join(buf) + "\n"))
                buf = []
            header, sep = lines[i], lines[i + 1]
            i += 2
            rows = []
            while i < len(lines) and is_row(lines[i]):
                rows.append(lines[i])
                i += 1
            parts.append(("table", header, sep, rows))
            continue
        buf.append(lines[i])
        i += 1
    # The document's last newline makes an empty last line; drop it so joining restores the text exactly.
    if buf and buf[-1] == "" and len(buf) >= 1:
        buf.pop()
        if buf:
            parts.append(("text", "\n".join(buf) + "\n"))
    elif buf:
        parts.append(("text", "\n".join(buf)))
    return parts


def join_parts(parts):
    out = []
    for p in parts:
        if p[0] == "text":
            out.append(p[1])
        else:
            out.append("\n".join([p[1], p[2]] + p[3]) + "\n")
    return "".join(out)


def slug(s, n=40):
    s = re.sub(r"[^a-z0-9]+", "-", s.lower()).strip("-")
    return s[:n].strip("-") or "part"


def headings_before(parts, k):
    """The nearest ## and ### headings above part k, as the table's address."""
    h2 = h3 = ""
    for p in parts[:k]:
        if p[0] != "text":
            continue
        for line in p[1].split("\n"):
            if line.startswith("## "):
                h2, h3 = line[3:].strip(), ""
            elif line.startswith("### "):
                h3 = line[4:].strip()
    return h2, h3


def iso_date(cell):
    """'26 Nov 2026' as '2026-11-26', so ruling files sort by date; None for anything else."""
    m = re.match(r"(\d{1,2}) ([A-Z][a-z]{2}) (\d{4})", cell)
    months = "Jan Feb Mar Apr May Jun Jul Aug Sep Oct Nov Dec".split()
    if m and m.group(2) in months:
        return f"{m.group(3)}-{months.index(m.group(2)) + 1:02d}-{int(m.group(1)):02d}"
    return None


def row_name(reg, index, row, date_names):
    """The file name of an existing row: its order number, then its id or date, so the order is kept."""
    first = cells(row)[0] if cells(row) else ""
    if date_names and iso_date(first):
        return f"{iso_date(first)}-{index:03d}.md"
    tail = ""
    if reg["id"] and re.fullmatch(reg["id"], first):
        tail = "-" + first
    return f"{index * 10:04d}{tail}.md"


def row_file_text(header, sep, row):
    return f"{header}\n{sep}\n{row}\n"


def explode(name, text):
    """Write a one-file register into its folder (used once, at the split, and by the tests)."""
    reg = REGISTERS[name]
    parts = split_parts(text)
    top = path(reg["dir"])
    os.makedirs(top, exist_ok=True)
    for k, p in enumerate(parts):
        h2, h3 = headings_before(parts + [("text", "")], k + 1) if p[0] == "text" else headings_before(parts, k)
        base = f"{(k + 1) * 10:04d}-{slug(' '.join(x for x in (h2, h3) if x) or reg['title'], 50)}"
        if p[0] == "text":
            with open(os.path.join(top, base + ".md"), "w", encoding="utf-8", newline="\n") as fh:
                fh.write(p[1])
        else:
            d = os.path.join(top, base)
            os.makedirs(d, exist_ok=True)
            for i, row in enumerate(p[3], 1):
                with open(os.path.join(d, row_name(reg, i, row, name == "decisions")), "w", encoding="utf-8", newline="\n") as fh:
                    fh.write(row_file_text(p[1], p[2], row))
    write_pointer(name)


def read_tree(name):
    """The register's parts from its folder, in name order, with each table's folder. Fails on a malformed row."""
    reg = REGISTERS[name]
    top = path(reg["dir"])
    if not os.path.isdir(top):
        raise RegisterError(f"registers: {reg['dir']}/ is missing")
    parts = []
    for entry in sorted(os.listdir(top)):
        full = os.path.join(top, entry)
        if os.path.isdir(full):
            header = sep = None
            rows = []
            files = sorted(f for f in os.listdir(full) if not f.startswith("."))
            if not files:
                raise RegisterError(f"registers: {reg['dir']}/{entry}/ is an empty table")
            for f in files:
                where = f"{reg['dir']}/{entry}/{f}"
                if not f.endswith(".md"):
                    raise RegisterError(f"registers: {where} is not a .md row file")
                with open(os.path.join(full, f), encoding="utf-8") as fh:
                    lines = fh.read().split("\n")
                if len(lines) != 4 or lines[3] != "" or not is_row(lines[0]) or not is_sep(lines[1]) or not is_row(lines[2]):
                    raise RegisterError(f"registers: {where} must be three lines: the table header, its separator, one row")
                if header is None:
                    header, sep = lines[0], lines[1]
                elif (lines[0], lines[1]) != (header, sep):
                    raise RegisterError(f"registers: {where} has a different header from the other rows of its table")
                rows.append((f, lines[2]))
            parts.append(("table", header, sep, [r for _, r in rows], entry, [f for f, _ in rows]))
        elif entry.endswith(".md"):
            with open(full, encoding="utf-8") as fh:
                t = fh.read()
            parts.append(("text", t, entry))
        else:
            raise RegisterError(f"registers: {reg['dir']}/{entry} is neither a .md part nor a table folder")
    return parts


def render(name):
    return join_parts([p[:4] if p[0] == "table" else p[:2] for p in read_tree(name)])


def pointer_text(name):
    reg = REGISTERS[name]
    rel = os.path.relpath(path(reg["dir"]), os.path.dirname(path(reg["pointer"]))).replace(os.sep, "/")
    return POINTER.format(title=reg["title"], dir=reg["dir"], rel=rel, name=name, example=EXAMPLES[name])


def write_pointer(name):
    with open(path(REGISTERS[name]["pointer"]), "w", encoding="utf-8", newline="\n") as fh:
        fh.write(pointer_text(name))


def check_one(name):
    reg = REGISTERS[name]
    parts = read_tree(name)
    with open(path(reg["pointer"]), encoding="utf-8") as fh:
        pointer = fh.read()
    if pointer != pointer_text(name):
        hint = ""
        if any(is_row(line) for line in pointer.split("\n")):
            hint = (
                f" It holds table rows: a branch from before the split. Take main's {reg['pointer']}, keep your copy"
                f" elsewhere, and run python3 scripts/registers.py --absorb {name} --from <your copy>."
            )
        raise RegisterError(f"registers: {reg['pointer']} is not the pointer page this script writes.{hint}")
    n = 0
    if reg["id"]:
        seen = {}
        for p in parts:
            if p[0] != "table":
                continue
            for f, row in zip(p[5], p[3]):
                n += 1
                first = cells(row)[0]
                if re.fullmatch(reg["id"], first):
                    if first in seen:
                        raise RegisterError(f"registers: id {first} is used twice ({seen[first]} and {p[4]}/{f}); take the next free number")
                    seen[first] = f"{p[4]}/{f}"
    else:
        n = sum(len(p[3]) for p in parts if p[0] == "table")
    return n


def check():
    try:
        counts = {name: check_one(name) for name in REGISTERS}
    except RegisterError as e:
        print(e, file=sys.stderr)
        sys.exit(1)
    print("registers: " + ", ".join(f"{name} {n} rows" for name, n in counts.items()) + ", well formed")


def new_row_name(reg, row, existing):
    first = cells(row)[0] if cells(row) else ""
    if reg["id"] and re.fullmatch(reg["id"], first):
        base = first
    elif reg["id"] is None and iso_date(first):
        base = iso_date(first) + "-" + slug(cells(row)[1] if len(cells(row)) > 1 else "", 40)
    else:
        base = slug(" ".join(cells(row)[:2]), 50) + "-" + hashlib.sha1(row.encode("utf-8")).hexdigest()[:6]
    name = base + ".md"
    k = 2
    while name in existing:
        name = f"{base}-{k}.md"
        k += 1
    return name


def absorb(name, src):
    """Move the rows of an old one-file register (a branch from before the split) into the folder.

    A row already in the folder, as written, is skipped. A row whose id is in the folder replaces that row (an
    edit). Any other row is added to the table with the same headings and header, or to a new table at the end
    when its table is new. Rows the old file lacks are left alone: main may have added them since."""
    reg = REGISTERS[name]
    with open(src, encoding="utf-8") as fh:
        old = split_parts(fh.read())
    tree = read_tree(name)
    top = path(reg["dir"])
    have = {row for p in tree if p[0] == "table" for row in p[3]}
    by_id = {}
    tables = {}
    for k, p in enumerate(tree):
        if p[0] != "table":
            continue
        h = headings_before(tree, k)
        tables.setdefault((h, p[1]), p)
        for f, row in zip(p[5], p[3]):
            first = cells(row)[0]
            if reg["id"] and re.fullmatch(reg["id"], first):
                by_id[first] = (p, f)
    added = replaced = 0
    for k, p in enumerate(old):
        if p[0] != "table":
            continue
        key = (headings_before(old, k), p[1])
        for row in p[3]:
            if row in have:
                continue
            first = cells(row)[0]
            if first in by_id:
                t, f = by_id[first]
                with open(os.path.join(top, t[4], f), "w", encoding="utf-8", newline="\n") as fh:
                    fh.write(row_file_text(t[1], t[2], row))
                replaced += 1
                continue
            t = tables.get(key)
            if t is None:
                entries = sorted(os.listdir(top))
                last = max([int(e[:4]) for e in entries if e[:4].isdigit()] or [0])
                (h2, h3) = key[0]
                name_ = slug(" ".join(x for x in (h2, h3) if x) or "new", 50)
                # The prose just above the new table (its heading) comes with it, unless the folder has it.
                prose = {p_[1] for p_ in tree if p_[0] == "text"}
                if k > 0 and old[k - 1][0] == "text" and old[k - 1][1] not in prose:
                    last += 10
                    with open(os.path.join(top, f"{last:04d}-{name_}.md"), "w", encoding="utf-8", newline="\n") as fh:
                        fh.write(old[k - 1][1])
                d = f"{last + 10:04d}-{name_}"
                os.makedirs(os.path.join(top, d))
                t = ("table", p[1], p[2], [], d, [])
                tables[key] = t
                print(f"registers: new table {reg['dir']}/{d}/ (under {h2} / {h3}); move it if it belongs elsewhere")
            f = new_row_name(reg, row, set(os.listdir(os.path.join(top, t[4]))))
            with open(os.path.join(top, t[4], f), "w", encoding="utf-8", newline="\n") as fh:
                fh.write(row_file_text(t[1], t[2], row))
            have.add(row)
            added += 1
    write_pointer(name)
    print(f"registers: {name}: {added} rows added, {replaced} replaced by id; prose edits in the old file are not moved")


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--check", action="store_true")
    ap.add_argument("--print", metavar="NAME", choices=sorted(REGISTERS))
    ap.add_argument("--absorb", metavar="NAME", choices=sorted(REGISTERS))
    ap.add_argument("--from", dest="src", metavar="FILE", help="with --absorb: the old one-file register")
    a = ap.parse_args()
    try:
        if a.absorb:
            absorb(a.absorb, a.src or path(REGISTERS[a.absorb]["pointer"]))
        if a.print:
            sys.stdout.write(render(a.print))
    except RegisterError as e:
        print(e, file=sys.stderr)
        sys.exit(1)
    if a.check:
        check()
    if not (a.check or a.print or a.absorb):
        ap.print_help()


if __name__ == "__main__":
    main()
