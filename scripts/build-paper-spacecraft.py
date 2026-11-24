#!/usr/bin/env python3
"""Build the "Paper spacecraft" store pack (track RK, package RK6).

Writes store-packs/paper-spacecraft/pack.json from the figures below. Every
figure is quoted from the primary study named in its `src` (the quotes and
pages are in handover/packages/RK6-data/RK6-sources.md). A figure the builder
works out is marked `derived` and says how. Run:

    python3 -I scripts/build-paper-spacecraft.py

The pack is published on Explorers and not built into SSE4 (like Sol today).
Its ships fly on SSE4's own physics: fitted engines (RK9's power books) set the
drive, NAV plans the course, and the tech reader marks each advance "not yet
built". crates/sse-kernel/tests/paper_spacecraft.rs flies them.
"""

import json
import os
import sys

G0 = 9.80665  # standard gravity, m/s2 (exact by definition, CGPM 1901)
LBF_N = 4.4482216152605  # pound-force in newtons (exact by definition)
LB_KG = 0.45359237  # pound in kilograms (exact by definition)

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(ROOT, "store-packs", "paper-spacecraft", "pack.json")


def isp_km_s(isp_s):
    """Exhaust speed from specific impulse in seconds (lbf s / lbm is the same number)."""
    return round(isp_s * G0 / 1000.0, 3)


def jet_w(thrust_n, exhaust_km_s):
    return 0.5 * thrust_n * exhaust_km_s * 1000.0


SOURCES = {
    "williams2005": "Williams, C. H., Dudzinski, L. A., Borowski, S. K. and Juhasz, A. J. 2005, Realizing '2001: A Space Odyssey': piloted spherical torus nuclear fusion propulsion, NASA/TM-2005-213559 (AIAA-2001-3805), Tables 1 and 2. https://ntrs.nasa.gov/api/citations/20050160960/downloads/20050160960.pdf",
    "adams2003-callisto": "Adams, R. B. et al. 2003, Crewed mission to Callisto using advanced plasma propulsion systems, AIAA 2003-4527 (NASA HOPE study), Tables 1 to 8. https://ntrs.nasa.gov/api/citations/20030065984/downloads/20030065984_Update.pdf",
    "beals1988": "Beals, K. A. et al. 1988, Project Longshot: an unmanned probe to Alpha Centauri, US Naval Academy, NASA CR-184718 (OCR scan). https://ntrs.nasa.gov/api/citations/19890007533/downloads/19890007533.pdf",
    "ga4891": "General Atomic 1964, Nuclear pulse vehicle study, condensed summary report, GA-4891, contract NAS 8-11053 for NASA Marshall (OCR scan). https://www.classe.cornell.edu/~seb/celestia/orion/files/19770085619_1977085619.pdf",
    "ga5009-3": "Shipps, P. R. and Nance, J. C. (General Atomic) 1964, Nuclear pulse space vehicle study, vol. III, conceptual vehicle designs and operational systems, GA-5009, section 2.1 (OCR scan). https://www.classe.cornell.edu/~seb/celestia/orion/files/19760065935_1976065935.pdf",
    "nance1964": "Nance, J. C. 1964, Nuclear pulse propulsion, GA-5572, IEEE 11th Nuclear Science Symposium, DTIC AD0450629. https://apps.dtic.mil/sti/pdfs/AD0450629.pdf",
    "werka2012": "Werka, R., Clark, R., Sheldon, R. and Percy, T. 2012, Concept assessment of a fission fragment rocket engine (FFRE) propelled spacecraft, NASA NIAC phase I final report, part 1. https://www.nasa.gov/wp-content/uploads/2019/04/niac_2011_phasei_werka_ffre_tagged.pdf",
    "mclafferty1968": "McLafferty, G. H. and Bauer, H. E. 1968, Studies of specific nuclear light bulb and open-cycle vortex-stabilized gaseous nuclear rocket engines, United Aircraft F-910093-37, NASA CR-1030, Results item 1. https://ntrs.nasa.gov/api/citations/19680012569/downloads/19680012569.pdf",
    "ragsdale1991": "Ragsdale, R. 1991, Open cycle gas core nuclear rockets, NASA conference talk (N92), figure 6 baseline. https://ntrs.nasa.gov/api/citations/19920001890/downloads/19920001890.pdf",
    "ragsdale-willis1971": "Ragsdale, R. G. and Willis, E. A. 1971, Gas-core rocket reactors: a new look, NASA TM X-67823 (OCR scan). https://ntrs.nasa.gov/api/citations/19710014431/downloads/19710014431.pdf",
    "johnson2003": "Johnson, S. 2003, Project Prometheus, workshop slides, NASA/TM-2003-212598, slide 13: up to 100 kWe, specific mass 50 kg/kWe. https://ntrs.nasa.gov/api/citations/20040034798/downloads/20040034798.pdf",
    "mason2003": "Mason, L. S. 2003, A power conversion concept for the Jupiter Icy Moons Orbiter, NASA/TM-2003-212596: two 100 kWe Brayton units, 496 kWt. https://ntrs.nasa.gov/api/citations/20030105581/downloads/20030105581.pdf",
    "borowski2009": "Borowski, S. K., McCurdy, D. R. and Packard, T. W. 2009, Nuclear thermal rocket/vehicle characteristics and sensitivity trades for NASA's Mars Design Reference Architecture (DRA) 5.0 study, Nuclear and Emerging Technologies for Space 2009, paper 203599 (NASA E-18236), Tables I and II and section V. https://ntrs.nasa.gov/api/citations/20120012928/downloads/20120012928.pdf",
    "borowski2012": "Borowski, S. K., McCurdy, D. R. and Packard, T. W. 2012, Modular growth NTR space transportation system for future NASA human lunar, NEA and Mars exploration missions, AIAA-2012-5144: Isp ~906 s; four 12.5 kWe arrays give ~50 kWe at Mars. https://ntrs.nasa.gov/api/citations/20150006732/downloads/20150006732.pdf",
    "parkin2018": "Parkin, K. L. G. 2018, The Breakthrough Starshot system model, Acta Astronautica 152, 370, arXiv:1805.01306 v3, Tables 1 and 2. https://arxiv.org/pdf/1805.01306",
    "gwin-mee1959": "Gwin, R. and Mee, W. T. 1959, Critical assemblies of uranium metal, Y-1248, Table I: a bare sphere of 93 to 94% U-235 metal is critical at 53.9 kg. https://www.osti.gov/servlets/purl/4246875",
}

