#!/usr/bin/env python3
"""Catalogued starter systems beside Sol: packs/base-starters/pack.json.

Two systems the start sheet can open as measured, chosen because they are the
cases pairs must get right (editing design 10A, Alex 24 Nov 2026 15:13):

  alpha_centauri  a close pair (A and B, 80 years) with Proxima as a wide
                  third round the pair: a nested pair.
  kepler_16       a world round both stars of a close pair (a circumbinary
                  planet).

Each system has the shape of base-sol's sheet (about, bodies, regions,
sources), so the start sheet's door reads it the same way. Values are as
printed in the source beside them, in the unit printed, and turned into SI
here only where the source prints solar or Jupiter units (the nominal values
named below). Angles stay in degrees; the door turns them into radians. Every
body's `kind` is the catalogue's: the door should send it as body.kind, since
no star here has to be found by its temperature.

Orbit angles are on the plane of the sky (the observer's frame), the same
frame for every orbit in a system, so the mutual inclinations are real. No
orbital phase is entered: the universe draws one, labelled resolved, until a
dated phase is added.

  scripts/build-base-starters.py          write the pack
  scripts/build-base-starters.py --check  exit 1 if the pack is stale
"""
import json
import os
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(ROOT, "packs", "base-starters", "pack.json")

# IAU 2015 Resolution B3 nominal values (iau2015b3), as base-sol and the master
# data sheet use them. M_sun is the NSSDCA mass base-sol's Sun carries.
M_SUN = 1.9884e30
R_SUN = 6.957e8
L_SUN = 3.828e26
AU = 1.495978707e11
M_JUP = 1.89813e27  # NSSDCA Jupiter fact sheet, as base-sol's Jupiter
R_JUP = 7.1492e7  # IAU 2015 B3 nominal equatorial radius of Jupiter
DAY = 86400.0
YR = 365.25 * DAY


def val(v, unit, src, note=None):
    out = {"v": float(f"{v:.10g}"), "unit": unit, "src": src}
    if note:
        out["note"] = note
    return out


SOURCES = [
    {"id": "iau2015b3", "text": "Prša, A. et al. 2016, AJ 152, 41: Nominal values for selected solar and planetary quantities, IAU 2015 Resolution B3",
     "status": "verified", "doi": "10.3847/0004-6256/152/2/41"},
    {"id": "kervella2016", "text": "Kervella, P. et al. 2016, A&A 594, A107: Close stellar conjunctions of alpha Centauri A and B until 2050, Table 1 (K16 column: parallax, orbital elements and masses; read 24 Nov 2026)",
     "status": "verified", "doi": "10.1051/0004-6361/201629201"},
    {"id": "kervella2017ab", "text": "Kervella, P. et al. 2017, A&A 597, A137: The radii and limb darkenings of alpha Centauri A and B, Table 5 (radii, temperatures, luminosities; read 24 Nov 2026)",
     "status": "verified", "doi": "10.1051/0004-6361/201629505"},
    {"id": "kervella2017prox", "text": "Kervella, P., Thévenin, F. and Lovis, C. 2017, A&A 598, L7: Proxima's orbit around alpha Centauri, Table 3 (orbit) and Table 1 note c (mass and radius, from Mann et al. 2015; read 24 Nov 2026)",
     "status": "verified", "doi": "10.1051/0004-6361/201629930"},
    {"id": "doyle2011", "text": "Doyle, L. R. et al. 2011, Science 333, 1602: Kepler-16: a transiting circumbinary planet, Table 1 (star A and planet b as the NASA Exoplanet Archive prints the Doyle 2011 set; read 24 Nov 2026)",
     "status": "verified", "doi": "10.1126/science.1210923"},
    {"id": "doyle2011-binary", "text": "Doyle, L. R. et al. 2011, Science 333, 1602, Table 1: star B and the binary's orbit. Not yet read at source: the archive does not print them and the paper's table was not reachable; star B's radius matches the value Wikipedia's Kepler-16 page prints",
     "status": "unverified", "doi": "10.1126/science.1210923"},
]


