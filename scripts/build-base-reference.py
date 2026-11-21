#!/usr/bin/env python3
"""Build packs/base-reference/pack.json, the master data sheet's first wave (track MP, package MP2).

Shape agreed with the kernel thread (design section 4): tables under provides.reference
(constants, elements, isotopes, substances, mixtures, sources). Every value is
{"v": number, "unit": "<SI text>", "src": "<source id>"}, plus "check" (the citations
register's status for that number: verified, unverified, authored), "at_k" / "at_pa" for
the conditions, and "note". Substances carry no molar mass: the reader works it out from
the formula and the element weights.

Inputs: this file (constants, isotopes, substances, mixtures, sources) and
handover/packages/MP2-data/elements.csv (118 elements: IUPAC 2021 weights, Lodders 2021
protosolar abundances). Run from the repo root:

    python3 scripts/build-base-reference.py          # writes the pack
    python3 scripts/build-base-reference.py --check  # exits 1 if the pack is stale

Python 3 standard library only. The output is sorted and stable, so a rerun with no input
change gives the same bytes.
"""
import csv
import json
import math
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "packs/base-reference/pack.json"
ELEMENTS_CSV = ROOT / "handover/packages/MP2-data/elements.csv"
ORIGINS_CSV = ROOT / "handover/packages/MP2-data/origins.csv"

sys.path.insert(0, str(Path(__file__).resolve().parent))
import element_origins  # noqa: E402

JULIAN_YEAR_S = 31557600.0


def val(v, unit, src, check="verified", **extra):
    d = {"v": v, "unit": unit, "src": src}
    if check != "verified":
        d["check"] = check
    d.update(extra)
    return d