# ---- Engines (provides.other.engines: sse-nav EngineDef by id) -------------

ds2_j_f = 6250 * LBF_N
ds2_j_v = isp_km_s(35435)
ds2_s_f = 4690 * LBF_N
ds2_s_v = isp_km_s(47205)

hope_mtf_v = isp_km_s(77000)
hope_mtf_jet = 2.071e9
hope_mtf_f = 2 * hope_mtf_jet / (hope_mtf_v * 1000)
hope_mtf_heat = 176.5e6 + 33.9e6

hope_mpd_v = isp_km_s(8000)
hope_mpd_jet = 24e6
hope_mpd_draw = 37e6
hope_mpd_f = 2 * hope_mpd_jet / (hope_mpd_v * 1000)

ls_v = isp_km_s(1.02e6)
ls_f = 1838.0

orion_v = isp_km_s(1800)

ffre_f = 10 * LBF_N
ffre_v = isp_km_s(527000)

nlb_f = 92000 * LBF_N
nlb_v = isp_km_s(1870)

ocgc_f = 50000 * LBF_N
ocgc_v = isp_km_s(5200)


def r(x, n=6):
    return float(f"{x:.{n}g}")


ENGINES = {
    "paper.discovery-ii-jupiter": {
        "name": "Discovery II spherical torus (Jupiter setting)",
        "fuels": ["deuterium", "helium-3", "hydrogen"],
        "thrust_n": r(ds2_j_f), "exhaust_km_s": ds2_j_v, "energy_from": "fuel",
        "efficiency": 0.8, "mass_t": 310.0,
        "exhaust_range_km_s": [ds2_j_v, ds2_s_v],
    },
    "paper.discovery-ii-saturn": {
        "name": "Discovery II spherical torus (Saturn setting)",
        "fuels": ["deuterium", "helium-3", "hydrogen"],
        "thrust_n": r(ds2_s_f), "exhaust_km_s": ds2_s_v, "energy_from": "fuel",
        "efficiency": 0.8, "mass_t": 310.0,
        "exhaust_range_km_s": [ds2_j_v, ds2_s_v],
    },
    "paper.hope-mtf": {
        "name": "HOPE magnetised target fusion drive (D-He3)",
        "fuels": ["deuterium", "helium-3", "hydrogen"],
        "thrust_n": r(hope_mtf_f), "exhaust_km_s": hope_mtf_v, "energy_from": "fuel",
        "efficiency": r(hope_mtf_jet / (hope_mtf_jet + hope_mtf_heat), 3), "reject_k": 1250.0,
    },
    "paper.hope-mpd": {
        "name": "HOPE magnetoplasmadynamic thrusters (piloted, 37 MW)",
        "fuels": ["hydrogen"],
        "thrust_n": r(hope_mpd_f), "exhaust_km_s": hope_mpd_v, "energy_from": "electric",
        "electric_w": hope_mpd_draw, "efficiency": r(hope_mpd_jet / hope_mpd_draw, 3),
    },
    "paper.longshot-icf": {
        "name": "Longshot inertial fusion pulse drive (D-He3 pellets)",
        "fuels": ["deuterium", "helium-3"],
        "thrust_n": ls_f, "exhaust_km_s": ls_v, "energy_from": "fuel",
    },
    "paper.orion-10m": {
        "name": "Orion 10 m nuclear pulse module",
        "fuels": ["pulse units"],
        "thrust_n": r(0.3 * 599000 * G0), "exhaust_km_s": orion_v, "energy_from": "fuel",
        "mass_t": r(200000 * LB_KG / 1000),
        "exhaust_range_km_s": [orion_v, isp_km_s(2500)],
    },
    "paper.pewee-25klbf": {
        "name": "Pewee-class nuclear thermal engine (25 klbf)",
        "fuels": ["hydrogen"],
        "thrust_n": r(25000 * LBF_N), "exhaust_km_s": isp_km_s(900), "energy_from": "fuel",
    },
    "paper.ffre": {
        "name": "Fission fragment rocket engine",
        "fuels": ["uranium dioxide"],
        "thrust_n": r(ffre_f), "exhaust_km_s": ffre_v, "energy_from": "fuel",
        "efficiency": r(jet_w(ffre_f, ffre_v) / 1.0e9, 3), "reject_k": 1350.0, "mass_t": 113.4,
    },
    "paper.nuclear-light-bulb": {
        "name": "Nuclear light bulb (closed-cycle gas core)",
        "fuels": ["hydrogen"],
        "thrust_n": r(nlb_f), "exhaust_km_s": nlb_v, "energy_from": "fuel",
        "efficiency": r(jet_w(nlb_f, nlb_v) / 4.6e9, 3), "mass_t": r(70000 * LB_KG / 1000),
    },
    "paper.open-cycle-gas-core": {
        "name": "Open-cycle gas core (5,000 MW baseline)",
        "fuels": ["hydrogen"],
        "thrust_n": r(ocgc_f), "exhaust_km_s": ocgc_v, "energy_from": "fuel",
        "reject_k": 1100.0, "mass_t": r(250000 * LB_KG / 1000),
    },
}

