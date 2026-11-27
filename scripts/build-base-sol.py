#!/usr/bin/env python3
"""The catalogued Solar System: packs/base-sol/pack.json.

Sol as measured, for the start sheet's "Open the Solar System" door and any
reader that needs real Sol bodies. These are body facts, not the master data
sheet's reference objects: the sheet's Sun is a yardstick for units, this Sun
is a star in a system. A check proves the two agree on the Sun's nominal
values, so neither can drift.

Every value is read from the source beside it. NSSDCA fact sheet numbers are
copied as printed (22 Nov 2026) and turned into SI here. Values worked out
from others say how. Field names are the kernel's (body.*, orbit.*, star.*).
Python standard library only; deterministic.

  scripts/build-base-sol.py          write the pack
  scripts/build-base-sol.py --check  exit 1 if the pack is stale or disagrees
                                     with packs/base-reference
"""
import json
import math
import os
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(ROOT, "packs", "base-sol", "pack.json")
SHEET = os.path.join(ROOT, "packs", "base-reference", "pack.json")

NS = "https://nssdc.gsfc.nasa.gov/planetary/factsheet/%sfact.html"
HOUR = 3600.0

SOURCES = [
    {"id": "nssdca-%s" % n, "text": "Williams, D. R., NASA NSSDCA %s Fact Sheet (read 22 Nov 2026)" % t,
     "status": "verified", "url": NS % n}
    for n, t in [("sun", "Sun"), ("mercury", "Mercury"), ("venus", "Venus"), ("earth", "Earth"), ("moon", "Moon"),
                 ("mars", "Mars"), ("jupiter", "Jupiter"), ("saturn", "Saturn"), ("uranus", "Uranus"),
                 ("neptune", "Neptune"), ("pluto", "Pluto"), ("asteroid", "Asteroid")]
] + [
    {"id": "nssdca-%s" % n, "text": "Williams, D. R., NASA NSSDCA %s (read 24 Nov 2026)" % t,
     "status": "verified", "url": "https://nssdc.gsfc.nasa.gov/planetary/factsheet/%s.html" % page}
    for n, t, page in [("jovian-sats", "Jovian Satellite Fact Sheet", "joviansatfact"),
                       ("saturnian-sats", "Saturnian Satellite Fact Sheet", "saturniansatfact"),
                       ("neptunian-sats", "Neptunian Satellite Fact Sheet", "neptuniansatfact"),
                       ("uranian-sats", "Uranian Satellite Fact Sheet", "uraniansatfact")]
] + [
    {"id": "iau2015b3", "text": "Prša, A. et al. 2016, AJ 152, 41: Nominal values for selected solar and planetary quantities, IAU 2015 Resolution B3",
     "status": "verified", "doi": "10.3847/0004-6256/152/2/41"},
    {"id": "hipparcos1997", "text": "ESA 1997, The Hipparcos and Tycho Catalogues, ESA SP-1200, vol. 1, sec. 1.5.3: the ICRS to galactic matrix A_G",
     "status": "verified", "url": "https://www.cosmos.esa.int/documents/532822/552851/vol1_all.pdf"},
    {"id": "hilton2006", "text": "Hilton, J. L. et al. 2006, Celest. Mech. Dyn. Astron. 94, 351: Report of the IAU Division I Working Group on Precession and the Ecliptic (obliquity at J2000, 84381.406 arcsec)",
     "status": "verified", "doi": "10.1007/s10569-006-0001-2"},
    {"id": "charette-smith2010", "text": "Charette, M. A. and Smith, W. H. F. 2010, Oceanography 23(2), 112: The volume of Earth's ocean (1.335e9 km3, the size of one Earth ocean as the kernel counts it; Earth holds 1 by definition)",
     "status": "unverified", "doi": "10.5670/oceanog.2010.51"},
]

# ICRS to galactic (Hipparcos A_G, rows are galactic x, y, z) and the J2000
# obliquity: the Sol fixture's elements are J2000 ecliptic, so its frame in
# the galaxy is the ecliptic pole and the equinox turned into l and b.
A_G = [(-0.0548755604, -0.8734370902, -0.4838350155),
       (+0.4941094279, -0.4448296300, +0.7469822445),
       (-0.8676661490, -0.1980763734, +0.4559837762)]
OBLIQUITY_ARCSEC = 84381.406


def galactic(v):
    g = [sum(r[i] * v[i] for i in range(3)) for r in A_G]
    l = math.degrees(math.atan2(g[1], g[0])) % 360.0
    return round(l, 4), round(math.degrees(math.asin(g[2])), 4)


