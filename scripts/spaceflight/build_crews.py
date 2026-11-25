#!/usr/bin/env python3
"""RS9: everyone who has been to space, and where each of them was, for the Sol today pack.

Usage (from the repo root, after build_logs.py and build_departed_logs.py):
  python3 -I scripts/spaceflight/build_crews.py <dir with McDowell's astro.tsv and segments.tsv>

The two tables are Jonathan McDowell's human spaceflight lists (https://planet4589.org/space/astro/web/index.html,
TSV at https://planet4589.org/space/astro/tsv/astro.tsv and segments.tsv; same author and licence as GCAT,
CC BY 4.0). astro.tsv has one row per space traveller (anyone above 80 km): name, born, died, citizenship.
segments.tsv has one row per leg of each person's flight: from and to (dated, often to the second), how it began and
ended ("Launch on Soyuz TMA-12", "Docked with ISS", "Transfer to Soyuz TMA-12", "Landed on Soyuz TMA-12"), and the
role on that leg. A crew that goes up on one ship and comes home on another is two craft in one person's legs, which
is why the roster follows people, not ships.

Writes store-packs/sol-today/crews.json:
  people: one per traveller (McDowell's AS- number as id), with born and died dates and whether they died in flight
          (their death falls on the last day of a flight);
  aboard: one per leg: person, from_s, to_s (universe seconds), the craft or station named (McDowell's words), the
          log it resolves to in logs/ (construct id) or null when the pack has no log for it (suborbital hops, for
          one), and the role.
Who was in space at time t is every leg with from_s <= t < to_s.
"""
import glob, json, os, re, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
PACK = os.path.join(ROOT, "store-packs/sol-today")
SRC = sys.argv[1]

MONTHS = {m: i + 1 for i, m in enumerate("Jan Feb Mar Apr May Jun Jul Aug Sep Oct Nov Dec".split())}
J2000_S = 63115200000
TT_UTC = [(1972, 42.184), (1973, 43.184), (1974, 44.184), (1975, 45.184), (1976, 46.184), (1977, 47.184),
          (1978, 48.184), (1979, 49.184), (1980, 50.184), (1981, 51.184), (1982, 52.184), (1983, 53.184),
          (1985, 54.184), (1988, 55.184), (1990, 56.184), (1991, 57.184), (1992, 58.184), (1993, 59.184),
          (1994, 60.184), (1996, 61.184), (1997, 62.184), (1999, 63.184), (2006, 64.184), (2009, 65.184),
          (2012, 66.184), (2015, 67.184), (2017, 69.184)]


def tt_minus_utc(year):
    out = 32.184 + 10.0 if year >= 1972 else 32.184 + 4.2 + 0.8 * max(0.0, year - 1961) * 0.6
    for y, d in TT_UTC:
        if year >= y:
            out = d
    return out


def gdate(s):
    """McDowell's date ('2008 Apr  8 1116:39', '1961 Apr 12 0607', '2021 Jul 20 1311:00?') to (seconds, exact?)."""
    import datetime
    s = (s or "").strip()
    if not s or s.startswith("-"):
        return None
    sure = "?" not in s
    m = re.match(r"(\d{4})(?: (\w{3})(?: +(\d+)(?: (\d{2})(\d{2})(?::(\d{2}(?:\.\d+)?))?)?)?)?", s.replace("?", ""))
    if not m:
        return None
    y, mon, d, hh, mm, ss = m.groups()
    sure = sure and hh is not None
    base = datetime.datetime(int(y), MONTHS.get(mon, 7) if mon else 7, int(d) if d else 1)
    base += datetime.timedelta(hours=int(hh), minutes=int(mm), seconds=float(ss or 0)) if hh else \
        datetime.timedelta(hours=12)
    secs = (base - datetime.datetime(2000, 1, 1, 12)).total_seconds() + tt_minus_utc(int(y))
    return int(round(J2000_S + secs)), sure


def table(name):
    rows, head = [], None
    with open(os.path.join(SRC, name), encoding="utf-8") as fh:
        for line in fh:
            line = line.rstrip("\n")
            if line.startswith("#"):
                if head is None:
                    head = line[1:].split("\t")
                continue
            f = line.split("\t")
            rows.append({k: (f[j].strip() if j < len(f) else "") for j, k in enumerate(head)})
    return rows


UPDATED = next(line[len("# Updated "):].strip() for line in open(os.path.join(SRC, "segments.tsv"), encoding="utf-8")
               if line.startswith("# Updated"))

# ---------------------------------------------------------------- the pack's logs, by name
LOGS = {}
for f in glob.glob(os.path.join(PACK, "logs/**/*.json"), recursive=True):
    d = json.load(open(f, encoding="utf-8"))
    LOGS.setdefault(d.get("name", ""), d["construct"])
    LOGS.setdefault(d["construct"], d["construct"])
