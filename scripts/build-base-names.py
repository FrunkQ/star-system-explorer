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


SHEETS = os.path.join(ROOT, "packs", "base-names", "sheets")
STRESS = {"first": "first", "last": "last", "second from last": "penult", "penult": "penult"}


def sheet(path):
    """Read an NM3d name sheet (design 4.11) into a JSON scheme for NM1's engine.

    The plain-text grammar is NM7's to settle; this reader takes the sheets as
    the design thread wrote them. `replace` and `join` have no engine field
    yet: the sheets' forms already put the parts in order, and their `never`
    rules already stop the doubled vowels `replace` would merge.
    """
    head, block, cur = {}, {}, None
    for raw in open(path, encoding="utf-8"):
        line = raw.rstrip("\n")
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        if not line.startswith(" ") and ":" in line:
            key, _, val = line.partition(":")
            key, val = key.strip(), val.strip()
            if val:
                head[key] = val
                cur = None
            else:
                cur = key
                block[cur] = []
            continue
        if cur is None:
            raise SystemExit(f"{path}: an indented line outside a block: {line!r}")
        block[cur].append(line.strip())

    def pairs(text):
        return [p.strip() for p in text.split(",") if p.strip()]

    weights = {}
    for row in block.get("weights", []):
        cls, _, rest = row.partition("=")
        weights[cls.strip()] = [[p.rsplit(" ", 1)[0], int(p.rsplit(" ", 1)[1])] for p in pairs(rest)]
    for row in block.get("sounds", []):
        cls, _, rest = row.partition("=")
        cls = cls.strip()
        listed = rest.split()
        if sorted(s for s, _ in weights.get(cls, [])) != sorted(listed):
            raise SystemExit(f"{path}: class {cls} sounds and weights do not match")
    lo, _, hi = head["length"].partition("-")

    def quoted(text):
        out, rest = {}, text
        while rest.strip():
            k, _, rest = rest.strip().partition(" ")
            if not rest.startswith('"'):
                raise SystemExit(f"{path}: say {k} needs a quoted hint")
            v, _, rest = rest[1:].partition('"')
            out[k] = v
            rest = rest.lstrip(", ")
        return out

    forms = {}
    for row in block.get("forms", []):
        kind, _, form = row.partition("=")
        forms.setdefault(kind.strip(), []).append([form.strip(), 1])
    means = {}
    for row in block.get("means", []):
        k, _, v = row.partition("=")
        means[k.strip()] = v.strip()
    name = head["name"]
    return {
        "id": name.lower(),
        "label": name,
        "kind": head.get("kind", "invented"),
        "script": head.get("script", "latin"),
        "about": head["about"] + " An original invented language (base-names, NM3d); not from any published setting.",
        "sounds": {
            "classes": weights,
            "shapes": [[p.rsplit(" ", 1)[0], int(p.rsplit(" ", 1)[1])] for p in pairs(head["shape"])],
            "length": [int(lo), int(hi or lo)],
            "never": [p.split("+") for p in pairs(head.get("never", ""))],
            "stress": STRESS[head.get("stress", "first")],
            "say": quoted(head.get("say", "")),
            "ipa": dict(p.split(" ", 1) for p in pairs(head.get("ipa", ""))),
        },
        "means": means,
        "forms": forms,
        "follow": sorted(k for k, fs in forms.items() if any("{parent}" in f for f, _ in fs)),
    }


def invented():
    return [sheet(os.path.join(SHEETS, f)) for f in sorted(os.listdir(SHEETS)) if f.endswith(".names.txt")]


