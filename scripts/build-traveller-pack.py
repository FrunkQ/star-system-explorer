#!/usr/bin/env python3
"""Build packs/traveller/pack.json: the rules part of the Traveller pack
(design handoff 7A parts 1, 8 and 9; build plan X3a).

Everything here is ordinary pack data read by general machinery:

  schemes, axes, classes   one region-scored scheme per profile digit
                           (starport, size, atmosphere, hydrographics,
                           population, government, law, tech), read by the
                           class atlas (K8, C1)
  rules                    K8 rules: flag facts the atmosphere classes read,
                           the body counts behind PBG, trade codes, travel
                           zones and bases
  code_sheets              one sheet (sse-codes) that strings the digits into
                           the profile line and PBG, and names the tag groups

Digit boundaries follow the published size, pressure, hydrographic and
population bands that every edition shares; the trade codes are the T5 /
Second Survey set (what Travellermap publishes, so imported remarks can be
compared). The descriptions are SSE4's own words. Choices where physics had to
be squared with a code are listed in docs/fudges.md. Run after editing:

    python3 scripts/build-traveller-pack.py
"""
import json
import math
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
# The master data sheet's values (MP22: no typed copies). The pack is built
# from them, and check.sh runs --check so an edited sheet rebuilds it.
REF = {c["id"]: c["value"]["v"] for c in json.loads((ROOT / "packs/base-reference/pack.json").read_text())["provides"]["reference"]["constants"]}
RE = REF["R_earth_mean"]
def au_km(n):
    """n astronomical units in km, from the sheet's au (in metres, exactly)."""
    return n * REF["au"] / 1e3
INF = 1e6
EHEX = "0123456789ABCDEFGHJKLMNPQRSTUVWXYZ"


def below(x):
    """The largest double below x: bands are [lo, below(hi)], so no two touch."""
    return math.nextafter(x, -math.inf)


def above(x):
    return math.nextafter(x, math.inf)


def ax(field, scale, span=None, label="", unit="", div=None, missing=None, prov=False):
    a = {"field": field, "scale": scale, "label": label or field}
    if span:
        a["span"] = list(span)
    if unit:
        a["unit"] = unit
    if div:
        a["divide_by"] = div
    if missing is not None:
        a["missing"] = missing
    if prov:
        a["provisional"] = True
    return a


AXES = {
    "traveller.size": ax("body.radius", "linear", (0, 16), "diameter", "x 1600 km", 800000.0),
    "traveller.pressure": ax("atmosphere.surface_pressure", "linear", (0, 100), "surface pressure", "bar", 1e5, missing=0),
    "traveller.pp_o2": ax("atmosphere.pp.o2", "linear", (0, 10), "oxygen partial pressure", "bar", prov=True),
    "traveller.pp_co2": ax("atmosphere.pp.co2", "linear", (0, 100), "carbon dioxide partial pressure", "bar", prov=True),
    "traveller.pp_so2": ax("atmosphere.pp.so2", "linear", (0, 10), "sulphur dioxide partial pressure", "bar", prov=True),
    "traveller.oxygen": ax("traveller.prep.oxygen", "flag", label="air carries oxygen (10% or more)"),
    "traveller.tainted": ax("traveller.prep.tainted", "flag", label="air is tainted"),
    "traveller.corrosive": ax("traveller.prep.corrosive", "flag", label="air is corrosive"),
    "traveller.belt": ax("traveller.prep.belt", "flag", label="an asteroid belt"),
    "traveller.air_kind": ax("atmosphere.traveller_kind", "category", label="authored unusual air", missing="none", prov=True),
    "traveller.liquid": ax("surface.liquid_fraction", "fraction", (0, 1), "surface liquid cover", missing=0, prov=True),
    "traveller.population": ax("settlements.population", "linear", (0, 1e16), "population", "people", missing=0, prov=True),
    "traveller.government": ax("society.government", "linear", (0, 15), "government type", missing=0, prov=True),
    "traveller.law": ax("society.law_level", "linear", (0, 18), "law level", missing=0, prov=True),
    "traveller.tech": ax("society.tech_level", "linear", (0, 33), "tech level", missing=0, prov=True),
    "traveller.starport": ax("society.starport", "category", label="starport class", missing="X", prov=True),
}

CLASSES = []
SHEET_SYMBOLS = {}


def n(axis, lo, hi):
    return {"axis": axis, "core": [lo, hi], "fringe": [lo, hi]}


def cat(axis, *vals):
    return {"axis": axis, "any_of": list(vals)}


def flag(axis, v):
    return {"axis": axis, "is": v}


def cls(scheme, key, symbol, region, physics, plain, summary, typical=None, value=None, parent=None,
        plain_facts=None, evidence="science"):
    cid = f"{scheme}/{key}"
    c = {"id": cid, "scheme": scheme, "kind": "base", "region": region, "threshold": 1.0,
         "names": {"physics": physics, "plain": plain, "story": plain},
         "learn": {"summary": summary}, "source": "Traveller pack (SSE4)"}
    if parent:
        c["parent"] = f"{scheme}/{parent}"
    if evidence != "science":
        c["evidence"] = evidence
    if typical:
        c["typical"] = [{"axis": a, "centre": v, "spread": 0} for a, v in typical]
    CLASSES.append(c)
    y = {"class": cid, "symbol": symbol}
    if value is not None:
        y["value"] = value
    if plain_facts:
        y["plain"] = plain_facts
    SHEET_SYMBOLS.setdefault(scheme, []).append(y)
    return cid


# ---- size: diameter in steps of 1600 km --------------------------------
S = "traveller.size"
cls(S, "0", "0", [n("traveller.size", 0, 0.5), flag("traveller.belt", False)], "size 0", "tiny",
    "Under 800 km across: a planetoid.", [("traveller.size", 0.25)], 0)
cls(S, "0-belt", "0", [flag("traveller.belt", True)], "size 0 (belt)", "asteroid belt",
    "An asteroid belt: the world is many small bodies.", None, 0)
for d in range(1, 16):
    hi = INF if d == 15 else d + 0.5
    km = d * 1600
    cls(S, f"{d:02d}", EHEX[d], [n("traveller.size", above(d - 0.5), hi), flag("traveller.belt", False)],
        f"size {EHEX[d]}", f"about {km:,} km across",
        f"About {km:,} km in diameter (from {km - 800:,} km)." + (" Size F also holds anything larger." if d == 15 else ""),
        [("traveller.size", float(d))], d)

# ---- atmosphere: pressure band and what the air is made of ---------------
A = "traveller.atmosphere"
NONE = {"atmosphere.main": "none"}


PLAIN_AIR = cat("traveller.air_kind", "none")


def air(p, o2, co2, so2):
    return [("traveller.pressure", p), ("traveller.pp_o2", o2), ("traveller.pp_co2", co2), ("traveller.pp_so2", so2)]


cls(A, "00", "0", [n("traveller.pressure", 0, below(0.001)), PLAIN_AIR], "atmosphere 0", "vacuum",
    "No air worth the name (under 0.001 bar).", air(0, 0, 0, 0), 0, plain_facts=NONE)
cls(A, "01", "1", [n("traveller.pressure", 0.001, below(0.1)), PLAIN_AIR], "atmosphere 1", "trace",
    "A trace of gas (0.001 to 0.1 bar): a vacc suit is needed.", air(0.01, 0, 0.0095, 0), 1,
    plain_facts={"atmosphere.main": "co2"})
for lo, hi, dt, dc, name in [(0.1, 0.43, 2, 3, "very thin"), (0.43, 0.71, 4, 5, "thin"),
                             (0.71, 1.5, 7, 6, "standard"), (1.5, 2.5, 9, 8, "dense")]:
    # (tainted digit, clean digit): the tainted one is even below standard, odd from it.
    mid = {2: 0.25, 4: 0.57, 7: 1.0, 9: 2.0}[dt]
    band = [n("traveller.pressure", lo, below(hi)), flag("traveller.oxygen", True), flag("traveller.corrosive", False), PLAIN_AIR]
    cls(A, f"{dc:02d}", EHEX[dc], band + [flag("traveller.tainted", False)], f"atmosphere {EHEX[dc]}", name,
        f"{name.capitalize()} air with enough oxygen ({lo} to {hi} bar).", air(mid, 0.21 * mid, 0.0004 * mid, 0), dc,
        plain_facts={"atmosphere.main": "n2"})
    cls(A, f"{dt:02d}", EHEX[dt], band + [flag("traveller.tainted", True)], f"atmosphere {EHEX[dt]}", f"{name}, tainted",
        f"{name.capitalize()} air with oxygen but tainted: too much carbon dioxide, oxygen or a named taint ({lo} to {hi} bar).",
        air(mid, 0.21 * mid, 0.02, 0), dt, plain_facts={"atmosphere.main": "n2"})
cls(A, "13", "D", [n("traveller.pressure", 2.5, INF), flag("traveller.oxygen", True), flag("traveller.corrosive", False), PLAIN_AIR],
    "atmosphere D", "very dense", "Very dense air with oxygen (2.5 bar and up): breathable only high up.",
    air(5.0, 1.05, 0.002, 0), 13, plain_facts={"atmosphere.main": "n2"})
cls(A, "10", "A", [n("traveller.pressure", 0.1, INF), flag("traveller.oxygen", False), flag("traveller.corrosive", False), PLAIN_AIR],
    "atmosphere A", "exotic", "Air without enough oxygen to breathe, but not corrosive: an air supply is needed.",
    air(1.5, 0, 0.001, 0), 10, plain_facts={"atmosphere.main": "n2"})
cls(A, "11", "B", [n("traveller.pressure", 0.1, below(10)), flag("traveller.corrosive", True), PLAIN_AIR],
    "atmosphere B", "corrosive", "Corrosive air (sulphur or halogen compounds): a protective suit is needed.",
    air(3.0, 0, 2.9, 0.01), 11, plain_facts={"atmosphere.main": "co2"})
cls(A, "12", "C", [n("traveller.pressure", 10, INF), flag("traveller.corrosive", True), PLAIN_AIR],
    "atmosphere C", "insidious", "Corrosive air at crushing pressure (10 bar and up): it defeats protective suits in hours.",
    air(90.0, 0, 87.0, 0.015), 12, plain_facts={"atmosphere.main": "co2"})
