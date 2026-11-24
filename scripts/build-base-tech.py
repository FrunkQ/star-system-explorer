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
# GPHS-RTG at start of mission (Schmidt 2009); read by a mark, an advance and REAL_CRAFT.
GPHS_W, GPHS_KG = 292.0, 55.9
ELEMENTS = {e["id"]: e for e in SHEET["elements"]}
H2O = next(x for x in SHEET["substances"] if x["id"] == "h2o")
# Hydrogen burned with oxygen, per kilogram of liquid water made (the higher
# heating value): the sheet's heat of formation over water's molar mass.
M_H2O = (2 * ELEMENTS["h"]["atomic_weight"]["v"] + ELEMENTS["o"]["atomic_weight"]["v"]) * 1e-3
HYDROLOX_J_KG = round(-H2O["chemical"]["states"]["liquid"]["dhf"]["v"] / M_H2O, -5)
# Exhaust leaves as vapour, so what the jet can have is the lower heating
# value: less the heat that would condense the water (the sheet's latent heat).
HYDROLOX_VAPOUR_J_KG = round(HYDROLOX_J_KG - H2O["phases"]["latent_vaporisation"]["v"], -5)
# Energy pays for the exhaust: v <= sqrt(2 E) (design 15.1), a computed ceiling.
HYDROLOX_EXHAUST_CEILING_KM_S = round((2.0 * HYDROLOX_VAPOUR_J_KG) ** 0.5 / 1e3, 2)
# D-He3 fusion: 3.7 MeV per nucleon (Ball 2019), in J/kg from the sheet's e and m_u.
DHE3_J_KG = round(3.7e6 * REF["e"] / REF["m_u"], -12)


def isp_km_s(isp_s):
    """Exhaust speed in km/s from a specific impulse in seconds (Isp times g0)."""
    return round(isp_s * G0 / 1e3, 3)


def strength_for_weight(mid):
    """A sheet material's tensile strength over its density, J/kg."""
    m = MIX[mid]
    return round(m["tensile_strength"]["v"] / m["density"]["v"], -3)


def sheet_k(mid, key):
    return MIX[mid][key]["v"]


