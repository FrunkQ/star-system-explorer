#!/usr/bin/env python3
"""RS8: a flight log for every craft that left Earth orbit, from GCAT's phase tables.

Usage: python3 -I scripts/spaceflight/build_departed_logs.py <dir with GCAT .tsv files>

Download deepcat.tsv, hcocat.tsv, lprcat.tsv and landercat.tsv from https://planet4589.org/space/gcat/tsv/
(the same files import_gcat.py reads). Writes store-packs/sol-today/logs/departed/<construct>.json, one log for
each payload in catalogue.json's `departed` list, except the showcase missions that build_logs.py writes in full.

GCAT gives each craft's life as phases: which body it was going around, from when to when, its periapsis and
apoapsis heights and inclination, and how the phase ended (left for another body, flew by, went into orbit,
landed, struck, burnt up, docked). Each phase becomes one piece of the log:

- around a planet or moon: an orbit (a hyperbola for an escape or a flyby) with GCAT's shape and inclination;
- around the Sun, ending at a planet the NAV Sol fixture holds: a course solved from its ends (Lambert), kept
  only when the solved arc's perihelion and aphelion agree with GCAT's within 20%;
- around the Sun otherwise: an orbit with GCAT's perihelion, aphelion and inclination, turned so that it starts
  where the craft left its last body;
- on a surface: landed, at landercat's site.

GCAT does not give the orientation of an orbit (node, argument of periapsis) or where along it the craft was,
so every orbit piece is marked estimated and says so; the shape, the inclination, the bodies and the dates are
GCAT's. Earth-centred inclinations are equatorial, turned to the ecliptic with the node put at the equinox.
Times are UTC in GCAT, turned to TT here; a date GCAT marks uncertain ('?') or gives without an hour is marked
estimated. Big burns and flybys only (fudge register: "flight logs: big burns and flybys only").
"""
import json, math, os, re, sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
PACK = os.path.join(ROOT, "store-packs/sol-today")
OUT = os.path.join(PACK, "logs/departed")
SRC_DIR = sys.argv[1]

J2000_S = 63115200000
DAY = 86400
AU = 1.495978707e11
SUN = "u:sol/sys:sun"
P = lambda n: SUN + "/body:" + n
G = 6.6743e-11
MONTHS = {m: i + 1 for i, m in enumerate("Jan Feb Mar Apr May Jun Jul Aug Sep Oct Nov Dec".split())}

# The NAV Sol fixture: planets on fixed elements, as the Rust side reads them.
FIX = json.load(open(os.path.join(ROOT, "tests/fixtures/nav/sol.json"), encoding="utf-8"))
M_SUN, M_EARTH = 1.98847e30, 5.9722e24
MU_SUN = G * FIX["star_mass_msun"] * M_SUN
PLANETS = {b["name"]: b for b in FIX["bodies"]}

# Mean radii in km for turning GCAT's heights into distances (NSSDCA planetary fact sheets for the planets and
# the Moon, as in packs/base-sol; the moons and small bodies from memory, to verify against JPL SSD's physical
# parameters; they move a flyby distance by a few km at most).
WORLDS = {
    "Mercury": (P("mercury"), 2439.7), "Venus": (P("venus"), 6051.8), "Earth": (P("earth"), 6378.137),
    "Luna": (P("earth") + "/body:moon", 1737.4), "Mars": (P("mars"), 3389.5), "Jupiter": (P("jupiter"), 69911.0),
    "Saturn": (P("saturn"), 58232.0), "Uranus": (P("uranus"), 25362.0), "Neptune": (P("neptune"), 24622.0),
    "Pluto": (P("pluto"), 1188.3), "Ceres": (P("ceres"), 469.7), "Vesta": (P("vesta"), 262.7),
    "Phobos": (P("mars") + "/body:phobos", 11.08), "Deimos": (P("mars") + "/body:deimos", 6.2),
    "Io": (P("jupiter") + "/body:io", 1821.6), "Europa": (P("jupiter") + "/body:europa", 1560.8),
    "Ganymede": (P("jupiter") + "/body:ganymede", 2631.2), "Callisto": (P("jupiter") + "/body:callisto", 2410.3),
    "Titan": (P("saturn") + "/body:titan", 2574.7), "Enceladus": (P("saturn") + "/body:enceladus", 252.1),
    "Dione": (P("saturn") + "/body:dione", 561.4), "Rhea": (P("saturn") + "/body:rhea", 763.8),
    "Tethys": (P("saturn") + "/body:tethys", 531.1), "Phoebe": (P("saturn") + "/body:phoebe", 106.5),
    "Ryugu": (P("ryugu"), 0.45), "Itokawa": (P("itokawa"), 0.16), "Bennu": (P("bennu"), 0.245),
    "Eros": (P("eros"), 8.4), "(951) Gaspra": (P("gaspra"), 6.1), "(243) Ida": (P("ida"), 15.7),
    "Mathilde": (P("mathilde"), 26.4), "Steins": (P("steins"), 2.6), "Lutetia": (P("lutetia"), 49.0),
    "Didymos": (P("didymos"), 0.38), "Toutatis": (P("toutatis"), 1.2), "67P/": (P("67p-churyumov-gerasimenko"), 2.0),
    "9P/Tempel 1": (P("9p-tempel-1"), 3.0), "81P/Wild 2": (P("81p-wild-2"), 2.0),
}
# Atmospheres a craft burns up in (GCAT 'R' and 'D' there mean a destructive entry; elsewhere an impact).
AIR = {"Earth", "Venus", "Mars", "Titan", "Jupiter", "Saturn", "Uranus", "Neptune"}
# The showcase missions build_logs.py writes in full, by GCAT name.
SHOWCASE = {"Cassini", "Huygens", "Juno", "Dawn", "Curiosity", "Mars Science Laboratory", "MSL", "JWST",
            "James Webb Space Telescope", "Voyager 2", "Columbia", "Eagle"}
