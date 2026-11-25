"""Optical constants of condensates for the renderer's bands, clouds, hazes and
ground stains (package MP33, asked by the giant-bands and plumes threads).

Shape: optical.refractive_index = {kind table, lambda_nm, n, k, src}; the
absorption coefficient follows as alpha = 4 pi k / lambda.

  titan-tholin   organic haze made by a DC discharge through 90 % N2 and
                 10 % CH4 at 0.2 mbar, room temperature (Khare et al. 1984,
                 Table I, 0.26 to 2.42 micron)

  jsc-mars-1, mms-2, mgs-1  Mars dust simulants (palagonitic JSC Mars-1,
                 Enhanced Mojave Mars Simulant, Mars Global Simulant): k from
                 200 to 2000 nm retrieved from reflectance with n fixed at 1.5
                 (Martikainen et al. 2023, Table 3, shape GRS1); package MP34

  troilite       FeS at 300 K on two crystal axes (Kranhold et al. 2026)
  hematite       Fe2O3, 250 to 1000 nm (Sokolik and Toon 1999)
  basalt, limonite  260 to 2000 nm (Egan and Hilgeman 1979)
  magnetite      Fe3O4, 310 to 1230 nm (Schlegel et al. 1979)
  lunar-soil-*   reflectance factor (i 30, e 0) of Apollo soils: mature highland
                 62231 (Pieters 1999), mature, submature and immature mare
                 10084, 71501, 12030 (LSCC, Taylor et al. 2001)

The Table I values were read on 24 Nov 2026 from the text layer of the
HITRAN aerosol archive's copy of the paper, which drops the minus sign of
each exponent: k is printed as a mantissa and an exponent b with k = a x 10^-b
(the paper's own text gives k near 4e-4 at 1.5 micron and a few hundredths in
the visible, which fixes the sign). Unverified until checked against the
page image (page 133).
"""

SOURCES = {
    "khare1984": ("Khare, B. N., Sagan, C., Arakawa, E. T., Suits, F., Callcott, T. A. and Williams, M. W. 1984, Icarus 60, 127: optical constants of organic tholins produced in a simulated Titanian atmosphere, from soft X-ray to microwave frequencies; Table I, n and k = a 10^b (n to +/-0.05, k to +/-30 %)", "10.1016/0019-1035(84)90142-8", "https://hitran.org/data/Aerosols/Aerosols%20data%20from%20previous%20HITRAN/Aerosols-2016/papers/khare_tholins_1984.pdf", "unverified"),
}

SOURCES["martikainen2023"] = ("Martikainen, J., Munoz, O., Jardiel, T., Gomez Martin, J. C., Peiteado, M., Willame, Y., Penttila, A., Muinonen, K., Wurm, G. and Becker, T. 2023, ApJS 268, 47: optical constants of Martian dust analogs at UV-visible-near-infrared wavelengths; Table 3, k for shape GRS1 with n = 1.5 (k is insensitive to n between 1.4 and 1.6, Table 4)", "10.3847/1538-4365/acf0be", "https://scattering.iaa.es/sites/default/files/media/downloads/2023_Martikainen.pdf", "unverified")
SOURCES["dabrowska2015"] = ("Dabrowska, D. D., Munoz, O., Moreno, F., Ramos, J. L., Martinez-Frias, J. and Wurm, G. 2015, Icarus 250, 83: scattering matrices of martian dust analogs at 488 nm and 647 nm; Table 1 lists the Martian dust index of Wolff et al. 2009 (1.495 + 0.012 to 0.008i at 488 nm, 1.5 + 0.002 to 0.001i at 647 nm)", "10.1016/j.icarus.2014.11.024", "https://old-scattering.iaa.csic.es/site_media/download/DabrowskaetalIcarus2015.pdf", "unverified")

SOURCES["kranhold2026"] = ("Kranhold et al. 2026, Astronomy and Astrophysics: optical constants of pyrrhotites and troilite, UV to far infrared (title not printed where read); troilite FeS at 300 K, E parallel to x and to z, from the Jena Database of Optical Constants for Cosmic Dust files troilite_FeS_300K_ex.txt and _ez.txt (wl_um, n, k)", "10.1051/0004-6361/202659441", "https://www2.astro.uni-jena.de/Laboratory/OCDB/sulfides_2026.html", "unverified")

