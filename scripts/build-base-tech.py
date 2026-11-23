#!/usr/bin/env python3
"""Build packs/base-tech/pack.json: the future tech base pack (track FT,
package FT1; design handover/design/SSE4-future-tech-design.md sections 3 to 5).

Technology is how far each limit on building things has been pushed. The pack
holds plain data, read by general machinery (FT2 is the kernel's):

  limits     the ten standing problems, each with the law behind it and the
             figures it holds back
  figures    numbers with a unit that a sizing block reads, each with cited
             marks: built (flown or in service), allowed (no known law forbids
             it, nobody has built it) and ceiling (where a named law stops it).
             `better` is the way an advance pushes the figure; past the
             ceiling in that direction is fictional by arithmetic
  ladder     the neutral ladder, real history on its lower rungs, and how a
             figure improves past an advance's home level (half as much
             better again per level, until its next cited mark)
  families   part families (design 5.2): how much worse an advance's own
             figure is per level short of home (engines a seventh, bulk
             materials a fifth, stores a third, near-ceiling figures two-thirds
             of the way, plain mass the same, least-size devices a floor);
             only fictional and unknown devices scale the part's size (ten
             times per level short, half per level ahead for two levels)
  advances   real and near-real advances: what each sets, its family, the
             figures physics fixes, its home level and the mark it earns

Numbers that follow from the master data sheet (c, g0, strength for weight of
its materials) are worked out from it here, never typed (MP22; this builder is
SHEET_FED in the audit and check.sh runs --check). Atomic Rockets
(projectrho.com) was used only as a map to the primary sources cited; nothing
is cited from it or copied. Run after editing:

    python3 scripts/build-base-tech.py
"""
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "packs/base-tech/pack.json"
SHEET = json.loads((ROOT / "packs/base-reference/pack.json").read_text())["provides"]["reference"]
REF = {c["id"]: c["value"]["v"] for c in SHEET["constants"]}
MIX = {m["id"]: m for m in SHEET["mixtures"]}
C = REF["c"]
G0 = REF["g0"]
# The international pound, exact by definition (1959); SNAP-10A's mass is given in pounds.
LB_KG = 0.45359237
SNAP10A_W_KG = round(500.0 / (950.0 * LB_KG), 2)
ELEMENTS = {e["id"]: e for e in SHEET["elements"]}
H2O = next(x for x in SHEET["substances"] if x["id"] == "h2o")
# Hydrogen burned with oxygen, per kilogram of liquid water made (the higher
# heating value): the sheet's heat of formation over water's molar mass.
M_H2O = (2 * ELEMENTS["h"]["atomic_weight"]["v"] + ELEMENTS["o"]["atomic_weight"]["v"]) * 1e-3
HYDROLOX_J_KG = round(-H2O["chemical"]["states"]["liquid"]["dhf"]["v"] / M_H2O, -5)
# D-He3 fusion: 3.7 MeV per nucleon (Ball 2019), in J/kg from the sheet's e and m_u.
DHE3_J_KG = round(3.7e6 * REF["e"] / REF["m_u"], -12)


def isp_km_s(isp_s):
    """Exhaust speed in km/s from a specific impulse in seconds (Isp times g0)."""
    return round(isp_s * G0 / 1e3, 3)


def strength_for_weight(mid):
    """A sheet material's tensile strength over its density, J/kg."""
    m = MIX[mid]
    return round(m["tensile_strength"]["v"] / m["density"]["v"], -3)


def mark(kind, v, what, src, check=None, law=None, note=None, sheet=None):
    d = {"mark": kind, "v": v, "what": what, "src": src}
    for k, x in (("check", check), ("law", law), ("note", note), ("sheet", sheet)):
        if x is not None:
            d[k] = x
    return d


