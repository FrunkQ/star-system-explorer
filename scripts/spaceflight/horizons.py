#!/usr/bin/env python3
"""JPL Horizons helpers for track RS: how the horizons_*.json files in this folder were gathered.

Usage (from the repo root):
  python3 -I scripts/spaceflight/horizons.py elements <command> <centre> <start> [stop] [step]
  python3 -I scripts/spaceflight/horizons.py vector <command> <centre> <julian date TDB>
  python3 -I scripts/spaceflight/horizons.py lookup <name>

<command> is a Horizons id: spacecraft are negative (Cassini -82, Huygens -150, Juno -61, Dawn -203, MSL -76,
JWST -170, ISS -125544, Voyager 2 -32, New Horizons -98, MAVEN -202, Mars Odyssey -53, Rosetta -226,
MESSENGER -236, MRO -74); find others with `lookup` (matches by name only, not by COSPAR id).
<centre> is a body code: 10 Sun, 199 Mercury, 299 Venus, 399 Earth, 301 Moon, 499 Mars, 599 Jupiter, 699 Saturn,
799 Uranus, 899 Neptune; small bodies as 2000004 (Vesta), 2000001 (Ceres). Horizons will not give elements about
Vesta or Ceres ("Required masses not defined"): take vectors there and turn them into elements with
`elements_from_state` below, with GM from the master data sheet.

Everything is ecliptic J2000, km and km/s, times TDB. Universe seconds are J2000 = 63115200000 plus
(JD - 2451545.0) * 86400. The API is https://ssd.jpl.nasa.gov/api/horizons.api (free; credit "JPL Horizons").

Burn delta-v from Horizons: take osculating elements a little before and after the burn and compare the velocities
they give at the burn's midpoint. Near a giant planet's periapsis its oblateness swings the elements, so use the
speed at periapsis worked out from elements taken far out instead (how Juno's 516 m/s and Cassini's 576 m/s were got).
"""
import json, math, re, sys, urllib.parse, urllib.request

API = "https://ssd.jpl.nasa.gov/api/horizons.api"
COLS = ["JD", "date", "EC", "QR", "IN", "OM", "W", "Tp", "N", "MA", "TA", "A"]


def _ask(params):
    p = {k: "'%s'" % v for k, v in params.items()}
    p["format"] = "text"
    text = urllib.request.urlopen(API + "?" + urllib.parse.urlencode(p), timeout=120).read().decode()
    m = re.search(r"\$\$SOE(.*?)\$\$EOE", text, re.S)
    if not m:
        raise SystemExit(text[-1500:])
    return [line for line in m.group(1).strip().split("\n") if line.strip()]


def elements(cmd, centre, start, stop=None, step="1"):
    """Osculating elements rows (dicts with COLS; distances km, angles degrees, Tp as JD)."""
    if stop is None:
        stop = start + " 00:01" if len(start) == 10 else start[:-2] + "01"
    rows = _ask({"COMMAND": cmd, "EPHEM_TYPE": "ELEMENTS", "CENTER": "500@" + str(centre), "START_TIME": start,
                 "STOP_TIME": stop, "STEP_SIZE": step, "OUT_UNITS": "KM-S", "REF_PLANE": "ECLIPTIC",
                 "CSV_FORMAT": "YES"})
    out = []
    for r in rows:
        f = [x.strip() for x in r.split(",")]
        out.append({k: (f[j] if k == "date" else float(f[j])) for j, k in enumerate(COLS)})
    return out


def vector(cmd, centre, jd):
    """Position (km) and velocity (km/s) at one Julian date (TDB)."""
    r = _ask({"COMMAND": cmd, "EPHEM_TYPE": "VECTORS", "CENTER": "500@" + str(centre), "TLIST": str(jd),
              "TLIST_TYPE": "JD", "TIME_TYPE": "TDB", "OUT_UNITS": "KM-S", "REF_PLANE": "ECLIPTIC",
              "CSV_FORMAT": "YES", "VEC_TABLE": "2"})[0].split(",")
    v = [float(x) for x in r[2:8]]
    return v[:3], v[3:]


