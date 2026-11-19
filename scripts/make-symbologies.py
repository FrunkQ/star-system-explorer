#!/usr/bin/env python3
"""Generate the two base symbologies (H8.4A, V12) from the SSE 3 classic
classes in packs/sse3-classic/pack.json, so every binding names a real class.

Diagrammatic: a class family is drawn by shape and fill pattern (one ink).
Colour-coded: V3's own colours (docs/v3-colours.json, read from V3's
colors.ts), by V3's rule, first match wins: a habitability tag, then a
biosphere, then brown dwarf, ice giant, gas giant, else terrestrial. V3 gave a
body several classes and matched literal ids ("classes contain gas-giant");
V4 gives one base class, so each class takes the colour V3 would give a
typical member: biosphere when its region requires one, else the first of
brown dwarf, ice giant, gas giant whose core region holds the class's typical
values (or its own core midpoints when it lists none). Stars by spectral key,
belts and constructs by kind.

Usage: scripts/make-symbologies.py  (writes packages/shell/src/documents/symbology.*.json,
including symbology.names.json: plain class names for legends)
"""
import json, pathlib

ROOT = pathlib.Path(__file__).resolve().parent.parent
classes = json.load(open(ROOT / "packs/sse3-classic/pack.json"))["provides"]["other"]["classes"]
base = [c["id"] for c in classes if c["kind"] == "base"]
mods = [c["id"] for c in classes if c["kind"] == "modifier"]

# Families: (name, shape, pattern, hue in degrees, members). Every base class is in exactly one.
FAMILIES = [
    ("rocky", "disc", "solid", 30, "dwarf-planet sub-earth terrestrial super-earth mega-earth supermassive-terrestrial iron silicate coreless carbon barren crater mesoplanet chthonian"),
    ("hot", "disc", "hatch", 10, "lava desert hot-eyeball"),
    ("chemical", "disc", "dash", 300, "sulfur phosphorus chlorine fluorine"),
    ("living water", "disc", "band", 140, "ocean hycean earth-analogue earth-like superhabitable eyeball forest jungle swamp"),
    ("ice", "disc", "hollow", 200, "ice subsurface-ocean cold-eyeball methane ammonia-planet"),
    ("neptunes", "disc", "cross", 225, "mini-neptune sub-neptune ice-giant super-neptune hot-neptune ultra-hot-neptune"),
    ("gas giants", "disc", "stipple", 40, "gas-giant mini-jupiter super-jupiter hot-jupiter ultra-hot-jupiter puffy super-puff cloudless-gas-giant water-clouds-gas-giant ammonia-clouds-gas-giant alkali-metal-clouds-gas-giant silicate-clouds-gas-giant helium"),
    ("substellar", "diamond", "solid", 0, "sub-brown-dwarf brown-dwarf ultra-cool-dwarf"),
    ("built", "square", "grid", 50, "ecumenopolis"),
    ("unbound and young", "triangle", "solid", 270, "rogue protoplanet planetesimal"),
]
SMALL = ("square", "solid", 20, "c-type s-type m-type comet")

family_of = {}
for name, shape, pattern, hue, members in FAMILIES:
    for m in members.split():
        family_of[f"planet/{m}"] = (name, shape, pattern, hue)
for m in SMALL[3].split():
    family_of[f"asteroid/{m}"] = ("small bodies", SMALL[0], SMALL[1], SMALL[2])
missing = [c for c in base if c not in family_of]
extra = [c for c in family_of if c not in base]
assert not missing and not extra, (missing, extra)

V3 = json.load(open(ROOT / "docs/v3-colours.json"))
BODY = {b["key"]: b for b in V3["bodies"]}
by_id = {c["id"]: c for c in classes}

def centres(c):
    t = {a["axis"]: a["centre"] for a in c.get("typical") or []}
    for r in c["region"]:
        if "core" in r and r["axis"] not in t:
            t[r["axis"]] = (r["core"][0] + r["core"][1]) / 2
    return t

def inside(c, target):
    t, hits = centres(c), 0
    for r in by_id[target]["region"]:
        if "core" in r and r["axis"] in t:
            if not (r["core"][0] <= t[r["axis"]] <= r["core"][1]):
                return False
            hits += 1
    return hits > 0

def v3_group(cid):
    c = by_id[cid]
    if any(r["axis"] == "hasBiosphere" for r in c["region"]):
        return "biosphere"
    for g, target in (("brownDwarf", "planet/brown-dwarf"), ("iceGiant", "planet/ice-giant"), ("gasGiant", "planet/gas-giant")):
        if inside(c, target):
            return g
    return "terrestrial"

shape, pattern, col, order = {}, {}, {}, []
by_family = {}
for c in base:
    by_family.setdefault(family_of[c][0], []).append(c)
for c in base:
    fam, sh, pat, hue = family_of[c]
    members = by_family[fam]
    shape[c], pattern[c] = sh, pat
    col[c] = BODY[v3_group(c)]["hex"]
# Chart codes: a short letter code per class (the cartographer's key), so a
# one-ink chart tells every class apart, not only every family.
def code_candidates(c):
    words = c.split("/")[1].split("-")
    w = "".join(words)
    if len(words) > 1:
        yield "".join(x[0] for x in words).upper()[:3]
        yield (words[0][0] + words[-1][:2]).capitalize()
    yield w[:2].capitalize()
    for i in range(2, len(w)):
        yield (w[0] + w[i]).capitalize()
    for i in range(1, len(w)):
        for j in range(i + 1, len(w)):
            yield (w[0] + w[i] + w[j]).capitalize()
