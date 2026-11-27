#!/usr/bin/env python3
"""MP22 acceptance tests for scripts/audit-matter.py --ledger.

Builds a tiny repo in a temporary folder and runs the check against it:
a new typed density fails; a labelled one passes and is counted; a ledger
that gains a line fails; a stale line fails; an edited line in the same file
and row is allowed; an empty ledger over a clean tree passes with 0 left.
Python standard library only. Run: python3 scripts/test-audit-matter.py
"""
import json
import os
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
AUDIT = os.path.join(HERE, "audit-matter.py")

SHEET = {"provides": {"reference": {
    "constants": [{"id": "G"}, {"id": "au"}],
    "substances": [{"id": "h2o"}],
    "mixtures": [{"id": "dry-air"}],
}}}


def write(root, rel, text):
    path = os.path.join(root, rel)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8", newline="\n") as fh:
        fh.write(text)


def run(root, *args):
    r = subprocess.run([sys.executable, AUDIT, "--root", root] + list(args), capture_output=True, text=True)
    return r.returncode, r.stdout


def tree(root, code, pack):
    write(root, "packs/base-reference/pack.json", json.dumps(SHEET))
    write(root, "crates/sse-demo/src/lib.rs", code)
    write(root, "packs/base-demo/pack.json", json.dumps(pack, indent=1))
    # A copy inside the sheet's own crate and inside tests is never counted.
    write(root, "crates/sse-reference/src/lib.rs", "pub const G: f64 = 6.674_30e-11;\n")
    write(root, "crates/sse-demo/src/tests.rs", "const AU: f64 = 1.495_978_707e11;\n")


FAILS = []


def expect(name, cond, out=""):
    print(("ok   " if cond else "FAIL ") + name)
    if not cond:
        FAILS.append(name)
        print(out)


def main():
    with tempfile.TemporaryDirectory() as root:
        ledger = os.path.join(root, "ledger.txt")
        base = os.path.join(root, "base.txt")

        # Clean tree, empty ledger: passes, nothing left.
        tree(root, "pub fn f() -> f64 { 1.0 }\n", {"other": {"x": 1}})
        write(root, "ledger.txt", "# empty\n")
        code, out = run(root, "--ledger", ledger, "--base-ledger", ledger)
        expect("clean tree and empty ledger pass", code == 0 and "typed copies left: 0" in out, out)
        code, out = run(root, "--closed")
        expect("closed: a clean tree passes", code == 0 and "typed copies left: 0" in out, out)

        # A new typed density in a pack fails while unlabelled.
        tree(root, "pub fn f() -> f64 { 1.0 }\n", {"other": {"water": {"density_kg_m3": 997.0}}})
        code, out = run(root, "--ledger", ledger, "--base-ledger", ledger)
        expect("new typed density fails", code == 1 and "unlabelled typed copy" in out and "density_kg_m3" in out, out)

        # Closed (MP22 done): the same copy fails with no ledger at all.
        code, out = run(root, "--closed")
        expect("closed: a typed density fails", code == 1 and "unlabelled typed copy" in out, out)

        # By value and unit (SN5): G in working units, and two-figure values
        # beside their unit, fail; the same bare number with no unit passes.
        for name, code_text, bad in [
            ("G in pc (km/s)^2 per solar mass fails", "pub fn g() -> f64 { 4.300_917e-3 }\n", True),
            ("G in Mpc (km/s)^2 per solar mass fails", "pub fn g() -> f64 { 4.30091e-9 }\n", True),
            ("water's refractive index by its name fails", "/// refractive index of water\npub fn n() -> f64 { 1.333 }\n", True),
            ("a photolysis rate beside its unit fails", "/// photo rate, s⁻¹\npub const J: f64 = 1.2e-5;\n", True),
            ("a bare 1.2e-5 tolerance passes", "pub fn tol() -> f64 { 1.2e-5 }\n", False),
            ("a bare 1.333 ratio passes", "pub fn r() -> f64 { 1.333 }\n", False),
        ]:
            tree(root, code_text, {"other": {"x": 1}})
            code, out = run(root, "--closed")
            expect("closed: " + name, (code == 1 and "unlabelled typed copy" in out) if bad else code == 0, out)

        # A new typed constant in Rust fails while unlabelled.
        tree(root, "pub const AU_M: f64 = 149_597_870_700.0;\n", {"other": {"x": 1}})
        code, out = run(root, "--ledger", ledger)
        expect("new typed constant fails", code == 1 and "constant:au" in out, out)

        # Labelled copies pass and are counted.
        tree(root, "pub const AU_M: f64 = 149_597_870_700.0;\n", {"other": {"water": {"density_kg_m3": 997.0}}})
        code, out = run(root, "--write-ledger", ledger)
        code, out = run(root, "--ledger", ledger)
        expect("labelled copies pass and are counted", code == 0 and "typed copies left: 2" in out, out)
        text = open(ledger, encoding="utf-8").read()
        expect("ledger names the sheet row", "substance:h2o/density" in text and "constant:au" in text, text)

        # The ledger may not gain a line against its base.
        write(root, "base.txt", text)
        tree(root, "pub const AU_M: f64 = 149_597_870_700.0;\npub const G: f64 = 6.674_30e-11;\n",
             {"other": {"water": {"density_kg_m3": 997.0}}})
        code, out = run(root, "--write-ledger", ledger)
        code, out = run(root, "--ledger", ledger, "--base-ledger", base)
        expect("a ledger that grew fails", code == 1 and "ledger grew" in out and "ledger line added" in out, out)

        # Removing a copy without deleting its line fails (stale line).
        write(root, "ledger.txt", text)
        tree(root, "pub fn f() -> f64 { 1.0 }\n", {"other": {"water": {"density_kg_m3": 997.0}}})
        code, out = run(root, "--ledger", ledger, "--base-ledger", base)
        expect("a stale ledger line fails", code == 1 and "stale ledger line" in out, out)

        # Deleting the line with the copy passes, and the count drops.
        code, out = run(root, "--write-ledger", ledger)
        code, out = run(root, "--ledger", ledger, "--base-ledger", base)
        expect("shrinking passes and the count drops", code == 0 and "typed copies left: 1" in out, out)

        # Editing a labelled line in place (same file, same row) is allowed.
        tree(root, "pub const AU_M: f64 = 149597870700.0; // reformatted\n",
             {"other": {"water": {"density_kg_m3": 997.0}}})
        code, out = run(root, "--write-ledger", ledger)
        code, out = run(root, "--ledger", ledger, "--base-ledger", base)
        expect("an edited line in the same file and row passes", code == 0 and "typed copies left: 2" in out, out)

        # Bad arguments and unreadable files exit 2.
        code, out = run(root, "--ledger", os.path.join(root, "missing.txt"))
        expect("an unreadable ledger exits 2", code == 2, out)
        write(root, "bad.txt", "only two\tcolumns\n")
        code, out = run(root, "--ledger", os.path.join(root, "bad.txt"))
        expect("a malformed ledger exits 2", code == 2, out)

    print("%d failed" % len(FAILS) if FAILS else "all audit-matter ledger tests passed")
    return 1 if FAILS else 0


if __name__ == "__main__":
    sys.exit(main())