# Troilite FeS at 300 K (Kranhold et al. 2026, Jena OCDB files): micron, n_ex, k_ex, n_ez, k_ez, as printed.
TROILITE = [
    (0.25, 1.92, 1.41, 1.87, 1.50), (0.30, 2.04, 1.36, 1.98, 1.45), (0.35, 2.08, 1.37, 2.00, 1.49),
    (0.40, 2.09, 1.45, 2.02, 1.62), (0.45, 2.10, 1.56, 2.07, 1.79), (0.50, 2.13, 1.70, 2.17, 1.97),
    (0.55, 2.19, 1.85, 2.31, 2.13), (0.60, 2.27, 2.00, 2.47, 2.28), (0.65, 2.37, 2.13, 2.64, 2.39),
    (0.70, 2.47, 2.26, 2.82, 2.47), (0.774, 2.64, 2.43, 3.08, 2.54),
]

SOURCES["sokolik-toon1999"] = ("Sokolik, I. N. and Toon, O. B. 1999, Journal of Geophysical Research 104(D8), 9423-9444: incorporation of mineralogical composition into models of the radiative properties of mineral aerosol from UV to IR wavelengths; hematite n and k as compiled there, read from the University of Waterloo Natural Phenomena Simulation Group's files riHm.txt and ecHm.txt (interpolated to 1 nm, 212 to 1001 nm) on 24 Nov 2026; crystal orientation not stated", None, "https://www.npsg.uwaterloo.ca/data/sand.php", "unverified")

# Hematite Fe2O3 (Sokolik and Toon 1999 via NPSG files): nm, n, k, as printed.
HEMATITE = [
    (250, 1.853, 1.311), (300, 2.234, 1.213), (350, 2.354, 1.078), (400, 2.686, 1.245), (450, 3.063, 0.9663),
    (500, 3.147, 0.6541), (550, 3.203, 0.4903), (600, 3.12, 0.1565), (650, 2.962, 0.07494), (700, 2.868, 0.04628),
    (800, 2.756, 0.03242), (900, 2.716, 0.03529), (1000, 2.69, 0.02602),
]

SOURCES["egan-hilgeman1979"] = ("Egan, W. G. and Hilgeman, T. W. 1979, Optical Properties of Inhomogeneous Materials: Applications to Geology, Astronomy, Chemistry, and Engineering, Academic Press; basalt and limonite n and k as read from the University of Waterloo Natural Phenomena Simulation Group's files riBas/ecBas and riLim/ecLim (185 to 2500 and 185 to 2600 nm) on 24 Nov 2026; no sample description on the page", None, "https://www.npsg.uwaterloo.ca/data/sand.php", "unverified")
SOURCES["schlegel1979"] = ("Schlegel, A., Alvarado, S. F. and Wachter, P. 1979, Journal of Physics C: Solid State Physics 12, 1157-1164: optical properties of magnetite (Fe3O4); n and k as read from the University of Waterloo Natural Phenomena Simulation Group's files riMag.txt and ecMag.txt (interpolated to 1 nm, 310 to 1230 nm) on 24 Nov 2026", None, "https://www.npsg.uwaterloo.ca/data/sand.php", "unverified")

# Egan and Hilgeman 1979 via NPSG files, rows as printed (the files' own wavelengths): nm, n, k.
BASALT = [
    (260, 1.644, 0.002692), (300, 1.644, 0.002692), (355, 1.634, 0.002399), (400, 1.621, 0.002239), (466, 1.611, 0.001995),
    (500, 1.613, 0.00195), (566, 1.599, 0.001698), (600, 1.609, 0.001738), (666, 1.597, 0.001698), (700, 1.597, 0.001738),
    (817, 1.595, 0.00195), (907, 1.595, 0.002455), (1000, 1.593, 0.00302), (1500, 1.59, 0.002455), (2000, 1.589, 0.003311),
]
LIMONITE = [
    (260, 2.15, 0.007079), (300, 2.282, 0.00955), (355, 2.33, 0.0123), (400, 2.528, 0.02239), (466, 2.544, 0.02399),
    (500, 2.513, 0.01698), (566, 2.4, 0.004786), (600, 2.35, 0.004169), (666, 2.228, 0.003388), (700, 2.2, 0.00309),
    (817, 2.146, 0.003162), (907, 2.122, 0.003802), (1000, 2.108, 0.003802), (1500, 2.045, 0.00263), (2000, 2.03, 0.002455),
]
# Schlegel et al. 1979 via NPSG files: nm, n, k.
MAGNETITE = [
    (310, 2.103, 0.6236), (350, 2.179, 0.7265), (400, 2.233, 0.874), (450, 2.186, 1.015), (500, 2.169, 1.051),
    (550, 2.164, 1.1), (600, 2.152, 1.153), (650, 2.164, 1.191), (700, 2.125, 1.243), (800, 2.05, 1.27),
    (900, 1.938, 1.212), (1000, 1.939, 1.109), (1230, 2.082, 0.9478),
]
MINERALS = [
    ("basalt", BASALT, "egan-hilgeman1979", ("basalt", "basalt", "dark volcanic rock"),
     "the commonest volcanic rock on the rocky planets; grey-black and only faintly coloured",
     "rock sample as measured by Egan and Hilgeman; no sample or locality named where read"),
    ("limonite", LIMONITE, "egan-hilgeman1979", ("limonite (hydrous ferric oxides, mostly goethite)", "yellow-brown rust", "ochre"),
     "the yellow-brown rust of weathered iron-bearing rock; absorbs violet and blue",
     "limonite is the general term for hydrous ferric oxides, mostly goethite (the NPSG page, after Roberts et al. 1990)"),
    ("magnetite", MAGNETITE, "schlegel1979", ("magnetite (Fe3O4)", "black iron oxide", "lodestone"),
     "the black, magnetic iron oxide of black sands and lodestones; absorbs strongly at every colour",
     "single-crystal values; the files read run from 310 to 1230 nm"),
]

