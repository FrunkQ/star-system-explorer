#!/usr/bin/env python3
"""RS3: flight logs for the Sol today pack's showcase missions.

Usage: python3 -I scripts/spaceflight/build_logs.py

Writes store-packs/sol-today/logs/<construct>.json, one flight log per construct, in the shape of
crates/sse-nav/src/flightlog.rs (pieces and events, universe seconds on TT/TDB, J2000 = 63115200000
as in tests/fixtures/nav/sol.json). Big burns and flybys only (Alex, 23 Nov 2026); the arcs between are
stable orbits or courses solved from their ends (fudge register: "flight logs: big burns and flybys only").

Every split names constructs that get their own log here, so a stage, booster, tank, lander or probe is
a construct from the moment it leaves: it ends "gone" (burnt up, struck, left), or "landed" when it came
down whole (a reused booster). Each piece and event carries a mark and a source id from
docs/citations/spaceflight.md; an estimate says why. Nothing here is typed from memory unless marked so.
Horizons figures come from scripts/spaceflight/horizons_cassini.json (queried 24 Nov 2026).
"""
import datetime, json, os

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
OUT = os.path.join(ROOT, "store-packs/sol-today/logs")
HZ = json.load(open(os.path.join(ROOT, "scripts/spaceflight/horizons_cassini.json"), encoding="utf-8"))

J2000_S = 63115200000
DAY = 86400
SUN = "u:sol/sys:sun"
P = lambda n: SUN + "/body:" + n
EARTH, MOON, SATURN = P("earth"), P("earth") + "/body:moon", P("saturn")
TITAN = SATURN + "/body:titan"
RE, RM = 6378137.0, 1737400.0
NMI, FT = 1852.0, 0.3048

# TT - UTC by date (TAI - UTC + 32.184 s; before 1972 approximate).
LEAP = [("1969-01-01", 39.4), ("1981-07-01", 52.184), ("1981-01-01", 51.184), ("1997-07-01", 63.184), ("1997-01-01", 62.184),
        ("1999-01-01", 64.184), ("2006-01-01", 65.184), ("2009-01-01", 66.184), ("2012-07-01", 67.184), ("2015-07-01", 68.184),
        ("2017-01-01", 69.184)]


def tt_minus_utc(s):
    best = 39.4
    for d, v in sorted(LEAP):
        if s[:10] >= d:
            best = v
    return best


def jd_s(jd):
    """JD on TT/TDB to universe seconds."""
    return int(round(J2000_S + (jd - 2451545.0) * DAY))


def utc(s):
    """'YYYY-MM-DD HH:MM:SS[.f]' UTC to universe seconds."""
    fmt = "%Y-%m-%d %H:%M:%S.%f" if "." in s else "%Y-%m-%d %H:%M:%S"
    d = datetime.datetime.strptime(s, fmt)
    return int(round(J2000_S + (d - datetime.datetime(2000, 1, 1, 12)).total_seconds() + tt_minus_utc(s)))


def tdb(s):
    fmt = "%Y-%m-%d %H:%M:%S.%f" if "." in s else ("%Y-%m-%d %H:%M:%S" if s.count(":") == 2 else "%Y-%m-%d %H:%M")
    d = datetime.datetime.strptime(s, fmt)
    return int(round(J2000_S + (d - datetime.datetime(2000, 1, 1, 12)).total_seconds()))


def met(t0, s):
    """Mission elapsed 'h:mm:ss[.f]' after t0."""
    h, m_, sec = s.split(":")
    return t0 + int(round(int(h) * 3600 + int(m_) * 60 + float(sec)))


def orbit(around, hp_m, ha_m, i, epoch, r0, **ang):
    a = r0 + (hp_m + ha_m) / 2
    e = (ha_m - hp_m) / (2 * r0 + hp_m + ha_m)
    el = {"a_m": round(a, 1), "e": e, "i_deg": i, "node_deg": 0.0, "peri_deg": 0.0, "m0_deg": 0.0, "epoch_s": epoch}
    el.update(ang)
    return {"kind": "orbit", "around": around, "elements": el}


