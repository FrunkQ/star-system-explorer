#!/usr/bin/env python3
"""Turn SSE 3.1's starter-sf classification.json into the SSE4 "SSE 3 classic"
scheme pack (packs/sse3-classic/pack.json). Classification design section 11:
every V3 class id is kept, `match` bands become region cores in the same
order (the classic scorer sums them in that order), `gate`, `weight`,
`note`, `range`, `formation` and images carry over.

    python3 scripts/import-v3-classes.py <v3>/static/rulepacks/starter-sf/classification.json
"""
import json
import sys
from pathlib import Path

SRC = Path(sys.argv[1])
OUT = Path(__file__).resolve().parent.parent / "packs/sse3-classic/pack.json"
TOL = 0.15
# SSE 3's own constants (src/lib/constants.ts) were Earth 5.972e24 kg and
# 6.371e6 m, the au and the Julian Gyr; the pack divides by the master data
# sheet's instead (MP22), written as sheet references the loader resolves.
ME = {"sheet": "units/m_earth"}
RE = {"sheet": "constants/R_earth_mean"}
AU = {"sheet": "units/au"}
GYR = {"sheet": "units/gyr"}
DAY = {"sheet": "units/day"}

# V3 feature -> SSE4 fact (and unit change) where a V4 flow publishes it.
MAPPED = {
    "mass_Me": ("body.mass", ME, "Earth masses"),
    "radius_Re": ("body.radius", RE, "Earth radii"),
    "density": ("body.bulk_density", 1000.0, "g/cm3"),
    "escapeVelocity_kms": ("body.escape_velocity", 1000.0, "km/s"),
    "a_AU": ("orbit.semi_major_axis", AU, "au"),
    "eccentricity": ("orbit.eccentricity", None, ""),
    "Teq_K": ("planet.equilibrium_temperature", None, "K"),
    "SurfaceTemp_K": ("planet.surface_temperature", None, "K"),
    "orbital_period_days": ("orbit.period", DAY, "days"),
    "atm.pressure_bar": ("atmosphere.surface_pressure", 1e5, "bar"),
    "age_Gyr": ("system.age", GYR, "Gyr"),
    "parentId": ("parent", None, ""),
}
# SSE 3 fills these when a body has none (SystemProcessor feature map), so
# airless and dry bodies can match; the classic scheme reproduces that.
MISSING = {
    "parentId": "", "a_AU": 0, "eccentricity": 0, "orbitsStar": 0, "Teq_K": 0,
    "orbital_period_days": 0, "starTidallyLocked": 0, "atm.main": "None",
    "atm.pressure_bar": 0, "hydrosphere.coverage": 0, "hydrosphere.composition": "none",
    "hydrosphere.liquidCoverage": 0, "hasSubsurfaceOcean": 0, "hasBiosphere": 0,
    "has_ring_child": 0, "settledCover": 0,
}
LOG = {"mass_Me", "radius_Re", "density", "a_AU", "orbital_period_days", "escapeVelocity_kms", "atm.pressure_bar"}
FLOURISH = {"asteroid/contact-binary", "planet/ringed"}
SPECULATIVE = {"planet/carbon", "planet/helium", "planet/hycean", "planet/superhabitable", "planet/chthonian", "planet/toroidal", "planet/coreless"}
FICTIONAL = {"planet/ecumenopolis"}


def is_cat(b):
    return isinstance(b, str) or (isinstance(b, list) and any(isinstance(x, str) for x in b[:2]))


def bound(axis, b):
    if is_cat(b):
        return {"axis": axis, "any_of": [b] if isinstance(b, str) else b}
    lo, hi = b
    rlo = abs(lo) if lo != 0 else (abs(hi) if hi != 0 else 1)
    rhi = abs(hi) if hi != 0 else 1
    return {"axis": axis, "core": [lo, hi], "fringe": [lo - TOL * rlo, hi + TOL * rhi]}


def nice(cid):
    return cid.split("/", 1)[1].replace("-", " ")


v3 = json.loads(SRC.read_text())["classifier"]
fps = v3["fingerprints"]
images = v3.get("planetImages", {})

spans = {}
for fp in fps:
    for k, b in list(fp["match"].items()) + list(fp.get("gate", {}).items()):
        if is_cat(b):
            spans.setdefault(k, None)
        else:
            lo, hi = spans.get(k) or (b[0], b[1])
            spans[k] = (min(lo, b[0]), max(hi, b[1]))

axes = []
for k in sorted(spans):
    field, div, unit = MAPPED.get(k, (k, None, ""))
    cat = spans[k] is None
    a = {"field": field, "scale": "category" if cat else ("log" if k in LOG else "linear"), "label": k}
    if not cat:
        a["span"] = list(spans[k])
    if unit:
        a["unit"] = unit
    if div:
        a["divide_by"] = div
    if k in MISSING:
        a["missing"] = MISSING[k]
    if k not in MAPPED:
        a["provisional"] = True
    axes.append([k, a])

