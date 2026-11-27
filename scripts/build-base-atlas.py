#!/usr/bin/env python3
"""Build packs/base-atlas/pack.json: SSE4's shipped physical atlas (C2,
classification design sections 6 and 10). Region scoring: a class is a core
(fully in) and a fringe (fading out) on each axis it names; the lowest fit
decides; the deepest fitting class wins. Four schemes:

  physical      what kind of object this is (bulk families; giant weather
                variants; stacking modifiers)
  surface       what its surface is like (for bodies with one)
  air           its gas mix (base) and pressure band (modifiers)
  habitability  requirement profiles for humans, nested from "sealed
                habitat" to "shirt-sleeve"

Examples come from tests/fixtures/classify/{solar-system,exoplanets}.json
(their `expect` blocks) and a few illustrative bodies built here, named
"illustrative: ...". Run after editing either:

    python3 scripts/build-base-atlas.py
"""
import copy
import sys
import math
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
# The yardsticks come from the master data sheet, and the Solar System's own
# bodies from the catalogued Sol (packs/base-sol), so this pack types no
# constant of its own (MP22; scripts/audit-matter.py reads this script in the
# pack's place).
SHEET = {c["id"]: c["value"]["v"] for c in json.loads((ROOT / "packs/base-reference/pack.json").read_text())["provides"]["reference"]["constants"]}
SOL = {b["name"]: b for b in json.loads((ROOT / "packs/base-sol/pack.json").read_text())["provides"]["other"]["sol"]["bodies"]}
ME, RE, G0, GYR = SHEET["M_earth"], SHEET["R_earth_mean"], SHEET["g0"], 1e9 * SHEET["yr_julian"]
INF = 1e9


def ax(field, scale, span=None, label="", unit="", div=None, prov=False):
    a = {"field": field, "scale": scale, "label": label or field}
    if span:
        a["span"] = list(span)
    if unit:
        a["unit"] = unit
    if div:
        a["divide_by"] = div
    if prov:
        a["provisional"] = True
    return a


AXES = {
    "mass": ax("body.mass", "log", (1e-15, 3e4), "mass", "Earth masses", ME),
    "radius": ax("body.radius", "log", (1e-5, 30), "radius", "Earth radii", RE),
    "density": ax("body.bulk_density", "linear", (0, 30), "bulk density", "g/cm3", 1000.0),
    "gravity": ax("body.surface_gravity", "linear", (0, 10), "surface gravity", "g", G0),
    "teq": ax("planet.equilibrium_temperature", "linear", (0, 5000), "equilibrium temperature", "K"),
    "tsurf": ax("planet.surface_temperature", "linear", (0, 5000), "mean surface temperature", "K"),
    "pressure": ax("atmosphere.surface_pressure", "linear", (0, 1e4), "surface pressure", "bar", 1e5),
    "ecc": ax("orbit.eccentricity", "linear", (0, 1), "eccentricity"),
    "age": ax("system.age", "linear", (0, 14), "age", "Gyr", GYR),
    "metal": ax("makeup.metal", "fraction", (0, 1), "metal fraction", prov=True),
    "rock": ax("makeup.rock", "fraction", (0, 1), "rock fraction", prov=True),
    "carbon": ax("makeup.carbon", "fraction", (0, 1), "carbon fraction", prov=True),
    "ice": ax("makeup.ice", "fraction", (0, 1), "water and ice fraction", prov=True),
    "gas": ax("makeup.gas", "fraction", (0, 1), "hydrogen-helium envelope fraction", prov=True),
    "porosity": ax("body.porosity", "fraction", (0, 1), "macroporosity", prov=True),
    "lobes": ax("shape.lobes", "linear", (1, 8), "lobes", prov=True),
    "ice_cover": ax("surface.ice_fraction", "fraction", (0, 1), "ice cover", prov=True),
    "liquid_fraction": ax("surface.liquid_fraction", "fraction", (0, 1), "surface liquid cover", prov=True),
    "liquid": ax("surface.liquid", "category", label="surface liquid", prov=True),
    "subsurface_ocean": ax("interior.subsurface_ocean", "flag", label="ocean under the surface", prov=True),
    "air_main": ax("atmosphere.main", "category", label="main gas", prov=True),
    "pp_o2": ax("atmosphere.pp.o2", "linear", (0, 100), "oxygen partial pressure", "bar", prov=True),
    "pp_co2": ax("atmosphere.pp.co2", "linear", (0, 100), "carbon dioxide partial pressure", "bar", prov=True),
    "dose": ax("surface.radiation_dose", "linear", (0, 1e7), "surface radiation dose", "mSv/yr", prov=True),
    "field": ax("magnetosphere.field", "linear", (0, 1000), "surface magnetic field", "uT", prov=True),
    "star_locked": ax("spin.star_locked", "flag", label="tidally locked to its star", prov=True),
    "rings": ax("rings", "flag", label="has rings", prov=True),
    "volcanic": ax("activity.volcanic", "fraction", (0, 1), "volcanic activity", prov=True),
    "tidal_heat": ax("interior.tidal_heat_flux", "linear", (0, 100), "tidal heat flow", "W/m2", prov=True),
    "host": ax("orbit.host_kind", "category", label="what it orbits", prov=True),
    "origin": ax("origin", "category", label="origin (authored)", prov=True),
    "terraformed": ax("history.terraformed", "flag", label="terraformed (authored)", prov=True),
    "life": ax("life.stage", "category", label="biosphere stage", prov=True),
    "population": ax("settlements.population", "linear", (0, 1e12), "population", prov=True),
    # The star scheme reads the kernel's star class (sse_formation::spectral),
    # published for every star at the date read.
    "star_family": ax("star.class_family", "category", label="star family (from temperature, gravity and phase)"),
    "star_kind": ax("star.class_kind", "category", label="spectral letter or remnant kind"),
}


