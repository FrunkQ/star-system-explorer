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
import math
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
    {"id": "hipparcos1997", "text": "ESA 1997, The Hipparcos and Tycho Catalogues, ESA SP-1200, vol. 1, sec. 1.5.3: the ICRS to galactic matrix A_G",
     "status": "verified", "url": "https://www.cosmos.esa.int/documents/532822/552851/vol1_all.pdf"},
    {"id": "hilton2006", "text": "Hilton, J. L. et al. 2006, Celest. Mech. Dyn. Astron. 94, 351: Report of the IAU Division I Working Group on Precession and the Ecliptic (obliquity at J2000, 84381.406 arcsec)",
     "status": "verified", "doi": "10.1007/s10569-006-0001-2"},
    {"id": "charette-smith2010", "text": "Charette, M. A. and Smith, W. H. F. 2010, Oceanography 23(2), 112: The volume of Earth's ocean (1.335e9 km3, the size of one Earth ocean as the kernel counts it; Earth holds 1 by definition)",
     "status": "unverified", "doi": "10.5670/oceanog.2010.51"},
]

# ICRS to galactic (Hipparcos A_G, rows are galactic x, y, z) and the J2000
# obliquity: the Sol fixture's elements are J2000 ecliptic, so its frame in
# the galaxy is the ecliptic pole and the equinox turned into l and b.
A_G = [(-0.0548755604, -0.8734370902, -0.4838350155),
       (+0.4941094279, -0.4448296300, +0.7469822445),
       (-0.8676661490, -0.1980763734, +0.4559837762)]
OBLIQUITY_ARCSEC = 84381.406


def galactic(v):
    g = [sum(r[i] * v[i] for i in range(3)) for r in A_G]
    l = math.degrees(math.atan2(g[1], g[0])) % 360.0
    return round(l, 4), round(math.degrees(math.asin(g[2])), 4)


def sun_frame():
    e = math.radians(OBLIQUITY_ARCSEC / 3600.0)
    pl, pb = galactic((0.0, -math.sin(e), math.cos(e)))
    zl, zb = galactic((1.0, 0.0, 0.0))
    note = "J2000 ecliptic %s in galactic coordinates, from the obliquity (hilton2006) and A_G"
    return {
        "system.frame.pole_l_deg": val(pl, "deg", "hipparcos1997", note=note % "north pole"),
        "system.frame.pole_b_deg": val(pb, "deg", "hipparcos1997", note=note % "north pole"),
        "system.frame.zero_l_deg": val(zl, "deg", "hipparcos1997", note=note % "equinox (longitude zero)"),
        "system.frame.zero_b_deg": val(zb, "deg", "hipparcos1997", note=note % "equinox (longitude zero)"),
    }

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
    values.update(surface_facts(pid, src))
    values.update(composition_facts(pid, src))
    values.update(interior_facts(pid))
    if pid in NOTES:
        row["note"] = NOTES[pid]
    return row


# What a catalogued world's surface is like, as the NSSDCA sheet prints it
# (read 23 Nov 2026), for the surface chain to use before any estimate
# (proven core, test drive). Only a printed average is kept: the Moon and Pluto
# print a range, so they have no temperature here; Mercury's and the Moon's
# surface pressures are printed as upper bounds of an exosphere, so they have
# none either. atmosphere.main is the main gas by volume, as a substance id on
# the master data sheet. Fields: average surface temperature (K), surface
# pressure (bar), main gas.
SURFACE = {
    "mercury": (440.0, None, None),
    "venus": (737.0, 92.0, "co2"),
    "earth": (288.0, 1.014, "n2"),
    "mars": (214.0, 6.36e-3, "co2"),
    "pluto": (None, 13e-6, "n2"),
}
SURFACE_NOTES = {
    "mars": "average; NSSDCA prints ~214 K",
    "mars_p": "at mean radius; varies 4.0 to 8.7 mb with the season",
    "pluto_p": "NSSDCA prints ~13 microbar",
    "venus_gas": "96.5% CO2, 3.5% N2 near the surface, by volume",
    "earth_gas": "78.08% N2, 20.95% O2, dry air by volume",
    "mars_gas": "95.1% CO2, 2.59% N2, 1.94% Ar by volume",
    "pluto_gas": "99% N2, 0.5% CH4 by volume",
}
BAR = 1e5