SOURCES = {
    "l3harris-rl10": "L3Harris 2024, RL10 engine specification sheet: RL10B-2 specific impulse 465.5 s, thrust 24,750 lbf. https://l3harris.com/sites/default/files/2024-07/l3harris-ar-rl10-spec-sheet.pdf",
    "nasem2021": "National Academies of Sciences, Engineering, and Medicine 2021, Space Nuclear Propulsion for Human Mars Exploration, chapter 2, Table 2.1: Rover/NERVA reactor tests, Pewee 1 about 875 s (vacuum, ideal expansion, from measured propellant temperature). https://www.nationalacademies.org/read/25977/chapter/4",
    "williams2005": "Williams, C. H., Dudzinski, L. A., Borowski, S. K. and Juhasz, A. J. 2005, Realizing '2001: A Space Odyssey': piloted spherical torus nuclear fusion propulsion, NASA/TM-2005-213559 (AIAA 2001-3805): D-He3, specific impulse 35,435 s (Jupiter mission), jet power 4,830 MW, specific power 8.62 kW/kg",
    "adams2003": "Adams, R. B. et al. 2003, Conceptual design of in-space vehicles for human exploration of the outer planets, AIAA 2003-4527 (NTRS 20030065984): D-He3 magnetised target fusion, specific impulse 77,000 s",
    "ball2019": "Ball, J. 2019, Maximizing specific energy by breeding deuterium, arXiv:1908.00834, Table 1: D-He3 3.7 MeV per nucleon",
    "openstax-up3": "Ling, S. J., Sanny, J. and Moebs, W. 2016, University Physics volume 3, OpenStax, section 10.5: fission of 1.00 kg of U-235 releases 8.21e13 J. https://openstax.org/books/university-physics-volume-3/pages/10-5-fission",
    "pattarini2020": "Pattarini, J. M. et al. 2020, Artemis sustained translational acceleration limits: human tolerance evidence from Apollo to ISS, NASA/TM-20205008196: seated, long duration, 4.91 m/s2 from 120 s on (head-to-foot axis)",
    "nasa-std-3001-v2": "NASA-STD-3001 Volume 2, requirement V2 6064, sustained translational acceleration limits, Tables 6.5-1 to 6.5-6 (tables not read)",
    "globus-hall2017": "Globus, A. and Hall, T. 2017, Space settlement population rotation tolerance, NSS Space Settlement Journal: up to 4 rpm (56 m radius at 1 g) acceptable, up to 6 rpm for residents. https://nss.org/wp-content/uploads/NSS-JOURNAL-Space-Settlement-Population-Rotation-Tolerance.pdf",
    "nasa-iss-atcs": "NASA and Boeing, International Space Station Active Thermal Control System overview: 70 kW rejected (35 kW per loop), ammonia supply 2.8 C. https://www.nasa.gov/wp-content/uploads/2021/02/473486main_iss_atcs_overview.pdf",
    "santarius-logan-1998": "Santarius, J. F. and Logan, B. G. 1998, Generic magnetic fusion rocket model, Journal of Propulsion and Power 14, 519 (abstract read; full text not read)",
    "lubin2016": "Lubin, P. 2016, A roadmap to interstellar flight, NASA NIAC phase I final report (NTRS 20190001185): a photon thrust of 3.3 nN per watt emitted",
    "nasa-std-3001-v1": "NASA-STD-3001 Volume 1 revision C 2023, requirement V1 4030: career effective dose below 600 mSv. https://standards.nasa.gov/",
    "ochmo-tb-020": "NASA OCHMO-TB-020 revision E 2022, radiation protection technical brief: at least 20 g/cm2 of shielding for missions longer than six months (solar particle events)",
    "koontz2016": "Koontz, S. 2016, The International Space Station ionizing radiation environment, NASA JSC (NTRS 20160011221): median shielding 40 to 50 g/cm2 in the US Lab",
    "lynch2023": "Lynch, C. et al. 2023, Logistics rates and assumptions for future human spaceflight missions beyond LEO, AIAA ASCEND 2023 (NTRS 20230013896): oxygen 0.89, drinking water 2.79, food as shipped 2.39 kg per crew member a day",
    "nasa-water2023": "NASA 2023 (20 June), NASA achieves water recovery milestone on International Space Station: 98% total water recovery with the brine processor. https://www.nasa.gov/missions/station/iss-research/nasa-achieves-water-recovery-milestone-on-international-space-station",
    "peng2008": "Peng, B. et al. 2008, Nature Nanotechnology 3, 626: single carbon nanotube shells near 100 GPa (as summarised; not read)",
    "doe-snap": "US DOE Energy Technology Engineering Center, System for Nuclear Auxiliary Power (SNAP) overview: SNAP-10A (1965) made at least 500 W (over 600 at first) and weighed under 950 lb with instruments and shielding. https://www.energy.gov/etec/system-nuclear-auxiliary-power-snap-overview",
    "sp100": "Manvi, R. and Fujita, T. 1986, SP-100 program users handbook, JPL D-2185 issue 2: 100 kWe at end of life for about 3,000 kg (design; never flown). https://www.osti.gov/servlets/purl/10184673",
    "ft-design-5.2": "SSE4 future tech design, section 5.2 (accepted by Alex 23 Nov 2026): first working generation against the mature one, by family of part; figures checked in its section 14.1 (V-2 about 22 against Merlin 1D over 150; Jumo 004 1.2 against EJ200 about 9; carbon fibre 0.5 to 1 GPa early, 3.5 in 1971, 7.0 now; nanotube tubes 100 GPa against fibre 4.2; lithium-ion 80 Wh/kg in 1991, about three times now)",
    "sheet": "the master data sheet (pack base-reference): the value is worked out from its rows",
    "law": "a law of physics, named on the mark",
}

