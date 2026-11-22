#!/usr/bin/env python3
"""The catalogued Solar System: packs/base-sol/pack.json.

Sol as measured, for the start sheet's "Open the Solar System" door and any
reader that needs real Sol bodies. These are body facts, not the master data
sheet's reference objects: the sheet's Sun is a yardstick for units, this Sun
is a star in a system. A check proves the two agree on the Sun's nominal
values, so neither can drift.

Every value is read from the source beside it. NSSDCA fact sheet numbers are
copied as printed (22 Nov 2026) and turned into SI here. Values worked out
from others say how. Field names are the kernel's (body.*, orbit.*, star.*).
Python standard library only; deterministic.

  scripts/build-base-sol.py          write the pack
  scripts/build-base-sol.py --check  exit 1 if the pack is stale or disagrees
                                     with packs/base-reference
"""
import json
import os
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(ROOT, "packs", "base-sol", "pack.json")
SHEET = os.path.join(ROOT, "packs", "base-reference", "pack.json")

NS = "https://nssdc.gsfc.nasa.gov/planetary/factsheet/%sfact.html"
HOUR = 3600.0

SOURCES = [
    {"id": "nssdca-%s" % n, "text": "Williams, D. R., NASA NSSDCA %s Fact Sheet (read 22 Nov 2026)" % t,
     "status": "verified", "url": NS % n}
    for n, t in [("sun", "Sun"), ("mercury", "Mercury"), ("venus", "Venus"), ("earth", "Earth"), ("moon", "Moon"),
                 ("mars", "Mars"), ("jupiter", "Jupiter"), ("saturn", "Saturn"), ("uranus", "Uranus"),
                 ("neptune", "Neptune"), ("pluto", "Pluto"), ("asteroid", "Asteroid")]
] + [
    {"id": "iau2015b3", "text": "Prša, A. et al. 2016, AJ 152, 41: Nominal values for selected solar and planetary quantities, IAU 2015 Resolution B3",
     "status": "verified", "doi": "10.3847/0004-6256/152/2/41"},
]

# name, id, kind, host, mass (1e24 kg), mean radius (km), a (1e6 km), e,
# sidereal rotation (h, as printed: negative = retrograde), Bond albedo,
# geometric albedo, obliquity to orbit (deg). From the NSSDCA sheet named.
PLANETS = [
    ("Mercury", "mercury", "planet", "sun", 0.33010, 2439.7, 57.909, 0.2056, 1407.6, 0.068, 0.142, 0.034),
    ("Venus", "venus", "planet", "sun", 4.8673, 6051.8, 108.210, 0.0068, -5832.6, 0.77, 0.689, 177.36),
    ("Earth", "earth", "planet", "sun", 5.9722, 6371.000, 149.598, 0.0167, 23.9345, 0.294, 0.434, 23.44),
    ("Moon", "moon", "moon", "earth", 0.07346, 1737.4, 0.3844, 0.0549, 655.720, 0.11, 0.12, 6.68),
    ("Mars", "mars", "planet", "sun", 0.64169, 3389.5, 227.956, 0.0935, 24.6229, 0.250, 0.170, 25.19),
    ("Jupiter", "jupiter", "planet", "sun", 1898.13, 69911, 778.479, 0.0487, 9.9250, 0.343, 0.538, 3.13),
    ("Saturn", "saturn", "planet", "sun", 568.32, 58232, 1432.041, 0.0520, 10.656, 0.342, 0.499, 26.73),
    ("Uranus", "uranus", "planet", "sun", 86.811, 25362, 2867.043, 0.0469, -17.24, 0.300, 0.488, 97.77),
    ("Neptune", "neptune", "planet", "sun", 102.409, 24622, 4514.953, 0.0097, 16.11, 0.290, 0.442, 28.32),
    ("Pluto", "pluto", "dwarf_planet", "sun", 0.01303, 1188, 5869.656, 0.2444, -153.2928, 0.72, 0.52, 119.51),
]
NOTES = {
    "jupiter": "rotation in System III (1965.0)",
    "saturn": "rotation in Saturnian System III",
    "uranus": "rotation from Voyager 2 radio science (magnetic coordinates)",
    "neptune": "rotation from Voyager 2 radio science (magnetic coordinates)",
    "moon": "semi-major axis about Earth",
}


def val(v, unit, src, **kw):
    out = {"v": v, "unit": unit, "src": src}
    out.update(kw)
    return out


def planet_row(name, pid, kind, host, m24, r_km, a6, e, rot_h, bond, geo, obl):
    src = "nssdca-" + pid
    values = {
        "body.mass": val(m24 * 1e24, "kg", src),
        "body.radius": val(r_km * 1e3, "m", src, note="volumetric mean radius"),
        "orbit.semi_major_axis": val(a6 * 1e9, "m", src),
        "orbit.eccentricity": val(e, "1", src),
        "body.rotation_period": val(abs(rot_h) * HOUR, "s", src, note="sidereal; spin sense is in body.obliquity"),
        "body.obliquity": val(obl, "deg", src, note="obliquity to orbit; above 90 means retrograde spin"),
        "body.albedo": val(bond, "1", src, note="Bond albedo"),
        "body.albedo_geometric": val(geo, "1", src),
    }
    row = {"id": pid, "name": name, "kind": kind, "host": host, "values": values}
    if pid in NOTES:
        row["note"] = NOTES[pid]
    return row