def sun_frame():
    e = math.radians(OBLIQUITY_ARCSEC / 3600.0)
    pl, pb = galactic((0.0, -math.sin(e), math.cos(e)))
    zl, zb = galactic((1.0, 0.0, 0.0))
    note = "J2000 ecliptic %s in galactic coordinates, from the obliquity (hilton2006) and A_G"
    return {
        "system.frame.pole_l_deg": val(pl, "deg", "hipparcos1997", note=note % "north pole"),
        "system.frame.pole_b_deg": val(pb, "deg", "hipparcos1997", note=note % "north pole"),
        "system.frame.zero_l_deg": val(zl, "deg", "hipparcos1997", note=note % "equinox (longitude zero)"),
        "system.frame.zero_b_deg": val(zb, "deg", "hipparcos1997", note=note % "equinox (longitude zero)"),
    }

# name, id, kind, host, mass (1e24 kg), mean radius (km), a (1e6 km), e,
# sidereal rotation (h, as printed: negative = retrograde), Bond albedo,
# geometric albedo, obliquity to orbit (deg). From the NSSDCA sheet named.
PLANETS = [
    ("Mercury", "mercury", "planet", "sun", 0.33010, 2439.7, 57.909, 0.2056, 1407.6, 0.068, 0.142, 0.034),
    ("Venus", "venus", "planet", "sun", 4.8673, 6051.8, 108.210, 0.0068, -5832.6, 0.77, 0.689, 177.36),
    ("Earth", "earth", "planet", "sun", 5.9722, 6371.000, 149.598, 0.0167, 23.9345, 0.294, 0.434, 23.44),
    ("Moon", "moon", "moon", "earth", 0.07346, 1737.4, 0.3844, 0.0549, 655.720, 0.11, 0.12, 6.68),
    ("Mars", "mars", "planet", "sun", 0.64169, 3389.5, 227.956, 0.0935, 24.6229, 0.250, 0.170, 25.19),
    ("Jupiter", "jupiter", "planet", "sun", 1898.13, 69911, 778.479, 0.0487, 9.9250, 0.343, 0.538, 3.13),
    ("Saturn", "saturn", "planet", "sun", 568.32, 58232, 1432.041, 0.0520, 10.656, 0.342, 0.499, 26.73),
    ("Uranus", "uranus", "planet", "sun", 86.811, 25362, 2867.043, 0.0469, -17.24, 0.300, 0.488, 97.77),
    ("Neptune", "neptune", "planet", "sun", 102.409, 24622, 4514.953, 0.0097, 16.11, 0.290, 0.442, 28.32),
    ("Pluto", "pluto", "dwarf_planet", "sun", 0.01303, 1188, 5869.656, 0.2444, -153.2928, 0.72, 0.52, 119.51),
]
# The large moons and Charon (barycentre always: Pluto and Charon go round a
# point outside Pluto, Alex 24 Nov 2026). name, id, host, mass (kg), radius
# (km), a (km), e, inclination to the host's equator or orbit (deg; Triton's
# is retrograde), geometric albedo, Bond albedo or None, source. All
# synchronous: rotation equals the orbit, so none is kept here. Phobos and
# Deimos are printed with the Mars sheet (mass 1e15 kg, axis radii: the radius
# is the mean of the three), Charon with the Pluto sheet (mean distance).
MOONS = [
    ("Phobos", "phobos", "mars", 10.6e15, (13.0 + 11.4 + 9.1) / 3, 9378, 0.0151, 1.08, 0.07, None, "nssdca-mars"),
    ("Deimos", "deimos", "mars", 2.4e15, (7.8 + 6.0 + 5.1) / 3, 23459, 0.0005, 1.79, 0.08, None, "nssdca-mars"),
    ("Io", "io", "jupiter", 893.2e20, 1821.5, 421.8e3, 0.004, 0.04, 0.62, None, "nssdca-jovian-sats"),
    ("Europa", "europa", "jupiter", 480.0e20, 1560.8, 671.1e3, 0.009, 0.47, 0.68, None, "nssdca-jovian-sats"),
    ("Ganymede", "ganymede", "jupiter", 1481.9e20, 2631.2, 1070.4e3, 0.001, 0.18, 0.44, None, "nssdca-jovian-sats"),
    ("Callisto", "callisto", "jupiter", 1075.9e20, 2410.3, 1882.7e3, 0.007, 0.19, 0.19, None, "nssdca-jovian-sats"),
    # Saturn's other major moons and Uranus's five (read 24 Nov 2026; asked
    # for so Enceladus's plumes and the icy moons show in True). Radii are
    # the mean of the three printed axes where three are printed. Iapetus's
    # albedo is printed two-toned (0.05 / 0.5), so none is kept here.
    ("Mimas", "mimas", "saturn", 0.379e20, (208 + 197 + 191) / 3, 185.52e3, 0.0202, 1.53, 0.6, None, "nssdca-saturnian-sats"),
    ("Enceladus", "enceladus", "saturn", 1.08e20, (257 + 251 + 248) / 3, 238.02e3, 0.0045, 0.00, 1.0, None, "nssdca-saturnian-sats"),
    ("Tethys", "tethys", "saturn", 6.18e20, (538 + 528 + 526) / 3, 294.66e3, 0.0000, 1.86, 0.8, None, "nssdca-saturnian-sats"),
    ("Dione", "dione", "saturn", 11.0e20, (563 + 561 + 560) / 3, 377.40e3, 0.0022, 0.02, 0.7, None, "nssdca-saturnian-sats"),
    ("Rhea", "rhea", "saturn", 23.1e20, (765 + 763 + 762) / 3, 527.04e3, 0.0010, 0.35, 0.7, None, "nssdca-saturnian-sats"),
    ("Iapetus", "iapetus", "saturn", 18.1e20, (746 + 746 + 712) / 3, 3560.85e3, 0.0283, 14.72, None, None, "nssdca-saturnian-sats"),
    ("Miranda", "miranda", "uranus", 0.66e20, (240 + 234.2 + 232.9) / 3, 129.90e3, 0.0013, 4.34, 0.32, None, "nssdca-uranian-sats"),
    ("Ariel", "ariel", "uranus", 12.9e20, (581.1 + 577.9 + 577.7) / 3, 190.90e3, 0.0012, 0.04, 0.39, None, "nssdca-uranian-sats"),
    ("Umbriel", "umbriel", "uranus", 12.2e20, 584.7, 266.00e3, 0.0039, 0.13, 0.21, None, "nssdca-uranian-sats"),
    ("Titania", "titania", "uranus", 34.2e20, 788.9, 436.30e3, 0.0011, 0.08, 0.27, None, "nssdca-uranian-sats"),
    ("Oberon", "oberon", "uranus", 28.8e20, 761.4, 583.50e3, 0.0014, 0.07, 0.23, None, "nssdca-uranian-sats"),
    ("Titan", "titan", "saturn", 1345.5e20, 2575, 1221.87e3, 0.0292, 0.33, 0.22, None, "nssdca-saturnian-sats"),
    # Saturn's small inner moons (read 26 Nov 2026, sheet last updated 22 Jul
    # 2025; asked for by the ring structure work so Pan and Daphnis open the
    # Encke and Keeler gaps and the shepherds and co-orbitals sculpt the
    # rings). Radii are the mean of the three printed axis radii. Daphnis
    # prints no albedo. Epimetheus and Janus are the co-orbital pair.
    # Listed after Titan so the moon slots already in use (Titan is Saturn's
    # slot 6) keep their addresses.
    ("Pan", "pan", "saturn", 0.00005e20, (17 + 16 + 10) / 3, 133.583e3, 0.000, 0.0, 0.5, None, "nssdca-saturnian-sats"),
    ("Daphnis", "daphnis", "saturn", 0.000001e20, (4.3 + 4.1 + 3.2) / 3, 136.500e3, 0.000, 0.0, None, None, "nssdca-saturnian-sats"),
    ("Atlas", "atlas", "saturn", 0.00007e20, (20 + 18 + 9) / 3, 137.670e3, 0.000, 0.3, 0.8, None, "nssdca-saturnian-sats"),
    ("Prometheus", "prometheus", "saturn", 0.0016e20, (68 + 40 + 30) / 3, 139.353e3, 0.00204, 0.0, 0.5, None, "nssdca-saturnian-sats"),
    ("Pandora", "pandora", "saturn", 0.0014e20, (52 + 41 + 32) / 3, 141.700e3, 0.0042, 0.0, 0.7, None, "nssdca-saturnian-sats"),
    ("Epimetheus", "epimetheus", "saturn", 0.0053e20, (65 + 57 + 53) / 3, 151.422e3, 0.009, 0.34, 0.8, None, "nssdca-saturnian-sats"),
    ("Janus", "janus", "saturn", 0.0190e20, (102 + 93 + 76) / 3, 151.472e3, 0.007, 0.14, 0.9, None, "nssdca-saturnian-sats"),
    ("Triton", "triton", "neptune", 214e20, 1353.4, 354.76e3, 0.000016, 157.345, 0.72, None, "nssdca-neptunian-sats"),
    ("Charon", "charon", "pluto", 1.586e21, 606, 19596, 0.0, 0.00005, 0.42, 0.25, "nssdca-pluto"),
    # Round Pluto and Charon (the sheet's distances are barycentric, which
    # the Jacobi hierarchy reads as round the pair: they lie outside Charon).
    # No mass, eccentricity or lean is printed; the radius is half the mean
    # diameter (the mean of three axes where three are printed).
    ("Styx", "styx", "pluto", None, 10.5 / 2, 42413, None, None, 0.65, None, "nssdca-pluto"),
    ("Nix", "nix", "pluto", None, (48.4 + 33.8 + 31.4) / 6, 48690, None, None, 0.56, None, "nssdca-pluto"),
    ("Kerberos", "kerberos", "pluto", None, (19 + 10 + 9) / 6, 57750, None, None, 0.56, None, "nssdca-pluto"),
    ("Hydra", "hydra", "pluto", None, (62.0 + 36.5 + 29.3) / 6, 64721, None, None, 0.83, None, "nssdca-pluto"),
]
MOON_NOTES = {
    "phobos": "radius: the mean of the three axis radii printed; a is the mean distance from the centre of Mars",
    "deimos": "radius: the mean of the three axis radii printed; a is the mean distance from the centre of Mars",
    "pan": "radius: the mean of the three axis radii printed; orbits in the Encke gap",
    "daphnis": "radius: the mean of the three axis radii printed; orbits in the Keeler gap; no albedo printed",
    "atlas": "radius: the mean of the three axis radii printed",
    "prometheus": "radius: the mean of the three axis radii printed; inner shepherd of the F ring",
    "pandora": "radius: the mean of the three axis radii printed; outer shepherd of the F ring",
    "epimetheus": "radius: the mean of the three axis radii printed; co-orbital with Janus, the two swap orbits every four years",
    "janus": "radius: the mean of the three axis radii printed; co-orbital with Epimetheus, the two swap orbits every four years",
    "triton": "retrograde orbit (inclination over 90 degrees)",
    "charon": "a is the mean distance from Pluto; radius is equatorial; the pair goes round a point outside Pluto",
    "styx": "a is barycentric (round Pluto and Charon); no mass printed",
    "nix": "a is barycentric (round Pluto and Charon); no mass printed",
    "kerberos": "a is barycentric (round Pluto and Charon); no mass printed",
    "hydra": "a is barycentric (round Pluto and Charon); no mass printed",
}


