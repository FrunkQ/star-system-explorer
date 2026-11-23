#!/usr/bin/env python3
"""Build the stock tag packs of tags design TG8 (part 1).

- packs/base-hooks/pack.json (`sse.base-hooks`): `access.*` labels, derived
  by rules over the facts a world carries (V3's `frontier/*` tags, done as
  rules): where a ship can skim fuel, aerobrake, take a gravity assist, mine
  ice or crack open water. Labels only: no rule here feeds physics, and none
  carries a chance, because physics decides each of them.
- packs/intrigue/pack.json (`sse.intrigue`): V3's `intrigue/*` hooks as
  seeded-chance rule tags (TG4). Optional and off by default: it is not in
  the opening stack; a GM adds it from Packs. Its tags carry no tier, so they
  are GM-only until the GM discloses one.

The `resource.*` labels (over the TG11 inventory) are TG8 part 2 and wait on
the inventory blocks. Conditions read SI facts (`body.mass`,
`atmosphere.surface_pressure` in Pa, `planet.surface_temperature` in K) and
the chart room's names (`mass_earth`, `surface_k`). Unit factors come from
the master data sheet. Edit this script, never the JSON.
"""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
REF = json.loads((ROOT / "packs/base-reference/pack.json").read_text())["provides"]["reference"]


def sheet(id_):
    def find(o):
        if isinstance(o, dict):
            if o.get("id") == id_ and isinstance(o.get("value"), dict):
                return o["value"]["v"]
            for v in o.values():
                r = find(v)
                if r is not None:
                    return r
        elif isinstance(o, list):
            for v in o:
                r = find(v)
                if r is not None:
                    return r
        return None

    v = find(REF)
    if v is None:
        raise SystemExit(f"base-reference has no {id_}")
    return v


EARTH_KG = sheet("M_earth")


def either(si_field, si_scale, chart_field, lo=None, hi=None):
    """A range on a fact under both namings: SI (`si_field`, the bound times
    `si_scale`) or the chart room's (`chart_field`, the bound as written)."""
    def rng(field, scale):
        r = {"if": "range", "field": field}
        if lo is not None:
            r["min"] = lo * scale
        if hi is not None:
            r["max"] = hi * scale
        return r
    alts = [rng(si_field, si_scale)]
    if chart_field:
        alts.append(rng(chart_field, 1.0))
    return alts[0] if len(alts) == 1 else {"if": "any", "of": alts}


def all_of(*ps):
    return {"if": "all", "of": list(ps)}


def not_(p):
    return {"if": "not", "p": p}


NOT_STAR = all_of(not_({"if": "has", "field": "star.effective_temperature"}), not_({"if": "has", "field": "mass_sun"}))
GIANT = all_of(NOT_STAR, either("body.mass", EARTH_KG, "mass_earth", 10, 4000))
SOLID = all_of(NOT_STAR, either("body.mass", EARTH_KG, "mass_earth", None, 10))
# 100 Pa at the surface: Mars (about 600 Pa) has been aerobraked by orbiters
# (Mars Global Surveyor, Odyssey, MRO); Pluto (about 1 Pa) is far too thin.
AIR = {"if": "any", "of": [GIANT, {"if": "range", "field": "atmosphere.surface_pressure", "min": 100.0}]}
COLD = either("planet.surface_temperature", 1.0, "surface_k", None, 150)


def rule(id_, when, about):
    return {"id": f"{id_}.v1", "tag": id_, "applies_to": ["body"], "when": when, "_about": about}


ACCESS = [
    rule("access.gas-skimming", GIANT,
         "a giant: a ship with scoops can skim hydrogen from its air for fuel"),
    rule("access.aerobraking", AIR,
         "enough air (at least 100 Pa at the surface, as at Mars, or a giant's envelope) to shed speed in a pass through it"),
    rule("access.gravity-assist", all_of(NOT_STAR, either("body.mass", EARTH_KG, "mass_earth", 0.05, None)),
         "planet-sized (at least 0.05 Earth masses, as Mercury): a flyby bends a ship's path usefully, as Venus, Earth, Mars and Mercury flybys did for Galileo, Cassini, Rosetta and MESSENGER"),
    rule("access.ice-mining", all_of(SOLID, COLD),
         "a solid world at or below 150 K, where water stays frozen at the surface to be mined"),
    rule("access.water", all_of(SOLID, {"if": "is", "field": "surface.liquid", "value": "water"}),
         "open water at the surface, to crack for fuel and drink"),
]

INTRIGUE = [
    ("intrigue.anomalous-signal", 0.03, "a signal nobody can account for, heard here"),
    ("intrigue.derelict-rumour", 0.05, "a rumour of a derelict in orbit"),
    ("intrigue.uncharted-feature", 0.08, "a surface feature the charts leave out"),
    ("intrigue.legend", 0.02, "a story told about this world, true or not"),
]


def pack(id_, kind, label, about, group, tags, rules):
    return {
        "format": "sse.pack/1", "id": id_, "version": "0.1.0", "kind": [kind], "kernel_range": "^0.1", "licence": "Apache-2.0",
        "provides": {"other": {
            "about": {"label": label, "text": about},
            "tags": {"about": "Tags this pack names (tags design, TG1): labels only, never an input to physics.",
                     "groups": [group], "tags": tags},
            "rules": rules,
        }},
    }


def write(rel, doc):
    out = ROOT / rel
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(doc, indent=1, ensure_ascii=False) + "\n")
    print(f"wrote {out}: {len(doc['provides']['other']['rules'])} rules")


access_tags = [{"id": r["tag"], "label": r["tag"].split(".", 1)[1].replace("-", " "), "group": "access",
                "about": r["_about"], "source": "derived", "tier": "scanned"} for r in ACCESS]
for r in ACCESS:
    del r["_about"]
write("packs/base-hooks/pack.json", pack(
    "sse.base-hooks", "presentation", "Access",
    "Where a ship can refuel, shed speed or take what a world offers, read off its facts (V3's frontier tags, done as rules).",
    {"id": "access", "label": "Access", "about": "What a ship can do at a world: derived from its facts, never set by hand.",
     "applies_to": ["body"]},
    access_tags, ACCESS))

write("packs/intrigue/pack.json", pack(
    "sse.intrigue", "universe", "Intrigue",
    "Story hooks drawn from the seed, for what physics cannot decide. Optional, off until a GM adds it; GM-only until disclosed.",
    {"id": "intrigue", "label": "Intrigue", "about": "Story hooks drawn from the seed: a roll, not a finding.", "applies_to": ["body"]},
    [{"id": t, "label": t.split(".", 1)[1].replace("-", " "), "group": "intrigue", "about": a, "source": "derived"}
     for t, _, a in INTRIGUE],
    [{"id": f"{t}.v1", "tag": t, "applies_to": ["body"], "when": NOT_STAR, "chance": c} for t, c, _ in INTRIGUE]))