SOURCES["pieters1999-62231"] = ("Pieters, C. M. (about 1999; venue not printed on the copy read): the Moon as a spectral calibration standard enabled by lunar samples, the Clementine example; Table 2, reflectance of mature Apollo 16 soil 62231 at the Clementine UVVIS wavelengths, i = 30, e = 0, as used in the USGS Clementine mosaics. Read from the PDS Geosciences copy on 25 Nov 2026", None, "https://pds-geosciences.wustl.edu/Lunar/mk88-l-120cvf-3-rdr-120color-v1/mkls_0001/document/pieter99.pdf", "unverified")
SOURCES["taylor2001-lscc"] = ("Taylor, L. A., Pieters, C. M., Morris, R. V., Keller, L. P. and McKay, D. S. 2001, Journal of Geophysical Research 106(E11), 27985: lunar mare soils, space weathering and the major effects of surface-correlated nanophase Fe. Lunar Soil Characterization Consortium reflectance-factor spectra (i = 30, e = 0, 300 to 2600 nm), bulk <45 micron column, read from RELAB's files nnnnnKdata.txt on 25 Nov 2026", None, "https://sites.brown.edu/relab/lscc/", "unverified")
SOURCES["wu2024-ce5"] = ("Wu, Y. Z., Chai, Y. M., Lu, Y., Chen, X. J., Wang, W. W. and Jin, Q. 2024, Astronomy and Astrophysics 682, A112: visible and near-infrared spectral results of Chang'E-5 surficial and subsurface soils; Table 1 lists the Is/FeO maturity of the LSCC Apollo soils (<45 micron, from Taylor et al. 2001) and the classes immature < 30, submature 30 to 60, mature > 60 (Morris 1976, 1978)", "10.1051/0004-6361/202347577", None, "unverified")

# Lunar soils, reflectance factor at i = 30, e = 0: (id, physics name, plain, sample, maturity, Is/FeO or None, src, rows nm -> r).
LUNAR_SOILS = [
    ("lunar-soil-mature-highland", "mature lunar highland soil (Apollo 16, 62231)", "Moon dust (highlands)", "62231", "mature", None, "pieters1999-62231",
     [(414.9, 0.1077), (753.3, 0.1776), (898.8, 0.1893), (951.5, 0.1941), (1000.4, 0.2004)]),
    ("lunar-soil-mature-mare", "mature lunar mare soil (Apollo 11, 10084)", "Moon dust (maria)", "10084", "mature", 88.0, "taylor2001-lscc",
     [(400, 0.043507), (450, 0.048341), (500, 0.052659), (550, 0.057191), (600, 0.061273), (650, 0.065615), (700, 0.069243), (750, 0.072632), (900, 0.079021), (1000, 0.081303)]),
    ("lunar-soil-submature-mare", "submature lunar mare soil (Apollo 17, 71501)", "Moon dust (young maria)", "71501", "submature", 44.0, "taylor2001-lscc",
     [(400, 0.044713), (450, 0.050048), (500, 0.054586), (550, 0.059588), (600, 0.063925), (650, 0.068261), (700, 0.072944), (750, 0.076743), (900, 0.083770), (1000, 0.086709)]),
    ("lunar-soil-immature-mare", "immature lunar mare soil (Apollo 12, 12030)", "fresh Moon dust", "12030", "immature", 20.0, "taylor2001-lscc",
     [(400, 0.10514), (450, 0.11845), (500, 0.12714), (550, 0.13629), (600, 0.14455), (650, 0.15286), (700, 0.16113), (750, 0.16827), (900, 0.15721), (1000, 0.16071)]),
]