# The figures. `reads` names where a sizing block reads the figure today, or
# "FT2" where the kernel package adds the reader.
FIGURES = [
    # 1 Heat has to leave
    {"id": "radiator_k", "limit": "heat", "plain": "radiator temperature", "unit": "K", "better": "higher",
     "reads": "base-ships ship_design.drive_heat[].radiator_k",
     "marks": [mark("built", 276.0, "International Space Station radiators: ammonia supplied at 2.8 C", "nasa-iss-atcs", note="the coolant's supply temperature, not the panel's"),
               mark("allowed", 1500.0, "a fusion drive's structure radiating its side-reaction heat", "santarius-logan-1998", check="unverified")]},
    {"id": "waste_per_jet", "limit": "heat", "plain": "waste heat per watt of jet", "unit": "1", "better": "lower",
     "reads": "base-ships ship_design.drive_heat[].waste_per_jet",
     "marks": [mark("built", 0.0, "a chemical rocket: the heat leaves with the exhaust", "sheet"),
               mark("allowed", 0.1, "a D-He3 fusion drive: neutrons and bremsstrahlung heat the engine", "santarius-logan-1998", check="unverified")]},
    {"id": "heat_sent_nowhere", "limit": "heat", "plain": "share of waste heat that leaves without being radiated or carried", "unit": "1", "better": "higher",
     "reads": "FT2",
     "marks": [mark("ceiling", 0.0, "a heat sink that sends heat nowhere", "law", law="second law of thermodynamics")]},
    # 2 Thrust needs something to throw
    {"id": "exhaust_km_s", "limit": "thrust", "plain": "exhaust speed", "unit": "km s-1", "better": "higher",
     "reads": "ship exhaust_km_s (sse-nav Ship)",
     "marks": [mark("built", isp_km_s(465.5), "RL10B-2 hydrogen-oxygen engine, 465.5 s", "l3harris-rl10"),
               mark("built", isp_km_s(875.0), "Pewee 1 nuclear thermal reactor, about 875 s (ground test, never flown)", "nasem2021"),
               mark("allowed", isp_km_s(35435.0), "D-He3 spherical torus fusion ship, 35,435 s", "williams2005"),
               mark("allowed", isp_km_s(77000.0), "D-He3 magnetised target fusion ship, 77,000 s", "adams2003"),
               mark("ceiling", round(C / 1e3, 3), "the speed of light", "law", law="special relativity", sheet="constants/c")]},
    {"id": "unthrown_n_per_w", "limit": "thrust", "plain": "thrust per watt with nothing thrown", "unit": "N W-1", "better": "higher",
     "reads": "FT2",
     "marks": [mark("built", 1.0 / C, "light pressure: a photon rocket or a lamp, 3.3 N per GW", "lubin2016", sheet="constants/c"),
               mark("ceiling", 1.0 / C, "a reactionless drive above 1/c gives out more energy than it takes in", "law", law="conservation of momentum and energy", sheet="constants/c")]},
    # 3 Energy has weight
    {"id": "fuel_j_per_kg", "limit": "energy", "plain": "energy per kilogram of fuel", "unit": "J kg-1", "better": "higher",
     "reads": "FT2 (the fuel's energy beside base-nav fuels)",
     "marks": [mark("built", HYDROLOX_J_KG, "hydrogen burned with oxygen, per kilogram of liquid water made", "sheet", sheet="substances/h2o", note="the higher heating value; about 13 MJ/kg if the water leaves as vapour"),
               mark("built", 8.21e13, "U-235 fission, 200 MeV a fission", "openstax-up3"),
               mark("allowed", DHE3_J_KG, "D-He3 fusion, 3.7 MeV per nucleon", "ball2019"),
               mark("ceiling", C * C, "matter with antimatter, all of the mass", "law", law="mass-energy equivalence", sheet="constants/c")]},
    {"id": "plant_w_per_kg", "limit": "energy", "plain": "power per kilogram of power plant and drive", "unit": "W kg-1", "better": "higher",
     "reads": "FT2",
     "marks": [mark("built", SNAP10A_W_KG, "SNAP-10A, the only reactor the US flew: 500 W for under 950 lb", "doe-snap"),
               mark("allowed", round(100e3 / 3000.0), "SP-100 space reactor design, 100 kWe for about 3,000 kg", "sp100"),
               mark("allowed", 8620.0, "D-He3 spherical torus fusion ship, drive and plant together", "williams2005")]},
    # 4 Crews break under hard acceleration
    {"id": "crew_g", "limit": "acceleration", "plain": "sustained acceleration a crew can take", "unit": "g", "better": "higher",
     "reads": "ship g_limit (sse-nav Ship); the species' tolerance",
     "marks": [mark("built", round(4.91 / G0, 2), "seated, long duration, head-to-foot axis", "pattarini2020", note="chest-to-back tolerance is higher; NASA-STD-3001 V2 6064 tables not read")]},
    {"id": "inertia_screened", "limit": "acceleration", "plain": "share of the ship's acceleration the crew does not feel", "unit": "1", "better": "higher",
     "reads": "FT2",
     "marks": [mark("ceiling", 0.0, "inertial compensators", "law", law="equivalence principle: inertia cannot be screened")]},
    # 5 Crews need weight
    {"id": "spin_rpm", "limit": "weight", "plain": "highest comfortable spin", "unit": "rpm", "better": "higher",
     "reads": "FT2 (the spin ring part)",
     "marks": [mark("allowed", 4.0, "anyone, after a short adaptation", "globus-hall2017"),
               mark("allowed", 6.0, "trained residents", "globus-hall2017")]},
    {"id": "weight_without_spin_g", "limit": "weight", "plain": "weight felt without spin or thrust", "unit": "g", "better": "higher",
     "reads": "FT2",
     "marks": [mark("ceiling", 0.0, "gravity plates", "law", law="equivalence principle; no negative mass is known")]},
    # 7 Materials only bear so much
    {"id": "strength_for_weight", "limit": "materials", "plain": "strength for weight", "unit": "J kg-1", "better": "higher",
     "reads": "Look up: tensile_strength over density of a sheet material",
     "marks": [mark("built", strength_for_weight("steel-wire"), "high-strength steel wire", "sheet", sheet="mixtures/steel-wire"),
               mark("built", strength_for_weight("aramid"), "aramid fibre", "sheet", sheet="mixtures/aramid"),
               mark("built", strength_for_weight("pbo-fibre"), "PBO fibre", "sheet", sheet="mixtures/pbo-fibre"),
               mark("allowed", strength_for_weight("carbon-nanotube"), "carbon nanotube, the best measured shell; not made at length", "sheet", sheet="mixtures/carbon-nanotube"),
               mark("ceiling", 7.7e7, "an ideal nanotube shell, about 100 GPa over 1,300 kg/m3: the carbon bond itself", "peng2008", check="unverified", law="chemical bond energy")]},
    # 8 Radiation gets through
    {"id": "shield_g_cm2", "limit": "radiation", "plain": "shielding round the crew", "unit": "g cm-2", "better": "lower",
     "reads": "ship shield_g_cm2 (sse-nav Ship)",
     "marks": [mark("built", 45.0, "International Space Station, US Lab median 40 to 50", "koontz2016"),
               mark("allowed", 20.0, "a long mission's storm shelter, NASA's guidance", "ochmo-tb-020")]},
    {"id": "career_dose_sv", "limit": "radiation", "plain": "career radiation dose allowed", "unit": "Sv", "better": "higher",
     "reads": "FT2 (the shielding part's target)",
     "marks": [mark("built", 0.6, "NASA's career limit", "nasa-std-3001-v1")]},
    # 9 Crews need air, food and time
    {"id": "supplies_kg_per_day", "limit": "supplies", "plain": "oxygen, drinking water and food per person a day, before recycling", "unit": "kg", "better": "lower",
     "reads": "FT2 (the stores part)",
     "marks": [mark("built", round(0.89 + 2.79 + 2.39, 2), "oxygen 0.89, drinking water 2.79, food 2.39", "lynch2023")]},
    {"id": "water_recycled", "limit": "supplies", "plain": "share of water recovered", "unit": "1", "better": "higher",
     "reads": "FT2 (the stores part)",
     "marks": [mark("built", 0.98, "International Space Station with its brine processor", "nasa-water2023"),
               mark("ceiling", 1.0, "all of it", "law", law="conservation of mass")]},
    # 10 Anything warm can be seen
    {"id": "light_hidden", "limit": "seen", "plain": "share of a construct's light that goes nowhere", "unit": "1", "better": "higher",
     "reads": "FT2",
     "marks": [mark("ceiling", 0.0, "a cloak that does not say where the light goes", "law", law="conservation of energy; second law of thermodynamics")]},
]