def n(axis, lo, hi, flo=None, fhi=None):
    return {"axis": axis, "core": [lo, hi], "fringe": [lo if flo is None else flo, hi if fhi is None else fhi]}


def cat(axis, *vals):
    return {"axis": axis, "any_of": list(vals)}


def flag(axis, v):
    return {"axis": axis, "is": v}


CLASSES = []


def cls(cid, scheme, region, kind="base", parent=None, gate=None, summary="", evidence="science",
        meaning="identity", plain=None, typical=None):
    c = {"id": cid, "scheme": scheme, "kind": kind, "region": region}
    if parent:
        c["parent"] = parent
    if gate:
        c["gate"] = gate
    name = cid.split("/")[-1].replace("-", " ")
    c["names"] = {"physics": name, "plain": plain or name, "story": ""}
    if meaning != "identity":
        c["meaning"] = meaning
    if typical:
        c["typical"] = [{"axis": a, "centre": m, "spread": s} for a, m, s in typical]
    c["learn"] = {"summary": summary}
    c["source"] = "sse.base-atlas"
    if evidence != "science":
        c["evidence"] = evidence
    CLASSES.append(c)


SMALL = n("mass", 1e-15, 1e-5, 1e-16, 1.5e-5)
SOLID = [n("gas", 0, 0.005, 0, 0.01)]

# ---- physical: bulk families (section 10, "by bulk") ----
P = "physical"
cls("small-body", P, [SMALL], summary="Too small to pull itself round: an asteroid, comet or small moon.")
cls("small-body/carbonaceous", P, [n("carbon", 0.2, 1, 0.15, 1), n("ice", 0, 0.35, 0, 0.4)], parent="small-body",
    summary="Dark, carbon-rich and primitive (C-type), like Bennu.")
cls("small-body/icy", P, [n("ice", 0.35, 1, 0.3, 1)], parent="small-body",
    summary="A dirty snowball: a comet or an icy small body, like 67P.")
cls("small-body/metallic", P, [n("metal", 0.4, 1, 0.35, 1), n("carbon", 0, 0.2, 0, 0.25), n("ice", 0, 0.35, 0, 0.4)], parent="small-body",
    summary="Mostly iron and nickel (M-type), perhaps a stripped core, like 16 Psyche.")
cls("small-body/stony", P, [n("metal", 0, 0.4, 0, 0.45), n("carbon", 0, 0.2, 0, 0.25), n("ice", 0, 0.35, 0, 0.4)], parent="small-body",
    summary="Silicate rock with some metal (S-type), like Eros.")
cls("iron-rich", P, [n("mass", 1e-5, 10, 5e-6, 15), n("metal", 0.45, 1, 0.4, 1), n("ice", 0, 0.15, 0, 0.2)] + SOLID,
    summary="A world with an outsized iron core, like Mercury.")
cls("rocky", P, [n("mass", 1e-5, 10, 5e-6, 15), n("metal", 0, 0.45, 0, 0.5), n("ice", 0, 0.15, 0, 0.2)] + SOLID,
    summary="Rock around an iron core: Earth, Venus, Mars, the Moon, Io.",
    typical=[("density", 4.5, 1.2), ("metal", 0.3, 0.1)])
cls("water-rich", P, [n("mass", 1e-5, 15, 5e-6, 20), n("ice", 0.15, 1, 0.1, 1)] + SOLID,
    summary="Rock and a deep layer of water or ice: Ganymede, Titan, Pluto, perhaps some exoplanets.")
cls("sub-neptune", P, [n("mass", 1, 20, 0.7, 25), n("radius", 1.5, 3.5, 1.3, 3.8), n("gas", 0.005, 0.3, 0.003, 0.35)],
    summary="Rock and ice under a thick hydrogen envelope; the commonest planets in the galaxy and absent from ours.")
