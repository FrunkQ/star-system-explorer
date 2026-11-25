"""RS9 part 3: real memorials in space, for the Sol today pack (store-packs/sol-today/memorials.json).

Usage (from the repo root): python3 -I scripts/spaceflight/memorials.py [--fetch]
  --fetch queries Wikidata (CC0) for small bodies named after those in in-memory.json (P138 "named after", with
  their Minor Planet Center number P5736) and JPL Horizons for each one's osculating orbit, and saves both in
  scripts/spaceflight/memorials_sources.json; without it the saved file is used.

Only real memorials (Alex, 25 Nov 2026, track SH: "stay restrained and respectful - one place we don't joke"),
each with two sources. This part holds the named small bodies; named surface features (lunar craters, the hills
on Mars), objects left (the Fallen Astronaut at Hadley) and remains carried (Tombaugh on New Horizons, Shoemaker on
Lunar Prospector) wait for their second sources (handover/packages/RS9.md).
"""
import json, os, re, sys, time, urllib.error, urllib.parse, urllib.request

HERE = os.path.dirname(os.path.abspath(__file__))
PACK = os.path.join(HERE, "..", "..", "store-packs", "sol-today")
SRC_FILE = os.path.join(HERE, "memorials_sources.json")
J2000_S = 63115200000
AU_M = 149597870700.0


def get(url, accept=None):
    req = urllib.request.Request(url, headers={"User-Agent": "sse-build/1.0", **({"Accept": accept} if accept else {})})
    for tries in range(4):
        try:
            return urllib.request.urlopen(req, timeout=120).read().decode()
        except urllib.error.HTTPError as e:
            if e.code != 429 or tries == 3:
                raise
            time.sleep(65)


def fetch():
    wd = json.load(open(os.path.join(HERE, "in_memory_wikidata.json"), encoding="utf-8"))["items"]
    people = sorted(k for k, v in wd.items() if "died" in v)
    q = ("SELECT ?x ?mpc ?who WHERE { VALUES ?who { %s } ?x wdt:P138 ?who . ?x wdt:P31 wd:Q3863 . "
         "?x wdt:P5736 ?mpc . }" % " ".join("wd:" + p for p in people))
    rows = json.loads(get("https://query.wikidata.org/sparql?" + urllib.parse.urlencode({"query": q}),
                          "application/sparql-results+json"))["results"]["bindings"]
    bodies = {}
    for r in rows:
        num = r["mpc"]["value"]
        b = bodies.setdefault(num, {"wikidata": r["x"]["value"].rsplit("/", 1)[-1], "named_after": []})
        who = r["who"]["value"].rsplit("/", 1)[-1]
        if who not in b["named_after"]:
            b["named_after"].append(who)
    for num, b in sorted(bodies.items(), key=lambda kv: int(kv[0])):
        text = get("https://ssd.jpl.nasa.gov/api/horizons.api?" + urllib.parse.urlencode(
            {"format": "text", "COMMAND": "'%s;'" % num, "OBJ_DATA": "'YES'", "MAKE_EPHEM": "'NO'"}))
        m = re.search(r"JPL/HORIZONS\s+(.+?)\s{2,}", text)
        b["horizons_name"] = m.group(1).strip() if m else None
        b["horizons_soln"] = (re.search(r"Soln\.date:\s*(\S+)", text) or [None, None])[1]
        for k in ("EPOCH", "EC", "QR", "OM", "W", "IN", "A", "MA", "H", "G", "RAD", "ALBEDO"):
            m = re.search(r"\b%s=\s*([-\d.E+]+)" % k, text)
            b[k] = float(m.group(1)) if m else None
    # Absolute magnitude H, second source: Wikidata P1457 (it keeps several published values; one must agree).
    qh = "SELECT ?x ?h WHERE { VALUES ?x { %s } ?x wdt:P1457 ?h . }" % " ".join(
        "wd:" + b["wikidata"] for b in bodies.values())
    for r in json.loads(get("https://query.wikidata.org/sparql?" + urllib.parse.urlencode({"query": qh}),
                            "application/sparql-results+json"))["results"]["bindings"]:
        qid = r["x"]["value"].rsplit("/", 1)[-1]
        for b in bodies.values():
            if b["wikidata"] == qid:
                b.setdefault("wikidata_h", []).append(float(r["h"]["value"]))
    for b in bodies.values():
        b["wikidata_h"] = sorted(set(b.get("wikidata_h", [])))
    with open(SRC_FILE, "w", encoding="utf-8") as fh:
        json.dump({"about": "Sources for scripts/spaceflight/memorials.py, queried 25 Nov 2026: Wikidata (CC0) at "
                            "https://query.wikidata.org/sparql, and JPL Horizons small-body records (OBJ_DATA) at "
                            "https://ssd.jpl.nasa.gov/api/horizons.api (credit JPL Horizons)",
                   "query": q, "bodies": bodies}, fh, ensure_ascii=False, indent=1)
        fh.write("\n")