# ---------------------------------------------------------------- sources
# id -> (text, doi, url, register status). The register is docs/citations/matter-reference.md.
SOURCES = {
    "codata2018": ("Tiesinga, E., Mohr, P. J., Newell, D. B. and Taylor, B. N. 2021, Rev. Mod. Phys. 93, 025010: CODATA recommended values of the fundamental physical constants, 2018", "10.1103/RevModPhys.93.025010", None, "verified"),
    "si2019": ("The International System of Units (SI), 9th edition 2019, BIPM: the seven defining constants are exact (CODATA 2018 Table I)", None, "https://www.bipm.org/en/publications/si-brochure", "verified"),
    "iau2015b3": ("Prša, A. et al. 2016, AJ 152, 41: nominal values for selected solar and planetary quantities, IAU 2015 Resolution B3", "10.3847/0004-6256/152/2/41", None, "verified"),
    "iau2012b2": ("IAU 2012 Resolution B2: re-definition of the astronomical unit of length, 149 597 870 700 m exactly", None, "https://www.iau.org/static/resolutions/IAU2012_English.pdf", "unverified"),
    "iau2015b2": ("IAU 2015 Resolution B2: the parsec is 648000/π au", None, "https://www.iau.org/static/resolutions/IAU2015_English.pdf", "unverified"),
    "iupac2021": ("Prohaska, T. et al. 2022, Pure Appl. Chem. 94, 573: standard atomic weights of the elements 2021 (IUPAC Technical Report)", "10.1515/pac-2019-0603", None, "verified"),
    "lodders2021": ("Lodders, K. 2021, Space Sci. Rev. 217, 44: relative atomic solar system abundances, mass fractions, and atomic masses of the elements and their isotopes (Table 3, proto-solar columns)", "10.1007/s11214-021-00825-8", None, "verified"),
    "lodders2003": ("Lodders, K. 2003, ApJ 591, 1220: solar system abundances and condensation temperatures of the elements", "10.1086/375492", None, "verified"),
    "oberg2011": ("Öberg, K. I., Murray-Clay, R. and Bergin, E. A. 2011, ApJL 743, L16: the effects of snowlines on C/O in planetary atmospheres (H2O 135 K, CO2 47 K, CO 20 K)", "10.1088/2041-8205/743/1/L16", None, "unverified"),
    "nubase2020": ("Kondev, F. G., Wang, M., Huang, W. J., Naimi, S. and Audi, G. 2021, Chinese Phys. C 45, 030001: the NUBASE2020 evaluation of nuclear physics properties", "10.1088/1674-1137/abddae", None, "verified"),
    "ruedas2017": ("Ruedas, T. 2017, Geochem. Geophys. Geosyst. 18, 3530: radioactive heat production of six geologically important nuclides", "10.1002/2017GC006997", None, "verified"),
    "kita2013": ("Kita, N. T. et al. 2013, Meteoritics and Planetary Science 48, 1383: 26Al-26Mg isotope systematics of the first solids in the early solar system", "10.1111/maps.12141", None, "verified"),
    "nist-webbook": ("NIST Chemistry WebBook, NIST Standard Reference Database 69 (phase-change data and thermochemistry per substance)", None, "https://webbook.nist.gov/chemistry/", "textbook"),
    "pdg-gas": ("Particle Data Group, Atomic and Nuclear Properties of Materials: gas refractive indices (n-1) at 0 °C, 1 atm, 589.3 nm, from Kaye and Laby", None, "https://pdg.lbl.gov/2023/AtomicNuclearProperties/explain_gas.html", "verified"),
    "kaye-laby": ("Kaye and Laby, Tables of Physical and Chemical Constants, section 2.5.7 refractive index of gases (online edition, NPL; now offline)", None, "https://pdg.lbl.gov/2023/AtomicNuclearProperties/explain_gas.html", "unverified"),
    "peck-reeder1972": ("Peck, E. R. and Reeder, K. 1972, J. Opt. Soc. Am. 62, 958: dispersion of air", "10.1364/JOSA.62.000958", None, "unverified"),
    "bodhaine1999": ("Bodhaine, B. A., Wood, N. B., Dutton, E. G. and Slusser, J. R. 1999, J. Atmos. Oceanic Technol. 16, 1854: on Rayleigh optical depth calculations (King factors)", "10.1175/1520-0426(1999)016<1854:ORODC>2.0.CO;2", None, "verified"),
    "thauer1977": ("Thauer, R. K., Jungermann, K. and Decker, K. 1977, Bacteriological Reviews 41, 100: energy conservation in chemotrophic anaerobic bacteria (appendix: free energies and enthalpies of formation at 298.15 K)", None, "https://pmc.ncbi.nlm.nih.gov/articles/PMC413997", "verified"),
    "convention": ("Elements in their standard state and the hydrogen ion at unit activity have zero free energy and enthalpy of formation by convention", None, "https://goldbook.iupac.org/", "textbook"),
    "nssdca-earth": ("NASA NSSDCA Earth fact sheet (volumetric mean radius 6371.000 km)", None, "https://nssdc.gsfc.nasa.gov/planetary/factsheet/earthfact.html", "verified"),
    "nssdca-moon": ("NASA NSSDCA Moon fact sheet (mass 0.07346e24 kg, mean radius 1737.4 km, semi-major axis 0.3844e6 km)", None, "https://nssdc.gsfc.nasa.gov/planetary/factsheet/moonfact.html", "verified"),
    "nssdca-jupiter": ("NASA NSSDCA Jupiter fact sheet (volumetric mean radius 69911 km)", None, "https://nssdc.gsfc.nasa.gov/planetary/factsheet/jupiterfact.html", "verified"),
    "cgpm1901": ("3rd CGPM 1901, Resolution 2: standard acceleration of gravity 980.665 cm/s2", None, "https://www.bipm.org/en/committees/cg/cgpm/3-1901/resolution-2", "textbook"),
    "cgpm1954": ("10th CGPM 1954, Resolution 4: standard atmosphere 101 325 Pa", None, "https://www.bipm.org/en/committees/cg/cgpm/10-1954/resolution-4", "textbook"),
    "petrenko1999": ("Petrenko, V. F. and Whitworth, R. W. 1999, Physics of Ice, Oxford University Press (ice Ih density and conductivity k = 651/T)", "10.1093/acprof:oso/9780198518945.001.0001", None, "verified"),
    "lineweaver2010": ("Lineweaver, C. H. and Norman, M. 2010, Proc. 9th Australian Space Science Conference: the potato radius, rock and ice yield strengths (calibrated stand-in; docs/fudges.md)", None, "https://arxiv.org/abs/1004.1091", "unverified"),
    "atom-isotropic": ("A spherically symmetric atom has no anisotropic polarisability, so its King correction factor is exactly 1 (Bodhaine et al. 1999 discussion of depolarisation)", "10.1175/1520-0426(1999)016<1854:ORODC>2.0.CO;2", None, "textbook"),
    "mastin2009": ("Mastin, L. G. et al. 2009, J. Volcanol. Geotherm. Res. 186, 10: eruption column heights, magma at 2500 kg/m3 dense-rock equivalent", "10.1016/j.jvolgeores.2009.01.008", None, "verified"),
    "ussa1976": ("U.S. Standard Atmosphere 1976, NOAA/NASA/USAF (sea-level dry-air composition by volume)", None, "https://ntrs.nasa.gov/citations/19770009539", "verified"),
    "kobayashi2020": ("Kobayashi, C., Karakas, A. I. and Lugaro, M. 2020, ApJ 900, 179: the origin of elements from carbon to uranium (half of C, N and F from AGB stars; half of the iron-peak elements from Type Ia supernovae; the r-process shortfall)", "10.3847/1538-4357/abae65", None, "verified"),
    "johnson-origins": ("Johnson, J. A., Origin of the Elements in the Solar System: periodic table colour-coded by origin (Ohio State University, CC BY-SA 4.0; the six-source legend)", None, "https://www.astronomy.ohio-state.edu/johnson.3064/nucleo/index.html", "verified"),
    "bisterzo2014": ("Bisterzo, S. et al. 2014, ApJ 787, 10: Galactic chemical evolution and solar s-process abundances, dependence on the 13C-pocket structure (s-process percentages per isotope, Kr to Bi)", "10.1088/0004-637X/787/1/10", None, "verified"),
    "kasen2017": ("Kasen, D. et al. 2017, Nature 551, 80: origin of the heavy elements in binary neutron-star mergers from a gravitational-wave event (the GW170817 kilonova)", "10.1038/nature24453", None, "verified"),
    "abbott2017": ("Abbott, B. P. et al. (LIGO and Virgo) 2017, Phys. Rev. Lett. 119, 161101: GW170817, observation of gravitational waves from a binary neutron star inspiral", "10.1103/PhysRevLett.119.161101", None, "verified"),
    "fields2020": ("Fields, B. D., Olive, K. A., Yeh, T.-H. and Young, C. 2020, JCAP 03, 010: Big-Bang nucleosynthesis after Planck (primordial helium and the lithium problem)", None, "https://arxiv.org/abs/1912.01132", "verified"),
    "fudge-register": ("SSE4 fudge register: an authored stand-in kept because no measured value fits (reason in docs/fudges.md)", None, "https://github.com/FrunkQ/star-system-explorer/blob/main/docs/fudges.md", "authored"),
}

# ---------------------------------------------------------------- constants
G = 6.67430e-11
GM_SUN = 1.3271244e20
GM_EARTH = 3.986004e14
GM_JUP = 1.2668653e17
AU = 149597870700.0
K_B = 1.380649e-23
N_A = 6.02214076e23
H = 6.62607015e-34
C = 299792458.0
PI = math.pi
SIGMA = 2 * PI**5 * K_B**4 / (15 * H**3 * C**2)

