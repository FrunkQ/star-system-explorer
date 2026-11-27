#!/usr/bin/env python3
"""Check the measured-motion tables and the eclipse figures at source (RW17). Run where JPL and NASA can be reached
(the cloud build machine is refused by both): on AC-ProArt, from the repo root,

  python3 -I scripts/check-sol-motion.py            check, print a report, write handover/packages/RW17-check.json

1. JPL Horizons (DE440/441): the Moon from Earth's centre and every planet (the Earth-Moon barycentre for Earth)
   from the Sun's centre, on the J2000 ecliptic, at dates from 2000 BC to AD 2900. Each table in packs/base-sol/motion
   is evaluated here in plain Python and compared. Expect arcseconds or better inside the spans; DE440 and the series
   differ by their fits, so this measures the series, it does not test the code.
2. NASA's eclipse pages (Espenak): the greatest-eclipse time (TD), gamma and place for the eclipses the kernel tests
   name (crates/sse-kernel/src/eclipse_tests.rs), printed beside the figures typed there from memory.

Credit: "JPL Horizons" (https://ssd.jpl.nasa.gov/horizons/); NASA eclipse predictions by Fred Espenak, NASA GSFC.
Python standard library only.
"""
import json
import math
import os
import re
import sys
import urllib.parse
import urllib.request

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
MOTION = os.path.join(ROOT, "packs", "base-sol", "motion")
API = "https://ssd.jpl.nasa.gov/api/horizons.api"
AU_KM = 149597870.7
# Horizons ids: (table, target, centre)
BODIES = [("vsop87a.mer", "199", "10"), ("vsop87a.ven", "299", "10"), ("vsop87a.emb", "3", "10"),
          ("vsop87a.mar", "4", "10"), ("vsop87a.jup", "5", "10"), ("vsop87a.sat", "6", "10"),
          ("vsop87a.ura", "7", "10"), ("vsop87a.nep", "8", "10"), ("elpmpp02.moon", "301", "399")]
DATES_JD = [2451545.0, 2457987.25, 2460409.26, 2461265.24, 2378495.0, 2268920.0, 2086302.5, 1442902.8, 1086302.5,
            2817045.0, 2780520.0]
# The eclipses the kernel tests name, and the figures it holds for each (TD, gamma, lat, lon).
ECLIPSES = [
    ("2017 Aug 21", "https://eclipse.gsfc.nasa.gov/SEgoogle/SEgoogle2001/SE2017Aug21Tgoogle.html", ("18:26:40", 0.4367, 36.97, -87.67)),
    ("2024 Apr 8", "https://eclipse.gsfc.nasa.gov/SEgoogle/SEgoogle2001/SE2024Apr08Tgoogle.html", ("18:18:29", 0.3431, 25.29, -104.13)),
    ("2026 Aug 12", "https://eclipse.gsfc.nasa.gov/SEgoogle/SEgoogle2001/SE2026Aug12Tgoogle.html", ("17:47:06", 0.8977, 65.17, -25.23)),
    ("2022 Nov 8 lunar", "https://eclipse.gsfc.nasa.gov/LEplot/LEplot2001/LE2022Nov08T.pdf", ("11:00:22", 0.2570, None, None)),
    ("763 BC Jun 15", "https://eclipse.gsfc.nasa.gov/SEcat5/SE-0799--0700.html", (None, None, None, None)),
]


def horner(c, t):
    s = 0.0
    for x in reversed(c):
        s = s * t + x
    return s