TT_UTC = [(1972, 42.184), (1973, 43.184), (1974, 44.184), (1975, 45.184), (1976, 46.184), (1977, 47.184),
          (1978, 48.184), (1979, 49.184), (1980, 50.184), (1981.5, 51.184), (1982.5, 52.184), (1983.5, 53.184),
          (1985.5, 54.184), (1988, 55.184), (1990, 56.184), (1991, 57.184), (1992.5, 58.184), (1993.5, 59.184),
          (1994.5, 60.184), (1996, 61.184), (1997.5, 62.184), (1999, 64.184), (2006, 65.184), (2009, 66.184),
          (2012.5, 67.184), (2015.5, 68.184), (2017, 69.184)]


def tt_minus_utc(year):
    out = 32.184 + 10.0 if year >= 1972 else 32.184 + 4.2 + 0.8 * max(0.0, year - 1961) * 0.6
    for y, d in TT_UTC:
        if year >= y:
            out = d
    return out


def gdate(s):
    """GCAT's vague date to (universe seconds on TT, exact?). None for '-'."""
    s = s.strip()
    if not s or s.startswith("-"):
        return None
    sure = "?" not in s
    s = s.replace("?", "").strip()
    m = re.match(r"(\d{4})(?: (\w{3})(?: +(\d+(?:\.\d+)?)(?: (\d{2})(\d{2})(?::(\d{2}(?:\.\d+)?))?)?)?)?", s)
    if not m:
        return None
    y, mon, d, hh, mm, ss = m.groups()
    sure = sure and hh is not None
    mon = MONTHS.get(mon, 7) if mon else 7
    day = float(d) if d else (15.0 if mon else 1.0)
    import datetime
    base = datetime.datetime(int(y), mon, 1) + datetime.timedelta(days=day - 1)
    if hh is not None:
        base += datetime.timedelta(hours=int(hh), minutes=int(mm), seconds=float(ss or 0))
    elif d and "." not in d:
        base += datetime.timedelta(hours=12)
    j2000 = datetime.datetime(2000, 1, 1, 12)
    secs = (base - j2000).total_seconds() + tt_minus_utc(int(y) + (mon - 0.5) / 12)
    return int(round(J2000_S + secs)), sure


def num(s):
    s = s.strip()
    try:
        x = float(s)
    except ValueError:
        return None
    return x if math.isfinite(x) else None


# ---------------------------------------------------------------- vectors and orbits
def dot(a, b):
    return sum(x * y for x, y in zip(a, b))


def cross(a, b):
    return [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]


def nrm(a):
    return math.sqrt(dot(a, a))


def kepler_state(a, e, i, node, argp, m0, mu, dt):
    if e < 1:
        n = math.sqrt(mu / a ** 3)
        M = m0 + n * dt
        E = M
        for _ in range(60):
            E -= (E - e * math.sin(E) - M) / (1 - e * math.cos(E))
        x, y = a * (math.cos(E) - e), a * math.sqrt(1 - e * e) * math.sin(E)
        r = a * (1 - e * math.cos(E))
        vx, vy = -math.sqrt(mu * a) / r * math.sin(E), math.sqrt(mu * a * (1 - e * e)) / r * math.cos(E)
    else:
        aa = -a
        n = math.sqrt(mu / aa ** 3)
        M = m0 + n * dt
        H = math.asinh(M / e)
        for _ in range(100):
            H -= (e * math.sinh(H) - H - M) / (e * math.cosh(H) - 1)
        x, y = aa * (e - math.cosh(H)), aa * math.sqrt(e * e - 1) * math.sinh(H)
        r = aa * (e * math.cosh(H) - 1)
        vx, vy = -math.sqrt(mu * aa) / r * math.sinh(H), math.sqrt(mu * aa * (e * e - 1)) / r * math.cosh(H)
    cO, sO, cw, sw, ci, si = math.cos(node), math.sin(node), math.cos(argp), math.sin(argp), math.cos(i), math.sin(i)
    rot = lambda px, py: [px * (cO * cw - sO * sw * ci) - py * (cO * sw + sO * cw * ci),
                          px * (sO * cw + cO * sw * ci) - py * (sO * sw - cO * cw * ci),
                          px * sw * si + py * cw * si]
    return rot(x, y), rot(vx, vy)