# NM3b, English-sounding (design 4.8 B). Real English place-name elements
# with what they meant, joined first + end as English place names are
# (Ash + ton, Thorn + ley). The glosses are our own short wording of
# well-known meanings (NM0: the Smith 1956 and Nottingham lists are under
# copyright and are not copied); the citation is Blackie 1887, public domain.
EN_SRC = "Blackie, C. 1887, Geographical Etymology, 3rd ed. (Project Gutenberg #69018)"
EN_FIRST = [
    # trees and plants
    ("Ash", "ash tree"), ("Ac", "oak"), ("Elm", "elm"), ("Birk", "birch"), ("Thorn", "thorn bush"),
    ("Hasel", "hazel"), ("Aller", "alder"), ("Sal", "willow"), ("Lind", "lime tree"), ("Brack", "bracken"),
    ("Fern", "fern"), ("Brom", "broom"), ("Hay", "hedged land"), ("Rush", "rushes"), ("Reed", "reeds"),
    ("Holly", "holly"), ("Rowan", "rowan"), ("Bly", "blackthorn"), ("Wheat", "wheat"), ("Bar", "barley"),
    ("Rye", "rye"), ("Flax", "flax"), ("Kers", "watercress"), ("Ivy", "ivy"), ("Wood", "wood"),
    # land and water
    ("Heath", "heathland"), ("Moor", "moor"), ("Marsh", "marsh"), ("Fen", "fen"), ("Mere", "lake"),
    ("Brook", "brook"), ("Wel", "spring"), ("Burn", "stream"), ("Beck", "stream"), ("Ford", "river crossing"),
    ("Dene", "valley"), ("Comb", "deep valley"), ("Hope", "hidden valley"), ("Clif", "cliff"), ("Nes", "headland"),
    ("Holm", "river island"), ("Ey", "island"), ("Knap", "hilltop"), ("Edge", "escarpment"), ("Tor", "rocky hill"),
    ("Crag", "crag"), ("Scar", "rock face"), ("Sand", "sand"), ("Chal", "chalk"), ("Clay", "clay"),
    ("Flint", "flint"), ("Stan", "stone"), ("Pen", "hill"), ("Dun", "hill"), ("Bryn", "hill"),
    ("Bran", "steep slope"), ("Wath", "ford"), ("Fleet", "creek"), ("Sea", "sea"), ("Lake", "stream"),
    ("Pool", "pool"), ("Mar", "boundary"), ("Slade", "valley"), ("Hale", "nook of land"), ("Rid", "cleared land"),
    ("Strat", "paved road"), ("Wal", "wall"), ("Win", "pasture"), ("Field", "open land"), ("Hol", "hollow"),
    # beasts and birds
    ("Hart", "stag"), ("Buck", "buck"), ("Ox", "oxen"), ("Swin", "pigs"), ("Shep", "sheep"), ("Gos", "geese"),
    ("Cran", "cranes"), ("Ern", "eagle"), ("Craw", "crows"), ("Hawk", "hawks"), ("Wolf", "wolves"),
    ("Fox", "foxes"), ("Brock", "badger"), ("Otter", "otters"), ("Hen", "wild birds"), ("Duck", "ducks"),
    ("Beo", "bees"), ("Lark", "larks"), ("Ram", "rams"), ("Hind", "hinds"), ("Cat", "wild cats"),
    ("Hun", "hounds"), ("Pid", "peewits"), ("Fin", "woodpecker"), ("Swan", "swans"),
    # look and lie of the land
    ("Red", "red"), ("Black", "black"), ("White", "white"), ("Grey", "grey"), ("Green", "green"),
    ("Brad", "broad"), ("Long", "long"), ("Cold", "cold"), ("Fair", "fair"), ("High", "high"),
    ("Mid", "middle"), ("Nor", "north"), ("Sut", "south"), ("Eas", "east"), ("Wes", "west"),
    ("New", "new"), ("Ald", "old"), ("Lang", "long"), ("Mickle", "great"), ("Lit", "little"),
    ("Hor", "muddy"), ("Smal", "narrow"), ("Ful", "foul"), ("Bright", "bright"),
    ("Sheer", "bright"), ("Wet", "wet"), ("Drey", "dry"), ("Wind", "windy"),
    # people and work
    ("Kings", "the king's"), ("Earls", "the earl's"), ("Knigh", "young men's"), ("Charl", "free farmers'"),
    ("Shire", "shire"), ("Salt", "salt"), ("Mil", "mill"), ("Kirk", "church"), ("Chip", "market"),
    ("Smith", "smiths'"), ("Wool", "wool"), ("Brew", "brew house"), ("Hythe", "landing place"),
]
EN_END = [
    ("ton", "farmstead"), ("ham", "homestead"), ("ley", "woodland clearing"), ("by", "farm"),
    ("thorpe", "outlying farm"), ("wick", "dairy farm"), ("worth", "enclosure"), ("field", "open land"),
    ("ford", "river crossing"), ("bury", "fort"), ("chester", "old fort"), ("stead", "place"),
    ("stow", "meeting place"), ("wold", "high forest"), ("wood", "wood"), ("hurst", "wooded hill"),
    ("den", "valley"), ("dale", "valley"), ("combe", "deep valley"), ("mere", "lake"), ("mouth", "river mouth"),
    ("holm", "river island"), ("ness", "headland"), ("ey", "island"), ("well", "spring"), ("bourne", "stream"),
    ("brook", "brook"), ("beck", "stream"), ("thwaite", "clearing"), ("garth", "enclosure"), ("toft", "homestead"),
    ("hope", "hidden valley"), ("ridge", "ridge"), ("don", "hill"), ("low", "burial mound"), ("hithe", "landing place"),
    ("wich", "salt works"), ("port", "market town"), ("mead", "meadow"), ("ing", "people of"), ("ingham", "home of the people of"),
    ("bridge", "bridge"), ("gate", "road"), ("shaw", "copse"), ("stoke", "outlying farm"), ("cote", "cottage"),
    ("hall", "hall"), ("ock", "little"), ("fold", "animal pen"), ("lea", "meadow"), ("marsh", "marsh"),
    ("moor", "moor"), ("pool", "pool"), ("side", "hillside"), ("stone", "stone"), ("tor", "rocky hill"),
    ("nell", "knoll"), ("hay", "hedged enclosure"), ("ridding", "cleared land"),
    ("wardine", "enclosed farm"), ("cliffe", "cliff"),
]
EN_PRE = [
    ("Great", "great"), ("Little", "little"), ("North", "north"), ("South", "south"), ("East", "east"),
    ("West", "west"), ("Upper", "upper"), ("Lower", "lower"), ("Nether", "lower"), ("Long", "long"),
    ("High", "high"), ("Old", "old"), ("New", "new"), ("Far", "far"),
]
# Long-dead English and British astronomers and explorers (IAU rule: dead
# for many years, known for their science or travels, not for politics,
# war or religion), for craters and other features named after people.
EN_PEOPLE = [
    ("Halley", "Edmond Halley (1656-1742), astronomer"), ("Flamsteed", "John Flamsteed (1646-1719), astronomer"),
    ("Bradley", "James Bradley (1693-1762), astronomer"), ("Herschel", "William Herschel (1738-1822), astronomer"),
    ("Horrocks", "Jeremiah Horrocks (1618-1641), astronomer"), ("Hooke", "Robert Hooke (1635-1703), natural philosopher"),
    ("Maskelyne", "Nevil Maskelyne (1732-1811), astronomer"), ("Airy", "George Airy (1801-1892), astronomer"),
    ("Adams", "John Couch Adams (1819-1892), astronomer"), ("Lassell", "William Lassell (1799-1880), astronomer"),
    ("Huggins", "William Huggins (1824-1910), astronomer"), ("Lockyer", "Norman Lockyer (1836-1920), astronomer"),
    ("Harriot", "Thomas Harriot (1560-1621), astronomer"), ("Digges", "Thomas Digges (1546-1595), astronomer"),
    ("Goodricke", "John Goodricke (1764-1786), astronomer"), ("Pigott", "Edward Pigott (1753-1825), astronomer"),
    ("Michell", "John Michell (1724-1793), natural philosopher"), ("Somerville", "Mary Somerville (1780-1872), astronomer"),
    ("Eddington", "Arthur Eddington (1882-1944), astrophysicist"), ("Jeans", "James Jeans (1877-1946), astrophysicist"),
    ("Hind", "John Russell Hind (1823-1895), astronomer"), ("Challis", "James Challis (1803-1882), astronomer"),
    ("Cook", "James Cook (1728-1779), navigator"), ("Franklin", "John Franklin (1786-1847), explorer"),
    ("Flinders", "Matthew Flinders (1774-1814), navigator"), ("Vancouver", "George Vancouver (1757-1798), navigator"),
    ("Hudson", "Henry Hudson (c.1565-1611), navigator"), ("Frobisher", "Martin Frobisher (c.1535-1594), navigator"),
    ("Dampier", "William Dampier (1651-1715), navigator"), ("Parry", "William Parry (1790-1855), explorer"),
    ("Shackleton", "Ernest Shackleton (1874-1922), explorer"), ("Livingstone", "David Livingstone (1813-1873), explorer"),
    ("Bird", "Isabella Bird (1831-1904), explorer"), ("Kingsley", "Mary Kingsley (1862-1900), explorer"),
    ("Bass", "George Bass (1771-1803), navigator"), ("Speke", "John Hanning Speke (1827-1864), explorer"),
]


