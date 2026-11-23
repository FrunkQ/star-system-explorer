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
import water_phases  # noqa: E402
import element_affinity  # noqa: E402

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
    "fray2009": ("Fray, N. and Schmitt, B. 2009, Planet. Space Sci. 57, 2053: sublimation of ices of astrophysical interest, a bibliographic review (recommended fits ln P[bar] = A0 + sum A_i / T^i; water by Feistel and Wagner 2007). Coefficients read from Lisse et al. 2021's reproduction of the table (SOM Table 2); which fit covers which range checked against each ice's triple point", "10.1016/j.pss.2009.09.011", None, "verified"),
    "lisse2021": ("Lisse, C. M. et al. 2021, Icarus 356, 114072: on the origin and thermal stability of Arrokoth's and Pluto's ices; Supplementary Online Material Table 2 reproduces Fray and Schmitt 2009's sublimation fits", None, "https://ntrs.nasa.gov/api/citations/20210019502/downloads/Supplementary%20Online%20Material%2020210019502.pdf", "verified"),
    "ame2020": ("Wang, M., Huang, W. J., Kondev, F. G., Audi, G. and Naimi, S. 2021, Chinese Phys. C 45, 030003: the AME 2020 atomic mass evaluation (II), tables", "10.1088/1674-1137/abddaf", None, "unverified"),
    "leachman2009": ("Leachman, J. W., Jacobsen, R. T., Penoncello, S. G. and Lemmon, E. W. 2009, J. Phys. Chem. Ref. Data 38, 721: fundamental equations of state for parahydrogen, normal hydrogen and orthohydrogen", "10.1063/1.3160306", None, "unverified"),
    "schmidt-wagner1985": ("Schmidt, R. and Wagner, W. 1985, Fluid Phase Equilibria 19, 175: a new form of the equation of state for pure substances and its application to oxygen", "10.1016/0378-3812(85)87016-3", None, "unverified"),
    "richardson2014": ("Richardson, I. A., Leachman, J. W. and Lemmon, E. W. 2014, J. Phys. Chem. Ref. Data 43, 013103: fundamental equation of state for deuterium", "10.1063/1.4864752", None, "unverified"),
    "coolprop2014": ("Bell, I. H., Wronski, J., Quoilin, S. and Lemort, V. 2014, Ind. Eng. Chem. Res. 53, 2498: CoolProp, pure and pseudo-pure fluid property evaluation (MIT licence; its saturated-liquid density fits to the equations of state, dev/fluids/*.json)", "10.1021/ie4033999", "https://github.com/CoolProp/CoolProp", "unverified"),
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
    "nssdca-sun": ("NASA NSSDCA Sun fact sheet: sidereal rotation period 25.38 days (the Carrington rotation)", None, "https://nssdc.gsfc.nasa.gov/planetary/factsheet/sunfact.html", "unverified"),
    "mcdonough-sun1995": ("McDonough, W. F. and Sun, S.-s. 1995, Chemical Geology 120, 223: the composition of the Earth (CI chondrite and pyrolite columns; read from two machine-readable reproductions, handover/packages/MP-affinity-data)", "10.1016/0009-2541(94)00140-4", None, "unverified"),
    "mcdonough2003": ("McDonough, W. F. 2003, Compositional model for the Earth's core, Treatise on Geochemistry vol. 2, 547, Table 4 (read from the GERM reservoir database's copy)", "10.1016/B0-08-043751-6/02015-6", "https://earthref.org/GERMRD/34/", "unverified"),
    "rudnick-gao2003": ("Rudnick, R. L. and Gao, S. 2003, Composition of the continental crust, Treatise on Geochemistry vol. 3, 1, Table 11, bulk crust (read from the pyrolite package's copy, checked against the 2014 edition)", "10.1016/B0-08-043751-6/03016-4", None, "unverified"),
    "nssdca-earth": ("NASA NSSDCA Earth fact sheet (volumetric mean radius 6371.000 km)", None, "https://nssdc.gsfc.nasa.gov/planetary/factsheet/earthfact.html", "verified"),
    "nssdca-moon": ("NASA NSSDCA Moon fact sheet (mass 0.07346e24 kg, mean radius 1737.4 km, semi-major axis 0.3844e6 km)", None, "https://nssdc.gsfc.nasa.gov/planetary/factsheet/moonfact.html", "verified"),
    "nssdca-jupiter": ("NASA NSSDCA Jupiter fact sheet (volumetric mean radius 69911 km)", None, "https://nssdc.gsfc.nasa.gov/planetary/factsheet/jupiterfact.html", "verified"),
    "cgpm1901": ("3rd CGPM 1901, Resolution 2: standard acceleration of gravity 980.665 cm/s2", None, "https://www.bipm.org/en/committees/cg/cgpm/3-1901/resolution-2", "textbook"),
    "cgpm1954": ("10th CGPM 1954, Resolution 4: standard atmosphere 101 325 Pa", None, "https://www.bipm.org/en/committees/cg/cgpm/10-1954/resolution-4", "textbook"),
    "petrenko1999": ("Petrenko, V. F. and Whitworth, R. W. 1999, Physics of Ice, Oxford University Press (ice Ih density and conductivity k = 651/T)", "10.1093/acprof:oso/9780198518945.001.0001", None, "verified"),
    "lineweaver2010": ("Lineweaver, C. H. and Norman, M. 2010, Proc. 9th Australian Space Science Conference: the potato radius, rock and ice yield strengths (calibrated stand-in; docs/fudges.md)", None, "https://arxiv.org/abs/1004.1091", "unverified"),
    "atom-isotropic": ("A spherically symmetric atom has no anisotropic polarisability, so its King correction factor is exactly 1 (Bodhaine et al. 1999 discussion of depolarisation)", "10.1175/1520-0426(1999)016<1854:ORODC>2.0.CO;2", None, "textbook"),
    "mastin2009": ("Mastin, L. G. et al. 2009, J. Volcanol. Geotherm. Res. 186, 10: eruption column heights, magma at 2500 kg/m3 dense-rock equivalent", "10.1016/j.jvolgeores.2009.01.008", None, "verified"),
    "moritz2000": ("Moritz, H. 2000, J. Geodesy 74, 128: Geodetic Reference System 1980 (Earth's angular velocity 7 292 115e-11 rad/s)", "10.1007/s001900050278", None, "verified"),
    "chandrasekhar1931": ("Chandrasekhar, S. 1931, ApJ 74, 81: the maximum mass of ideal white dwarfs (1.44 solar masses is the usual figure for a carbon-oxygen core, mu_e = 2, with Coulomb corrections)", "10.1086/143324", None, "textbook"),
    "eakins2010": ("Eakins, B. W. and Sharman, G. F. 2010, Volumes of the World's Oceans from ETOPO1, NOAA National Geophysical Data Center (1.335e9 km3, mean depth 3688 m)", None, "https://www.ngdc.noaa.gov/mgg/global/etopo1_ocean_volumes.html", "unverified"),
    "dupont-kevlar2017": ("DuPont 2017, Kevlar aramid fiber technical guide, Table II-1: typical properties of Kevlar 29 and 49 yarns (Kevlar 49 straight-test yarn 3,000 MPa, resin-impregnated strand 3,600 MPa, density 1.44 g/cm3; typical values, not specifications)", None, "https://www.r-g.de/wiki/images/e/ec/Td_en_Kevlar_guide.pdf", "verified"),
    "fiberline-zylon": ("Fiber-Line, Zylon PBO datasheet (fibre made by Toyobo): high-modulus grade breaking tenacity 42.0 g/d, specific gravity 1.56; typical values, not a specification. Toyobo's own technical information gives 5.8 GPa", None, "https://automotive.polyone.com/sites/default/files/2021-06/fl.us-.datasheet-zylonr-pbo.pdf", "verified"),
    "yu2000": ("Yu, M.-F., Lourie, O., Dyer, M. J., Moloni, K., Kelly, T. F. and Ruoff, R. S. 2000, Science 287, 637: strength and breaking mechanism of multiwalled carbon nanotubes under tensile load (outer shells broke at 11 to 63 GPa)", "10.1126/science.287.5453.637", None, "unverified"),
    "turcotte2014": ("Turcotte, D. L. and Schubert, G. 2014, Geodynamics, 3rd edition, Cambridge University Press (typical crust 2700, mantle 3300 and sediment 2300 kg/m3 of its worked problems)", "10.1017/CBO9780511843877", None, "unverified"),
    "wallace-hobbs2006": ("Wallace, J. M. and Hobbs, P. V. 2006, Atmospheric Science: An Introductory Survey, 2nd edition, Academic Press (dry air c_p = 1004 J/kg/K)", "10.1016/C2009-0-00034-8", None, "unverified"),
    "nssdca-venus": ("NASA NSSDCA Venus fact sheet (atmosphere 96.5% CO2, 3.5% N2 by volume)", None, "https://nssdc.gsfc.nasa.gov/planetary/factsheet/venusfact.html", "unverified"),
    "nssdca-mars": ("NASA NSSDCA Mars fact sheet (atmosphere 95.1% CO2, 2.59% N2, 1.94% Ar, 0.16% O2, 0.06% CO by volume)", None, "https://nssdc.gsfc.nasa.gov/planetary/factsheet/marsfact.html", "unverified"),
    "niemann2005": ("Niemann, H. B. et al. 2005, Nature 438, 779: Titan's atmosphere from the Huygens probe (5.65% methane by volume near the surface)", "10.1038/nature04122", None, "unverified"),
    "crc-handbook": ("Haynes, W. M. (ed.) 2016, CRC Handbook of Chemistry and Physics, 97th edition, CRC Press (quartz 2650 kg/m3)", "10.1201/9781315380476", None, "unverified"),
    "ussa1976": ("U.S. Standard Atmosphere 1976, NOAA/NASA/USAF (sea-level dry-air composition by volume)", None, "https://ntrs.nasa.gov/citations/19770009539", "verified"),
    "kobayashi2020": ("Kobayashi, C., Karakas, A. I. and Lugaro, M. 2020, ApJ 900, 179: the origin of elements from carbon to uranium (half of C, N and F from AGB stars; half of the iron-peak elements from Type Ia supernovae; the r-process shortfall)", "10.3847/1538-4357/abae65", None, "verified"),
    "johnson-origins": ("Johnson, J. A., Origin of the Elements in the Solar System: periodic table colour-coded by origin (Ohio State University, CC BY-SA 4.0; the six-source legend)", None, "https://www.astronomy.ohio-state.edu/johnson.3064/nucleo/index.html", "verified"),
    "bisterzo2014": ("Bisterzo, S. et al. 2014, ApJ 787, 10: Galactic chemical evolution and solar s-process abundances, dependence on the 13C-pocket structure (s-process percentages per isotope, Kr to Bi)", "10.1088/0004-637X/787/1/10", None, "verified"),
    "kasen2017": ("Kasen, D. et al. 2017, Nature 551, 80: origin of the heavy elements in binary neutron-star mergers from a gravitational-wave event (the GW170817 kilonova)", "10.1038/nature24453", None, "verified"),
    "abbott2017": ("Abbott, B. P. et al. (LIGO and Virgo) 2017, Phys. Rev. Lett. 119, 161101: GW170817, observation of gravitational waves from a binary neutron star inspiral", "10.1103/PhysRevLett.119.161101", None, "verified"),
    "fields2020": ("Fields, B. D., Olive, K. A., Yeh, T.-H. and Young, C. 2020, JCAP 03, 010: Big-Bang nucleosynthesis after Planck (primordial helium and the lithium problem)", None, "https://arxiv.org/abs/1912.01132", "verified"),
    "planck2018": ("Planck Collaboration 2020, A&A 641, A6: Planck 2018 results VI, cosmological parameters. Table 2, last column (TT,TE,EE+lowE+lensing+BAO, 68% limits): Omega_b h2 0.02242 +- 0.00014, Omega_c h2 0.11933 +- 0.00091, n_s 0.9665 +- 0.0038, H0 67.66 +- 0.42, Omega_Lambda 0.6889 +- 0.0056, Omega_m 0.3111 +- 0.0056, Omega_m h2 0.14240 +- 0.00087, sigma8 0.8102 +- 0.0060, age 13.787 +- 0.020 Gyr. Section 2.1: one massive neutrino of 0.06 eV", "10.1051/0004-6361/201833910", "https://www.aanda.org/articles/aa/full_html/2020/09/aa33910-18/T2.html", "verified"),
    "fixsen2009": ("Fixsen, D. J. 2009, ApJ 707, 916: the temperature of the cosmic microwave background (2.72548 +- 0.00057 K)", "10.1088/0004-637X/707/2/916", None, "verified"),
    "mangano2005": ("Mangano, G. et al. 2005, Nucl. Phys. B 729, 221: relic neutrino decoupling including flavour oscillations (N_eff 3.046; Omega_nu h2 = sum of neutrino masses / 93.14 eV)", "10.1016/j.nuclphysb.2005.09.041", None, "verified"),
    "chevallier2001": ("Chevallier, M. and Polarski, D. 2001, Int. J. Mod. Phys. D 10, 213: accelerating universes with scaling dark matter (w = w0 + wa (1 - a))", "10.1142/S0218271801000822", None, "verified"),
    "linder2003": ("Linder, E. V. 2003, Phys. Rev. Lett. 90, 091301: exploring the expansion history of the universe (the w0, wa form)", "10.1103/PhysRevLett.90.091301", None, "verified"),
    "schneider2012": ("Schneider, A. et al. 2012, MNRAS 424, 684: non-linear evolution of cosmological structures in warm dark matter models (the half-mode mass)", "10.1111/j.1365-2966.2012.21252.x", None, "verified"),
    "irsic2017": ("Iršič, V. et al. 2017, Phys. Rev. D 96, 023522: new constraints on the free-streaming of warm dark matter from Lyman-alpha forest data", "10.1103/PhysRevD.96.023522", None, "verified"),
    "nadler2021": ("Nadler, E. O. et al. 2021, Phys. Rev. Lett. 126, 091101: constraints on dark matter properties from observations of Milky Way satellite galaxies", "10.1103/PhysRevLett.126.091101", None, "verified"),
    "rocha2013": ("Rocha, M. et al. 2013, MNRAS 430, 81: cosmological simulations with self-interacting dark matter I, constant-density cores and substructure", "10.1093/mnras/sts514", None, "verified"),
    "harvey2015": ("Harvey, D. et al. 2015, Science 347, 1462: the nongravitational interactions of dark matter in colliding galaxy clusters", "10.1126/science.1261381", None, "verified"),
    "kaplinghat2016": ("Kaplinghat, M., Tulin, S. and Yu, H.-B. 2016, Phys. Rev. Lett. 116, 041302: dark matter halos as particle colliders", "10.1103/PhysRevLett.116.041302", None, "verified"),
    "hui2017": ("Hui, L., Ostriker, J. P., Tremaine, S. and Witten, E. 2017, Phys. Rev. D 95, 043541: ultralight scalars as cosmological dark matter", "10.1103/PhysRevD.95.043541", None, "verified"),
    "clowe2006": ("Clowe, D. et al. 2006, ApJ 648, L109: a direct empirical proof of the existence of dark matter (the Bullet Cluster)", "10.1086/508162", None, "verified"),
    "boran2018": ("Boran, S. et al. 2018, Phys. Rev. D 97, 041501: GW170817 falsifies dark matter emulators", "10.1103/PhysRevD.97.041501", None, "verified"),
    "des2025": ("DES Collaboration (Abbott, T. M. C. et al.) 2025, Phys. Rev. D 112, 083535: Dark Energy Survey year 3, cosmological constraints from cluster abundances, weak lensing and galaxy clustering", "10.1103/3dzh-d8f5", None, "verified"),
    "vijaykumar2021": ("Vijaykumar, A., Kapadia, S. J. and Ajith, P. 2021, Phys. Rev. Lett. 126, 141104: constraints on the time variation of the gravitational constant using gravitational-wave observations of binary neutron stars", "10.1103/PhysRevLett.126.141104", None, "verified"),
    "sakstein2026": ("Sakstein, J. and Desmond, H. 2026, arXiv:2610.04048: stars would remember a recent gravitational transition, testing G-step resolutions of the Hubble tension with red dwarfs (excluded at 3.1 and 4.0 sigma)", None, "https://arxiv.org/abs/2610.04048", "verified"),
    "mcgaugh2016": ("McGaugh, S. S., Lelli, F. and Schombert, J. M. 2016, Phys. Rev. Lett. 117, 201101: radial acceleration relation in rotationally supported galaxies", "10.1103/PhysRevLett.117.201101", None, "verified"),
    "randall2008": ("Randall, S. W. et al. 2008, ApJ 679, 1173: constraints on the self-interaction cross section of dark matter from the Bullet Cluster", "10.1086/587859", None, "verified"),
    "desi2024": ("DESI Collaboration (Adame, A. G. et al.) 2025, JCAP 02, 021: DESI 2024 VI, cosmological constraints from baryon acoustic oscillations", "10.1088/1475-7516/2025/02/021", None, "verified"),
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
    ("P_rot_sun", "solar sidereal rotation period (Carrington)", val(25.38 * 86400.0, "s", "nssdca-sun", "unverified", note="25.38 days; the surface turns faster at the equator and slower near the poles")),
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
    ("omega_earth", "Earth's angular velocity (sidereal rotation)", val(7.292115e-5, "rad s-1", "moritz2000")),
    ("V_ocean_earth", "volume of Earth's ocean", val(1.335e18, "m3", "eakins2010", "unverified")),
    ("depth_ocean_earth", "mean depth of Earth's ocean", val(3688.0, "m", "eakins2010", "unverified")),
    ("M_ocean_earth", "mass of Earth's ocean", val(1.4e21, "kg", "eakins2010", "unverified", note="1.335e18 m3 at about 1030 kg/m3, to two figures")),
    ("M_ch", "Chandrasekhar mass", val(1.44, "M_sun", "chandrasekhar1931", "textbook")),
    ("M_moon", "Moon mass", val(7.346e22, "kg", "nssdca-moon")),
    ("R_moon", "Moon mean radius", val(1.7374e6, "m", "nssdca-moon")),
    ("a_moon", "Moon semi-major axis", val(3.844e8, "m", "nssdca-moon")),
]