BY_LOWER = {k.lower(): v for k, v in LOGS.items()}
# McDowell's station and craft names where the pack's logs are named otherwise (the showcase logs).
ALIASES = {"ISS": "iss", "Skylab": "skylab-orbital-workshop", "Apollo 11 CSM": "apollo-11", "Apollo 11": "apollo-11",
           "Columbia (STS-1)": "sts-1-columbia", "LM 5": "apollo-11-eagle"}


# Apollo lunar modules by number (GCAT names them by call sign; the full LM's log, launched with the CSM).
LM_NAMES = {"3": "Spider", "4": "snoopy", "5": "apollo-11-eagle", "6": "intrepid", "7": "aquarius", "8": "antares",
            "10": "falcon", "11": "orion", "12": "challenger"}


def resolve(craft):
    if craft in ALIASES and ALIASES[craft] in LOGS:
        return ALIASES[craft]
    m = re.search(r"\b(Crew-\d+|SpX DM-2|DM-2|STS[- ][\w-]+)\)?$", craft)
    if m:  # "Dragon Crew-8" is GCAT's "Dragon Endeavour (Crew-8)"; "STS-71" is "Atlantis (STS-71)"
        key = "(" + ("SpX DM-2" if m.group(1) == "DM-2" else m.group(1)) + ")"
        hit = next((v for k, v in LOGS.items() if k.endswith(key)), None)
        if hit:
            return hit
    m = re.match(r"TG-(\d) Expedition", craft)
    if m:
        return LOGS.get("Tiangong %s" % m.group(1))
    m = re.match(r"Skylab (\d)$", craft)
    if m:
        return LOGS.get("Skylab SL-%s" % m.group(1))
    hit = LOGS.get(craft) or BY_LOWER.get(craft.lower())
    if hit:
        return hit
    m = re.match(r"LM (\d+)$", craft)
    if m and m.group(1) in LM_NAMES:
        n = LM_NAMES[m.group(1)]
        return n if n in LOGS and LOGS[n] == n else LOGS.get(n)
    m = re.match(r"(Apollo \d+)(?: CSM)?$", craft)
    if m:
        return LOGS.get(m.group(1) + " CSM") or next((v for k, v in LOGS.items() if k.startswith(m.group(1) + " (")),
                                                     None)
    return None


def target(event):
    """The craft or station an event names: 'Launch on X', 'Transfer to X', 'Docked with X', 'Landed on X'."""
    m = re.match(r"(?:Launch on|Transfer to|Docked with|Redocked with|Landed on|Land on)\s+(.+)$", event)
    return m.group(1).strip() if m else None


# ---------------------------------------------------------------- people
people, index = [], {}
for r in table("astro.tsv"):
    born, died = gdate(r["Born"]), gdate(r["Died"])
    p = {"id": r["No"], "name": r["Name"], "short": r["ShortName"], "being": "human", "citizen": r["Citizen"],
         "sources": {"mcdowell": r["No"]},
         "born": r["Born"] or None, "died": r["Died"] if r["Died"] and not r["Died"].startswith("-") else None,
         "died_in_flight": False}
    index[r["No"]] = p
    people.append(p)

# ---------------------------------------------------------------- legs
aboard, unresolved = [], {}
by_person = {}
for r in table("segments.tsv"):
    by_person.setdefault(r["ID"], []).append(r)
for pid, rows in by_person.items():
    if pid not in index:  # the chimpanzees Ham and Enos (AA- numbers) fly in the segments but not in astro.tsv
        index[pid] = {"id": pid, "name": rows[0]["Name"], "short": rows[0]["Name"],
                      "being": "chimpanzee" if pid.startswith("AA-") else None, "citizen": None,
                      "sources": {"mcdowell": pid}, "born": None, "died": None, "died_in_flight": False}
        people.append(index[pid])
    where = None
    for r in rows:
        a, b = gdate(r["Date1"]), gdate(r["Date2"])
        if not a or "Abort" in r["Event1"] or re.search(r"X-15-\d Flight \d-A-|\[Soyuz T\]", r["Event1"]):
            continue  # pad aborts (RSLS, Soyuz T-10-1's escape) and aborted X-15 drops never reached 80 km
        start = target(r["Event1"])
        if start:
            where = start
        elif r["Event1"].startswith(("Expedition", "Reassigned", "Mission ", "TG-")) and where:
            pass  # a stay on the station they are already on (an expedition, or a visiting mission)
        elif r["Event1"]:
            where = r["Event1"]  # the leg names the craft itself ("Soyuz TMA-12", "LM 5")
        on = resolve(where) if where else None
        if where and not on:
            unresolved[where] = unresolved.get(where, 0) + 1
        leg = {"person": pid, "from_s": a[0], "to_s": max(b[0], a[0] + 1) if b else None, "craft": where, "on": on,
               "role": r["Role"], "ride": r["RideRoleCode"], "src": "mcdowell",
               "mark": "recorded" if a[1] and (b is None or b[1]) else "estimated"}
        if leg["mark"] == "estimated":
            leg["why"] = "McDowell gives a time here without the hour, or marks it uncertain"
        if b is None:
            leg["still"] = "in space when the list was last updated (%s)" % UPDATED
        aboard.append(leg)
        nxt = target(r["Event2"])
        if nxt and not r["Event2"].startswith(("Land", "Landed")):
            where = nxt
    # Died in flight: the death falls on the last day of one of their flights.
    p = index.get(pid)
    if p and p["died"]:
        d = gdate(p["died"])
        ends = [leg["to_s"] for leg in aboard if leg["person"] == pid and leg["to_s"]]
        if d and any(abs(d[0] - e) < 86_400 for e in ends):
            p["died_in_flight"] = True

