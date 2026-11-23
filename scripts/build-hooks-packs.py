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

- packs/world-hooks/pack.json (`sse.world-hooks`): one-line story prompts
  read off a world's facts (example packs design, "World hooks", in our own
  words): a day that lasts weeks, air that crushes, a giant filling the sky.
  Labels only, never physics; each prompt says what physics it read, and
  the threshold sits where that physics changes (oxygen liquefies at 90 K).

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


# World hooks: (id, label, when, prompt, why). The prompt is the story; the
# why is the physics it read. Thresholds are physical where one exists and
# round numbers otherwise (said so in the why).
DAY_S = 86400.0
# Oxygen's normal boiling point: the temperature the sheet gives its liquid density at.
O2_BOILS_K = next(x for x in REF["substances"] if x["id"] == "o2")["phases"]["density_liquid"]["at_k"]
MOON = either("parent.body.mass", EARTH_KG, "parent.mass_earth", 0.05, 4000)
HOOKS = [
    ("hook.long-night", "long night",
     all_of(SOLID, {"if": "any", "of": [{"if": "range", "field": "body.rotation_period", "min": 30 * DAY_S},
                                        {"if": "range", "field": "day_h", "min": 30 * 24.0}]}),
     "A day that lasts weeks: towns roll slowly round the world to stay in the light, or dig in for the long night.",
     "a solid world that turns once in 30 days or more (a round number; Mercury and Venus qualify, the Moon at 27 days just misses)"),
    ("hook.backwards-sun", "backwards sun",
     all_of(NOT_STAR, {"if": "range", "field": "body.obliquity", "min": 120.0}),
     "The sun rises in the west here, and every newcomer's sense of direction is wrong for a week.",
     "an axial tilt over 120 degrees: the world spins backwards against its orbit, as Venus does"),
    ("hook.on-its-side", "on its side",
     all_of(NOT_STAR, {"if": "range", "field": "body.obliquity", "min": 60.0, "max": 120.0}),
     "The world rolls on its side round its star: half the year is endless day at the pole, half endless night, and cities follow the light.",
     "an axial tilt between 60 and 120 degrees, as Uranus at 98 degrees: each pole spends a season facing the star"),
    ("hook.crushing-air", "crushing air",
     all_of(SOLID, {"if": "range", "field": "atmosphere.surface_pressure", "min": 5e6}),
     "The air at the ground presses like the deep sea; only hardened landers, and very brave crews, go down.",
     "a surface pressure of 50 bar or more, the pressure about 500 m under Earth's sea (Venus has 92 bar)"),
    ("hook.thin-air", "thin air",
     all_of(SOLID, {"if": "range", "field": "atmosphere.surface_pressure", "min": 100.0, "max": 1e4}),
     "Air far too thin to breathe but thick enough to raise dust storms: a pressure suit is everyday clothing.",
     "a surface pressure between 100 Pa and 10 kPa, a tenth of Earth's (round numbers); Mars has about 600 Pa"),
    ("hook.furnace", "furnace",
     all_of(SOLID, either("planet.surface_temperature", 1.0, "surface_k", 600, None)),
     "Hot enough to cook a ship's electronics where it lands: anything built here hides in the shade or underground.",
     "a surface at 600 K or more (a round number; Venus, at about 740 K, qualifies)"),
    ("hook.oxygen-rain", "oxygen rain",
     all_of(SOLID, either("planet.surface_temperature", 1.0, "surface_k", None, O2_BOILS_K)),
     "So cold that oxygen would fall as rain: a leaking suit is a race against the cold, measured in minutes.",
     f"a surface at {O2_BOILS_K:.1f} K or less, where oxygen at one atmosphere is a liquid (the master data sheet's boiling point)"),
    ("hook.ocean-world", "ocean world",
     all_of(SOLID, {"if": "range", "field": "surface.liquid_fraction", "min": 0.9}),
     "Water from pole to pole: every settlement floats, and dry land is the rarest thing anyone trades.",
     "liquid over at least nine tenths of the surface"),
    ("hook.heavy-world", "heavy world",
     all_of(SOLID, either("body.mass", EARTH_KG, "mass_earth", 3, 10)),
     "Several times Earth's mass: the ground pulls harder, locals are short and strong, and every launch to orbit costs dear.",
     "a solid world of 3 to 10 Earth masses, with a higher escape speed than Earth's"),
    ("hook.featherweight", "featherweight",
     all_of(SOLID, either("body.mass", EARTH_KG, "mass_earth", None, 0.02)),
     "Gravity so weak that a stumble becomes a long, slow fall, and a thrown rock might not come back down.",
     "a world under 0.02 Earth masses (the Moon is 0.012); its escape speed is a few km/s or less"),
    ("hook.wild-orbit", "wild orbit",
     all_of(NOT_STAR, {"if": "any", "of": [{"if": "range", "field": "orbit.eccentricity", "min": 0.2},
                                           {"if": "range", "field": "eccentricity", "min": 0.2}]}),
     "Its orbit swings near and far: a short, scorching summer at the close pass, a long winter after, and a calendar built round the approach.",
     "an orbital eccentricity of 0.2 or more (Mercury's is 0.21): the starlight at the close pass is at least twice that at the far one"),
    ("hook.sun-filled-sky", "sun-filled sky",
     all_of(NOT_STAR, not_(MOON), {"if": "any", "of": [{"if": "range", "field": "orbit.semi_major_axis", "max": 0.1 * sheet("au")},
                                                        {"if": "range", "field": "a_au", "max": 0.1}]}),
     "So close to its star that the sun fills the sky, and flares are weather.",
     "an orbit within 0.1 au of its star, a quarter of Mercury's distance"),
    ("hook.giant-in-the-sky", "giant in the sky",
     all_of(NOT_STAR, either("parent.body.mass", EARTH_KG, "parent.mass_earth", 10, 4000)),
     "A giant planet fills the sky; people here tell the time by its phases and its shadow.",
     "a moon of a giant planet (its host at least 10 Earth masses)"),
]


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

hook_tags = [{"id": h, "label": label, "group": "hook", "about": prompt, "why": why, "source": "derived", "tier": "scanned"}
             for h, label, _, prompt, why in HOOKS]
write("packs/world-hooks/pack.json", pack(
    "sse.world-hooks", "presentation", "World hooks",
    "One-line story prompts read off a world's physics: what it would be like to live there, in a sentence. Labels only; each says what it read.",
    {"id": "hook", "label": "World hooks", "about": "A story prompt read off a world's facts: never set by hand, never an input to physics.",
     "applies_to": ["body"]},
    hook_tags, [{"id": f"{h}.v1", "tag": h, "applies_to": ["body"], "when": w} for h, _, w, _, _ in HOOKS]))