cls("ice-giant", P, [n("mass", 5, 60, 4, 80), n("radius", 3.5, 7, 3.2, 8), n("gas", 0.05, 0.5, 0.03, 0.55)],
    summary="Mostly water, ammonia and methane ices under a hydrogen-helium sky: Uranus, Neptune.")
cls("gas-giant", P, [n("mass", 20, 4130, 12, 4300), n("gas", 0.5, 1, 0.45, 1)],
    summary="Mostly hydrogen and helium: Jupiter, Saturn.")
for cid, lo, hi, flo, fhi, s in [
    ("ammonia-clouds", 0, 150, 0, 170, "Cold enough for ammonia clouds: Jupiter and Saturn (Sudarsky class I)."),
    ("water-clouds", 150, 350, 130, 370, "Warm enough that water clouds form instead (class II)."),
    ("cloudless", 350, 900, 320, 950, "Too warm for water clouds, too cool for metal ones: clear blue skies (class III)."),
    ("alkali-clouds", 900, 1400, 850, 1450, "Hot: sodium and potassium dominate the colours (class IV)."),
    ("silicate-clouds", 1400, 2200, 1350, 2300, "Very hot: clouds of rock vapour (class V)."),
    ("ultra-hot", 2200, 10000, 2100, 10000, "So hot that molecules break apart on the day side."),
]:
    cls(f"gas-giant/{cid}", P, [n("teq", lo, hi, flo, fhi)], parent="gas-giant", summary=s)
cls("brown-dwarf", P, [n("mass", 4130, 26000, 3900, 28000)],
    summary="Above about 13 Jupiter masses: deuterium burns for a while. A failed star, not a planet.")

# ---- physical: modifiers (stack) ----
M = lambda cid, region, s, **k: cls("physical/" + cid, P, region, kind="modifier", summary=s, **k)
M("tidally-locked", [flag("star_locked", True)], "One face always to its star: a day side and a night side.")
M("eccentric", [n("ecc", 0.2, 1, 0.15, 1)], "A strongly oval orbit: seasons set by distance.")
M("ringed", [flag("rings", True)], "Has a ring system.", meaning="flourish")
M("volcanic", [n("volcanic", 0.5, 1, 0.4, 1)], "Strongly volcanic, with lava or with ice.")
M("tide-heated", [n("tidal_heat", 0.05, 100, 0.03, 1000)], "Heated from inside by tides.")
M("unshielded", [n("field", 0, 0.1, 0, 0.2)], "No magnetic field of its own to keep the stellar wind off.")
M("young", [n("age", 0, 0.1, 0, 0.15)], "Less than about 100 million years old.")
M("ancient", [n("age", 10, 14, 9, 14)], "Older than about 10 billion years.")
M("rogue", [cat("host", "none")], "Drifts between the stars with no star of its own.")
M("captured", [cat("origin", "captured")], "Captured rather than formed where it is.")
M("living", [cat("life", "microbial", "complex", "intelligent")], "Has a biosphere.")
M("inhabited", [n("population", 1, 1e15, 0.5, 1e15)], "People live here.")
M("terraformed", [flag("terraformed", True)], "Made habitable by deliberate engineering.", evidence="fictional")
M("rubble-pile", [SMALL, n("porosity", 0.2, 1, 0.15, 1)], "A loose pile of rubble held by its own weak gravity.")
M("contact-binary", [SMALL, n("lobes", 2, 8, 2, 8)], "Two bodies that met gently and stayed together.", meaning="flourish")
M("puffed", [n("density", 0, 0.4, 0, 0.45), n("radius", 6, 30, 5.5, 30)], "A giant blown up far larger than its mass suggests.")
M("massive", [n("mass", 2000, 4130, 1800, 4130)], "A super-Jupiter, close to the brown-dwarf limit.")

# ---- surface (section 10, "by surface state") ----
S = "surface"
LOW_LIQ = n("liquid_fraction", 0, 0.001, 0, 0.002)
cls("surface/airless", S, [n("pressure", 0, 0.001, 0, 0.002), n("tsurf", 0, 1300, 0, 1350), n("ice_cover", 0, 0.95, 0, 0.97)], gate=SOLID,
    summary="Bare rock or regolith under a black sky: the Moon, Mercury.")
cls("surface/molten", S, [n("tsurf", 1300, 6000, 1200, 6000)], gate=SOLID, summary="A surface of lava.")
cls("surface/hot-greenhouse", S, [n("pressure", 10, 1e4, 8, 1e4), n("tsurf", 450, 1300, 420, 1350)], gate=SOLID,
    summary="A crushing, scorching air trapping its heat: Venus.")
cls("surface/hot-desert", S, [n("pressure", 0.001, 10, 0.0008, 12), n("tsurf", 330, 1300, 320, 1350), n("liquid_fraction", 0, 0.05, 0, 0.1)], gate=SOLID,
    summary="Hot, dry and open to the sky.")