# ---------------------------------------------------------------- the Cosmos group (DM1)
# Galaxy histories design section 3.2. The measured parameters of the universe, moved here from
# packs/base-cosmology (its tuned model numbers stay there). Planck 2018 VI table 2, last column.
# The kernel reads these through the constants handle (sse_core::constants::Cosmos), whose
# compiled values must equal these bit for bit, so an unedited sheet changes no hash.
H_PLANCK = 0.6766
OMEGA_B = 0.04897      # Omega_b h2 0.02242 / h2 = 0.048975, to four figures as G1 has always used it
OMEGA_NU = 0.0014072   # one 0.06 eV neutrino: Omega_nu h2 = 0.06 / 93.14 = 0.000644, / h2
OMEGA_M = 0.3111       # Planck's own derived Omega_m (it includes the 0.06 eV neutrino)
OMEGA_C = 0.2607228    # Omega_m - Omega_b - Omega_nu; Planck's Omega_c h2 / h2 is 0.26067, inside its 0.0020
T_CMB = 2.7255
N_EFF = 3.046
THEORY_DM = ("The amount is measured through gravity alone (the cosmic background, lensing, clusters); no dark matter "
             "particle has been detected. A detection, or a theory of gravity that needed none, would change the picture.")
THEORY_DE = ("The amount is measured through the expansion (supernovae, the cosmic background, baryon acoustic "
             "oscillations); what it is is unknown. A measured change in w, which DESI's 2024 and 2025 results lean "
             "towards, would change the picture.")