aboard.sort(key=lambda x: (x["from_s"], x["person"]))

# Peak heights of the short flights, from McDowell's suborbital table (suborb.html, saved beside the TSVs from
# https://planet4589.org/space/astro/web/suborb.html). The pack's edge of space is one value (edge_of_space_km,
# 100 by default); McDowell lists everyone above 80 km, so a viewer marks the flights between the two (most X-15 and
# SpaceShipTwo flights). Matched by launch day; flights newer than the table keep no height.
PEAKS = {}
SUB = os.path.join(SRC, "suborb.html")
if os.path.exists(SUB):
    for line in re.sub(r"<[^>]*>", " ", open(SUB, encoding="utf-8").read()).split("\n"):
        m = re.match(r"\s*SO\d+ (\d{4}) (\w{3}) +(\d+) .*? (\d+(?:\.\d+)?)(\??) (\d+:[\d?]+)", re.sub(r" +", " ", line))
        if m:
            PEAKS[(int(m.group(1)), MONTHS[m.group(2)], int(m.group(3)))] = (float(m.group(4)), not m.group(5))

# How many were in space, as steps through time (each change in the count), for drawing along the time ruler.
# The legs with no pack log are mostly the suborbital hops (X-15, New Shepard, SpaceShipTwo), flagged per leg.
for leg in aboard:
    leg["suborbital"] = leg["on"] is None and bool(re.match(
        r"(X-15|New Shepard|Spaceship|Unity|Galactic|Mercury MR|Freedom Seven|Liberty Bell|\[Soyuz|Soyuz MS-10$)",
        leg["craft"] or ""))
    if leg["suborbital"]:
        import datetime
        day = datetime.datetime(2000, 1, 1, 12) + datetime.timedelta(seconds=leg["from_s"] - J2000_S - 60)
        peak = PEAKS.get((day.year, day.month, day.day))
        leg["peak_km"] = peak[0] if peak else None
        if peak and not peak[1]:
            leg["peak_why"] = "McDowell marks this height uncertain"
changes = {}
for leg in aboard:
    changes[leg["from_s"]] = changes.get(leg["from_s"], 0) + 1
    if leg["to_s"] is not None:
        changes[leg["to_s"]] = changes.get(leg["to_s"], 0) - 1
steps, n = [], 0
for t in sorted(changes):
    if changes[t]:
        n += changes[t]
        steps.append([t, n])
out = {"about": ("Everyone who has been above 80 km, and each leg of their flights: who was where, when. From "
                 "Jonathan McDowell's human spaceflight lists (astro.tsv, segments.tsv, suborb.html for the short flights' heights), written by "
                 "scripts/spaceflight/build_crews.py. Who was in space at time t is every leg with from_s <= t < to_s; "
                 "'on' is the pack log for the craft or station, null where the pack has none (suborbital hops)."),
       "credit": "Jonathan C. McDowell, Jonathan's Space Report human spaceflight lists, planet4589.org, CC BY 4.0",
       "updated": UPDATED, "listed_from_km": 80, "edge_of_space_km": 100,
       "people": people, "aboard": aboard, "in_space": steps}
with open(os.path.join(PACK, "crews.json"), "w", encoding="utf-8") as fh:
    json.dump(out, fh, ensure_ascii=False, indent=0)
    fh.write("\n")
n_on = sum(1 for x in aboard if x["on"])
print(len(PEAKS), "suborbital heights;", sum(1 for x in aboard if x.get("peak_km")), "legs with one,",
      sum(1 for x in aboard if x.get("peak_km") and x["peak_km"] < 100), "of them below 100 km")
print(len(people), "people,", len(aboard), "legs,", n_on, "on a pack log;",
      sum(p["died_in_flight"] for p in people), "died in flight")
print("unresolved", [k for k in sorted(unresolved, key=lambda k: -unresolved[k]) if not k.startswith(("New Shepard", "Galactic", "Unity"))])