def build():
    src = json.load(open(SRC_FILE, encoding="utf-8"))["bodies"]
    crews = json.load(open(os.path.join(PACK, "crews.json"), encoding="utf-8"))
    by_qid = {p["sources"].get("wikidata"): p for p in crews["people"] if p["sources"].get("wikidata")}
    out = []
    for num, b in sorted(src.items(), key=lambda kv: int(kv[0])):
        who = [by_qid[q] for q in b["named_after"] if q in by_qid]
        if not who or b["A"] is None:
            raise SystemExit("%s: no person or no orbit" % num)
        name = b["horizons_name"].split("(")[0].strip()
        # H only when Wikidata holds a value within 0.5 mag of Horizons' (two sources); Horizons' is the one given.
        h_ok = b["H"] is not None and any(abs(b["H"] - w) <= 0.5 for w in b.get("wikidata_h", []))
        out.append({
            "id": "minor-planet-" + num, "kind": "named_small_body", "name": name,
            "people": [p["id"] for p in who], "names_text": "; ".join(p["name"] for p in who),
            "h_mag": b["H"] if h_ok else None,
            "h_why": ("absolute magnitude H from JPL Horizons (slope G %s); Wikidata P1457 lists %s" % (
                b["G"], ", ".join("%g" % w for w in b.get("wikidata_h", [])))) if h_ok else "no two sources agree",
            "since_s": None, "since_date": None,
            "since_why": "the naming date is in the Minor Planet Center's citation, not yet read here",
            "where": {"mpc": num, "elements": {
                "a_m": b["A"] * AU_M, "e": b["EC"], "i_deg": b["IN"], "node_deg": b["OM"], "peri_deg": b["W"],
                "m0_deg": b["MA"], "epoch_s": int(round(J2000_S + (b["EPOCH"] - 2451545.0) * 86400)),
                "frame": "heliocentric ecliptic J2000"}, "src": "jpl-horizons (solution %s)" % b["horizons_soln"]},
            "sources": [{"text": "JPL Horizons small-body record, minor planet number %s" % num,
                         "url": "https://ssd.jpl.nasa.gov/tools/sbdb_lookup.html#/?sstr=%s" % num},
                        {"text": "Wikidata %s (named after)" % b["wikidata"],
                         "url": "https://www.wikidata.org/wiki/%s" % b["wikidata"]}]})
    with open(os.path.join(PACK, "memorials.json"), "w", encoding="utf-8") as fh:
        json.dump({"about": ("Real memorials in space, each with two sources (scripts/spaceflight/memorials.py). This "
                             "part: the small bodies named for those in in-memory.json, with their orbits."),
                   "memorials": out}, fh, ensure_ascii=False, indent=1)
        fh.write("\n")
    print(len(out), "named small bodies")


if __name__ == "__main__":
    if "--fetch" in sys.argv:
        fetch()
    build()
