"""Lines that ions give off, on the master data sheet (stray-numbers sweep SN2, 26 Nov 2026).

Ion rows (substances with a charge) carry `optical.emission`: one entry per line, beside the sodium and potassium
lines that `optical.absorption` already holds. The light pack (`packs/base-light`) names a line by its id and the
row it lives on (`sheet`), and its reader (sse-light) fills the wavelength and the atomic data from here.

  - forbidden lines of glowing gas (O4 part 1, PR 588): [O III] 495.9 and 500.7 nm, [S II] 671.6 and 673.1 nm, with
    what the nebula block reads: the photon energies that make and strip the ion, the effective collision strength
    at 10,000 K, the statistical weights, the share of the upper level's decays or its Einstein A, and for S II its
    local balance with S III. Moved unchanged from the light pack, where they were typed (values and their notes are
    those of PR 588).
  - comet ion tails (SB4 part 2, PR 613): CO+ and H2O+ glow where the renderer draws an ion tail. Their wavelengths
    were typed as fallbacks in packages/renderer/src/comet.ts (426 and 615 nm, "about", Brandt and Chapman 2004).
    They are the brightest band heads of CO+'s comet-tail system (A-X) near 426 nm and H2O+'s A-X bands near
    615 nm as the textbook quotes them, kept at the textbook's three figures and marked unverified.

Wavelengths are in standard air, as astronomers quote optical lines; `lambda_nm` reads to metres exactly as the
decimal written (sse-light parses "<nm>e-9").
"""

NIST_ION = ("NIST Atomic Spectra Database ionisation energies (Kramida, A., Ralchenko, Yu., Reader, J. and NIST ASD Team)", None, "https://physics.nist.gov/PhysRefData/ASD/ionEnergy.html", "unverified")

SOURCES = {
    "storey2014": ("Storey, P. J., Sochi, T. and Badnell, N. R. 2014: collision strengths for nebular [O III] optical and infrared lines (table 6, effective collision strengths at 10,000 K; table 5, level energies)", None, "https://arxiv.org/abs/1311.6517", "unverified"),
    "storey-nlevel": ("Storey, N-level atoms notes (University of Maryland): the [O III] 500.7 and 495.9 nm lines share the 1D2 level's decays about three to one", None, "https://pages.astro.umd.edu/~jpha/N-level.pdf", "unverified"),
    "sii-aa2025": ("Astronomy and Astrophysics 694, A133 (2025), table B.5: [S II] effective collision strengths at 10,000 K and Einstein A, a fit to Tayal, S. S. and Zatsarinny, O. 2010, ApJS 188, 32 (doi:10.1088/0067-0049/188/1/32)", "10.1051/0004-6361/202451865", None, "unverified"),
    "verner1996": ("Verner, D. A., Ferland, G. J., Korista, K. T. and Yakovlev, D. G. 1996, ApJ 465, 487: analytic fits for partial photoionisation cross-sections (table 1)", "10.1086/177435", None, "unverified"),
    "badnell2015": ("Badnell, N. R. et al. 2015: dielectronic and radiative recombination rate coefficients (table 6)", None, "https://arxiv.org/abs/1503.05905", "unverified"),
    "nist-asd-ion": NIST_ION,
    "brandt-chapman2004": ("Brandt, J. C. and Chapman, R. D. 2004, Introduction to Comets, 2nd edition, Cambridge University Press: ion tails glow in CO+ near 426 nm and H2O+ near 615 nm", "10.1017/CBO9781139165327", None, "unverified"),
}

O3_NOTE = ("Collision strengths 0.24917, 0.75062 and 1.26198 from 3P0, 3P1 and 3P2 to 1D2, summed over the 3P term "
           "(weight 9); 1D2 lies 20273.27 cm-1 above 3P0 (table 5), 20065.5 above the term's weighted centre. The 500.7 "
           "and 495.9 nm lines share the 1D2 level's decays about three to one; the exact ratio from Storey and Zeippen "
           "2000 (MNRAS 312, 813) is a stand-in until checked (fudge register F64). De-excitation not worked: 1D2's "
           "critical density is far above nebular densities.")
S2_BALANCE = {
    "fit": {"e0_ev": 8.787, "sigma0_m2": 3.136e-20, "ya": 3.442, "p": 12.81, "yw": 0.7354, "y0": 2.782, "y1": 0.1788, "max_ev": 184.6},
    "recombination_m3_s": 4.48e-18,
    "src": "verner1996", "also": ["badnell2015"], "check": "unverified",
    "note": ("Once- and twice-ionised sulphur balance where they are: S+ stripped by starlight with Verner et al.'s "
             "cross-section fit (table 1, row S II), S2+ rejoined at 4.48e-12 cm3 s-1 at 10,000 K, the sum of the "
             "dielectronic 2.94e-12 and radiative 1.54e-12 of Badnell et al. 2015, table 6."),
}


def _o3(id_, nm, share):
    return {"id": id_, "kind": "forbidden", "lambda_nm": nm, "medium": "air", "src": "storey2014", "also": ["storey-nlevel", "nist-asd-ion"], "check": "unverified",
            "nebular": {"makes_ev": 35.12112, "strips_ev": 54.93554, "upsilon": 2.26177, "lower_weight": 9, "upper_weight": 5, "upper_cm": 20065.5, "share": share},
            "note": O3_NOTE}


def _s2(id_, nm, upsilon, upper_weight, rate_s, level):
    return {"id": id_, "kind": "forbidden", "lambda_nm": nm, "medium": "air", "src": "sii-aa2025", "also": ["nist-asd-ion"], "check": "unverified",
            "nebular": {"makes_ev": 10.36001, "strips_ev": 23.33788, "upsilon": upsilon, "lower_weight": 4, "upper_weight": upper_weight, "rate_s": rate_s, "balance": S2_BALANCE},
            "note": f"[S II] {nm:.1f} nm, {level} to 4S3/2."}


def _comet(id_, nm, what):
    return {"id": id_, "kind": "band", "lambda_nm": nm, "medium": "air", "src": "brandt-chapman2004", "check": "unverified", "note": what}


ROWS = [
    ("o+2", {"o": 1}, 2, "doubly ionised oxygen (O III)", "oxygen stripped of two electrons", [
        _o3("oiii-4959", 495.891, 0.25), _o3("oiii", 500.684, 0.75)]),
    ("s+", {"s": 1}, 1, "singly ionised sulphur (S II)", "sulphur stripped of one electron", [
        _s2("sii", 671.644, 3.83, 6, 0.0002019, "2D5/2"), _s2("sii-6731", 673.082, 2.56, 4, 0.000684, "2D3/2")]),
    ("co+", {"c": 1, "o": 1}, 1, "carbon monoxide ion", "carbon monoxide stripped of one electron", [
        _comet("co+", 426.0, "CO+ comet-tail bands (A-X), the blue of a CO-driven ion tail; about 426 nm, three figures as the textbook gives them")]),
    ("h2o+", {"h": 2, "o": 1}, 1, "water ion", "water stripped of one electron", [
        _comet("h2o+", 615.0, "H2O+ bands (A-X), the orange of a water-driven ion tail; about 615 nm, three figures as the textbook gives them")]),
]


def apply(substances):
    ids = {s["id"] for s in substances}
    seen = set()
    for sid, formula, charge, physics, plain, lines in ROWS:
        assert sid not in ids, sid
        for ln in lines:
            assert ln["id"] not in seen, ln["id"]
            seen.add(ln["id"])
        substances.append({"id": sid, "formula": formula, "charge": charge,
                           "names": {"physics": physics, "plain": plain, "story": plain},
                           "optical": {"emission": lines}})