LIMITS = [
    ("heat", 1, "Heat has to leave", "Second law; radiated power goes as area times temperature to the fourth", "radiators, heat shields, glow in infrared", "heat sinks that send it nowhere"),
    ("thrust", 2, "Thrust needs something to throw", "Conservation of momentum; the rocket equation", "tanks, mass ratio, staging, lifting off a world", "reactionless and gravity drives"),
    ("energy", 3, "Energy has weight", "Mass-energy; energy per kilogram of fuel, power per kilogram of plant", "fuel tanks, reactor size, range", "anything richer than annihilation"),
    ("acceleration", 4, "Crews break under hard acceleration", "The crew's physiology; inertia cannot be screened (equivalence principle)", "trip times, engine size, launch track length", "inertial compensators"),
    ("weight", 5, "Crews need weight", "Spin gravity: radius is g over spin rate squared; comfort caps the spin rate", "spin rings and tumbling pairs", "gravity plates"),
    ("light-speed", 6, "Nothing outruns light", "Special relativity", "travel between stars, message delay, a crew's age against home", "every FTL drive and radio: a per-setting rule (voyages design), not a figure"),
    ("materials", 7, "Materials only bear so much", "Strength for weight (pascals over density); bond energy sets a ceiling", "tank and structure mass, domes, towers, spin radius, tethers, megastructures", "ringworld floors, rigid shells round a star"),
    ("radiation", 8, "Radiation gets through", "Stopping of charged particles and gammas by mass in the way", "mass round the quarters, storm shelters, close passes", "force screens"),
    ("supplies", 9, "Crews need air, food and time", "Mass balance of a closed habitat; ageing", "habitat and stores size, greenhouses, who is awake", "stasis with no upkeep"),
    ("seen", 10, "Anything warm can be seen", "Every body radiates at its temperature", "detection range by band, hiding", "cloaks, which must say where the light goes"),
]