def hz_orbit(around, el):
    """Horizons osculating elements (km, deg, Tp in JD TDB) as a NAV orbit at the periapsis passage."""
    a_km = el["A"]
    return {"kind": "orbit", "around": around, "elements": {
        "a_m": a_km * 1000.0, "e": el["EC"], "i_deg": el["IN"], "node_deg": el["OM"], "peri_deg": el["W"],
        "m0_deg": 0.0, "epoch_s": jd_s(el["Tp"])}}


def course(kind, **k):
    return {"kind": "course", "course": dict({"kind": kind}, **k)}


def planned(what):
    return course("planned", course=what)


def lambert(to, arrive):
    return course("lambert", to=to, arrive_s=arrive)


class Log:
    def __init__(self, construct, name, about, src):
        self.d = {"construct": construct, "name": name, "about": about, "pieces": [], "events": []}
        self.src = src

    def piece(self, t, state, mark="recorded", why=None, src=None):
        p = {"from_s": t, "state": state, "mark": mark, "src": src or self.src, "writer": "pack"}
        if why:
            p["why"] = why
        ps = self.d["pieces"]
        if ps:
            ps[-1]["to_s"] = t
        ps.append(p)
        return self

    def ev(self, t, kind, mark="recorded", why=None, src=None, **k):
        e = dict({"at_s": t, "kind": kind, "mark": mark, "src": src or self.src, "writer": "pack"}, **k)
        if why:
            e["why"] = why
        self.d["events"].append(e)
        return self

    def end(self, t, how, mark="recorded", why=None, src=None):
        return self.piece(t, {"kind": "gone", "how": how}, mark, why, src)

    def split(self, t, into, mark="recorded", why=None, src=None):
        return self.ev(t, "split", mark, why, src, into=into, keeps_identity=True)


LOGS = []


def keep(*logs):
    for log in logs:
        log.d["events"].sort(key=lambda e: e["at_s"])
        LOGS.append(log.d)


PAD = lambda lat, lon: {"kind": "landed", "body": EARTH, "lat_deg": lat, "lon_deg": lon}

# ---------------------------------------------------------------- Apollo 11 and Saturn V AS-506
# Times: NASA SP-4029 Apollo by the Numbers, Apollo 11 ascent and timeline pages (src nasa-sp-4029).
# Orbits and delta-v: Apollo 11 Mission Report (MSC-00171), written from memory and marked estimated.
SP, MR = "nasa-sp-4029", "nasa-apollo-11-mission-report"
L = utc("1969-07-16 13:32:00")
g = lambda s: met(L, s)
csm = Log("apollo-11", "Apollo 11 (Columbia)",
          "The CSM Columbia, which keeps the mission's name from the pad to the Pacific. The Saturn V's stages, "
          "Eagle and the service module become their own constructs at each split, with their own logs. Event times "
          "from NASA SP-4029; orbit sizes and burn delta-v from the Apollo 11 Mission Report, written from memory and "
          "marked estimated until read from the report. Needs a Moon body (u:sol/sys:sun/body:earth/body:moon), which "
          "the NAV Sol fixture does not hold yet.", SP)
t_eoi, t_tli, t_ext = g("0:11:49.33"), utc("1969-07-16 16:16:16"), utc("1969-07-16 17:49:03")
t_loi1, t_loi2 = utc("1969-07-19 17:21:50"), utc("1969-07-19 21:43:36")
t_und, t_dock, t_jet = utc("1969-07-20 17:44:00"), utc("1969-07-21 21:35:00"), utc("1969-07-21 23:41:31")
t_tei, t_cmsm, t_ei, t_spl = utc("1969-07-22 04:55:42"), utc("1969-07-24 16:21:12"), utc("1969-07-24 16:35:05"), utc("1969-07-24 16:50:35")
csm.piece(L - 60 * DAY, PAD(28.6082, -80.6041), "authored", "60 days on pad 39A before launch")
csm.piece(L, planned("ascent"), "recorded", "ascent to orbit insertion at 11 min 49 s; the ascent itself waits on RS5")
csm.piece(t_eoi, orbit(EARTH, 183.2e3, 186.0e3, 32.52, t_eoi, RE), "estimated",
          "parking orbit 183 by 186 km at 32.5 degrees (mission report, from memory); node and phase not set", MR)