def surface_facts(pid, src):
    out = {}
    if pid == "earth":
        out["body.surface_water_oceans"] = val(1.0, "earth_oceans", "charette-smith2010",
                                               note="1 by definition of the unit (1.335e18 m3 of ocean)")
    if pid not in SURFACE:
        return out
    t, p_bar, gas = SURFACE[pid]
    if t is not None:
        out["planet.surface_temperature"] = val(t, "K", src, note=SURFACE_NOTES.get(pid, "average temperature"))
    if p_bar is not None:
        out["atmosphere.surface_pressure"] = val(round(p_bar * BAR, 9), "Pa", src, **(
            {"note": SURFACE_NOTES[pid + "_p"]} if pid + "_p" in SURFACE_NOTES else {}))
    if gas is not None:
        out["atmosphere.main"] = val(gas, "substance", src, note=SURFACE_NOTES[pid + "_gas"])
    return out

# The air's make-up by volume (mole fraction), gas by gas, as the NSSDCA sheets
# print it (read 23 Nov 2026), keyed by substance id on the master data sheet:
# one row per gas, atmosphere.composition.<id>, unit 1. Shares printed in ppm are
# converted here. Gases the sheet has no substance for are left out and named in
# the level note. The giants' make-up is at the 1 bar level (no surface).
PPM = 1e-6
COMPOSITION = {
    "earth": ({"n2": 0.7808, "o2": 0.2095, "ar": 9340 * PPM, "co2": 420 * PPM, "ne": 18.18 * PPM, "he": 5.24 * PPM,
               "ch4": 1.94 * PPM, "kr": 1.14 * PPM, "h2": 0.55 * PPM},
              "dry air; water is highly variable, typically about 1%"),
    "venus": ({"co2": 0.965, "n2": 0.035, "so2": 150 * PPM, "ar": 70 * PPM, "h2o": 20 * PPM, "co": 17 * PPM,
               "he": 12 * PPM, "ne": 7 * PPM}, "near the surface"),
    "mars": ({"co2": 0.951, "n2": 0.0259, "ar": 0.0194, "o2": 0.0016, "co": 0.0006, "h2o": 210 * PPM,
              "ne": 2.5 * PPM, "kr": 0.3 * PPM, "xe": 0.08 * PPM}, "nitric oxide (100 ppm) and HDO (0.85 ppm) left out: not on the sheet"),
    "pluto": ({"n2": 0.99, "ch4": 0.005, "co": 0.0005}, "trace HCN and C2Hx hydrocarbons left out"),
    "jupiter": ({"h2": 0.898, "he": 0.102, "ch4": 3000 * PPM, "nh3": 260 * PPM, "h2o": 4 * PPM},
                "at the 1 bar level; HD (28 ppm) and ethane (5.8 ppm) left out; water varies with pressure"),
    "saturn": ({"h2": 0.963, "he": 0.0325, "ch4": 4500 * PPM, "nh3": 125 * PPM},
               "at the 1 bar level; HD (110 ppm) and ethane (7 ppm) left out"),
    "uranus": ({"h2": 0.825, "he": 0.152, "ch4": 0.023}, "at the 1 bar level; HD (148 ppm) left out"),
    "neptune": ({"h2": 0.80, "he": 0.19, "ch4": 0.015}, "at the 1 bar level; HD (192 ppm) and ethane (1.5 ppm) left out"),
}