# E and F are never produced by physics alone: they are authored kinds.
cls(A, "authored", "F", [cat("traveller.air_kind", "low", "unusual")], "atmosphere E or F", "authored air kind",
    "Air set by hand as low (E) or unusual (F).", None, 15)
cls(A, "15", "F", [cat("traveller.air_kind", "unusual")], "atmosphere F", "unusual",
    "Unusual air (authored): it varies with place or time.", air(1.0, 0.21, 0.0004, 0), 15, parent="authored",
    plain_facts={"atmosphere.main": "n2", "atmosphere.traveller_kind": "unusual"})
cls(A, "14", "E", [cat("traveller.air_kind", "low")], "atmosphere E", "low",
    "Thin air that is breathable only in the lowlands (authored).", air(0.57, 0.12, 0.0002, 0), 14, parent="authored",
    plain_facts={"atmosphere.main": "n2", "atmosphere.traveller_kind": "low"})
# Put the children first so a code reads back to them, not to the parent.
SHEET_SYMBOLS[A].sort(key=lambda y: y["class"].endswith("/authored"))

# ---- hydrographics: surface liquid in tenths ------------------------------
H = "traveller.hydrographics"
for d in range(0, 11):
    lo = 0 if d == 0 else above((d - 0.5) / 10)
    hi = 1.0 if d == 10 else (d + 0.5) / 10
    cls(H, f"{d:02d}", EHEX[d], [n("traveller.liquid", lo, hi)], f"hydrographics {EHEX[d]}",
        "dry" if d == 0 else ("all liquid" if d == 10 else f"about {d * 10}% liquid"),
        f"About {d * 10}% of the surface under liquid.", [("traveller.liquid", d / 10)], d,
        plain_facts={"surface.liquid": "water"} if d else None)

# ---- population: powers of ten ------------------------------------------
P = "traveller.population"
cls(P, "00", "0", [n("traveller.population", 0, below(1))], "population 0", "uninhabited",
    "Nobody lives here.", [("traveller.population", 0)], 0)
for d in range(1, 16):
    hi = 1e18 if d == 15 else below(10.0 ** (d + 1))
    cls(P, f"{d:02d}", EHEX[d], [n("traveller.population", 10.0 ** d, hi)], f"population {EHEX[d]}",
        f"10^{d} people or more", f"From 10^{d} to under 10^{d + 1} people.", [("traveller.population", 3 * 10.0 ** d)], d)


# ---- social digits: authored or from a society suite ---------------------
def whole(scheme, axis, top, word, words):
    for d in range(0, top + 1):
        cls(scheme, f"{d:02d}", EHEX[d], [n(axis, d, d)], f"{word} {EHEX[d]}", words.get(d, f"{word} {d}"),
            f"{word.capitalize()} {d}.", [(axis, float(d))], d)


whole("traveller.government", "traveller.government", 15, "government",
      {0: "none", 1: "company or corporation", 2: "participating democracy", 3: "self-perpetuating oligarchy",
       4: "representative democracy", 5: "feudal technocracy", 6: "captive government", 7: "balkanised",
       8: "civil service bureaucracy", 9: "impersonal bureaucracy", 10: "charismatic dictator",
       11: "non-charismatic leader", 12: "charismatic oligarchy", 13: "religious dictatorship",
       14: "religious autocracy", 15: "totalitarian oligarchy"})
whole("traveller.law", "traveller.law", 18, "law level", {0: "no law"})
whole("traveller.tech", "traveller.tech", 33, "tech level", {})
for letter, words in [("A", "excellent"), ("B", "good"), ("C", "routine"), ("D", "poor"), ("E", "frontier"),
                      ("X", "none"), ("F", "good spaceport"), ("G", "poor spaceport"), ("H", "primitive spaceport"),
                      ("Y", "no spaceport")]:
    cls("traveller.starport", letter, letter, [cat("traveller.starport", letter)], f"starport {letter}", words,
        f"Starport class {letter}: {words}.", None, EHEX.index(letter), plain_facts={"society.starport": letter})

SCHEMES = [
    {"id": "traveller.starport", "question": "What class is its starport?"},
    {"id": "traveller.size", "question": "How big is it, in Traveller size steps?"},
    {"id": "traveller.atmosphere", "question": "What is its air, in Traveller atmosphere codes?"},
    {"id": "traveller.hydrographics", "question": "How much of its surface is liquid, in tenths?"},
    {"id": "traveller.population", "question": "How many people live here, as a power of ten?"},
    {"id": "traveller.government", "question": "What government does it have?"},
    {"id": "traveller.law", "question": "What is its law level?"},
    {"id": "traveller.tech", "question": "What is its tech level?"},
]

# ---- rules ---------------------------------------------------------------
RULES = []


def rule(tag, when, applies=None, rid=None):
    r = {"id": rid or f"{tag}.v1", "tag": tag, "when": when}
    if applies:
        r["applies_to"] = applies
    RULES.append(r)


def rng(field, lo=None, hi=None):
    p = {"if": "range", "field": field}
    if lo is not None:
        p["min"] = lo
    if hi is not None:
        p["max"] = hi
    return p


def is_(field, value):
    return {"if": "is", "field": field, "value": value}


def all_(*ps):
    return {"if": "all", "of": list(ps)}


def any_(*ps):
    return {"if": "any", "of": list(ps)}


def has(field):
    return {"if": "has", "field": field}


ALWAYS = all_()

# Flag facts the atmosphere classes read.
rule("traveller.prep.oxygen", {"if": "compare", "left": "atmosphere.pp.o2", "op": "ge",
                               "right": "atmosphere.surface_pressure", "factor": 1e-6})
rule("traveller.prep.corrosive", any_(rng("atmosphere.pp.so2", 0.001), rng("atmosphere.pp.cl2", 0.001),
                                      rng("atmosphere.pp.f2", 0.001), rng("atmosphere.pp.h2so4", 0.001),
                                      *[is_("atmosphere.main", g) for g in ["so2", "h2s", "cl2", "f2", "hcl", "h2so4"]]))
rule("traveller.prep.tainted", any_(has("atmosphere.taint"), rng("atmosphere.pp.co2", above(0.005)),
                                    rng("atmosphere.pp.o2", above(0.5)), rng("atmosphere.pp.so2", 1e-5)))
rule("traveller.prep.belt", ALWAYS, ["belt"])
# Bodies the PBG digits count.
rule("traveller.count.belt", ALWAYS, ["belt"])
rule("traveller.count.gas-giant", all_(rng("body.radius", 2.5 * RE), rng("body.mass", None, 13 * REF["M_jup"])), ["body"])

U = "traveller.uwp."


def d(seg, *vals):
    """The digit is one of vals: numbers or (lo, hi) pairs."""
    ps = []
    for v in vals:
        if isinstance(v, tuple):
            ps.append(rng(U + seg, v[0], v[1]))
        else:
            ps.append(is_(U + seg, v))
    return ps[0] if len(ps) == 1 else any_(*ps)


# T5 / Second Survey trade codes. Planetary:
TRADE = {
    "As": all_(d("size", 0), d("atmosphere", 0), d("hydrographics", 0)),
    "De": all_(d("atmosphere", (2, 9)), d("hydrographics", 0)),
    "Fl": all_(d("atmosphere", (10, 12)), d("hydrographics", (1, 10))),
    "Ga": all_(d("size", (6, 8)), d("atmosphere", 5, 6, 8), d("hydrographics", (5, 7))),
    "He": all_(d("size", (3, 12)), d("atmosphere", 2, 4, 7, (9, 12)), d("hydrographics", (0, 2))),
    "Ic": all_(d("atmosphere", (0, 1)), d("hydrographics", (1, 10))),
    "Oc": all_(d("size", (10, 15)), d("atmosphere", (3, 9), (13, 15)), d("hydrographics", 10)),
    "Va": d("atmosphere", 0),
    "Wa": all_(d("size", (3, 9)), d("atmosphere", (3, 9), (13, 15)), d("hydrographics", 10)),
    # Population:
    "Di": all_(d("population", 0), d("government", 0), d("law", 0), d("tech", (1, 33))),
    "Ba": all_(d("population", 0), d("government", 0), d("law", 0), d("tech", 0)),
    "Lo": d("population", (1, 3)),
    "Ni": d("population", (4, 6)),
    "Ph": d("population", 8),
    "Hi": d("population", (9, 15)),
    # Economic:
    "Pa": all_(d("atmosphere", (4, 9)), d("hydrographics", (4, 8)), d("population", 4, 8)),
    "Ag": all_(d("atmosphere", (4, 9)), d("hydrographics", (4, 8)), d("population", (5, 7))),
    "Na": all_(d("atmosphere", (0, 3)), d("hydrographics", (0, 3)), d("population", (6, 15))),
    "Pi": all_(d("atmosphere", (0, 2), 4, 7, 9), d("population", (7, 8))),
    "In": all_(d("atmosphere", (0, 2), 4, 7, (9, 12)), d("population", (9, 15))),
    "Po": all_(d("atmosphere", (2, 5)), d("hydrographics", (0, 3))),
    "Pr": all_(d("atmosphere", 6, 8), d("population", 5, 9)),
    "Ri": all_(d("atmosphere", 6, 8), d("population", (6, 8))),
    # From physics directly (orbit and spin):
    "Sa": is_("orbit.host_kind", "planet"),
    "Lk": all_(is_("orbit.host_kind", "planet"), is_("spin.host_locked", True)),
    "Tz": all_(is_("orbit.host_kind", "star"), is_("spin.star_locked", True)),
    # Climate, from the mean surface temperature (see docs/fudges.md):
    "Fr": all_(d("size", (2, 9)), d("hydrographics", (1, 10)), rng("planet.surface_temperature", None, below(223))),
    "Co": all_(d("size", (2, 15)), rng("planet.surface_temperature", 223, below(263))),
    "Ho": all_(d("size", (2, 15)), rng("planet.surface_temperature", above(303), 353)),
    "Tu": all_(d("size", (6, 9)), d("atmosphere", (4, 9)), d("hydrographics", (3, 7)),
               rng("planet.surface_temperature", 223, below(263))),
    "Tr": all_(d("size", (6, 9)), d("atmosphere", (4, 9)), d("hydrographics", (3, 7)),
               rng("planet.surface_temperature", above(303), 353)),
}
for code, when in TRADE.items():
    rule(f"traveller.trade.{code}", when)

