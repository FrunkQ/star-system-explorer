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
import re
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
    moon = "{parent} {roman}" if letters else "{parent} {letter:a}"
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
               "Planets take Roman numerals by orbit (the default) and their moons letters from a (Sol III a). ")
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


# NM3c, Many nations (design 4.8 C). Real words only, from two CC BY 4.0
# datasets that scripts/fetch-name-sources.py extracts into
# sources/many-nations.tsv. The romanisation rules below are ours (a first
# pass by non-speakers), so every row also carries its own script and IPA.
MN_TSV = os.path.join(ROOT, "packs", "base-names", "sources", "many-nations.tsv")
MN_SOURCES = {
    "northeuralex": "NorthEuraLex 0.9, Dellert et al. 2020, Lang Resources & Evaluation 54, 273, "
                    "doi:10.1007/s10579-019-09480-6 (CC BY 4.0)",
    "wold": "World Loanword Database, Haspelmath and Tadmor (eds.) 2009, https://wold.clld.org (CC BY 4.0)",
}
MN_LANGS = [
    # id, label, romanisation
    ("en", "English", "as written"),
    ("zh", "Mandarin", "Hanyu Pinyin with tone marks, from WOLD's numbered pinyin"),
    ("hi", "Hindi", "Hunterian, with long vowels marked, from the IPA"),
    ("es", "Spanish", "as written"),
    ("ar", "Arabic", "close to ALA-LC (ḥ, ṣ, ḍ, ṭ, ẓ, ʿ, ʾ, long vowels marked), from the IPA"),
    ("fr", "French", "as written"),
    ("pt", "Portuguese", "as written"),
    ("ru", "Russian", "BGN/PCGN, without the soft and hard signs"),
    ("ja", "Japanese", "Hepburn with macrons, as WOLD gives it"),
    ("sw", "Swahili", "as written"),
]
# Mix presets (design decision 8). Even is the default: the pack makes no
# claim about who inherits the stars.
MN_PRESETS = {
    "even": ("Many nations", {l: 1 for l, _, _ in MN_LANGS},
             "Each of the ten languages equally."),
    # Rounded total speakers in millions, Ethnologue 26th ed. (2023), as an approximate weighting.
    "speakers": ("Many nations, by speakers", {"en": 1456, "zh": 1138, "hi": 610, "es": 559, "ar": 274,
                                              "fr": 310, "pt": 264, "ru": 255, "ja": 123, "sw": 72},
                 "Weighted by total speakers today (Ethnologue 2023, rounded): a fictional future, not a forecast."),
    # Languages of agencies that had sent a craft to the Moon or beyond by
    # 2025: NASA, Roscosmos (and the Soviet programme), JAXA, CNSA, ISRO,
    # ESA (French and English) and the UAE's Hope mission.
    "agencies": ("Many nations, today's space agencies", {"en": 2, "ru": 1, "ja": 1, "zh": 1, "hi": 1, "fr": 1,
                                                         "ar": 1, "es": 0, "pt": 0, "sw": 0},
                 "The languages of agencies that had sent a craft to the Moon or beyond by 2025 (authored)."),
}

TONES = {"a": "āáǎà", "e": "ēéěè", "i": "īíǐì", "o": "ōóǒò", "u": "ūúǔù", "ü": "ǖǘǚǜ"}


def pinyin(numbered):
    """'xing1xing' -> 'xīngxing', 'hai3an4' -> "hǎi'àn" (the standard tone rules)."""
    out = []
    for syl, tone in re.findall(r"([a-zü]+)(\d?)", numbered.replace("v", "ü")):
        t = int(tone) if tone else 5
        if 1 <= t <= 4:
            at = next((v for v in ("a", "e") if v in syl), None)
            if at is None:
                at = "o" if "ou" in syl else [c for c in syl if c in TONES][-1]
            i = syl.index(at) if at in ("a", "e") or "ou" in syl else syl.rindex(at)
            syl = syl[:i] + TONES[at][t - 1] + syl[i + 1:]
        if out and syl[0] in "aeoāáǎàēéěèōóǒò":
            syl = "'" + syl
        out.append(syl)
    return "".join(out)


RU = dict(zip("абвгдезийклмнопрстуфхцчшщыэ", "a b v g d e z i y k l m n o p r s t u f kh ts ch sh shch y e".split()))
RU.update({"ё": "yo", "ж": "zh", "ю": "yu", "я": "ya", "ъ": "", "ь": ""})