# Martikainen et al. 2023 Table 3 (GRS1, n = 1.5): nm, k for JSC Mars-1, MMS-2, MGS-1, as printed.
MARTIKAINEN = [
    (250, 0.001308, 0.000704, 0.000493), (300, 0.001296, 0.000751, 0.000431), (350, 0.001170, 0.000773, 0.000357),
    (400, 0.000926, 0.000849, 0.000311), (450, 0.000742, 0.000819, 0.000300), (500, 0.000666, 0.000788, 0.000300),
    (550, 0.000571, 0.000658, 0.000302), (600, 0.000498, 0.000292, 0.000300), (650, 0.000468, 0.000243, 0.000315),
    (700, 0.000452, 0.000205, 0.000323), (800, 0.000510, 0.000202, 0.000394), (900, 0.000620, 0.000236, 0.000571),
    (1000, 0.000721, 0.000184, 0.000596), (1500, 0.001036, 0.000227, 0.000685), (2000, 0.001192, 0.000287, 0.000880),
]
MARS_SIMULANTS = [
    ("jsc-mars-1", "JSC Mars-1 palagonitic tephra (Mars dust simulant)", "Mars dust simulant (JSC Mars-1)", "red Mars dust"),
    ("mms-2", "Enhanced Mojave Mars Simulant (MMS-2)", "Mars soil simulant (Mojave)", "Mars soil"),
    ("mgs-1", "Mars Global Simulant (MGS-1)", "Mars soil simulant (global)", "Mars soil"),
]

# Khare et al. 1984 Table I as read: (micron, k mantissa, k exponent b, n), k = a 10^-b.
KHARE = [
    (0.2638, 1.8, 1, 1.68), (0.2952, 1.5, 1, 1.66), (0.3263, 1.3, 1, 1.64), (0.3542, 1.1, 1, 1.63),
    (0.3874, 9.1, 2, 1.66), (0.4133, 7.6, 2, 1.69), (0.4428, 6.0, 2, 1.72), (0.5635, 2.3, 2, 1.70),
    (0.6888, 8.8, 3, 1.68), (0.8731, 2.4, 3, 1.66), (1.016, 1.0, 3, 1.65), (1.148, 6.4, 4, 1.65),
    (1.192, 5.2, 4, 1.65), (1.357, 4.2, 4, 1.64), (1.381, 4.1, 4, 1.64), (1.802, 4.0, 4, 1.64),
    (1.813, 4.2, 4, 1.64), (1.823, 4.4, 4, 1.64), (1.873, 5.2, 4, 1.63), (2.019, 7.2, 4, 1.63),
    (2.214, 8.9, 4, 1.63), (2.393, 8.0, 4, 1.62), (2.403, 8.5, 4, 1.62), (2.422, 1.2, 3, 1.62),
]


SOURCES["protopapa2015"] = ("Protopapa, S., Grundy, W. M., Tegler, S. C. and Bergonio, J. M. 2015, Icarus (arXiv:1503.00703): absorption coefficients of the methane-nitrogen binary ice system, implications for Pluto; section 3.1 takes the real index of methane ice n0 = 1.32 at 15800 cm-1 from Trotta 1996", None, "https://arxiv.org/abs/1503.00703", "unverified")
SOURCES["quirico-sshade-n2"] = ("Quirico, E., Schmitt, B. and Grundy, W. 1992, near-IR optical constants spectra of alpha and beta N2 ices at different temperatures, SSHADE/GhoSST dataset (linked to Grundy et al. 1993, Icarus 105, 254); its analysis note holds the real index constant at the visible value n = 1.23", "10.26302/SSHADE/EXPERIMENT_BS_20120925_001", "https://sshade.eu/data/SPECTRUM_BS_20120925_005/details", "unverified")


SOURCES["warren-brandt2008"] = ("Warren, S. G. and Brandt, R. E. 2008, Journal of Geophysical Research 113, D14220: optical constants of ice from the ultraviolet to the microwave, a revised compilation (ice Ih at -7 C, 43 nm to 2 m). Read from the printable table IOPtable_2008.pdf on 24 Nov 2026", "10.1029/2007JD009744", "https://atmos.uw.edu/ice_optical_constants/", "unverified")