CONSTANTS = [
    # Constants of nature (CODATA 2018). The seven SI defining constants are exact.
    ("c", "speed of light in vacuum", val(C, "m s-1", "si2019", exact=True)),
    ("h", "Planck constant", val(H, "J s", "si2019", exact=True)),
    ("hbar", "reduced Planck constant", val(1.054571817e-34, "J s", "codata2018", note="CODATA 2018 published figure"), "h / (2 pi)"),
    ("e", "elementary charge", val(1.602176634e-19, "C", "si2019", exact=True)),
    ("k_B", "Boltzmann constant", val(K_B, "J K-1", "si2019", exact=True)),
    ("N_A", "Avogadro constant", val(N_A, "mol-1", "si2019", exact=True)),
    ("R_gas", "molar gas constant", val(N_A * K_B, "J mol-1 K-1", "si2019"), "N_A k_B"),
    ("sigma_SB", "Stefan-Boltzmann constant", val(5.670374419e-8, "W m-2 K-4", "codata2018", note="CODATA 2018 published figure; the worked-out value agrees to 3e-11"), "2 pi^5 k_B^4 / (15 h^3 c^2)"),
    ("a_rad", "radiation density constant", val(4 * SIGMA / C, "J m-3 K-4", "si2019"), "4 sigma_SB / c"),
    ("G", "Newtonian constant of gravitation", val(G, "m3 kg-1 s-2", "codata2018", "unverified", uncertainty=1.5e-15)),
    ("m_e", "electron mass", val(9.1093837015e-31, "kg", "codata2018", "unverified")),
    ("m_p", "proton mass", val(1.67262192369e-27, "kg", "codata2018", "unverified")),
    ("m_n", "neutron mass", val(1.67492749804e-27, "kg", "codata2018", "unverified")),
    ("m_u", "atomic mass constant", val(1.66053906660e-27, "kg", "codata2018", "unverified")),
    ("alpha", "fine-structure constant", val(7.2973525693e-3, "1", "codata2018", "unverified")),
    ("eps0", "vacuum electric permittivity", val(8.8541878128e-12, "F m-1", "codata2018", "unverified")),
    ("mu0", "vacuum magnetic permeability", val(1.25663706212e-6, "N A-2", "codata2018", "unverified")),
    ("R_inf", "Rydberg constant", val(10973731.568160, "m-1", "codata2018", "unverified")),
    ("a0", "Bohr radius", val(5.29177210903e-11, "m", "codata2018", "unverified")),
    ("sigma_T", "Thomson cross section", val(6.6524587321e-29, "m2", "codata2018", "unverified")),
    ("b_wien", "Wien wavelength displacement constant", val(2.897771955e-3, "m K", "codata2018", "unverified")),
    # Units and conventions.
    ("au", "astronomical unit", val(AU, "m", "iau2012b2", "unverified", exact=True)),
    ("pc", "parsec", val(AU * 648000 / PI, "m", "iau2015b2", "unverified"), "au 648000 / pi"),
    ("yr_julian", "Julian year", val(JULIAN_YEAR_S, "s", "iau2012b2", "textbook", exact=True)),
    ("ly", "light year", val(C * JULIAN_YEAR_S, "m", "iau2012b2", "textbook"), "c yr_julian"),
    ("day", "day", val(86400.0, "s", "si2019", exact=True)),
    ("atm", "standard atmosphere", val(101325.0, "Pa", "cgpm1954", "textbook", exact=True)),
    ("g0", "standard acceleration of gravity", val(9.80665, "m s-2", "cgpm1901", "textbook", exact=True)),
    ("T0C", "zero degrees Celsius", val(273.15, "K", "si2019", exact=True)),
    # Reference objects (IAU 2015 B3 nominal values are exact conversion factors).
    ("GM_sun", "nominal solar mass parameter", val(GM_SUN, "m3 s-2", "iau2015b3", exact=True)),
    ("M_sun", "solar mass", val(GM_SUN / G, "kg", "iau2015b3"), "GM_sun / G"),
    ("R_sun", "nominal solar radius", val(6.957e8, "m", "iau2015b3", exact=True)),
    ("L_sun", "nominal solar luminosity", val(3.828e26, "W", "iau2015b3", exact=True)),
    ("T_sun", "nominal solar effective temperature", val(5772.0, "K", "iau2015b3", exact=True)),
    ("S_sun", "nominal total solar irradiance", val(1361.0, "W m-2", "iau2015b3", exact=True)),
    ("GM_earth", "nominal Earth mass parameter", val(GM_EARTH, "m3 s-2", "iau2015b3", exact=True)),
    ("M_earth", "Earth mass", val(GM_EARTH / G, "kg", "iau2015b3"), "GM_earth / G"),
    ("R_earth_equatorial", "nominal Earth equatorial radius", val(6.3781e6, "m", "iau2015b3", exact=True)),
    ("R_earth_polar", "nominal Earth polar radius", val(6.3568e6, "m", "iau2015b3", exact=True)),
    ("R_earth_mean", "Earth volumetric mean radius", val(6.371e6, "m", "nssdca-earth")),
    ("GM_jup", "nominal Jupiter mass parameter", val(GM_JUP, "m3 s-2", "iau2015b3", exact=True)),
    ("M_jup", "Jupiter mass", val(GM_JUP / G, "kg", "iau2015b3"), "GM_jup / G"),
    ("R_jup_equatorial", "nominal Jupiter equatorial radius", val(7.1492e7, "m", "iau2015b3", exact=True)),
    ("R_jup_polar", "nominal Jupiter polar radius", val(6.6854e7, "m", "iau2015b3", exact=True)),
    ("R_jup_mean", "Jupiter volumetric mean radius", val(6.9911e7, "m", "nssdca-jupiter")),
    ("M_moon", "Moon mass", val(7.346e22, "kg", "nssdca-moon")),
    ("R_moon", "Moon mean radius", val(1.7374e6, "m", "nssdca-moon")),
    ("a_moon", "Moon semi-major axis", val(3.844e8, "m", "nssdca-moon")),
]


