"""RS9 part 2: In memory, for the Sol today pack. Called by build_crews.py; writes store-packs/sol-today/in-memory.json
and adds a `death` to each person it names in crews.json (people who died preparing to fly join crews.json with no
legs).

Alex, 25 Nov 2026, on track SH: "stay restrained and respectful - one place we don't joke". So: every loss here has
at least two sources; a sentence is one plain statement of fact taken from those sources, with no adjectives; no
name is guessed, and where the sources give a number of dead but not every name, `unconfirmed` says so in words.

Sources, as fetched for this file (25 Nov 2026):
- `mcdowell`: Jonathan McDowell's human spaceflight lists (astro.tsv, trainee.tsv, segments.tsv), CC BY 4.0.
- `wikidata`: Wikidata items (CC0), facts in scripts/spaceflight/in_memory_wikidata.json (written with --fetch):
  each person's date of death (P570), each event's date (P585) and number of deaths (P1120).
- `wikipedia-list`: Wikipedia, "List of spaceflight-related accidents and incidents",
  https://en.wikipedia.org/wiki/List_of_spaceflight-related_accidents_and_incidents (read 25 Nov 2026; CC BY-SA 4.0;
  only short facts are taken, in our own words).
"""
import json, os, re, sys, urllib.parse, urllib.request

HERE = os.path.dirname(os.path.abspath(__file__))
WD_FILE = os.path.join(HERE, "in_memory_wikidata.json")
WIKI_LIST = {"text": "Wikipedia, List of spaceflight-related accidents and incidents (read 25 Nov 2026)",
             "url": "https://en.wikipedia.org/wiki/List_of_spaceflight-related_accidents_and_incidents"}
MCDOWELL = {"text": "J. C. McDowell, Jonathan's Space Report human spaceflight lists",
            "url": "https://planet4589.org/space/astro/web/index.html"}


def wd(qid):
    return {"text": "Wikidata %s" % qid, "url": "https://www.wikidata.org/wiki/%s" % qid}