def moon_row(name, pid, host, mass, r_km, a_km, e, inc, geo, bond, src):
    values = {
        "body.mass": val(mass, "kg", src),
        "body.radius": val(round(r_km, 4) * 1e3, "m", src),
        "orbit.semi_major_axis": val(a_km * 1e3, "m", src),
        "orbit.eccentricity": val(e, "1", src),
        "orbit.inclination": val(inc, "deg", src),
        "body.albedo_geometric": val(geo, "1", src),
    }
    values = {k: v for k, v in values.items() if v["v"] is not None}
    if bond is not None:
        values["body.albedo"] = val(bond, "1", src, note="Bond albedo")
    values.update(MOON_SURFACE.get(pid, {}))
    row = {"id": pid, "name": name, "kind": "moon", "host": host, "values": values}
    if pid in MOON_NOTES:
        row["note"] = MOON_NOTES[pid]
    return row


NOTES = {
    "jupiter": "rotation in System III (1965.0)",
    "saturn": "rotation in Saturnian System III",
    "uranus": "rotation from Voyager 2 radio science (magnetic coordinates)",
    "neptune": "rotation from Voyager 2 radio science (magnetic coordinates)",
    "moon": "semi-major axis about Earth",
}


def val(v, unit, src, **kw):
    out = {"v": v, "unit": unit, "src": src}
    out.update(kw)
    return out


