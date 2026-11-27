#!/usr/bin/env python3
"""RW7: every known Solar System small body and moon as sse.catalogue/1 files.

The cloud build machines cannot reach JPL, so this runs on a machine that can
(Alex's PC, C:\\Development\\sse4). It has two halves, so the slow download is
done once and the build can be re-run from what was saved:

  python scripts/build-sol-catalogues.py fetch CACHE
      Download from JPL into the folder CACHE (outside the repo): the
      small-body database (asteroids and comets, every row, paged), and the
      planetary satellite mean elements and physical parameters pages. Saves
      the raw answers as they came, plus CACHE/retrieved.json (date and URLs).

  python scripts/build-sol-catalogues.py build CACHE OUT [--sse PATH] [--min-size KM]
      Read CACHE only (no network, no clock: the date is the one saved at
      fetch) and write into OUT:
        sol-small-bodies.catalogue.json   text form, biggest first
        sol-small-bodies.ssecat           packed form (sse catalogue pack)
        sol-moons-<plane>.catalogue.json  one per reference plane JPL uses
        sol-moons-<plane>.ssecat
        report.md                         counts, what was left out and why
      --sse names the sse binary (default: cargo run -p sse-cli --release).
      --min-size drops rows below that diameter, for a quick trial run.

  python scripts/build-sol-catalogues.py all CACHE OUT   both halves

Rules this keeps (design SSE4-real-catalogues-design.md 6.1, catalogue format
SSE4-catalogue-format.md, RW1 terms docs/citations/real-catalogues.md):
- No value is typed in: every number comes from the saved JPL answers.
- Measured diameters are graded measured. A body with no measured diameter
  gets one estimated from its absolute magnitude and an albedo (its own when
  measured, else the median measured albedo of its JPL orbit class in this
  same download), graded estimated (design question 7). This estimate is
  fudge RW7-size-from-h in docs/fudges.md, a candidate to move into the RW3b
  flows.
- A row the format cannot place or size is left out and counted in report.md,
  never filled in.
Python standard library only. Deterministic for a given CACHE.
"""
import html.parser
import json
import os
import subprocess
import sys
import time
import urllib.parse
import urllib.request

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SHEET = os.path.join(ROOT, "packs", "base-reference", "pack.json")
NOTABLE = os.path.join(ROOT, "scripts", "sol-notables.json")
BASE_SOL = os.path.join(ROOT, "packs", "base-sol", "pack.json")
MEASURED = os.path.join(ROOT, "scripts", "sol-measured-sizes.json")

SBDB = "https://ssd-api.jpl.nasa.gov/sbdb_query.api"
SATS_ELEM = "https://ssd.jpl.nasa.gov/sats/elem/"
SATS_PHYS = "https://ssd.jpl.nasa.gov/sats/phys_par/"
PAGE = 50000
AGENT = "sse4-catalogue-build/1"

# The SBDB query fields kept (names exactly as the API's field table).
FIELDS = ["spkid", "pdes", "name", "full_name", "prefix", "kind", "class", "neo", "pha",
          "epoch", "equinox", "a", "q", "e", "i", "om", "w", "ma", "tp",
          "H", "G", "M1", "M2", "diameter", "albedo", "GM", "density", "rot_per",
          "spec_B", "spec_T", "first_obs"]

# Stops (design 6.1's table: 100, 30, 10, 3 and 1 km). RW2 measures the
# Expected stop and the device ceilings; it replaces or adds to these.
STEPS_KM = [(100.0, "100 km"), (30.0, "30 km"), (10.0, "10 km"), (3.0, "3 km"), (1.0, "1 km")]
# The diameter of an albedo-1 body of absolute magnitude 0, in D = k / sqrt(p)
# x 10^(-H/5) (Pravec and Harris 2007, Icarus 190, 250, appendix; recalled,
# unverified until the paper search is back on 1 Dec 2026). The same relation
# RW1 used for docs/citations/real-catalogues.md section 4. Fudge
# RW7-size-from-h; it belongs in the RW3b size flow.
H_TO_D_KM = 1329.0
# A class needs this many measured albedos before its median is used.
MIN_ALBEDOS = 30
# Below that, a class's own median is still used if it has this many: a
# class's albedo differs more from another class's than a small sample's
# median does from the class's true one (the distant TNOs are far brighter
# than the main belt's dark bodies). Fewer than this, the kind's median.
MIN_FEW_ALBEDOS = 5

MONTHS = ["January", "February", "March", "April", "May", "June", "July", "August",
          "September", "October", "November", "December"]


def sheet_value(cid):
    for c in json.load(open(SHEET, encoding="utf-8"))["provides"]["reference"]["constants"]:
        if c["id"] == cid:
            return c["value"]["v"]
    raise SystemExit("base-reference has no constant %r" % cid)