# Travel zones: set by the referee (authored); hints from the codes.
rule("traveller.zone.A", is_("traveller.zone", "A"))
rule("traveller.zone.R", is_("traveller.zone", "R"))
rule("traveller.zonehint.amber", any_(d("atmosphere", (10, 15)), d("government", 0, 7, 10), d("law", 0, (9, 18))))
# Bases: authored flags (stations as constructs come with X3d).
BASES = {"N": "naval base", "S": "scout base", "W": "scout way station", "D": "naval depot",
         "M": "military base", "C": "corsair base", "K": "naval base of another power"}
for b in BASES:
    rule(f"traveller.base.{b}", is_(f"traveller.bases.{b}", True))

# What a traveller needs to breathe or survive the air, from the atmosphere
# digit (7A part 5: "each lists what a traveller needs as a derived tag").
NEEDS = {
    "vacc_suit": ("a vacuum suit", d("atmosphere", 0, 1)),
    "respirator": ("a respirator (compressor) for the thin air", d("atmosphere", 2, 3)),
    "filter": ("a filter mask against the taint", d("atmosphere", 2, 4, 7, 9)),
    "air_supply": ("an air supply: no breathable oxygen", d("atmosphere", 10)),
    "protective_suit": ("a protective suit against corrosive air", d("atmosphere", 11)),
    "sealed_suit": ("a sealed suit, renewed often: insidious air defeats seals", d("atmosphere", 12)),
    "altitude": ("high ground: the air is breathable only at altitude", d("atmosphere", 13)),
    "low_ground": ("low ground: the air is breathable only in the lowlands", d("atmosphere", 14)),
}
for k, (_, when) in NEEDS.items():
    rule(f"traveller.need.{k}", when)

TRADE_WORDS = {
    "As": "asteroid", "De": "desert", "Fl": "fluid oceans", "Ga": "garden", "He": "hellworld", "Ic": "ice-capped",
    "Oc": "ocean world", "Va": "vacuum", "Wa": "water world", "Di": "dieback", "Ba": "barren", "Lo": "low population",
    "Ni": "non-industrial", "Ph": "pre-high population", "Hi": "high population", "Pa": "pre-agricultural",
    "Ag": "agricultural", "Na": "non-agricultural", "Pi": "pre-industrial", "In": "industrial", "Po": "poor",
    "Pr": "pre-rich", "Ri": "rich", "Sa": "satellite", "Lk": "locked to its planet", "Tz": "twilight zone",
    "Fr": "frozen", "Co": "cold", "Ho": "hot", "Tu": "tundra", "Tr": "tropic",
}

SEGMENTS = [
    ("starport", "Starport", "traveller.starport"),
    ("size", "Size", "traveller.size"),
    ("atmosphere", "Atmosphere", "traveller.atmosphere"),
    ("hydrographics", "Hydrographics", "traveller.hydrographics"),
    ("population", "Population", "traveller.population"),
    ("government", "Government", "traveller.government"),
    ("law", "Law level", "traveller.law"),
    ("tech", "Tech level", "traveller.tech"),
]
segments = [{"id": sid, "label": lab, "authored": f"traveller.code.{sid}",
             "source": {"from": "classify", "scheme": sc, "symbols": SHEET_SYMBOLS[sc]}}
            for sid, lab, sc in SEGMENTS]
segments += [
    {"id": "pop_mult", "label": "Population multiplier", "authored": "traveller.code.pop_mult",
     "source": {"from": "leading", "field": "settlements.population", "segment": "population"}},
    {"id": "belts", "label": "Planetoid belts", "authored": "traveller.code.belts",
     "source": {"from": "count", "tag": "traveller.count.belt", "max": 33}},
    {"id": "gas_giants", "label": "Gas giants", "authored": "traveller.code.gas_giants",
     "source": {"from": "count", "tag": "traveller.count.gas-giant", "max": 33}},
]

SHEET = {
    "id": "traveller.world",
    "label": "Traveller world profile",
    "about": "The world profile line (UWP), PBG, trade codes, travel zone and bases, read from the world's facts.",
    "prepare": ["traveller.prep."],
    "segments": segments,
    "layouts": {
        "uwp": "{starport}{size}{atmosphere}{hydrographics}{population}{government}{law}-{tech}",
        "pbg": "{pop_mult}{belts}{gas_giants}",
    },
    "fact_prefix": U,
    "groups": [
        {"id": "trade", "label": "Trade codes", "prefix": "traveller.trade."},
        {"id": "zone", "label": "Travel zone", "prefix": "traveller.zone."},
        {"id": "zone_hint", "label": "Travel zone hints", "prefix": "traveller.zonehint."},
        {"id": "bases", "label": "Bases", "prefix": "traveller.base."},
        {"id": "needs", "label": "Needed to go outside", "prefix": "traveller.need."},
    ],
    "system": {"giants": "gas_giants", "belts": "belts", "tag": "traveller-generated"},
    # Sector files (X3c): which columns are code lines, which list codes the
    # rules also read (compared, and kept as traveller.listed.*), and which
    # are the referee's call, written as facts (zone, bases).
    "import": {
        "lines": {"uwp": "uwp", "pbg": "pbg"},
        "profile": "uwp",
        "listed": {"trade": "remarks"},
        "fact_prefix": "traveller.listed.",
        "facts": [
            {"column": "zone", "field": "traveller.zone"},
            {"column": "bases", "field": "traveller.bases.{}"},
        ],
    },
    "rules": RULES,
}

# The extended world profile (Alex 23 Nov: the World Builder's Handbook way,
# as one "big block"), part 1: the physical profile, every figure read off the
# world's own physics and never rolled. One code sheet a GM can grab, swap or
# edit: each line is a fact, its scale and its figures (sse-codes `value`
# segments; `per` names a constant on the master data sheet). The idea of a
# fuller physical profile beside the UWP follows Roy Martin's
# TravellerSystemGenerator (GPL-3.0, credited in sources; no code or tables
# used). WBH tables stay out (Mongoose copyright). The social layer is part 2.
def val(id_, label, field, times=1.0, per=None, digits=3):
    src = {"from": "value", "field": field, "times": times, "digits": digits}
    if per:
        src["per"] = per
    return {"id": id_, "label": label, "source": src}
EXTENDED = {
    "id": "traveller.extended",
    "label": "Extended world profile",
    "about": "The physical profile behind the UWP, read off the world's own physics (never rolled): size, density, gravity, air, surface and orbit. One block: swap it or edit a line.",
    "segments": [
        val("diameter_km", "Diameter (km)", "body.radius", times=2.0e-3, digits=4),
        val("mass", "Mass (Earth masses)", "body.mass", per="M_earth"),
        val("density", "Density (g/cm³)", "body.bulk_density", times=1.0e-3),
        val("gravity", "Surface gravity (g)", "body.surface_gravity", per="g0", digits=2),
        val("escape", "Escape velocity (km/s)", "body.escape_velocity", times=1.0e-3),
        val("pressure", "Surface pressure (atm)", "atmosphere.surface_pressure", per="atm"),
        val("oxygen", "Oxygen partial pressure (bar)", "atmosphere.pp.o2", digits=2),
        val("liquid", "Surface liquid (%)", "surface.liquid_fraction", times=100.0, digits=2),
        val("temperature", "Mean surface temperature (K)", "planet.surface_temperature"),
        val("distance", "Distance from its star (au)", "orbit.semi_major_axis", per="au"),
        val("eccentricity", "Eccentricity", "orbit.eccentricity", digits=2),
        val("year", "Year (Earth years)", "orbit.period", per="yr_julian"),
    ],
    "layouts": {
        "size": "{diameter_km} km across, {mass} Earth masses, {density} g/cm³, {gravity} g, escape {escape} km/s",
        "air": "{pressure} atm at the surface, oxygen {oxygen} bar",
        "surface": "{liquid}% liquid, {temperature} K",
        "orbit": "{distance} au, eccentricity {eccentricity}, a year of {year} Earth years",
    },
    "fact_prefix": U,
}

# The hex map (a core view) as this pack configures it: one hex a parsec,
# subsectors 8 by 10, sectors 4 by 4 subsectors, four-digit labels.
HEX_MAP = {
    "grid": {"block": {"columns": 8, "rows": 10}, "blocks": {"across": 4, "down": 4}, "label": "{c2}{r2}",
             "block_names": list("ABCDEFGHIJKLMNOP"), "shifted": "even", "hex_distance": {"value": 1, "unit": "pc"}},
    "sifters": [
        {"id": "refuel", "label": "Good refuelling stops", "any": [{"field": "gas_giants", "gt": 0}, {"field": "hydrographics", "gt": 0}],
         "explain": "a gas giant to skim or surface water to crack, so a ship can refuel without a port"},
        {"id": "refined", "label": "Refined fuel", "any": [{"field": "starport", "in": ["A", "B"]}],
         "explain": "a class A or B starport sells refined fuel"},
        {"id": "red-candidates", "label": "Red zone candidates", "all": [{"field": "zone_first", "is": "none"}], "any": [{"field": "atmosphere", "in": [11, 12]}, {"field": "law", "gt": 12}],
         "explain": "not yet zoned, but corrosive or insidious air, or extreme law"},
    ],
    "glyphs": {
        "label_field": "name",
        "marks": [
            {"at": "top", "mark": "starport", "text_field": "starport"},
            {"when": {"field": "size", "is": 0}, "at": "centre", "mark": "belt"},
            {"when": {"field": "hydrographics", "gt": 0}, "at": "centre", "mark": "world-wet"},
            {"at": "centre", "mark": "world"},
            {"when": {"field": "gas_giants", "gt": 0}, "at": "top-right", "mark": "gas-giant"},
            {"when": {"field": "bases", "has": "N"}, "at": "top-left", "mark": "star"},
            {"when": {"field": "bases", "has": "S"}, "at": "bottom-left", "mark": "triangle"},
            {"when": {"field": "bases", "has": "W"}, "at": "bottom-left", "mark": "triangle", "tone": "muted"},
            {"when": {"field": "bases", "has": "D"}, "at": "top-left", "mark": "square"},
            {"when": {"field": "zone", "has": "A"}, "at": "ring", "mark": "zone", "tone": "caution"},
            {"when": {"field": "zone", "has": "R"}, "at": "ring", "mark": "zone", "tone": "danger"},
        ],
    },
}