# Moons' measured surfaces, where a probe has stood on one. Titan: Huygens
# landed on 14 Jan 2005 and its HASI read 93.65 +/- 0.25 K and 1467 +/- 1 hPa
# at the surface (Fulchignoni et al. 2005, abstract read 26 Nov 2026). Its
# air is nitrogen with methane (Niemann et al. 2005, abstract read 26 Nov
# 2026); the 5.65% methane near the surface is the data sheet's figure from
# the same paper, not yet re-read against it, so the shares are unverified.
TITAN_CH4 = 0.0565
MOON_SURFACE = {
    "titan": {
        "planet.surface_temperature": val(93.65, "K", "fulchignoni2005", note="at the Huygens landing site"),
        "atmosphere.surface_pressure": val(146700.0, "Pa", "fulchignoni2005", note="at the Huygens landing site"),
        "atmosphere.main": val("n2", "substance", "niemann2005", note="nitrogen with methane, by volume"),
        "atmosphere.composition.n2": val(round(1.0 - TITAN_CH4, 6), "1", "niemann2005", check="unverified",
                                         note="by volume near the surface; argon and trace organics left out"),
        "atmosphere.composition.ch4": val(TITAN_CH4, "1", "niemann2005", check="unverified",
                                          note="by volume near the surface (GCMS)"),
    },
}
MOON_SURFACE_SOURCES = [
    {"id": "fulchignoni2005", "text": "Fulchignoni, M. et al. 2005, Nature 438, 785: in situ measurements of the physical characteristics of Titan's environment (surface 93.65 +/- 0.25 K, 1467 +/- 1 hPa)",
     "status": "verified", "doi": "10.1038/nature04314"},
    {"id": "niemann2005", "text": "Niemann, H. B. et al. 2005, Nature 438, 779: the abundances of constituents of Titan's atmosphere from the GCMS instrument on the Huygens probe (nitrogen and methane; methane share unverified)",
     "status": "unverified", "doi": "10.1038/nature04122"},
]


