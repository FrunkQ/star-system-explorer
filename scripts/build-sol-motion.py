#!/usr/bin/env python3
"""Measured motion for the catalogued Solar System: packs/base-sol/motion/*.json (RW17, design 6A.2).

A measured motion is a published series: long sums of waves in time that give where a body is at any date inside
the span the series was fitted over. This script turns the published tables into the kernel's one table form,
`sse.series/1`, cut to the terms that matter, and records exactly what it kept and what that costs in accuracy.

The one form. Each coordinate is a sum over powers k of t^k times a sum of terms, and each term is
    A * wave(p0 + p1 t + p2 t^2 + ...)
with wave sin or cos as the source prints it, so the numbers are copied as published and never re-derived.

  VSOP87A (Bretagnon and Francou 1988): heliocentric x, y, z in au on the J2000 ecliptic, t in Julian millennia of
    TDB from J2000, terms A cos(B + C t). Mercury to Neptune, and the Earth-Moon barycentre (the kernel places
    Earth's group by its barycentre, and Earth and the Moon round it).
  ELP/MPP02 (Chapront and Francou 2003), fitted to DE405: the Moon's longitude, latitude and distance from Earth's
    centre, t in Julian centuries of TDB from J2000, terms A sin(p0 + p1 t + ... + p4 t^4) (the published
    arguments already combined into one polynomial per term), plus the mean longitude W1, and Laskar's P and Q
    to turn the mean ecliptic of date into the J2000 ecliptic.

Inputs are files on disk, never fetched here (the cloud build machine is refused by the source sites):
  --vsop87a DIR  either IMCCE's own files (VSOP87A.mer ... VSOP87A.emb) or the vsop87 crate's src/vsop87a
                 folder (a transcription of the same files; MIT or Apache-2.0)
  --elp FILE     the combined ELP/MPP02 (DE405) table as the astronomia package ships it (data/elpMppDeFull.js,
                 MIT), which carries the published series with its arguments combined

Each output records the input's sha256 and which form it was read from, so the copy can be checked against the
originals (scripts/check-sol-motion.py, run on a machine that can reach JPL and IMCCE).

  python3 -I scripts/build-sol-motion.py --vsop87a DIR --elp FILE     write the tables
  python3 -I scripts/build-sol-motion.py --check                       exit 1 if a table is malformed
Python standard library only; deterministic output.
"""
import hashlib
import json
import math
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(ROOT, "packs", "base-sol", "motion")
AU_M = 149597870700.0  # IAU 2012 Resolution B2: the au is defined as this many metres (a definition, not a measurement)

BRETAGNON = {
    "id": "bretagnon-francou1988",
    "text": "Bretagnon, P. and Francou, G. 1988, A&A 202, 309: Planetary theories in rectangular and spherical variables. VSOP 87 solutions",
    "status": "unverified",
    "url": "https://ui.adsabs.harvard.edu/abs/1988A&A...202..309B",
}
CHAPRONT = {
    "id": "chapront-francou2003",
    "text": "Chapront, J. and Francou, G. 2003, A&A 404, 735: The lunar theory ELP revisited. Introduction of new planetary perturbations (ELP/MPP02)",
    "status": "unverified",
    "doi": "10.1051/0004-6361:20030529",
}
# (file stem in IMCCE's naming, the vsop87 crate's file, body id in packs/base-sol, name, cut in au, span in years
# either side of J2000). The cut keeps every term whose largest size over the span (A times 4^k for t^k, as t
# reaches 4 millennia) is at least this; the spans are the ones the series' authors give for an arcsecond.
PLANETS = [
    ("mer", "mercury", "mercury", "Mercury", 1e-7, 4000),
    ("ven", "venus", "venus", "Venus", 1e-7, 4000),
    ("emb", "earth_moon", "earth", "the Earth-Moon barycentre", 1e-8, 4000),
    ("mar", "mars", "mars", "Mars", 1e-7, 4000),
    ("jup", "jupiter", "jupiter", "Jupiter", 1e-6, 2000),
    ("sat", "saturn", "saturn", "Saturn", 1e-6, 2000),
    ("ura", "uranus", "uranus", "Uranus", 1e-6, 6000),
    ("nep", "neptune", "neptune", "Neptune", 1e-6, 6000),
]
# The Moon: terms of at least 0.001 arcsec in longitude and latitude and 1 m in distance (at 30 centuries for the
# t^k terms). Span: ELP/MPP02 is fitted to DE405/DE406, which run 3000 BC to AD 3000.
ELP_CUT = {"L": 1e-3, "B": 1e-3, "R": 1e-3}
ELP_SPAN_Y = 3000
J2000_JD = 2451545.0
YEAR_D = 365.25


def sha(path):
    return hashlib.sha256(open(path, "rb").read()).hexdigest()