def mark(kind, v, what, src, check=None, law=None, note=None, sheet=None, **extra):
    d = {"mark": kind, "v": v, "what": what, "src": src, **extra}
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
    "ft-design-15": "SSE4 future tech design, section 15 (keeping the books, accepted by Alex 23 Nov 2026): balance checks, heat grades (electronics near 350 K, quarters near 300 K), the four fields of a fictional advance",
    "fudge-register": "docs/fudges.md, Future tech: the part-family maturity defaults are rules of thumb from a small sample (design 15.7); their examples stay cited in ft-design-5.2",
    "nasa-sp4029": "Orloff, R. W. 2000, Apollo by the Numbers, NASA SP-4029: Apollo 11 (AS-506) ground ignition weights in lb (actual, from the flight evaluation reports) and launch vehicle key facts (F-1 1,522,000 lbf sea level; J-2 230,000 lbf). https://www.nasa.gov/wp-content/uploads/static/history/SP-4029/Apollo_18-19_Ground_Ignition_Weights.htm",
    "msfc-as506": "MSFC 1969, Saturn V launch vehicle flight evaluation report AS-506 Apollo 11: 2,941,221 kg at S-IC ignition, 135,669 kg at orbit insertion (text extraction partial). https://ibiblio.org/apollo/Documents/lvfea-AS506-Apollo11.pdf",
    "msfc-as505": "MSFC 1969, Technical information summary Apollo-10 (AS-505): spacecraft at translunar injection 94,500 lb (noisy scan). https://www.nasa.gov/wp-content/uploads/static/history/afj/ap10fj/pdf/19700015780_as-505-a10-saturn-v-technical-info-summary.pdf",
    "rocketdyne-f1": "Rocketdyne 1967, R-3896-1 Technical manual, engine data, F-1 rocket engine, figure 1-5: sea-level specific impulse 265.3 s, thrust 1,522,000 lbf, mixture ratio 2.27, dry weight 18,619 lb",
    "coffman-j2": "Coffman, P. 2010, Remembering the Giants, chapter 2 (NASA, NTRS 20100027318): J-2 specific impulse 425 s (vacuum implied), up to 230,000 lbf",
    "spacex-f9": "SpaceX, Falcon 9 vehicle page (read Nov 2026): 549,054 kg, 22,800 kg to low Earth orbit (expendable, not stated), second stage 981 kN. https://www.spacex.com/vehicles/falcon-9",
    "nasa-mmrtg": "NASA Facts NF-2020-05-619-HQ, Multi-Mission Radioisotope Thermoelectric Generator: about 110 W electric at launch, about 45 kg, 4.8 kg of plutonium dioxide",
    "schmidt2009": "Schmidt, G. R., Sutliff, T. J. and Dudzinski, L. A. 2009, Radioisotope power: a key technology for deep space exploration, IAC-09 (NTRS 20120016365): MHW-RTG 37.7 kg and 158 W at start of mission; GPHS-RTG 55.9 kg and up to 300 W (sidebar 292 W)",
    "jpl-juno2011": "NASA and JPL 2011, Juno launch press kit: three arrays of more than 60 m2 in all, 18,698 cells, about 14 kW at Earth's distance and about 400 W at Jupiter's, launch mass 3,625 kg. https://www.jpl.nasa.gov/news/press_kits/JunoLaunch.pdf",
    "nasa-orbiter-tps": "NASA Facts FS-2008-02-042-KSC, Orbiter thermal protection system: reinforced carbon-carbon above 3000 F, tiles for 1,200 to 2,500 F, entry in excess of 17,000 mph",
    "pavlosky1974": "Pavlosky, J. E. and St. Leger, L. G. 1974, Apollo experience report: thermal protection subsystem, NASA TN D-7564: Avco 5026-39G at 31 lb/ft3; design lunar return 36,333 ft/s; Apollo 4 peak 425 Btu/ft2 s and total 37,522 Btu/ft2; surfaces approaching 5000 F",
    "nasa-iss-facts": "NASA, Space station facts and figures (read Nov 2026): 1,005 m3 pressurised, 388 m3 habitable, 419,725 kg, 75 to 90 kW from 8 arrays. https://www.nasa.gov/international-space-station/space-station-facts-and-figures/",
    "sovey2001": "Sovey, J. S., Rawlin, V. K. and Patterson, M. J. 2001, Ion propulsion development projects in U.S.: Space Electric Rocket Test I to Deep Space 1, J. Propulsion and Power 17(3), 517-526: NSTAR 1900 s at 0.5 kW to 3100 s at 2.3 kW, 19 to 92 mN; flight-set masses thruster 8.2 kg, PPU 14.77 kg; 9,241 h of thrusting by 17 Feb 2001. https://ntrs.nasa.gov/api/citations/20010093217/downloads/20010093217.pdf",
    "rayman1999": "Rayman, M. D., Varghese, P., Lehman, D. H. and Livesay, L. L. 1999, Results from the Deep Space 1 technology validation mission, IAC-99-IAA.11.2.01, Acta Astronautica 47, 475 (2000): 3180 s at 2140 W in flight; 11.4 kg of xenon gave 699.6 m/s in 1799.4 h. https://www.jpl.nasa.gov/nmp/ds1/DS1_Primary_Mission.pdf",
    "brophy1992": "Brophy, J. R. 1992, Stationary plasma thruster evaluation in Russia, JPL Publication 92-4 (NASA-CR-192823): SPT first operated in space 1 March 1972; SPT-100 1600 s, 0.080 N, 1.35 kW, efficiency 0.50, thruster 4 kg (Table 1). https://ntrs.nasa.gov/api/citations/19930016017/downloads/19930016017.pdf",
    "sankovic1993": "Sankovic, J. M., Hamley, J. A. and Haag, T. W. 1993, Performance evaluation of the Russian SPT-100 thruster at NASA LeRC, IEPC-93-094 (NASA TM-106401): 1610 s and 0.50 at 300 V, 4.5 A. https://ntrs.nasa.gov/api/citations/19940019157/downloads/19940019157.pdf",
    "curran-byers1994": "Curran, F. M. and Byers, D. C. 1994, New developments and research findings: NASA hydrazine arcjets, NASA TM-106695 (AIAA-94-2463): kW-class arcjets operational on Telstar 401 (launched December 1993), a nominal mission average of 500 s at a 1.4 kW system power level. https://ntrs.nasa.gov/api/citations/19950004443/downloads/19950004443.pdf",
    "jaxa-ikaros": "JAXA IKAROS Demonstration Team (JSPEC), Introduction of Small Solar Power Sail Demonstrator IKAROS (press kit, 2010): square sail 14 m a side, 7.5 micrometre aluminised polyimide, membrane 15 kg with 2 kg of tip masses, 310 kg at launch; Tsuda et al. 2013 (J. Space Tech. Sci. 27, 1_79) give 200 m2; eoPortal: 100 m/s from sunlight within half a year. https://global.jaxa.jp/countdown/f17/pdf/presskit_ikaros_e.pdf",
    "gwin-mee1959": "Gwin, R. and Mee, W. T. 1959, Critical assemblies of uranium metal, Y-1248, Union Carbide Nuclear Company, Oak Ridge Y-12 Plant, Table I: bare sphere of 93 to 94% U-235 metal (about 18.5 g/cm3), critical at 53.9 kg. https://www.osti.gov/servlets/purl/4246875",
    "airy1835": "Airy, G. B. 1835, On the diffraction of an object-glass with circular aperture, Trans. Cambridge Phil. Soc. 5, 283: the first dark ring at 1.22 wavelengths over the aperture",
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
    {"id": "waste_heat_k", "limit": "heat", "plain": "temperature of a waste heat source", "unit": "K", "better": "higher",
     "reads": "FT2 (heat grades: a radiator runs no hotter than its source unless a heat pump is paid for)",
     "marks": [mark("built", 350.0, "electronics", "ft-design-15", check="authored"),
               mark("built", 300.0, "a crew's quarters", "ft-design-15", check="authored"),
               mark("allowed", 1500.0, "a D-He3 fusion drive's structure", "santarius-logan-1998", check="unverified")]},
    {"id": "heat_sent_nowhere", "limit": "heat", "plain": "share of waste heat that leaves without being radiated or carried", "unit": "1", "better": "higher",
     "reads": "FT2",
     "marks": [mark("ceiling", 0.0, "a heat sink that sends heat nowhere", "law", law="second law of thermodynamics")]},
    # 2 Thrust needs something to throw
    {"id": "exhaust_km_s", "limit": "thrust", "plain": "exhaust speed", "unit": "km s-1", "better": "higher",
     "reads": "ship exhaust_km_s (sse-nav Ship)",
     "marks": [mark("built", isp_km_s(465.5), "RL10B-2 hydrogen-oxygen engine, 465.5 s", "l3harris-rl10"),
               mark("built", isp_km_s(875.0), "Pewee 1 nuclear thermal reactor, about 875 s (ground test, never flown)", "nasem2021"),
               mark("built", isp_km_s(3100.0), "NSTAR ion thruster on Deep Space 1, 3100 s at 2.3 kW", "sovey2001"),
               mark("allowed", isp_km_s(35435.0), "D-He3 spherical torus fusion ship, 35,435 s", "williams2005"),
               mark("allowed", isp_km_s(77000.0), "D-He3 magnetised target fusion ship, 77,000 s", "adams2003"),
               mark("ceiling", HYDROLOX_EXHAUST_CEILING_KM_S, "hydrogen and oxygen: every joule of the fuel in the jet", "sheet", law="conservation of energy",
                    note="computed from sqrt(2 x fuel_j_per_kg), with the fuel's lower heating value; replaces the 'near 5 km/s' once held from memory",
                    **{"for": "hydrogen-oxygen", "computed_from": {"balance": "energy-pays-exhaust", "figures": {"fuel_j_per_kg": HYDROLOX_VAPOUR_J_KG}}}),
               mark("ceiling", round(C / 1e3, 3), "the speed of light", "law", law="special relativity", sheet="constants/c")]},
    {"id": "unthrown_n_per_w", "limit": "thrust", "plain": "thrust per watt with nothing thrown", "unit": "N W-1", "better": "higher",
     "reads": "FT2",
     "marks": [mark("built", 1.0 / C, "light pressure: a photon rocket or a lamp, 3.3 N per GW", "lubin2016", sheet="constants/c"),
               mark("ceiling", 1.0 / C, "a reactionless drive above 1/c gives out more energy than it takes in", "law", law="conservation of momentum and energy", sheet="constants/c")]},
    # 3 Energy has weight
    {"id": "fuel_j_per_kg", "limit": "energy", "plain": "energy per kilogram of fuel", "unit": "J kg-1", "better": "higher",
     "reads": "FT2 (the fuel's energy beside base-nav fuels)",
     "marks": [mark("built", HYDROLOX_VAPOUR_J_KG, "hydrogen burned with oxygen, per kilogram of water vapour made", "sheet", sheet="substances/h2o", note=f"the lower heating value: water's heat of formation less its latent heat of vaporisation; {HYDROLOX_J_KG / 1e6:.1f} MJ/kg if the water condenses"),
               mark("built", 8.21e13, "U-235 fission, 200 MeV a fission", "openstax-up3"),
               mark("allowed", DHE3_J_KG, "D-He3 fusion, 3.7 MeV per nucleon", "ball2019"),
               mark("ceiling", C * C, "matter with antimatter, all of the mass", "law", law="mass-energy equivalence", sheet="constants/c")]},
    {"id": "plant_w_per_kg", "limit": "energy", "plain": "power per kilogram of power plant and drive", "unit": "W kg-1", "better": "higher",
     "reads": "FT2",
     "marks": [mark("built", SNAP10A_W_KG, "SNAP-10A, the only reactor the US flew: 500 W for under 950 lb", "doe-snap"),
               mark("built", round(GPHS_W / GPHS_KG, 2), "GPHS-RTG radioisotope generator, 292 W for 55.9 kg", "schmidt2009"),
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
    # 7, second figure (RK2): materials only take so much heat. A hot-gas
    # rocket's exhaust speed goes as the root of temperature over gas weight,
    # so this caps every nuclear thermal engine, nozzle and radiator.
    {"id": "working_k", "limit": "materials", "plain": "hottest a working part can run", "unit": "K", "better": "higher",
     "reads": "FT2 (hot-gas exhaust from temperature; nozzle, fuel, radiator and heat shield limits)",
     "marks": [mark("built", sheet_k("inconel-718", "service_limit_k"), "nickel superalloy (Inconel 718): turbopumps and engine structure", "sheet", sheet="mixtures/inconel-718"),
               mark("built", sheet_k("tungsten", "service_limit_k"), "tungsten heat shields in vacuum furnaces", "sheet", sheet="mixtures/tungsten"),
               mark("ceiling", sheet_k("uo2-fuel", "melting_point_k"), "uranium dioxide fuel melts", "sheet", sheet="mixtures/uo2-fuel", law="melting: past it the solid is not one", **{"for": "uo2-fuel"}),
               mark("ceiling", sheet_k("tungsten", "melting_point_k"), "tungsten melts, the highest melting metal", "sheet", sheet="mixtures/tungsten", law="melting: past it the solid is not one", **{"for": "tungsten"})]},
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
    # 11 Beams spread (RK2): one figure for beamed power, laser sails, links,
    # sensors and telescopes; the Equation is the same in each.
    {"id": "spread_over_diffraction", "limit": "beams", "plain": "how much wider a beam spreads than diffraction allows", "unit": "1", "better": "lower",
     "reads": "FT2 (beamed power, laser sails, link rates, sensor and telescope sharpness)",
     "marks": [mark("ceiling", 1.0, "a perfect beam: it still spreads at 1.22 wavelengths over the aperture", "airy1835", law="diffraction")]},
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
    ("beams", 11, "Beams spread", "Diffraction: a beam's spot grows as wavelength times distance over aperture", "beamed power, laser sails, link rates, how sharp a telescope or sensor sees", "a beam that stays narrow at any distance"),
]

LADDER = {
    "id": "neutral", "name": "Neutral ladder",
    "about": "Real history on the lower rungs, so 'Sol today' is level 4. A setting defines its own ladder; this one is the default.",
    "improve_per_level_beyond": 1.5,
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
     "maturity": {"rule": "worse_by", "factor": 7.0}, "src": "fudge-register", "examples_src": "ft-design-5.2"},
    {"id": "near-ceiling", "name": "Figures held down by a nearby ceiling", "about": "exhaust speed for a given fuel, conversion efficiency, how much a habitat recycles",
     "maturity": {"rule": "toward", "share": 2.0 / 3.0}, "src": "fudge-register", "examples_src": "ft-design-5.2"},
    {"id": "material", "name": "Materials in bulk", "about": "strength for weight of something bought by the kilometre",
     "maturity": {"rule": "worse_by", "factor": 5.0}, "src": "fudge-register", "examples_src": "ft-design-5.2"},
    {"id": "store", "name": "Stores of energy", "about": "batteries, fuel handling",
     "maturity": {"rule": "worse_by", "factor": 3.0}, "src": "fudge-register", "examples_src": "ft-design-5.2"},
    {"id": "plain-mass", "name": "Plain mass", "about": "shielding by matter, ballast, reaction mass, structure that only has to be there",
     "maturity": {"rule": "same"}, "src": "fudge-register", "examples_src": "ft-design-5.2"},
    {"id": "least-size", "name": "Devices with a least size set by physics", "about": "a critical mass, a plasma big enough to hold its heat: never smaller at any maturity",
     "maturity": {"rule": "floor"}, "src": "fudge-register", "examples_src": "ft-design-5.2"},
    {"id": "fictional", "name": "Fictional devices and anything unknown", "about": "nothing to go on: Alex's rule of thumb",
     "maturity": {"rule": "size", "size_per_level_short": 10.0, "shrink_per_level_beyond": 0.5, "shrink_levels": 2}, "src": "fudge-register", "examples_src": "ft-design-5.2"},
]