def planet_at(name, t):
    b = PLANETS[name]
    argp = math.radians(b["varpi_deg"] - b["node_deg"])
    m0 = math.radians(b["mean_longitude_deg"] - b["varpi_deg"])
    return kepler_state(b["a_au"] * AU, b["e"], math.radians(b["i_deg"]), math.radians(b["node_deg"]), argp, m0,
                        MU_SUN, t - FIX["t0_s"])


def stumpff(z):
    if z > 1e-6:
        s = math.sqrt(z)
        return (1 - math.cos(s)) / z, (s - math.sin(s)) / s ** 3
    if z < -1e-6:
        s = math.sqrt(-z)
        return (math.cosh(s) - 1) / -z, (math.sinh(s) - s) / s ** 3
    return 0.5, 1 / 6


def lambert(r1, r2, tof, mu):
    """Prograde single-revolution Lambert (universal variables); the departure velocity or None."""
    n1, n2 = nrm(r1), nrm(r2)
    c = max(-1.0, min(1.0, dot(r1, r2) / (n1 * n2)))
    dth = math.acos(c)
    if cross(r1, r2)[2] < 0:
        dth = 2 * math.pi - dth
    A = math.sin(dth) * math.sqrt(n1 * n2 / (1 - c))
    if abs(A) < 1e-9:
        return None

    def y(z):
        C, S = stumpff(z)
        return n1 + n2 + A * (z * S - 1) / math.sqrt(C)

    def F(z):
        C, S = stumpff(z)
        yy = y(z)
        if yy < 0:
            return None
        return (yy / C) ** 1.5 * S + A * math.sqrt(yy) - math.sqrt(mu) * tof

    lo, hi = -4 * math.pi ** 2, 4 * math.pi ** 2 - 1e-6
    while y(lo) < 0 or F(lo) is None:
        lo += 0.1
        if lo > hi:
            return None
    flo, fhi = F(lo), F(hi)
    if flo is None or fhi is None or flo * fhi > 0:
        return None
    for _ in range(200):
        mid = 0.5 * (lo + hi)
        fm = F(mid)
        if fm is None:
            lo = mid
            continue
        if (fm < 0) == (flo < 0):
            lo, flo = mid, fm
        else:
            hi = mid
    z = 0.5 * (lo + hi)
    yy = y(z)
    f = 1 - yy / n1
    g = A * math.sqrt(yy / mu)
    return [(r2[k] - f * r1[k]) / g for k in range(3)]


def peri_apo(r, v, mu):
    a = 1 / (2 / nrm(r) - dot(v, v) / mu)
    h = cross(r, v)
    e = math.sqrt(max(0.0, 1 - dot(h, h) / (mu * a)))
    return a * (1 - e), a * (1 + e)


EPS = math.radians(23.4392911)


def equatorial_to_ecliptic(i_deg, node_deg=0.0):
    """Inclination and node about the ecliptic for an orbit given about the Earth's equator."""
    i, node = math.radians(i_deg), math.radians(node_deg)
    hq = [math.sin(i) * math.sin(node), -math.sin(i) * math.cos(node), math.cos(i)]
    h = [hq[0], hq[1] * math.cos(EPS) + hq[2] * math.sin(EPS), -hq[1] * math.sin(EPS) + hq[2] * math.cos(EPS)]
    ie = math.degrees(math.acos(max(-1, min(1, h[2]))))
    ne = math.degrees(math.atan2(h[0], -h[1])) % 360
    return ie, ne


# ---------------------------------------------------------------- GCAT tables
def table(name):
    rows = []
    with open(os.path.join(SRC_DIR, name), encoding="utf-8") as fh:
        head = None
        for line in fh:
            line = line.rstrip("\n")
            if line.startswith("#"):
                if head is None:
                    head = line[1:].split("\t")
                continue
            f = line.split("\t")
            rows.append({k: (f[j].strip() if j < len(f) else "") for j, k in enumerate(head)})
    return rows


PHASES = {}
for n, name in enumerate(("deepcat.tsv", "hcocat.tsv", "lprcat.tsv")):
    for k, r in enumerate(table(name)):
        r["_order"] = (n, k)
        PHASES.setdefault(r["JCAT"], []).append(r)
LANDER = {r["JCAT"]: r for r in table("landercat.tsv")}
CAT = json.load(open(os.path.join(PACK, "catalogue.json"), encoding="utf-8"))