LADDER = {
    "id": "neutral", "name": "Neutral ladder",
    "about": "Real history on the lower rungs, so 'Sol today' is level 4. A setting defines its own ladder; this one is the default.",
    "improve_per_level_beyond": 1.5,
    # Kept while FT2's first reader still reads them; the kernel removes them
    # when its reader moves to the families (the fictional family holds the same numbers).
    "size_per_level_short": 10.0, "shrink_per_level_beyond": 0.5, "shrink_levels": 2,
    "improve_note": "past an advance's home level its figure gets half as much better again per level (times 1.5 where higher is better, divided by 1.5 where lower is), until it meets the next cited allowed or ceiling mark, then stops",
    "levels": [
        {"n": 0, "id": "muscle", "name": "Muscle and wind", "about": "before engines"},
        {"n": 1, "id": "steam", "name": "Steam and steel", "about": "about 1800 to 1900: engines, steel wire"},
        {"n": 2, "id": "combustion", "name": "Combustion and electricity", "about": "about 1900 to 1957: aircraft, early rockets"},
        {"n": 3, "id": "rocket", "name": "Rocket age", "about": "1957 to about 2000: orbit, the Moon, fission reactors, aramid"},
        {"n": 4, "id": "orbital", "name": "Orbital", "about": "today: a crewed station, recycled water, reusable rockets"},
        {"n": 5, "id": "torch", "name": "Torch", "about": "not yet built: fusion drives, spin habitats, closed life support"},
        {"n": 6, "id": "deep", "name": "Deep", "about": "not yet built: nanotube cable at length, antimatter in bulk"},
    ],
}

