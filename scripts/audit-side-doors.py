#!/usr/bin/env python3
"""ER side-door ledger (edits-reach design sections 4 and 12).

An edit made in one universe reaches compiled code only through the physics
in force (`sse-physics`). This scan counts the places that still read physics
through a side door, so the count can be kept from growing while each area's
agent converts its own sites as it next touches that code.

Kinds of side door, per file outside tests (one count per kind and file):

  block     a block run with its shipped settings: the library taken straight
            from sse-atoms (`formulas::loaded()`, `formulas::atoms()`,
            `sse_atoms::registry()`, `sse_atoms::atoms()`), or a block's
            settings taken as shipped (`.params.iter().map(|p| p.default)`).
            Door: `physics::run` (ER2).
  sheet     the data sheet's built-in values (`sse_core::sheet`). Door: the
            sheet's fast table (ER4), then ER5.
  sheet-units  the same, in a file that reads only units and conventions
            (au, day, the Julian year, the parsec, atm, g0, 0 C ...). Alex
            ruled on 26 Nov that units never move, so these are listed by
            --list and the totals but are not on the ledger and never fail.
  settings  a shipped pack's settings instead of the stack's: a crate's
            `scenario::settings()` / `envelope()` / `block()`, a
            `base_*_pack()` or `base_*()` reader, or a pack file read with
            `include_str!` outside its crate's scenario module. Door:
            `physics::settings` (ER6).

Not scanned: test files (`*_tests.rs`, `tests.rs`, `tests/`, `benches/`,
`examples/`) and
inline `#[cfg(test)] mod ... { }` blocks; each crate's `scenario.rs` (the
shipped scenario itself); and the homes of the doors (`sse-core/src/sheet.rs`,
`sse-atoms`, `sse-reference`, `sse-physics`).

Usage:
  scripts/audit-side-doors.py                  print the totals
  scripts/audit-side-doors.py --list           every site, by file and line
  scripts/audit-side-doors.py --check [--base REF]
                                               the ledger must equal the scan
                                               (a new site fails; a converted
                                               site whose line is still in the
                                               ledger fails too), and with
                                               --base no ledger line may be
                                               higher than at REF (a git ref)
  scripts/audit-side-doors.py --shrink         rewrite the ledger to today's
                                               counts; refuses if any count
                                               would rise (run it after
                                               converting sites)
  scripts/audit-side-doors.py --write-ledger   write the ledger from scratch
                                               (once, ER1; never to label a
                                               new site)
  --ledger FILE   the ledger (default handover/packages/ER-side-doors.txt)
  --root DIR      scan DIR instead of the repo (tests)

Exit codes: 0 pass, 1 fail, 2 could not run. Textual on purpose: no build
needed, runs in a second, deterministic (files and sites are sorted).
"""
import json
import os
import re
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
LEDGER = os.path.join("handover", "packages", "ER-side-doors.txt")
KINDS = ["block", "sheet", "sheet-units", "settings"]
# Kinds the ledger holds and --check enforces (units never move: Alex, 26 Nov).
LEDGERED = ["block", "sheet", "settings"]

SKIP_PATHS = [
    "crates/sse-core/src/sheet.rs",
    "crates/sse-atoms/",
    "crates/sse-reference/",
    "crates/sse-physics/",
]

BLOCK = re.compile(
    r"\bformulas::(loaded|atoms)\(|\bsse_atoms::(registry|atoms)\(|\.params\.iter\(\)\.map\(\|\w+\| \w+\.default\)"
)
# Exact exceptions to the `block` kind: (path, the line's code, why). Each is
# a line that matches the pattern but is not a side door.
BLOCK_EXCEPTIONS = [
    (
        "crates/sse-flow/src/flow.rs",
        "let mut params: Vec<f64> = decl.params.iter().map(|p| p.default).collect();",
        "the flow compiler starts from the registry it is handed, which is the blocks in force (ER2)",
    ),
]
SHEET_PATH = re.compile(r"(?<![\w:])(?:sse_core::)?sheet::((?:[a-z_]+::)*)([A-Za-z_][A-Za-z0-9_]*)")
SHEET_MENTION = re.compile(r"\bsse_core::sheet\b")
SETTINGS = re.compile(
    r"\bscenario::(settings|envelope|block)\(|\bbase_[a-z_]+_pack\(\)|\bbase_(settings|fits|references|atlas|rules|pack)\(\)"
    r'|include_str!\("[./]*packs/(?!base-physics/formulas/)'
)
DEFINES = re.compile(r"^\s*(pub(\([a-z]+\))?\s+)?(const\s+)?fn\s|^\s*use\s")