csm.piece(t_tli, lambert(MOON, t_loi1), "recorded", "translunar coast; the one midcourse correction (3 s burn, 26:44:58) is below the big-burns line")
csm.piece(t_loi1, orbit(MOON, 61 * NMI, 169 * NMI, 1.25, t_loi1, RM), "estimated", "after LOI-1, 61 by 169 nautical miles (mission report, from memory)", MR)
csm.piece(t_loi2, orbit(MOON, 54 * NMI, 66 * NMI, 1.25, t_loi2, RM), "estimated", "after LOI-2, 54 by 66 nautical miles, held stable to TEI (mission report, from memory)", MR)
csm.piece(t_tei, lambert(EARTH, t_ei))
csm.piece(t_ei, planned("entry"), "recorded", "entry flown by sse-nav entry.rs")
csm.piece(t_spl, {"kind": "landed", "body": EARTH, "lat_deg": 13.32, "lon_deg": -169.15}, "estimated",
          "splashdown 13 deg 19 min N, 169 deg 9 min W (mission report, from memory); recovered by USS Hornet", MR)
csm.ev(L, "launch")
csm.split(g("0:02:42.30"), ["as-506-s-ic"], why="S-IC/S-II separation (commanded time)")
csm.split(g("0:09:09.00"), ["as-506-s-ii"], why="S-II/S-IVB separation (commanded time)")
csm.ev(t_tli, "burn", "estimated", "trans-lunar injection, S-IVB second burn 16:16:16 to 16:22:03; about 3.18 km/s from the parking orbit (from memory)",
       MR, dv_m_s=3180.0, engine="j-2")
csm.split(t_ext, ["as-506-s-ivb"], why="CSM and the docked LM drawn out of the S-IVB (ejection); Eagle rides docked to Columbia")
csm.ev(t_loi1, "burn", "estimated", "lunar orbit insertion 1, 357.5 s, 2,917 ft/s (delta-v from memory)", MR, dv_m_s=889.0, engine="aj10-137")
csm.ev(t_loi2, "burn", "estimated", "lunar orbit insertion 2, 16.9 s, 159 ft/s (delta-v from memory)", MR, dv_m_s=48.0, engine="aj10-137")
csm.split(t_und, ["apollo-11-eagle"], why="Eagle undocks")
csm.ev(t_dock, "join", why="Eagle's ascent stage docks with Columbia", **{"with": "apollo-11-eagle"})
csm.split(t_jet, ["apollo-11-eagle"], why="the ascent stage is jettisoned and left in lunar orbit")
csm.ev(t_tei, "burn", "estimated", "trans-Earth injection, 3,283 ft/s (delta-v from memory)", MR, dv_m_s=1001.0, engine="aj10-137")
csm.split(t_cmsm, ["apollo-11-service-module"], why="command module and service module separate")
csm.ev(t_ei, "entry", why="entry interface")
csm.ev(t_spl, "landing", why="splashdown, Pacific Ocean")

sic = Log("as-506-s-ic", "Saturn V S-IC (AS-506)", "Apollo 11's first stage, from separation to the Atlantic.", SP)
sic.piece(g("0:02:42.30"), planned("ballistic fall"), "recorded",
          "separated at 36.0 nmi altitude, 7,883 ft/s Earth-fixed, 28.70 N 79.68 W; falls on a ballistic arc")
