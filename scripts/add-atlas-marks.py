#!/usr/bin/env python3
"""Marks for SSE4's own class atlas (packs/base-atlas) in the shipped symbologies.

The symbologies were made (scripts/make-symbologies.py, since hand-edited) for the SSE 3 classic atlas only, so a world
classed by SSE4's atlas ("rocky", "gas-giant/ammonia-clouds", "surface/temperate-ocean") drew as a plain disc. This adds
each SSE4 atlas class as one more key in every class-keyed table of the diagrammatic and colour-coded symbologies (the
mix inherits them): its shape, shading and colour are its nearest SSE 3 class's, so the two atlases read alike on a
chart, and its code is its own, so a one-colour chart still tells every SSE4 class apart. physical/ringed draws a ring,
as planet/ringed does. Legends keep their hand order; an SSE4 class on a chart joins the end. symbology.names.json
gains the SSE4 classes' plain names, and v3-colour-tokens.json lists each SSE4 class under its twin's V3 colour token, so a
V3 palette override recolours both atlases alike.

Only keys are added; nothing else in the documents is touched (they carry hand edits make-symbologies.py does not know).
Every base class of a drawn scheme must have a mark here, and every scheme must be either drawn or named below as not
drawn, so a new class cannot silently draw as a plain disc: --check fails on any gap, and on documents out of step.

Usage: scripts/add-atlas-marks.py [--check]
"""
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DOCS = ROOT / "packages/shell/src/documents"
ATLAS = ROOT / "packs/base-atlas/pack.json"

# SSE4 class -> (nearest SSE 3 classic class, whose shape, shading and colour it takes; its own code).
MARKS = {
    # physical: what kind of object it is
    "small-body": ("asteroid/c-type", "SB"),
    "small-body/carbonaceous": ("asteroid/c-type", "SBc"),
    "small-body/icy": ("asteroid/comet", "SBi"),
    "small-body/metallic": ("asteroid/m-type", "SBm"),
    "small-body/stony": ("asteroid/s-type", "SBs"),
    "iron-rich": ("planet/iron", "IrR"),
    "rocky": ("planet/terrestrial", "Rk"),
    "water-rich": ("planet/ocean", "WR"),
    "sub-neptune": ("planet/sub-neptune", "SbN"),
    "ice-giant": ("planet/ice-giant", "IcG"),
    "gas-giant": ("planet/gas-giant", "GaG"),
    "gas-giant/ammonia-clouds": ("planet/ammonia-clouds-gas-giant", "GaA"),
    "gas-giant/water-clouds": ("planet/water-clouds-gas-giant", "GaW"),
    "gas-giant/cloudless": ("planet/cloudless-gas-giant", "GaC"),
    "gas-giant/alkali-clouds": ("planet/alkali-metal-clouds-gas-giant", "GaK"),
    "gas-giant/silicate-clouds": ("planet/silicate-clouds-gas-giant", "GaS"),
    "gas-giant/ultra-hot": ("planet/ultra-hot-jupiter", "GaU"),
    "brown-dwarf": ("planet/brown-dwarf", "BrD"),
    # surface: what its surface is like
    "surface/airless": ("planet/barren", "sAl"),
    "surface/molten": ("planet/lava", "sMo"),
    "surface/hot-greenhouse": ("planet/sulfur", "sGh"),
    "surface/hot-desert": ("planet/desert", "sHD"),
    "surface/temperate-desert": ("planet/desert", "sTD"),
    "surface/temperate-ocean": ("planet/earth-like", "sTO"),
    "surface/ocean-covered": ("planet/ocean", "sOc"),
    "surface/cold-desert": ("planet/desert", "sCD"),
    "surface/ice-open-equator": ("planet/ice", "sIE"),
    "surface/frozen": ("planet/ice", "sFr"),
    "surface/ice-shell-ocean": ("planet/subsurface-ocean", "sIS"),
    "surface/hydrocarbon-lakes": ("planet/methane", "sHL"),
    "surface/hycean": ("planet/hycean", "sHy"),
}
# Modifiers that change the mark, as their SSE 3 counterpart does.
RINGS = {"physical/ringed": "planet/ringed"}
# Schemes not drawn as a world's mark: they answer another question than what the world is.
NOT_DRAWN = {
    "air": "a world's air is read on its card and in the air lens, not drawn as its mark",
    "habitability": "who can breathe there is the habitability tag's colour (colour-coded's first match), not a class mark",
}
CLASS_CHANNELS = ("shape", "fill_pattern", "colour", "glyph")