def unit_names(root):
    """Sheet constants that are units or conventions, as their Rust names."""
    path = os.path.join(root, "packs", "base-reference", "pack.json")
    names = set()
    try:
        with open(path, encoding="utf-8") as f:
            ref = json.load(f)["provides"]["reference"]
    except (OSError, KeyError, ValueError):
        return names
    for row in ref.get("constants", []):
        rid = row.get("id", "")
        if row.get("group") in ("convention", "factor") and not re.search(
            r"_(sun|earth|jup|moon)", rid
        ):
            names.add(re.sub(r"[^A-Za-z0-9]", "_", rid).upper())
    return names


def is_test_file(rel):
    base = os.path.basename(rel)
    return (
        base == "tests.rs"
        or base.endswith("_tests.rs")
        or "/tests/" in rel
        or "/benches/" in rel
        or "/examples/" in rel
        or base == "scenario.rs"
    )


def code_lines(text):
    """(line number, line) outside inline #[cfg(test)] modules and comments."""
    lines = text.split("\n")
    i = 0
    while i < len(lines):
        line = lines[i]
        if line.strip() == "#[cfg(test)]":
            j = i + 1
            while j < len(lines) and not lines[j].strip():
                j += 1
            if j < len(lines) and re.match(r"\s*(pub\s+)?mod\s+\w+\s*\{", lines[j]):
                depth = 0
                k = j
                while k < len(lines):
                    depth += lines[k].count("{") - lines[k].count("}")
                    if depth <= 0:
                        break
                    k += 1
                i = k + 1
                continue
        stripped = line.lstrip()
        if not stripped.startswith("//"):
            yield i + 1, line
        i += 1


def scan(root):
    """{(kind, path): [line numbers]}."""
    units = unit_names(root)
    out = {}
    crates = os.path.join(root, "crates")
    files = []
    for d, dirs, fs in os.walk(crates):
        dirs[:] = sorted(x for x in dirs if x not in ("target",))
        for f in sorted(fs):
            if f.endswith(".rs"):
                files.append(os.path.relpath(os.path.join(d, f), root).replace(os.sep, "/"))
    for rel in sorted(files):
        if is_test_file(rel) or any(rel.startswith(s) for s in SKIP_PATHS):
            continue
        with open(os.path.join(root, rel), encoding="utf-8") as f:
            text = f.read()
        sheet_lines, names_used = [], set()
        for n, line in code_lines(text):
            if BLOCK.search(line) and not any(
                rel == p and line.strip() == code for p, code, _ in BLOCK_EXCEPTIONS
            ):
                out.setdefault(("block", rel), []).append(n)
            if SETTINGS.search(line) and not DEFINES.search(line):
                out.setdefault(("settings", rel), []).append(n)
            hit = False
            for m in SHEET_PATH.finditer(line):
                if "sse_flow::" in line[max(0, m.start() - 9) : m.start()]:
                    continue
                hit = True
                if not m.group(1):
                    names_used.add(m.group(2))
                else:
                    names_used.add(m.group(1) + m.group(2))
            if SHEET_MENTION.search(line):
                hit = True
                for g in re.findall(r"sse_core::sheet::\{([^}]*)\}", line):
                    for nm in g.split(","):
                        nm = nm.strip().split(" as ")[0].strip()
                        if nm and nm != "self":
                            names_used.add(nm)
            if hit:
                sheet_lines.append(n)
        if sheet_lines:
            plain = {x for x in names_used if x not in ("self",)}
            kind = "sheet-units" if plain and all(x in units for x in plain) else "sheet"
            out[(kind, rel)] = sheet_lines
    return out


def counts(found):
    return {k: len(v) for k, v in found.items()}


def ledgered(c):
    return {k: n for k, n in c.items() if k[0] in LEDGERED}


def read_ledger_text(text):
    led = {}
    for line in text.splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        parts = line.split()
        if len(parts) != 3 or parts[0] not in LEDGERED:
            raise ValueError(f"bad ledger line: {line!r}")
        led[(parts[0], parts[1])] = int(parts[2])
    return led