ENGINE_NOTES = {
    "paper.discovery-ii-jupiter": {"src": ["williams2005"], "derived": "thrust 6,250 lbf in newtons; exhaust from 35,435 s; the two settings are one 4,830 MW jet geared for Jupiter and Saturn; jet efficiency 0.8 is the study's assumption; mass_t is the fusion reactor line of Table 1"},
    "paper.discovery-ii-saturn": {"src": ["williams2005"], "derived": "thrust 4,690 lbf in newtons; exhaust from 47,205 s"},
    "paper.hope-mtf": {"src": ["adams2003-callisto"], "derived": "thrust = 2 x jet power (2.071 GW) / exhaust (77,000 s): the paper gives no newtons; efficiency = jet over jet plus the 176.5 MW and 33.9 MW its radiators shed"},
    "paper.hope-mpd": {"src": ["adams2003-callisto"], "derived": "thrust = 2 x 24 MW / exhaust (8,000 s); efficiency = 24 MW of jet over 37 MW in"},
    "paper.longshot-icf": {"src": ["beals1988"], "derived": "exhaust from 1.02e6 s; thrust 1,838 N read from the OCR of the tank-sizing appendix"},
    "paper.orion-10m": {"src": ["ga4891", "ga5009-3", "nance1964"], "derived": "the reports give no average thrust; this is the 0.3 g start-up of GA-5009 on the 599,000 kg departure mass of GA-4891; exhaust from 1,800 s, the low end of 1,800 to 2,500 s; module 200,000 lb"},
    "paper.pewee-25klbf": {"src": ["borowski2009", "borowski2012"], "derived": "thrust 25 klbf in newtons; exhaust from 900 s, the value the DRA 5.0 analysis used (900 s at 2650 K to 910 s at 2700 K; ~906 s in the 2012 paper); engine mass not given (thrust-to-weight about 3.4)"},
    "paper.ffre": {"src": ["werka2012"], "derived": "thrust 10 lbf in newtons; exhaust from 527,000 s; efficiency = jet over 1000 MW thermal (the report says about 11%); 1350 K is its heat-shield radiator, which sheds 699 MW"},
    "paper.nuclear-light-bulb": {"src": ["mclafferty1968"], "derived": "thrust 92,000 lb in newtons; exhaust from 1870 s; efficiency = jet over the 4600 MW engine power; mass 70,000 lb"},
    "paper.open-cycle-gas-core": {"src": ["ragsdale1991", "ragsdale-willis1971"], "derived": "thrust 50,000 lb; exhaust from 5200 s; engine 250,000 lb with its radiator (1100 K from Ragsdale and Willis). As printed its jet (5.7 GW) is more than its 5,000 MW reactor: the books do not close, and the reading says so"},
}