sic.end(g("0:09:03.7"), "impacted", "recorded", "theoretical impact time in the Atlantic (SP-4029); place not given there")
sii = Log("as-506-s-ii", "Saturn V S-II (AS-506)", "Apollo 11's second stage, from separation to the Atlantic.", SP)
sii.piece(g("0:09:09.00"), planned("ballistic fall"), "recorded",
          "separated at 101.2 nmi altitude, 21,377 ft/s Earth-fixed, 31.72 N 64.15 W; short of orbit, falls back")
sii.end(g("0:20:13.7"), "impacted", "recorded", "theoretical impact time in the Atlantic (SP-4029); place not given there")
sivb = Log("as-506-s-ivb", "Saturn V S-IVB (AS-506)",
           "Apollo 11's third stage after the spacecraft left it: the slingshot past the Moon into orbit about the Sun.", SP)
t_moon = g("78:42:00")
sivb.piece(t_ext, lambert(MOON, t_moon), "recorded", "after the evasive burn (4:40:01) and slingshot burns (LOX dump, APS 5:37:47); closest to the Moon at 78:42")
sivb.piece(t_moon, planned("heliocentric orbit"), "estimated", "the slingshot leaves it in orbit about the Sun; its elements are not on record here yet")
eagle = Log("apollo-11-eagle", "Apollo 11 Eagle (LM-5)",
            "The lunar module from undocking: descent, Tranquility Base, ascent, docking and its last orbit. "
            "The descent stage stays on the Moon as its own construct.", SP)
t_doi, t_pdi, t_land = utc("1969-07-20 19:08:14"), utc("1969-07-20 20:05:05"), utc("1969-07-20 20:17:40")
t_lift, t_ins = utc("1969-07-21 17:54:01"), utc("1969-07-21 18:01:16")
TRANQ = {"kind": "landed", "body": MOON, "lat_deg": 0.67416, "lon_deg": 23.47314}
eagle.piece(t_und, orbit(MOON, 54 * NMI, 66 * NMI, 1.25, t_und, RM), "estimated", "Columbia's orbit after LOI-2")
eagle.piece(t_doi, orbit(MOON, 8.5 * NMI, 57.2 * NMI, 1.25, t_doi, RM), "estimated", "descent orbit about 8.5 by 57 nautical miles (mission report, from memory)", MR)
eagle.piece(t_pdi, planned("powered descent"), "recorded", "powered descent, 12 min 35 s")
eagle.piece(t_land, TRANQ, "recorded", "Tranquility Base (NSSDCA 1969-059C)", "nssdca-craft")
eagle.piece(t_lift, planned("ascent"), "recorded", "ascent stage lifts off; the descent stage stays")
eagle.piece(t_ins, orbit(MOON, 9.4 * NMI, 47.3 * NMI, 1.25, t_ins, RM), "estimated", "ascent orbit about 9.4 by 47 nautical miles (mission report, from memory)", MR)
eagle.piece(t_dock, {"kind": "docked", "to": "apollo-11"}, "recorded")
eagle.piece(t_jet, orbit(MOON, 54 * NMI, 66 * NMI, 1.25, t_jet, RM), "estimated", "left in lunar orbit; it later fell to the Moon, when and where unknown")
eagle.ev(t_doi, "burn", "estimated", "descent orbit insertion, 30 s, about 76 ft/s (from memory)", MR, dv_m_s=23.2, engine="lm-descent-engine")
eagle.ev(t_land, "landing", why="20:17:39.9, engine cutoff 20:17:41")
eagle.split(t_lift, ["apollo-11-eagle-descent-stage"], why="the ascent stage leaves the descent stage on the surface")
eagle.ev(t_dock, "join", why="docks with Columbia", **{"with": "apollo-11"})
eagle.ev(t_lift, "launch", why="lunar liftoff, 17:54:00.79")
desc = Log("apollo-11-eagle-descent-stage", "Eagle's descent stage", "Left at Tranquility Base on 21 July 1969, where it stands.", "nssdca-craft")
desc.piece(t_lift, TRANQ, "recorded")
sm = Log("apollo-11-service-module", "Apollo 11 service module", "Cast off before entry; it burnt up.", SP)
sm.piece(t_cmsm, lambert(EARTH, t_ei), "estimated", "follows the command module's course to entry")
sm.end(t_ei, "reentered", "estimated", "burnt up on entry; the time is the command module's entry interface")
keep(csm, sic, sii, sivb, eagle, desc, sm)