def slug(s):
    return re.sub(r"[^a-z0-9]+", "-", s.lower().replace("'", "")).strip("-")


IDS = {}
taken = {os.path.splitext(f)[0] for f in os.listdir(os.path.join(PACK, "logs")) if f.endswith(".json")}
for d in CAT["departed"]:
    if d["name"] in SHOWCASE:
        continue
    cid = slug(d["name"]) or slug(d["piece"])
    if cid in taken:
        cid = cid + "-" + slug(d["piece"])
    while cid in taken:
        cid += "-" + d["jcat"].lower()
    taken.add(cid)
    IDS[d["jcat"]] = cid

SKIP_STATUS = {"N", "NA", "REFLT", "ERR", "LEASE", "EVA DP", "EVA RP", "TFR", "TFR E", "TFR IN", "ATT"}
SURFACE = {"TX", "DEP", "TO", "TOA", "LO", "LOA"}
# Carried by another craft through the phase: GCAT's own figures can be stale copies, so the carrier's are used.
ATTACHED = {"EAO", "DSA", "DSA IN", "AO", "AO IN", "AL", "AL IN", "AR", "AR IN", "AS", "AF", "REL", "UDK"}
HOW = "GCAT gives the orbit's shape and inclination, not its orientation or where along it the craft was"


def coords(dest):
    """'63.834W 7.850N' or '26.22S 175.88W' to (lat, lon)."""
    lat = lon = None
    for v, h in re.findall(r"([\d.]+)([NSEW])", dest or ""):
        x = float(v) * (1 if h in "NE" else -1)
        if h in "NS":
            lat = x
        else:
            lon = x
    return (lat, lon) if lat is not None and lon is not None else None


def borrow(parent, world, t):
    """A carrier's own shape and inclination around `world` at time `t`, from its GCAT phases."""
    best = None
    for q in PHASES.get(parent, []):
        if q["Primary"] != world or num(q["Perigee"]) is None:
            continue
        s0, s1 = gdate(q["SDate"]), gdate(q["DDate"])
        if s0 and s0[0] <= t and (s1 is None or t <= s1[0]):
            best = q
    if best is None:
        return None
    return num(best["Perigee"]), num(best["Apogee"]), num(best["Inc"]) or 0.0