# The Orion dry mass: GA-4891 gives the departure mass (599,000 kg) and the
# mission's velocity (72,850 ft/s) but not a dry mass; at 1,800 s the rocket
# equation gives the mass ratio exp(22.205 / 17.652), about 3.52.
ORION_RATIO = 2.718281828459045 ** (72850 * 0.3048 / 1000 / orion_v)
assert abs(ORION_RATIO - 3.518) < 1e-3, ORION_RATIO

# ---- Ships (provides.other.nav_ships.ships: sse-nav Ship records) ---------

SHIPS = {
    "paper.discovery-ii": {
        "name": "Discovery II (to Jupiter)", "sustained": True,
        "accel_g": r(ds2_j_f / (1690e3 * G0)), "exhaust_km_s": ds2_j_v,
        "wet_t": 1690.0, "dry_t": 1690.0 - 807.0 - 11.0,
        "fuels": ["deuterium", "helium-3", "hydrogen"], "refuel": ["docking"],
        "crew": 6, "cargo_t": 172.0,
        "engines": [{"engine": "paper.discovery-ii-jupiter"}],
    },
    "paper.discovery-ii-saturn": {
        "name": "Discovery II (to Saturn)", "sustained": True,
        "accel_g": r(ds2_s_f / (1690e3 * G0)), "exhaust_km_s": ds2_s_v,
        "wet_t": 1690.0, "dry_t": 1690.0 - 807.0 - 11.0,
        "fuels": ["deuterium", "helium-3", "hydrogen"], "refuel": ["docking"],
        "crew": 6, "cargo_t": 172.0,
        "engines": [{"engine": "paper.discovery-ii-saturn"}],
    },
    "paper.hope-mtf": {
        "name": "HOPE fusion ship to Callisto (D-He3)", "sustained": True,
        "accel_g": r(hope_mtf_f / (691.892e3 * G0)), "exhaust_km_s": hope_mtf_v,
        "wet_t": 691.892, "dry_t": 691.892 - 142.0,
        "fuels": ["deuterium", "helium-3", "hydrogen"], "refuel": ["docking"],
        "crew": 6, "cargo_t": 163.933,
        "engines": [{"engine": "paper.hope-mtf"}],
    },
    "paper.hope-nep": {
        "name": "HOPE nuclear electric ship to Callisto (piloted)", "sustained": True,
        "accel_g": r(hope_mpd_f / (1072.336e3 * G0)), "exhaust_km_s": hope_mpd_v,
        "wet_t": 1072.336, "dry_t": 1072.336 - 577.0,
        "fuels": ["hydrogen"], "refuel": ["docking"],
        "crew": 6, "cargo_t": 57.885,
        "power": [{"id": "reactor", "kind": "fission-reactor", "electric_w": hope_mpd_draw}],
        "engines": [{"engine": "paper.hope-mpd"}],
    },
    "paper.longshot": {
        "name": "Longshot probe to Alpha Centauri", "sustained": True,
        "accel_g": r(ls_f / (396.414e3 * G0)), "exhaust_km_s": ls_v,
        "wet_t": 396.414, "dry_t": r(396.414 - 264.276),
        "fuels": ["deuterium", "helium-3"], "refuel": ["docking"],
        "crew": 0,
        "power": [{"id": "reactor", "kind": "fission-reactor", "electric_w": 300e3, "mass_t": 6.4}],
        "loads": [{"name": "laser link (one of six)", "electric_w": 250e3, "role": "sensor"}],
        "engines": [{"engine": "paper.longshot-icf"}],
    },
    "paper.orion-mars": {
        "name": "Orion 10 m Mars ship", "sustained": False,
        "accel_g": 0.3, "exhaust_km_s": orion_v,
        "wet_t": 599.0, "dry_t": r(599.0 / ORION_RATIO),
        "fuels": ["pulse units"], "refuel": ["docking"],
        "crew": 8,
        "engines": [{"engine": "paper.orion-10m"}],
    },
    "paper.dra5-ntr": {
        "name": "DRA 5.0 nuclear thermal Mars ship (crewed)", "sustained": False,
        "accel_g": r(3 * 25000 * LBF_N / (333e3 * G0)), "exhaust_km_s": isp_km_s(900),
        "wet_t": 333.0, "dry_t": r(333.0 - 180.2),
        "fuels": ["hydrogen"], "refuel": ["docking"],
        "crew": 6, "cargo_t": 51.3,
        "power": [{"id": "arrays", "kind": "solar-array", "electric_w": 50e3}],
        "loads": [{"name": "crew life support", "electric_w": 30e3, "role": "life support"},
                  {"name": "zero boil-off cryocoolers", "electric_w": 15e3},
                  {"name": "communications", "electric_w": 5e3, "role": "sensor"}],
        "engines": [{"engine": "paper.pewee-25klbf", "count": 3}],
    },
    "paper.ffre-callisto": {
        "name": "Fission fragment ship to Callisto", "sustained": True,
        "accel_g": r(ffre_f / (303.1e3 * G0)), "exhaust_km_s": ffre_v,
        "wet_t": 303.1, "dry_t": 303.1 - 4.0,
        "fuels": ["uranium dioxide"], "refuel": ["docking"],
        "crew": 6, "cargo_t": 58.0,
        "engines": [{"engine": "paper.ffre"}],
    },
}