# The second zonal gravity harmonic J2, as printed (x 10^-6; read 26 Nov 2026,
# Saturn sheet last updated 18 Mar 2025). The sheet names no reference radius
# for it. Only Saturn's is kept so far
# (asked for by the ring structure work, to place its resonances).
J2 = {"saturn": 16298.0}
J2_NOTE = "second zonal gravity harmonic; the sheet states no reference radius"


def planet_row(name, pid, kind, host, m24, r_km, a6, e, rot_h, bond, geo, obl):
    src = "nssdca-" + pid
    values = {
        "body.mass": val(m24 * 1e24, "kg", src),
        "body.radius": val(r_km * 1e3, "m", src, note="volumetric mean radius"),
        "orbit.semi_major_axis": val(a6 * 1e9, "m", src),
        "orbit.eccentricity": val(e, "1", src),
        "body.rotation_period": val(abs(rot_h) * HOUR, "s", src, note="sidereal; spin sense is in body.obliquity"),
        "body.obliquity": val(obl, "deg", src, note="obliquity to orbit; above 90 means retrograde spin"),
        "body.albedo": val(bond, "1", src, note="Bond albedo"),
        "body.albedo_geometric": val(geo, "1", src),
    }
    if pid in J2:
        values["body.j2"] = val(J2[pid] * 1e-6, "1", src, note=J2_NOTE)
    row = {"id": pid, "name": name, "kind": kind, "host": host, "values": values}
    values.update(surface_facts(pid, src))
    values.update(composition_facts(pid, src))
    values.update(interior_facts(pid))
    if pid in NOTES:
        row["note"] = NOTES[pid]
    return row


# What a catalogued world's surface is like, as the NSSDCA sheet prints it
# (read 23 Nov 2026), for the surface chain to use before any estimate
# (proven core, test drive). Only a printed average is kept: the Moon and Pluto
# print a range, so they have no temperature here; Mercury's and the Moon's
# surface pressures are printed as upper bounds of an exosphere, so they have
# none either. atmosphere.main is the main gas by volume, as a substance id on
# the master data sheet. Fields: average surface temperature (K), surface
# pressure (bar), main gas.
SURFACE = {
    "mercury": (440.0, None, None),
    "venus": (737.0, 92.0, "co2"),
    "earth": (288.0, 1.014, "n2"),
    "mars": (214.0, 6.36e-3, "co2"),
    "pluto": (None, 13e-6, "n2"),
    # The giants have no surface: NSSDCA's temperature at the 1 bar level.
    "jupiter": (165.0, None, None),
    "saturn": (134.0, None, None),
    "uranus": (76.0, None, None),
    "neptune": (72.0, None, None),
}
SURFACE_NOTES = {
    "mars": "average; NSSDCA prints ~214 K",
    "mars_p": "at mean radius; varies 4.0 to 8.7 mb with the season",
    "pluto_p": "NSSDCA prints ~13 microbar",
    "venus_gas": "96.5% CO2, 3.5% N2 near the surface, by volume",
    "earth_gas": "78.08% N2, 20.95% O2, dry air by volume",
    "mars_gas": "95.1% CO2, 2.59% N2, 1.94% Ar by volume",
    "pluto_gas": "99% N2, 0.5% CH4 by volume",
    "jupiter": "temperature at the 1 bar level (no surface)",
    "saturn": "temperature at the 1 bar level (no surface)",
    "uranus": "temperature at the 1 bar level (no surface)",
    "neptune": "temperature at the 1 bar level (no surface)",
}
BAR = 1e5