# ---------------------------------------------------------------- Falcon 9 flight 20: the first booster landing
F9 = "spaceflight101-og2"  # secondary; launch and landing times confirmed by the 45th Space Wing release
L9 = utc("2015-12-22 01:29:00")
f = lambda s: met(L9, "0:" + s)
s2 = Log("falcon-9-flight-20", "Falcon 9 flight 20 (ORBCOMM OG2 M2)",
         "The second stage and its eleven ORBCOMM OG2 satellites, 22 Dec 2015 UTC, from SLC-40. The first stage, "
         "B1019, splits off at T+2:25 and becomes the first orbital booster to land, at Landing Zone 1. Times from "
         "Spaceflight101's reconstruction (approximate) and the 45th Space Wing release.", F9)
s2.piece(L9 - DAY, PAD(28.5618, -80.5772), "estimated", "on SLC-40 the day before; pad position from memory")
s2.piece(L9, planned("ascent"), "recorded", "first stage burn 143 s, second stage 412 s, shutdown at T+9:26")
t_seco = f("09:26")
s2.piece(t_seco, orbit(EARTH, 614e3, 656e3, 47.0, t_seco, RE), "recorded", "614 by 656 km at 47.00 degrees, as achieved")
s2.ev(L9, "launch", src="usaf-45sw-og2")
s2.split(f("02:25"), ["falcon-9-b1019"], why="stage separation")
deploys = [("14:26", [1, 2]), ("15:38", [3, 4]), ("16:16", [5, 6]), ("17:23", [7, 8]), ("18:07", [9, 10]), ("19:05", [11])]
sats = []
for t, ns in deploys:
    ids = [f"orbcomm-og2-m2-{n:02d}" for n in ns]
    s2.split(f(t), ids, why="satellites released")
    for n, i in zip(ns, ids):
        s = Log(i, f"ORBCOMM OG2 (flight 20, satellite {n})", "Released from Falcon 9 flight 20 into its working orbit.", F9)
        s.piece(f(t), orbit(EARTH, 614e3, 656e3, 47.0, f(t), RE), "recorded", "the second stage's orbit at release")
        sats.append(s)
b = Log("falcon-9-b1019", "Falcon 9 first stage B1019",
        "Flight 20's booster: boost-back, entry and landing burns, then down on Landing Zone 1. It never flew again.", F9)
b.piece(f("02:25"), planned("return to launch site"), "recorded",
        "MECO at T+2:23, 75 km, 1.67 km/s; boost-back from T+3:47, entry burn T+8:01 for 20 s, landing burn T+9:11 for 33 s. "
        "Delta-v of the three burns is not published; RS5's ascent and return physics flies them")
t_lz = f("09:44")
b.piece(t_lz, PAD(28.4858, -80.5444), "estimated", "Landing Zone 1 (formerly LC-13), position from memory; touchdown T+9:44")
b.ev(f("08:01"), "entry", why="entry burn start")
b.ev(t_lz, "landing", why="first landing of an orbital booster, 20:38 EST on 21 Dec", src="usaf-45sw-og2")
keep(s2, b, *sats)

# ---------------------------------------------------------------- STS-1 Columbia
# Times are the press kit's pre-flight plan (src nasa-sts-1-press-kit) and so are marked estimated; the flown
# mission lasted 54 h 20 min 53 s (NSSDCA). Launch time from memory, to verify.
PK = "nasa-sts-1-press-kit"
LS = utc("1981-04-12 12:00:04")
m1 = lambda s: met(LS, s)
sts = Log("sts-1-columbia", "STS-1 Columbia (OV-102)",
          "The first Space Shuttle flight, 12 to 14 April 1981, from pad 39A to Rogers Dry Lake. The two solid boosters "
          "and the external tank become their own constructs when they separate. Burn times and delta-v are the press kit's "
          "pre-flight plan; the launch time is from memory and the landing is NSSDCA's flown duration.", PK)