cls("surface/temperate-desert", S, [n("pressure", 0.001, 10, 0.0008, 12), n("tsurf", 260, 330, 250, 340), n("liquid_fraction", 0, 0.1, 0, 0.15), n("ice_cover", 0, 0.5, 0, 0.55)], gate=SOLID,
    summary="Mild temperatures but little open water.")
cls("surface/temperate-ocean", S, [n("pressure", 0.006, 100, 0.005, 120), n("tsurf", 260, 330, 250, 340), n("liquid_fraction", 0.1, 0.9, 0.05, 0.95), cat("liquid", "water"), n("ice_cover", 0, 0.5, 0, 0.55)], gate=SOLID,
    summary="Seas and land under a mild sky: Earth.")
cls("surface/ocean-covered", S, [n("pressure", 0.006, 1000, 0.005, 1200), n("tsurf", 260, 400, 250, 420), n("liquid_fraction", 0.9, 1, 0.85, 1), cat("liquid", "water"), n("ice_cover", 0, 0.5, 0, 0.55)], gate=SOLID,
    summary="Water from pole to pole.")
cls("surface/cold-desert", S, [n("pressure", 0.001, 100, 0.0008, 120), n("tsurf", 0, 260, 0, 270), n("ice_cover", 0, 0.5, 0, 0.55), LOW_LIQ], gate=SOLID,
    summary="Cold, dry, thin-aired: Mars.")
cls("surface/ice-open-equator", S, [n("pressure", 0.001, 100, 0.0008, 120), n("tsurf", 0, 273, 0, 280), n("ice_cover", 0.5, 0.95, 0.45, 0.97), cat("liquid", "water", "none")], gate=SOLID,
    summary="Ice-covered except for a band of open water or bare ground at the equator.")
cls("surface/frozen", S, [n("ice_cover", 0.95, 1, 0.9, 1), flag("subsurface_ocean", False), n("tsurf", 0, 273, 0, 280), LOW_LIQ], gate=SOLID,
    summary="Frozen solid from pole to pole: Pluto, Triton.")
cls("surface/ice-shell-ocean", S, [n("ice_cover", 0.95, 1, 0.9, 1), flag("subsurface_ocean", True), n("tsurf", 0, 273, 0, 280), LOW_LIQ], gate=SOLID,
    summary="An ice shell over a hidden ocean: Europa, Enceladus, Ganymede.")
cls("surface/hydrocarbon-lakes", S, [cat("liquid", "methane", "ethane"), n("liquid_fraction", 0.001, 0.5, 0.0005, 0.6), n("tsurf", 60, 200, 50, 210), n("pressure", 0.01, 100, 0.008, 120)], gate=SOLID,
    summary="Lakes and seas of liquid methane and ethane: Titan.")
cls("surface/hycean", S, [cat("liquid", "water"), n("liquid_fraction", 0.9, 1, 0.85, 1), n("tsurf", 250, 400, 240, 420)], gate=[n("gas", 0.005, 0.1, 0.003, 0.12)],
    summary="A global ocean under a hydrogen sky: proposed, not yet confirmed.", evidence="speculative")

# ---- air: gas mixes (base) and pressure bands (modifiers) ----
A = "air"
cls("air/none", A, [cat("air_main", "none")], summary="No air to speak of.")
cls("air/nitrogen-oxygen", A, [cat("air_main", "n2"), n("pp_o2", 0.01, 10)], summary="Nitrogen with free oxygen: Earth's mix.")
cls("air/nitrogen", A, [cat("air_main", "n2"), n("pp_o2", 0, 0.01)], summary="Nitrogen without free oxygen: Titan, Triton, Pluto.")
cls("air/carbon-dioxide", A, [cat("air_main", "co2")], summary="Carbon dioxide: Venus, Mars.")
cls("air/hydrogen-helium", A, [cat("air_main", "h2", "he")], summary="The primordial mix of the giants.")
cls("air/sulphurous", A, [cat("air_main", "so2", "s", "h2s")], summary="Sulphur compounds: Io's thin air.")
cls("air/steam", A, [cat("air_main", "h2o")], summary="Mostly water vapour: a steam world.")
cls("air/methane", A, [cat("air_main", "ch4")], summary="Mostly methane.")
for cid, lo, hi, s in [("trace", 0, 0.001, "Too little to feel."), ("very-thin", 0.001, 0.1, "A wisp: Mars."), ("thin", 0.1, 0.7, "Thin: a mountaintop and above."),
                       ("standard", 0.7, 1.5, "About Earth's sea level."), ("dense", 1.5, 2.5, "Dense."), ("very-dense", 2.5, 10, "Very dense."),
                       ("crushing", 10, 1e7, "Crushing: Venus, or deep in a giant.")]:
    cls(f"air/{cid}", A, [n("pressure", lo, hi)], kind="modifier", summary=s)