# ---------------------------------------------------------------- units (design section 7A.6)
# (id, symbol, name, measures, group, factor_from formula | ("v", value, src), offset)
UNITS = [
    ("one", "", "one", "dimensionless", "si-base", ("v", 1.0, "si2019")),
    ("m", "m", "metre", "length", "si-base", ("v", 1.0, "si2019")),
    ("kg", "kg", "kilogram", "mass", "si-base", ("v", 1.0, "si2019")),
    ("s", "s", "second", "time", "si-base", ("v", 1.0, "si2019")),
    ("a", "A", "ampere", "current", "si-base", ("v", 1.0, "si2019")),
    ("k", "K", "kelvin", "temperature", "si-base", ("v", 1.0, "si2019")),
    ("mol", "mol", "mole", "amount", "si-base", ("v", 1.0, "si2019")),
    ("cd", "cd", "candela", "luminous_intensity", "si-base", ("v", 1.0, "si2019")),
    ("n", "N", "newton", "force", "si-named", ("v", 1.0, "si2019")),
    ("pa", "Pa", "pascal", "pressure", "si-named", ("v", 1.0, "si2019")),
    ("j", "J", "joule", "energy", "si-named", ("v", 1.0, "si2019")),
    ("w", "W", "watt", "power", "si-named", ("v", 1.0, "si2019")),
    ("hz", "Hz", "hertz", "frequency", "si-named", ("v", 1.0, "si2019")),
    ("t", "T", "tesla", "magnetic_field", "si-named", ("v", 1.0, "si2019")),
    ("km", "km", "kilometre", "length", "working", ("v", 1e3, "si2019")),
    ("au", "au", "astronomical unit", "length", "working", "au"),
    ("pc", "pc", "parsec", "length", "working", "au 648000 / pi"),
    ("ly", "ly", "light year", "length", "working", "c yr_julian"),
    ("r_sun", "R☉", "solar radius", "length", "working", "R_sun"),
    ("r_earth", "R⊕", "Earth radius (equatorial)", "length", "working", "R_earth_equatorial"),
    ("r_jupiter", "RJ", "Jupiter radius (equatorial)", "length", "working", "R_jup_equatorial"),
    ("m_sun", "M☉", "solar mass", "mass", "working", "M_sun"),
    ("m_earth", "M⊕", "Earth mass", "mass", "working", "M_earth"),
    ("m_jupiter", "MJ", "Jupiter mass", "mass", "working", "M_jup"),
    ("day", "d", "day", "time", "working", "day"),
    ("yr", "yr", "Julian year", "time", "working", "yr_julian"),
    ("myr", "Myr", "million years", "time", "working", "1e6 yr_julian"),
    ("gyr", "Gyr", "billion years", "time", "working", "1e9 yr_julian"),
    ("degc", "°C", "degree Celsius", "temperature", "working", ("v", 1.0, "si2019"), 273.15),
    ("l_sun", "L☉", "solar luminosity", "power", "working", "L_sun"),
    ("w_m2", "W/m²", "watt per square metre", "flux", "working", ("v", 1.0, "si2019")),
    ("bar", "bar", "bar", "pressure", "working", ("v", 1e5, "si2019")),
    ("atm", "atm", "standard atmosphere", "pressure", "working", "atm"),
    ("ev", "eV", "electronvolt", "energy", "working", "e"),
    ("gee", "g", "standard gravity", "acceleration", "working", "g0"),
    ("degree", "°", "degree of arc", "angle", "working", "pi / 180"),
    ("arcsec", "″", "arcsecond", "angle", "working", "pi / 648000"),
    ("m_s", "m/s", "metre per second", "velocity", "working", ("v", 1.0, "si2019")),
    ("km_s", "km/s", "kilometre per second", "velocity", "working", ("v", 1e3, "si2019")),
    ("m_s2", "m/s²", "metre per second squared", "acceleration", "working", ("v", 1.0, "si2019")),
    ("kg_m3", "kg/m³", "kilogram per cubic metre", "density", "working", ("v", 1.0, "si2019")),
    ("g_cm3", "g/cm³", "gram per cubic centimetre", "density", "working", ("v", 1e3, "si2019")),
    ("ut", "µT", "microtesla", "magnetic_field", "working", ("v", 1e-6, "si2019")),
    ("gauss", "G", "gauss", "magnetic_field", "working", ("v", 1e-4, "si2019")),
    ("a_m2", "A m²", "ampere square metre", "magnetic_moment", "working", ("v", 1.0, "si2019")),
]

CONVENTION_IDS = {"au", "yr_julian", "day", "atm", "g0", "T0C", "GM_sun", "R_sun", "L_sun", "T_sun", "S_sun",
                  "GM_earth", "R_earth_equatorial", "R_earth_polar", "GM_jup", "R_jup_equatorial", "R_jup_polar"}
FACTOR_IDS = {"pc", "ly"}


def units():
    out = []
    for u in UNITS:
        uid, sym, name, measures, group, f = u[:6]
        row = {"id": uid, "symbol": sym, "name": name, "measures": measures, "group": group}
        if isinstance(f, tuple):
            row["factor"] = {"v": f[1], "src": f[2]}
        else:
            row["factor_from"] = f
        if len(u) > 6:
            row["offset"] = u[6]
        out.append(row)
    return out


# ---------------------------------------------------------------- isotopes
def iso(id_, element, a, half_life_yr=None, src="nubase2020", check="unverified", **extra):
    row = {"id": id_, "element": element, "a": a}
    if half_life_yr is None:
        row["stable"] = True
    else:
        row["half_life_s"] = val(half_life_yr * JULIAN_YEAR_S, "s", src, check, note="year taken as the Julian year")
    row.update(extra)
    return row