def build():
    sys.path.insert(0, os.path.join(ROOT, "scripts"))
    import sol_extra  # the Sun's age, Ceres and the regions, each with its source
    sun = {"id": "sun", "name": "Sun", "kind": "star", "host": None, "values": {
        "body.mass": val(1988400 * 1e24, "kg", "nssdca-sun"),
        "body.radius": val(695700 * 1e3, "m", "nssdca-sun", note="volumetric mean radius; equals the IAU nominal radius"),
        "star.luminosity": val(382.8 * 1e24, "W", "nssdca-sun", note="equals the IAU nominal luminosity"),
        "star.effective_temperature": val(5772.0, "K", "nssdca-sun", note="equals the IAU nominal temperature"),
    }}
    sun["values"].update(sol_extra.SUN_AGE)
    bodies = [sun] + [planet_row(*p) for p in PLANETS] + sol_extra.BODIES
    sources = sorted(SOURCES + sol_extra.SOURCES, key=lambda s: s["id"])
    regions = sol_extra.regions({b["id"]: b for b in bodies})
    return {
        "format": "sse.pack/1", "id": "sse.base-sol", "version": "0.1.0", "kind": ["base-physics"],
        "kernel_range": "^0.1", "licence": "Apache-2.0",
        "provides": {"other": {"sol": {
            "about": "The catalogued Solar System: the Sun, the planets, the Moon, Ceres and Pluto, and the belts and the Oort cloud, as measured. Built by scripts/build-base-sol.py; every value names its source. Body facts, not the master data sheet's reference objects.",
            "bodies": bodies, "regions": regions, "sources": sources,
        }}},
    }


def validate(pack):
    sol = pack["provides"]["other"]["sol"]
    ids = {s["id"] for s in sol["sources"]}
    assert len(ids) == len(sol["sources"]), "duplicate source id"
    for s in sol["sources"]:
        assert s.get("doi") or s.get("url"), "source %s has no DOI or URL" % s["id"]
    bids = {b["id"] for b in sol["bodies"]}
    for b in sol["bodies"]:
        assert b["host"] is None or b["host"] in bids, "%s: unknown host %s" % (b["id"], b["host"])
        for k, v in b["values"].items():
            assert v["src"] in ids, "%s %s: unknown source %s" % (b["id"], k, v["src"])
            assert isinstance(v["v"], (int, float)) and v["v"] == v["v"], "%s %s: not a number" % (b["id"], k)
    for r in sol["regions"]:
        assert r["host"] in bids and r["src"] in ids and 0 < r["inner_m"] < r["outer_m"], r["id"]
    # The sheet's yardsticks and this Sun must agree.
    ref = json.load(open(SHEET, encoding="utf-8"))["provides"]["reference"]["constants"]
    c = {x["id"]: x["value"]["v"] for x in ref}
    sun = next(b for b in sol["bodies"] if b["id"] == "sun")["values"]
    for field, cid, tol in [("body.radius", "R_sun", 0.0), ("star.luminosity", "L_sun", 0.0),
                            ("star.effective_temperature", "T_sun", 0.0), ("body.mass", "M_sun", 2e-4)]:
        rel = abs(sun[field]["v"] - c[cid]) / c[cid]
        assert rel <= tol + 1e-12, "Sun %s differs from the sheet's %s by %.2e" % (field, cid, rel)
    earth = next(b for b in sol["bodies"] if b["id"] == "earth")["values"]
    for field, cid in [("body.mass", "M_earth"), ("body.radius", "R_earth_mean")]:
        rel = abs(earth[field]["v"] - c[cid]) / c[cid]
        assert rel <= 2e-4, "Earth %s differs from the sheet's %s by %.2e" % (field, cid, rel)


def main():
    pack = build()
    validate(pack)
    text = json.dumps(pack, indent=1, ensure_ascii=False, allow_nan=False) + "\n"
    if "--check" in sys.argv:
        cur = open(OUT, encoding="utf-8").read() if os.path.exists(OUT) else ""
        if cur != text:
            print("packs/base-sol is stale: run scripts/build-base-sol.py")
            return 1
        print("base-sol pack is current")
        return 0
    os.makedirs(os.path.dirname(OUT), exist_ok=True)
    with open(OUT, "w", encoding="utf-8", newline="\n") as fh:
        fh.write(text)
    print("wrote packs/base-sol/pack.json (%d bodies, %d regions)" % (len(pack["provides"]["other"]["sol"]["bodies"]), len(pack["provides"]["other"]["sol"]["regions"])))
    return 0


if __name__ == "__main__":
    sys.exit(main())
