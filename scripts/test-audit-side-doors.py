#!/usr/bin/env python3
"""Tests for scripts/audit-side-doors.py (the ER side-door ledger).

Each case builds a small tree, so the check is seen to go red on a new side
door, red on a converted site left in the ledger, and green once shrunk.
"""
import os
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
AUDIT = os.path.join(HERE, "audit-side-doors.py")
LEDGER = "ledger.txt"


def run(root, *args):
    p = subprocess.run(
        [sys.executable, AUDIT, "--root", root, "--ledger", LEDGER, *args],
        capture_output=True,
        text=True,
    )
    return p.returncode, p.stdout + p.stderr


def write(root, rel, text):
    path = os.path.join(root, rel)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(text)


def main():
    fails = []

    def expect(cond, what, out=""):
        if not cond:
            fails.append(f"{what}\n{out}")

    with tempfile.TemporaryDirectory() as root:
        write(root, "packs/base-reference/pack.json",
              '{"provides": {"reference": {"constants": ['
              '{"id": "au", "group": "convention"}, {"id": "G", "group": "nature"}]}}}')
        write(root, "crates/sse-a/src/lib.rs",
              "use sse_core::sheet;\nfn f() -> f64 { sheet::G }\n"
              "fn s() {\n    let x = sse_surface::scenario::settings();\n}\n"
              "#[cfg(test)]\nmod tests {\n    fn t() { let x = sse_surface::scenario::settings(); }\n}\n")
        write(root, "crates/sse-a/src/units.rs", "fn d() -> f64 { sse_core::sheet::AU }\n")
        write(root, "crates/sse-a/src/a_tests.rs", "fn t() { sse_atoms::registry(); }\n")
        write(root, "crates/sse-a/src/scenario.rs", "fn s() { base_surface_pack(); }\n")
        code, out = run(root, "--write-ledger")
        expect(code == 0, "write the ledger", out)
        with open(os.path.join(root, LEDGER), encoding="utf-8") as f:
            led = f.read()
        expect("sheet crates/sse-a/src/lib.rs 2" in led, "sheet lines counted", led)
        expect("units.rs" not in led, "a units-only file is not on the ledger", led)
        code, out = run(root, "--list")
        expect("sheet-units crates/sse-a/src/units.rs: lines 1" in out, "a units-only file is listed as its own kind", out)
        expect("settings crates/sse-a/src/lib.rs 1" in led, "an inline test module is not scanned", led)
        expect("a_tests.rs" not in led and "scenario.rs" not in led, "test files and scenario.rs are not scanned", led)
        code, out = run(root, "--check")
        expect(code == 0, "the fresh ledger passes", out)

        # A new side door goes red, and --shrink refuses to take it in.
        write(root, "crates/sse-b/src/lib.rs", "fn r() { let a = sse_atoms::formulas::loaded(); }\n")
        code, out = run(root, "--check")
        expect(code == 1 and "NEW SIDE DOOR block crates/sse-b/src/lib.rs" in out, "a new block site fails", out)
        code, out = run(root, "--shrink")
        expect(code == 1 and "refused" in out, "--shrink never raises a count", out)
        os.remove(os.path.join(root, "crates/sse-b/src/lib.rs"))

        # A new units read never fails.
        write(root, "crates/sse-c/src/lib.rs", "fn d() -> f64 { sse_core::sheet::AU }\n")
        code, out = run(root, "--check")
        expect(code == 0, "a new units-only read passes", out)

        # A converted site whose line is left in the ledger goes red too.
        write(root, "crates/sse-a/src/lib.rs", "fn s() {\n    let x = sse_surface::scenario::settings();\n}\n")
        code, out = run(root, "--check")
        expect(code == 1 and "CONVERTED sheet crates/sse-a/src/lib.rs" in out, "a stale ledger line fails", out)
        code, out = run(root, "--shrink")
        expect(code == 0, "--shrink takes the converted file off", out)
        code, out = run(root, "--check")
        expect(code == 0, "green once shrunk", out)
        with open(os.path.join(root, LEDGER), encoding="utf-8") as f:
            expect("sheet crates/sse-a/src/lib.rs" not in f.read(), "the converted file left the ledger")

        # --base: a ledger line raised by hand against the base fails.
        subprocess.run(["git", "init", "-q"], cwd=root, check=True)
        subprocess.run(["git", "add", "-A"], cwd=root, check=True)
        subprocess.run(["git", "-c", "user.name=t", "-c", "user.email=t@t", "commit", "-qm", "base"],
                       cwd=root, check=True)
        write(root, "crates/sse-a/src/lib.rs",
              "use sse_core::sheet;\nfn f() -> f64 { sheet::G + sheet::C }\nfn g() -> f64 { sheet::G }\n"
              "fn s() {\n    let x = sse_surface::scenario::settings();\n}\n")
        with open(os.path.join(root, LEDGER), encoding="utf-8") as f:
            led = f.read()
        with open(os.path.join(root, LEDGER), "w", encoding="utf-8") as f:
            f.write(led.replace("settings crates/sse-a/src/lib.rs 1",
                                "settings crates/sse-a/src/lib.rs 1\nsheet crates/sse-a/src/lib.rs 3"))
        code, out = run(root, "--check")
        expect(code == 0, "the hand-raised ledger matches the scan", out)
        code, out = run(root, "--check", "--base", "HEAD")
        expect(code == 1 and "LEDGER GREW sheet crates/sse-a/src/lib.rs" in out, "a grown ledger fails against the base", out)

    if fails:
        for f in fails:
            print("FAIL:", f)
        return 1
    print("audit-side-doors tests passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
