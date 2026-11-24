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


def apply_ices(substances, val):
    """Visible real index of the Pluto and Triton ices, on the substance rows."""
    by_id = {s["id"]: s for s in substances}
    by_id["ch4"]["optical"]["n_solid"] = val(1.32, "1", "protopapa2015", "unverified", at_place="methane ice, 15800 cm-1 (633 nm)", note="real index of methane ice in the visible; its visible k is not on the sheet yet")
    by_id["n2"]["optical"]["n_solid"] = val(1.23, "1", "quirico-sshade-n2", "unverified", at_place="nitrogen ice, visible", note="real index of nitrogen ice in the visible, held constant through the near infrared in the SSHADE optical constants")


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
    return out