# Rocky worlds' measured interiors (proven core, 24 Nov 2026): the core's share
# of the mass, and the moment of inertia factor C = I/(M R^2). Recalled from the
# papers named, not yet re-read against them, so every value is marked
# unverified. Venus has no measured core, so it has no core share here.
INTERIOR = {
    "mercury": ((0.69, "hauck2013", "from a core radius of 2020 +/- 30 km"), (0.346, "margot2012")),
    "venus": (None, (0.337, "margot2021")),
    "earth": ((0.325, "mcdonough2003", "the core is 32.5% of Earth's mass"), (0.3307, "yoder1995")),
    "moon": ((0.015, "weber2011", "from a core radius of about 330 km"), (0.3932, "konopliv1998")),
    "mars": ((0.24, "staehler2021", "from a core radius of 1830 +/- 40 km and a core density near 6000 kg/m3"), (0.3644, "konopliv2011")),
}
INTERIOR_SOURCES = [
    {"id": "hauck2013", "text": "Hauck, S. A. et al. 2013, JGR Planets 118, 1204: the curious case of Mercury's internal structure (core radius 2020 +/- 30 km)", "status": "unverified", "doi": "10.1002/jgre.20091"},
    {"id": "margot2012", "text": "Margot, J.-L. et al. 2012, JGR Planets 117, E00L09: Mercury's moment of inertia from spin and gravity data (C/MR^2 = 0.346 +/- 0.014)", "status": "unverified", "doi": "10.1029/2012JE004161"},
    {"id": "margot2021", "text": "Margot, J.-L. et al. 2021, Nature Astronomy 5, 676: spin state and moment of inertia of Venus (C/MR^2 = 0.337 +/- 0.024)", "status": "unverified", "doi": "10.1038/s41550-021-01339-7"},
    {"id": "mcdonough2003", "text": "McDonough, W. F. 2003, Treatise on Geochemistry 2, 547: compositional model for the Earth's core (the core is 32.5% of Earth's mass)", "status": "unverified", "doi": "10.1016/B0-08-043751-6/02015-6"},
    {"id": "yoder1995", "text": "Yoder, C. F. 1995, Astrometric and geodetic properties of Earth and the Solar System, AGU Reference Shelf 1, 1 (Earth C/MR^2 = 0.3307)", "status": "unverified", "doi": "10.1029/RF001p0001"},
    {"id": "weber2011", "text": "Weber, R. C. et al. 2011, Science 331, 309: seismic detection of the lunar core (radius about 330 km)", "status": "unverified", "doi": "10.1126/science.1199375"},
    {"id": "konopliv1998", "text": "Konopliv, A. S. et al. 1998, Science 281, 1476: improved gravity field of the Moon from Lunar Prospector (C/MR^2 = 0.3932 +/- 0.0002)", "status": "unverified", "doi": "10.1126/science.281.5382.1476"},
    {"id": "staehler2021", "text": "Staehler, S. C. et al. 2021, Science 373, 443: seismic detection of the martian core (radius 1830 +/- 40 km)", "status": "unverified", "doi": "10.1126/science.abi7730"},
    {"id": "konopliv2011", "text": "Konopliv, A. S. et al. 2011, Icarus 211, 401: Mars high resolution gravity fields from MRO, Mars seasonal gravity and other dynamical parameters (C/MR^2 = 0.3644 +/- 0.0005)", "status": "unverified", "doi": "10.1016/j.icarus.2010.10.004"},
]


def interior_facts(pid):
    if pid not in INTERIOR:
        return {}
    core, moi = INTERIOR[pid]
    out = {}
    if core is not None:
        out["interior.core_mass_fraction"] = val(core[0], "1", core[1], note=core[2], check="unverified")
    out["body.moment_of_inertia_factor"] = val(moi[0], "1", moi[1], note="C = I/(M R^2)", check="unverified")
    return out


def composition_facts(pid, src):
    if pid not in COMPOSITION:
        return {}
    mix, where = COMPOSITION[pid]
    return {"atmosphere.composition." + g: val(float(f"{x:.6g}"), "1", src, note="by volume, " + where)
            for g, x in mix.items()}


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
    sun["values"].update(sun_frame())
    bodies = [sun] + [planet_row(*p) for p in PLANETS] + sol_extra.BODIES
    sources = sorted(SOURCES + INTERIOR_SOURCES + sol_extra.SOURCES, key=lambda s: s["id"])
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
    sheet = json.load(open(SHEET, encoding="utf-8"))["provides"]["reference"]
    subs = {x["id"] for x in sheet["substances"]}
    for b in sol["bodies"]:
        assert b["host"] is None or b["host"] in bids, "%s: unknown host %s" % (b["id"], b["host"])
        for k, v in b["values"].items():
            assert v["src"] in ids, "%s %s: unknown source %s" % (b["id"], k, v["src"])
            if k.startswith("atmosphere.composition."):
                assert k.rsplit(".", 1)[1] in subs, "%s %s: not a substance on the sheet" % (b["id"], k)
                assert 0.0 < v["v"] <= 1.0, (b["id"], k)
            if v["unit"] == "substance":
                assert v["v"] in subs, "%s %s: %s is not a substance on the sheet" % (b["id"], k, v["v"])
                continue
            assert isinstance(v["v"], (int, float)) and v["v"] == v["v"], "%s %s: not a number" % (b["id"], k)
    for b in sol["bodies"]:
        mix = [v["v"] for k, v in b["values"].items() if k.startswith("atmosphere.composition.")]
        assert not mix or 0.99 <= sum(mix) <= 1.01, "%s: air shares sum to %.4f" % (b["id"], sum(mix))
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