ISOTOPES = [
    # The heat producers the code reads today (sse-atoms heat.rs; Ruedas 2017, checked in the citations pass).
    iso("u238", "u", 238, 4.468e9, "ruedas2017", "verified", daughter="pb206",
        share_today=val(0.9929, "1", "ruedas2017", note="mass share of the element today"),
        decay_heat_w_per_kg=val(9.46e-5, "W kg-1", "ruedas2017")),
    iso("u235", "u", 235, 7.04e8, "ruedas2017", "verified", daughter="pb207",
        share_today=val(0.00711, "1", "ruedas2017", note="mass share of the element today"),
        decay_heat_w_per_kg=val(5.69e-4, "W kg-1", "ruedas2017")),
    iso("th232", "th", 232, 1.405e10, "nubase2020", "unverified", daughter="pb208", note="ENSDF 14.05 Gyr as the code holds; Ruedas 2017 lists 14.0 Gyr",
        share_today=val(1.0, "1", "ruedas2017"),
        decay_heat_w_per_kg=val(2.64e-5, "W kg-1", "ruedas2017")),
    iso("k40", "k", 40, 1.248e9, "ruedas2017", "verified", daughter="ar40 ca40",
        share_today=val(1.19e-4, "1", "ruedas2017", note="mass share of the element today"),
        decay_heat_w_per_kg=val(2.92e-5, "W kg-1", "ruedas2017")),
    iso("al26", "al", 26, 7.17e5, "ruedas2017", "verified", daughter="mg26",
        share_at_sun_birth=val(5.25e-5, "1", "kita2013", note="26Al/27Al by number in the first solids"),
        decay_heat_w_per_kg=val(0.3583, "W kg-1", "ruedas2017")),
    # Fuels.
    iso("h1", "h", 1), iso("h2", "h", 2), iso("h3", "h", 3, 12.32, daughter="he3"),
    iso("he3", "he", 3), iso("he4", "he", 4), iso("li6", "li", 6), iso("li7", "li", 7),
    # Stable references and clocks.
    iso("c12", "c", 12), iso("c13", "c", 13), iso("c14", "c", 14, 5700.0, daughter="n14"),
    iso("n14", "n", 14), iso("n15", "n", 15), iso("o16", "o", 16), iso("o18", "o", 18),
    iso("be10", "be", 10, 1.387e6, daughter="b10"),
    iso("cl36", "cl", 36, 3.01e5, daughter="ar36"),
    iso("fe60", "fe", 60, 2.62e6, daughter="ni60"),
    iso("i129", "i", 129, 1.61e7, daughter="xe129"),
    iso("hf182", "hf", 182, 8.9e6, daughter="w182"),
    iso("pu244", "pu", 244, 8.13e7, daughter="fission"),
    iso("rb87", "rb", 87, 4.97e10, daughter="sr87"),
    iso("sm147", "sm", 147, 1.06e11, daughter="nd143"),
    iso("u234", "u", 234, 2.455e5, daughter="th230"),
    iso("th230", "th", 230, 7.54e4, daughter="ra226"),
    iso("ra226", "ra", 226, 1600.0, daughter="rn222"),
    iso("rn222", "rn", 222, 3.8235 / 365.25, daughter="po218"),
    iso("pb206", "pb", 206), iso("pb207", "pb", 207), iso("pb208", "pb", 208),
]

# ---------------------------------------------------------------- substances
REF_T = 298.15


def gibbs(dg, dh, src):
    return {"dgf": val(dg * 1000.0, "J mol-1", src, at_k=REF_T), "dhf": val(dh * 1000.0, "J mol-1", src, at_k=REF_T)}


def substance(id_, formula, physics, plain=None, story=None, charge=0, **groups):
    row = {"id": id_, "formula": formula}
    if charge:
        row["charge"] = charge
    row["names"] = {"physics": physics, "plain": plain or physics, "story": story or plain or physics}
    for k in ("phases", "thermal", "mechanical", "optical", "chemical", "cosmochemical"):
        if k in groups:
            row[k] = groups[k]
    return row


def gas_optics(n589, king, src_n, check_n="verified", king_src="bodhaine1999", king_check="verified"):
    return {
        "refractivity_589nm": val(n589, "1", src_n, check_n, at_k=273.15, at_pa=101325.0),
        "king_factor": {"fit": "a + b lambda^-2 + c lambda^-4, lambda in um", "coeffs": king, "src": king_src, **({} if king_check == "verified" else {"check": king_check})},
    }


KING_ONE = ("fudge-register", "authored")  # King factor 1: molecular depolarisation not yet tabulated
T = "thauer1977"
NIST = "nist-webbook"