def english():
    rows = lambda items: [{"name": n, "means": m, "source": EN_SRC} for n, m in items]
    word = "{list:first}{list:end}"
    forms = {
        "default": [[word, 6], ["{list:pre} " + word, 1]],
        "planet": [[word, 6], ["{list:pre} " + word, 1]],
        "star": [[word, 1]],
        "moon": [[word, 1]],
        "continent": [[word, 1]],
        "settlement": [[word, 6], ["{list:pre} " + word, 2]],
        "peak": [["Mount " + word, 3], [word + " Pike", 1], [word + " Fell", 1]],
        "summit": [["Mount " + word, 1]],
        "shield": [["Mount " + word, 1]],
        "range": [[word + " Hills", 2], [word + " Fells", 1], [word + " Range", 1]],
        "ocean": [[word + " Ocean", 1]],
        "sea": [["Sea of " + word, 1], [word + " Sea", 1]],
        "lake": [["Lake " + word, 1], [word + " Water", 1]],
        "island": [[word + " Island", 1], [word + " Isle", 1]],
        "crater": [["{list:person}", 2], [word, 1]],
        "deep": [[word + " Deep", 1]],
        "ice-cap": [[word + " Ice", 1]],
        "plains": [[word + " Plain", 1], [word + " Flats", 1]],
        "rift": [[word + " Rift", 1]],
        "chasma": [[word + " Gorge", 1]],
        "scarp": [[word + " Edge", 1], [word + " Scar", 1]],
        "port": [["Port " + word, 1]],
        "mine": [[word + " Mine", 1], [word + " Pit", 1]],
        "base": [[word + " Base", 1]],
    }
    return {
        "id": "english",
        "label": "English-sounding",
        "kind": "list",
        "about": (
            "Names built the way English place names are: a first element and an ending, each a real Old English "
            "or Norse place-name element with what it meant (Ash + ton, 'ash tree farmstead'), sometimes with Great, "
            "Little, North or Upper before it. Craters may take the surnames of long-dead British astronomers and "
            "explorers. The meanings are our own short glosses, citing Blackie 1887. First pass, corrections welcome."
        ),
        "lists": {"first": rows(EN_FIRST), "end": rows(EN_END), "pre": rows(EN_PRE),
                  "person": rows(EN_PEOPLE)},
        "forms": forms,
    }


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
                    "schemes": [catalogue(False), catalogue(True), english()] + invented(),
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