# Balance checks (design 15.1): relations between figures a design already
# has. Names in a relation are figure ids, or the design's own quantities
# (thrust_n, mdot_kg_s, jet_w, plant_w, waste_w, stored_w, loads_w, q_w,
# t_hot_k, t_cold_k, pump_w). A check that does not close is fictional by
# arithmetic, like a figure past its ceiling, and is named on the card.
BALANCES = [
    {"id": "energy-pays-exhaust", "name": "Energy pays for the exhaust",
     "relation": "exhaust_km_s * 1000 <= sqrt(2 * fuel_j_per_kg)",
     "near_c": "gamma - 1 <= fuel_j_per_kg / c^2 for the exhaust's speed (the relativistic form)",
     "figures": ["exhaust_km_s", "fuel_j_per_kg"], "law": "conservation of energy",
     "catches": "a chemical fuel with a torch's exhaust speed"},
    {"id": "jet-power", "name": "Jet power",
     "relation": "jet_w = 0.5 * thrust_n * exhaust_km_s * 1000; jet_w <= plant_w",
     "figures": ["exhaust_km_s", "plant_w_per_kg"], "law": "kinetic energy of the exhaust",
     "catches": "thrust and exhaust speed set apart that need more power than the ship makes"},
    {"id": "energy-in-out", "name": "Energy in, energy out",
     "relation": "plant_w = jet_w + waste_w + stored_w + loads_w",
     "figures": ["plant_w_per_kg", "waste_per_jet"], "law": "conservation of energy",
     "catches": "waste heat set to nothing while the plant's efficiency says otherwise"},
    {"id": "momentum", "name": "Momentum",
     "relation": "thrust_n = mdot_kg_s * exhaust_km_s * 1000, or thrust_n <= plant_w * unthrown_n_per_w for light alone",
     "figures": ["exhaust_km_s", "unthrown_n_per_w"], "law": "conservation of momentum",
     "catches": "thrust with nothing thrown and no stated partner"},
    {"id": "heat-downhill", "name": "Heat flows downhill",
     "relation": "radiator_k <= waste_heat_k, unless a heat pump is paid for: pump_w >= q_w * (t_hot_k - t_cold_k) / t_cold_k, and pump_w is shed too",
     "figures": ["radiator_k", "waste_heat_k"], "law": "second law of thermodynamics (Carnot)",
     "catches": "a hot radiator cooling a cool room for free"},
]
for b in BALANCES:
    b["src"] = "ft-design-15"