def surface_facts(pid, src):
    out = {}
    if pid == "earth":
        out["body.surface_water_oceans"] = val(1.0, "earth_oceans", "charette-smith2010",
                                               note="1 by definition of the unit (1.335e18 m3 of ocean)")
    if pid not in SURFACE:
        return out
    t, p_bar, gas = SURFACE[pid]
    if t is not None:
        out["planet.surface_temperature"] = val(t, "K", src, note=SURFACE_NOTES.get(pid, "average temperature"))
    if p_bar is not None:
        out["atmosphere.surface_pressure"] = val(round(p_bar * BAR, 9), "Pa", src, **(
            {"note": SURFACE_NOTES[pid + "_p"]} if pid + "_p" in SURFACE_NOTES else {}))
    if gas is not None:
        out["atmosphere.main"] = val(gas, "substance", src, note=SURFACE_NOTES[pid + "_gas"])
    return out

# The air's make-up by volume (mole fraction), gas by gas, as the NSSDCA sheets
# print it (read 23 Nov 2026), keyed by substance id on the master data sheet:
# one row per gas, atmosphere.composition.<id>, unit 1. Shares printed in ppm are
# converted here. Gases the sheet has no substance for are left out and named in
# the level note. The giants' make-up is at the 1 bar level (no surface).
PPM = 1e-6
COMPOSITION = {
    "earth": ({"n2": 0.7808, "o2": 0.2095, "ar": 9340 * PPM, "co2": 420 * PPM, "ne": 18.18 * PPM, "he": 5.24 * PPM,
               "ch4": 1.94 * PPM, "kr": 1.14 * PPM, "h2": 0.55 * PPM},
              "dry air; water is highly variable, typically about 1%"),
    "venus": ({"co2": 0.965, "n2": 0.035, "so2": 150 * PPM, "ar": 70 * PPM, "h2o": 20 * PPM, "co": 17 * PPM,
               "he": 12 * PPM, "ne": 7 * PPM}, "near the surface"),
    "mars": ({"co2": 0.951, "n2": 0.0259, "ar": 0.0194, "o2": 0.0016, "co": 0.0006, "h2o": 210 * PPM,
              "ne": 2.5 * PPM, "kr": 0.3 * PPM, "xe": 0.08 * PPM}, "nitric oxide (100 ppm) and HDO (0.85 ppm) left out: not on the sheet"),
    "pluto": ({"n2": 0.99, "ch4": 0.005, "co": 0.0005}, "trace HCN and C2Hx hydrocarbons left out"),
    "jupiter": ({"h2": 0.898, "he": 0.102, "ch4": 3000 * PPM, "nh3": 260 * PPM, "h2o": 4 * PPM},
                "at the 1 bar level; HD (28 ppm) and ethane (5.8 ppm) left out; water varies with pressure"),
    "saturn": ({"h2": 0.963, "he": 0.0325, "ch4": 4500 * PPM, "nh3": 125 * PPM},
               "at the 1 bar level; HD (110 ppm) and ethane (7 ppm) left out"),
    "uranus": ({"h2": 0.825, "he": 0.152, "ch4": 0.023}, "at the 1 bar level; HD (148 ppm) left out"),
    "neptune": ({"h2": 0.80, "he": 0.19, "ch4": 0.015}, "at the 1 bar level; HD (192 ppm) and ethane (1.5 ppm) left out"),
}