# Ice Ih at 266 K (Warren and Brandt 2008 printable table): micron, mre, mim, as printed.
# Below 400 nm the table prints mim as "<2.0E-11", an upper limit; it is held at 2e-11 here.
WATER_ICE = [
    (0.25, 1.3509, 2.0e-11), (0.30, 1.3339, 2.0e-11), (0.35, 1.3249, 2.0e-11), (0.40, 1.3194, 2.365e-11),
    (0.45, 1.3157, 9.239e-11), (0.50, 1.3130, 5.889e-10), (0.55, 1.3110, 2.289e-9), (0.60, 1.3094, 5.730e-9),
    (0.65, 1.3080, 1.430e-8), (0.70, 1.3069, 2.900e-8), (0.80, 1.3049, 1.340e-7), (0.90, 1.3032, 4.200e-7),
    (1.00, 1.3015, 1.620e-6), (1.25, 1.2971, 1.290e-5), (1.504, 1.2916, 5.373e-4), (2.00, 1.2744, 1.640e-3),
]


def apply_ices(substances, val):
    """Visible real index of the Pluto and Triton ices, on the substance rows."""
    by_id = {s["id"]: s for s in substances}
    by_id["ch4"]["optical"]["n_solid"] = val(1.32, "1", "protopapa2015", "unverified", at_place="methane ice, 15800 cm-1 (633 nm)", note="real index of methane ice in the visible; its visible k is not on the sheet yet")
    by_id["n2"]["optical"]["n_solid"] = val(1.23, "1", "quirico-sshade-n2", "unverified", at_place="nitrogen ice, visible", note="real index of nitrogen ice in the visible, held constant through the near infrared in the SSHADE optical constants")
    lam = [round(r[0] * 1000.0, 1) for r in WATER_ICE]
    assert lam == sorted(lam)
    by_id["h2o"]["optical"]["refractive_index_solid"] = {
        "kind": "table", "lambda_nm": lam, "n": [r[1] for r in WATER_ICE], "k": [r[2] for r in WATER_ICE],
        "src": "warren-brandt2008", "check": "unverified", "at_k": 266.15,
        "note": "ordinary ice Ih at -7 C; k below 400 nm is an upper limit (printed as <2.0E-11); alpha = 4 pi k / lambda; ice is bluest where k is least",
    }