def russian(word):
    out, prev = [], ""
    for c in word.lower():
        if c == "е" and (not prev or prev in "аеёиоуыэюяъь"):
            out.append("ye")
        else:
            out.append(RU[c])
        prev = c
    return "".join(out)


# IPA symbols to letters. Longest first; a following "ː" on a vowel marks
# it long and on a consonant doubles it.
HI_CONS = {"t͡ʃʰ": "chh", "t͡ʃ": "ch", "d͡ʒʰ": "jh", "d͡ʒ": "j", "ɖʰ": "dh", "ʈʰ": "th", "kʰ": "kh", "ɡʰ": "gh",
           "tʰ": "th", "dʰ": "dh", "pʰ": "ph", "bʰ": "bh", "ʈ": "t", "ɖ": "d", "ɳ": "n", "ɽ": "r", "ɕ": "sh",
           "ʃ": "sh", "ʂ": "sh", "ɡ": "g", "ŋ": "n", "ɲ": "n", "ʋ": "v", "w": "v", "j": "y", "ɦ": "h", "x": "kh",
           "r̥": "ri", "tt͡ʃ": "chch"}
HI_CONS.update({c: c for c in "kgtdpbmnlrsshzfqv"})
HI_VOW = {"aː": "ā", "iː": "ī", "uː": "ū", "eː": "e", "oː": "o", "ɛː": "ai", "ɔː": "au", "ə": "a", "ɪ": "i",
          "ʊ": "u", "a": "a", "i": "i", "u": "u", "e": "e", "o": "o", "ɛ": "ai", "ɔ": "au"}
AR_CONS = {"d͡ʒ": "j", "sˤ": "ṣ", "dˤ": "ḍ", "tˤ": "ṭ", "ðˤ": "ẓ", "ħ": "ḥ", "ʕ": "ʿ", "ʔ": "ʾ", "x": "kh",
           "ɣ": "gh", "θ": "th", "ð": "dh", "ʃ": "sh", "j": "y"}
AR_CONS.update({c: c for c in "btdrzsfqklmnhw"})
AR_VOW = {"aː": "ā", "iː": "ī", "uː": "ū", "ɨː": "ī", "ɑː": "ā", "a": "a", "i": "i", "u": "u", "ɑ": "a", "ɨ": "i"}


def from_ipa(ipa, cons, vow):
    ipa = ipa.replace("ˑ", "").replace("\u0303ː", "ː\u0303").replace("\u0303", "n")  # nasal vowels written with an n, as Hunterian does
    keys = sorted(list(cons) + list(vow), key=len, reverse=True)
    out, i = [], 0
    while i < len(ipa):
        k = next((k for k in keys if ipa.startswith(k, i)), None)
        if k is None:
            out.append(ipa[i])
            i += 1
            continue
        i += len(k)
        if k in cons:
            letter = cons[k]
            if ipa.startswith("ː", i):  # a doubled consonant: an aspirate doubles its plain stop
                out.append(letter[0] if "ʰ" in k else letter)
                i += 1
            out.append(letter)
        else:
            out.append(vow[k])
    return "".join(out)


def arabic(native, ipa):
    if "ظ" in native and "ض" not in native:
        ipa = ipa.replace("dˤ", "ðˤ")  # the source writes ẓ as ḍ
    ipa = ipa.replace("ɑ", "a")
    word = from_ipa(ipa.replace("au", "aw").replace("ai", "ay").replace("uw", "uː"), AR_CONS, AR_VOW)
    if native.endswith("ى") and word.endswith("an"):
        word = word[:-2] + "ā"  # alif maqṣūra, not the case ending
    if word.endswith("u") and not native.endswith("و"):
        word = word[:-1]  # the case ending some colour words carry
    return word.lstrip("ʾ")


def romanise(lang, native, given, ipa):
    if lang in ("en", "es", "fr", "pt", "sw"):
        return native
    if lang == "ja":
        return given
    if lang == "zh":
        return pinyin(given)
    if lang == "ru":
        return russian(native)
    if lang == "hi":
        return from_ipa(ipa, HI_CONS, HI_VOW)
    return arabic(native, ipa)


def title(word):
    i = 1 if word[:1] in "ʿʾ" else 0
    return word[:i] + word[i:i + 1].upper() + word[i + 1:]