RECIPES = {
    "paper.discovery-ii": {
        "study": "Discovery II", "year": 2001, "by": "NASA Glenn Research Center",
        "src": ["williams2005"], "mark": "not yet built",
        "advances": ["fusion-torch"],
        "study_says": {"from": "Earth", "to": "Jupiter", "trip_days": 118, "distance_au": 4.70, "note": "one way, from and to C3 ~ 0 (no spiral counted)"},
        "derived": "dry_t is IMLEO less the main-impulse hydrogen (807 mt) and the D-He3 (11 mt); reaction control, reserve and residuals stay aboard",
        "about": "A 240 m crewed fusion ship designed to make the voyages of the film 2001 real: six crew, 172 t of payload, a 4.8 GW jet.",
    },
    "paper.discovery-ii-saturn": {
        "study": "Discovery II", "year": 2001, "by": "NASA Glenn Research Center",
        "src": ["williams2005"], "mark": "not yet built",
        "advances": ["fusion-torch"],
        "study_says": {"from": "Earth", "to": "Saturn", "trip_days": 212, "distance_au": 9.57, "note": "one way; the same ship geared to a faster exhaust and less thrust"},
        "derived": "as the Jupiter ship; IMLEO 1,690 mt (Table 2 prints 1,699 for Saturn, but the text says the masses were fixed for both)",
        "about": "The same Discovery II, geared for Saturn: the same jet power spent on a faster, thinner exhaust.",
    },
    "paper.hope-mtf": {
        "study": "HOPE (Human Outer Planet Exploration), magnetised target fusion", "year": 2003, "by": "NASA Marshall Space Flight Center",
        "src": ["adams2003-callisto"], "mark": "not yet built",
        "advances": ["paper.mtf-fusion"],
        "study_says": {"from": "Earth-Moon L1", "to": "Callisto", "trip_days": 249, "note": "outbound; 183 days at Callisto, about 652 days in all, departing 27 Apr 2045"},
        "derived": "dry_t is IML1 less the 142,000 main propellant; the reaction control propellant stays aboard",
        "about": "NASA's 2003 study of a crewed base on Callisto, flown by a fusion drive of 77,000 seconds.",
    },
    "paper.hope-nep": {
        "study": "HOPE, nuclear electric (MSR-LMR-MPD, piloted)", "year": 2003, "by": "NASA Marshall Space Flight Center",
        "src": ["adams2003-callisto"], "mark": "not yet built",
        "advances": ["paper.mpd-thruster", "fission-reactor"],
        "study_says": {"from": "Earth-Moon L1", "to": "Callisto", "trip_days": 832, "note": "flight time, thrusting about 812 days; about 120 days at Callisto, about 1,661 days in all"},
        "derived": "dry_t is IML1 less the 577,000 propellant",
        "about": "The same Callisto mission on 37 MW of reactor electricity and magnetoplasmadynamic thrusters: the slow, honest way.",
    },
    "paper.longshot": {
        "study": "Project Longshot", "year": 1988, "by": "US Naval Academy for NASA",
        "src": ["beals1988"], "mark": "not yet built",
        "advances": ["paper.icf-pulse", "fission-reactor"],
        "study_says": {"to": "Alpha Centauri", "distance_ly": 4.3, "trip_days": 36525, "note": "about 100 years, turning over at 71.27 years and braking into orbit about Beta Centauri (the report says 4.3 light years, sec. 3.4); cruising near 5% of light speed; no crew"},
        "derived": "dry_t is the overall total (396.414 t) less the fuel (264.276 t); the OCR of the weight table is garbled",
        "about": "A 1988 student design for NASA: an uncrewed fusion probe that reaches Alpha Centauri in about a century.",
    },
    "paper.orion-mars": {
        "study": "Orion nuclear pulse, 10 m module, Mars by orbital rendezvous", "year": 1964, "by": "General Atomic for NASA Marshall",
        "src": ["ga4891", "ga5009-3"], "mark": "not yet built",
        "advances": ["paper.orion-pulse"],
        "study_says": {"from": "Earth", "to": "Mars", "trip_days": 450, "note": "round trip; mission velocity 72,850 ft/s"},
        "derived": "dry_t from the rocket equation: 599,000 kg at departure over exp(22.2 km/s / 17.65 km/s); thrust is the 0.3 g start-up",
        "about": "Pushed by a bomb a second against a 10 m plate: the 1964 design for a Saturn V-launched Mars ship with eight crew.",
    },
    "paper.dra5-ntr": {
        "study": "NASA Mars Design Reference Architecture 5.0, nuclear thermal crew vehicle (4-launch, twin drop tanks)", "year": 2009, "by": "NASA Glenn Research Center",
        "src": ["borowski2009", "borowski2012"], "mark": "not yet built",
        "advances": ["nuclear-thermal"],
        "study_says": {"from": "Earth", "to": "Mars", "trip_days": 180, "dv_km_s": {"trans_mars": 3.992, "mars_capture": 1.771, "trans_earth": 1.562}, "note": "180 days each way, about 540 at Mars, about 900 in all; ideal delta-v without gravity losses"},
        "derived": "dry_t is the ~333 t IMLEO less the ~180.2 t of usable hydrogen; the real ship drops its two drop tanks after the Earth departure burn, which a single-stage record cannot (it carries their mass to Mars). The arrays are held at their 50 kWe at Mars rather than following the starlight",
        "about": "Three Rover-descended 25,000 lbf nuclear engines on one stage, six crew, Mars and back in about 900 days: NASA's own reference design.",
    },
    "paper.ffre-callisto": {
        "study": "Fission fragment rocket engine, crewed Callisto mission", "year": 2012, "by": "NASA Marshall (NIAC phase I)",
        "src": ["werka2012"], "mark": "not yet built",
        "advances": ["paper.fission-fragment"],
        "study_says": {"from": "Earth-Moon L1", "to": "Jupiter", "trip_days": 2100, "note": "'over 2100 days were needed to reach the orbit of Jupiter'"},
        "derived": "dry_t is the wet mass less the 4.0 mT of uranium dioxide dust",
        "about": "Fission fragments thrown straight out at 1.7% of light speed: half a million seconds of impulse, and 10 lbf of thrust.",
    },
}