# Part families (design 5.2): how much worse an advance's own figure is for
# each level short of its home level. The size of the part then falls out of
# the sizing blocks. `rule`: "worse_by" divides the figure by `factor` per
# level short (multiplies where lower is better); "toward" puts it `share` of
# the way from the older figure to the mature one; "same" leaves it; "floor"
# keeps the advance's stated least size; "size" (fictional and unknown
# devices only) leaves the figure and scales the part's mass and volume.
FAMILIES = [
    {"id": "engine", "name": "Engines and power plants", "about": "output for their mass: drives, reactors, generators, radiators",
     "maturity": {"rule": "worse_by", "factor": 7.0}, "src": "ft-design-5.2"},
    {"id": "near-ceiling", "name": "Figures held down by a nearby ceiling", "about": "exhaust speed for a given fuel, conversion efficiency, how much a habitat recycles",
     "maturity": {"rule": "toward", "share": 2.0 / 3.0}, "src": "ft-design-5.2"},
    {"id": "material", "name": "Materials in bulk", "about": "strength for weight of something bought by the kilometre",
     "maturity": {"rule": "worse_by", "factor": 5.0}, "src": "ft-design-5.2"},
    {"id": "store", "name": "Stores of energy", "about": "batteries, fuel handling",
     "maturity": {"rule": "worse_by", "factor": 3.0}, "src": "ft-design-5.2"},
    {"id": "plain-mass", "name": "Plain mass", "about": "shielding by matter, ballast, reaction mass, structure that only has to be there",
     "maturity": {"rule": "same"}, "src": "ft-design-5.2"},
    {"id": "least-size", "name": "Devices with a least size set by physics", "about": "a critical mass, a plasma big enough to hold its heat: never smaller at any maturity",
     "maturity": {"rule": "floor"}, "src": "ft-design-5.2"},
    {"id": "fictional", "name": "Fictional devices and anything unknown", "about": "nothing to go on: Alex's rule of thumb",
     "maturity": {"rule": "size", "size_per_level_short": 10.0, "shrink_per_level_beyond": 0.5, "shrink_levels": 2}, "src": "ft-design-5.2"},
]