def many_nations():
    with open(MN_TSV, encoding="utf-8") as f:
        lines = [l.rstrip("\n").split("\t") for l in f if l.strip()]
    head, rows = lines[0], lines[1:]
    assert head == ["language", "concept", "gloss", "native", "romanised", "ipa", "source"], head
    by_lang = {l: [] for l, _, _ in MN_LANGS}
    labels = {l: label for l, label, _ in MN_LANGS}
    for lang, _concept, gloss, native, given, ipa, src in rows:
        name = title(romanise(lang, native, given, ipa))
        row = {"name": name, "means": gloss, "language": labels[lang], "source": MN_SOURCES[src]}
        if native.lower() != name.lower():
            row["native"] = native
        if ipa:
            row["ipa"] = ipa
        same = next((r for r in by_lang[lang] if r["name"] == name), None)
        if same:  # one word for two concepts (snow and ice): one row, both meanings
            if gloss not in same["means"].split("; "):
                same["means"] += "; " + gloss
            continue
        by_lang[lang].append(row)
    w = "{list:word}"
    forms = {
        "default": [[w, 1]], "planet": [[w, 1]], "star": [[w, 1]], "moon": [[w, 1]], "continent": [[w, 1]],
        "settlement": [[w, 1]], "crater": [[w, 1]],
        "peak": [["Mount " + w, 1]], "summit": [["Mount " + w, 1]], "shield": [["Mount " + w, 1]],
        "range": [[w + " Mountains", 1], [w + " Range", 1]],
        "ocean": [[w + " Ocean", 1]], "sea": [[w + " Sea", 1]], "lake": [["Lake " + w, 1]],
        "island": [[w + " Island", 1]], "deep": [[w + " Deep", 1]], "ice-cap": [[w + " Ice", 1]],
        "plains": [[w + " Plain", 1]], "rift": [[w + " Rift", 1]], "chasma": [[w + " Gorge", 1]],
        "scarp": [[w + " Scarp", 1]], "port": [["Port " + w, 1]], "mine": [[w + " Mine", 1]],
        "base": [[w + " Base", 1]],
    }
    out = []
    for lang, label, how in MN_LANGS:
        words = sorted(by_lang[lang], key=lambda r: (r["name"].lower(), r["means"]))
        out.append({
            "id": "nations-" + lang,
            "label": "Many nations (%s)" % label,
            "kind": "real",
            "script": "latin",
            "about": (
                "%d real %s words for nature, sky, land, water, seasons and colours, each with its meaning, its "
                "own script and its source. Romanised: %s. Not yet reviewed by a speaker: first pass, "
                "corrections welcome." % (len(words), label, how)
            ),
            "lists": {"word": words},
            "forms": forms,
        })
    for key, (label, weights, about) in MN_PRESETS.items():
        out.append({
            "id": "many-nations" if key == "even" else "many-nations-" + key,
            "label": label,
            "kind": "real",
            "about": about + " Each system is named in one language, with one name in ten from another. "
                     "Feature names put an English term round the word (Mount, Lake), as international maps do.",
            "mix": {"of": [["nations-" + l, weights[l]] for l, _, _ in MN_LANGS], "clump": "sys",
                    "strays_permille": 100},
        })
    return out


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
                    "schemes": [catalogue(False), catalogue(True), english()] + invented() + many_nations(),
                    "avoid": sorted(set(AVOID_WORDS + AVOID_BRANDS)),
                    # Designations are on by default. Common names are the GM's choice of scheme.
                    "defaults": {"designation": "catalogue"},
                }
            }
        },
        "about": (
            "SSE4's built-in name schemes (naming design 4.8): Catalogue designations in two forms, English-sounding, "
            "Many nations (ten real languages, mixed by system, in three presets), the invented languages Vesh and "
            "Oluen, and the avoid "
            "list that generated names are checked against (best effort: no list catches every word in every "
            "language or every trade mark, so every name can be renamed by hand). Built by scripts/build-base-names.py."
        ),
        "notice": [
            "Descriptor terms and naming rules: Gazetteer of Planetary Nomenclature, USGS Astrogeology and the IAU "
            "Working Group for Planetary System Nomenclature (a U.S. Government work).",
            "Designation conventions: International Astronomical Union (CC BY 4.0); Washington Double Star Catalog, "
            "U.S. Naval Observatory.",
            "Many nations words (sources/many-nations.tsv): NorthEuraLex 0.9, Dellert et al. 2020 (CC BY 4.0), "
            "and the World Loanword Database, Haspelmath and Tadmor (eds.) 2009 (CC BY 4.0). Rows were chosen, "
            "some were left out on review, and the romanisation of Mandarin, Hindi, Arabic and Russian is ours.",
            "English-sounding element meanings: our own glosses, citing Blackie 1887 (public domain).",
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