# What every fictional advance must say (design 15.3, question 11): where its
# light, heat, momentum and energy go. "nowhere" or "nothing" is allowed and shows.
FICTIONAL_WHERE = [
    {"id": "light", "question": "what it gives off, by band", "example": "a blue flash on entry"},
    {"id": "heat", "question": "where the heat ends up", "example": "stored in the sink, dumped in port; holds 40 hours"},
    {"id": "momentum", "question": "what it pushes on", "example": "the nearest large mass"},
    {"id": "energy", "question": "where the energy comes from, or goes", "example": "the ship's plant, with the surplus from nowhere"},
]

ADVANCES = [
    {"id": "steel-wire", "name": "Steel cable", "level": 1, "sets": {"strength_for_weight": strength_for_weight("steel-wire")}, "family": "material", "part": "structure"},
    {"id": "chemical-rocket", "name": "Hydrogen-oxygen rocket", "level": 3, "sets": {"exhaust_km_s": isp_km_s(465.5), "waste_per_jet": 0.0}, "fuel": "hydrogen-oxygen", "family": "near-ceiling", "fixed": ["waste_per_jet"], "part": "drive"},
    {"id": "fission-reactor", "name": "Fission reactor", "level": 3, "sets": {"plant_w_per_kg": SNAP10A_W_KG}, "family": "engine", "part": "reactor",
     "least_size": {"v": 53.9, "unit": "kg", "what": "fissile metal: a bare sphere of 93 to 94% U-235 goes critical at 53.9 kg; a reflector lowers it", "src": "gwin-mee1959", "check": "unverified"}},
    {"id": "aramid", "name": "Aramid fibre", "level": 3, "sets": {"strength_for_weight": strength_for_weight("aramid")}, "family": "material", "part": "structure"},
    {"id": "pumped-radiator", "name": "Pumped ammonia radiators", "level": 3, "sets": {"radiator_k": 276.0}, "family": "engine", "part": "radiator"},
    {"id": "nuclear-thermal", "name": "Nuclear thermal rocket", "level": 4, "sets": {"exhaust_km_s": isp_km_s(875.0), "waste_per_jet": 0.0}, "family": "near-ceiling", "fixed": ["waste_per_jet"], "part": "drive",
     "note": "ground-tested 1959 to 1972, never flown: built at level 3 on the bench, home here as a flying part"},
    {"id": "pbo-fibre", "name": "PBO fibre", "level": 4, "sets": {"strength_for_weight": strength_for_weight("pbo-fibre")}, "family": "material", "part": "structure"},
    {"id": "water-recovery", "name": "Water recovery with brine processing", "level": 4, "sets": {"water_recycled": 0.98}, "family": "near-ceiling", "part": "stores"},
    {"id": "light-sail", "name": "Light sail or photon rocket", "level": 4, "sets": {"unthrown_n_per_w": 1.0 / C}, "family": "plain-mass", "fixed": ["unthrown_n_per_w"], "part": "drive",
     "flown": {"what": "IKAROS (2010): a 200 m2 sail of 7.5 micrometre film, 15 kg, gained 100 m/s from sunlight in half a year", "src": "jaxa-ikaros"}},
    {"id": "fusion-torch", "name": "Fusion torch (D-He3)", "level": 5, "sets": {"exhaust_km_s": isp_km_s(35435.0), "plant_w_per_kg": 8620.0, "fuel_j_per_kg": DHE3_J_KG, "waste_per_jet": 0.1, "waste_heat_k": 1500.0, "radiator_k": 1500.0}, "family": "engine", "fixed": ["fuel_j_per_kg"], "part": "drive"},
    {"id": "spin-habitat", "name": "Comfortable spin habitat", "level": 5, "sets": {"spin_rpm": 4.0}, "family": "plain-mass", "fixed": ["spin_rpm"], "part": "spin ring",
     "note": "comfort is the crew's physiology, not the ring's maturity"},
    {"id": "nanotube-cable", "name": "Woven carbon cable", "level": 6, "sets": {"strength_for_weight": strength_for_weight("carbon-nanotube")}, "family": "material", "part": "structure"},
]