def apply(mixtures):
    """Return the new mixture rows."""
    U = "unverified"
    lam = [um * 1000.0 for um, _, _, _ in KHARE]
    assert lam == sorted(lam)
    k = [float("%.3g" % (a * 10.0 ** -b)) for _, a, b, _ in KHARE]
    # The visible absorbs far more than the near infrared: that is what makes the haze orange.
    assert k[6] > 50 * k[15]  # 443 nm against 1802 nm
    tholin = {
        "id": "titan-tholin",
        "names": {"physics": "Titan tholin (N2-CH4 discharge organics)", "plain": "Titan's orange haze stuff", "story": "tholin"},
        "recipe": {}, "by": "mass",
        "note": "a laboratory stand-in for Titan's organic haze and for the red-brown stains on icy worlds; real hazes differ with how they were made",
        "optical": {"refractive_index": {
            "kind": "table", "lambda_nm": [round(x, 1) for x in lam], "n": [n for _, _, _, n in KHARE], "k": k,
            "src": "khare1984", "check": U, "at_k": 300.0,
            "note": "thin films and pellets of tholin from 0.9 N2 / 0.1 CH4 at 0.2 mbar, room temperature; alpha = 4 pi k / lambda",
        }},
    }
    assert all(m["id"] != tholin["id"] for m in mixtures)
    out = [tholin]
    lam_m = [float(r[0]) for r in MARTIKAINEN]
    assert lam_m == sorted(lam_m)
    for col, (mid, physics, plain, story) in enumerate(MARS_SIMULANTS, start=1):
        assert all(m["id"] != mid for m in mixtures)
        out.append({
            "id": mid, "names": {"physics": physics, "plain": plain, "story": story},
            "recipe": {}, "by": "mass",
            "note": "a laboratory stand-in for Martian dust and soil; Mars' airborne dust absorbs more (Wolff et al. 2009: k about 0.01 at 488 nm and 0.001 to 0.002 at 647 nm, via Dabrowska et al. 2015)",
            "optical": {"refractive_index": {
                "kind": "table", "lambda_nm": lam_m, "n": [1.5] * len(lam_m), "k": [r[col] for r in MARTIKAINEN],
                "src": "martikainen2023", "also": ["dabrowska2015"], "check": U,
                "note": "k retrieved from the powder's reflectance with irregular grains (GRS1) and n fixed at 1.5; n is assumed, not measured; alpha = 4 pi k / lambda",
            }},
        })
    lam_t = [round(r[0] * 1000.0, 1) for r in TROILITE]
    troilite = {
        "id": "troilite", "names": {"physics": "troilite (FeS)", "plain": "iron sulphide", "story": "iron sulphide"},
        "recipe": {}, "by": "mass",
        "note": "the iron sulphide of meteorites and of the sulphide clouds of hot giants; dark and metallic in the visible",
        "optical": {
            "refractive_index": {
                "kind": "table", "lambda_nm": lam_t, "n": [r[1] for r in TROILITE], "k": [r[2] for r in TROILITE],
                "src": "kranhold2026", "check": U, "at_k": 300.0,
                "note": "light polarised along the crystal's x axis; the z axis is in refractive_index_ez; the files read stop at 774 nm",
            },
            "refractive_index_ez": {
                "kind": "table", "lambda_nm": lam_t, "n": [r[3] for r in TROILITE], "k": [r[4] for r in TROILITE],
                "src": "kranhold2026", "check": U, "at_k": 300.0,
                "note": "light polarised along the crystal's z axis",
            },
        },
    }
    assert all(m["id"] != "troilite" for m in mixtures)
    out.append(troilite)
    hem = [r[0] for r in HEMATITE]
    assert hem == sorted(hem)
    assert all(m["id"] != "hematite" for m in mixtures)
    out.append({
        "id": "hematite", "names": {"physics": "hematite (Fe2O3)", "plain": "rust-red iron oxide", "story": "red iron oxide"},
        "recipe": {}, "by": "mass",
        "note": "the ferric oxide that reddens Mars and desert dust; strong blue and green absorption, nearly clear past 600 nm",
        "optical": {"refractive_index": {
            "kind": "table", "lambda_nm": [float(x) for x in hem], "n": [r[1] for r in HEMATITE], "k": [r[2] for r in HEMATITE],
            "src": "sokolik-toon1999", "check": U,
            "note": "bulk crystal values as compiled by Sokolik and Toon 1999; orientation not stated; the files read stop at 1001 nm; alpha = 4 pi k / lambda",
        }},
    })
    for mid, physics, plain, sample, maturity, is_feo, src, rows in LUNAR_SOILS:
        lam = [float(r[0]) for r in rows]
        assert lam == sorted(lam) and all(0.0 < r[1] < 1.0 for r in rows)
        assert all(m["id"] != mid for m in mixtures)
        refl = {
            "kind": "table", "lambda_nm": lam, "r": [r[1] for r in rows],
            "quantity": "reflectance factor", "i_deg": 30.0, "e_deg": 0.0, "maturity": maturity,
            "src": src, "check": U,
            "note": "laboratory spectrum of Apollo soil %s, relative to a Lambertian surface; space weathering (nanophase iron) darkens and reddens soil as it matures; interpolate linearly, hold the ends" % sample,
        }
        if is_feo is not None:
            refl["maturity_index"] = {"v": is_feo, "unit": "1", "src": "wu2024-ce5", "check": U, "note": "Is/FeO of the <45 micron fraction (Taylor et al. 2001, via Wu et al. 2024 Table 1); immature < 30, submature 30 to 60, mature > 60"}
        out.append({
            "id": mid, "names": {"physics": physics, "plain": plain, "story": "regolith"},
            "recipe": {}, "by": "mass",
            "note": "the ground of an airless rocky world after long exposure to space: grey to the eye, a few to twenty per cent reflective, rising slowly towards the red",
            "optical": {"reflectance": refl},
        })
    for mid, rows, src, (physics, plain, story), note, tnote in MINERALS:
        lam = [float(r[0]) for r in rows]
        assert lam == sorted(lam) and len(set(lam)) == len(lam)
        assert all(m["id"] != mid for m in mixtures)
        out.append({
            "id": mid, "names": {"physics": physics, "plain": plain, "story": story},
            "recipe": {}, "by": "mass", "note": note,
            "optical": {"refractive_index": {
                "kind": "table", "lambda_nm": lam, "n": [r[1] for r in rows], "k": [r[2] for r in rows],
                "src": src, "check": U, "note": tnote + "; alpha = 4 pi k / lambda",
            }},
        })
    return out