classes = []
for fp in fps:
    c = {"id": fp["class"], "scheme": "sse3-classic", "kind": fp["kind"],
         "region": [bound(k, b) for k, b in fp["match"].items()]}
    if fp.get("gate"):
        c["gate"] = [bound(k, b) for k, b in fp["gate"].items()]
    if "weight" in fp:
        c["weight"] = fp["weight"]
    c["names"] = {"physics": nice(fp["class"]), "plain": nice(fp["class"]), "story": ""}
    if fp["class"] in FLOURISH:
        c["meaning"] = "flourish"
    if fp.get("range"):
        c["typical"] = [{"axis": k, "centre": (lo + hi) / 2, "spread": (hi - lo) / 2} for k, (lo, hi) in fp["range"].items()]
    if fp.get("formation"):
        c["generate"] = [bound(k, b) for k, b in fp["formation"].items()]
    if fp["class"] in images:
        c["look"] = {"image": {"file": images[fp["class"]], "licence": "", "credit": "SSE 3 starter-sf pack"}}
    if fp.get("note"):
        c["learn"] = {"summary": fp["note"]}
    c["source"] = "SSE 3.1 starter-sf classification.json"
    if fp["class"] in FICTIONAL:
        c["evidence"] = "fictional"
    elif fp["class"] in SPECULATIVE:
        c["evidence"] = "speculative"
    classes.append(c)


def facts(**v3):
    """An example's values as SSE4 facts, from SSE 3 feature values."""
    out = {}
    for k, x in v3.items():
        k = k.replace("__", ".")
        field, div, _ = MAPPED.get(k, (k, None, ""))
        out[field] = x * div if div else x
    return out


# SSE 3's own real-body cases (classification.spec.ts), as the classes' examples.
EX = {
    "Bennu": (facts(mass_Me=1.2e-14, radius_Re=3.9e-5, density=1.19, porosity=0.55, makeup__carbon=0.5, makeup__rock=0.5, makeup__metal=0, makeup__ice=0, makeup__gas=0),
              {"asteroid/c-type": "in", "asteroid/rubble-pile": "in"}),
    "Eros": (facts(mass_Me=1.12e-9, radius_Re=1.3e-3, density=2.67, porosity=0.1, makeup__rock=0.85, makeup__metal=0.15, makeup__carbon=0, makeup__ice=0, makeup__gas=0),
             {"asteroid/s-type": "in", "asteroid/rubble-pile": "out"}),
    "16 Psyche": (facts(mass_Me=3.8e-6, radius_Re=0.017, density=3.9, porosity=0.05, makeup__metal=0.6, makeup__rock=0.4, makeup__carbon=0, makeup__ice=0, makeup__gas=0),
                  {"asteroid/m-type": "in"}),
    "67P/Churyumov-Gerasimenko": (facts(mass_Me=1.7e-12, radius_Re=3.1e-4, density=0.53, porosity=0.7, makeup__ice=0.55, makeup__carbon=0.25, makeup__rock=0.2, makeup__metal=0, makeup__gas=0),
                                  {"asteroid/comet": "in"}),
    "Ceres": (facts(mass_Me=1.57e-4, radius_Re=0.0737, density=2.16, porosity=0, makeup__rock=0.7, makeup__ice=0.3, makeup__metal=0, makeup__carbon=0, makeup__gas=0),
              {"asteroid/c-type": "out", "asteroid/s-type": "out", "asteroid/m-type": "out", "asteroid/comet": "out"}),
    "GJ 876 b": (facts(starTidallyLocked=1, orbitsStar=1, makeup__gas=0.85, Teq_K=174, SurfaceTemp_K=174, radius_Re=12, density=1.2, mass_Me=700),
                 {"planet/cold-eyeball": "out", "planet/eyeball": "out", "planet/hot-eyeball": "out"}),
}
for c in classes:
    for name, (vals, want) in EX.items():
        if c["id"] in want:
            c.setdefault("examples", []).append({"name": name, "values": vals, "expect": want[c["id"]]})

scheme = {"id": "sse3-classic", "question": "What did SSE 3 call it?", "scoring": "classic",
          "tolerance": TOL, "max_classes": v3.get("maxClasses", 4), "borderline": 0.9,
          "fallback": {"axis": "mass_Me", "above": 10, "high": "planet/gas-giant", "low": "planet/terrestrial", "unknown": "planet/unclassified"}}

pack = {
    "format": "sse.pack/1", "id": "sse.sse3-classic", "version": "0.1.0", "kind": ["presentation"],
    "kernel_range": "^0.1", "licence": "Apache-2.0",
    "provides": {"other": {
        "about": "SSE 3's 72 classes as the 'SSE 3 classic' scheme, so imported V3 saves keep the labels their authors knew. Generated by scripts/import-v3-classes.py from SSE 3.1 starter-sf classification.json.",
        "schemes": [scheme], "axes": axes, "classes": classes}},
}
OUT.write_text(json.dumps(pack, indent=1) + "\n")
print(f"{len(classes)} classes, {len(axes)} axes -> {OUT}")