def lookup(name):
    u = "https://ssd.jpl.nasa.gov/api/horizons_lookup.api?" + urllib.parse.urlencode({"sstr": name, "group": "sct"})
    return json.loads(urllib.request.urlopen(u, timeout=60).read().decode())


def state(el, mu, jd):
    """Position and velocity from a Horizons elements row (EC, QR, IN, OM, W, Tp) at `jd`; mu in km^3/s^2."""
    e, q = el["EC"], el["QR"]
    i, node, w = (math.radians(el[k]) for k in ("IN", "OM", "W"))
    dt = (jd - el["Tp"]) * 86400
    if e < 1:
        a = q / (1 - e)
        M = math.sqrt(mu / a ** 3) * dt
        E = M
        for _ in range(100):
            E -= (E - e * math.sin(E) - M) / (1 - e * math.cos(E))
        x, y = a * (math.cos(E) - e), a * math.sqrt(1 - e * e) * math.sin(E)
        r = a * (1 - e * math.cos(E))
        vx, vy = -math.sqrt(mu * a) / r * math.sin(E), math.sqrt(mu * a * (1 - e * e)) / r * math.cos(E)
    else:
        a = q / (e - 1)
        M = math.sqrt(mu / a ** 3) * dt
        H = math.asinh(M / e) if M else 0.0
        for _ in range(200):
            H -= (e * math.sinh(H) - H - M) / (e * math.cosh(H) - 1)
        x, y = a * (e - math.cosh(H)), a * math.sqrt(e * e - 1) * math.sinh(H)
        r = a * (e * math.cosh(H) - 1)
        vx, vy = -math.sqrt(mu * a) / r * math.sinh(H), math.sqrt(mu * a * (e * e - 1)) / r * math.cosh(H)
    cO, sO, cw, sw, ci, si = math.cos(node), math.sin(node), math.cos(w), math.sin(w), math.cos(i), math.sin(i)

    def rot(px, py):
        return [px * (cO * cw - sO * sw * ci) - py * (cO * sw + sO * cw * ci),
                px * (sO * cw + cO * sw * ci) - py * (sO * sw - cO * cw * ci),
                px * sw * si + py * cw * si]

    return rot(x, y), rot(vx, vy)


def elements_from_state(r, v, mu, jd):
    """Horizons-style elements (EC, QR, IN, OM, W, Tp, A) from a position and velocity at `jd`."""
    dot = lambda a, b: sum(x * y for x, y in zip(a, b))
    nrm = lambda a: math.sqrt(dot(a, a))
    h = [r[1] * v[2] - r[2] * v[1], r[2] * v[0] - r[0] * v[2], r[0] * v[1] - r[1] * v[0]]
    n = [-h[1], h[0], 0.0]
    rr, vv = nrm(r), nrm(v)
    ev = [(vv * vv - mu / rr) * r[k] / mu - dot(r, v) * v[k] / mu for k in range(3)]
    e = nrm(ev)
    a = 1 / (2 / rr - vv * vv / mu)
    inc = math.degrees(math.acos(h[2] / nrm(h)))
    node = math.degrees(math.atan2(n[1], n[0])) % 360
    w = math.degrees(math.acos(max(-1, min(1, dot(n, ev) / (nrm(n) * e)))))
    w = w if ev[2] >= 0 else 360 - w
    nu = math.acos(max(-1, min(1, dot(ev, r) / (e * rr))))
    nu = nu if dot(r, v) >= 0 else 2 * math.pi - nu
    E = 2 * math.atan(math.sqrt((1 - e) / (1 + e)) * math.tan(nu / 2))
    M = E - e * math.sin(E)
    tp = jd - M / math.sqrt(mu / a ** 3) / 86400
    return {"EC": e, "QR": a * (1 - e), "IN": inc, "OM": node, "W": w, "Tp": tp, "A": a}


if __name__ == "__main__":
    what, args = sys.argv[1], sys.argv[2:]
    if what == "elements":
        for row in elements(*args):
            print(json.dumps(row))
    elif what == "vector":
        print(json.dumps(vector(args[0], args[1], float(args[2]))))
    elif what == "lookup":
        print(json.dumps(lookup(args[0]), indent=1))
    else:
        raise SystemExit(__doc__)