SUBSTANCES = [
    substance("h2o", {"h": 2, "o": 1}, "water", plain="water", story="water",
              phases={
                  "triple_point_k": val(273.16, "K", NIST),
                  "triple_point_pa": val(611.657, "Pa", NIST),
                  "latent_vaporisation": val(2.468e6, "J kg-1", NIST, "verified", at_k=288.0, note="from the slope of the NIST Antoine fit, 273-303 K row"),
                  "density_liquid": val(1000.0, "kg m-3", NIST, "verified", at_k=277.0),
                  "density_solid": val(917.0, "kg m-3", "petrenko1999", "verified", at_k=273.15, note="ice Ih"),
              },
              thermal={"conductivity_solid": {"fit": "k = a / T", "coeffs": [651.0], "valid_k": [100.0, 273.15], "unit": "W m-1 K-1", "src": "petrenko1999"}},
              optical={**gas_optics(2.56e-4, [1.0, 0.0, 0.0], "kaye-laby", "unverified", *KING_ONE), "condensed_colour_rgb": val([0.95, 0.95, 0.97], "1", "fudge-register", "authored")},
              chemical={"states": {"liquid": gibbs(-237.18, -285.83, T)}}),
    substance("co2", {"c": 1, "o": 2}, "carbon dioxide", plain="carbon dioxide", story="carbon dioxide",
              phases={
                  "triple_point_k": val(216.58, "K", NIST),
                  "triple_point_pa": val(518500.0, "Pa", NIST),
                  "latent_sublimation": val(573000.0, "J kg-1", NIST, at_k=195.0, note="25.2 kJ/mol"),
                  "density_solid": val(1560.0, "kg m-3", NIST, "unverified"),
              },
              optical={**gas_optics(4.49e-4, [1.15, 0.0, 0.0], "pdg-gas"), "condensed_colour_rgb": val([0.97, 0.97, 1.0], "1", "fudge-register", "authored")},
              chemical={"states": {"gas": gibbs(-394.36, -393.51, T), "dissolved": gibbs(-386.02, -413.8, T)}},
              cosmochemical={"ice_line_k": val(47.0, "K", "oberg2011", "unverified")}),
    substance("ch4", {"c": 1, "h": 4}, "methane",
              phases={
                  "triple_point_k": val(90.67, "K", NIST, "unverified", note="NIST lists 90.69 K"),
                  "triple_point_pa": val(11690.0, "Pa", NIST),
                  "latent_vaporisation": val(531000.0, "J kg-1", NIST, at_k=99.5, note="8.519 kJ/mol"),
                  "density_liquid": val(450.0, "kg m-3", NIST, "unverified", at_k=90.7),
              },
              optical=gas_optics(4.44e-4, [1.0, 0.0, 0.0], "kaye-laby", "unverified", *KING_ONE),
              chemical={"states": {"gas": gibbs(-50.75, -74.81, T), "dissolved": gibbs(-34.39, -89.04, T)}}),
    substance("nh3", {"n": 1, "h": 3}, "ammonia",
              phases={
                  "triple_point_k": val(195.4, "K", NIST, "unverified", note="the climate pack holds 194.95 K, the melting point"),
                  "triple_point_pa": val(6060.0, "Pa", NIST),
                  "latent_sublimation": val(1832000.0, "J kg-1", NIST, note="31.2 kJ/mol"),
                  "density_solid": val(820.0, "kg m-3", NIST, "unverified"),
              },
              optical=gas_optics(3.76e-4, [1.0, 0.0, 0.0], "pdg-gas", "verified", *KING_ONE)),
    substance("n2", {"n": 2}, "nitrogen",
              phases={
                  "triple_point_k": val(63.14, "K", NIST),
                  "triple_point_pa": val(12520.0, "Pa", NIST),
                  "latent_vaporisation": val(200000.0, "J kg-1", NIST, at_k=77.0, note="5.6 kJ/mol"),
                  "density_liquid": val(800.0, "kg m-3", NIST, "unverified", at_k=77.0, note="808 kg/m3 at 77 K"),
              },
              optical=gas_optics(2.98e-4, [1.034, 3.17e-4, 0.0], "kaye-laby", "unverified"),
              chemical={"states": {"gas": gibbs(0.0, 0.0, "convention")}}),
    substance("o2", {"o": 2}, "oxygen",
              optical=gas_optics(2.71e-4, [1.096, 1.385e-3, 1.448e-4], "kaye-laby", "unverified"),
              chemical={"states": {"gas": gibbs(0.0, 0.0, "convention"), "dissolved": gibbs(16.32, -11.7, T)}}),
    substance("ar", {"ar": 1}, "argon", optical=gas_optics(2.81e-4, [1.0, 0.0, 0.0], "kaye-laby", "unverified", king_src="atom-isotropic")),
    substance("h2", {"h": 2}, "hydrogen",
              optical=gas_optics(1.32e-4, [1.0, 0.0, 0.0], "kaye-laby", "unverified", *KING_ONE),
              chemical={"states": {"gas": gibbs(0.0, 0.0, "convention"), "dissolved": gibbs(17.72, -4.2, T)}}),
    substance("he", {"he": 1}, "helium", optical=gas_optics(3.5e-5, [1.0, 0.0, 0.0], "kaye-laby", "unverified", king_src="atom-isotropic")),
    substance("ne", {"ne": 1}, "neon", optical=gas_optics(6.71e-5, [1.0, 0.0, 0.0], "pdg-gas", "verified", king_src="atom-isotropic")),
    substance("kr", {"kr": 1}, "krypton"),
    substance("xe", {"xe": 1}, "xenon"),
    substance("so2", {"s": 1, "o": 2}, "sulphur dioxide", optical=gas_optics(6.86e-4, [1.0, 0.0, 0.0], "kaye-laby", "unverified", *KING_ONE)),
    substance("co", {"c": 1, "o": 1}, "carbon monoxide",
              optical=gas_optics(3.38e-4, [1.0, 0.0, 0.0], "kaye-laby", "unverified", *KING_ONE),
              chemical={"states": {"gas": gibbs(-137.15, -110.53, T)}},
              cosmochemical={"ice_line_k": val(20.0, "K", "oberg2011", "unverified")}),
    substance("h2s", {"h": 2, "s": 1}, "hydrogen sulfide", chemical={"states": {"dissolved": gibbs(-27.87, -39.7, T)}}),
    substance("h+", {"h": 1}, "hydrogen ion", charge=1, chemical={"states": {"dissolved": gibbs(0.0, 0.0, "convention")}}),
    substance("hco3-", {"h": 1, "c": 1, "o": 3}, "bicarbonate", charge=-1, chemical={"states": {"dissolved": gibbs(-586.85, -691.99, T)}}),
    substance("hs-", {"h": 1, "s": 1}, "bisulfide", charge=-1, chemical={"states": {"dissolved": gibbs(12.05, -17.6, T)}}),
    substance("s", {"s": 1}, "sulfur", chemical={"states": {"solid": gibbs(0.0, 0.0, "convention")}}),
    substance("so4-2", {"s": 1, "o": 4}, "sulfate", charge=-2, chemical={"states": {"dissolved": gibbs(-744.6, -909.27, T)}}),
    substance("nh4+", {"n": 1, "h": 4}, "ammonium", charge=1, chemical={"states": {"dissolved": gibbs(-79.37, -132.51, T)}}),
    substance("no2-", {"n": 1, "o": 2}, "nitrite", charge=-1, chemical={"states": {"dissolved": gibbs(-37.2, -104.6, T)}}),
    substance("no3-", {"n": 1, "o": 3}, "nitrate", charge=-1, chemical={"states": {"dissolved": gibbs(-111.34, -205.0, T)}}),
    substance("fe+2", {"fe": 1}, "ferrous iron", charge=2, chemical={"states": {"dissolved": gibbs(-78.87, -89.1, T)}}),
    substance("fe+3", {"fe": 1}, "ferric iron", charge=3, chemical={"states": {"dissolved": gibbs(-4.6, -48.5, T)}}),
    substance("acetate-", {"c": 2, "h": 3, "o": 2}, "acetate", charge=-1, chemical={"states": {"dissolved": gibbs(-369.41, -486.01, T)}}),
    substance("glucose", {"c": 6, "h": 12, "o": 6}, "glucose", chemical={"states": {"dissolved": gibbs(-917.22, -1262.19, T)}}),
    substance("ethanol", {"c": 2, "h": 6, "o": 1}, "ethanol", chemical={"states": {"dissolved": gibbs(-181.75, -288.3, T)}}),
    substance("ch2o", {"c": 1, "h": 2, "o": 1}, "organic carbon (one sixth of glucose)", plain="organic matter",
              chemical={"states": {"dissolved": gibbs(-152.87, -210.37, T)}}),
]

