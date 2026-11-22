#!/usr/bin/env python3
"""The built-in naming pack: packs/base-names/pack.json (track NM).

NM3a: the Catalogue scheme (designations, naming design 4.8 A) in its two
forms and the avoid list (4.9). Later packages add English-sounding (NM3b),
Many nations (NM3c) and the two invented languages (NM3d) to the same pack.

Real practice, checked in NM0 (docs/citations/naming.md):
- stars: a survey prefix and a running number, or a coordinate name from sky
  position (J hhmm +ddmm), as catalogues do;
- components of a multiple star: A, B, C (Washington Multiplicity Catalog;
  Hessman et al. 2010);
- planets: Roman numerals by orbit (the default form, design decision 6), or
  lower-case letters from b (the IAU's form; letters go by discovery order,
  which the survey layer supplies, and by orbit otherwise);
- moons: a letter after a numeral ("SX 000102 II b"), a Roman numeral after
  a letter ("SX 000102 b I", as Kepler-1625 b I);
- surface features: the IAU's Latin descriptor terms (Gazetteer of
  Planetary Nomenclature) with the world's name and a number;
- galaxies, clusters, nebulae: prefix and number.
Every designation is unique by construction: numbers come from the caller
(orbit order or a survey's running number), never from a draw.

The avoid list is best effort (design 4.9): a short list of English and
other European profanity and well-known brands, applied to generated names
only. Packs and the GM add their own.

  scripts/build-base-names.py          write the pack
  scripts/build-base-names.py --check  exit 1 if the pack is stale
"""
import json
import os
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(ROOT, "packs", "base-names", "pack.json")

# The IAU descriptor term for each surface feature kind the surface crate
# names (sse-surface named.rs, base-surface naming patterns). Kinds the IAU
# has no term for (settlements, ports, mines, bases) take a plain word.
FEATURE_TERMS = {
    "ocean": "Oceanus",
    "sea": "Mare",
    "lake": "Lacus",
    "continent": "Terra",
    "island": "Insula",
    "range": "Montes",
    "peak": "Mons",
    "summit": "Mons",
    "deep": "Fossa",
    "ice-cap": "Planum",
    "crater": "Crater",
    "shield": "Mons",
    "plains": "Planitia",
    "rift": "Fossae",
    "chasma": "Chasma",
    "corona": "Corona",
    "patera": "Patera",
    "scarp": "Rupes",
    "settlement": "Site",
    "port": "Port",
    "mine": "Mine",
    "base": "Base",
}

SURVEY = "SX"  # SSE4's own survey prefix: fictional, so it names no real catalogue


def catalogue(letters):
    """The Catalogue scheme; `letters` picks the real-practice planet form."""
    planet = "{parent} {letter}" if letters else "{parent} {roman}"
    moon = "{parent} {roman}" if letters else "{parent} {letter}"
    forms = {
        "star": [[SURVEY + " {serial:6}", 1]],
        "star-sky": [[SURVEY + " {coord}", 1]],
        "component": [["{parent} {Letter}", 1]],
        "planet": [[planet, 1]],
        "moon": [[moon, 1]],
        "small-body": [["{parent} {n}", 1]],
        "galaxy": [[SURVEY + "G {serial:6}", 1]],
        "cluster": [[SURVEY + "C {serial:5}", 1]],
        "nebula": [[SURVEY + "N {serial:5}", 1]],
        "stream": [["{parent} Stream {n}", 1]],
        "default": [["{parent} {n}", 1]],
    }
    for kind, term in FEATURE_TERMS.items():
        forms[kind] = [["{parent} " + term + " {n}", 1]]
    return {
        "id": "catalogue-letters" if letters else "catalogue",
        "label": "Catalogue (letters)" if letters else "Catalogue",
        "kind": "catalogue",
        "about": (
            "Formal designations, as surveys write them. "
            + ("Planets take letters from b (the IAU's form) and their moons Roman numerals (as Kepler-1625 b I). "
               if letters else
               "Planets take Roman numerals by orbit (the default) and their moons letters. ")
            + "Stars are SX and a running number, or SX J and their sky position; the components of a multiple star are A, B, C; "
            "surface features take the IAU's Latin descriptor terms with the world's name and a number. "
            "Every designation follows its parent's name when that is renamed. Sources: IAU naming rules, "
            "Hessman et al. 2010 (arXiv:1012.0707), the Washington Multiplicity Catalog, and the USGS and IAU "
            "Gazetteer of Planetary Nomenclature (docs/citations/naming.md)."
        ),
        "forms": forms,
        "follow": sorted(k for k, v in forms.items() if "{parent}" in v[0][0]),
    }