def omega_r():
    return 2.473e-5 * (T_CMB / 2.7255) ** 4 * (1 + 0.2271073177 * N_EFF) / H_PLANCK ** 2


def cos(id_, name, value, derived=None, mark=None, note=None):
    row = {"id": id_, "name": name, "group": "cosmos", "value": value}
    if derived:
        row["derived_from"] = derived
    if mark:
        row["mark"] = "theoretical"
        row["mark_note"] = mark
    if note:
        row["note"] = note
    return row


def cosmos():
    om_l = 1 - OMEGA_M - omega_r()
    pc = AU * 648000 / PI
    h_si = 67.66 * 1000 / (1e6 * pc)
    rho_l = om_l * 3 * h_si ** 2 * C ** 2 / (8 * PI * G)
    return [
        cos("H0", "Hubble constant", val(67.66, "km s-1 Mpc-1", "planck2018", uncertainty=0.42),
            note="In km/s per megaparsec, as astronomers quote it; H0_si is the same rate per second."),
        cos("H0_si", "Hubble constant per second", val(h_si, "s-1", "planck2018"), "H0 1000 / (1e6 pc)"),
        cos("Omega_b", "ordinary matter today, Omega_b", val(OMEGA_B, "1", "planck2018", uncertainty=0.0003),
            note="Planck's Omega_b h2 0.02242 +- 0.00014 divided by h2, to four figures."),
        cos("Omega_c", "dark matter today, Omega_c", val(OMEGA_C, "1", "planck2018", uncertainty=0.002), mark=THEORY_DM,
            note="Cold dark matter: Planck's Omega_m less ordinary matter and neutrinos, so the three sum to its 0.3111. "
                 "Planck's own Omega_c h2 0.11933 +- 0.00091 gives 0.26067, inside the uncertainty."),
        cos("Omega_nu", "massive neutrinos today, Omega_nu", val(OMEGA_NU, "1", "mangano2005", "unverified", also=["planck2018"]),
            note="Planck's base model has one neutrino of 0.06 eV; Omega_nu h2 = sum of masses / 93.14 eV."),
        cos("Omega_m", "all matter today, Omega_m", val(OMEGA_M, "1", "planck2018", uncertainty=0.0056),
            "Omega_b + Omega_c + Omega_nu"),
        cos("dm_per_ordinary", "dark matter per ordinary matter", val(OMEGA_C / OMEGA_B, "1", "planck2018"),
            "Omega_c / Omega_b"),
        cos("f_b", "cosmic baryon share", val(OMEGA_B / OMEGA_M, "1", "planck2018"), "Omega_b / Omega_m",
            note="The share of every halo's matter that is ordinary: what each galaxy's gas supply reads."),
        cos("sigma8", "fluctuation size on 8 Mpc/h, sigma_8", val(0.8102, "1", "planck2018", uncertainty=0.006)),
        cos("n_s", "tilt of the primordial spectrum, n_s", val(0.9665, "1", "planck2018", uncertainty=0.0038)),
        cos("T_cmb", "cosmic background temperature today", val(T_CMB, "K", "fixsen2009", "unverified", uncertainty=0.00057)),
        cos("N_eff", "effective number of neutrino species", val(N_EFF, "1", "mangano2005", "unverified"),
            note="The Standard Model prediction; Planck finds 2.99 +- 0.17 with BAO."),
        cos("Omega_r", "radiation today, Omega_r", val(omega_r(), "1", "fixsen2009", also=["mangano2005"]),
            "2.473e-5 (T_cmb / 2.7255)^4 (1 + 0.2271073177 N_eff) / (H0 / 100)^2",
            note="Photons from the background temperature, plus 7/8 (4/11)^(4/3) per neutrino species."),
        cos("Omega_Lambda", "dark energy today, Omega_Lambda", val(om_l, "1", "planck2018"), "1 - Omega_m - Omega_r",
            mark=THEORY_DE, note="Worked out for a flat universe. Planck: 0.6889 +- 0.0056."),
        cos("w0", "dark energy pressure today, w0", val(-1.0, "1", "chevallier2001", also=["linder2003", "planck2018"]),
            mark=THEORY_DE, note="Pressure over density; -1 is a cosmological constant. w = w0 + wa (1 - a)."),
        cos("wa", "drift of dark energy pressure, wa", val(0.0, "1", "chevallier2001", also=["linder2003", "desi2024"]),
            mark=THEORY_DE, note="0 is a cosmological constant. Read from DM2 on; until then the expansion uses w = -1."),
        cos("rho_Lambda", "dark energy density", val(rho_l, "J m-3", "planck2018"),
            "Omega_Lambda 3 H0_si^2 c^2 / (8 pi G)"),
        cos("m_wdm", "dark matter warmth: thermal relic mass", val(0.0, "keV", "schneider2012", also=["irsic2017", "nadler2021"]),
            mark=THEORY_DM, note="0 is cold dark matter, with no cut-off. A mass in keV gives a smallest halo size "
                                 "(the half-mode mass); today's limits say heavier than about 5 to 6.5 keV. Read from DM5 on."),
        cos("sigma_dm", "dark matter self-interaction per mass", val(0.0, "m2 kg-1", "rocha2013", also=["harvey2015", "randall2008", "kaplinghat2016"]),
            mark=THEORY_DM, note="0 is collisionless. A cross-section gives haloes a core; clusters limit it to under "
                                 "about 0.5 cm2/g (5e-5 m2/kg). Read from DM5 on."),
        cos("m_fdm", "fuzzy (ultra-light) dark matter particle mass", val(0.0, "eV", "hui2017"), mark=THEORY_DM,
            note="0 is off. Listed, not wired: it would give both a cut-off and a solid core."),
    ]