ABOUT = (
    "The Traveller pack, rules part (X3a): world profile codes read from physics by classification rules, so every "
    "digit can say why. An unofficial, free, fan-made aid: it uses no publisher text, artwork or logos, and bundles "
    "no official setting data. Generated by scripts/build-traveller-pack.py."
)
# The notice Mongoose Publishing's Fair Use Policy asks fan work to carry
# (Traveller, 2300AD, Twilight: 2000 and Dark Conspiracy Fair Use Policy,
# 2025, FAQ item 1), word for word. The pack shows it wherever it is listed.
NOTICE = (
    "The Traveller, 2300AD, Twilight: 2000 and Dark Conspiracy games in all forms are owned by Mongoose Publishing. "
    "Copyright 1977 - 2025 Mongoose Publishing. Traveller is a registered trademark of Mongoose Publishing. "
    "Mongoose Publishing permits web sites and fanzines for this game, provided it contains this notice, that "
    "Mongoose Publishing is notified, and subject to a withdrawal of permission on 90 days notice. The contents of "
    "this site are for personal, non-commercial use only. Any use of Mongoose Publishing's copyrighted material or "
    "trademarks anywhere on this web site and its files should not be viewed as a challenge to those copyrights or "
    "trademarks. In addition, any program/articles/file on this site cannot be republished or distributed without "
    "the consent of the author who contributed it."
)
NOTICE_SOURCE = "https://www.mongoosepublishing.com/pages/traveller-licensing (Fair Use Policy, 2025)"

# The game's task difficulties beside SSE4's own words (Alex, 23 Nov): GM-only,
# nothing rolled. From the Cepheus Engine System Reference Document (Samardan
# Press, Open Game Licence): 2D6 plus modifiers against 8+, with difficulty as a
# modifier, and the skills that fly, navigate and keep a ship running. A typical
# trained operator is skill 1 with no characteristic modifier (a game default
# written for this pack). SSE4's words map by target: its "routine" (nothing to
# worry about) is the game's Easy.
DIFFICULTY = {
    "system": "Traveller task",
    "dice": {"count": 2, "sides": 6},
    "base_target": 8,
    "typical_dm": 1,
    "steps": [{"name": n, "dm": d} for n, d in [
        ("Simple", 6), ("Easy", 4), ("Routine", 2), ("Average", 0),
        ("Difficult", -2), ("Very Difficult", -4), ("Formidable", -6)]],
    "words": {"routine": "Easy", "easy": "Routine", "moderate": "Average", "hard": "Difficult", "very hard": "Very Difficult"},
    "skills": {"pilot": "Piloting", "navigator": "Navigation", "engineer": "Engineering"},
    "source": "Cepheus Engine System Reference Document (Samardan Press, Open Game Licence): task difficulty and skills. https://www.orffenspace.com/cepheus-srd/",
}

# Gas mixes (7A part 5), ported from V3's starter-sf atmospheres.json
# ("Traveller-0" to "Traveller-F" and the exotic presets), as composition
# only: the pressure comes from the code's own class. Where a V3 preset would
# read back as another digit under this pack's rules it was moved or changed,
# and its note says so.
V3 = "V3 starter-sf atmospheres.json"
def mix(id_, label, codes, comp, taint=None, weight=1.0, note=None, source=V3):
    m = {"id": f"traveller.mix.{id_}", "label": label, "codes": {"atmosphere": codes},
         "composition": {g.lower(): list(r) for g, r in comp.items()}, "weight": weight, "source": source}
    if taint:
        m["taint"] = taint
    if note:
        m["note"] = note
    return m
GAS_MIXES = [
    mix("0-traces", "vacuum with traces of hydrogen, helium and sodium", ["0"], {"H": (0, 0.9), "He": (0, 0.1), "Na": (0, 0.1)}),
    mix("1-mars", "trace carbon dioxide (ancient Mars)", ["1"], {"CO2": (0.95, 0.98), "N2": (0.02, 0.03), "Ar": (0.01, 0.02)}),
    mix("2-co2", "very thin, tainted by carbon dioxide", ["2"], {"N2": (0.7, 0.8), "CO2": (0.15, 0.25), "O2": (0.1, 0.12)},
        note="V3 had 5 to 10% oxygen, which reads as exotic (under 10% oxygen) here; raised to 10 to 12%"),
    mix("2-sulphur", "very thin, tainted by sulphur dioxide", ["2"], {"N2": (0.8, 0.9), "O2": (0.08, 0.15), "SO2": (0.001, 0.005)}),
    mix("3-alpine", "very thin (alpine)", ["3"], {"N2": (0.75, 0.85), "O2": (0.15, 0.24), "Ar": (0.01, 0.02)}),
    mix("4-smog", "thin, tainted by industrial pollutants", ["4"], {"N2": (0.7, 0.8), "O2": (0.15, 0.2), "CO": (0.002, 0.005), "NO2": (0.001, 0.003)},
        taint="pollutants"),
    mix("5-plateau", "thin (plateau)", ["5"], {"N2": (0.78, 0.82), "O2": (0.18, 0.21), "Ar": (0.01, 0.01)}),
    mix("6-earth", "standard nitrogen and oxygen (Earth-like)", ["6"], {"N2": (0.78, 0.79), "O2": (0.2, 0.21), "Ar": (0.009, 0.01), "CO2": (0.0004, 0.0006)}),
    mix("7-biological", "standard, tainted by pollen and spores", ["7"], {"N2": (0.75, 0.78), "O2": (0.2, 0.23), "H2O": (0.02, 0.05)},
        taint="biological"),
    mix("8-heavy-n2", "dense, heavy in nitrogen", ["8"], {"N2": (0.7, 0.8), "O2": (0.1, 0.15), "He": (0.05, 0.1)}),
    mix("9-ammonia", "dense, tainted by ammonia", ["9"], {"N2": (0.6, 0.7), "O2": (0.15, 0.2), "NH3": (0.005, 0.02)}, taint="ammonia"),
    mix("9-oxygen", "dense, tainted by too much oxygen (a fire hazard)", ["9"], {"N2": (0.5, 0.6), "O2": (0.35, 0.45), "Ar": (0.05, 0.1)}),
    mix("a-ammonia", "exotic: nitrogen and ammonia", ["A"], {"N2": (0.8, 0.9), "NH3": (0.1, 0.2)}),
    mix("a-co2", "exotic: dense carbon dioxide (greenhouse)", ["A"], {"CO2": (0.8, 0.9), "N2": (0.1, 0.2)},
        note="V3's Traveller-D; with no oxygen it reads as exotic here, and D is kept for very dense air with oxygen"),
    mix("a-phosphine", "exotic: reducing phosphine and hydrogen", ["A"], {"PH3": (0.4, 0.6), "H2": (0.3, 0.5), "CH4": (0, 0.2)}, weight=0.2),
    mix("a-carbon", "exotic: reducing carbon monoxide and methane", ["A"], {"CO": (0.5, 0.7), "CH4": (0.2, 0.4), "H2": (0, 0.1)}, weight=0.3),
    mix("a-dense-ammonia", "exotic: ammonia world", ["A"], {"NH3": (0.5, 0.7), "N2": (0.2, 0.4), "CH4": (0, 0.1)}, weight=0.2),
    mix("b-venusian", "corrosive: carbon dioxide with sulphur compounds (Venusian)", ["B", "C"], {"CO2": (0.95, 0.97), "SO2": (0.01, 0.03), "H2SO4": (0.01, 0.02)}),
    mix("b-chlorine", "corrosive chlorine haze", ["B", "C"], {"Cl2": (0.6, 0.8), "N2": (0.1, 0.3), "HCl": (0, 0.1)}, weight=0.2),
    mix("b-fluorine", "hyper-corrosive fluorine", ["B", "C"], {"F2": (0.5, 0.7), "N2": (0.2, 0.4), "HF": (0, 0.1)}, weight=0.2),
    mix("c-halogens", "insidious: nitrogen with chlorine and fluorine", ["C"], {"N2": (0.8, 0.9), "Cl2": (0.05, 0.1), "F2": (0.01, 0.05)},
        note="V3 gave 1 to 5 bar; insidious air here is corrosive at 10 bar and up, so the pressure comes from the code"),
    mix("d-dense", "very dense nitrogen and oxygen", ["D"], {"N2": (0.78, 0.8), "O2": (0.19, 0.21), "Ar": (0.009, 0.01)},
        source="Traveller pack (SSE4)", note="new: V3's D preset had no oxygen (now an exotic mix)"),
    mix("f-methane", "unusual: thin nitrogen and methane (Titan-like)", ["F"], {"N2": (0.9, 0.95), "CH4": (0.05, 0.1)},
        note="V3 labelled it F (thin, low); low air is E here, so this is kept as unusual"),
]

# Sensors and what players can know (7A part 6), ported from V3's starter-sf
# sensors.json. Range bands are the usual Traveller ones; sensor kinds say how
# far they reach and which rung of the knowledge ladder they can lift a body
# to. A ship carries `sensor.<kind>.range_m` and `crew.audience` (whose
# players it carries, e.g. "players"); the trigger below moves a body up the
# ladder for that ship's own audience when a ship with that sensor stays in
# range for an hour (a general trigger and grant, not a Traveller special), so
# each crew learns only what its own ship saw.
KM = 1000.0
RANGE_BANDS = [
    {"id": "adjacent", "label": "adjacent", "range_m": 1 * KM},
    {"id": "close", "label": "close", "range_m": 10 * KM},
    {"id": "short", "label": "short", "range_m": 1250 * KM},
    {"id": "medium", "label": "medium", "range_m": 10000 * KM},
    {"id": "long", "label": "long", "range_m": 25000 * KM},
    {"id": "very_long", "label": "very long", "range_m": 50000 * KM},
    {"id": "distant", "label": "distant", "range_m": 300000 * KM},
    {"id": "very_distant", "label": "very distant", "range_m": 5000000 * KM},
    {"id": "far", "label": "far", "range_m": 150000000 * KM},
]
LADDER = ["long-range", "scanned", "surveyed", "detailed"]
# RK11 (Atomic Rockets plan section 12): each kind is also an instrument the
# kernel's `detect` reads (method, band, aperture, noise, stare), so how far it
# reaches a given target is computed from what that target gives off. V3's
# typed range stays as `authored_range_m` (range_source "authored"), used by the
# hour-in-range triggers below until they can ask `detect`, and by the kernel
# only where an instrument has no figures to compute from (the datalink). The
# figures are game defaults for a Traveller ship's fit (fudge register), in the
# shape of sse-sense's Sensor.
UM = 1e-6
VISIBLE = {"from_m": 0.4 * UM, "to_m": 0.9 * UM}
def inst(name, **kw):
    return {"name": name, **kw}