# Rocky worlds' measured interiors (proven core, 24 Nov 2026): the core's share
# of the mass, and the moment of inertia factor C = I/(M R^2). Recalled from the
# papers named, not yet re-read against them, so every value is marked
# unverified. Venus has no measured core, so it has no core share here.
INTERIOR = {
    "mercury": ((0.69, "hauck2013", "from a core radius of 2020 +/- 30 km"), (0.346, "margot2012")),
    "venus": (None, (0.337, "margot2021")),
    "earth": ((0.325, "mcdonough2003", "the core is 32.5% of Earth's mass"), (0.3307, "yoder1995")),
    "moon": ((0.015, "weber2011", "from a core radius of about 330 km"), (0.3932, "konopliv1998")),
    "mars": ((0.24, "staehler2021", "from a core radius of 1830 +/- 40 km and a core density near 6000 kg/m3"), (0.3644, "konopliv2011")),
}
INTERIOR_SOURCES = [
    {"id": "hauck2013", "text": "Hauck, S. A. et al. 2013, JGR Planets 118, 1204: the curious case of Mercury's internal structure (core radius 2020 +/- 30 km)", "status": "unverified", "doi": "10.1002/jgre.20091"},
    {"id": "margot2012", "text": "Margot, J.-L. et al. 2012, JGR Planets 117, E00L09: Mercury's moment of inertia from spin and gravity data (C/MR^2 = 0.346 +/- 0.014)", "status": "unverified", "doi": "10.1029/2012JE004161"},
    {"id": "margot2021", "text": "Margot, J.-L. et al. 2021, Nature Astronomy 5, 676: spin state and moment of inertia of Venus (C/MR^2 = 0.337 +/- 0.024)", "status": "unverified", "doi": "10.1038/s41550-021-01339-7"},
    {"id": "mcdonough2003", "text": "McDonough, W. F. 2003, Treatise on Geochemistry 2, 547: compositional model for the Earth's core (the core is 32.5% of Earth's mass)", "status": "unverified", "doi": "10.1016/B0-08-043751-6/02015-6"},
    {"id": "yoder1995", "text": "Yoder, C. F. 1995, Astrometric and geodetic properties of Earth and the Solar System, AGU Reference Shelf 1, 1 (Earth C/MR^2 = 0.3307)", "status": "unverified", "doi": "10.1029/RF001p0001"},
    {"id": "weber2011", "text": "Weber, R. C. et al. 2011, Science 331, 309: seismic detection of the lunar core (radius about 330 km)", "status": "unverified", "doi": "10.1126/science.1199375"},
    {"id": "konopliv1998", "text": "Konopliv, A. S. et al. 1998, Science 281, 1476: improved gravity field of the Moon from Lunar Prospector (C/MR^2 = 0.3932 +/- 0.0002)", "status": "unverified", "doi": "10.1126/science.281.5382.1476"},
    {"id": "staehler2021", "text": "Staehler, S. C. et al. 2021, Science 373, 443: seismic detection of the martian core (radius 1830 +/- 40 km)", "status": "unverified", "doi": "10.1126/science.abi7730"},
    {"id": "konopliv2011", "text": "Konopliv, A. S. et al. 2011, Icarus 211, 401: Mars high resolution gravity fields from MRO, Mars seasonal gravity and other dynamical parameters (C/MR^2 = 0.3644 +/- 0.0005)", "status": "unverified", "doi": "10.1016/j.icarus.2010.10.004"},
]


def interior_facts(pid):
    if pid not in INTERIOR:
        return {}
    core, moi = INTERIOR[pid]
    out = {}
    if core is not None:
        out["interior.core_mass_fraction"] = val(core[0], "1", core[1], note=core[2], check="unverified")
    out["body.moment_of_inertia_factor"] = val(moi[0], "1", moi[1], note="C = I/(M R^2)", check="unverified")
    return out


def composition_facts(pid, src):
    if pid not in COMPOSITION:
        return {}
    mix, where = COMPOSITION[pid]
    return {"atmosphere.composition." + g: val(float(f"{x:.6g}"), "1", src, note="by volume, " + where)
            for g, x in mix.items()}


def build():
    sys.path.insert(0, os.path.join(ROOT, "scripts"))
    import sol_extra  # the Sun's age, Ceres and the regions, each with its source
    sun = {"id": "sun", "name": "Sun", "kind": "star", "host": None, "values": {
        "body.mass": val(1988400 * 1e24, "kg", "nssdca-sun"),
        "body.radius": val(695700 * 1e3, "m", "nssdca-sun", note="volumetric mean radius; equals the IAU nominal radius"),
        "star.luminosity": val(382.8 * 1e24, "W", "nssdca-sun", note="equals the IAU nominal luminosity"),
        "star.effective_temperature": val(5772.0, "K", "nssdca-sun", note="equals the IAU nominal temperature"),
    }}
    sun["values"].update(sol_extra.SUN_AGE)
    sun["values"].update(sun_frame())
    bodies = [sun] + [planet_row(*p) for p in PLANETS] + [moon_row(*x) for x in MOONS] + sol_extra.BODIES
    sources = sorted(SOURCES + INTERIOR_SOURCES + MOON_SURFACE_SOURCES + sol_extra.SOURCES, key=lambda s: s["id"])
    regions = sol_extra.regions({b["id"]: b for b in bodies})
    return {
        "format": "sse.pack/1", "id": "sse.base-sol", "version": "0.1.0", "kind": ["base-physics"],
        "kernel_range": "^0.1", "licence": "Apache-2.0",
        "provides": {"other": {"sol": {
            "about": "The catalogued Solar System: the Sun, the planets, the Moon and the large moons, Ceres, Pluto and Charon, and the belts and the Oort cloud, as measured. Built by scripts/build-base-sol.py; every value names its source. Body facts, not the master data sheet's reference objects.",
            "bodies": bodies, "regions": regions, "sources": sources,
        }}},
    }