def alpha_centauri():
    k16, k17, kp = "kervella2016", "kervella2017ab", "kervella2017prox"
    parallax_arcsec = 0.74717
    a_ab = 17.592 / parallax_arcsec  # au: angular semi-major axis over parallax
    a = {"id": "alpha-cen-a", "name": "Alpha Centauri A", "kind": "star", "host": None, "values": {
        "body.mass": val(1.1055 * M_SUN, "kg", k16, note="1.1055 ± 0.0039 solar masses"),
        "body.radius": val(8.511e8, "m", k17, note="1.2234 ± 0.0053 solar radii"),
        "star.effective_temperature": val(5795.0, "K", k17, note="± 19 K"),
        "star.luminosity": val(5.821e26, "W", k17, note="1.521 ± 0.015 solar luminosities"),
    }}
    b = {"id": "alpha-cen-b", "name": "Alpha Centauri B", "kind": "star", "host": "alpha-cen-a", "values": {
        "body.mass": val(0.9373 * M_SUN, "kg", k16, note="0.9373 ± 0.0033 solar masses"),
        "body.radius": val(6.006e8, "m", k17, note="0.8632 ± 0.0037 solar radii"),
        "star.effective_temperature": val(5231.0, "K", k17, note="± 21 K"),
        "star.luminosity": val(1.925e26, "W", k17, note="0.503 ± 0.007 solar luminosities"),
        "orbit.semi_major_axis": val(a_ab * AU, "m", k16, note="17.592 ± 0.013 arcsec at a parallax of 747.17 ± 0.61 mas (%.3f au); the period, 79.929 ± 0.013 yr, follows from it and the masses" % a_ab),
        "orbit.eccentricity": val(0.5208, "1", k16, note="± 0.0011"),
        "orbit.inclination": val(79.320, "deg", k16, note="to the plane of the sky, ± 0.011"),
        "orbit.ascending_node": val(205.064, "deg", k16, note="± 0.033"),
        "orbit.argument_of_periapsis": val(232.006, "deg", k16, note="± 0.051; periastron 1955.604"),
    }}
    prox = {"id": "proxima", "name": "Proxima Centauri", "kind": "star", "host": "alpha-cen-a", "values": {
        "body.mass": val(0.1221 * M_SUN, "kg", kp, note="0.1221 ± 0.0022 solar masses"),
        "body.radius": val(0.1542 * R_SUN, "m", kp, note="0.1542 ± 0.0045 solar radii"),
        "orbit.semi_major_axis": val(8.7e3 * AU, "m", kp, note="8.7 (+0.7, -0.4) thousand au, round the A and B pair; period 547 (+66, -40) thousand years"),
        "orbit.eccentricity": val(0.50, "1", kp, note="+0.08, -0.09"),
        "orbit.inclination": val(107.6, "deg", kp, note="to the plane of the sky, +1.8, -2.0"),
        "orbit.ascending_node": val(126.0, "deg", kp, note="± 5"),
        "orbit.argument_of_periapsis": val(72.3, "deg", kp, note="+8.7, -6.6; next periastron in 283 (+59, -41) thousand years"),
    }}
    return {
        "about": "Alpha Centauri as measured: A and B, a close pair on an 80-year orbit, and Proxima on a wide orbit round the pair, a nested pair. Built by scripts/build-base-starters.py; every value names its source. Proxima's planets are not entered yet.",
        "bodies": [a, b, prox], "regions": [], "sources": [s for s in SOURCES if s["id"] in ("kervella2016", "kervella2017ab", "kervella2017prox")],
    }