FT_S = 0.3048
BTU_FT2 = 1055.05585262 / (FT_S * FT_S)  # J/m2 per Btu/ft2 (International Table Btu)


def f_to_k(f):
    return round((f - 32.0) * 5.0 / 9.0 + REF["T0C"], 1)


def q(v, unit, note=None):
    d = {"v": v, "unit": unit}
    if note:
        d["note"] = note
    return d


def lb(x):
    return round(x * LB_KG, 0)


# Real craft for the real-hardware test (design 15.6): the sizing blocks, given
# a craft's own figures, should come out near what was built, and every
# balance should close. Held here by hand until a real spaceflight pack
# supplies them. Quantities stored in SI with units; conversions from the
# source's units are done above, never typed.
REAL_CRAFT = [
    {"id": "iss-radiators", "name": "International Space Station heat rejection", "flew": "1998 to now",
     "quantities": {"q_w": q(70000.0, "W", "35 kW per loop, two loops"),
                    "radiator_k": q(275.95, "K", "ammonia supply temperature, not the panel's"),
                    "planform_m2": q(round(6 * 23.3 * 3.4, 0), "m2", "six radiator units of 23.3 by 3.4 m; worked out here, not stated; whether both faces radiate is not stated")},
     "tests": ["heat-downhill", "radiator sizing"], "src": ["nasa-iss-atcs"]},
    {"id": "saturn-v-s-ic", "name": "Saturn V first stage (Apollo 11)", "flew": "1969",
     "quantities": {"dry_kg": q(lb(287531.0), "kg"), "propellant_kg": q(lb(3305786.0 + 1424889.0), "kg", "oxidiser and fuel"),
                    "thrust_n": q(round(5 * 1522000.0 * LB_KG * G0, -3), "N", "five F-1, sea level, rated"),
                    "exhaust_km_s": q(isp_km_s(265.3), "km/s", "F-1 at sea level; vacuum value not found")},
     "tests": ["momentum", "jet-power", "tank sizing"], "src": ["nasa-sp4029", "rocketdyne-f1"]},
    {"id": "saturn-v-s-ii", "name": "Saturn V second stage (Apollo 11)", "flew": "1969",
     "quantities": {"dry_kg": q(lb(79714.0), "kg"), "propellant_kg": q(lb(819050.0 + 158116.0), "kg"),
                    "thrust_n": q(round(5 * 230000.0 * LB_KG * G0, -3), "N", "five J-2"),
                    "exhaust_km_s": q(isp_km_s(425.0), "km/s", "J-2, vacuum implied")},
     "tests": ["energy-pays-exhaust", "momentum", "tank sizing"], "src": ["nasa-sp4029", "coffman-j2"]},
    {"id": "saturn-v-s-ivb", "name": "Saturn V third stage (Apollo 11)", "flew": "1969",
     "quantities": {"dry_kg": q(lb(24852.0), "kg"), "propellant_kg": q(lb(192497.0 + 43608.0), "kg"),
                    "thrust_n": q(round(202603.0 * LB_KG * G0, -3), "N", "first burn, one J-2"),
                    "exhaust_km_s": q(isp_km_s(425.0), "km/s", "J-2, vacuum implied")},
     "tests": ["energy-pays-exhaust", "tank sizing"], "src": ["nasa-sp4029", "coffman-j2"]},
    {"id": "saturn-v-stack", "name": "Saturn V with Apollo", "flew": "1969",
     "quantities": {"ignition_kg": q(2941221.0, "kg", "Apollo 11 at S-IC ignition"), "orbit_kg": q(135669.0, "kg", "Apollo 11 at orbit insertion"),
                    "tli_payload_kg": q(lb(94500.0), "kg", "Apollo 10 spacecraft at translunar injection")},
     "tests": ["rocket equation over the three stages"], "src": ["msfc-as506", "msfc-as505"]},
    {"id": "falcon-9", "name": "Falcon 9", "flew": "2010 to now",
     "quantities": {"liftoff_kg": q(549054.0, "kg"), "leo_payload_kg": q(22800.0, "kg", "expendable, not stated on the page"),
                    "upper_thrust_n": q(981000.0, "N")},
     "tests": ["payload share against the Saturn V"], "src": ["spacex-f9"],
     "note": "stage masses and Merlin specific impulse are not published by the maker"},
    {"id": "mmrtg", "name": "Multi-mission radioisotope generator (Curiosity, Perseverance)", "flew": "2011 to now",
     "quantities": {"plant_w": q(110.0, "W", "electric, at launch"), "mass_kg": q(45.0, "kg")},
     "tests": ["energy-in-out", "plant sizing"], "src": ["nasa-mmrtg"]},
    {"id": "mhw-rtg", "name": "Voyager's radioisotope generators (one of three)", "flew": "1976 to now",
     "quantities": {"plant_w": q(158.0, "W", "electric, start of mission"), "mass_kg": q(37.7, "kg")},
     "tests": ["plant sizing"], "src": ["schmidt2009"]},
    {"id": "gphs-rtg", "name": "GPHS radioisotope generator (Galileo, Cassini, New Horizons)", "flew": "1989 to now",
     "quantities": {"plant_w": q(GPHS_W, "W", "electric, start of mission; New Horizons' own unit made about 213 to 250 W"), "mass_kg": q(GPHS_KG, "kg")},
     "tests": ["plant sizing"], "src": ["schmidt2009"]},
    {"id": "juno-arrays", "name": "Juno's solar arrays", "flew": "2011 to now",
     "quantities": {"area_m2": q(60.0, "m2", "more than"), "power_1au_w": q(14000.0, "W"), "power_jupiter_w": q(400.0, "W")},
     "tests": ["light falls with the square of distance (about 520 W expected at 5.2 au before the cold and dim penalty)"], "src": ["jpl-juno2011"]},
    {"id": "shuttle-entry", "name": "Space Shuttle orbiter entry", "flew": "1981 to 2011",
     "quantities": {"entry_m_s": q(round(17000.0 * 1609.344 / 3600.0, 0), "m/s", "in excess of 17,000 mph"),
                    "rcc_surface_k": q(f_to_k(3000.0), "K", "nose cap and wing leading edges, up to 3,000 F"),
                    "tile_max_k": q(f_to_k(2500.0), "K")},
     "tests": ["entry heating", "heat shield sizing"], "src": ["nasa-orbiter-tps"]},
    {"id": "apollo-heat-shield", "name": "Apollo command module heat shield", "flew": "1966 to 1975",
     "quantities": {"entry_m_s": q(round(36333.0 * FT_S, 0), "m/s", "design lunar return, inertial"),
                    "peak_heating_w_m2": q(round(425.0 * BTU_FT2, -3), "W/m2", "Apollo 4"),
                    "heat_load_j_m2": q(round(37522.0 * BTU_FT2, -4), "J/m2", "Apollo 4, total"),
                    "ablator_kg_m3": q(round(31.0 * LB_KG / FT_S ** 3, 0), "kg/m3", "Avco 5026-39G")},
     "tests": ["entry heating", "ablator sizing"], "src": ["pavlosky1974"]},
    {"id": "iss", "name": "International Space Station", "flew": "1998 to now",
     "quantities": {"pressurised_m3": q(round(35491.0 * FT_S ** 3, 0), "m3", "35,491 cubic feet"), "habitable_m3": q(388.0, "m3", "visiting craft not counted"),
                    "mass_kg": q(419725.0, "kg"), "array_power_w": q(90000.0, "W", "75 to 90 kW from eight arrays")},
     "tests": ["habitat sizing", "energy-in-out against the 70 kW rejected"], "src": ["nasa-iss-facts", "nasa-iss-atcs"]},
]