# Tainted is the same region as the filter-mask profile less the shirt-sleeve
# band. Bounds are closed, and at exactly 0.005 bar the deeper shirt-sleeve
# profile wins, so tainted starts at the next double up.
cls("air/tainted", A, [n("pp_o2", 0.16, 0.5), n("pp_co2", math.nextafter(0.005, 1), 0.05)], kind="modifier",
    summary="Breathable oxygen but enough carbon dioxide to need a filter mask.")

# ---- habitability: human requirement profiles, nested ----
H = "habitability"
cls("habitable/sealed-habitat", H, [n("gravity", 0, 2.5, 0, 3), n("gas", 0, 0.01, 0, 0.02)],
    summary="People can live here only inside a sealed, shielded habitat.")
cls("habitable/pressure-suit", H, [n("tsurf", 40, 500, 30, 550), n("pressure", 0, 10, 0, 12), n("dose", 0, 500, 0, 1000)], parent="habitable/sealed-habitat",
    summary="A pressure suit lets people work outside.")
cls("habitable/breathing-apparatus", H, [n("pressure", 0.25, 5, 0.2, 6), n("tsurf", 230, 330, 220, 340)], parent="habitable/pressure-suit",
    summary="Warm clothes and bottled air are enough.")
cls("habitable/filter-mask", H, [n("pp_o2", 0.16, 0.5), n("pp_co2", 0, 0.05)], parent="habitable/breathing-apparatus",
    summary="The oxygen is breathable; a filter mask takes out the carbon dioxide.")
cls("habitable/shirt-sleeve", H, [n("pp_co2", 0, 0.005), n("tsurf", 263, 313, 255, 318), n("gravity", 0.3, 1.5, 0.2, 1.8), n("dose", 0, 20, 0, 50)], parent="habitable/filter-mask",
    summary="Step outside in a shirt.")


# ---- star: every star and remnant (the kernel's star class) ----
# The kernel reads each star's class off its life-line results
# (sse_formation::spectral: temperature against a scale, surface gravity
# against the luminosity-class lines, the phase). This scheme names, explains
# and samples those classes; tests/golden/star-classes.json carries the
# kernel's own tables and one life-line star per family and kind.
STARS = json.loads((ROOT / "tests/golden/star-classes.json").read_text())
ST = "star"
FAM = [
    ("pre-main-sequence", "", "A young star still contracting towards the main sequence, lit by its own squeeze (a T Tauri or Herbig star)."),
    ("main-sequence", "V", "A dwarf: burning hydrogen in its core, as the Sun does. Luminosity class V."),
    ("subgiant", "IV", "Its core hydrogen spent, swelling and cooling on the way to being a giant. Luminosity class IV."),
    ("giant", "III", "Swollen to many times its old size, with a low surface gravity. Luminosity class III."),
    ("bright-giant", "II", "Between the giants and the supergiants in gravity. Luminosity class II."),
    ("supergiant", "I", "A massive star swollen enormous, with the lowest surface gravity of all. Luminosity class Ia, Iab or Ib."),
    ("brown-dwarf", "", "Too light ever to burn hydrogen: it glows from its formation heat and fades for ever. Spectral types L, T and Y (young ones read as late M)."),
    ("white-dwarf", "", "The Earth-sized, white-hot core a sun-like star leaves behind, cooling for ever. D for degenerate, a letter for what its spectrum shows, and the temperature index 50400 / T."),
    ("neutron-star", "", "What is left when a star of about 8 to 25 solar masses collapses: about 1.4 solar masses in a ball some 20 km across."),
    ("black-hole", "", "What is left when the most massive stars collapse: nothing escapes from inside its horizon."),
]
assert [f for f, _, _ in FAM] == ["pre-main-sequence", "main-sequence", "subgiant", "giant", "bright-giant", "supergiant", "brown-dwarf", "white-dwarf", "neutron-star", "black-hole"]
COLOUR = {"O": "blue", "B": "blue-white", "A": "white", "F": "yellow-white", "G": "yellow", "K": "orange", "M": "red", "L": "deep red", "T": "magenta, methane-dark", "Y": "the coldest, barely glowing"}
DW = STARS["tables"]["dwarf_teff_k"]
LETTERS = ["O", "B", "A", "F", "G", "K", "M"]


def span_k(letter):
    """The letter's temperature span on the dwarf scale, in words."""
    row = DW[letter]
    nxt = ["O", "B", "A", "F", "G", "K", "M", "L", "T", "Y"]
    i = nxt.index(letter)
    lo = DW[nxt[i + 1]][0] if i + 1 < len(nxt) else None
    hi = row[0]
    k = lambda x: f"{round(x, -2):,.0f} K"
    if i == 0:
        return f"hotter than about {k(DW['B'][0])}"
    return f"about {k(lo)} to {k(hi)}" if lo else f"below about {k(row[0])}"