def read_imcce(path):
    """IMCCE's VSOP87A text file: blocks headed 'VSOP87 VERSION A? ... VARIABLE n (XYZ) *T**k  N TERMS', each term
    line ending in A B C."""
    out = {}
    cur = None
    for line in open(path):
        h = re.search(r"VARIABLE\s+(\d)\s+\(XYZ\)\s+\*T\*\*(\d)\s+(\d+)\s+TERMS", line)
        if h:
            cur = ("XYZ"[int(h.group(1)) - 1], int(h.group(2)))
            out[cur] = []
            continue
        if cur and line.strip():
            f = line.split()
            out[cur].append((float(f[-3]), float(f[-2]), float(f[-1])))
    return out


def read_crate(path):
    """The vsop87 crate's transcription: const X0: [[f64; N]; 3] = [[A...], [B...], [C...]];"""
    s = open(path).read()
    out = {}
    for m in re.finditer(r"const ([XYZ])(\d): \[\[f64; (\d+)\]; 3\] = \[(.*?)\];", s, re.S):
        rows = re.findall(r"\[(.*?)\]", m.group(4), re.S)
        nums = [[math.pi if x == "PI" else float(x) for x in re.findall(r"PI|-?[\d.]+(?:e-?\d+)?", r)] for r in rows]
        n = int(m.group(3))
        if len(nums) != 3 or any(len(x) != n for x in nums):
            raise SystemExit("%s: %s%s does not have 3 rows of %d" % (path, m.group(1), m.group(2), n))
        out[(m.group(1), int(m.group(2)))] = list(zip(*nums))
    return out


def cut_terms(terms, k, eps, reach):
    kept = [list(t) for t in terms if abs(t[0]) * reach ** k >= eps]
    dropped = [abs(t[0]) * reach ** k for t in terms if abs(t[0]) * reach ** k < eps]
    return kept, sum(dropped), math.sqrt(sum(d * d for d in dropped))


def planet(stem, crate, body, name, eps, span_y, src):
    if os.path.exists(os.path.join(src, "VSOP87A." + stem)):
        path, terms, form = os.path.join(src, "VSOP87A." + stem), read_imcce(os.path.join(src, "VSOP87A." + stem)), "IMCCE VSOP87A file"
    else:
        path, terms, form = os.path.join(src, crate + ".rs"), read_crate(os.path.join(src, crate + ".rs")), "vsop87 crate 3.0.0 transcription of IMCCE VSOP87A"
    vars_, kept_n, all_n, bound, rss = [], 0, 0, 0.0, 0.0
    for c in "XYZ":
        powers = []
        for k in range(6):
            t = terms.get((c, k), [])
            all_n += len(t)
            kept, b, r = cut_terms(t, k, eps, 4.0)
            kept_n += len(kept)
            bound = max(bound, b)
            rss = max(rss, r)
            powers.append(kept)
        while powers and not powers[-1]:
            powers.pop()
        vars_.append(powers)
    return {
        "format": "sse.series/1",
        "id": "vsop87a." + stem,
        "body": body,
        "about": "Where %s is: VSOP87A, heliocentric, cut to %d of its %d terms." % (name, kept_n, all_n),
        "centre": "sun",
        "gives": "group" if body == "earth" else "body",
        "frame": "j2000_ecliptic",
        "form": "xyz",
        "wave": "cos",
        "unit_m": AU_M,
        "time": {"epoch_jd": J2000_JD, "unit_days": 365250.0, "scale": "tdb"},
        "span": {"from_jd": J2000_JD - span_y * YEAR_D, "to_jd": J2000_JD + span_y * YEAR_D},
        "accuracy": "About an arcsecond over the span for the full series (Bretagnon and Francou 1988); the cut adds at most %.2g au in any coordinate (every dropped term at its largest, added), typically %.2g au." % (bound, rss),
        "cut": {"min_size": eps, "unit": "au", "at_t": 4.0, "kept": kept_n, "of": all_n, "bound_au": bound, "rss_au": rss},
        "src": BRETAGNON["id"],
        "sources": [BRETAGNON],
        "input": {"form": form, "file": os.path.basename(path), "sha256": sha(path)},
        "licence": "The VSOP87 tables are distributed freely by IMCCE (Paris Observatory) for use with credit to the authors; terms to be read at source by RW1.",
        "vars": vars_,
    }


def read_elp(path):
    s = open(path).read()
    s = s[s.index("{"):s.rindex("}") + 1]
    s = re.sub(r"(\n\s*)(name|W1|L|B|R):", r'\1"\2":', s)
    return json.loads(s)