# Studies the pack records but that SSE4 cannot fly yet.
WAITING = {
    "paper.starshot": {
        "study": "Breakthrough Starshot point design", "year": 2018, "by": "Parkin Research for Breakthrough Initiatives",
        "src": ["parkin2018"], "mark": "not yet built",
        "advances": ["light-sail"],
        "figures": {"laser_w": 2.0e11, "optics_m": 2700.0, "wavelength_um": 1.06, "sail_m": 4.1, "sailcraft_g": 3.6, "payload_g": 1.0, "speed_c": 0.2, "push_s": 550.0, "peak_g": 14900.0, "stored_wh": 6.3e10, "kinetic_wh": 1.9e9},
        "waits_on": "RK5: a beamed leg that depends on its sender (the travel thread)",
        "about": "A 3.6 g sail thrown to a fifth of light speed by 200 GW of laser in nine minutes.",
    },
}

# ---- Advances (provides.other.tech.advances, joining base-tech's) ----------

ADVANCES = [
    {"id": "paper.nuclear-light-bulb", "name": "Nuclear light bulb", "level": 5,
     "sets": {"exhaust_km_s": nlb_v}, "fuel": "hydrogen", "family": "near-ceiling", "part": "drive",
     "mark": "not yet built", "src": "mclafferty1968",
     "note": "uranium plasma held in a quartz bulb, its light heating hydrogen outside: 1870 s, 92,000 lb, 4600 MW"},
    {"id": "paper.open-cycle-gas-core", "name": "Open-cycle gas core", "level": 5,
     "sets": {"exhaust_km_s": ocgc_v}, "fuel": "hydrogen", "family": "near-ceiling", "part": "drive",
     "mark": "not yet built", "src": "ragsdale1991",
     "note": "a uranium plasma ball inside flowing hydrogen, some of it lost with the exhaust: 5200 s from 5,000 MW"},
    {"id": "paper.orion-pulse", "name": "Nuclear pulse (Orion)", "level": 4,
     "sets": {"exhaust_km_s": orion_v}, "family": "engine", "part": "drive",
     "mark": "not yet built", "src": "ga4891",
     "least_size": {"v": 53.9, "unit": "kg", "what": "fissile metal per pulse unit at most: a bare sphere of 93 to 94% U-235 goes critical at 53.9 kg, so a fission charge needs a share of that and cannot be made arbitrarily small; plutonium and a reflector lower it", "src": "gwin-mee1959", "check": "unverified"},
     "note": "within the reach of 1960s engineering in General Atomic's view; never built (1963 test ban)"},
    {"id": "paper.mtf-fusion", "name": "Magnetised target fusion drive", "level": 5,
     "sets": {"exhaust_km_s": hope_mtf_v, "plant_w_per_kg": r(1000 / 0.0193)}, "family": "engine", "part": "drive",
     "mark": "not yet built", "src": "adams2003-callisto",
     "note": "plant_w_per_kg from 0.0193 kg per kW of jet (derived)"},
    {"id": "paper.icf-pulse", "name": "Inertial fusion pulse drive", "level": 6,
     "sets": {"exhaust_km_s": ls_v}, "family": "engine", "part": "drive",
     "mark": "not yet built", "src": "beals1988"},
    {"id": "paper.fission-fragment", "name": "Fission fragment rocket", "level": 5,
     "sets": {"exhaust_km_s": ffre_v, "waste_per_jet": r((1.0e9 - jet_w(ffre_f, ffre_v)) / jet_w(ffre_f, ffre_v), 3), "radiator_k": 1350.0},
     "family": "engine", "part": "drive",
     "mark": "not yet built", "src": "werka2012",
     "note": "waste_per_jet = heat over jet at 1000 MW thermal (derived); most of it leaves at 1350 K"},
    {"id": "paper.mpd-thruster", "name": "Megawatt magnetoplasmadynamic thruster", "level": 5,
     "sets": {"exhaust_km_s": hope_mpd_v}, "family": "engine", "part": "electric drive",
     "mark": "not yet built", "src": "adams2003-callisto",
     "note": "electric: thrust is set by the power plant"},
    {"id": "paper.brayton-reactor", "name": "Space fission reactor with Brayton conversion", "level": 5,
     "sets": {"plant_w_per_kg": 20.0}, "family": "engine", "part": "power plant",
     "mark": "not yet built", "src": "johnson2003",
     "note": "Prometheus's goal of 50 kg per kWe; two 100 kWe Brayton units designed for the Jupiter Icy Moons Orbiter (mason2003), cancelled 2005"},
]