KINDS = {f: [] for f, _, _ in FAM}
for f in ["pre-main-sequence", "main-sequence", "subgiant", "giant", "bright-giant", "supergiant"]:
    KINDS[f] = LETTERS
KINDS["brown-dwarf"] = ["M", "L", "T", "Y"]
KINDS["white-dwarf"] = ["DX", "DC"]
KINDS["black-hole"] = ["BH", "IMBH", "SMBH"]
BH = STARS["tables"]["black_hole_msun"]
WDK = STARS["tables"]["white_dwarf_featureless_k"]
KIND_TEXT = {
    "DX": f"A white dwarf hot enough to show lines (above about {WDK:,.0f} K). Whether they are hydrogen (DA) or helium (DB, DO) needs the hydrogen layer left on it, which the kernel does not yet track: X says it cannot be said.",
    "DC": f"A white dwarf cooler than about {WDK:,.0f} K: too cool for lines, so its spectrum is featureless whatever its atmosphere.",
    "BH": f"A stellar-mass black hole, under {BH[0]:,.0f} solar masses: the collapsed core of a star that began above about 25 solar masses.",
    "IMBH": f"An intermediate-mass black hole, {BH[0]:,.0f} to {BH[1]:,.0f} solar masses. No single star leaves one.",
    "SMBH": f"A supermassive black hole, above {BH[1]:,.0f} solar masses: the kind at a galaxy's heart. No single star leaves one.",
}
SAMPLE = {(x["family"], x["kind"]): x for x in STARS["samples"]}


def star_facts(fam, kind, code):
    return {"star.class_family": fam, "star.class_kind": kind, "star.spectral_class": code}


FAM_WORDS = {"pre-main-sequence": "pre-main-sequence star", "main-sequence": "main-sequence star", "subgiant": "subgiant",
             "giant": "giant", "bright-giant": "bright giant", "supergiant": "supergiant", "brown-dwarf": "brown dwarf",
             "white-dwarf": "white dwarf", "neutron-star": "neutron star", "black-hole": "black hole"}
KIND_WORDS = {"DX": "White dwarf with lines (DX)", "DC": "Featureless white dwarf (DC)", "BH": "Stellar-mass black hole",
              "IMBH": "Intermediate-mass black hole", "SMBH": "Supermassive black hole"}
for fam, lum, about in FAM:
    cls(f"star/{fam}", ST, [cat("star_family", fam)], summary=about, plain=FAM_WORDS[fam][0].upper() + FAM_WORDS[fam][1:] + (f" ({lum})" if lum else ""))
    for kind in KINDS[fam]:
        if kind in KIND_TEXT:
            text = KIND_TEXT[kind]
        elif fam == "brown-dwarf":
            text = f"A {kind} brown dwarf: {COLOUR[kind]}, {span_k(kind)}."
        else:
            text = f"{kind}-type: {COLOUR[kind]}, {span_k(kind)} on the dwarf scale" + (" (giants of G, K and M are a little cooler at the same type)" if fam in ("giant", "bright-giant", "supergiant") and kind in "GKM" else "") + "."
        smp = SAMPLE.get((fam, kind))
        if not smp:
            text += " The life line does not make one today, so it has no sample star."
        cls(f"star/{fam}/{kind}", ST, [cat("star_kind", kind)], parent=f"star/{fam}", summary=text,
            plain=KIND_WORDS.get(kind) or f"{kind} {FAM_WORDS[fam]}" + (f" ({kind} {lum})" if lum and lum != "I" else ""))

# ---- examples ----
by_id = {c["id"]: c for c in CLASSES}


def example(cid, name, facts, expect="in"):
    by_id[cid].setdefault("examples", []).append({"name": name, "values": facts, "expect": expect})


refs = json.loads((ROOT / "tests/fixtures/classify/solar-system.json").read_text())["bodies"]
facts = {b["name"]: {k: v for k, v in b["facts"].items() if k != "name"} for b in refs}


def catalogued(name, f):
    """A Solar System body's size and orbit as catalogued in base-sol, with
    its surface gravity (G M / R^2) and its period (Kepler, about its host)
    worked out from them; other facts stay as the fixture has them."""
    b = SOL.get(name)
    if b is None:
        return f
    v = {k: x["v"] for k, x in b["values"].items()}
    by_id = {x["id"]: x for x in SOL.values()}
    host = by_id[b["host"]]["values"]["body.mass"]["v"] if b["host"] else None
    f = dict(f)
    f["body.mass"], f["body.radius"] = v["body.mass"], v["body.radius"]
    f["body.surface_gravity"] = SHEET["G"] * v["body.mass"] / v["body.radius"] ** 2
    if host is not None and "orbit.semi_major_axis" in v:
        a = v["orbit.semi_major_axis"]
        f["orbit.semi_major_axis"] = a
        f["orbit.period"] = 2 * math.pi * math.sqrt(a ** 3 / (SHEET["G"] * (host + v["body.mass"])))
    return f