orb_s = lambda hp, ha, t: orbit(EARTH, hp * NMI, ha * NMI, 40.3, t, RE)
plan = "press kit plan; flown within seconds"
sts.piece(LS - 30 * DAY, PAD(28.6082, -80.6041), "authored", "on pad 39A before launch")
sts.piece(LS, planned("ascent"), "estimated", "SRB separation T+2:12 at 49.7 km, MECO T+8:32 at 118.5 km and 26,715 km/h")
t_meco, t_o1, t_o2, t_o3, t_o4, t_deo = m1("0:08:32"), m1("0:10:32"), m1("0:44:00"), m1("6:20:41"), m1("7:05:31"), m1("53:28:00")
t_ei9, t_land9 = m1("53:56:02"), m1("54:20:53")
sts.piece(t_meco, orb_s(13, 80, t_meco), "estimated", "after MECO, 13 by 80 nautical miles: the tank's orbit, which dips into the air")
sts.piece(t_o1, orb_s(57, 130, t_o1), "estimated", "after OMS-1, 57 by 130 nmi; " + plan)
sts.piece(t_o2, orb_s(130, 130, t_o2), "estimated", "after OMS-2, 130 nmi (241 km) circular; " + plan)
sts.piece(t_o3, orb_s(134, 150, t_o3), "estimated", "after OMS-3, 134 by 150 nmi; " + plan)
sts.piece(t_o4, orb_s(150, 150, t_o4), "estimated", "after OMS-4, 150 nmi (278 km) circular; " + plan)
sts.piece(t_deo, orb_s(2, 149, t_deo), "estimated", "after the deorbit burn, 2 by 149 nmi; " + plan)
sts.piece(t_ei9, planned("entry"), "estimated", "entry interface at 53:56:02 (plan); glides to Edwards")
sts.piece(t_land9, {"kind": "landed", "body": EARTH, "lat_deg": 34.9, "lon_deg": -117.9}, "estimated",
          "Rogers Dry Lake, Edwards AFB; flown duration 54:20:53 (NSSDCA); position from memory", "nssdca-craft")
sts.ev(LS, "launch")
sts.split(m1("0:02:12"), ["sts-1-srb-left", "sts-1-srb-right"], "estimated", "solid boosters separate at 49.7 km (plan)")
sts.split(m1("0:08:51"), ["sts-1-external-tank"], "estimated", "external tank separates (plan)")
for t, dv, n in ((t_o1, 165, "OMS-1"), (t_o2, 137, "OMS-2"), (t_o3, 36.5, "OMS-3"), (t_o4, 37.5, "OMS-4"), (t_deo, 299.0, "deorbit")):
    sts.ev(t, "burn", "estimated", f"{n}, {dv} ft/s in the press kit's plan", dv_m_s=round(dv * FT, 1), engine="oms-aj10-190")
sts.ev(t_ei9, "entry", "estimated", "entry interface (plan)")
sts.ev(t_land9, "landing", why="touchdown on Rogers Dry Lake", src="nssdca-craft")
srbs = []
for side in ("left", "right"):
    s = Log(f"sts-1-srb-{side}", f"STS-1 solid rocket booster ({side})",
            "Separated at T+2:12, came down under parachutes in the Atlantic and was recovered for reuse.", PK)
    s.piece(m1("0:02:12"), planned("coast and parachute descent"), "estimated", "separated at 49.7 km (plan)")
    s.piece(m1("0:07:00"), {"kind": "landed", "body": EARTH, "lat_deg": 30.2, "lon_deg": -78.6}, "estimated",
            "splashdown about 140 nmi downrange; time and place estimated, recovered by ship")
    s.ev(m1("0:07:00"), "landing", "estimated", "splashdown under three parachutes")
    srbs.append(s)