# --- fetch ------------------------------------------------------------------

def get(url):
    req = urllib.request.Request(url, headers={"User-Agent": AGENT})
    for attempt in range(5):
        try:
            with urllib.request.urlopen(req, timeout=300) as r:
                return r.read()
        except Exception as e:  # network trouble: back off and retry
            if attempt == 4:
                raise SystemExit("%s: %s" % (url, e))
            time.sleep(5 * 2 ** attempt)


def fetch_sbdb(cache, kind):
    query = {"fields": ",".join(FIELDS), "sb-kind": kind, "sort": "spkid", "full-prec": "1"}
    count = json.loads(get(SBDB + "?" + urllib.parse.urlencode({"sb-kind": kind})))
    pages, start = [], 0
    while True:
        q = dict(query, limit=str(PAGE), **{"limit-from": str(start)})
        body = get(SBDB + "?" + urllib.parse.urlencode(q))
        doc = json.loads(body)
        if "fields" not in doc or "data" not in doc:
            raise SystemExit("SBDB answer has no fields/data: keys %s" % sorted(doc))
        if doc["fields"] != FIELDS:
            raise SystemExit("SBDB fields came back as %s" % doc["fields"])
        name = "sbdb-%s-%07d.json" % (kind, start)
        with open(os.path.join(cache, name), "wb") as f:
            f.write(body)
        pages.append(name)
        print("  %s: %d rows" % (name, len(doc["data"])), flush=True)
        if len(doc["data"]) < PAGE:
            break
        start += PAGE
        time.sleep(1.0)
    return {"count_answer": count, "pages": pages}


def fetch(cache):
    os.makedirs(cache, exist_ok=True)
    got = {"retrieved_utc": time.strftime("%Y-%m-%d", time.gmtime()), "sbdb": {}}
    for kind in ("a", "c"):
        print("SBDB kind %s" % kind, flush=True)
        got["sbdb"][kind] = fetch_sbdb(cache, kind)
    for name, url in (("sats-elem.html", SATS_ELEM), ("sats-phys.html", SATS_PHYS)):
        print(url, flush=True)
        with open(os.path.join(cache, name), "wb") as f:
            f.write(get(url))
    got["urls"] = {"sbdb": SBDB, "sats_elem": SATS_ELEM, "sats_phys": SATS_PHYS}
    with open(os.path.join(cache, "retrieved.json"), "w", encoding="utf-8") as f:
        json.dump(got, f, indent=1)
    print("saved in %s" % cache)


# --- shared -----------------------------------------------------------------

def credit(retrieved, title):
    """JPL's citation form, word for word from ssd.jpl.nasa.gov/about (RW1, P1)."""
    y, m, d = (int(x) for x in retrieved.split("-"))
    return ("NASA/JPL Solar System Dynamics. Data retrieved on %d, %s, %d. %s. https://ssd.jpl.nasa.gov"
            % (y, MONTHS[m - 1], d, title))


def num(s):
    if s is None:
        return None
    s = str(s).strip()
    if s in ("", "n/a", "-"):
        return None
    try:
        v = float(s)
    except ValueError:
        return None
    return v if v == v else None