facts = {n: catalogued(n, f) for n, f in facts.items()}
for b in refs:
    e = b["expect"]
    for cid in [e["physical"], e["surface"], e["habitability"], e["air"], *e["physical_modifiers"], *e["air_modifiers"]]:
        if cid:
            example(cid, b["name"], facts[b["name"]])
for b in json.loads((ROOT / "tests/fixtures/classify/exoplanets.json").read_text())["bodies"]:
    f = {k: v for k, v in b["facts"].items() if k != "name"}
    for scheme, ids in b["expect"].items():
        for cid in ids[:1] if scheme != "modifiers" else ids:
            example(cid, b["name"], f)

earth = facts["Earth"]


def like(base, **over):
    f = copy.deepcopy(base)
    for k, v in over.items():
        f[k.replace("__", ".")] = v
    return f


small = lambda m_kg, **over: like({"body.mass": m_kg, "makeup.metal": 0, "makeup.rock": 0, "makeup.carbon": 0, "makeup.ice": 0, "makeup.gas": 0, "body.porosity": 0.1, "shape.lobes": 1}, **over)
example("small-body/carbonaceous", "Bennu", small(7.33e10, makeup__carbon=0.5, makeup__rock=0.5, body__porosity=0.55))
example("physical/rubble-pile", "Bennu", small(7.33e10, makeup__carbon=0.5, makeup__rock=0.5, body__porosity=0.55))
example("small-body/stony", "Eros", small(6.69e15, makeup__rock=0.85, makeup__metal=0.15))
example("small-body/metallic", "16 Psyche", small(2.27e19, makeup__metal=0.6, makeup__rock=0.4))
example("small-body/icy", "67P/Churyumov-Gerasimenko", small(1.0e13, makeup__ice=0.55, makeup__carbon=0.25, makeup__rock=0.2, body__porosity=0.7))
example("physical/contact-binary", "Arrokoth", small(7.5e17, makeup__ice=0.6, makeup__rock=0.4, shape__lobes=2))
example("small-body", "Eros", small(6.69e15, makeup__rock=0.85, makeup__metal=0.15))
example("small-body", "Ceres", facts["Ceres"], "out")
example("gas-giant", "Jupiter", facts["Jupiter"])
pso = {"body.mass": 6.5 * 317.83 * ME, "body.radius": 1.5 * 11.21 * RE, "makeup.gas": 0.9, "planet.equilibrium_temperature": 1160, "orbit.host_kind": "none"}
example("physical/rogue", "PSO J318.5-22", pso)
example("physical/massive", "PSO J318.5-22", pso)
example("physical/young", "illustrative: Earth at 50 million years", like(earth, system__age=0.05 * GYR))
example("physical/ancient", "illustrative: an Earth-like world 11 billion years old", like(earth, system__age=11 * GYR))
example("physical/terraformed", "illustrative: a terraformed Mars", like(facts["Mars"], history__terraformed=True))
example("physical/tidally-locked", "Earth", earth, "out")
dry = lambda t, **o: like(earth, planet__surface_temperature=t, surface__liquid_fraction=0, surface__liquid="none", **o)
example("surface/temperate-desert", "illustrative: Earth with its oceans gone", dry(300))
example("surface/hot-desert", "illustrative: a dry Earth at 0.7 au", dry(380))
example("surface/ocean-covered", "illustrative: an ocean world", like(earth, surface__liquid_fraction=1.0, surface__ice_fraction=0))
example("surface/temperate-ocean", "illustrative: an ocean world", like(earth, surface__liquid_fraction=1.0, surface__ice_fraction=0), "out")
example("surface/ice-open-equator", "illustrative: a slushball Earth", like(earth, planet__surface_temperature=250, surface__ice_fraction=0.8, surface__liquid_fraction=0.15))
example("habitable/filter-mask", "illustrative: Earth with 2% carbon dioxide", like(earth, atmosphere__pp__co2=0.02))
example("air/tainted", "illustrative: Earth with 2% carbon dioxide", like(earth, atmosphere__pp__co2=0.02))
example("habitable/shirt-sleeve", "illustrative: Earth with 2% carbon dioxide", like(earth, atmosphere__pp__co2=0.02), "out")
example("habitable/breathing-apparatus", "illustrative: a high-plateau world with thin oxygen", like(earth, atmosphere__pp__o2=0.08, atmosphere__surface_pressure=0.6e5))
example("air/thin", "illustrative: a high-plateau world with thin oxygen", like(earth, atmosphere__pp__o2=0.08, atmosphere__surface_pressure=0.6e5))
example("air/steam", "illustrative: a steam world", like(earth, atmosphere__main="h2o", atmosphere__surface_pressure=50e5))
example("air/methane", "illustrative: a methane-sky moon", like(facts["Titan"], atmosphere__main="ch4"))
example("air/dense", "illustrative: Earth at 2 bar", like(earth, atmosphere__surface_pressure=2e5))
example("air/very-dense", "illustrative: Earth at 5 bar", like(earth, atmosphere__surface_pressure=5e5))
example("gas-giant/water-clouds", "Jupiter", facts["Jupiter"], "out")