def kepler_16():
    d, db = "doyle2011", "doyle2011-binary"
    a = {"id": "kepler-16-a", "name": "Kepler-16 A", "kind": "star", "host": None, "values": {
        "body.mass": val(0.6897 * M_SUN, "kg", d, note="0.6897 (+0.0035, -0.0034) solar masses"),
        "body.radius": val(0.6489 * R_SUN, "m", d, note="0.6489 ± 0.0013 solar radii"),
        "star.effective_temperature": val(4450.0, "K", d, note="± 150 K"),
    }}
    b = {"id": "kepler-16-b-star", "name": "Kepler-16 B", "kind": "star", "host": "kepler-16-a", "values": {
        "body.mass": val(0.20255 * M_SUN, "kg", db, note="0.20255 solar masses"),
        "body.radius": val(0.22623 * R_SUN, "m", db, note="0.22623 solar radii"),
        "orbit.semi_major_axis": val(0.22431 * AU, "m", db, note="0.22431 au; period 41.07922 days"),
        "orbit.eccentricity": val(0.15944, "1", db),
        "orbit.inclination": val(90.3401, "deg", db, note="to the plane of the sky: the pair eclipses"),
        "orbit.argument_of_periapsis": val(263.464, "deg", db),
    }}
    planet = {"id": "kepler-16-b", "name": "Kepler-16 b", "kind": "planet", "host": "kepler-16-a", "values": {
        "body.mass": val(0.333 * M_JUP, "kg", d, note="0.333 ± 0.016 Jupiter masses"),
        "body.radius": val(0.7538 * R_JUP, "m", d, note="0.7538 ± 0.0026 Jupiter radii (equatorial nominal)"),
        "orbit.semi_major_axis": val(0.7048 * AU, "m", d, note="0.7048 ± 0.0011 au, round both stars; period 228.776 (+0.020, -0.037) days"),
        "orbit.eccentricity": val(0.0069, "1", d, note="+0.0010, -0.0015"),
        "orbit.inclination": val(90.0322, "deg", d, note="to the plane of the sky, +0.0022, -0.0023; within 0.4 degrees of the pair's plane"),
        "orbit.argument_of_periapsis": val(318.0, "deg", d, note="+10.0, -22.0"),
    }}
    return {
        "about": "Kepler-16 as measured: a K dwarf and an M dwarf on a 41-day orbit, eclipsing, and a Saturn-mass world on a 229-day orbit round both. Built by scripts/build-base-starters.py; every value names its source.",
        "bodies": [a, b, planet], "regions": [], "sources": [s for s in SOURCES if s["id"] in ("doyle2011", "doyle2011-binary")],
    }


def build():
    return {
        "format": "sse.pack/1", "id": "sse.base-starters", "version": "0.1.0", "kind": ["base-physics"],
        "kernel_range": "^0.1", "licence": "Apache-2.0",
        "provides": {"other": {"alpha_centauri": alpha_centauri(), "kepler_16": kepler_16()}},
    }


def validate(pack):
    for key, s in pack["provides"]["other"].items():
        ids = {x["id"] for x in s["sources"]}
        for x in s["sources"]:
            assert x.get("doi") or x.get("url"), "%s: source %s has no DOI or URL" % (key, x["id"])
        bids = {b["id"] for b in s["bodies"]}
        assert len(bids) == len(s["bodies"]), key
        assert sum(1 for b in s["bodies"] if b["host"] is None) == 1, "%s: one body without a host" % key
        for b in s["bodies"]:
            assert b["kind"] in ("star", "planet", "moon", "dwarf_planet"), (key, b["id"])
            assert b["host"] is None or b["host"] in bids, "%s %s: unknown host" % (key, b["id"])
            for f, v in b["values"].items():
                assert v["src"] in ids, "%s %s %s: unknown source" % (key, b["id"], f)
                assert v["unit"] in ("1", "m", "kg", "K", "W", "deg"), (key, b["id"], f)
                assert isinstance(v["v"], float) and v["v"] == v["v"], (key, b["id"], f)


def main():
    pack = build()
    validate(pack)
    text = json.dumps(pack, indent=1, ensure_ascii=False, allow_nan=False) + "\n"
    if "--check" in sys.argv:
        cur = open(OUT, encoding="utf-8").read() if os.path.exists(OUT) else ""
        if cur != text:
            print("packs/base-starters is stale: run scripts/build-base-starters.py")
            return 1
        print("base-starters pack is current")
        return 0
    os.makedirs(os.path.dirname(OUT), exist_ok=True)
    with open(OUT, "w", encoding="utf-8", newline="\n") as fh:
        fh.write(text)
    print("wrote packs/base-starters/pack.json")
    return 0


if __name__ == "__main__":
    sys.exit(main())