def atlas():
    return json.loads(ATLAS.read_text())["provides"]["other"]


def gaps(a):
    """Every way the table above falls short of the atlas, in words."""
    out = []
    schemes = {s["id"] for s in a["schemes"]}
    for s in sorted(schemes - set(NOT_DRAWN)):
        for c in a["classes"]:
            if c["scheme"] == s and c["kind"] == "base" and c["id"] not in MARKS:
                out.append(f"{c['id']} ({s}) has no mark: add it to MARKS, or its scheme to NOT_DRAWN")
    ids = {c["id"] for c in a["classes"]}
    for k in [*MARKS, *RINGS]:
        if k not in ids:
            out.append(f"{k} is marked but is not in the atlas")
    for s in NOT_DRAWN:
        if s not in schemes:
            out.append(f"NOT_DRAWN names {s}, which is not a scheme of the atlas")
    codes = [g for _, g in MARKS.values()]
    if len(set(codes)) != len(codes):
        out.append("two SSE4 classes share a code")
    return out


def marked(doc):
    """The document with every SSE4 class key added (or put right); other keys untouched."""
    doc = json.loads(json.dumps(doc))
    for b in doc["bindings"]:
        if b.get("from") != "class" or "table" not in b.get("scale", {}):
            continue
        t = b["scale"]["table"]
        if b["channel"] == "ring":
            for k, twin in RINGS.items():
                if twin in t:
                    t[k] = t[twin]
            continue
        if b["channel"] not in CLASS_CHANNELS:
            continue
        theirs = {k: v for k, v in t.items() if k not in MARKS}
        for k, (twin, code) in MARKS.items():
            if b["channel"] == "glyph":
                theirs[k] = code
            elif twin in t:
                theirs[k] = t[twin]
        b["scale"]["table"] = theirs
    return doc


def names_doc(doc, a):
    doc = json.loads(json.dumps(doc))
    theirs = {k: v for k, v in doc["names"].items() if k not in MARKS and k not in RINGS}
    for c in a["classes"]:
        if c["id"] in MARKS or c["id"] in RINGS:
            n = c["names"]["plain"]
            theirs[c["id"]] = n[0].upper() + n[1:]
    doc["names"] = theirs
    doc["from"] = "packs/sse3-classic, packs/base-atlas"
    return doc


def tokens_doc(doc):
    """A V3 palette override recolours the SSE4 classes too: each joins its twin's V3 colour token."""
    doc = json.loads(json.dumps(doc))
    for t in doc["tokens"].values():
        if "classes" not in t:
            continue
        theirs = [c for c in t["classes"] if c not in MARKS]
        theirs += [k for k, (twin, _) in MARKS.items() if twin in theirs]
        t["classes"] = theirs
    return doc


def dump(doc):
    return json.dumps(doc, indent=1, ensure_ascii=False) + "\n"


def main():
    check = "--check" in sys.argv[1:]
    a = atlas()
    bad = gaps(a)
    for g in bad:
        print(f"add-atlas-marks: {g}", file=sys.stderr)
    stale = []
    for name in ("symbology.diagrammatic.json", "symbology.colour-coded.json", "symbology.names.json", "v3-colour-tokens.json"):
        p = DOCS / name
        now = p.read_text()
        doc = json.loads(now)
        want = dump(names_doc(doc, a) if name == "symbology.names.json" else tokens_doc(doc) if name == "v3-colour-tokens.json" else marked(doc))
        if want != now:
            stale.append(name)
            if not check:
                p.write_text(want)
    if check and stale:
        print(f"add-atlas-marks: out of step: {', '.join(stale)} (run scripts/add-atlas-marks.py)", file=sys.stderr)
    if bad or (check and stale):
        sys.exit(1)
    print("add-atlas-marks: " + ("in step" if check else f"wrote {', '.join(stale) or 'nothing'}"))


if __name__ == "__main__":
    main()