# Star examples: the kernel's life-line sample for the class (its first "in"
# example carries star.sample_mass_msun, star.sample_age_yr, star.sample_feh
# for the class gallery), else an illustrative one; and real stars where they
# fit.
for fam, _, _ in FAM:
    first = None
    for kind in KINDS[fam]:
        smp = SAMPLE.get((fam, kind))
        cid = f"star/{fam}/{kind}"
        if smp:
            vals = star_facts(fam, kind, smp["code"]) | {"star.sample_mass_msun": smp["mass_msun"], "star.sample_age_yr": smp["age_yr"], "star.sample_feh": smp["feh"]}
            name = f"a {smp['mass_msun']:.3g} solar-mass star at {smp['age_yr'] / 1e6:.3g} million years ({smp['code']})"
            example(cid, name, vals)
            first = first or (name, vals)
        else:
            example(cid, f"illustrative: a {kind} {fam.replace('-', ' ')}", star_facts(fam, kind, kind))
    if first:
        example(f"star/{fam}", *first)
    elif fam == "neutron-star" and (fam, "NS") in SAMPLE:
        smp = SAMPLE[(fam, "NS")]
        example(f"star/{fam}", f"a {smp['mass_msun']:.3g} solar-mass star at {smp['age_yr'] / 1e6:.3g} million years ({smp['code']})",
                star_facts(fam, "NS", "NS") | {"star.sample_mass_msun": smp["mass_msun"], "star.sample_age_yr": smp["age_yr"], "star.sample_feh": smp["feh"]})
    else:
        example(f"star/{fam}", f"illustrative: a {fam.replace('-', ' ')}", star_facts(fam, KINDS[fam][0] if KINDS[fam] else "", ""))
for cid, name, fam, kind, code in [
    ("star/main-sequence/G", "the Sun", "main-sequence", "G", "G2 V"),
    ("star/main-sequence/M", "Proxima Centauri", "main-sequence", "M", "M5.5 V"),
    ("star/main-sequence/A", "Sirius A", "main-sequence", "A", "A1 V"),
    ("star/giant/K", "Arcturus", "giant", "K", "K1.5 III"),
    ("star/supergiant/B", "Rigel", "supergiant", "B", "B8 Ia"),
    ("star/supergiant/M", "Betelgeuse", "supergiant", "M", "M1-2 Ia-ab"),
    ("star/white-dwarf/DX", "Sirius B (DA2, hydrogen shows)", "white-dwarf", "DX", "DA2"),
]:
    example(cid, name, star_facts(fam, kind, code))

# Cross-references (C3): what another suite or the author must settle before
# a body can be placed in the class. "life:<capability>" asks the life suite;
# "authored:<what>" can only be an authored intervention.
for cid, needs in [
    ("air/nitrogen-oxygen", ["life:oxygenic_photosynthesis"]),
    ("physical/living", ["life:any"]),
    ("physical/inhabited", ["authored:settlement"]),
    ("physical/terraformed", ["authored:terraforming"]),
]:
    by_id[cid]["needs"] = needs

missing = [c["id"] for c in CLASSES if not any(e["expect"] == "in" for e in c.get("examples", []))]
assert not missing, f"classes with no example: {missing}"

SCHEMES = [
    {"id": "physical", "question": "What kind of object is this?"},
    {"id": "surface", "question": "What is its surface like?"},
    {"id": "air", "question": "What is its air made of, and how much is there?"},
    {"id": "habitability", "question": "What do people need to live here?"},
    {"id": "star", "question": "What kind of star is this?"},
]
pack = {
    "format": "sse.pack/1", "id": "sse.base-atlas", "version": "0.1.0", "kind": ["base-physics"],
    "kernel_range": "^0.1", "licence": "Apache-2.0",
    "provides": {"other": {
        "about": "SSE4's shipped class atlas (C2): physical type, surface state, gas mix and pressure, human habitability profiles, and every star and remnant's class. Generated by scripts/build-base-atlas.py.",
        "schemes": SCHEMES, "axes": [[k, v] for k, v in sorted(AXES.items())], "classes": CLASSES}},
}
text = json.dumps(pack, indent=1) + "\n"
out = ROOT / "packs/base-atlas/pack.json"
if "--check" in sys.argv:
    if out.read_text() != text:
        print("packs/base-atlas is stale: run scripts/build-base-atlas.py")
        sys.exit(1)
    print("base-atlas pack is current")
else:
    out.write_text(text)
    print(f"{len(CLASSES)} classes in {len(SCHEMES)} schemes, {len(AXES)} axes")