def mark_of(fig, v):
    """The mark an advance's value earns on a figure: past the ceiling is
    fictional, at or below the best built value is built, otherwise allowed."""
    up = fig["better"] == "higher"
    beyond = (lambda a, b: a > b) if up else (lambda a, b: a < b)
    ceil = [m["v"] for m in fig["marks"] if m["mark"] == "ceiling" and "for" not in m]
    if ceil and beyond(v, ceil[0]):
        return "fictional"
    built = [m["v"] for m in fig["marks"] if m["mark"] == "built"]
    best = (max if up else min)(built) if built else None
    if best is not None and not beyond(v, best):
        return "built"
    return "allowed"


# RK2: real, flown advances for the families that have flown (Atomic Rockets
# plan section 7). Each says what flew.
ADVANCES += [
    {"id": "hall-thruster", "name": "Hall thruster", "level": 3, "sets": {"exhaust_km_s": isp_km_s(1600.0)}, "family": "engine", "part": "electric drive",
     "flown": {"what": "stationary plasma thrusters, first run in space on 1 March 1972; the SPT-100 gives 1600 s and 80 mN from 1.35 kW for a 4 kg thruster", "src": "brophy1992", "also": ["sankovic1993"]},
     "note": "electric: thrust is set by the power plant; exhaust speed is the figure the advance pushes"},
    {"id": "arcjet", "name": "Hydrazine arcjet", "level": 3, "sets": {"exhaust_km_s": isp_km_s(500.0)}, "family": "engine", "part": "electric drive",
     "flown": {"what": "kW-class arcjets on Telstar 401 (launched December 1993): a nominal mission average of 500 s at 1.4 kW", "src": "curran-byers1994"}},
    {"id": "ion-thruster", "name": "Gridded ion thruster", "level": 3, "sets": {"exhaust_km_s": isp_km_s(3100.0)}, "family": "engine", "part": "electric drive",
     "flown": {"what": "NSTAR on Deep Space 1 (1998): 3100 s and 92 mN at 2.3 kW, 9,241 hours of thrusting by February 2001", "src": "sovey2001", "also": ["rayman1999"]}},
    {"id": "rtg", "name": "Radioisotope generator", "level": 3,
     "sets": {"plant_w_per_kg": round(GPHS_W / GPHS_KG, 2)}, "family": "engine", "part": "power plant",
     "flown": {"what": "GPHS-RTG (Galileo 1989, Ulysses, Cassini, New Horizons): 292 W for 55.9 kg", "src": "schmidt2009"}},
]


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
            "limits": limits, "figures": FIGURES, "ladder": LADDER, "families": FAMILIES, "balances": BALANCES, "fictional_where": FICTIONAL_WHERE, "advances": advances,
            "real_craft": REAL_CRAFT, "sources": SOURCES,
        }}},
    }