et = Log("sts-1-external-tank", "STS-1 external tank",
         "Separated after MECO on an orbit that dips into the air; it broke up on entry over the Indian Ocean.", PK)
et.piece(m1("0:08:51"), orb_s(13, 80, t_meco), "estimated", "MECO orbit, 13 by 80 nautical miles")
et.end(m1("0:55:00"), "reentered", "estimated", "about half an orbit after separation, as a 13 nmi perigee allows; time estimated")
keep(sts, *srbs, et)

# ---------------------------------------------------------------- Cassini-Huygens
H = "jpl-horizons"
LC = utc("1997-10-15 08:43:00")
cas = Log("cassini", "Cassini",
          "Cassini from Cape Canaveral to Saturn: four flybys, the deep space manoeuvre and the Saturn orbit insertion, "
          "then thirteen years of a tour reshaped by Titan flybys, kept here as one osculating orbit per year (the "
          "big-burns fudge), and the plunge into Saturn on 15 Sep 2017. Huygens splits off on 25 Dec 2004. Flyby times "
          "and distances, the burns and the orbits are read from JPL Horizons (scripts/spaceflight/horizons_cassini.json).", H)
B = HZ["burns"]
fly = HZ["flybys"]
t_dep = LC + 1800
cas.piece(LC - 60 * DAY, PAD(28.562, -80.577), "authored", "on SLC-40 before launch")
cas.piece(LC, planned("ascent"), "estimated", "Titan IVB-Centaur ascent; RS5 flies it")
cas.piece(LC + 600, orbit(EARTH, 185e3, 185e3, 28.5, LC + 600, RE), "estimated", "a 185 km parking orbit stands in for the Centaur's coast before departure")
t_v1 = jd_s(fly["venus-1"]["Tp"])
cas.piece(t_dep, lambert(P("venus"), t_v1), "recorded", "to the first Venus flyby")
t_dsm = tdb("1998-12-03 06:45")
cas.piece(t_v1, hz_orbit(SUN, HZ["cas_helio_after_venus1"]), "recorded", "about the Sun after Venus 1 (Horizons, 1998-06-15)")
t_v2 = jd_s(fly["venus-2"]["Tp"])
t_e = jd_s(fly["earth"]["Tp"])
t_j = jd_s(fly["jupiter"]["Tp"])
t_soi = jd_s(B["soi"]["post_Tp"])
cas.piece(t_dsm, lambert(P("venus"), t_v2), "recorded", "after the deep space manoeuvre, to the second Venus flyby")
cas.piece(t_v2, lambert(EARTH, t_e), "recorded")
cas.piece(t_e, lambert(P("jupiter"), t_j), "recorded")
cas.piece(t_j, lambert(SATURN, t_soi), "recorded")
cas.piece(t_soi, hz_orbit(SATURN, HZ["cas_after_soi"]), "recorded", "the capture orbit (Horizons, 2004-07-02)")
t_hsep = tdb("2004-12-25 02:10")
for y in range(2005, 2018):
    k = f"cas_{y}"
    t = tdb(f"{y}-01-01 00:00") if y < 2017 else tdb("2017-09-01 00:00")
    cas.piece(t, hz_orbit(SATURN, HZ[k]), "recorded", f"osculating orbit on {HZ[k]['date'].strip()[5:16]} (Horizons); Titan flybys between are below the big-burns line")