# Best effort (design 4.9). Folded by the engine (case, accents, look-alikes),
# so one spelling each. Entries of 4 letters or more also match inside a name.
AVOID_WORDS = [
    # English
    "arse", "ass", "bastard", "bitch", "bollocks", "bugger", "cock", "crap", "cunt", "damn", "dick", "fag", "fuck",
    "nazi", "nigger", "piss", "prick", "rape", "shit", "slut", "spastic", "twat", "wank", "whore",
    # Spanish, Portuguese, Italian, French, German
    "cabron", "carajo", "cojones", "coño", "joder", "mierda", "puta", "puto", "caralho", "porra", "merda", "buceta",
    "cazzo", "stronzo", "vaffanculo", "merde", "putain", "connard", "salope", "encule", "scheisse", "fotze", "arsch",
    "wichser", "hure",
]
AVOID_BRANDS = [
    "adidas", "amazon", "apple", "coca", "cola", "disney", "facebook", "ferrari", "google", "honda", "ikea", "intel",
    "lego", "marvel", "mcdonald", "microsoft", "netflix", "nike", "nintendo", "pepsi", "samsung", "sony", "starbucks",
    "tesla", "toyota", "twitter", "walmart", "yahoo",
]


def build():
    return {
        "format": "sse.pack/1",
        "id": "sse.base-names",
        "version": "0.1.0",
        "kind": ["presentation"],
        "kernel_range": "^0.1",
        "licence": "Apache-2.0",
        "provides": {
            "other": {
                "names": {
                    "schemes": [catalogue(False), catalogue(True)],
                    "avoid": sorted(set(AVOID_WORDS + AVOID_BRANDS)),
                }
            }
        },
        "about": (
            "SSE4's built-in name schemes (naming design 4.8): Catalogue designations in two forms, and the avoid "
            "list that generated names are checked against (best effort: no list catches every word in every "
            "language or every trade mark, so every name can be renamed by hand). Built by scripts/build-base-names.py."
        ),
        "notice": [
            "Descriptor terms and naming rules: Gazetteer of Planetary Nomenclature, USGS Astrogeology and the IAU "
            "Working Group for Planetary System Nomenclature (a U.S. Government work).",
            "Designation conventions: International Astronomical Union (CC BY 4.0); Washington Double Star Catalog, "
            "U.S. Naval Observatory.",
        ],
    }


def main():
    text = json.dumps(build(), indent=1, ensure_ascii=False) + "\n"
    if "--check" in sys.argv:
        try:
            with open(OUT, encoding="utf-8") as f:
                current = f.read()
        except FileNotFoundError:
            current = None
        if current != text:
            print("build-base-names: %s is stale; run scripts/build-base-names.py" % os.path.relpath(OUT, ROOT))
            sys.exit(1)
        print("build-base-names: up to date")
        return
    os.makedirs(os.path.dirname(OUT), exist_ok=True)
    with open(OUT, "w", encoding="utf-8") as f:
        f.write(text)
    print("build-base-names: wrote %s" % os.path.relpath(OUT, ROOT))


if __name__ == "__main__":
    main()