def validate(pack):
    t = pack["provides"]["other"]["tech"]
    lids = {lim["id"] for lim in t["limits"]}
    assert len(t["limits"]) == 11, "ten starting limits and beams spread (RK2)"
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
    for f in t["figures"]:
        assert f["unit"] and f["unit"] not in ("x", "factor", "times"), (f["id"], "stored as the quantity, never a factor")
    for b in t["balances"]:
        for k in b["figures"]:
            assert k in fids, (b["id"], k)
        assert b["src"] in t["sources"], b["id"]
    bids = {b["id"] for b in t["balances"]}
    for f in t["figures"]:
        for m in f["marks"]:
            cf = m.get("computed_from")
            if cf:
                assert cf["balance"] in bids, (f["id"], cf)
                for k in cf["figures"]:
                    assert k in fids, (f["id"], k)
    assert len(t["real_craft"]) >= 10, "about ten real craft for the real-hardware test"
    for rc in t["real_craft"]:
        assert rc["src"] and all(x in t["sources"] for x in rc["src"]), rc["id"]
        for k, v in rc["quantities"].items():
            assert v["unit"] and isinstance(v["v"], float), (rc["id"], k)
    where = [w["id"] for w in t["fictional_where"]]
    assert where == ["light", "heat", "momentum", "energy"]
    levels = {lv["n"] for lv in t["ladder"]["levels"]}
    fams = {f["id"] for f in t["families"]}
    assert {"engine", "near-ceiling", "material", "store", "plain-mass", "least-size", "fictional"} <= fams
    for a in t["advances"]:
        assert a["level"] in levels, a["id"]
        for k in a["sets"]:
            assert k in fids, (a["id"], k)
        assert a["family"] in fams, (a["id"], a["family"])
        assert a["family"] != "fictional", (a["id"], "the base pack holds no fiction")
        if a["family"] == "fictional":
            assert all(isinstance(a.get("where", {}).get(w), str) for w in where), (a["id"], "a fictional advance says where its light, heat, momentum and energy go")
        if a["sets"].get("waste_per_jet", 0.0) > 0.0:
            assert "waste_heat_k" in a["sets"], (a["id"], "every waste heat source carries its temperature")
        for k in a.get("fixed", []):
            assert k in a["sets"], (a["id"], k)
        assert a["mark"] != "fictional", (a["id"], "the base pack holds no fiction")
        if a["family"] == "least-size":
            assert "least_size" in a, (a["id"], "a least-size device states its floor")
        if "least_size" in a:
            ls = a["least_size"]
            assert ls["v"] > 0 and ls["unit"] and ls["src"] in t["sources"], a["id"]
        if "flown" in a:
            assert a["flown"]["src"] in t["sources"] and all(x in t["sources"] for x in a["flown"].get("also", [])), a["id"]
            assert a["mark"] == "built", (a["id"], "what flew is built")


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