t_end = tdb("2017-09-15 11:58")
cas.end(t_end, "impacted", "recorded", "the Grand Finale: into Saturn's atmosphere; Horizons' trajectory ends here")
cas.ev(LC, "launch", src="jpl-cassini-press-kit")
cas.ev(t_dep, "burn", "estimated", B["earth_departure"]["how"], dv_m_s=float(B["earth_departure"]["dv_m_s"]), engine="rl10a-4")
cas.split(t_dep, ["cassini-centaur"], "estimated", "the Centaur upper stage separates after the departure burn")
for k, body in (("venus-1", "venus"), ("venus-2", "venus"), ("earth", "earth"), ("jupiter", "jupiter")):
    cas.ev(jd_s(fly[k]["Tp"]), "flyby", body=P(body), periapsis_m=round(fly[k]["QR"] * 1000.0))
cas.ev(t_dsm, "burn", "estimated", "deep space manoeuvre: " + B["dsm"]["how"] + ", between " + B["dsm"]["pre"] + " and " + B["dsm"]["post"],
       dv_m_s=B["dsm"]["dv_m_s"], engine="cassini-main-445n")
cas.ev(t_soi, "burn", "estimated", "Saturn orbit insertion (96 min): " + B["soi"]["how"], dv_m_s=B["soi"]["dv_m_s"], engine="cassini-main-445n")
cas.split(t_hsep, ["huygens"], "estimated", "Huygens released; " + HZ["huygens_separation"]["how"] + " between 02:00 and 02:20")
hu = Log("huygens", "Huygens", "ESA's Titan probe: 20 days' coast from Cassini to Titan, then the descent and landing.", H)
hu.piece(t_hsep, hz_orbit(SATURN, HZ["huygens_saturn"]), "recorded", "about Saturn on its way to Titan (Horizons, 2004-12-26)")
t_hl = tdb("2005-01-14 12:06:59")
hu.piece(t_hl, {"kind": "landed", "body": TITAN, "lat_deg": -10.25, "lon_deg": 167.68}, "estimated",
         "Horizons' predicted descent ends here; the flown landing was earlier that day. Site about 10.2 S, 192.3 W, from memory")
hu.ev(t_hl, "landing", "estimated", "on Titan; Huygens kept sending for about 70 minutes after")
cen = Log("cassini-centaur", "Cassini's Centaur upper stage", "Left in orbit about the Sun after the Earth departure burn.", H)
cen.piece(t_dep, planned("heliocentric orbit"), "estimated", "follows Cassini's departure; its own elements are not read yet")
keep(cas, hu, cen)

# ---------------------------------------------------------------- Voyager 2 (the travel thread's fixture) and its stages
voy = json.load(open(os.path.join(ROOT, "tests/fixtures/flightlog/voyager-2.json"), encoding="utf-8"))
t_v = [e for e in voy["events"] if e["kind"] == "split"][0]["at_s"]
vc = Log("voyager-2-centaur", "Voyager 2's Centaur upper stage", "Left about the Sun after the departure burn.", "nssdca-voyager-2")
vc.piece(t_v, planned("heliocentric orbit"), "estimated", "follows Voyager's departure; its own elements are not read yet")
vp = Log("voyager-2-propulsion-module", "Voyager 2's propulsion module", "The solid kick motor and its frame, cast off after the departure burn.", "nssdca-voyager-2")
vp.piece(t_v, planned("heliocentric orbit"), "estimated", "follows Voyager's departure; its own elements are not read yet")
for p in voy["pieces"]:
    if p["mark"] == "estimated" and "why" not in p:
        p["why"] = "a 185 km parking orbit stands in for the Centaur's coast before departure"
for e in voy["events"]:
    if e["mark"] == "estimated" and "why" not in e:
        e["why"] = "the Centaur and the propulsion module are cast off after the departure burn; the hour is estimated"
LOGS.append(voy)
keep(vc, vp)

os.makedirs(OUT, exist_ok=True)
for old in os.listdir(OUT):
    if old.endswith(".json"):
        os.remove(os.path.join(OUT, old))
for d in LOGS:
    with open(os.path.join(OUT, d["construct"] + ".json"), "w", encoding="utf-8") as fh:
        fh.write(json.dumps(d, indent=1, ensure_ascii=False) + "\n")
print(len(LOGS), "logs")