# ---------------------------------------------------------------- mixtures
MIXTURES = [
    {"id": "dry-air", "names": {"physics": "dry air", "plain": "air", "story": "air"},
     "recipe": {"n2": 0.78084, "o2": 0.209476, "ar": 0.00934, "co2": 0.000314, "ne": 1.818e-5, "he": 5.24e-6, "ch4": 2e-6, "kr": 1.14e-6, "h2": 5e-7, "xe": 8.7e-8}, "by": "volume", "src": "ussa1976", "check": "unverified",
     "note": "molar mass is worked out from the recipe (the 1976 table gives 28.9644 g/mol); CO2 has risen since 1976",
     "optical": {"refractivity_589nm": val(2.926e-4, "1", "peck-reeder1972", "unverified", at_k=273.15, at_pa=101325.0), "dispersion": "peck-reeder",
                 "king_factor": {"fit": "a + b lambda^-2 + c lambda^-4, lambda in um", "coeffs": [1.0469, 0.0, 0.0], "src": "bodhaine1999"}}},
    {"id": "giant-air", "names": {"physics": "hydrogen-helium envelope", "plain": "giant-planet air", "story": "giant air"},
     "recipe": {"h2": 0.86, "he": 0.14}, "by": "volume", "src": "fudge-register", "check": "authored",
     "note": "the weather pack's giant air at 0.0023 kg/mol; the reader's worked-out molar mass replaces it"},
    {"id": "surface-rock", "names": {"physics": "generic silicate rock", "plain": "rock", "story": "rock"},
     "recipe": {}, "by": "mass", "density": val(3500.0, "kg m-3", "fudge-register", "authored"),
     "yield_strength": val(1.54e8, "Pa", "lineweaver2010", "unverified", note="calibrated so rock rounds at about 300 km")},
    {"id": "surface-ice", "names": {"physics": "water ice with some rock", "plain": "ice", "story": "ice"},
     "recipe": {"h2o": 1.0}, "by": "mass", "density": val(930.0, "kg m-3", "fudge-register", "authored", note="pure ice Ih is 917 kg/m3 (water's density_solid)"),
     "yield_strength": val(5.6e6, "Pa", "lineweaver2010", "unverified", note="calibrated so ice rounds at about 200 km"),
     "melting_point": val(273.15, "K", NIST, note="at low pressure")},
    {"id": "magma-dre", "names": {"physics": "magma, dense-rock equivalent", "plain": "magma", "story": "magma"},
     "recipe": {}, "by": "mass", "density": val(2500.0, "kg m-3", "mastin2009")},
]
# The six formation materials (sse-formation disc.rs and leftovers.rs).
for gid, name, t_k, t_src, rho, share in [
    ("formation-metal", "iron and nickel", 1400.0, "lodders2003", 7800.0, 0.0018),
    ("formation-rock", "silicates and oxides", 1400.0, "lodders2003", 3000.0, 0.0035),
    ("formation-carbon", "refractory carbon and organics", 500.0, "fudge-register", 2000.0, 0.0010),
    ("formation-water", "water ice", 135.0, "oberg2011", 1000.0, 0.0045),
    ("formation-co2", "carbon dioxide ice", 47.0, "oberg2011", 1500.0, 0.0008),
    ("formation-co", "carbon monoxide ice", 20.0, "oberg2011", 900.0, 0.0022),
]:
    MIXTURES.append({
        "id": gid, "names": {"physics": name, "plain": name, "story": name}, "recipe": {}, "by": "mass",
        "cosmochemical": {
            "condenses_below_k": val(t_k, "K", t_src, "verified" if t_src == "lodders2003" else ("authored" if t_src == "fudge-register" else "unverified")),
            "share_of_solar_gas": val(share, "1", "fudge-register", "authored", note="split of protosolar Z by carrier (Asplund 2009, Oberg 2011)"),
        },
        "density": val(rho, "kg m-3", "fudge-register", "authored"),
    })


# ---------------------------------------------------------------- elements
ORIGINS = element_origins.load(ORIGINS_CSV)