# Page groups of the Constants page: names and notes for people and Orac.
GROUPS = [
    {"id": "nature", "name": "Constants of nature", "about": "What the universe gives."},
    {"id": "convention", "name": "Definitions and conventions", "about": "Exact by agreement: c in SI, the astronomical unit, the Julian year, the IAU's nominal Sun."},
    {"id": "factor", "name": "Conversion factors", "about": "Pure arithmetic from the rows above."},
    {"id": "cosmos", "name": "Cosmos",
     "about": "The measured parameters of the universe. Dark matter and dark energy are marked theoretical: their amounts are measured through gravity, their nature is unknown. Change one and the cosmos chain follows: expansion, growth, the web, halo counts, every galaxy's gas supply.",
     "notes": [
         {"text": "SSE4 takes the particle route: dark matter is an unseen particle that adds mass. It does not offer gravity that changes at low acceleration (MOND and its relatives). The web, the count of haloes and the growth of galaxies are built on dark matter, and no agreed theory of altered gravity replaces all three: colliding clusters show the mass sitting apart from the gas, and the neutron-star merger of 2017 ruled out the whole class of theories where gravity waves and light travel at different speeds.", "src": "clowe2006", "also": ["boran2018"]},
         {"text": "The standard picture keeps passing new tests: counts of 16,000 galaxy clusters agree with the cosmic background's parameters. Gravity's strength has not changed over time where it can be tested: a neutron-star merger bounds the change in G, and red dwarfs, which would remember a jump in G, rule out the recent 4% and 11% steps proposed to ease the Hubble tension. Those test G changing with time, a different idea from MOND.", "src": "des2025", "also": ["vijaykumar2021", "sakstein2026"]},
         {"text": "Kept as a check: the tight link between ordinary matter and rotation speed in galaxies, which haloes must reproduce.", "src": "mcgaugh2016"},
         {"text": "Energy is conserved for a bound object, not for the expanding universe as a whole, so the sheet's ledgers are mass per co-moving region and energy per bound system.", "src": "planck2018"},
     ]},
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
    ("h", "h", "hour", "time", "working", ("v", 3600.0, "si2019")),
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


for _k, _v in water_phases.SOURCES.items():
    assert _k not in SOURCES, _k
    SOURCES[_k] = _v


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
    # Masses of the light isotopes the fuels use: AME2020 atomic masses, in u.
    iso("h1", "h", 1, mass=val(1.00782503224, "u", "ame2020")),
    iso("h2", "h", 2, mass=val(2.01410177812, "u", "ame2020")),
    iso("h3", "h", 3, 12.32, daughter="he3", mass=val(3.01604928132, "u", "ame2020")),
    iso("he3", "he", 3, mass=val(3.01602932197, "u", "ame2020")),
    iso("he4", "he", 4, mass=val(4.00260325413, "u", "ame2020")), iso("li6", "li", 6), iso("li7", "li", 7),
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


FS = "fray2009"


def fs_fit(coeffs, lo, hi, note=None):
    """A Fray and Schmitt 2009 sublimation fit, ln P[bar] = A0 + sum A_i / T^i."""
    d = {"fit": "fray-schmitt-2009", "coeffs": coeffs, "valid_k": [lo, hi], "src": FS, "also": ["lisse2021"]}
    if note:
        d["note"] = note
    return d


KING_ONE = ("fudge-register", "authored")  # King factor 1: molecular depolarisation not yet tabulated
T = "thauer1977"
NIST = "nist-webbook"

SUBSTANCES = [
    substance("h2o", {"h": 2, "o": 1}, "water", plain="water", story="water",
              phases={
                  "triple_point_k": val(273.16, "K", NIST),
                  "triple_point_pa": val(611.657, "Pa", NIST),
                  "latent_vaporisation": val(2.468e6, "J kg-1", NIST, "verified", at_k=288.0, note="from the slope of the NIST Antoine fit, 273-303 K row"),
                  "latent_fusion": val(3.3355e5, "J kg-1", "crc-handbook", "unverified", at_k=273.15, at_pa=101325.0, note="ice Ih to liquid, 6.009 kJ/mol"),
                  "density_liquid": val(1000.0, "kg m-3", NIST, "verified", at_k=277.0),
                  "density_solid": val(917.0, "kg m-3", "petrenko1999", "verified", at_k=273.15, note="ice Ih"),
                  "vapour_pressure": {"fit": "feistel-wagner-2007", "coeffs": [273.16, 611.657, 20.9969665107897, 3.72437478271362, -13.9205483215524, 29.6988765013566, -40.1972392635944, 29.7880481050215, -9.13050963547721], "valid_k": [50.0, 273.16], "src": FS, "also": ["lisse2021"], "note": "ln(P/Pt) = 1.5 ln(T/Tt) + (1 - Tt/T) sum e_i (T/Tt)^i; coeffs [Tt, Pt Pa, e0..e6]"},
                  # MP18: every phase of water, its boundaries, triple points and the phase_at table (scripts/water_phases.py)
                  **water_phases.build(),
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
                  "vapour_pressure": fs_fit([18.61, -4154.0, 1.041e5], 194.7, 216.58, "CO2-2"),
                  "vapour_pressure_low": fs_fit([14.76, -2571.0, -7.781e4, 4.325e6, -1.207e8, 1.350e9], 40.0, 194.7, "CO2-1"),
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
                  "density_solid": val(494.0, "kg m-3", "fudge-register", "unverified", note="methane ice as the climate's sand grain (Telfer et al. 2018 use it); not read from a measurement"),
                  "vapour_pressure": fs_fit([10.51, -1110.0, -4341.0, 1.035e5, -7.910e5], 20.0, 90.67, "CH4-1"),
              },
              optical={**gas_optics(4.44e-4, [1.0, 0.0, 0.0], "kaye-laby", "unverified", *KING_ONE), "condensed_colour_rgb": val([0.9, 0.9, 0.95], "1", "fudge-register", "authored")},
              chemical={"states": {"gas": gibbs(-50.75, -74.81, T), "dissolved": gibbs(-34.39, -89.04, T)}}),
    substance("nh3", {"n": 1, "h": 3}, "ammonia",
              phases={
                  "triple_point_k": val(195.4, "K", NIST, "unverified", note="the climate pack holds 194.95 K, the melting point"),
                  "triple_point_pa": val(6060.0, "Pa", NIST),
                  "latent_sublimation": val(1832000.0, "J kg-1", NIST, note="31.2 kJ/mol"),
                  "density_solid": val(820.0, "kg m-3", NIST, "unverified"),
                  "vapour_pressure": fs_fit([15.96, -3537.0, -3.310e4, 1.742e6, -2.995e7], 15.0, 195.4, "NH3-1"),
              },
              optical={**gas_optics(3.76e-4, [1.0, 0.0, 0.0], "pdg-gas", "verified", *KING_ONE), "condensed_colour_rgb": val([0.98, 0.96, 0.9], "1", "fudge-register", "authored")}),
    substance("n2", {"n": 2}, "nitrogen",
              phases={
                  "triple_point_k": val(63.14, "K", NIST),
                  "triple_point_pa": val(12520.0, "Pa", NIST),
                  "latent_vaporisation": val(200000.0, "J kg-1", NIST, at_k=77.0, note="5.6 kJ/mol"),
                  "density_liquid": val(800.0, "kg m-3", NIST, "unverified", at_k=77.0, note="808 kg/m3 at 77 K"),
                  "density_solid": val(1027.0, "kg m-3", "fudge-register", "unverified", note="alpha nitrogen ice near 20 K, from memory; not read"),
                  "vapour_pressure": fs_fit([8.514, -458.4, -1.987e4, 4.800e5, -4.524e6], 35.61, 63.14, "N2-2, beta nitrogen"),
                  "vapour_pressure_low": fs_fit([12.40, -807.4, -3926.0, 6.297e4, -4.633e5, 1.325e6], 20.0, 35.61, "N2-1, alpha nitrogen"),
              },
              optical={**gas_optics(2.98e-4, [1.034, 3.17e-4, 0.0], "kaye-laby", "unverified"), "condensed_colour_rgb": val([0.97, 0.97, 0.97], "1", "fudge-register", "authored")},
              chemical={"states": {"gas": gibbs(0.0, 0.0, "convention")}}),
    substance("o2", {"o": 2}, "oxygen",
              phases={
                  "density_liquid": val(1141.2, "kg m-3", "schmidt-wagner1985", "unverified", at_k=90.188, also=["coolprop2014"],
                                        note="saturated liquid at its normal boiling point; CoolProp's fit gives 1141.20"),
              },
              optical=gas_optics(2.71e-4, [1.096, 1.385e-3, 1.448e-4], "kaye-laby", "unverified"),
              chemical={"states": {"gas": gibbs(0.0, 0.0, "convention"), "dissolved": gibbs(16.32, -11.7, T)}}),
    substance("ar", {"ar": 1}, "argon",
              phases={
                  "triple_point_k": val(83.81, "K", NIST, "unverified"),
                  "triple_point_pa": val(68900.0, "Pa", NIST, "unverified"),
                  "density_solid": val(1616.0, "kg m-3", "fudge-register", "unverified", note="from memory; not read"),
                  "vapour_pressure": fs_fit([10.69, -893.2, -3567.0, 6.574e4, -4.280e5], 21.0, 83.81, "Ar-1"),
              },
              optical={**gas_optics(2.81e-4, [1.0, 0.0, 0.0], "kaye-laby", "unverified", king_src="atom-isotropic"), "condensed_colour_rgb": val([0.97, 0.97, 0.97], "1", "fudge-register", "authored")}),
    substance("h2", {"h": 2}, "hydrogen",
              phases={
                  # Normal hydrogen, saturated liquid at 1 atm; CoolProp's fit to the same equation gives 70.87.
                  "density_liquid": val(70.85, "kg m-3", "leachman2009", "unverified", at_k=20.369, also=["coolprop2014"],
                                        note="normal hydrogen at its normal boiling point"),
              },
              optical=gas_optics(1.32e-4, [1.0, 0.0, 0.0], "kaye-laby", "unverified", *KING_ONE),
              chemical={"states": {"gas": gibbs(0.0, 0.0, "convention"), "dissolved": gibbs(17.72, -4.2, T)}}),
    substance("he", {"he": 1}, "helium", optical=gas_optics(3.5e-5, [1.0, 0.0, 0.0], "kaye-laby", "unverified", king_src="atom-isotropic")),
    # Fusion fuels (hull tanks, ship ranges). Formulas name the isotope, so the molar mass is the isotope's own.
    substance("d2", {"h2": 2}, "deuterium", "heavy hydrogen",
              phases={"density_liquid": val(163.5, "kg m-3", "coolprop2014", "computed", at_k=23.31, also=["richardson2014"],
                                            note="normal deuterium at its normal boiling point, from CoolProp's saturated-liquid fit to Richardson et al. 2014 (fit good to 0.17%)")}),
    substance("he3", {"he3": 1}, "helium-3",
              phases={"density_liquid": val(59.0, "kg m-3", "fudge-register", "unverified", at_k=3.19,
                                            note="about 59 kg/m3 at its 3.19 K boiling point; from memory, not read: check against a helium-3 equation of state")}),
    substance("ne", {"ne": 1}, "neon", optical=gas_optics(6.71e-5, [1.0, 0.0, 0.0], "pdg-gas", "verified", king_src="atom-isotropic")),
    substance("kr", {"kr": 1}, "krypton"),
    substance("xe", {"xe": 1}, "xenon"),
    substance("so2", {"s": 1, "o": 2}, "sulphur dioxide", optical=gas_optics(6.86e-4, [1.0, 0.0, 0.0], "kaye-laby", "unverified", *KING_ONE)),
    substance("co", {"c": 1, "o": 1}, "carbon monoxide",
              phases={
                  "triple_point_k": val(68.13, "K", NIST, "unverified"),
                  "triple_point_pa": val(15370.0, "Pa", NIST, "unverified"),
                  "density_solid": val(929.0, "kg m-3", "fudge-register", "unverified", note="from memory; not read"),
                  "vapour_pressure": fs_fit([10.25, -748.2, -5843.0, 3.939e4], 61.55, 68.13, "CO-2, beta carbon monoxide"),
                  "vapour_pressure_low": fs_fit([10.43, -721.3, -1.074e4, 2.341e5, -2.392e6, 9.478e6], 14.0, 61.55, "CO-1, alpha carbon monoxide"),
              },
              optical={**gas_optics(3.38e-4, [1.0, 0.0, 0.0], "kaye-laby", "unverified", *KING_ONE), "condensed_colour_rgb": val([0.97, 0.97, 0.97], "1", "fudge-register", "authored")},
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
                 "king_factor": {"fit": "a + b lambda^-2 + c lambda^-4, lambda in um", "coeffs": [1.0469, 0.0, 0.0], "src": "bodhaine1999"}},
     "density": val(1.225, "kg m-3", "ussa1976", at_k=288.15, at_pa=101325.0, note="sea level in the standard atmosphere"),
     "thermal": {"heat_capacity": val(1004.0, "J kg-1 K-1", "wallace-hobbs2006", "unverified", note="at constant pressure, near 300 K")}},
    {"id": "giant-air", "names": {"physics": "hydrogen-helium envelope", "plain": "giant-planet air", "story": "giant air"},
     "recipe": {"h2": 0.86, "he": 0.14}, "by": "volume", "src": "fudge-register", "check": "authored",
     "note": "the weather pack's giant air at 0.0023 kg/mol; the reader's worked-out molar mass replaces it",
     "thermal": {"heat_capacity": val(12000.0, "J kg-1 K-1", NIST, "unverified", note="by mass from hydrogen near 300 K and helium (5193); colder hydrogen holds less")}},
    # Ship fuels by mass, one per base-nav fuel id ("fuel-" + id). Antimatter has no row: no stored density.
    {"id": "fuel-hydrogen", "names": {"physics": "liquid hydrogen", "plain": "hydrogen fuel", "story": "hydrogen"},
     "recipe": {"h2": 1.0}, "by": "mass", "src": "fudge-register", "check": "authored"},
    {"id": "fuel-hydrogen-oxygen", "names": {"physics": "liquid hydrogen and liquid oxygen", "plain": "hydrogen and oxygen", "story": "hydrolox"},
     "recipe": {"h2": 1.0 / 7.0, "o2": 6.0 / 7.0}, "by": "mass", "src": "fudge-register", "check": "authored",
     "note": "oxidiser to fuel 6:1 by mass, near what hydrogen-oxygen engines burn (the stoichiometric ratio is about 7.9)"},
    {"id": "fuel-deuterium", "names": {"physics": "liquid deuterium", "plain": "deuterium fuel", "story": "deuterium"},
     "recipe": {"d2": 1.0}, "by": "mass", "src": "fudge-register", "check": "authored"},
    {"id": "fuel-helium-3", "names": {"physics": "liquid helium-3", "plain": "helium-3 fuel", "story": "helium-3"},
     "recipe": {"he3": 1.0}, "by": "mass", "src": "fudge-register", "check": "authored"},
    {"id": "surface-rock", "names": {"physics": "generic silicate rock", "plain": "rock", "story": "rock"},
     "recipe": {}, "by": "mass", "density": val(3500.0, "kg m-3", "fudge-register", "authored"),
     "yield_strength": val(1.54e8, "Pa", "lineweaver2010", "unverified", note="calibrated so rock rounds at about 300 km")},
    {"id": "surface-ice", "names": {"physics": "water ice with some rock", "plain": "ice", "story": "ice"},
     "recipe": {"h2o": 1.0}, "by": "mass", "density": val(930.0, "kg m-3", "fudge-register", "authored", note="pure ice Ih is 917 kg/m3 (water's density_solid)"),
     "yield_strength": val(5.6e6, "Pa", "lineweaver2010", "unverified", note="calibrated so ice rounds at about 200 km"),
     "melting_point": val(273.15, "K", NIST, note="at low pressure")},
    {"id": "crust-rock", "names": {"physics": "continental crust rock", "plain": "crust rock", "story": "rock"},
     "recipe": {}, "by": "mass", "density": val(2700.0, "kg m-3", "turcotte2014", "unverified")},
    {"id": "mantle-rock", "names": {"physics": "upper mantle rock", "plain": "mantle rock", "story": "deep rock"},
     "recipe": {}, "by": "mass", "density": val(3300.0, "kg m-3", "turcotte2014", "unverified")},
    {"id": "sediment", "names": {"physics": "compacted sediment", "plain": "sediment", "story": "sediment"},
     "recipe": {}, "by": "mass", "density": val(2300.0, "kg m-3", "turcotte2014", "unverified")},
    {"id": "venus-air", "names": {"physics": "carbon dioxide atmosphere (Venus)", "plain": "thick carbon dioxide air", "story": "furnace air"},
     "recipe": {"co2": 0.965, "n2": 0.035}, "by": "volume", "src": "nssdca-venus", "check": "unverified",
     "thermal": {"heat_capacity": val(1100.0, "J kg-1 K-1", NIST, "unverified", note="carbon dioxide near 700 K")}},
    {"id": "mars-air", "names": {"physics": "thin carbon dioxide atmosphere (Mars)", "plain": "thin carbon dioxide air", "story": "thin air"},
     "recipe": {"co2": 0.9525, "n2": 0.0259, "ar": 0.0194, "o2": 0.0016, "co": 0.0006}, "by": "volume", "src": "nssdca-mars", "check": "unverified",
     "note": "carbon dioxide taken as 95.25% (the fact sheet's 95.1% plus the traces it leaves out) so the recipe sums to one",
     "thermal": {"heat_capacity": val(750.0, "J kg-1 K-1", NIST, "unverified", note="carbon dioxide near 210 K")}},
    {"id": "titan-air", "names": {"physics": "nitrogen-methane atmosphere (Titan)", "plain": "nitrogen and methane air", "story": "cold haze air"},
     "recipe": {"n2": 0.9435, "ch4": 0.0565}, "by": "volume", "src": "niemann2005", "check": "unverified",
     "thermal": {"heat_capacity": val(1040.0, "J kg-1 K-1", NIST, "unverified", note="nitrogen near 94 K")}},
    {"id": "quartz-sand", "names": {"physics": "quartz sand", "plain": "sand", "story": "sand"},
     "recipe": {}, "by": "mass", "density": val(2650.0, "kg m-3", "crc-handbook", "unverified", note="grain density, not bulk")},
    # Materials that carry load: strength for weight is tensile strength over density (FT1, limit 7).
    {"id": "steel-wire", "names": {"physics": "high-strength steel wire", "plain": "steel cable", "story": "steel"},
     "recipe": {}, "by": "mass", "density": val(7850.0, "kg m-3", "fudge-register", "unverified", note="textbook carbon steel; from memory, not read"),
     "tensile_strength": val(2.0e9, "Pa", "fudge-register", "unverified", note="drawn high-carbon wire is about 1.8 to 2.2 GPa; from memory, not read")},
    {"id": "aramid", "names": {"physics": "aramid fibre (Kevlar 49 yarn)", "plain": "aramid fibre", "story": "aramid"},
     "recipe": {}, "by": "mass", "density": val(1440.0, "kg m-3", "dupont-kevlar2017"),
     "tensile_strength": val(3.0e9, "Pa", "dupont-kevlar2017", note="yarn, straight test; a resin-impregnated strand reaches 3.6 GPa")},
    {"id": "pbo-fibre", "names": {"physics": "PBO fibre (Zylon HM)", "plain": "PBO fibre", "story": "Zylon"},
     "recipe": {}, "by": "mass", "density": val(1560.0, "kg m-3", "fiberline-zylon", note="specific gravity 1.56"),
     "tensile_strength": val(5.8e9, "Pa", "fiberline-zylon", note="42.0 g/d times specific gravity 1.56 is 5.8 GPa, as Toyobo quotes")},
    {"id": "carbon-nanotube", "names": {"physics": "multiwalled carbon nanotube (best measured shell)", "plain": "carbon nanotube", "story": "nanotube"},
     "recipe": {}, "by": "mass", "density": val(1340.0, "kg m-3", "fudge-register", "unverified", note="1.3 to 1.4 g/cm3 is usually quoted; not read from a paper"),
     "tensile_strength": val(6.3e10, "Pa", "yu2000", "unverified", note="the strongest of the measured outer shells; no cable this strong has been made")},
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
AFFINITY = element_affinity.load()


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
            element_affinity.apply(row, AFFINITY, val)
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
    consts += cosmos()
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
                "groups": GROUPS,
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
        iso_mass = {i["id"] for i in ref["isotopes"] if "mass" in i}
        for el in s["formula"]:
            if el not in el_ids and el not in iso_mass:
                errors.append(f"substance {s['id']}: unknown element or isotope {el}")
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
    errors += validate_phases(ref)
    if len(ref["elements"]) != 118:
        errors.append("elements: expected 118 rows")
    return errors


def validate_phases(ref):
    """MP17's load checks, run here too so a bad table never reaches the pack."""
    errors = []
    for sub in ref["substances"]:
        ph = sub.get("phases", {})
        if "forms" not in ph:
            continue
        sid = sub["id"]
        form_ids = [f["id"] for f in ph["forms"]]
        if len(form_ids) != len(set(form_ids)):
            errors.append(f"{sid}: duplicate phase ids")
        stable = {f["id"] for f in ph["forms"] if f["stability"] == "stable"}
        linked = {x["id"] for x in ref["sources"] if x.get("doi") or x.get("url")}
        grades = ("measured", "computed", "extrapolated", "convention")
        for f in ph["forms"]:
            if f["evidence"]["grade"] not in grades:
                errors.append(f"{sid}: {f['id']} has grade {f['evidence']['grade']}")
            if f["evidence"]["grade"] == "computed" and f["evidence"]["src"] not in linked:
                errors.append(f"{sid}: {f['id']} is computed but its source has no doi or url")
        for b in ph["boundaries"]:
            if b["evidence"]["grade"] not in grades:
                errors.append(f"{sid}: boundary {b['between']} has grade {b['evidence']['grade']}")
            if b["evidence"]["grade"] == "computed" and b["src"] not in linked:
                errors.append(f"{sid}: boundary {b['between']} is computed but its source has no doi or url")
        pairs = set()
        for b in ph["boundaries"]:
            for x in b["between"]:
                if x not in stable:
                    errors.append(f"{sid}: boundary names {x}, not a stable phase")
            pairs.add(tuple(sorted(b["between"])))
        for m in ph["meets"]:
            for x in m["phases"]:
                if x not in stable:
                    errors.append(f"{sid}: triple point names {x}")
            for a, b in ((0, 1), (0, 2), (1, 2)):
                pr = tuple(sorted((m["phases"][a], m["phases"][b])))
                if pr not in pairs:
                    errors.append(f"{sid}: triple point {m['phases']} has no boundary {pr}")
        fam = [m for m in ph["meets"] if set(m["phases"]) == {f"{sid}.ice_ih", f"{sid}.liquid", f"{sid}.vapour"}]
        if fam and (fam[0]["t_k"] != ph["triple_point_k"]["v"] or fam[0]["p_pa"] != ph["triple_point_pa"]["v"]):
            errors.append(f"{sid}: the solid-liquid-vapour triple point disagrees with phases.triple_point")
        t = ph["table"]
        n_p, n_t = t["log10_p_pa"]["n"], t["log10_t_k"]["n"]
        grid = water_phases.decode(t)
        if len(grid) != n_p or any(len(r) != n_t for r in grid):
            errors.append(f"{sid}: table does not decode to {n_p} x {n_t}")
            continue
        ids = t["phases"]
        used = set()
        for i in range(n_p):
            row = grid[i]
            for j, a in enumerate(row):
                used.add(a)
                for b in ([row[j + 1]] if j + 1 < n_t else []) + ([grid[i + 1][j]] if i + 1 < n_p else []):
                    if a != b and tuple(sorted((a, b))) not in pairs and not any(
                            tuple(sorted((a, c))) in pairs and tuple(sorted((c, b))) in pairs for c in stable):
                        # a phase thinner than one cell may sit between two that do not touch
                        errors.append(f"{sid}: table puts {a} next to {b} with no boundary between them (cell {i},{j})")
        for x in ids:
            if x not in stable:
                errors.append(f"{sid}: table phase {x} is not a stable form")
        for x in stable - used:
            errors.append(f"{sid}: stable phase {x} never appears in the table")
    return sorted(set(errors))[:40]


def main():
    pack = build()
    errors = validate(pack)
    if errors:
        print("\n".join(errors))
        sys.exit(1)
    text = json.dumps(pack, indent=1, ensure_ascii=False, allow_nan=False) + "\n"
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