def median(xs):
    xs = sorted(xs)
    n = len(xs)
    if n == 0:
        return None
    return xs[n // 2] if n % 2 else 0.5 * (xs[n // 2 - 1] + xs[n // 2])


def col(name, typ, role="value", **kw):
    c = {"name": name, "type": typ, "role": role}
    c.update(kw)
    return c


def jd_of_calendar(s):
    """'2000-01-01.5' (TDB) to a Julian date (Meeus, Astronomical Algorithms, ch. 7)."""
    y, m, rest = s.split("-")
    y, m, day = int(y), int(m), float(rest)
    if m <= 2:
        y, m = y - 1, m + 12
    a = y // 100
    b = 2 - a + a // 4
    return int(365.25 * (y + 4716)) + int(30.6001 * (m + 1)) + day + b - 1524.5


def notables():
    """Bodies flagged notable (shown whatever the Detail cut, design 7.3)."""
    if not os.path.exists(NOTABLE):
        return {}
    return {n["pdes"]: (n["name"], n["why"]) for n in json.load(open(NOTABLE, encoding="utf-8"))["small_bodies"]}


def norm(s):
    return "".join(ch for ch in (s or "").lower() if ch.isalnum())


def base_sol():
    """base-sol's bodies by plain name: a catalogue row for one of them is that
    body, not another, so the join (RW4) must not count it twice."""
    sol = json.load(open(BASE_SOL, encoding="utf-8"))["provides"]["other"]["sol"]["bodies"]
    return {norm(b["name"]): b for b in sol}


# --- small bodies -----------------------------------------------------------

def size_from_h(h, albedo, k_km):
    # D = k / sqrt(p) x 10^(-H/5): fudge RW7-size-from-h.
    return k_km / albedo ** 0.5 * 10.0 ** (-h / 5.0)


def build_small_bodies(cache, got, out, min_size):
    recs = []
    for kind in ("a", "c"):
        for name in got["sbdb"][kind]["pages"]:
            doc = json.load(open(os.path.join(cache, name), encoding="utf-8"))
            for row in doc["data"]:
                r = dict(zip(doc["fields"], row))
                # JPL sends spkid as a JSON number and everything else as text.
                r["spkid"] = None if r["spkid"] is None else str(r["spkid"])
                recs.append(r)
    k_km = H_TO_D_KM
    # Medians of measured albedo per JPL orbit class, and per kind, from this download.
    # Measured sizes JPL lacks (the large distant bodies), each quoted from its
    # paper: they take part as measured, and their albedos join their class's.
    measured = json.load(open(MEASURED, encoding="utf-8"))
    papers = {x["id"]: x for x in measured["sources"]}
    supplement = {x["pdes"]: x for x in measured["small_bodies"]}
    for r in recs:
        x = supplement.get(r["pdes"])
        if x is None or num(r["diameter"]) is not None:
            continue
        # An unnamed body is checked by its designation in JPL's full name.
        if norm(x["name"]) != norm(r["name"]) and norm(x["name"]) not in norm(r["full_name"]):
            raise SystemExit("sol-measured-sizes: %s is %r at JPL, %r in the file" % (r["pdes"], r["name"], x["name"]))
        r["diameter"], r["_dsrc"] = x["diameter_km"], x["src"]
        if num(r["albedo"]) is None:
            r["albedo"], r["_psrc"] = x["albedo"], x["src"]
    by_class, by_kind = {}, {}
    for r in recs:
        p = num(r["albedo"])
        if p and p > 0:
            by_class.setdefault(r["class"], []).append(p)
            by_kind.setdefault(r["kind"][0], []).append(p)
    class_p = {c: median(v) for c, v in sorted(by_class.items()) if len(v) >= MIN_FEW_ALBEDOS}
    class_n = {c: len(v) for c, v in sorted(by_class.items())}
    kind_p = {k: median(v) for k, v in sorted(by_kind.items())}
    flagged, known = notables(), base_sol()
    rows, left_out, ids, mismatched = [], {}, set(), []
    grades = {"measured": 0, "estimated": 0}
    for r in recs:
        why = None
        if (r.get("equinox") or "J2000") != "J2000":
            why = "elements not on the J2000 ecliptic"
        d, dg, p, pg, hsrc = num(r["diameter"]), "measured", num(r["albedo"]), "measured", "H"
        if d is None:
            h = num(r["H"])
            if h is None and r["kind"].startswith("c"):
                h, hsrc = num(r["M2"]), "M2"
            if p is None or p <= 0:
                p, pg = class_p.get(r["class"], kind_p.get(r["kind"][0])), "estimated"
            if h is None:
                why = why or "no measured diameter and no absolute magnitude"
            elif p is None:
                why = why or "no albedo to estimate a size with"
            else:
                d, dg = size_from_h(h, p, k_km), "estimated"
        elif p is not None and p <= 0:
            p = None
        q, tp = num(r["q"]), num(r["tp"])
        if why is None and (q is None or tp is None or num(r["epoch"]) is None):
            why = "no perihelion distance, perihelion time or epoch"
        if why is None and min_size and d < min_size:
            why = "below --min-size"
        if why:
            left_out[why] = left_out.get(why, 0) + 1
            continue
        rid = r["pdes"] if r["pdes"] not in ids else "spk" + r["spkid"]
        ids.add(rid)
        grades[dg] += 1
        row = {
            "id": rid, "spkid": r["spkid"], "name": (r["name"] or r["full_name"] or "").strip() or None,
            "orbit_class": r["class"], "diameter_km": d, "diameter_grade": dg,
            "diameter_source": r.get("_dsrc", "jpl-sbdb") if dg == "measured" else "rw7-build",
            "albedo": p, "albedo_grade": pg if p is not None else None,
            "albedo_source": (r.get("_psrc", "jpl-sbdb") if pg == "measured" else "rw7-build") if p is not None else None,
            "h_mag": num(r["H"]), "m1_mag": num(r["M1"]), "m2_mag": num(r["M2"]), "size_from": hsrc if dg == "estimated" else None,
            "epoch_jd": num(r["epoch"]), "a_au": num(r["a"]), "q_au": q, "e": num(r["e"]), "i_deg": num(r["i"]),
            "node_deg": num(r["om"]), "peri_deg": num(r["w"]), "mean_anomaly_deg": num(r["ma"]), "tp_jd": tp,
            "gm_km3_s2": num(r["GM"]), "density_g_cm3": num(r["density"]), "rot_per_h": num(r["rot_per"]),
            "spec_b": r["spec_B"] or None, "spec_t": r["spec_T"] or None, "first_obs": r["first_obs"] or None,
            "neo": {"Y": 1, "N": 0}.get(r["neo"]), "pha": {"Y": 1, "N": 0}.get(r["pha"]),
            "flags": 0,
        }
        if r["pdes"] in flagged:
            if norm(flagged[r["pdes"]][0]) == norm(row["name"]):
                row["flags"] = 1
            else:
                mismatched.append("%s is %r at JPL, %r on the list" % (r["pdes"], row["name"], flagged[r["pdes"]][0]))
        twin = known.get(norm(row["name"]))
        if twin is not None and twin["kind"] != "moon":
            row["base_sol"] = twin["id"]
            # The same body measured in base-sol (Pluto: NSSDCA) beats a size
            # estimated from brightness; a JPL-measured diameter is kept.
            radius = twin["values"].get("body.radius")
            if dg == "estimated" and radius:
                grades["estimated"] -= 1
                grades["measured"] += 1
                row.update(diameter_km=2 * radius["v"] / 1000.0, diameter_grade="measured",
                           diameter_source="base-sol", size_from=None)
                geo = twin["values"].get("body.albedo_geometric")
                if geo and row.get("albedo_grade") != "measured":
                    row.update(albedo=geo["v"], albedo_grade="measured", albedo_source="base-sol")
        rows.append({k: v for k, v in row.items() if v is not None})
    rows.sort(key=lambda x: (-x["diameter_km"], x["id"]))
    jpl = "jpl-sbdb"
    graded = dict(grade_column="diameter_grade", source_column="diameter_source")
    columns = [
        col("id", "str", "id", doc="JPL primary designation (pdes); spk<id> where two share one"),
        col("spkid", "str", grade="convention", source=jpl, doc="JPL SPK id"),
        col("name", "str", grade="convention", source=jpl),
        col("orbit_class", "str", grade="convention", source=jpl, doc="JPL orbit class code (MBA, TJN, TNO, JFc, ...)"),
        col("diameter_km", "f32", unit="km", **graded),
        col("diameter_grade", "u8", "meta"), col("diameter_source", "u8", "meta"),
        col("albedo", "f32", unit="1", grade_column="albedo_grade", source_column="albedo_source", doc="geometric albedo"),
        col("albedo_grade", "u8", "meta"), col("albedo_source", "u8", "meta"),
        col("h_mag", "f32", unit="mag", grade="measured", source=jpl, doc="absolute magnitude H"),
        col("m1_mag", "f32", unit="mag", grade="measured", source=jpl, doc="comet total magnitude parameter M1"),
        col("m2_mag", "f32", unit="mag", grade="measured", source=jpl, doc="comet nuclear magnitude parameter M2"),
        col("size_from", "str", "meta", doc="for an estimated diameter: H or M2"),
        col("epoch_jd", "f64", unit="d", grade="measured", source=jpl, doc="osculating epoch, JD TDB"),
        col("a_au", "f64", unit="au", grade="measured", source=jpl),
        col("q_au", "f64", unit="au", grade="measured", source=jpl),
        col("e", "f64", unit="1", grade="measured", source=jpl),
        col("i_deg", "f64", unit="deg", grade="measured", source=jpl),
        col("node_deg", "f64", unit="deg", grade="measured", source=jpl),
        col("peri_deg", "f64", unit="deg", grade="measured", source=jpl),
        col("mean_anomaly_deg", "f64", unit="deg", grade="measured", source=jpl),
        col("tp_jd", "f64", unit="d", grade="measured", source=jpl, doc="time of perihelion, JD TDB"),
        col("gm_km3_s2", "f32", unit="km3 s-2", grade="measured", source=jpl),
        col("density_g_cm3", "f32", unit="g cm-3", grade="measured", source=jpl),
        col("rot_per_h", "f32", unit="h", grade="measured", source=jpl, doc="synodic rotation period"),
        col("spec_b", "str", grade="measured", source=jpl, doc="SMASSII spectral type"),
        col("spec_t", "str", grade="measured", source=jpl, doc="Tholen spectral type"),
        col("first_obs", "str", grade="measured", source=jpl, doc="first observation used, YYYY-MM-DD"),
        col("neo", "u8", grade="convention", source=jpl, doc="1 near-Earth object"),
        col("pha", "u8", grade="convention", source=jpl, doc="1 potentially hazardous"),
        col("flags", "u8", "meta"),
        col("base_sol", "str", "meta", doc="the base-sol body this row is (counted once)"),
    ]
    header = {
        "format": "sse.catalogue/1", "id": "sol-small-bodies",
        "title": "The Solar System's small bodies (JPL Small-Body Database)",
        "version": got["retrieved_utc"], "level": "small_body",
        "frame": {"kind": "orbit", "plane": "ecliptic_j2000", "primary": "Sun"},
        "anchor": {"name": "Sun"},
        "epoch": {"kind": "per_row", "column": "epoch_jd", "scale": "tdb"},
        "size": {"column": "diameter_km", "quantity": "diameter", "unit": "km"},
        "columns": columns,
        "sources": [
            {"id": jpl, "name": "JPL Small-Body Database, Query API", "version": "1.0", "kind": "downloaded",
             "licence": "NASA/JPL data; only a citation is asked",
             "credit": credit(got["retrieved_utc"], "Small-Body Database Query"),
             "link": "https://ssd.jpl.nasa.gov/tools/sbdb_query.html", "retrieved": got["retrieved_utc"]},
            {"id": "base-sol", "name": "SSE4 base-sol pack, from the NASA NSSDCA Planetary Fact Sheets", "version": "1",
             "kind": "downloaded", "licence": "US government work; no licence stated",
             "credit": "NASA Space Science Data Coordinated Archive, Planetary Fact Sheets, D. R. Williams",
             "link": "https://nssdc.gsfc.nasa.gov/planetary/factsheet/"},
            *[{"id": k, "name": v["citation"], "version": "published", "kind": "authored",
               "licence": "a published measurement, quoted and cited",
               "credit": v["citation"] + (" (doi:%s)" % v["doi"] if "doi" in v else " (arXiv:%s)" % v.get("arxiv", ""))}
              for k, v in sorted(papers.items())],
            {"id": "rw7-build", "name": "SSE4 catalogue build (scripts/build-sol-catalogues.py)", "version": "1",
             "kind": "authored", "licence": "Apache-2.0",
             "credit": "Sizes and albedos marked estimated were worked out by SSE4 from JPL's absolute magnitudes: "
                       "D = %g km / sqrt(albedo) x 10^(-H/5), with the body's own albedo where measured, else the "
                       "median measured albedo of its JPL orbit class in the same download." % k_km},
        ],
        "completeness": {
            "size_edges": [1000.0], "fraction": [[1.0]], "grade": "convention", "source": "rw7-build",
            "note": "a stated convention, not a measurement: bodies over 1,000 km are taken as all known. "
                    "The completeness flow (RW3b) replaces it",
        },
        "steps": [{"size": s, "label": lbl} for s, lbl in STEPS_KM if rows and rows[-1]["diameter_km"] <= s],
        "notes": [
            "Orbits are osculating heliocentric elements on the J2000 ecliptic at each row's epoch, as JPL gives them.",
            "Estimated diameters: %d of %d rows (graded estimated, size_from says H or M2)." % (grades["estimated"], len(rows)),
            "Class median albedos used: " + ", ".join("%s %.3f" % (c, v) for c, v in sorted(class_p.items())),
            "Rows left out: " + (", ".join("%s %d" % kv for kv in sorted(left_out.items())) or "none"),
            "Notable flags from scripts/sol-notables.json (authored).",
        ],
    }
    return write(out, "sol-small-bodies", header, rows), {
        "rows": len(rows), "grades": grades, "left_out": left_out, "class_albedo": class_p,
        "class_n": class_n, "steps": {lbl: sum(1 for r in rows if r["diameter_km"] >= s) for s, lbl in STEPS_KM},
        "classes": count_by(rows, "orbit_class"), "notables": sum(1 for r in rows if r["flags"] & 1),
        "notables_missing": sorted(set(flagged) - {r["id"] for r in rows if r["flags"] & 1}),
        "notables_mismatched": mismatched,
        "base_sol": sorted("%s = %s" % (r["id"], r["base_sol"]) for r in rows if "base_sol" in r),
    }


def count_by(rows, key):
    out = {}
    for r in rows:
        out[r.get(key)] = out.get(r.get(key), 0) + 1
    return dict(sorted(out.items(), key=lambda kv: str(kv[0])))


# --- moons ------------------------------------------------------------------

class Tables(html.parser.HTMLParser):
    """Every <table> on a page as rows of cell text. JPL's pages leave some
    <td> cells unclosed, so a new cell or row closes the open one; a heading
    with colspan counts once for each column it spans, so the headings line up
    with the data cells under them."""

    def __init__(self):
        super().__init__()
        self.tables, self.row, self.cell, self.span, self.depth = [], None, None, 1, 0

    def end_cell(self):
        if self.cell is not None and self.row is not None:
            text = " ".join("".join(self.cell).split())
            self.row.extend([text] * self.span)
        self.cell, self.span = None, 1

    def end_row(self):
        self.end_cell()
        if self.row:
            self.tables[-1].append(self.row)
        self.row = None

    def handle_starttag(self, tag, attrs):
        if tag == "table":
            self.tables.append([])
            self.depth += 1
        elif tag == "tr" and self.depth:
            if self.row is not None:
                self.end_row()
            self.row = []
        elif tag in ("td", "th") and self.row is not None:
            self.end_cell()
            self.cell = []
            span = dict(attrs).get("colspan")
            self.span = int(span) if span and span.isdigit() and tag == "th" else 1

    def handle_endtag(self, tag):
        if tag in ("td", "th"):
            self.end_cell()
        elif tag == "tr" and self.row is not None:
            self.end_row()
        elif tag == "table" and self.depth:
            if self.row is not None:
                self.end_row()
            self.depth -= 1

    def handle_data(self, data):
        if self.cell is not None:
            self.cell.append(data)


def table_with(path, *heads):
    p = Tables()
    p.feed(open(path, encoding="utf-8", errors="replace").read())
    for t in p.tables:
        for i, r in enumerate(t):
            low = [c.lower() for c in r]
            if all(any(h in c for c in low) for h in heads):
                return r, t[i + 1:]
    raise SystemExit("%s: no table with columns %s (the page layout changed; look at the saved file)" % (path, heads))


def find(header, *words):
    for i, c in enumerate(header):
        if all(w in c.lower() for w in words):
            return i
    raise SystemExit("column %s not found in %s" % (words, header))


PLANES = {"ecliptic": "ecliptic_j2000", "equatorial": "primary_equator", "laplace": "laplace"}


def build_moons(cache, got, out, min_size, au_km):
    ph, prows = table_with(os.path.join(cache, "sats-phys.html"), "satellite", "radius")
    i_code, i_r, i_gm, i_rho = find(ph, "code"), find(ph, "radius"), find(ph, "gm"), find(ph, "density")
    phys = {}
    for r in prows:
        if len(r) > i_code and r[i_code].strip().isdigit():
            gm = num(r[i_gm])
            # A printed GM of 0 (Nereid's 0.00000 +/- 0.00000) is no measurement:
            # every moon has a mass.
            phys[r[i_code].strip()] = (num(r[i_r]), gm if gm else None, num(r[i_rho]))
    eh, erows = table_with(os.path.join(cache, "sats-elem.html"), "satellite", "frame", "epoch")
    ix = moon_columns(eh)
    known = base_sol()
    files, left_out, by_plane, seen = {}, {}, {}, set()
    for r in erows:
        if len(r) < len(eh):
            continue
        code = r[ix["code"]].strip()
        if code in seen:
            left_out.setdefault("a second row for the same NAIF code (the first is kept)", []).append(r[ix["sat"]])
            continue
        seen.add(code)
        rad = phys.get(code, (None, None, None))[0]
        why = None
        if rad is None:
            why = "no mean radius on JPL's physical parameters page"
        plane = PLANES.get(r[ix["frame"]].strip().lower())
        if plane is None:
            why = why or "frame %r not known" % r[ix["frame"]]
        a_km = num(r[ix["a"]])
        if a_km is None:
            why = why or "no semi-major axis"
        if why is None and min_size and 2 * rad < min_size:
            why = "below --min-size"
        if why:
            left_out.setdefault(why, []).append(r[ix["sat"]])
            continue
        _, gm, rho = phys[code]
        name = r[ix["sat"]].strip()
        row = {
            "id": name.lower().replace(" ", "-").replace("/", "-"), "name": name, "primary": r[ix["planet"]].strip(),
            "naif": code, "diameter_km": 2 * rad, "gm_km3_s2": gm, "density_g_cm3": rho,
            "epoch_jd": jd_of_calendar(r[ix["epoch"]].strip()), "a_au": a_km / au_km, "a_km": a_km,
            "e": num(r[ix["e"]]), "i_deg": num(r[ix["i"]]), "node_deg": num(r[ix["node"]]),
            "peri_deg": num(r[ix["w"]]), "mean_anomaly_deg": num(r[ix["m"]]),
            "pole_ra_deg": num(r[ix["ra"]]), "pole_dec_deg": num(r[ix["dec"]]), "tilt_deg": num(r[ix["tilt"]]),
        }
        twin = known.get(norm(name))
        if twin is not None and twin["kind"] == "moon":
            row["base_sol"] = twin["id"]
            # base-sol's moons are the ones the app already shows, so they pass
            # the cut whatever their size (design 7.2 names Phobos and Deimos).
            row["flags"] = 1
        by_plane.setdefault(plane, []).append({k: v for k, v in row.items() if v is not None})
    jpl = "jpl-sats"
    m = dict(grade="measured", source=jpl)
    for plane, rows in sorted(by_plane.items()):
        rows.sort(key=lambda x: (-x["diameter_km"], x["id"]))
        header = {
            "format": "sse.catalogue/1", "id": "sol-moons-" + plane.replace("_", "-"),
            "title": "The Solar System's moons, elements on the %s (JPL)" % plane.replace("_", " "),
            "version": got["retrieved_utc"], "level": "world",
            "frame": {"kind": "orbit", "plane": plane},
            "anchor": {"name": "Sun"},
            "epoch": {"kind": "per_row", "column": "epoch_jd", "scale": "tdb"},
            "size": {"column": "diameter_km", "quantity": "diameter", "unit": "km"},
            "columns": [
                col("id", "str", "id"), col("name", "str", grade="convention", source=jpl),
                col("primary", "str", grade="convention", source=jpl, doc="the planet it goes round"),
                col("naif", "str", grade="convention", source=jpl, doc="NAIF id"),
                col("diameter_km", "f32", unit="km", doc="twice the mean radius", **m),
                col("gm_km3_s2", "f64", unit="km3 s-2", **m), col("density_g_cm3", "f32", unit="g cm-3", **m),
                col("epoch_jd", "f64", unit="d", **m), col("a_au", "f64", unit="au", **m),
                col("a_km", "f64", unit="km", **m), col("e", "f64", unit="1", **m),
                col("i_deg", "f64", unit="deg", **m), col("node_deg", "f64", unit="deg", **m),
                col("peri_deg", "f64", unit="deg", **m), col("mean_anomaly_deg", "f64", unit="deg", **m),
                col("pole_ra_deg", "f64", unit="deg", doc="the reference plane's pole", **m),
                col("pole_dec_deg", "f64", unit="deg", **m), col("tilt_deg", "f64", unit="deg", **m),
                col("flags", "u8", "meta"),
                col("base_sol", "str", "meta", doc="the base-sol body this row is (counted once)"),
            ],
            "sources": [{"id": jpl, "name": "JPL Planetary Satellite Mean Elements and Physical Parameters",
                         "version": got["retrieved_utc"], "kind": "downloaded",
                         "licence": "NASA/JPL data; only a citation is asked",
                         "credit": credit(got["retrieved_utc"], "Planetary Satellite Mean Elements; Planetary Satellite Physical Parameters"),
                         "link": "https://ssd.jpl.nasa.gov/sats/", "retrieved": got["retrieved_utc"]}],
            "completeness": {"size_edges": [1000.0], "fraction": [[1.0]], "grade": "convention", "source": jpl,
                             "note": "a stated convention, not a measurement: moons over 1,000 km are taken as all known"},
            "notes": ["Mean elements, which JPL says are not for ephemeris work; the measured motion (RW17) is.",
                      "Moons without a mean radius on JPL's page are left out of this file (report.md lists them)."],
        }
        files[plane] = write(out, header["id"], header, rows)
    return files, {"planes": {p: len(r) for p, r in by_plane.items()},
                   "left_out": {k: sorted(v) for k, v in left_out.items()}}


def moon_columns(h):
    """Columns of JPL's mean-elements table by their headings."""
    def at(test, what):
        for i, c in enumerate(h):
            if test(c.lower().strip()):
                return i
        raise SystemExit("sats-elem: no %s column in %s" % (what, h))
    return {
        "planet": at(lambda c: c.startswith("planet"), "Planet"),
        "sat": at(lambda c: c.startswith("satellite"), "Satellite"),
        "code": at(lambda c: c.startswith("code"), "Code"),
        "frame": at(lambda c: c.startswith("frame"), "Frame"),
        "epoch": at(lambda c: c.startswith("epoch"), "Epoch"),
        "a": at(lambda c: c.startswith("a ") or c == "a" or c.startswith("a("), "a (km)"),
        "e": at(lambda c: c == "e", "e"),
        "w": at(lambda c: c.startswith("ω") or c.startswith("w ") or c.startswith("&omega"), "ω (deg)"),
        "m": at(lambda c: c.startswith("m ") or c == "m" or c.startswith("m("), "M (deg)"),
        "i": at(lambda c: c.startswith("i ") or c == "i" or c.startswith("i("), "i (deg)"),
        "node": at(lambda c: c.startswith("node"), "node (deg)"),
        "ra": at(lambda c: c.startswith("r.a"), "R.A. (deg)"),
        "dec": at(lambda c: c.startswith("dec"), "Dec. (deg)"),
        "tilt": at(lambda c: c.startswith("tilt"), "Tilt (deg)"),
    }


# --- writing ----------------------------------------------------------------

SSE = None


def write(out, cid, header, rows):
    text = os.path.join(out, cid + ".catalogue.json")
    with open(text, "w", encoding="utf-8", newline="\n") as f:
        f.write('{"header": ')
        json.dump(header, f, ensure_ascii=False, separators=(",", ":"))
        f.write(',\n"rows": [\n')
        for k, r in enumerate(rows):
            f.write(("," if k else "") + json.dumps(r, ensure_ascii=False, separators=(",", ":")) + "\n")
        f.write("]}\n")
    packed = os.path.join(out, cid + ".ssecat")
    cmd = (SSE or ["cargo", "run", "-q", "-p", "sse-cli", "--release", "--"]) + ["catalogue", "pack", text, packed]
    r = subprocess.run(cmd, cwd=ROOT)
    if r.returncode != 0:
        raise SystemExit("sse catalogue pack refused %s (see above)" % text)
    return {"text": text, "packed": packed, "rows": len(rows)}


def build(cache, out, min_size):
    got = json.load(open(os.path.join(cache, "retrieved.json"), encoding="utf-8"))
    os.makedirs(out, exist_ok=True)
    au_km = sheet_value("au") / 1000.0
    sb_file, sb = build_small_bodies(cache, got, out, min_size)
    moon_files, mo = build_moons(cache, got, out, min_size, au_km)
    lines = ["# Solar System catalogues, built %s from JPL data retrieved %s" % (got["retrieved_utc"], got["retrieved_utc"]), "",
             "## Small bodies", "", "%d rows (%d measured diameters, %d estimated)." % (sb["rows"], sb["grades"]["measured"], sb["grades"]["estimated"]), "",
             "| Down to | Rows |", "|---|---|"] + ["| %s | %d |" % kv for kv in sb["steps"].items()] + [
             "", "By JPL orbit class: " + ", ".join("%s %d" % kv for kv in sb["classes"].items()), "",
             "Left out: " + (", ".join("%s %d" % kv for kv in sorted(sb["left_out"].items())) or "none"), "",
             "Class median albedos (measured count): " + ", ".join("%s %.3f (%d)" % (c, v, sb["class_n"][c]) for c, v in sorted(sb["class_albedo"].items())), "",
             "Classes under %d measured albedos, whose median is used anyway: " % MIN_ALBEDOS
             + (", ".join(c for c in sorted(sb["class_albedo"]) if sb["class_n"][c] < MIN_ALBEDOS) or "none"), "",
             "Notables flagged: %d; on the list but not flagged: %s" % (sb["notables"], ", ".join(sb["notables_missing"]) or "none"), "",
             "Name mismatches (not flagged): " + ("; ".join(sb["notables_mismatched"]) or "none"), "",
             "Rows that are base-sol bodies: " + (", ".join(sb["base_sol"]) or "none"), "",
             "## Moons", "", "Rows by plane: " + ", ".join("%s %d" % kv for kv in sorted(mo["planes"].items())), ""]
    for why, names in sorted(mo["left_out"].items()):
        lines += ["Left out (%s), %d: %s" % (why, len(names), ", ".join(names)), ""]
    lines += ["## Files", ""] + ["- %s: %d rows" % (os.path.basename(f["packed"]), f["rows"]) for f in [sb_file] + list(moon_files.values())]
    with open(os.path.join(out, "report.md"), "w", encoding="utf-8", newline="\n") as f:
        f.write("\n".join(lines) + "\n")
    print("\n".join(lines))


def main(argv):
    global SSE
    if "--sse" in argv:
        i = argv.index("--sse")
        SSE = [argv[i + 1]]
        argv = argv[:i] + argv[i + 2:]
    min_size = None
    if "--min-size" in argv:
        i = argv.index("--min-size")
        min_size = float(argv[i + 1])
        argv = argv[:i] + argv[i + 2:]
    if len(argv) >= 2 and argv[0] == "fetch":
        fetch(argv[1])
    elif len(argv) >= 3 and argv[0] == "build":
        build(argv[1], argv[2], min_size)
    elif len(argv) >= 3 and argv[0] == "all":
        fetch(argv[1])
        build(argv[1], argv[2], min_size)
    else:
        print(__doc__)
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