ADVANCES = [
    {"id": "steel-wire", "name": "Steel cable", "level": 1, "sets": {"strength_for_weight": strength_for_weight("steel-wire")}, "family": "material", "part": "structure"},
    {"id": "chemical-rocket", "name": "Hydrogen-oxygen rocket", "level": 3, "sets": {"exhaust_km_s": isp_km_s(465.5), "waste_per_jet": 0.0}, "family": "near-ceiling", "fixed": ["waste_per_jet"], "part": "drive"},
    {"id": "fission-reactor", "name": "Fission reactor", "level": 3, "sets": {"plant_w_per_kg": SNAP10A_W_KG}, "family": "engine", "part": "reactor"},
    {"id": "aramid", "name": "Aramid fibre", "level": 3, "sets": {"strength_for_weight": strength_for_weight("aramid")}, "family": "material", "part": "structure"},
    {"id": "pumped-radiator", "name": "Pumped ammonia radiators", "level": 3, "sets": {"radiator_k": 276.0}, "family": "engine", "part": "radiator"},
    {"id": "nuclear-thermal", "name": "Nuclear thermal rocket", "level": 4, "sets": {"exhaust_km_s": isp_km_s(875.0), "waste_per_jet": 0.0}, "family": "near-ceiling", "fixed": ["waste_per_jet"], "part": "drive",
     "note": "ground-tested 1959 to 1972, never flown: built at level 3 on the bench, home here as a flying part"},
    {"id": "pbo-fibre", "name": "PBO fibre", "level": 4, "sets": {"strength_for_weight": strength_for_weight("pbo-fibre")}, "family": "material", "part": "structure"},
    {"id": "water-recovery", "name": "Water recovery with brine processing", "level": 4, "sets": {"water_recycled": 0.98}, "family": "near-ceiling", "part": "stores"},
    {"id": "light-sail", "name": "Light sail or photon rocket", "level": 4, "sets": {"unthrown_n_per_w": 1.0 / C}, "family": "plain-mass", "fixed": ["unthrown_n_per_w"], "part": "drive"},
    {"id": "fusion-torch", "name": "Fusion torch (D-He3)", "level": 5, "sets": {"exhaust_km_s": isp_km_s(35435.0), "plant_w_per_kg": 8620.0, "fuel_j_per_kg": DHE3_J_KG, "waste_per_jet": 0.1, "radiator_k": 1500.0}, "family": "engine", "fixed": ["fuel_j_per_kg"], "part": "drive"},
    {"id": "spin-habitat", "name": "Comfortable spin habitat", "level": 5, "sets": {"spin_rpm": 4.0}, "family": "plain-mass", "fixed": ["spin_rpm"], "part": "spin ring",
     "note": "comfort is the crew's physiology, not the ring's maturity"},
    {"id": "nanotube-cable", "name": "Woven carbon cable", "level": 6, "sets": {"strength_for_weight": strength_for_weight("carbon-nanotube")}, "family": "material", "part": "structure"},
]