# Each loss: who (McDowell short names for those in his lists; Wikidata ids), the event's Wikidata item, where, what
# they were doing, and one sentence of fact from the Wikipedia list's row (verbatim rows read 25 Nov 2026).
# `crew` names McDowell's people by short name (astro.tsv) or by full name in trainee.tsv (`trainee`).
LOSSES = [
    {"id": "bondarenko-1961", "date": "1961-03-23", "during": "training", "event": None,
     "where": "Moscow, in a low-pressure chamber", "trainee": [("Bondarenko,Valentin Vasilevich 'Jr'", "Q538639")],
     "sentence": "A fire started in the oxygen-rich chamber during a fifteen-day endurance test; Valentin Bondarenko "
                 "died in hospital sixteen hours later."},
    {"id": "freeman-1964", "date": "1964-10-31", "during": "training", "event": None,
     "where": "on approach to Ellington Air Force Base, Texas",
     "trainee": [("Freeman, Theodore Cordy", "Q634141")],
     "sentence": "A goose struck his T-38 jet on a landing approach, both engines stopped, and he ejected too close "
                 "to the ground for his parachute to open.",
     "note": "McDowell's trainee list gives 21 October 1964; Wikidata and the Wikipedia list give 31 October."},
    {"id": "t38-1966", "date": "1966-02-28", "during": "training", "event": "Q4572362",
     "where": "Lambert Field, St. Louis", "trainee": [("See, Elliott McKay, Jr", "Q621462"),
                                                       ("Bassett, Charles Arthur,II", "Q704355")],
     "sentence": "Their T-38 jet crashed into the McDonnell Aircraft building while landing in bad weather; they "
                 "were the prime crew of Gemini 9, on their way to simulator training."},
    {"id": "apollo-1-1967", "date": "1967-01-27", "during": "ground_test", "event": "Q194082",
     "where": "Launch Complex 34, Cape Canaveral", "crew": [("Grissom, V.I.", "Q110879"), ("White, E.H.", "Q216726")],
     "trainee": [("Chaffee, Roger Bruce", "Q345678")],
     "sentence": "A fire in the command module's pure oxygen atmosphere during a launch rehearsal test killed the "
                 "crew of Apollo 1."},
    {"id": "soyuz-1-1967", "date": "1967-04-24", "during": "flight", "event": "Q211254", "on": "soyuz-1",
     "where": "near Karabutak, Orenburg", "crew": [("Komarov, V.M.", "Q104865")],
     "sentence": "The parachute did not open properly after reentry and the capsule struck the ground."},
    {"id": "williams-1967", "date": "1967-10-05", "during": "training", "event": None,
     "where": "north of Tallahassee, Florida", "trainee": [("Williams, Clifton Curtis, Jr", "Q709646")],
     "sentence": "The aileron controls of his T-38 jet failed on a solo flight, and he ejected too low."},
    {"id": "x-15-3-65-97", "date": "1967-11-15", "during": "flight", "event": "Q8041409",
     "where": "over the Mojave Desert, California", "crew": [("Adams, M.J.", "Q1122204")],
     "sentence": "The X-15 broke up during its return from a flight above 80 km after electrical and control "
                 "problems."},
    {"id": "lawrence-1967", "date": "1967-12-08", "during": "training", "event": None,
     "where": "Edwards Air Force Base, California", "trainee": [("Lawrence, Robert Henry, Jr", "Q391517")],
     "sentence": "His F-104 crashed during steep-descent landing practice; his parachute did not fully open."},
    {"id": "gagarin-1968", "date": "1968-03-27", "during": "training", "event": "Q8826711",
     "where": "near Novosyolovo, Vladimir Oblast", "crew": [("Gagarin, Yu.A.", "Q7327")],
     "unconfirmed": "and the instructor flying with him", "toll_event": True,
     "sentence": "His two-seat jet crashed on a training flight."},
    {"id": "soyuz-11-1971", "date": "1971-06-30", "during": "flight", "event": "Q648581", "on": "soyuz-11",
     "where": "returning from Salyut 1, above Kazakhstan",
     "crew": [("Dobrovol'skiy, G.T.", "Q355433"), ("Volkov, V.N.", "Q345658"), ("Patsaev, V.I.", "Q358525")],
     "sentence": "A valve opened as the modules separated and the cabin lost its air before reentry."},
    {"id": "challenger-1986", "date": "1986-01-28", "during": "flight", "event": "Q858145",
     "where": "off Cape Canaveral, Florida",
     "crew": [("Scobee, F.R.", "Q320047"), ("Smith, M.J.", "Q25981"), ("Onizuka, E.S.", "Q349365"),
              ("Resnik, J.A.", "Q49284"), ("McNair, R.E.", "Q315730"), ("McAuliffe, S.C.", "Q229674"),
              ("Jarvis, G.B.", "Q382013")],
     "sentence": "A seal in a solid rocket booster failed in the cold, and the Shuttle broke apart 73 seconds after "
                 "launch."},
    {"id": "vozovikov-1993", "date": "1993-07-11", "during": "training", "event": None,
     "where": "the Black Sea, near Anapa", "trainee": [("Vozovikov, Sergey Yurievich", "Q3479773")],
     "sentence": "He drowned during water recovery training."},
    {"id": "columbia-2003", "date": "2003-02-01", "during": "flight", "event": "Q580867", "on": "columbia-sts-107",
     "where": "over Texas and Louisiana, during reentry",
     "crew": [("Husband, R.D.", "Q346671"), ("McCool, W.C.", "Q334463"), ("Anderson, M.P.", "Q318542"),
              ("Chawla, K.", "Q237879"), ("Brown, D.M.", "Q350317"), ("Clark, L.B.", "Q232441"),
              ("Ramon, I.", "Q219569")],
     "sentence": "Foam struck the wing at launch; on reentry hot gas entered the damaged wing and the Shuttle broke "
                 "apart."},
    {"id": "vss-enterprise-2014", "date": "2014-10-31", "during": "flight", "event": "Q18396768",
     "where": "over the Mojave Desert, California", "trainee": [("Alsbury, Michael Tyner", "Q18420502")],
     "sentence": "The feathering system of SpaceShipTwo VSS Enterprise deployed early during a powered test flight; "
                 "the pilot, Peter Siebold, survived with serious injuries.",
     "note": "A test flight in the atmosphere, not above 80 km; the Wikipedia list files it under training and "
             "testing."},
]