def validate(pack):
    sol = pack["provides"]["other"]["sol"]
    ids = {s["id"] for s in sol["sources"]}
    assert len(ids) == len(sol["sources"]), "duplicate source id"
    for s in sol["sources"]:
        assert s.get("doi") or s.get("url"), "source %s has no DOI or URL" % s["id"]
    bids = {b["id"] for b in sol["bodies"]}
    sheet = json.load(open(SHEET, encoding="utf-8"))["provides"]["reference"]
    subs = {x["id"] for x in sheet["substances"]}
    for b in sol["bodies"]:
        assert b["host"] is None or b["host"] in bids, "%s: unknown host %s" % (b["id"], b["host"])
        for k, v in b["values"].items():
            assert v["src"] in ids, "%s %s: unknown source %s" % (b["id"], k, v["src"])
            if k.startswith("atmosphere.composition."):
                assert k.rsplit(".", 1)[1] in subs, "%s %s: not a substance on the sheet" % (b["id"], k)
                assert 0.0 < v["v"] <= 1.0, (b["id"], k)
            if v["unit"] == "substance":
                assert v["v"] in subs, "%s %s: %s is not a substance on the sheet" % (b["id"], k, v["v"])
                continue
            assert isinstance(v["v"], (int, float)) and v["v"] == v["v"], "%s %s: not a number" % (b["id"], k)
    for b in sol["bodies"]:
        mix = [v["v"] for k, v in b["values"].items() if k.startswith("atmosphere.composition.")]
        assert not mix or 0.99 <= sum(mix) <= 1.01, "%s: air shares sum to %.4f" % (b["id"], sum(mix))
    for r in sol["regions"]:
        assert r["host"] in bids and r["src"] in ids and 0 < r["inner_m"] < r["outer_m"], r["id"]
    # The sheet's yardsticks and this Sun must agree.
    ref = json.load(open(SHEET, encoding="utf-8"))["provides"]["reference"]["constants"]
    c = {x["id"]: x["value"]["v"] for x in ref}
    sun = next(b for b in sol["bodies"] if b["id"] == "sun")["values"]
    for field, cid, tol in [("body.radius", "R_sun", 0.0), ("star.luminosity", "L_sun", 0.0),
                            ("star.effective_temperature", "T_sun", 0.0), ("body.mass", "M_sun", 2e-4)]:
        rel = abs(sun[field]["v"] - c[cid]) / c[cid]
        assert rel <= tol + 1e-12, "Sun %s differs from the sheet's %s by %.2e" % (field, cid, rel)
    earth = next(b for b in sol["bodies"] if b["id"] == "earth")["values"]
    for field, cid in [("body.mass", "M_earth"), ("body.radius", "R_earth_mean")]:
        rel = abs(earth[field]["v"] - c[cid]) / c[cid]
        assert rel <= 2e-4, "Earth %s differs from the sheet's %s by %.2e" % (field, cid, rel)


def main():
    pack = build()
    validate(pack)
    text = json.dumps(pack, indent=1, ensure_ascii=False, allow_nan=False) + "\n"
    if "--check" in sys.argv:
        cur = open(OUT, encoding="utf-8").read() if os.path.exists(OUT) else ""
        if cur != text:
            print("packs/base-sol is stale: run scripts/build-base-sol.py")
            return 1
        print("base-sol pack is current")
        return 0
    os.makedirs(os.path.dirname(OUT), exist_ok=True)
    with open(OUT, "w", encoding="utf-8", newline="\n") as fh:
        fh.write(text)
    print("wrote packs/base-sol/pack.json (%d bodies, %d regions)" % (len(pack["provides"]["other"]["sol"]["bodies"]), len(pack["provides"]["other"]["sol"]["regions"])))
    return 0


if __name__ == "__main__":
    sys.exit(main())