def tours():
    """Tours (top-level design 3.2): ordered stops, each a bookmark (playbook
    design section 11, sse.bookmark/1) with a caption and something to try.
    They play once tours and bookmarks are built (PB2b)."""
    def stop(ship, caption, try_=None, to="Jupiter", view=None):
        b = {"schema": "sse.bookmark/1", "scope": "fixture:sol",
             "instrument": "voyages", "voyage": {"ship": ship, "from": "Earth", "to": to}}
        if view:
            b["view"] = view
        s = {"bookmark": b, "caption": caption}
        if try_:
            s["try"] = try_
        return s

    return [
        {"schema": "sse.tour/1", "id": "paper.tour.torchships-are-hard", "title": "Why torchships are hard",
         "stops": [
             stop("paper.discovery-ii", "Discovery II puts 4.8 billion watts into its exhaust, a large power station's output, for 28 kN of thrust. On 1,690 tonnes that is less than two thousandths of a g.", "Open the power line and read the jet power."),
             stop("paper.discovery-ii", "Thrust times exhaust speed is twice the jet power. Faster exhaust saves fuel, but at fixed power it gives less push: the same engine geared for Saturn has a quarter less thrust.", "Switch to the Saturn setting and compare the acceleration.", to="Saturn"),
             stop("paper.hope-nep", "With reactor electricity instead of fusion, 37 MW buys about a ninth of the HOPE fusion ship's push. The same trip to Callisto takes more than three times as long.", "Plan the trip and compare the days."),
             stop("paper.ffre-callisto", "Half a million seconds of impulse needs almost no fuel, but 10 lbf of thrust on 300 tonnes takes more than five years to Jupiter. Exhaust speed alone does not make a torch: power does.", "Read the acceleration in g."),
         ]},
        {"schema": "sse.tour/1", "id": "paper.tour.radiators", "title": "How big the radiators really are",
         "stops": [
             stop("paper.ffre-callisto", "The fission fragment ship throws away 700 of every 1000 MW it makes. At 1350 K the panels shed that through 1954 square metres.", "Look at the heat line in the power books."),
             stop("paper.ffre-callisto", "Its magnets must stay at 120 K. Shedding only 0.05 MW at that temperature needs 2247 square metres, more than the 699 MW panels: radiated power goes as the fourth power of temperature.", "Halve a radiator's temperature and count how much bigger it must be."),
             stop("paper.hope-mtf", "The HOPE fusion ship sheds 176.5 MW at 1250 K and 33.9 MW at 800 K. The cooler load is a fifth the size and needs the bigger panel.", None),
         ]},
        {"schema": "sse.tour/1", "id": "paper.tour.nothing-hides", "title": "Why nothing hides",
         "stops": [
             stop("paper.ffre-callisto", "Seen in infrared, 1954 square metres at 1350 K glow orange-hot. Heat has to leave, and wherever it goes it can be seen.", None, view={"band": "infrared"}),
             stop("paper.discovery-ii", "A fusion exhaust carrying 4.8 GW is brighter still. A cloak would have to say where that light and heat go, and in SSE4 it must.", "Try marking a ship as hidden and read what the GM view says.", view={"band": "infrared"}),
         ]},
    ]