def build(d):
    cid = IDS[d["jcat"]]
    rows = [r for r in PHASES.get(d["jcat"], []) if r["Status"] not in SKIP_STATUS]
    for r in rows:
        r["_s"] = gdate(r["SDate"])
        r["_d"] = gdate(r["DDate"])
    rows = [r for r in rows if r["_s"]]
    rows.sort(key=lambda r: (r["_s"][0], r["_order"]))
    # A phase that wholly holds another around a different body is a slip in the table (Galileo's probe has
    # an Earth phase dated a year early): drop it.
    span = lambda r: (r["_s"][0], r["_d"][0] if r["_d"] else None)
    rows = [r for r in rows if r["Status"] in SURFACE or not r["_d"] or not any(
        q is not r and q["Primary"] != r["Primary"] and q["_d"] and span(r)[0] < span(q)[0] and span(q)[1] < span(r)[1]
        for q in rows)]
    if not rows:
        return None
    pieces, events = [], []
    lander = LANDER.get(d["jcat"])

    def piece(t, state, sure, why):
        mark = "recorded" if sure and not why else "estimated"
        p = {"from_s": t, "state": state, "mark": mark, "src": "gcat", "writer": "pack"}
        if mark == "estimated":
            p["why"] = why or "GCAT marks this date uncertain or gives it without an hour"
        if pieces and pieces[-1]["from_s"] >= t:
            pieces.pop()
        if pieces:
            pieces[-1]["to_s"] = t
        pieces.append(p)

    def event(t, kind, sure, why=None, **k):
        e = dict({"at_s": t, "kind": kind, "mark": "recorded" if sure and not why else "estimated", "src": "gcat",
                  "writer": "pack"}, **k)
        if e["mark"] == "estimated":
            e["why"] = why or "GCAT marks this date uncertain or gives it without an hour"
        events.append(e)

    def landed_at(world, dest):
        if lander and lander["World"] == world and num(lander["Lat"]) is not None and num(lander["Lon"]) is not None:
            return num(lander["Lat"]), num(lander["Lon"]), True
        c = coords(dest)
        if c:
            return c[0], c[1], True
        return 0.0, 0.0, False

    last_body = None
    ended = False
    for k, r in enumerate(rows):
        t0, sure0 = r["_s"]
        world = r["Primary"]
        st = r["Status"]
        hp, ha, inc = num(r["Perigee"]), num(r["Apogee"]), num(r["Inc"]) or 0.0
        borrowed = False
        if r["Parent"].startswith("D") and (hp is None or st in ATTACHED):
            # Carried by (or docked to) another craft: GCAT gives that craft's orbit.
            got = borrow(r["Parent"], world, t0)
            if got:
                hp, ha, inc = got
                borrowed = True
        if st == "UDK" and r["Parent"].startswith("D"):
            piece(t0, {"kind": "docked", "to": IDS.get(r["Parent"], "gcat:" + r["Parent"])}, sure0, None)
            if nxt is None and r["_d"]:
                piece(max(r["_d"][0], t0 + 1), {"kind": "gone", "how": "lost_contact"}, r["_d"][1],
                      "undocked; GCAT's record of it ends here")
            last_body = world
            continue
        nxt = rows[k + 1] if k + 1 < len(rows) else None
        if st in SURFACE:
            if world in WORLDS:
                lat, lon, known = landed_at(world, r["Dest"])
                piece(t0, {"kind": "landed", "body": WORLDS[world][0], "lat_deg": lat, "lon_deg": lon}, sure0,
                      None if known else "GCAT gives no landing site; the sub-point is a stand-in")
            if st in ("LO", "LOA", "TO", "TOA") and r["_d"]:
                event(r["_d"][0], "launch", r["_d"][1])
                if not nxt:
                    pieces[-1]["to_s"] = r["_d"][0]
            elif st == "TX" and r["_d"] and not nxt:
                event(r["_d"][0], "stop", r["_d"][1], type="loiter", text="last signal from the surface")
            last_body = world
            continue
        if world == "Sun":
            dest = r["Dest"]
            arrive = r["_d"]
            q, Q = hp, ha
            done = False
            # A course solved from its ends, when it ends at a planet the fixture holds and agrees with GCAT.
            if dest in PLANETS and arrive and last_body in PLANETS and dest != last_body and not (q and Q) and q is None:
                r1, _ = planet_at(last_body, t0)
                r2, _ = planet_at(dest, arrive[0])
                v1 = lambert(r1, r2, arrive[0] - t0, MU_SUN)
                if v1:
                    sq, sQ = peri_apo(r1, v1, MU_SUN)
                    if 0 < sq and sQ > 0 and sQ / AU < 12:
                        piece(t0, {"kind": "course", "course": {"kind": "lambert", "to": P(dest.lower()),
                                                                 "arrive_s": arrive[0]}}, False,
                              "the arc is solved from its ends (leaving %s, meeting %s on GCAT's date); GCAT gives no "
                              "heliocentric orbit to check it against (solved: perihelion %.3f au, aphelion %.3f au)" % (
                                  last_body, dest, sq / AU, sQ / AU))
                        done = True
            if dest in PLANETS and arrive and last_body in PLANETS and dest != last_body and q and Q:
                r1, _ = planet_at(last_body, t0)
                r2, _ = planet_at(dest, arrive[0])
                v1 = lambert(r1, r2, arrive[0] - t0, MU_SUN)
                if v1:
                    sq, sQ = peri_apo(r1, v1, MU_SUN)
                    if (sQ > 0) == (Q > 0) and abs(sq / AU - q) / q < 0.2 and abs(sQ / AU - Q) / abs(Q) < 0.2:
                        piece(t0, {"kind": "course", "course": {"kind": "lambert", "to": P(dest.lower()),
                                                                 "arrive_s": arrive[0]}}, sure0 and arrive[1],
                              "the arc is solved from its ends (leaving %s, meeting %s on GCAT's date); its perihelion "
                              "%.3f au and aphelion %.3f au against GCAT's %.3f and %.3f" % (
                                  last_body, dest, sq / AU, sQ / AU, q, Q))
                        done = True
            if not done and q and Q:
                if Q < 0:
                    a = -abs((q + Q) / 2) * AU
                    e = (Q - q) / (Q + q)
                else:
                    a = (q + Q) / 2 * AU
                    e = (Q - q) / (Q + q)
                el = {"a_m": a, "e": e, "i_deg": inc, "node_deg": 0.0, "peri_deg": 0.0, "m0_deg": 0.0, "epoch_s": t0}
                why = HOW + "; turned to start where it left " + (last_body or "its last body")
                home = "Earth" if last_body in ("Luna", "SEL1", "SEL2", "EML1", "EML2") else last_body
                if home in PLANETS:
                    rb, _ = planet_at(home, t0)
                    r0 = nrm(rb)
                    p_ = a * (1 - e * e)
                    cnu = max(-1.0, min(1.0, (p_ / r0 - 1) / e)) if e > 1e-6 else 1.0
                    nu = math.acos(cnu)
                    if Q > 0 and (Q * AU - r0) < (r0 - q * AU):
                        nu = -nu
                    lam = math.atan2(rb[1], rb[0])
                    if e < 1:
                        E = 2 * math.atan(math.sqrt((1 - e) / (1 + e)) * math.tan(nu / 2))
                        M = E - e * math.sin(E)
                    else:
                        H = 2 * math.atanh(math.sqrt((e - 1) / (e + 1)) * math.tan(nu / 2))
                        M = e * math.sinh(H) - H
                    el.update(node_deg=math.degrees(lam) % 360, peri_deg=math.degrees(-nu) % 360,
                              m0_deg=math.degrees(M) % 360)
                elif home not in PLANETS:
                    why = HOW
                piece(t0, {"kind": "orbit", "around": SUN, "elements": el}, sure0, why)
            elif not done:
                piece(t0, {"kind": "course", "course": {"kind": "planned", "course": {"what": "around the Sun"}}},
                      sure0, "GCAT gives no heliocentric orbit for this phase")
            last_body = "Sun"
        elif world in ("SEL1", "SEL2"):
            b = PLANETS["Earth"]
            s = 0.99 if world == "SEL1" else 1.01
            earth_r, earth_v = planet_at("Earth", t0)
            lam = math.degrees(math.atan2(earth_r[1], earth_r[0]))
            el = {"a_m": b["a_au"] * AU * s, "e": b["e"], "i_deg": 0.0, "node_deg": 0.0,
                  "varpi_deg": b["varpi_deg"], "mean_longitude_deg": lam, "epoch_s": t0}
            piece(t0, {"kind": "orbit", "around": SUN, "elements": el}, sure0,
                  "about the Sun-Earth %s point, about 1.5 million km %s the Earth: shown as an orbit like the Earth's, "
                  "1%% %s, with the halo or Lissajous loop left out" % (
                      world[-2:], "sunward of" if s < 1 else "beyond", "inside" if s < 1 else "outside"))
            last_body = world
        elif world in ("EML1", "EML2"):
            s = 0.85 if world == "EML1" else 1.17
            el = {"a_m": 384400e3 * s, "e": 0.0, "i_deg": 5.1, "node_deg": 0.0, "peri_deg": 0.0, "m0_deg": 0.0,
                  "epoch_s": t0}
            piece(t0, {"kind": "orbit", "around": P("earth"), "elements": el}, sure0,
                  "about the Earth-Moon %s point: shown as a circle around the Earth at its distance, the loop left out"
                  % world[-2:])
            last_body = world
        elif world in WORLDS and hp is not None:
            addr, R = WORLDS[world]
            rp = (R + hp) * 1e3
            if hp < -0.5 * R or (ha is not None and 0 <= ha < 1):
                # GCAT's convention for a straight-in fall (periapsis deep below the ground): no orbit to draw.
                piece(t0, {"kind": "course", "course": {"kind": "planned", "course": {"what": "falling to " + world}}},
                      sure0, "GCAT gives a straight-in path to the ground, periapsis below the surface")
                last_body = world
                if nxt is None and r["_d"]:
                    t1, sure1 = r["_d"]
                    lat, lon, known = landed_at(world, r["Dest"])
                    soft = st in ("L", "AL", "AL IN") and not (lander and lander["Status"][:1] in ("I", "F"))
                    if soft:
                        piece(max(t1, t0 + 1), {"kind": "landed", "body": addr, "lat_deg": lat, "lon_deg": lon}, sure1,
                              None if known else "GCAT gives no landing site; the sub-point is a stand-in")
                    else:
                        piece(max(t1, t0 + 1), {"kind": "gone", "how": "reentered" if world in AIR and st != "L" else "impacted"},
                              sure1, None)
                    event(max(t1, t0 + 1), "landing", sure1)
                continue
            if ha is None and r["Apogee"].strip().lower() == "inf":
                # GCAT: escape on a parabola or a hyperbola whose excess speed it does not give.
                a, e = -rp / 0.05, 1.05
            elif ha is not None and ha < 0:
                ra = (R + ha) * 1e3
                a, e = (rp + ra) / 2, (ra - rp) / (ra + rp)
                if a >= 0 or e <= 1:
                    a, e = -abs(a) if a else -rp, max(e, 1.0001)
            elif ha is not None:
                ra = max(rp, (R + ha) * 1e3)
                a, e = (rp + ra) / 2, (ra - rp) / (ra + rp)
            else:
                a, e = rp, 0.0
            i, node = inc, 0.0
            if world == "Earth":
                i, node = equatorial_to_ecliptic(inc)
            od = gdate(r["ODate"])
            epoch = od[0] if st == "EN" and od and od[1] else t0
            el = {"a_m": a, "e": e, "i_deg": i, "node_deg": node, "peri_deg": 0.0, "m0_deg": 0.0, "epoch_s": epoch}
            piece(t0, {"kind": "orbit", "around": addr, "elements": el}, sure0,
                  HOW + ("; carried by another craft, whose orbit this is" if borrowed else ""))
            if st == "EN" and od:
                event(od[0] if od[0] > t0 else t0, "flyby", od[1], body=addr, periapsis_m=rp)
            last_body = world
        else:
            continue
        # How the phase ended, when the log stops there.
        if nxt is None and r["_d"]:
            t1, sure1 = r["_d"]
            if t1 <= pieces[-1]["from_s"]:
                t1 = pieces[-1]["from_s"] + 1
            if st in ("L", "AL", "AL IN", "LF"):
                lat, lon, known = landed_at(world, r["Dest"])
                lost = st == "LF" or (lander and lander["Status"][:1] in ("I", "F"))
                if lost:
                    piece(t1, {"kind": "gone", "how": "impacted"}, sure1, None)
                elif world in WORLDS:
                    piece(t1, {"kind": "landed", "body": WORLDS[world][0], "lat_deg": lat, "lon_deg": lon}, sure1,
                          None if known else "GCAT gives no landing site; the sub-point is a stand-in")
                event(t1, "landing", sure1)
            elif st in ("R", "R?", "D", "D?", "AR", "AR IN", "AS", "S"):
                how = "reentered" if world in AIR else "impacted"
                piece(t1, {"kind": "gone", "how": how}, sure1 and "?" not in st, None if "?" not in st else
                      "GCAT is not sure of this ending")
                event(t1, "entry" if how == "reentered" else "landing", sure1)
            elif st in ("F", "AF", "C", "E"):
                piece(t1, {"kind": "gone", "how": "impacted"}, sure1, None)
            elif st in ("OX",):
                piece(t1, {"kind": "gone", "how": "lost_contact"}, sure1, None)
            elif st in ("DK", "GRP") and r["Dest"]:
                piece(t1, {"kind": "docked", "to": "gcat:" + r["Dest"]}, sure1, None)
            ended = True
    if not pieces:
        return None
    for p in pieces:
        c = p["state"].get("course", {})
        if c.get("kind") == "lambert" and p.get("to_s") is not None and p["to_s"] < c["arrive_s"]:
            # The next phase begins before the meeting: keep the destination, not the solved arc.
            p["state"] = {"kind": "course", "course": {"kind": "planned", "course": {"what": "to " + c["to"]}}}
            p["mark"], p["why"] = "estimated", "GCAT's next phase begins before the meeting, so the arc is not solved"
    first = pieces[0]["from_s"]
    events = [e for e in events if e["at_s"] >= first]
    events.sort(key=lambda e: e["at_s"])
    about = ("%s (%s, %s), launched %s. Its phases from GCAT: %s. Orbits keep GCAT's shape and inclination; their "
             "orientation and the place along them are estimated." % (
                 d["name"], d["piece"], d.get("owner", "?"), d["launched"], ", ".join(d.get("destinations", [])) or "-"))
    return {"construct": cid, "name": d["name"], "about": about, "gcat": d["jcat"], "pieces": pieces, "events": events}