INSTRUMENTS = {
    "astrometer": inst("wide-field astrometer", **VISIBLE, aperture_m=0.5, method="astrometry", stare_s=600.0),
    "telescope_15au": inst("deep-field telescope", **VISIBLE, aperture_m=1.0, stare_s=600.0),
    "telescope_2au": inst("deep-field telescope (small)", **VISIBLE, aperture_m=0.3, stare_s=60.0),
    "spectral_5au": inst("spectral analyser", **VISIBLE, aperture_m=1.0, method="spectrum", resolving_power=1000.0, stare_s=600.0),
    "spectral_1au": inst("spectral analyser (small)", **VISIBLE, aperture_m=0.3, method="spectrum", resolving_power=1000.0, stare_s=600.0),
    "bio_scanner": inst("bio-scanner (near-infrared imager)", from_m=0.7 * UM, to_m=2.5 * UM, aperture_m=0.5, stare_s=60.0),
    "passive_rf": inst("passive radio antenna", kind="radio", from_m=0.03, to_m=0.3, aperture_m=5.0, bandwidth_hz=1e6, stare_s=10.0),
    "search_radar": inst("active search radar", kind="radio", from_m=0.03, to_m=0.031, aperture_m=3.0, method="active",
                         transmit_power_w=1e5),
    "thermal_ir": inst("thermal and infrared tracker", from_m=3 * UM, to_m=14 * UM, aperture_m=0.3, dark_photons_s=100.0, stare_s=10.0),
    "gravimetric": inst("gravimetric grid", **VISIBLE, aperture_m=1.0, method="gradiometer", noise_density=1e-11, stare_s=600.0),
    "visual_cam": inst("visual inspection camera", **VISIBLE, aperture_m=0.02, stare_s=1.0),
    # A datalink is a handshake, not a sensing channel: no figures, so its
    # range is the authored one.
    "datalink": inst("datalink", **VISIBLE, aperture_m=0.0),
}
def sensor(id_, label, range_km, targets, reveals, up_to):
    return {"id": id_, "label": label, "range_m": range_km * KM, "range_source": "authored", "targets": targets,
            "reveals": reveals, "up_to": up_to, "source": "V3 starter-sf sensors.json",
            "instrument": {**INSTRUMENTS[id_], "authored_range_m": range_km * KM}}
SENSORS = [
    sensor("astrometer", "wide-field astrometer", au_km(50), ["star", "gas giant", "ice giant"], "stellar details, planetary basics", "scanned"),
    sensor("telescope_15au", "deep-field telescope (15 au)", au_km(15), ["terrestrial planet", "belt"], "planetary basics", "scanned"),
    sensor("telescope_2au", "deep-field telescope (2 au)", au_km(2), ["dwarf planet", "moon"], "planetary basics", "scanned"),
    sensor("spectral_5au", "spectral analyser (5 au)", au_km(5), ["gas giant", "ice giant"], "air composition, surface liquids", "scanned"),
    sensor("spectral_1au", "spectral analyser (1 au)", au_km(1), ["terrestrial planet"], "air composition, surface liquids", "scanned"),
    sensor("bio_scanner", "high-resolution bio-scanner", 10000, ["planetary surface"], "biosphere, vegetation, settlements", "surveyed"),
    sensor("passive_rf", "passive radio antenna", au_km(100), ["construct (cooperative)"], "responder ping, transponder id", None),
    sensor("search_radar", "active search radar", 500000, ["construct over 10,000 t"], "construct basics, lock, trajectory, hull", None),
    sensor("thermal_ir", "thermal and infrared tracker", 50000, ["construct"], "drive plumes, heat sinks, power output", None),
    sensor("gravimetric", "gravimetric grid", 25000, ["construct (stealthy)"], "construct basics", None),
    sensor("visual_cam", "visual inspection camera", 50, ["construct"], "visual id, hull markings, damage", None),
    sensor("datalink", "datalink", 0.5, ["construct"], "everything it shares", None),
]
# RK12 (Atomic Rockets plan section 13, question 16): the ship weapons as
# "effect at range". Each kind keeps the game's own line beside the physics:
# tech level, optimum range band and damage dice, from the Cepheus Engine
# System Reference Document (Samardan Press, Open Game Licence; chapter 8
# turret and bay tables, read 24 Nov). The `weapon` is sse-sense's Weapon,
# read by the kernel's `effect_at_range` against a target: what arrives, what
# the skin can take, in range, marginal or out of range, the odds from light
# lag, and rings. Nothing is fired and nothing is rolled. The figures are game
# defaults (fudge register F54), set so that against the standard target
# below each beam is in range or marginal at the far edge of its game band.
# A weapon the physics cannot reach (the meson gun) says so and carries none.
CEPHEUS_SRD = "Cepheus Engine System Reference Document (Samardan Press, Open Game Licence), chapter 8 armament tables. https://www.orffenspace.com/cepheus-srd/book2/ship-design-and-construction.html"
def band_of(id_):
    i = [b["id"] for b in RANGE_BANDS].index(id_)
    return {"band": id_, "from_m": RANGE_BANDS[i - 1]["range_m"] if i else 0.0, "to_m": RANGE_BANDS[i]["range_m"]}
def weapon(id_, label, mount, tl, optimum, damage, w, about):
    game = {"tl": tl, "optimum": band_of(optimum) if optimum else None, "damage": damage, "source": CEPHEUS_SRD}
    return {"id": id_, "label": label, "mount": mount, "game": game, "weapon": w, "about": about}
def laser(power_w, dwell_s, aperture_m, wavelength_m=1.06 * UM, efficiency=0.3):
    return {"kind": "beam", "beam_power_w": power_w, "wavelength_m": wavelength_m, "aperture_m": aperture_m,
            "dwell_s": dwell_s, "efficiency": efficiency, "reject_k": 600.0}
WEAPONS = {
    "about": "Ship weapons as effect at range: the game's tech level, optimum band and damage beside what the physics says arrives. Nothing is fired.",
    # A small armed hull as the shooter sees it: 2 mm of aluminium over the
    # side it shows, 15 m across, able to pull 2 g to spoil an aim.
    "standard_target": {"label": "a small armed hull (2 mm aluminium, 15 m, 2 g)",
                        "skin": {"material": "aluminium", "thickness_m": 0.002}, "radius_m": 15.0, "accel_m_s2": 19.6},
    "kinds": [
        weapon("pulse_laser", "pulse laser", "turret", 7, "short", "2D6",
               laser(5.0e8, 0.1, 2.0, wavelength_m=0.35 * UM), "short, hard ultraviolet pulses: a lot of energy on a spot, but the spot grows with distance"),
        weapon("beam_laser", "beam laser", "turret", None, "medium", None,
               laser(5.0e7, 10.0, 3.0, wavelength_m=0.35 * UM),
               "a steady beam held on the spot; the SRD gives no stat line, so the band is the one its prose puts above the pulse laser"),
        weapon("particle_beam", "particle beam", "turret", 8, "long", "3D6 + radiation",
               {"kind": "beam", "beam_power_w": 2.0e8, "wavelength_m": 1.0e-12, "aperture_m": 0.5, "jitter_rad": 1.0e-7,
                "dwell_s": 10.0, "efficiency": 0.2, "reject_k": 800.0},
               "a neutral particle beam: its spread is its divergence, not diffraction"),
        weapon("particle_bay", "particle beam bay", "bay", 8, "long", "6D6 + radiation",
               {"kind": "beam", "beam_power_w": 1.0e9, "wavelength_m": 1.0e-12, "aperture_m": 1.0, "jitter_rad": 1.0e-7,
                "dwell_s": 10.0, "efficiency": 0.2, "reject_k": 800.0},
               "the bay's larger accelerator: five times the energy, so it is still in range at the far edge of long"),
        weapon("fusion_bay", "fusion gun bay", "bay", 12, "medium", "5D6",
               {"kind": "beam", "beam_power_w": 5.0e9, "wavelength_m": 1.0e-12, "aperture_m": 1.0, "jitter_rad": 1.0e-6,
                "dwell_s": 6.0, "efficiency": 0.5, "reject_k": 1200.0},
               "a plasma bolt: it spreads ten times faster than a particle beam, so its reach is shorter"),
        weapon("meson_bay", "meson gun bay", "bay", 11, "long", "5D6 + radiation", None,
               "fictional physics (mesons that decay inside the target): no figures to compute, so the game's band stands alone"),
        weapon("nuclear_missile", "nuclear missile", "turret", 6, None, "2D6 + radiation",
               {"kind": "burst", "yield_j": 4.184e12, "xray_share": 0.7},
               "a kiloton warhead read where it bursts: in vacuum it must burst within about 50 m of a small hull to boil through; how it gets there is the missile's flight (NAV)"),
        weapon("standard_missile", "standard missile", "turret", 6, None, "1D6",
               {"kind": "projectile", "mass_kg": 50.0, "muzzle_m_s": 3000.0, "face_m2": 0.05},
               "a kinetic and fragment hit at its closing speed; how it gets there is the missile's flight (NAV)"),
        weapon("sandcaster", "sandcaster", "turret", 7, "close", "special", None,
               "a defence, not a weapon at range: no figures"),
    ],
}
def ladder_to(top):
    return LADDER[: LADDER.index(top) + 1]