# Ground work and tests: those killed preparing rockets, named only where both sources name them. Each needs the
# Wikipedia list's row (read verbatim 25 Nov 2026) and the event's Wikidata item; a disputed toll is [low, high].
# Still waiting for a second source, so left out for now (handover/packages/RS9.md): Xichang 1996 (Long March 3B,
# 6 to 100), the STS-1 nitrogen deaths of 1981, the Scaled Composites test of 2007, the R-9 silo fire of 1963.
GROUND = [
    {"id": "nedelin-1960", "date": "1960-10-24", "during": "ground_work", "event": "Q386074",
     "where": "Baikonur Cosmodrome", "names": [("Mitrofan Nedelin", "Q707715")],
     "toll": [78, 121], "toll_why": "Wikidata gives 78; the Wikipedia list gives Marshal Nedelin and about 120 "
                                   "others",
     "unconfirmed": "and the others killed, whose names these sources do not give",
     "sentence": "An uncrewed R-16 rocket exploded on its launch pad."},
    {"id": "plesetsk-1980", "date": "1980-03-18", "during": "ground_work", "event": "Q16662654",
     "where": "Plesetsk Cosmodrome", "names": [], "toll": 48,
     "unconfirmed": "those killed, whose names these sources do not give",
     "sentence": "A Vostok-2M rocket exploded while it was being fuelled."},
    {"id": "alcantara-2003", "date": "2003-08-22", "during": "ground_work", "event": "Q377174",
     "where": "Alcântara Launch Center, Brazil", "names": [], "toll": 21,
     "unconfirmed": "those killed, whose names these sources do not give",
     "sentence": "An uncrewed VLS-1 rocket exploded during launch preparations."},
]


def fetch():
    """Query Wikidata for every person's date of death and each event's date and number of deaths."""
    qids = sorted({q for l in LOSSES for _, q in l.get("crew", []) + l.get("trainee", [])} |
                  {l["event"] for l in LOSSES if l["event"]} | {g["event"] for g in GROUND} |
                  {q for g in GROUND for _, q in g["names"]})
    q = ("SELECT ?x ?xLabel ?died ?date ?deaths WHERE { VALUES ?x { %s } OPTIONAL { ?x wdt:P570 ?died } "
         "OPTIONAL { ?x wdt:P585 ?date } OPTIONAL { ?x wdt:P1120 ?deaths } "
         "SERVICE wikibase:label { bd:serviceParam wikibase:language \"en\". } }" % " ".join("wd:" + x for x in qids))
    u = "https://query.wikidata.org/sparql?" + urllib.parse.urlencode({"query": q})
    req = urllib.request.Request(u, headers={"Accept": "application/sparql-results+json",
                                             "User-Agent": "sse-build/1.0"})
    rows = json.load(urllib.request.urlopen(req, timeout=120))["results"]["bindings"]
    out = {}
    for r in rows:
        x = r["x"]["value"].rsplit("/", 1)[-1]
        e = out.setdefault(x, {"label": r.get("xLabel", {}).get("value")})
        for k in ("died", "date", "deaths"):
            if k in r:
                e[k] = r[k]["value"]
    with open(WD_FILE, "w", encoding="utf-8") as fh:
        json.dump({"about": "Wikidata facts for scripts/spaceflight/in_memory.py (CC0), queried 25 Nov 2026 at "
                            "https://query.wikidata.org/sparql", "query": q, "items": out}, fh, ensure_ascii=False,
                  indent=1)
        fh.write("\n")