os.makedirs(OUT, exist_ok=True)
for old in os.listdir(OUT):
    if old.endswith(".json"):
        os.remove(os.path.join(OUT, old))
n, lam, skipped = 0, 0, []
for d in CAT["departed"]:
    if d["jcat"] not in IDS:
        continue
    log = build(d)
    if not log:
        skipped.append(d["name"])
        continue
    lam += sum(1 for p in log["pieces"] if p["state"]["kind"] == "course" and p["state"]["course"]["kind"] == "lambert")
    with open(os.path.join(OUT, log["construct"] + ".json"), "w", encoding="utf-8") as fh:
        fh.write(json.dumps(log, ensure_ascii=False) + "\n")
    n += 1
print(n, "logs,", lam, "solved courses; no phases for", len(skipped), skipped[:20])


# ---------------------------------------------------------------- RS8 part 2: stations and crewed flights in Earth orbit
# GCAT's satcat gives each object's first phase only (later phases are in its auxiliary catalogues, not read yet):
# launch, shape and inclination of the orbit, and how that phase ended (reentered, deorbited, landed, docked or
# grappled). One log per station core (stations.json) and per human spaceflight vehicle record (catalogue.json).
OUT2 = os.path.join(PACK, "logs/earth-orbit")
SAT = {r["JCAT"]: r for r in table("satcat.tsv")}
# Landing sites GCAT names without coordinates (positions from the sites' published runway and zone centres,
# written from memory and marked estimated).
SITES = {"KSC": (28.615, -80.695), "EAFB": (34.905, -117.884), "EDW": (34.905, -117.884),
         "WSSH": (32.943, -106.419), "Kazakhstan": (47.36, 69.59)}