TRIGGERS = {"rules": [
    {
        "id": f"traveller.sense.{s['id']}",
        "when": {"on_condition": {
            "condition": {"if": "compare", "left": "pair.distance_m", "op": "le", "right": f"observer.sensor.{s['id']}.range_m"},
            "hold_s": 3600,
        }},
        "scope": {"pairs": {"observer_tag": "ship"}},
        "effects": [{"effect": "grant", "audience": "{observer.crew.audience}", "ladder": ladder_to(s["up_to"])}],
        "explain": f"a ship's {s['label']} within {s['range_m'] / KM:,.0f} km for an hour reveals {s['reveals']}",
    }
    for s in SENSORS if s["up_to"]
]}
OBSERVE = {
    "disclosure": {
        "fields": {
            "name": "obvious", "kind": "obvious", "body.radius": "obvious", "colour": "obvious", "spectral": "obvious",
            "map.*": "obvious", "traveller.listed.*": "obvious", "traveller.zone": "obvious",
            "orbit.*": "scanned", "body.mass": "scanned", "atmosphere.*": "scanned", "surface.*": "scanned",
            "luminosity_w": "scanned", "hazards": "scanned",
            "interior.*": "surveyed", "resources": "surveyed", "settlements.*": "surveyed", "society.*": "surveyed",
            "traveller.bases.*": "surveyed",
            "geology.*": "detailed", "life.*": "detailed",
            "gm.*": "secret", "story.*": "secret",
        },
        "tags": {},
    },
    "views": [
        {"id": "long-range", "label": "Long-range", "description": "Exists, name, size, colour; what the charts list",
         "show_tiers": ["obvious"], "tags": {"mode": "none"}, "unknown_style": "placeholder"},
        {"id": "scanned", "label": "Scanned", "description": "Orbit, mass, air, temperature, hazards",
         "show_tiers": ["obvious", "scanned"], "tags": {"mode": "none"}, "unknown_style": "placeholder"},
        {"id": "surveyed", "label": "Surveyed", "description": "Adds interiors, resources, people and bases",
         "show_tiers": ["obvious", "scanned", "surveyed"], "tags": {"mode": "all"}, "unknown_style": "placeholder"},
        {"id": "detailed", "label": "Detailed survey", "description": "Adds geology and life",
         "show_tiers": ["obvious", "scanned", "surveyed", "detailed"], "tags": {"mode": "all"}, "unknown_style": "placeholder"},
    ],
    "campaign": {"default_view": "long-range", "before": "long-range", "after": "scanned"},
}

# Calendar (7A part 8): the Imperial date form, written as day of the year then
# year, over the campaign clock. 365 days of 86 400 s, no long years; day 1 is a
# holiday that stands outside the weeks, so weeks of seven run from day 2. The
# campaign sets the anchor (which universe second day 1 of which year starts).
IMPERIAL = {
    "name": "Imperial",
    # Campaigns set their own epoch (the universe second at which day 001 of
    # epoch_year begins); 0 and 1105 are a placeholder starting point.
    "epoch_s": 0.0,
    "epoch_year": 1105,
    "day_s": REF["day"],
    "year_days": 365,
    "format": "{day:03}-{year}",
    # Optional extras, checked by the kernel at load (J calendars) and read by
    # the UI formatter (packages/ui-contract/src/calendar.ts).
    "named_days": [{"day": 1, "name": "Holiday", "outside_weeks": True}],
    "week": {"names": ["Wonday", "Tuday", "Thirday", "Forday", "Fiday", "Sixday", "Senday"], "first_day": 2, "continuous": False},
}

# Units (7A part 8). Parsecs and g already exist as length and acceleration;
# the displacement ton is a volume (14 cubic metres, about one ton of liquid
# hydrogen), and credits are money, which has no physical dimension, so they
# are words, not units.
UNITS = [
    {"id": "dton", "symbol": "dT", "dim": [3, 0, 0, 0, 0, 0, 0], "scale": 14.0},
    {"id": "g_std", "symbol": "g", "dim": [1, 0, -2, 0, 0, 0, 0], "scale": REF["g0"]},
]

# Words (7A part 8): general terms as the game says them. Reports and views
# look a term up here and fall back to their own word.
TERMS = {
    "planet": "world", "main world": "mainworld", "spacecraft": "ship",
    "light year": "light year", "parsec": "parsec", "money": "credits",
    "money_symbol": "Cr", "technology level": "tech level", "law level": "law level",
    "space station": "highport", "surface port": "downport", "star system": "system",
    "region": "subsector", "large region": "sector", "travel advisory": "travel zone",
    "volume_unit": "dton", "acceleration_unit": "g_std", "distance_unit": "pc",
    "calendar": "traveller.imperial",
}

# Stock ships, stations and sites (7A part 7) as J7 templates: `create`
# makes `<under>/<kind>:<n>` with these facts, tags and kept values. Ported
# from V3's starter-sf construct_templates.json ("traveller" group and the
# two generic craft of the "ship" group), with setting names, trademarks and
# franchise craft left out: no "Imperial", "IISS", "Xboat" or "TAS", and none
# of V3's film and TV ships. The standard ships (below) are hull roles
# written for this pack (numbers only, no rules text); their drives are
# fictional and tagged so. Ships carry sensors as `sensor.<kind>.range_m`,
# so the sensor triggers above work on them as made; set `crew.audience` on a
# ship to say whose players it carries.
SENSOR_RANGE = {s["id"]: s["range_m"] for s in SENSORS}
def sensors(*ids):
    return {f"sensor.{i}.range_m": SENSOR_RANGE[i] for i in ids}
V3C = "V3 starter-sf construct_templates.json"
def construct(id_, kind, label, explain, *, mass_kg=None, length_m=None, cargo_t=None,
              crew=None, crew_max=None, power_mw=None, modules=(), tags=(), facts=None, source=V3C):
    f = dict(facts or {})
    for k, v in [("construct.mass_kg", mass_kg), ("construct.length_m", length_m),
                 ("construct.cargo_t", cargo_t), ("construct.crew_max", crew_max),
                 ("construct.power_mw", power_mw)]:
        if v is not None:
            f[k] = float(v)
    if modules:
        f["construct.modules"] = list(modules)
    f["construct.source"] = source
    kept = {} if crew is None else {"construct.crew": float(crew)}
    return {"id": id_, "kind": kind, "label": label, "facts": f, "kept": kept,
            "tags": ["construct", kind, *tags], "explain": explain}
def jump_ship(id_, label, explain, hull_dt, thrust_g, jump, cargo_dt, crew_max, sensor_ids, tags=()):
    return construct(id_, "ship", label, explain, crew=crew_max, crew_max=crew_max,
                     tags=("fictional_drive", *tags), source="written for this pack",
                     facts={"traveller.hull_dt": float(hull_dt), "traveller.thrust_g": float(thrust_g),
                            "traveller.jump": float(jump), "traveller.cargo_dt": float(cargo_dt),
                            **sensors(*sensor_ids)})
# Bases and ports as dressing rules (tags design TG12 and TG13): a world
# whose starport class or bases column says so gets a candidate station or
# site from the templates below, which the GM adopts or holds off. The class
# and the bases are the referee's call (imported columns or a planned code);
# a candidate is never placed until adopted, and adopting changes no fact.
def _port(letter):
    return {"if": "is", "field": "society.starport", "value": letter}


def _base(code):
    return {"if": "is", "field": f"traveller.bases.{code}", "value": True}


_BASE_WORDS = {"N": ("naval_base", "high-orbit", "a naval base"), "S": ("scout_base", "high-orbit", "a scout base"),
               "W": ("way_station", "high-orbit", "a way station"), "D": ("naval_depot", "high-orbit", "a naval depot"),
               "M": ("military_base", "surface", "a planetary military base"), "C": ("corsair_base", "high-orbit", "a corsair base")}
DRESSING = {
    "about": "Ports and bases a world's codes call for, as candidates the GM adopts or holds off.",
    "rules": [
        {"id": "traveller.port.highport-a.v1", "applies_to": ["body"], "when": _port("A"), "place": "highport_a", "where": "high-orbit",
         "explain": "a class A starport has a highport in orbit with a shipyard for jump ships"},
        {"id": "traveller.port.highport-b.v1", "applies_to": ["body"], "when": _port("B"), "place": "highport_b", "where": "high-orbit",
         "explain": "a class B starport has a highport in orbit that builds small craft"},
        {"id": "traveller.port.downport.v1", "applies_to": ["body"],
         "when": {"if": "any", "of": [_port("A"), _port("B"), _port("C")]}, "place": "downport", "where": "surface",
         "explain": "a class A, B or C starport has a downport on the surface"},
        {"id": "traveller.port.spaceport-d.v1", "applies_to": ["body"], "when": _port("D"), "place": "spaceport_d", "where": "surface",
         "explain": "a class D starport is a landing pad with unrefined fuel"},
        {"id": "traveller.port.beacon-e.v1", "applies_to": ["body"], "when": _port("E"), "place": "beacon_e", "where": "surface",
         "explain": "a class E starport is a marked patch of ground and a beacon"},
    ] + [
        {"id": f"traveller.base.{code}.v1", "applies_to": ["body"], "when": _base(code), "place": t, "where": w,
         "explain": f"the bases column lists {code}: {words} here"}
        for code, (t, w, words) in _BASE_WORDS.items()
    ],
}