def moon(path):
    d = read_elp(path)
    if d.get("name") != "ElpMppDE405":
        raise SystemExit("%s is not the DE405 fit of ELP/MPP02 (name %r)" % (path, d.get("name")))
    vars_, kept_n, all_n, notes = [], 0, 0, []
    reach = ELP_SPAN_Y / 100.0
    for v, unit in (("L", "arcsec"), ("B", "arcsec"), ("R", "km")):
        powers = []
        for k in range(4):
            t = d[v].get(str(k), [])
            all_n += len(t)
            kept, b, r = cut_terms(t, k, ELP_CUT[v], reach)
            kept_n += len(kept)
            powers.append(kept)
            if t:
                notes.append("%s t^%d: %d of %d kept, dropped at most %.2g %s (typically %.2g)" % (v, k, len(kept), len(t), b, unit, r))
        while powers and not powers[-1]:
            powers.pop()
        vars_.append(powers)
    return {
        "format": "sse.series/1",
        "id": "elpmpp02.moon",
        "body": "moon",
        "about": "Where the Moon is from Earth's centre: ELP/MPP02 fitted to DE405, cut to %d of its %d terms." % (kept_n, all_n),
        "centre": "earth",
        "gives": "body",
        "frame": "j2000_ecliptic",
        "form": "lbr",
        "wave": "sin",
        "angle_unit_rad": math.pi / 180.0 / 3600.0,
        "unit_m": 1000.0,
        "mean_longitude_rad": d["W1"],
        "to_j2000": {
            "method": "laskar_pq",
            "P": [0.0, 0.10180391e-4, 0.47020439e-6, -0.5417367e-9, -0.2507948e-11, 0.463486e-14],
            "Q": [0.0, -0.113469002e-3, 0.12372674e-6, 0.1265417e-8, -0.1371808e-11, -0.320334e-14],
            "note": "Laskar's precession of the ecliptic, as ELP/MPP02 gives it (Chapront and Francou 2003, section 6), from the mean ecliptic of date to the J2000 ecliptic",
        },
        "time": {"epoch_jd": J2000_JD, "unit_days": 36525.0, "scale": "tdb"},
        "span": {"from_jd": J2000_JD - ELP_SPAN_Y * YEAR_D, "to_jd": J2000_JD + ELP_SPAN_Y * YEAR_D},
        "accuracy": "Near the present, a few metres to tens of metres in distance and a few milliarcseconds in direction for the full series (unverified, from memory: Chapront and Francou 2003). The cut: " + "; ".join(notes) + ".",
        "cut": {"min_size": ELP_CUT, "unit": "arcsec (L, B), km (R)", "at_t": reach, "kept": kept_n, "of": all_n},
        "src": CHAPRONT["id"],
        "sources": [CHAPRONT],
        "input": {"form": "astronomia 4.3.0 data/elpMppDeFull.js (MIT): ELP/MPP02 DE405 with combined arguments", "file": os.path.basename(path), "sha256": sha(path)},
        "licence": "ELP/MPP02 is distributed freely by IMCCE (Paris Observatory) for use with credit to the authors; terms to be read at source by RW1.",
        "vars": vars_,
    }


def num(x):
    """A number as written: the shortest form that reads back to the same double, else 13 significant figures
    (the combined ELP arguments are worked-out doubles; 13 figures keep them to 1e-13, far below the cut)."""
    r = repr(float(x))
    if len(r) <= 16:
        return r
    g = "%.13g" % x
    return g if ("." in g or "e" in g) else g + ".0"


def write(t):
    p = os.path.join(OUT, t["id"] + ".json")
    with open(p, "w", newline="\n") as f:
        # One term per line: readable in a diff, and the file stays plain JSON.
        head = {k: v for k, v in t.items() if k != "vars"}
        f.write(json.dumps(head, indent=1, ensure_ascii=False)[:-2])
        f.write(',\n "vars": [\n')
        for i, powers in enumerate(t["vars"]):
            f.write("  [\n")
            for j, terms in enumerate(powers):
                f.write("   [\n")
                f.write(",\n".join("    [" + ", ".join(num(x) for x in term) + "]" for term in terms))
                f.write("\n   ]" + ("," if j + 1 < len(powers) else "") + "\n")
            f.write("  ]" + ("," if i + 1 < len(t["vars"]) else "") + "\n")
        f.write(" ]\n}\n")
    return p


def check():
    bad = 0
    names = sorted(x for x in os.listdir(OUT) if x.endswith(".json"))
    if not names:
        print("no tables in", OUT)
        return 1
    for n in names:
        t = json.load(open(os.path.join(OUT, n)))
        if t.get("format") == "sse.earth-orientation/1":
            continue
        width = 3 if t["form"] == "xyz" else 6
        ok = t["format"] == "sse.series/1" and len(t["vars"]) == 3 and t["id"] + ".json" == n
        for powers in t["vars"]:
            for terms in powers:
                ok = ok and all(len(x) == width and all(isinstance(y, float) for y in x) for x in terms)
        if not ok:
            print("malformed:", n)
            bad += 1
    return 1 if bad else 0


def main(argv):
    if "--check" in argv:
        sys.exit(check())
    args = dict(zip(argv[::2], argv[1::2]))
    if "--vsop87a" not in args or "--elp" not in args:
        raise SystemExit(__doc__)
    os.makedirs(OUT, exist_ok=True)
    for row in PLANETS:
        print(write(planet(*row, args["--vsop87a"])))
    print(write(moon(args["--elp"])))


if __name__ == "__main__":
    main(sys.argv[1:])