def mark_of(fig, v):
    """The mark an advance's value earns on a figure: past the ceiling is
    fictional, at or below the best built value is built, otherwise allowed."""
    up = fig["better"] == "higher"
    beyond = (lambda a, b: a > b) if up else (lambda a, b: a < b)
    ceil = [m["v"] for m in fig["marks"] if m["mark"] == "ceiling"]
    if ceil and beyond(v, ceil[0]):
        return "fictional"
    built = [m["v"] for m in fig["marks"] if m["mark"] == "built"]
    best = (max if up else min)(built) if built else None
    if best is not None and not beyond(v, best):
        return "built"
    return "allowed"


def build():
    figs = {f["id"]: f for f in FIGURES}
    limits = []
    for lid, n, plain, science, shapes, past in LIMITS:
        limits.append({"id": lid, "n": n, "plain": plain, "science": science, "shapes": shapes, "past_ceiling": past,
                       "figures": [f["id"] for f in FIGURES if f["limit"] == lid]})
    advances = []
    for a in ADVANCES:
        a = dict(a)
        marks = {k: mark_of(figs[k], v) for k, v in a["sets"].items()}
        a["mark"] = "fictional" if "fictional" in marks.values() else ("not yet built" if "allowed" in marks.values() else "built")
        advances.append(a)
    return {
        "format": "sse.pack/1", "id": "sse.base-tech", "version": "0.1.0", "kind": ["base-physics"],
        "kernel_range": "^0.1", "licence": "Apache-2.0",
        "provides": {"other": {"tech": {
            "about": "Future tech base pack (FT1): technology is how far each limit on building things has been pushed. An advance moves a figure; the same sizing block still runs; past a figure's ceiling is fictional by arithmetic. Built by scripts/build-base-tech.py from the master data sheet.",
            "limits": limits, "figures": FIGURES, "ladder": LADDER, "families": FAMILIES, "advances": advances,
            "sources": SOURCES,
        }}},
    }


def validate(pack):
    t = pack["provides"]["other"]["tech"]
    lids = {lim["id"] for lim in t["limits"]}
    assert len(t["limits"]) == 10, "ten starting limits"
    fids = set()
    for f in t["figures"]:
        assert f["limit"] in lids, f["id"]
        assert f["better"] in ("higher", "lower"), f["id"]
        assert f["id"] not in fids, f["id"]
        fids.add(f["id"])
        assert f["marks"], f["id"]
        for m in f["marks"]:
            assert m["mark"] in ("built", "allowed", "ceiling"), (f["id"], m)
            assert m["src"] in t["sources"], (f["id"], m["src"])
            assert m["mark"] != "ceiling" or m.get("law"), (f["id"], "a ceiling names its law")
            if m.get("sheet"):
                table, rid = m["sheet"].split("/")
                assert any(r["id"] == rid for r in SHEET[table]), m["sheet"]
    levels = {lv["n"] for lv in t["ladder"]["levels"]}
    fams = {f["id"] for f in t["families"]}
    assert {"engine", "near-ceiling", "material", "store", "plain-mass", "least-size", "fictional"} <= fams
    for a in t["advances"]:
        assert a["level"] in levels, a["id"]
        for k in a["sets"]:
            assert k in fids, (a["id"], k)
        assert a["family"] in fams, (a["id"], a["family"])
        assert a["family"] != "fictional", (a["id"], "the base pack holds no fiction")
        for k in a.get("fixed", []):
            assert k in a["sets"], (a["id"], k)
        assert a["mark"] != "fictional", (a["id"], "the base pack holds no fiction")


def main():
    pack = build()
    validate(pack)
    text = json.dumps(pack, indent=1, ensure_ascii=False) + "\n"
    if "--check" in sys.argv:
        if not OUT.exists() or OUT.read_text() != text:
            print("packs/base-tech/pack.json is out of date: run python3 scripts/build-base-tech.py")
            sys.exit(1)
        print("base-tech pack is current")
        return
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(text)
    print(f"wrote {OUT.relative_to(ROOT)}: {len(pack['provides']['other']['tech']['figures'])} figures, {len(pack['provides']['other']['tech']['advances'])} advances")


if __name__ == "__main__":
    main()