# The standard ships (example packs design 4.1, Alex 23 Nov): the twelve
# starships and six small craft people expect from the game, as hull
# tonnage, jump and thrust ratings, cargo, crew and berths: numbers and role
# names only, never rules text, deck plans or art. Tonnage, jump and thrust
# are the long-published figures; cargo, crew and berths are round values
# written for this pack (see handover/packages/X3g-standard-ships.md). Each
# is both a construct template (create puts one in a system) and a ship the
# voyage planner and the ship designer read (`nav_ships`), so its hull is our
# own, drawn from its parts. Roles name the role programs (voyages design):
# explorer, courier, miner, trader, patrol, pirate.
#   id, label, explain, hull dT, jump, thrust g, cargo dT, crew, berths, sensors, tags
STD = "standard"
STARSHIPS = [
    ("courier", "Scout and courier (100 dT)", "a small fast ship for messages, survey and scouting",
     100, 2, 2, 3, 1, 3, ["astrometer", "telescope_2au", "spectral_1au", "passive_rf"], ["survey", "explorer", "courier"]),
    ("trader", "Free trader (200 dT)", "a slow, cheap jump-1 cargo carrier, the workhorse of short routes",
     200, 1, 1, 82, 4, 6, ["spectral_1au", "passive_rf", "search_radar"], ["trade", "trader"]),
    ("far_trader", "Far trader (200 dT)", "a trader that gives up cargo space for jump-2 fuel",
     200, 2, 1, 64, 5, 6, ["spectral_1au", "passive_rf", "search_radar"], ["trade", "trader"]),
    ("subsidised_merchant", "Subsidised merchant (400 dT)", "a big jump-1 freighter run on a subsidised route between near worlds",
     400, 1, 1, 200, 5, 9, ["spectral_1au", "passive_rf", "search_radar"], ["trade", "trader"]),
    ("subsidised_liner", "Subsidised liner (600 dT)", "a passenger liner on a scheduled jump-3 route",
     600, 3, 1, 120, 9, 21, ["spectral_1au", "passive_rf", "search_radar"], ["trade", "trader"]),
    ("yacht", "Yacht (200 dT)", "a private ship for a wealthy owner and guests, with a small boat aboard",
     200, 1, 1, 15, 4, 6, ["spectral_1au", "passive_rf"], []),
    ("safari_ship", "Safari ship (200 dT)", "a hunting and touring ship with holds for specimens and trophies",
     200, 2, 1, 11, 5, 6, ["telescope_2au", "spectral_1au", "bio_scanner", "passive_rf"], ["explorer"]),
    ("seeker", "Seeker (100 dT)", "a prospector's ship: a scout hull with an ore hold and mining gear",
     100, 2, 2, 25, 2, 2, ["spectral_1au", "gravimetric", "passive_rf"], ["miner"]),
    ("lab_ship", "Laboratory ship (400 dT)", "a floating research station that jumps to its subject",
     400, 2, 2, 10, 5, 15, ["telescope_15au", "spectral_5au", "bio_scanner", "passive_rf"], ["research", "explorer"]),
    ("patrol", "Patrol cruiser (400 dT)", "an armed hull for customs, escort and anti-piracy work",
     400, 3, 4, 20, 11, 9, ["astrometer", "spectral_5au", "search_radar", "thermal_ir", "gravimetric"], ["armed", "patrol"]),
    ("mercenary_cruiser", "Mercenary cruiser (800 dT)", "a troop carrier for hired soldiers, with craft to land them",
     800, 3, 3, 50, 15, 30, ["spectral_5au", "search_radar", "thermal_ir"], ["armed", "patrol"]),
    ("corsair", "Corsair (400 dT)", "a fast armed raider with a big hold for what it takes",
     400, 2, 4, 160, 10, 10, ["search_radar", "thermal_ir", "passive_rf"], ["armed", "outlaw", "pirate"]),
]
#   id, label, explain, hull dT, thrust g, cargo dT, crew, seats, sensors, tags
SMALL_CRAFT = [
    ("launch", "Launch (20 dT)", "a slow utility boat for short hops between ships and ground",
     20, 1, 13, 1, 4, ["visual_cam", "datalink"], []),
    ("ships_boat", "Ship's boat (30 dT)", "a fast general-purpose boat carried by bigger ships",
     30, 6, 3, 1, 10, ["visual_cam", "datalink"], []),
    ("pinnace", "Pinnace (40 dT)", "a boat between orbit and ground for people and light cargo",
     40, 5, 6, 1, 16, ["visual_cam", "datalink"], []),
    ("cutter", "Modular cutter (50 dT)", "a workboat that carries one swappable module: cargo, fuel, a lab or seats",
     50, 4, 30, 2, 4, ["visual_cam", "datalink"], []),
    ("shuttle", "Shuttle (95 dT)", "a big small craft for people and cargo between orbit and ground",
     95, 3, 59, 2, 20, ["visual_cam", "datalink"], []),
    ("fighter", "Fighter (10 dT)", "a one-seat armed craft launched from a carrier or a base",
     10, 6, 0, 1, 0, ["search_radar", "thermal_ir"], ["armed"]),
]


def std_ship(id_, label, explain, hull, jump, thrust, cargo, crew, berths, sensor_ids, tags, small):
    t = jump_ship(id_, label, explain, hull, thrust, jump, cargo, crew + berths, sensor_ids,
                  ("small_craft", "lands", *tags) if small else ("starship", *tags))
    t["kept"]["construct.crew"] = float(crew)
    t["facts"]["traveller.crew"] = float(crew)
    t["facts"]["traveller.berths"] = float(berths)
    t["facts"]["construct.nav_ship"] = f"traveller.{id_}"
    t["facts"]["construct.source"] = "Traveller standard ships: figures for this pack (no rules text or art)"
    return t


# The same ships for the voyage planner and the ship designer (sse-nav
# `Ship`; any pack may list `nav_ships`). Their in-system drive is the
# setting's: thrust held for days at the rated g, drawn as a fusion torch
# with its real waste heat until the future tech packs (FT7, FT8) let the
# setting skip reaction mass and waste heat as marked fictions. The jump is
# the setting's rule (fictional): a tenth of the hull's volume in liquid
# hydrogen per parsec, a week a jump, whole parsecs, out past 100 diameters.
DTON_M3 = UNITS[0]["scale"]
H2_LIQUID_KG_M3 = next(s for s in json.loads((ROOT / "packs/base-reference/pack.json").read_text())["provides"]["reference"]["substances"]
                       if s["id"] == "h2")["phases"]["density_liquid"]["v"]
STOWAGE_KG_M3 = json.loads((ROOT / "packs/base-ships/pack.json").read_text())["provides"]["other"]["ship_design"]["cargo"]["stowage_kg_m3"]
NAV_SETTING = {
    "built_t_per_dt": 4.0,          # authored: a built hull's mass per displacement ton, cargo included (fudge register)
    "drive_exhaust_km_s": 3000.0,   # as base-nav's rescue cutter: a fusion torch (fictional at these thrusts)
    "reaction_share": 0.05,         # propellant carried for the torch, as a share of the dry mass
    "jump_share_per_pc": 0.1,       # the jump rule: a tenth of the hull per parsec
    "days_per_jump": 7.0,
    "limit_diameters": 100.0,
}


def nav_ship(id_, label, hull, jump, thrust, cargo, crew, berths):
    dry = hull * NAV_SETTING["built_t_per_dt"]
    s = {"name": label.split(" (")[0], "accel_g": float(thrust), "sustained": True,
         "exhaust_km_s": NAV_SETTING["drive_exhaust_km_s"],
         "dry_t": dry, "wet_t": round(dry * (1 + NAV_SETTING["reaction_share"]), 6),
         "fuels": ["deuterium"], "refuel": ["docking", "scoop", "ice processor"],
         "crew": crew + berths, "cargo_t": round(cargo * DTON_M3 * STOWAGE_KG_M3 / 1000.0, 6)}
    if jump:
        per_pc = round(NAV_SETTING["jump_share_per_pc"] * hull * DTON_M3 * H2_LIQUID_KG_M3 / 1000.0, 6)
        s["jump"] = {"rating_pc": float(jump), "fuel_t_per_pc": per_pc, "tank_t": round(per_pc * jump, 6),
                     "days_per_jump": NAV_SETTING["days_per_jump"], "whole_parsecs": True, "fuel": "hydrogen",
                     "limit_diameters": NAV_SETTING["limit_diameters"],
                     "rule": f"Traveller jump-{jump} (fictional): up to {jump} parsec{'s' if jump > 1 else ''} in about a week, a tenth of the hull in hydrogen per parsec, from 100 diameters out"}
    return s


NAV_SHIPS = {}
STD_TEMPLATES = []
for (id_, label, explain, hull, jump, thrust, cargo, crew, berths, sens, tags) in STARSHIPS:
    STD_TEMPLATES.append(std_ship(id_, label, explain, hull, jump, thrust, cargo, crew, berths, sens, tags, False))
    NAV_SHIPS[f"traveller.{id_}"] = nav_ship(id_, label, hull, jump, thrust, cargo, crew, berths)
for (id_, label, explain, hull, thrust, cargo, crew, seats, sens, tags) in SMALL_CRAFT:
    STD_TEMPLATES.append(std_ship(id_, label, explain, hull, 0, thrust, cargo, crew, seats, sens, tags, True))
    NAV_SHIPS[f"traveller.{id_}"] = nav_ship(id_, label, hull, 0, thrust, cargo, crew, seats)