def evaluate(tab, jd):
    t = (jd - tab["time"]["epoch_jd"]) / tab["time"]["unit_days"]
    wave = math.cos if tab["wave"] == "cos" else math.sin

    def var(i):
        tot, tk = 0.0, 1.0
        for terms in tab["vars"][i]:
            tot += tk * sum(r[0] * wave(horner(r[1:], t)) for r in terms)
            tk *= t
        return tot

    if tab["form"] == "xyz":
        return [var(i) * tab["unit_m"] / 1000.0 for i in range(3)]
    a = tab["angle_unit_rad"]
    lon = horner(tab["mean_longitude_rad"], t) + var(0) * a
    lat = var(1) * a
    r = var(2) * tab["unit_m"] / 1000.0
    x, y, z = r * math.cos(lon) * math.cos(lat), r * math.sin(lon) * math.cos(lat), r * math.sin(lat)
    p, q = horner(tab["to_j2000"]["P"], t), horner(tab["to_j2000"]["Q"], t)
    sq = math.sqrt(1 - p * p - q * q)
    return [(1 - 2 * p * p) * x + 2 * p * q * y + 2 * p * sq * z,
            2 * p * q * x + (1 - 2 * q * q) * y - 2 * q * sq * z,
            -2 * p * sq * x + 2 * q * sq * y + (1 - 2 * p * p - 2 * q * q) * z]


def horizons(target, centre, jd):
    p = {"COMMAND": target, "EPHEM_TYPE": "VECTORS", "CENTER": "500@" + centre, "TLIST": str(jd), "TLIST_TYPE": "JD",
         "TIME_TYPE": "TT", "OUT_UNITS": "KM-S", "REF_PLANE": "ECLIPTIC", "REF_SYSTEM": "ICRF", "CSV_FORMAT": "YES",
         "VEC_TABLE": "1", "VEC_CORR": "NONE"}
    q = {k: "'%s'" % v for k, v in p.items()}
    q["format"] = "text"
    text = urllib.request.urlopen(API + "?" + urllib.parse.urlencode(q), timeout=120).read().decode()
    m = re.search(r"\$\$SOE(.*?)\$\$EOE", text, re.S)
    if not m:
        return None
    f = [x.strip() for x in m.group(1).strip().split("\n")[0].split(",")]
    return [float(f[2]), float(f[3]), float(f[4])]


def main():
    out = {"horizons": [], "eclipses": []}
    print("1. The tables against JPL Horizons (arcseconds as seen from the centre; km of distance)")
    for tid, target, centre in BODIES:
        tab = json.load(open(os.path.join(MOTION, tid + ".json")))
        for jd in DATES_JD:
            inside = tab["span"]["from_jd"] <= jd <= tab["span"]["to_jd"]
            try:
                h = horizons(target, centre, jd)
            except Exception as e:  # a date Horizons does not cover, or the network
                h = None
                print("  %s JD %.2f: Horizons refused (%s)" % (tid, jd, e))
            if h is None:
                continue
            s = evaluate(tab, jd)
            rh, rs = math.sqrt(sum(x * x for x in h)), math.sqrt(sum(x * x for x in s))
            cosang = max(-1.0, min(1.0, sum(a * b for a, b in zip(h, s)) / rh / rs))
            arcsec = math.degrees(math.acos(cosang)) * 3600
            row = {"table": tid, "jd": jd, "inside_span": inside, "angle_arcsec": arcsec, "distance_km": rs - rh}
            out["horizons"].append(row)
            print("  %-14s JD %10.2f %s  %8.3f\"  %+10.3f km" % (tid, jd, "in " if inside else "OUT", arcsec, rs - rh))
    print("\n2. NASA's eclipse pages (read them beside the figures the tests hold)")
    for name, url, held in ECLIPSES:
        try:
            page = urllib.request.urlopen(url, timeout=60).read().decode("latin-1", "replace")
            text = re.sub(r"<[^>]+>", " ", page)
            bits = re.findall(r"(Greatest Eclipse[^\n]{0,160}|Gamma[^\n]{0,40}|Lat\.?[^\n]{0,60}Long[^\n]{0,60})", text)
            got = " | ".join(" ".join(b.split()) for b in bits[:6])
        except Exception as e:
            got = "could not read (%s)" % e
        out["eclipses"].append({"eclipse": name, "url": url, "held": held, "read": got})
        print("  %s: tests hold %s\n    page: %s" % (name, held, got))
    dest = os.path.join(ROOT, "handover", "packages", "RW17-check.json")
    with open(dest, "w", newline="\n") as f:
        json.dump(out, f, indent=1)
    print("\nwrote", os.path.relpath(dest, ROOT))


if __name__ == "__main__":
    main()