def elements():
    rows = []
    with open(ELEMENTS_CSV, newline="") as f:
        for r in csv.DictReader(f):
            row = {"id": r["id"], "symbol": r["symbol"], "name": r["name"], "z": int(r["z"])}
            if r["atomic_weight"]:
                w = {"v": float(r["atomic_weight"]), "unit": "1", "src": "iupac2021"}
                if r["weight_note"]:
                    w["note"] = r["weight_note"]
                row["atomic_weight"] = w
            else:
                row["no_standard_weight"] = True
            if r["n_protosolar_si1e6"]:
                ab = {"v": float(r["n_protosolar_si1e6"]), "unit": "atoms per 1e6 Si", "src": "lodders2021"}
                if r["n_sigma"] not in ("", "n/a"):
                    ab["sigma"] = float(r["n_sigma"])
                if r["check"]:
                    ab["check"] = "unverified"
                    ab["note"] = r["check"]
                row["abundance_protosolar"] = ab
            o = ORIGINS[row["id"]]
            row["origin"] = element_origins.origin_value(row["id"], o)
            row["origin_story"] = element_origins.story(row["id"], row["name"], o)
            rows.append(row)
    return rows


def build():
    consts = []
    for c in CONSTANTS:
        group = "convention" if c[0] in CONVENTION_IDS else ("factor" if c[0] in FACTOR_IDS else "nature")
        row = {"id": c[0], "name": c[1], "group": group, "value": c[2]}
        if len(c) > 3:
            # A mass worked out from a measured GM is checked against it, never recomputed after an
            # edit: doubling G is a what-if with the same masses and stronger gravity.
            row["checked_by" if c[0] in ("M_sun", "M_earth", "M_jup") else "derived_from"] = c[3]
        consts.append(row)
    sources = []
    for sid, (text, doi, url, status) in sorted(SOURCES.items()):
        s = {"id": sid, "text": text, "status": status}
        if doi:
            s["doi"] = doi
        if url:
            s["url"] = url
        sources.append(s)
    return {
        "format": "sse.pack/1",
        "id": "sse.base-reference",
        "version": "0.1.0",
        "kind": ["base-physics"],
        "kernel_range": "^0.1",
        "licence": "Apache-2.0",
        "provides": {
            "reference": {
                "about": "The master data sheet (track MP, design handover/design/SSE4-matter-pack-design.md): constants of nature, conventions, reference objects, elements, isotopes, substances and mixtures, each value with its source. Built by scripts/build-base-reference.py; MP2 first wave. A value without \"check\" is verified against its source; \"unverified\" means the source is named but the number was not read from it; \"authored\" is a documented stand-in (docs/fudges.md).",
                "constants": consts,
                "units": units(),
                "origin_legend": [{"id": k, "physics": v[0], "plain": v[1], "story": v[2]} for k, v in element_origins.SOURCE_NAMES.items()],
                "elements": elements(),
                "isotopes": ISOTOPES,
                "substances": SUBSTANCES,
                "mixtures": MIXTURES,
                "sources": sources,
            }
        },
    }


def validate(pack):
    ref = pack["provides"]["reference"]
    src_ids = {s["id"] for s in ref["sources"]}
    el_ids = {e["id"] for e in ref["elements"]}
    sub_ids = {s["id"] for s in ref["substances"]}
    errors = []

    def walk(o, where):
        if isinstance(o, dict):
            if "src" in o and isinstance(o["src"], str) and o["src"] not in src_ids:
                errors.append(f"{where}: unknown source {o['src']}")
            for extra in o.get("also", []) if isinstance(o.get("also"), list) else []:
                if extra not in src_ids:
                    errors.append(f"{where}: unknown source {extra}")
            for k, v in o.items():
                walk(v, f"{where}.{k}")
        elif isinstance(o, list):
            for i, v in enumerate(o):
                walk(v, f"{where}[{i}]")

    walk(ref, "reference")
    for s in ref["substances"]:
        for el in s["formula"]:
            if el not in el_ids:
                errors.append(f"substance {s['id']}: unknown element {el}")
    for m in ref["mixtures"]:
        for sid in m["recipe"]:
            if sid not in sub_ids:
                errors.append(f"mixture {m['id']}: unknown substance {sid}")
    for i in ref["isotopes"]:
        if i["element"] not in el_ids:
            errors.append(f"isotope {i['id']}: unknown element {i['element']}")
    for table in ("constants", "units", "elements", "isotopes", "substances", "mixtures", "sources"):
        ids = [r["id"] for r in ref[table]]
        if len(ids) != len(set(ids)):
            errors.append(f"{table}: duplicate ids")
    for e in ref["elements"]:
        t = sum(e["origin"]["shares"].values())
        if abs(t - 1.0) > 1e-9:
            errors.append(f"element {e['id']}: origin shares sum to {t}")
        for k in e["origin"]["shares"]:
            if k not in element_origins.SOURCE_NAMES:
                errors.append(f"element {e['id']}: unknown origin {k}")
    if len(ref["elements"]) != 118:
        errors.append("elements: expected 118 rows")
    return errors


def main():
    pack = build()
    errors = validate(pack)
    if errors:
        print("\n".join(errors))
        sys.exit(1)
    text = json.dumps(pack, indent=1, ensure_ascii=False) + "\n"
    if "--check" in sys.argv:
        if not OUT.exists() or OUT.read_text() != text:
            print(f"{OUT.relative_to(ROOT)} is stale: run scripts/build-base-reference.py")
            sys.exit(1)
        print("base-reference pack is current")
        return
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(text)
    ref = pack["provides"]["reference"]
    print(f"wrote {OUT.relative_to(ROOT)}: " + ", ".join(f"{len(ref[t])} {t}" for t in ("constants", "elements", "isotopes", "substances", "mixtures", "sources")))


if __name__ == "__main__":
    main()
