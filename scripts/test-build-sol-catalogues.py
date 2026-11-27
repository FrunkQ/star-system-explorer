#!/usr/bin/env python3
"""Checks scripts/build-sol-catalogues.py on a saved cache of invented rows
(tests/fixtures/catalogues/sol-build, shaped like JPL's answers; never real
data). Builds into a temporary folder with the sse CLI and reads the files
back with `sse catalogue show`, so the catalogue reader itself judges them.

  python3 scripts/test-build-sol-catalogues.py [--sse PATH]
"""
import json
import os
import subprocess
import sys
import tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
FIX = os.path.join(ROOT, "tests", "fixtures", "catalogues", "sol-build")


def main(argv):
    sse = argv[argv.index("--sse") + 1] if "--sse" in argv else None
    cmd = [sys.executable, os.path.join(ROOT, "scripts", "build-sol-catalogues.py"), "build", FIX]
    with tempfile.TemporaryDirectory() as out:
        extra = ["--sse", sse] if sse else []
        r = subprocess.run(cmd + [out] + extra, cwd=ROOT, capture_output=True, text=True)
        assert r.returncode == 0, r.stdout + r.stderr
        rows = json.load(open(os.path.join(out, "sol-small-bodies.catalogue.json")))["rows"]
        by = {x["id"]: x for x in rows}
        sizes = [x["diameter_km"] for x in rows]
        assert sizes == sorted(sizes, reverse=True), "biggest first"
        # Measured sizes stay measured; a missing one is estimated, never invented.
        assert by["1"]["diameter_grade"] == "measured" and by["1"]["diameter_km"] == 939.4
        est = by["101"]
        assert est["diameter_grade"] == "estimated" and est["albedo_grade"] == "estimated", est
        # The class median (MBA 0.1 in the fixture) gives D = 1329 / sqrt(0.1) x 10^(-H/5).
        want = 1329.0 / 0.1 ** 0.5 * 10 ** (-est["h_mag"] / 5)
        assert abs(est["diameter_km"] - want) < 1e-9 * want, (est, want)
        # A comet with no diameter and no H is sized from M2; one with neither is left out.
        assert by["2P"]["size_from"] == "M2", by["2P"]
        assert "3D" not in by
        # Notables are flagged only when JPL's name matches the list; base-sol bodies are linked.
        assert by["1P"]["flags"] == 1 and by["4"]["flags"] == 1 and by["101"]["flags"] == 0
        assert by["433"]["flags"] == 0, "a designation whose JPL name differs is not flagged"
        assert by["1"]["base_sol"] == "ceres"
        # A base-sol body's measured size beats an estimate from brightness (JPL gives Pluto none).
        assert by["134340"]["diameter_grade"] == "measured" and by["134340"]["diameter_source"] == "base-sol", by["134340"]
        assert by["134340"]["albedo_source"] == "base-sol", by["134340"]
        # A size JPL lacks comes from its paper (scripts/sol-measured-sizes.json), graded measured.
        assert by["136199"]["diameter_km"] == 2326.0 and by["136199"]["diameter_source"] == "sicardy2011", by["136199"]
        assert by["136199"]["albedo"] == 0.96 and by["136199"]["albedo_grade"] == "measured"
        assert by["1"]["spkid"] == "2000001", "spkid comes as a JSON number and is kept as text"
        moons = json.load(open(os.path.join(out, "sol-moons-ecliptic-j2000.catalogue.json"), encoding="utf-8"))["rows"]
        nereid = [m for m in moons if m["name"] == "Nereid"]
        assert len(nereid) == 1, "a repeated NAIF code is kept once"
        assert nereid[0]["diameter_km"] == 356.0 and "gm_km3_s2" not in nereid[0], nereid
        laplace = json.load(open(os.path.join(out, "sol-moons-laplace.catalogue.json"), encoding="utf-8"))["rows"]
        assert laplace[0]["diameter_km"] == 22.16 and laplace[0]["density_g_cm3"] == 1.86, laplace
        report = open(os.path.join(out, "report.md"), encoding="utf-8").read()
        fake = [m for m in moons if m["name"] == "Fakemoon"]
        assert fake and fake[0]["diameter_km"] == 24.0 and fake[0]["diameter_grade"] == "estimated" \
            and fake[0]["diameter_source"] == "jpl-horizons", fake
        assert "Nereid" in [m["name"] for m in moons]
        assert laplace[0]["diameter_grade"] == "measured"
        # The reader accepts every file and every credit is carried.
        for f in sorted(os.listdir(out)):
            if f.endswith(".ssecat"):
                show = (sse and [sse]) or ["cargo", "run", "-q", "-p", "sse-cli", "--release", "--"]
                r = subprocess.run(show + ["catalogue", "show", os.path.join(out, f), "--limit", "1"],
                                   cwd=ROOT, capture_output=True, text=True)
                assert r.returncode == 0, r.stderr
                assert "NASA/JPL Solar System Dynamics. Data retrieved on 2026, November, 26." in r.stdout, f
    print("build-sol-catalogues: ok")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
