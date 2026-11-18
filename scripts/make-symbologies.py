#!/usr/bin/env python3
"""Generate the two base symbologies (H8.4A, V12) from the SSE 3 classic
classes in packs/sse3-classic/pack.json, so every binding names a real class.

Diagrammatic: a class family is drawn by shape and fill pattern (one ink).
Colour-coded: each class gets a colour. V3's colour table has not been read
yet, so these colours are PLACEHOLDERS (one hue per family, steps of
lightness), and the document says so; replace them from V3's table.

Usage: scripts/make-symbologies.py  (writes packages/shell/src/documents/symbology.*.json,
including symbology.names.json: plain class names for legends)
"""
import colorsys, json, pathlib

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

def colour(hue, i, n):
    light = 0.38 + 0.30 * (i / max(1, n - 1))
    r, g, b = colorsys.hls_to_rgb(hue / 360, light, 0.55)
    return "#%02X%02X%02X" % (round(r * 255), round(g * 255), round(b * 255))

shape, pattern, col, order = {}, {}, {}, []
by_family = {}
for c in base:
    by_family.setdefault(family_of[c][0], []).append(c)
for c in base:
    fam, sh, pat, hue = family_of[c]
    members = by_family[fam]
    shape[c], pattern[c] = sh, pat
    col[c] = colour(hue, members.index(c), len(members))
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
coloured = {
    "schema": "sse.symbology/1",
    "id": "sse.symbology.colour-coded",
    "name": "Colour-coded",
    "description": "Class by colour on plain discs: SSE as people know it.",
    "placeholder": "Colours are placeholders (one hue per class family) until V3's colour table is read; replace them from it.",
    "bindings": common + [
        {"applies_to": "planet", "channel": "colour", "from": "class", "scale": {"table": col}, "fallback": None},
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
(out / "symbology.names.json").write_text(json.dumps({"schema": "sse.class-names/0", "from": "packs/sse3-classic", "names": names}, indent=1, ensure_ascii=False) + "\n")
print(f"{len(base)} base classes in {len(FAMILIES) + 1} families; {len(mods)} modifiers ({', '.join(mods)})")