TEMPLATES = [
    # Ships: the standard list, then one generic craft from V3.
    *STD_TEMPLATES,
    construct("heavy_lifter", "ship", "Heavy lifter (single stage to orbit)",
              "a reusable methane-oxygen rocket lifting about 100 t to orbit",
              mass_kg=100000, length_m=95, cargo_t=100, crew_max=20, crew=6, power_mw=16,
              modules=["large cargo bay", "header tanks", "heat tiles"], tags=("small_craft", "lands")),
    # Orbital stations.
    construct("highport_a", "station", "Highport, class A", "a major orbital port with a shipyard for jump ships",
              mass_kg=5e10, length_m=2000, cargo_t=1e6, crew_max=50000, crew=5000, power_mw=10000,
              modules=["shipyard (jump ships)", "refined fuel plant", "hostel", "concourse", "customs"],
              tags=("starport", "starport_a", "refined_fuel", "shipyard")),
    construct("highport_b", "station", "Highport, class B", "a good orbital port that builds small craft",
              mass_kg=2e10, crew_max=10000, crew=1000,
              modules=["shipyard (small craft)", "repair deck", "refined fuel plant", "trade concourse"],
              tags=("starport", "starport_b", "refined_fuel", "shipyard")),
    construct("naval_base", "station", "Naval base", "a fleet base: docking, repair, marines and command",
              mass_kg=8e10, crew_max=20000, crew=20000,
              modules=["heavy weapon emplacements", "fleet docking spines", "marine barracks", "brig"],
              tags=("base", "base_naval", "military")),
    construct("naval_depot", "station", "Naval depot", "a sector-scale fleet yard and strategic reserve",
              mass_kg=1e14, crew_max=500000, crew=100000,
              modules=["capital ship drydocks", "reserve fleet anchorages", "strategic fuel reserve", "command"],
              tags=("base", "base_depot", "military", "shipyard")),
    construct("scout_base", "station", "Scout base", "a survey and courier post with long-range sensors",
              mass_kg=5e5, crew_max=500, crew=200,
              modules=["long-range sensors", "courier link", "survey data core", "mess hall"],
              tags=("base", "base_scout"), facts=sensors("astrometer", "telescope_15au", "spectral_5au")),
    construct("way_station", "station", "Way station", "a refuelling and message relay stop on a courier route",
              mass_kg=1e9, crew_max=1000, crew=500,
              modules=["courier tender clamps", "refined fuel reservoirs", "crew lounge"],
              tags=("base", "base_way", "refined_fuel")),
    construct("research_station", "station", "Research station", "a small, isolated laboratory",
              mass_kg=2e5, crew_max=200, crew=50,
              modules=["containment labs", "sensor array", "isolation ward"], tags=("research",)),
    construct("corsair_base", "station", "Corsair base", "a hidden pirate haven for repair, fencing and leisure",
              mass_kg=1e11, crew_max=300, crew=100,
              modules=["hidden hangar", "loot storage", "gambling den"], tags=("base", "base_corsair", "outlaw")),
    # Surface sites.
    construct("downport", "site", "Downport", "a surface port: landing grid, tower, warehouses, customs line",
              mass_kg=1e9, crew_max=500, crew=50,
              modules=["landing grid", "control tower", "warehousing", "customs line"], tags=("starport", "unrefined_fuel")),
    construct("spaceport_d", "site", "Spaceport, class D", "a landing pad with unrefined fuel and a radio",
              mass_kg=1e5, crew_max=20, crew=5,
              modules=["landing pad", "fuel pump (unrefined)", "radio shack"], tags=("starport", "starport_d", "unrefined_fuel")),
    construct("beacon_e", "site", "Landing beacon, class E", "a marked patch of ground and a navigation beacon",
              mass_kg=500, modules=["navigation beacon", "wind sock"], tags=("starport", "starport_e")),
    construct("military_base", "site", "Planetary military base", "ground forces, defence silos and a command bunker",
              mass_kg=5e9, crew_max=10000, crew=5000,
              modules=["defence silos", "barracks", "motor pool", "command bunker"], tags=("base", "military")),
    construct("dome_city", "site", "Hardened dome city", "a shielded surface city built for a world with flares",
              crew_max=50000, crew=10000,
              modules=["radiation shielding", "flare warning grid", "storm cellars"], tags=("settlement",)),
    construct("arcology", "site", "Subsurface arcology", "a city underground, lit and fed from within",
              crew_max=200000, crew=50000,
              modules=["geothermal tap", "hydroponic gardens", "artificial sun"], tags=("settlement", "underground")),
    construct("solar_array", "site", "Hardened solar array", "a power farm built to ride out stellar flares",
              modules=["surge breakers", "repair drones"], tags=("power",)),
]
# TG9 (record-logic-programs design 8.1): a ship's or station's discipline
# and readiness are kept values from 0 to 1, never numbers riding on tags.
# The template gives the default (a navy hull is disciplined, a tramp less
# so: game defaults written for this pack, not physics); the GM, a program
# in scope, or one of the status triggers below may write them.
DISCIPLINE = {"courier": 0.8, "trader": 0.6, "far_trader": 0.5, "subsidised_merchant": 0.6, "subsidised_liner": 0.7,
              "yacht": 0.6, "safari_ship": 0.6, "seeker": 0.4, "lab_ship": 0.6, "patrol": 0.9, "mercenary_cruiser": 0.85,
              "corsair": 0.5, "launch": 0.6, "ships_boat": 0.6, "pinnace": 0.6, "cutter": 0.6, "shuttle": 0.6,
              "fighter": 0.9, "heavy_lifter": 0.7,
              "highport_a": 0.7, "highport_b": 0.6, "naval_base": 0.9, "naval_depot": 0.95, "scout_base": 0.8,
              "way_station": 0.6, "research_station": 0.6, "corsair_base": 0.4}
for t in TEMPLATES:
    if t["id"] in DISCIPLINE:
        t["kept"]["ship.readiness"] = 1.0
        t["kept"]["ship.discipline"] = DISCIPLINE[t["id"]]
# Status tags act only through these visible triggers. Derelict sets
# readiness to 0 once, when the tag appears: the GM can then set it to
# anything with the tag left in place. Damaged holds readiness at 0.5 or
# below for as long as the tag is there. Removing a tag restores nothing.
STATUS_RULES = [
    {"id": "traveller.status.derelict",
     "when": {"on_condition": {"condition": {"if": "tag", "tag": "status.derelict"}}},
     "scope": {"tag": "construct"},
     "effects": [{"effect": "set_kept", "kind": "ship.readiness", "value": 0.0,
                  "reason": "tagged derelict"}],
     "explain": "a construct tagged derelict drops to readiness 0 (the GM may set it otherwise afterwards)"},
    {"id": "traveller.status.damaged",
     "when": {"on_condition": {"condition": {"if": "all", "of": [
         {"if": "tag", "tag": "status.damaged"},
         {"if": "range", "field": "ship.readiness", "min": 0.5000001}]}}},
     "scope": {"tag": "construct"},
     "effects": [{"effect": "set_kept", "kind": "ship.readiness", "value": 0.5,
                  "reason": "tagged damaged: readiness at most 0.5"}],
     "explain": "a construct tagged damaged runs at readiness 0.5 at most until the tag is taken off"},
]
TRIGGERS["rules"].extend(STATUS_RULES)
KEPT_WRITERS = ["gm", *[f"template.{t['id']}" for t in TEMPLATES]]
RECORD = {"kept": [
    {"id": "construct.crew", "shape": "number", "range": [0.0, 1.0e7], "default": 0, "tier": "scanned",
     "writers": KEPT_WRITERS,
     "describes": "people aboard or living there now"},
    {"id": "ship.readiness", "shape": "number", "range": [0.0, 1.0], "default": 1, "tier": "scanned",
     "writers": [*KEPT_WRITERS, *[f"effect.{r['id']}" for r in STATUS_RULES]],
     "describes": "how ready it is to go or to fight now, from 0 (dead in space) to 1 (fully ready)"},
    {"id": "ship.discipline", "shape": "number", "range": [0.0, 1.0], "default": 0.5, "tier": "scanned",
     "writers": KEPT_WRITERS,
     "describes": "how well its crew keeps to orders and drill, from 0 to 1; programs read it (a patrol reacts faster when it is high)"},
]}

pack = {
    "format": "sse.pack/1",
    "id": "sse.traveller",
    "version": "0.1.0",
    "kind": ["game-system"],
    "kernel_range": "^0.1",
    # Alex, 23 Nov: this pack alone is non-commercial, as Mongoose's Fair Use
    # Policy asks of fan work (packs/traveller/LICENCE.md); the rest of the
    # repository, this script included, stays Apache-2.0.
    "licence": "CC-BY-NC-4.0",
    "provides": {"units": UNITS, "other": {
        "about": ABOUT,
        "notice": {"text": NOTICE, "source": NOTICE_SOURCE},
        "credits": [
            {"text": "Cepheus Engine System Reference Document (Samardan Press), Open Game Licence: task difficulty, skills, ship weapon lines.", "url": "https://www.orffenspace.com/cepheus-srd/"},
            {"text": "Roy Martin, TravellerSystemGenerator (GPL-3.0): the idea of an extended world profile beside the UWP; ideas only, no code or tables.", "url": "https://github.com/rtrm/TravellerSystemGenerator"},
        ],
        "difficulty": DIFFICULTY,
        "schemes": SCHEMES,
        "axes": [[k, v] for k, v in sorted(AXES.items())],
        "classes": CLASSES,
        "code_sheets": [SHEET, EXTENDED],
        "views": {"sse.view.hex-map": HEX_MAP},
        # V3 gap 18 (overlay.ts): the sector grid laid over the stars, where the GM puts it, under the HUD.
        "overlays": [{"id": "traveller.sector-grid", "label": "Sector grid", "frame": "galaxy-plane-pc", "grid": "sse.view.hex-map"}],
        "calendar": IMPERIAL,
        "gas_mixes": GAS_MIXES,
        "sensors": {"range_bands": RANGE_BANDS, "kinds": SENSORS, "ladder": LADDER},
        "weapons": WEAPONS,
        "observe": OBSERVE,
        "triggers": TRIGGERS,
        "templates": {"templates": TEMPLATES},
        # The standard ships as the voyage planner and ship designer read them (sse-nav Ship).
        "nav_ships": {"about": "The standard ships for voyages and the ship designer. Hull mass per ton, the torch and the jump rule are this setting's; see nav_setting.",
                      "nav_setting": NAV_SETTING, "ships": NAV_SHIPS},
        "dressing": DRESSING,
        "record": RECORD,
        "vocabulary": {"terms": TERMS, "traveller.trade": TRADE_WORDS, "traveller.bases": BASES,
                       "traveller.zone": {"A": "amber", "R": "red"},
                       "traveller.need": {k: v[0] for k, v in NEEDS.items()},
                       # V3 gap 25: what a player screen says while it joins ({seat}, {brand} filled in).
                       "join": {"joining": "Calling the ship's computer for {brand}…",
                                "waiting": "Link open. Receiving the charts…",
                                "away": "Signal lost. These are the last charts received, read only; they update when the link is back."}},
        # TG1: every tag the pack names is declared (labels only, never physics).
        "tags": json.loads((ROOT / "scripts/traveller-tags.json").read_text()),
    }},
}
out = ROOT / "packs/traveller/pack.json"
out.parent.mkdir(parents=True, exist_ok=True)
text = json.dumps(pack, indent=1) + "\n"
if "--check" in sys.argv:
    if not out.exists() or out.read_text() != text:
        sys.exit("packs/traveller/pack.json is out of date: run python3 scripts/build-traveller-pack.py")
    print("packs/traveller/pack.json is current")
    sys.exit(0)
out.write_text(text)
print(f"wrote {out}: {len(CLASSES)} classes, {len(RULES)} rules")