def build(people, aboard, pack_dir, gdate, logs, src):
    """Add deaths to `people` (and the people who died preparing to fly) and write in-memory.json."""
    facts = json.load(open(WD_FILE, encoding="utf-8"))["items"]
    by_short = {p["short"]: p for p in people}
    trainees, head = {}, None
    for line in open(os.path.join(src, "trainee.tsv"), encoding="utf-8"):
        f = line.rstrip("\n").split("\t")
        if line.startswith("#"):
            head = head or [h.lstrip("#") for h in f]
            continue
        r = {k: (f[j].strip() if j < len(f) else "") for j, k in enumerate(head)}
        trainees[r["Name"]] = r
    losses, problems = [], []
    for loss in LOSSES:
        y, m, d = (int(x) for x in loss["date"].split("-"))
        month = "Jan Feb Mar Apr May Jun Jul Aug Sep Oct Nov Dec".split()[m - 1]
        at = gdate("%d %s %d" % (y, month, d))[0]
        ids = []
        for short, qid in loss.get("crew", []):
            p = by_short.get(short)
            if not p:
                problems.append("%s: %s not in McDowell's list" % (loss["id"], short))
                continue
            ids.append((p, qid))
        for full, qid in loss.get("trainee", []):
            r = trainees.get(full)
            if not r:
                problems.append("%s: %s not in McDowell's trainee list" % (loss["id"], full))
                continue
            pid = "TR-" + re.sub(r"[^a-z]+", "-", full.split(",")[0].lower()).strip("-")
            p = next((x for x in people if x["id"] == pid), None)
            if not p:
                p = {"id": pid, "name": r["Name"], "short": r["ShortName"] or r["Name"], "being": "human",
                     "citizen": r["Citizen"], "sources": {"mcdowell": "trainee list"}, "born": r["Born"] or None,
                     "died": r["Died"] or None, "died_in_flight": False}
                people.append(p)
            ids.append((p, qid))
        # The flight's own end where McDowell has one (Challenger's breakup, Columbia's): the instant of the loss.
        if loss["during"] == "flight" and ids:
            ends = [leg["to_s"] for leg in aboard if leg["person"] == ids[0][0]["id"] and leg["to_s"]
                    and abs(leg["to_s"] - at) < 86_400]
            if ends:
                at = max(ends)
        ev = facts.get(loss["event"] or "", {})
        for p, qid in ids:
            died = facts.get(qid, {}).get("died", "")
            if not died.startswith(loss["date"]):
                problems.append("%s: Wikidata %s gives %s" % (loss["id"], qid, died or "no date"))
            p["sources"]["wikidata"] = qid
            p["death"] = {"at_s": at, "date": loss["date"], "where": loss["where"], "during": loss["during"],
                          "loss": loss["id"], "sources": [MCDOWELL, wd(qid), WIKI_LIST],
                          **({"on": loss["on"]} if loss.get("on") in logs else {})}
        toll = int(ev["deaths"]) if ev.get("deaths") else len(ids)
        if not loss.get("unconfirmed") and toll != len(ids):
            problems.append("%s: Wikidata gives %d deaths, %d named" % (loss["id"], toll, len(ids)))
        sources = [MCDOWELL, WIKI_LIST] + ([wd(loss["event"])] if loss["event"] else [])
        losses.append({"id": loss["id"], "at_s": at, "date": loss["date"], "where": loss["where"],
                       "during": loss["during"], "sentence": loss["sentence"],
                       "on": loss.get("on") if loss.get("on") in logs else None, "people": [p["id"] for p, _ in ids],
                       "unconfirmed": loss.get("unconfirmed"), "toll": toll, "sources": sources,
                       **({"note": loss["note"]} if loss.get("note") else {})})
    for g in GROUND:
        y, m, d = (int(x) for x in g["date"].split("-"))
        at = gdate("%d %s %d" % (y, "Jan Feb Mar Apr May Jun Jul Aug Sep Oct Nov Dec".split()[m - 1], d))[0]
        ev = facts.get(g["event"], {})
        if not ev.get("date", "").startswith(g["date"]):
            problems.append("%s: Wikidata date %s" % (g["id"], ev.get("date")))
        wd_toll = int(ev["deaths"]) if ev.get("deaths") else None
        low = g["toll"][0] if isinstance(g["toll"], list) else g["toll"]
        high = g["toll"][-1] if isinstance(g["toll"], list) else g["toll"]
        if wd_toll is None or not low <= wd_toll <= high:
            problems.append("%s: Wikidata toll %s outside %s" % (g["id"], wd_toll, g["toll"]))
        for name, qid in g["names"]:
            if not facts.get(qid, {}).get("died", "").startswith(g["date"]):
                problems.append("%s: %s's Wikidata date of death" % (g["id"], name))
        losses.append({"id": g["id"], "at_s": at, "date": g["date"], "where": g["where"], "during": g["during"],
                       "sentence": g["sentence"], "on": None, "people": [],
                       "names": [{"name": n, "sources": [WIKI_LIST, wd(q)]} for n, q in g["names"]],
                       "unconfirmed": g["unconfirmed"], "toll": g["toll"],
                       **({"toll_why": g["toll_why"]} if g.get("toll_why") else {}),
                       "sources": [WIKI_LIST, wd(g["event"])]})
    losses.sort(key=lambda x: x["at_s"])
    out = {"about": ("In memory: those who died on a flight, preparing to fly, or in ground work and tests, each with "
                     "at least two sources and one plain sentence of fact. Written by scripts/spaceflight/"
                     "build_crews.py (scripts/spaceflight/in_memory.py). No name is guessed: where the sources give "
                     "a number of dead but not every name, 'unconfirmed' says so."),
           "losses": losses}
    with open(os.path.join(pack_dir, "in-memory.json"), "w", encoding="utf-8") as fh:
        json.dump(out, fh, ensure_ascii=False, indent=1)
        fh.write("\n")
    return losses, problems


if __name__ == "__main__" and "--fetch" in sys.argv:
    fetch()
    print("wrote", WD_FILE)
