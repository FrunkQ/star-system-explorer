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
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
RE = 6.371e6
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
rule("traveller.prep.corrosive", any_(rng("atmosphere.pp.so2", 0.001),
                                      *[is_("atmosphere.main", g) for g in ["so2", "h2s", "cl2", "f2", "hcl", "h2so4"]]))
rule("traveller.prep.tainted", any_(has("atmosphere.taint"), rng("atmosphere.pp.co2", above(0.005)),
                                    rng("atmosphere.pp.o2", above(0.5)), rng("atmosphere.pp.so2", 1e-5)))
rule("traveller.prep.belt", ALWAYS, ["belt"])
# Bodies the PBG digits count.
rule("traveller.count.belt", ALWAYS, ["belt"])
rule("traveller.count.gas-giant", all_(rng("body.radius", 2.5 * RE), rng("body.mass", None, 13 * 1.898e27)), ["body"])

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
    ],
    "system": {"giants": "gas_giants", "belts": "belts", "tag": "traveller-generated"},
    "rules": RULES,
}

# The hex map (a core view) as this pack configures it: one hex a parsec,
# subsectors 8 by 10, sectors 4 by 4 subsectors, four-digit labels.
HEX_MAP = {
    "grid": {"block": {"columns": 8, "rows": 10}, "blocks": {"across": 4, "down": 4}, "label": "{c2}{r2}",
             "block_names": list("ABCDEFGHIJKLMNOP"), "shifted": "even", "hex_distance": {"value": 1, "unit": "pc"}},
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
            {"when": {"field": "zone", "is": "A"}, "at": "ring", "mark": "zone", "tone": "caution"},
            {"when": {"field": "zone", "is": "R"}, "at": "ring", "mark": "zone", "tone": "danger"},
        ],
    },
}

ABOUT = (
    "The Traveller pack, rules part (X3a): world profile codes read from physics by classification rules, so every "
    "digit can say why. Traveller is a registered trademark of Far Future Enterprises; this pack is an unofficial, "
    "fan-made aid that uses no publisher text, artwork or logos, and bundles no official setting data. "
    "Generated by scripts/build-traveller-pack.py."
)

pack = {
    "format": "sse.pack/1",
    "id": "sse.traveller",
    "version": "0.1.0",
    "kind": ["game-system"],
    "kernel_range": "^0.1",
    "licence": "Apache-2.0",
    "provides": {"other": {
        "about": ABOUT,
        "schemes": SCHEMES,
        "axes": [[k, v] for k, v in sorted(AXES.items())],
        "classes": CLASSES,
        "code_sheets": [SHEET],
        "views": {"sse.view.hex-map": HEX_MAP},
        "vocabulary": {"traveller.trade": TRADE_WORDS, "traveller.bases": BASES,
                       "traveller.zone": {"A": "amber", "R": "red"}},
    }},
}
out = ROOT / "packs/traveller/pack.json"
out.parent.mkdir(parents=True, exist_ok=True)
out.write_text(json.dumps(pack, indent=1) + "\n")
print(f"wrote {out}: {len(CLASSES)} classes, {len(RULES)} rules")