def ledger_text(c):
    out = [
        "# ER side-door ledger (scripts/audit-side-doors.py). It may only shrink.",
        "# kind  file  sites. Convert a file's sites to the physics in force, then",
        "# run `python3 scripts/audit-side-doors.py --shrink`. Never add a line.",
    ]
    for kind in LEDGERED:
        for (k, path), n in sorted(c.items()):
            if k == kind and n > 0:
                out.append(f"{k} {path} {n}")
    return "\n".join(out) + "\n"


def totals(c):
    t = {k: [0, 0] for k in KINDS}
    for (k, _), n in c.items():
        t[k][0] += n
        t[k][1] += 1
    return ", ".join(f"{k} {t[k][0]} in {t[k][1]} files" for k in KINDS)


def main(argv):
    args = argv[1:]
    root = ROOT
    ledger = None
    base = None
    mode = "summary"
    i = 0
    while i < len(args):
        a = args[i]
        if a in ("--root", "--ledger", "--base"):
            if i + 1 >= len(args):
                print(f"{a} needs a value", file=sys.stderr)
                return 2
            v = args[i + 1]
            if a == "--root":
                root = v
            elif a == "--ledger":
                ledger = v
            else:
                base = v
            i += 2
            continue
        if a in ("--list", "--check", "--shrink", "--write-ledger"):
            mode = a[2:]
        else:
            print(f"unknown argument {a}", file=sys.stderr)
            return 2
        i += 1
    ledger_rel = ledger or LEDGER
    ledger_path = ledger_rel if os.path.isabs(ledger_rel) else os.path.join(root, ledger_rel)
    found = scan(root)
    every = counts(found)
    now = ledgered(every)

    if mode == "summary":
        print("side doors:", totals(every))
        return 0
    if mode == "list":
        for (k, path), lines in sorted(found.items()):
            print(f"{k} {path}: lines {', '.join(map(str, lines))}")
        print("side doors:", totals(every))
        return 0
    if mode == "write-ledger":
        with open(ledger_path, "w", encoding="utf-8", newline="\n") as f:
            f.write(ledger_text(now))
        print("ledger written:", totals(every))
        return 0
    try:
        with open(ledger_path, encoding="utf-8") as f:
            led = read_ledger_text(f.read())
    except (OSError, ValueError) as e:
        print(f"cannot read the ledger: {e}", file=sys.stderr)
        return 2
    if mode == "shrink":
        rises = [(k, n, led.get(k, 0)) for k, n in sorted(now.items()) if n > led.get(k, 0)]
        if rises:
            for (kind, path), n, was in rises:
                print(f"NEW SIDE DOOR {kind} {path}: {n} sites, ledger {was}", file=sys.stderr)
            print("refused: --shrink never raises a count. Read physics through sse-physics instead.", file=sys.stderr)
            return 1
        with open(ledger_path, "w", encoding="utf-8", newline="\n") as f:
            f.write(ledger_text(now))
        print("ledger shrunk:", totals(every))
        return 0

    # --check
    bad = []
    for key in sorted(set(now) | set(led)):
        n, l = now.get(key, 0), led.get(key, 0)
        kind, path = key
        if n > l:
            lines = ", ".join(map(str, found.get(key, [])))
            bad.append(
                f"NEW SIDE DOOR {kind} {path}: {n} sites (lines {lines}), ledger allows {l}. "
                "Read it through sse-physics (the physics in force) instead."
            )
        elif n < l:
            bad.append(
                f"CONVERTED {kind} {path}: {n} sites left, ledger says {l}. "
                "Good: run `python3 scripts/audit-side-doors.py --shrink` and commit the ledger."
            )
    if base:
        try:
            text = subprocess.run(
                ["git", "show", f"{base}:{ledger_rel}"],
                cwd=root,
                capture_output=True,
                text=True,
                check=True,
            ).stdout
            was = read_ledger_text(text)
        except subprocess.CalledProcessError:
            was = None  # no ledger at the base yet: this change starts it
        except ValueError as e:
            print(f"cannot read the base ledger: {e}", file=sys.stderr)
            return 2
        if was is not None:
            for key, l in sorted(led.items()):
                if l > was.get(key, 0):
                    bad.append(
                        f"LEDGER GREW {key[0]} {key[1]}: {l}, was {was.get(key, 0)} at {base}. "
                        "The ledger may only shrink."
                    )
    for b in bad:
        print(b, file=sys.stderr)
    if bad:
        return 1
    print("side doors left:", totals(every))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