glyph = {}
for c in base:
    taken = {g.lower() for g in glyph.values()}
    glyph[c] = next(g for g in code_candidates(c) if g.lower() not in taken and len(g) <= 3)
names = {c["id"]: c["names"]["plain"].capitalize() for c in classes}
for fam, *_ in FAMILIES + [("small bodies",)]:
    order += by_family.get(fam, [])

common = [
    {"applies_to": "star", "channel": "shape", "from": "kind", "scale": {"table": {"star": "star"}}},
    {"applies_to": "star", "channel": "size", "from": "kind", "scale": {"table": {"star": 7}}},
    {"applies_to": "planet", "channel": "size", "from": "tag:size:", "scale": {"table": {"small": 4.5, "medium": 6, "large": 9, "terrestrial": 6, "giant": 9}}, "fallback": 5},
    {"applies_to": "planet", "channel": "ring", "from": "class", "scale": {"table": {"planet/ringed": True}}, "fallback": False},
    {"applies_to": "moon", "channel": "size", "from": "kind", "scale": {"table": {"moon": 2.5}}},
]
diagrammatic = {
    "schema": "sse.symbology/1",
    "id": "sse.symbology.diagrammatic",
    "name": "Diagrammatic",
    "description": "Class family by shape and shading, in one or two inks: the chart room. Works in print, on e-ink, on a green screen and for colour-blind readers.",
    "bindings": common + [
        {"applies_to": "planet", "channel": "shape", "from": "class", "scale": {"table": shape}, "fallback": "disc"},
        {"applies_to": "planet", "channel": "fill_pattern", "from": "class", "scale": {"table": pattern}, "fallback": "solid"},
        {"applies_to": "planet", "channel": "glyph", "from": "class", "scale": {"table": glyph}, "fallback": None},
    ],
    "legend": order,
}
stars = {x["key"]: x["hex"] for x in V3["stars"] if x["token"] and x["key"] not in ("brown-dwarf", "red-giant")}
groups = {}
for c in base:
    groups.setdefault(v3_group(c), []).append(c)
v3_tokens = {
    "schema": "sse.v3-colour-tokens/0",
    "from": f"docs/v3-colours.json (SSE {V3['source']['version']}, {V3['source']['commit']})",
    "note": "How a V3 palette override (localStorage sse-palette-overrides, {token: hex}) lands on a V4 symbology. Generated by scripts/make-symbologies.py.",
    "tokens": {
        **{BODY[g]["token"]: {"group": g, "classes": cs} for g, cs in groups.items()},
        BODY["habitable"]["token"]: {"group": "habitable", "tags": "habitability/", "values": ["earth-like", "human"]},
        BODY["belt"]["token"]: {"group": "belt", "kind": "belt"},
        BODY["construct"]["token"]: {"group": "construct", "kind": "station"},
        **{x["token"]: {"group": "star", "spectral": x["key"]} for x in V3["stars"] if x["token"] and x["token"].startswith("--star-") and x["key"] != "red-giant"},
    },
}
coloured = {
    "schema": "sse.symbology/1",
    "id": "sse.symbology.colour-coded",
    "name": "Colour-coded",
    "description": "SSE 3's own colours on plain discs: SSE as people know it. Colour tells V3's eight groups apart (habitable, biosphere, brown dwarf, ice giant, gas giant, terrestrial, belt, construct), not every class.",
    "bindings": common + [
        {"applies_to": "planet", "channel": "colour", "from": "class", "scale": {"table": col}, "fallback": None,
         "prefer": [{"from": "tag:habitability/", "scale": {"table": {"earth-like": BODY["habitable"]["hex"], "human": BODY["habitable"]["hex"]}}}]},
        {"applies_to": "star", "channel": "colour", "from": "tag:spectral:", "scale": {"table": stars}, "fallback": None},
        {"applies_to": "belt", "channel": "colour", "from": "kind", "scale": {"table": {"belt": BODY["belt"]["hex"]}}, "fallback": None},
        {"applies_to": "station", "channel": "colour", "from": "kind", "scale": {"table": {"station": BODY["construct"]["hex"]}}, "fallback": None},
        {"applies_to": "planet", "channel": "glyph", "from": "class", "scale": {"table": glyph}, "fallback": None, "when": "one-colour"},
    ],
    "legend": order,
}
mixed = {
    "schema": "sse.symbology/1",
    "id": "sse.symbology.mixed",
    "name": "Shapes and colours",
    "description": "Class by shape and shading and by colour at once: the most readable.",
    "inherits": "sse.symbology.diagrammatic",
    "mix": {"colour": "sse.symbology.colour-coded"},
    "bindings": [],
    "legend": order,
}
out = ROOT / "packages/shell/src/documents"
for name, doc in [("diagrammatic", diagrammatic), ("colour-coded", coloured), ("mixed", mixed)]:
    (out / f"symbology.{name}.json").write_text(json.dumps(doc, indent=1, ensure_ascii=False) + "\n")
(out / "v3-colour-tokens.json").write_text(json.dumps(v3_tokens, indent=1, ensure_ascii=False) + "\n")
(out / "symbology.names.json").write_text(json.dumps({"schema": "sse.class-names/0", "from": "packs/sse3-classic", "names": names}, indent=1, ensure_ascii=False) + "\n")
print(f"{len(base)} base classes in {len(FAMILIES) + 1} families; {len(mods)} modifiers ({', '.join(mods)})")