def earth_log(jcat, name, kind):
    r = SAT.get(jcat)
    if not r:
        return None
    s0 = gdate(r["SDate"]) or gdate(r["LDate"])
    if not s0:
        return None
    hp, ha, inc = num(r["Perigee"]), num(r["Apogee"]), num(r["Inc"]) or 0.0
    cid = slug(r["Name"] or name) or slug(r["Piece"])
    if cid in taken:
        cid = cid + "-" + slug(r["Piece"])
    if cid in taken:
        return None
    taken.add(cid)
    pieces, events = [], []

    def piece(t, state, sure, why):
        p = {"from_s": t, "state": state, "mark": "recorded" if sure and not why else "estimated", "src": "gcat",
             "writer": "pack"}
        if p["mark"] == "estimated":
            p["why"] = why or "GCAT marks this date uncertain or gives it without an hour"
        if pieces:
            pieces[-1]["to_s"] = t
        pieces.append(p)

    if hp is None or ha is None or ha < 0:
        piece(s0[0], {"kind": "course", "course": {"kind": "planned", "course": {"what": "in Earth orbit"}}}, s0[1],
              "GCAT gives no orbit for this phase")
    else:
        R = WORLDS["Earth"][1]
        rp, ra = (R + hp) * 1e3, (R + max(ha, hp)) * 1e3
        i, node = equatorial_to_ecliptic(inc)
        piece(s0[0], {"kind": "orbit", "around": P("earth"), "elements": {
            "a_m": (rp + ra) / 2, "e": (ra - rp) / (ra + rp), "i_deg": i, "node_deg": node, "peri_deg": 0.0,
            "m0_deg": 0.0, "epoch_s": s0[0]}}, s0[1], HOW + "; the orbit is GCAT's at its epoch (%s), held fixed" %
              (r["ODate"] or "-"))
    events.append({"at_s": s0[0], "kind": "launch", "mark": "recorded" if s0[1] else "estimated", "src": "gcat",
                   "writer": "pack", **({} if s0[1] else {"why": "GCAT gives the launch without an hour"})})
    d1 = gdate(r["DDate"])
    st = r["Status"].rstrip("?")
    if d1:
        t1 = max(d1[0], s0[0] + 1)
        if st in ("R", "D", "AR", "S", "F"):
            piece(t1, {"kind": "gone", "how": "reentered"}, d1[1], None)
            events.append({"at_s": t1, "kind": "entry", "mark": "recorded" if d1[1] else "estimated", "src": "gcat",
                           "writer": "pack", **({} if d1[1] else {"why": "GCAT marks this date uncertain"})})
        elif st in ("L", "LF", "AL"):
            c = coords(r["Dest"])
            why = None
            if not c:
                site = next((k for k in SITES if k.lower() in r["Dest"].lower()), None)
                c = SITES.get(site, (0.0, 0.0))
                why = ("GCAT names the site (%s); its coordinates are written from memory" % r["Dest"]) if site else \
                    ("GCAT names the site (%s) without coordinates; the point is a stand-in" % (r["Dest"] or "-"))
            piece(t1, {"kind": "landed", "body": P("earth"), "lat_deg": c[0], "lon_deg": c[1]}, d1[1], why)
            events.append({"at_s": t1, "kind": "landing", "mark": "recorded" if d1[1] else "estimated", "src": "gcat",
                           "writer": "pack", **({} if d1[1] else {"why": "GCAT marks this date uncertain"})})
        elif st in ("DK", "GRP", "ATT") and r["Dest"]:
            piece(t1, {"kind": "docked", "to": "gcat:" + r["Dest"]}, d1[1],
                  "docked to %s; GCAT's later phases (auxiliary catalogues) are not read yet" % r["Dest"])
        elif st in ("E", "C"):
            piece(t1, {"kind": "gone", "how": "impacted"}, d1[1], "broke up in orbit (GCAT status %s)" % st)
    return {"construct": cid, "name": r["Name"] or name, "gcat": jcat, "about": (
        "%s (%s), a %s, launched %s. One orbit from GCAT's satcat (shape and inclination; orientation estimated), "
        "then how that phase ended (%s)." % (r["Name"] or name, r["Piece"], kind, r["LDate"], st or "still in orbit")),
            "pieces": pieces, "events": sorted(events, key=lambda e: e["at_s"])}


os.makedirs(OUT2, exist_ok=True)
for old in os.listdir(OUT2):
    if old.endswith(".json"):
        os.remove(os.path.join(OUT2, old))
STATIONS = json.load(open(os.path.join(PACK, "stations.json"), encoding="utf-8"))["stations"]
seen, n2, ends = set(), 0, {}
for row in [(s_[0], s_[2], "station core") for s_ in STATIONS] + [(c[0], c[2], "crewed vehicle (" + c[4] + ")")
                                                                  for c in CAT["crewed"]]:
    if row[0] in seen or row[0] in IDS:
        continue
    seen.add(row[0])
    log = earth_log(*row)
    if not log:
        continue
    k = log["pieces"][-1]["state"]["kind"]
    ends[k] = ends.get(k, 0) + 1
    with open(os.path.join(OUT2, log["construct"] + ".json"), "w", encoding="utf-8") as fh:
        fh.write(json.dumps(log, ensure_ascii=False) + "\n")
    n2 += 1
print(n2, "Earth-orbit logs, ending", ends)
