"""The Sun's age, Ceres, and Sol's belts and Oort cloud, for build-base-sol.py.

Each value is read from the source named beside it (22 Nov 2026). Region edges
are conventions or estimates, graded so.
"""
import json
import os

_SHEET = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "packs", "base-reference", "pack.json")
# The astronomical unit, read from the sheet (constant:au, IAU 2012 B2), to turn AU into metres.
AU = next(c["value"]["v"] for c in json.load(open(_SHEET, encoding="utf-8"))["provides"]["reference"]["constants"] if c["id"] == "au")
HOUR = 3600.0
NS = "https://nssdc.gsfc.nasa.gov/planetary/factsheet/%sfact.html"

SOURCES = [
    {"id": "connelly2012", "text": "Connelly, J. N. et al. 2012, Science 338, 651: the absolute chronology and thermal processing of solids in the solar protoplanetary disk (CAIs at 4567.30 ± 0.16 Myr)",
     "status": "verified", "doi": "10.1126/science.1226919"},
    {"id": "russell2016", "text": "Russell, C. T. et al. 2016, Science 353, 1008: Dawn arrives at Ceres, exploration of a small, volatile-rich world (mean radius 469.7 km, mass 9.384e20 kg, obliquity 4°)",
     "status": "verified", "doi": "10.1126/science.aaf4219"},
    {"id": "park2016", "text": "Park, R. S. et al. 2016, Nature 537, 515: a partially differentiated interior for (1) Ceres deduced from its gravity field and shape (mass 938.416e18 kg)",
     "status": "verified", "doi": "10.1038/nature18955"},
    {"id": "ciarniello2017", "text": "Ciarniello, M. et al. 2017, A&A 598, A130: spectrophotometric properties of dwarf planet Ceres from the VIR spectrometer on board the Dawn mission (geometric albedo 0.094 ± 0.007 at 0.55 µm)",
     "status": "verified", "doi": "10.1051/0004-6361/201629490"},
    {"id": "li2016", "text": "Li, J.-Y. et al. 2016, ApJL 817, L22: surface albedo and spectral variability of Ceres (0.034 ± 0.001 for the Ceres average, Dawn FC clear filter)",
     "status": "verified", "doi": "10.3847/2041-8205/817/2/L22"},
    {"id": "trujillo2001", "text": "Trujillo, C. A., Jewitt, D. C. and Luu, J. X. 2001, AJ 122, 457: properties of the trans-Neptunian belt, statistics from the Canada-France-Hawaii Telescope survey (an outer edge to the classical belt at 50 AU)",
     "status": "verified", "doi": "10.1086/321117"},
    {"id": "kaib-volk2022", "text": "Kaib, N. A. and Volk, K. 2022, Dynamical population of comet reservoirs, arXiv:2206.00010 (the Galactic tide sets the Oort cloud's outer limit near 1 to 2 × 10^5 au)",
     "status": "verified", "url": "https://arxiv.org/abs/2206.00010"},
]


def val(v, unit, src, **kw):
    out = {"v": v, "unit": unit, "src": src}
    out.update(kw)
    return out


SUN_AGE = {"system.age_gyr": val(4.56730, "Gyr", "connelly2012", note="age of the first solids (CAIs), 4567.30 ± 0.16 Myr")}

BODIES = [
    {"id": "ceres", "name": "Ceres", "kind": "dwarf_planet", "host": "sun", "values": {
        "body.mass": val(9.384e20, "kg", "russell2016", also="park2016: 9.38416e20"),
        "body.radius": val(469.7e3, "m", "russell2016", note="geometric mean radius"),
        "orbit.semi_major_axis": val(2.768 * AU, "m", "nssdca-asteroid", note="2.768 AU as printed"),
        "orbit.eccentricity": val(0.0758, "1", "nssdca-asteroid"),
        "body.rotation_period": val(9.074 * HOUR, "s", "nssdca-asteroid"),
        "body.obliquity": val(4.0, "deg", "russell2016"),
        "body.albedo_geometric": val(0.094, "1", "ciarniello2017", note="at 0.55 µm"),
        "body.albedo": val(0.034, "1", "li2016", check="unverified",
                           note="the paper's Ceres average in the clear filter; whether it is a Bond albedo was not confirmed from the text"),
    }},
]


def regions(bodies):
    aj = bodies["jupiter"]["values"]["orbit.semi_major_axis"]["v"]
    an = bodies["neptune"]["values"]["orbit.semi_major_axis"]["v"]
    return [
        {"id": "main-belt", "name": "Main asteroid belt", "host": "sun",
         "inner_m": aj * (1 / 4) ** (2 / 3), "outer_m": aj * (1 / 2) ** (2 / 3),
         "src": "nssdca-jupiter", "grade": "convention",
         "note": "edges at the 4:1 and 2:1 mean-motion resonances with Jupiter (the outer Kirkwood gaps), worked out from Jupiter's semi-major axis by Kepler's third law, a = a_J (p/q)^(2/3)"},
        {"id": "kuiper-belt", "name": "Kuiper belt", "host": "sun",
         "inner_m": an, "outer_m": 50 * AU, "src": "trujillo2001", "grade": "measured",
         "note": "inner edge at Neptune's orbit (convention, from nssdca-neptune); outer edge of the classical belt at 50 AU"},
        {"id": "oort-cloud", "name": "Oort cloud", "host": "sun",
         "inner_m": 5000 * AU, "outer_m": 1e5 * AU, "src": "kaib-volk2022", "grade": "estimate", "check": "unverified",
         "note": "outer limit set by the Galactic tide at 1 to 2 × 10^5 au (quoted); the inner edge near 5,000 au for bodies scattered by Uranus and Neptune was read in paraphrase, not quoted. A sphere (ruling)."},
    ]
