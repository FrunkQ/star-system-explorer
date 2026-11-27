#!/usr/bin/env python3
"""Tests for scripts/registers.py: the split is lossless, --check refuses what it should, --absorb moves old rows."""

import importlib.util
import os
import shutil
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
spec = importlib.util.spec_from_file_location("registers", os.path.join(HERE, "registers.py"))
r = importlib.util.module_from_spec(spec)
spec.loader.exec_module(r)

DOC = """# Fudge register

Intro text.

## Stars

### Fudge register

| # | Where | Status |
|---|---|---|
| F-S1 | star.rs | Stand-in |
| F-S2 | lifeline.rs | Permanent |

Between tables.

## Orbits

| Where | Status |
|---|---|
| disc.rs | Stand-in |

Closing words.
"""

bad = 0
passed = 0


def expect(ok, what):
    global bad, passed
    if ok:
        passed += 1
    else:
        print(f"FAIL {what}")
        bad += 1


def refused(what):
    try:
        r.check_one("fudges")
    except r.RegisterError:
        expect(True, what)
        return
    expect(False, what)


def write(rel, text):
    with open(os.path.join(r.ROOT, rel), "w", encoding="utf-8") as fh:
        fh.write(text)


with tempfile.TemporaryDirectory() as tmp:
    r.ROOT = tmp
    os.makedirs(os.path.join(tmp, "docs"))
    r.explode("fudges", DOC)
    expect(r.render("fudges") == DOC, "split and join give the document back exactly")
    expect(r.check_one("fudges") == 3, "three rows counted")
    table = os.path.join(tmp, "docs/fudges", sorted(d for d in os.listdir(os.path.join(tmp, "docs/fudges")) if "." not in d)[0])
    expect(sorted(os.listdir(table)) == ["0010-F-S1.md", "0020-F-S2.md"], "rows named by order and id")

    # A new row named by its id sorts after the numbered rows: two PRs adding rows touch different files.
    write(os.path.join(table, "F-S3.md"), "| # | Where | Status |\n|---|---|---|\n| F-S3 | new.rs | Stand-in |\n")
    expect(r.render("fudges").index("F-S3") > r.render("fudges").index("F-S2"), "a new row sorts last")

    write(os.path.join(table, "F-S9.md"), "| # | Where | Status |\n|---|---|---|\n| F-S1 | twice.rs | Stand-in |\n")
    refused("an id used twice")
    os.remove(os.path.join(table, "F-S9.md"))

    write(os.path.join(table, "x.md"), "| F-S8 | no header | Stand-in |\n")
    refused("a row file without its header")
    os.remove(os.path.join(table, "x.md"))

    write(os.path.join(table, "y.md"), "| # | Where |\n|---|---|\n| F-S8 | other header |\n")
    refused("a row file with a different header")
    os.remove(os.path.join(table, "y.md"))

    # A branch from before the split: its old one-file register comes back with a merge.
    old = DOC.replace("| F-S2 | lifeline.rs | Permanent |", "| F-S2 | lifeline.rs:9 | Permanent |\n| F-S4 | added.rs | Stand-in |")
    old = old.replace("Closing words.\n", "Closing words.\n\n## Light\n\n| Where | Status |\n|---|---|\n| gpu.ts | Stand-in |\n")
    write("docs/fudges.md", old)
    refused("the pointer page holding rows again")
    write("old.md", old)
    r.absorb("fudges", os.path.join(tmp, "old.md"))
    text = r.render("fudges")
    expect("| F-S4 | added.rs | Stand-in |" in text, "absorb adds a new row")
    expect("lifeline.rs:9" in text and "| F-S2 | lifeline.rs | Permanent |" not in text, "absorb replaces an edited row by id")
    expect("| F-S3 | new.rs | Stand-in |" in text, "absorb keeps rows main added")
    expect(text.index("gpu.ts") > text.index("## Light") > text.index("Closing words."), "absorb puts a new table at the end, with its heading")
    expect(r.check_one("fudges") == 6, "after absorb the register checks")

# The deploy builds the readable pages with scripts/build-registers.mjs; it must read the folders as this
# script does, on the real registers.
r.ROOT = os.path.dirname(HERE)
if shutil.which("node"):
    for name in r.REGISTERS:
        got = subprocess.run(
            ["node", os.path.join(HERE, "build-registers.mjs"), "--markdown", name], capture_output=True, text=True
        ).stdout
        expect(got == r.render(name), f"the deploy page builder reads {name} as registers.py does")
else:
    print("registers tests: node not found; the page builder was not compared")

if bad:
    sys.exit(1)
print(f"registers tests: {passed} passed")