def build():
    for sid, s in SHIPS.items():
        assert sid in RECIPES, sid
        for m in s.get("engines", []):
            assert m["engine"] in ENGINES, m
    for eid in ENGINES:
        assert eid in ENGINE_NOTES, eid
        for src in ENGINE_NOTES[eid]["src"]:
            assert src in SOURCES, src
    adv_ids = {a["id"] for a in ADVANCES}
    for rid, rc in list(RECIPES.items()) + list(WAITING.items()):
        for src in rc["src"]:
            assert src in SOURCES, src
        for a in rc["advances"]:
            assert a in adv_ids or not a.startswith("paper."), (rid, a)
    for a in ADVANCES:
        assert a["src"] in SOURCES, a
    # The two Discovery II settings are one jet power (Table 2: 4,830 MW).
    for f, v in [(ds2_j_f, ds2_j_v), (ds2_s_f, ds2_s_v)]:
        assert abs(jet_w(f, v) - 4.83e9) / 4.83e9 < 0.005, jet_w(f, v)
    # The fission fragment engine's 11% (Werka p. 7).
    assert 0.10 < jet_w(ffre_f, ffre_v) / 1e9 < 0.12
    engines = {}
    for eid, e in ENGINES.items():
        engines[eid] = e
    pack = {
        "format": "sse.pack/1",
        "id": "sse.paper-spacecraft",
        "version": "0.1.0",
        "kind": ["universe"],
        "kernel_range": "^0.1",
        "licence": "Apache-2.0",
        "provides": {"other": {
            "about": {
                "label": "Paper spacecraft",
                "text": "Engines and ships that were designed in detail and never built, each from its own study: Discovery II, NASA's DRA 5.0 nuclear thermal Mars ship, NASA's HOPE ships to Callisto, Project Longshot, the Orion nuclear pulse ship, the fission fragment rocket, gaseous core engines and the Starshot sail. They fly on SSE4's own physics, marked not yet built.",
                "fictional": False,
                "note": "Track RK, package RK6 (handover/design/SSE4-atomic-rockets-plan.md section 6). Published on Explorers, not built into SSE4. Built by scripts/build-paper-spacecraft.py; do not edit by hand.",
            },
            "credit": [
                "Figures from NASA, NASA contractor and US Naval Academy reports (US Government works) and from Parkin 2018 (arXiv:1805.01306); each figure names its study. Our selection, arrangement and words are Apache-2.0.",
            ],
            "engines": engines,
            "engine_notes": ENGINE_NOTES,
            "nav_ships": {
                "about": "Ships from design studies, fitted with their engines so the power books set the drive. Each has a recipe saying what the study claimed.",
                "ships": SHIPS,
            },
            "recipes": RECIPES,
            "waiting": WAITING,
            "tech": {"advances": ADVANCES, "sources": SOURCES},
            "tours": tours(),
            "sources": SOURCES,
        }},
    }
    return pack


def main():
    pack = build()
    text = json.dumps(pack, indent=1, ensure_ascii=False) + "\n"
    if "--check" in sys.argv:
        with open(OUT, encoding="utf-8") as f:
            if f.read() != text:
                sys.exit("store-packs/paper-spacecraft/pack.json is out of date: run scripts/build-paper-spacecraft.py")
        return
    os.makedirs(os.path.dirname(OUT), exist_ok=True)
    with open(OUT, "w", encoding="utf-8") as f:
        f.write(text)
    print(f"wrote {OUT}: {len(SHIPS)} ships, {len(ENGINES)} engines, {len(ADVANCES)} advances, {len(pack['provides']['other']['tours'])} tours")


if __name__ == "__main__":
    main()
