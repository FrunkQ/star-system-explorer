"""Sea liquids (package MP32, asked by the climate thread for sea colour and
waves by liquid): optical absorption and refractive index of the liquid.

Part 1 holds liquid water, the one sea liquid with readable primary tables:
  absorption_liquid   absorption coefficient of pure liquid water, 380 to 700 nm
                      (Pope and Fry 1997, integrating cavity) and 750 to 2000 nm
                      (Hale and Querry 1973), on substance h2o's optical block
  refractive_index_liquid  real index n at 0.5 to 2 micron (Segelstein 1981)

The values were read on 24 Nov 2026 from the Oregon Medical Laser Center's
copies of the authors' tables (omlc.org/spectra/water/data/), printed in 1/cm
and turned to 1/m here. Unverified until checked against the papers.

Methane, ethane, ammonia, nitrogen and helium liquids, and the density,
viscosity and surface tension of every sea liquid, wait for readable tables
(handover/packages/MP32-sea-liquids.md).
"""

SOURCES = {
    "pope-fry1997": ("Pope, R. M. and Fry, E. S. 1997, Applied Optics 36, 8710: absorption spectrum (380-700 nm) of pure water. II. Integrating cavity measurements; table on a 2.5 nm grid in 1/cm (OMLC copy pope97.txt)", "10.1364/AO.36.008710", "https://omlc.org/spectra/water/data/pope97.txt", "unverified"),
    "hale-querry1973": ("Hale, G. M. and Querry, M. R. 1973, Applied Optics 12, 555: optical constants of water in the 200 nm to 200 micron wavelength region; absorption in 1/cm (OMLC copy hale73.txt)", "10.1364/AO.12.000555", "https://omlc.org/spectra/water/data/hale73.txt", "unverified"),
    "segelstein1981": ("Segelstein, D. J. 1981, The complex refractive index of water, M.S. thesis, University of Missouri-Kansas City; real and imaginary index against wavelength (OMLC copy segelstein81_index.txt)", None, "https://omlc.org/spectra/water/data/segelstein81_index.txt", "unverified"),
}

PER_CM = 100.0  # 1/cm to 1/m
# Pope and Fry 1997: nm, absorption 1/cm as printed.
POPE = [
    (380, 0.0001137), (400, 0.0000663), (420, 0.0000454), (440, 0.0000635),
    (460, 0.0000979), (480, 0.000127), (500, 0.000204), (520, 0.000409),
    (540, 0.000474), (560, 0.000619), (580, 0.000896), (600, 0.002224),
    (620, 0.002755), (640, 0.003108), (660, 0.0041), (680, 0.00465), (700, 0.00624),
]
# Hale and Querry 1973: nm, absorption 1/cm as printed.
HALE = [
    (750, 0.026), (800, 0.020), (900, 0.068), (1000, 0.36), (1200, 1.04),
    (1440, 28.8), (1460, 28.4), (1600, 6.72), (2000, 69.12),
]
# Segelstein 1981: micron (row printed), real index n, imaginary index k.
SEGELSTEIN = [
    (0.5, 1.339430, 9.243e-10), (0.5998, 1.333100, 9.634e-09),
    (0.7499, 1.327201, 1.559e-07), (0.7998, 1.325874, 1.250e-07),
    (0.8995, 1.323648, 4.862e-07), (1.0, 1.321695, 3.000e-06),
    (1.199, 1.318008, 1.199e-05), (1.449, 1.313055, 3.637e-04),
    (1.6, 1.309642, 9.347e-05), (2.0, 1.296913, 1.101e-03),
]


def apply(substances, pi):
    """Add the liquid optics to the substance rows in place."""
    U = "unverified"
    h2o = {s["id"]: s for s in substances}["h2o"]
    lam = [float(n) for n, _ in POPE + HALE]
    assert lam == sorted(lam)
    alpha = [round(a * PER_CM, 6) for _, a in POPE + HALE]
    h2o["optical"]["absorption_liquid"] = {
        "kind": "table", "lambda_nm": lam,
        "alpha": {"v": alpha, "unit": "m-1", "src": "pope-fry1997", "also": ["hale-querry1973"], "check": U},
        "src": "pope-fry1997", "also": ["hale-querry1973"], "check": U,
        "src_from_nm": {"pope-fry1997": [380.0, 700.0], "hale-querry1973": [750.0, 2000.0]},
        "note": "pure liquid water near room temperature; light falls to 1/e over 1/alpha metres (about 22 m at 420 nm, 0.4 m at 750 nm), which is why deep water is blue",
    }
    # The index rows' k agrees with alpha = 4 pi k / lambda, which ties the two tables.
    for um, _, k in SEGELSTEIN:
        if abs(um - 1.0) < 1e-9:
            assert abs(4 * pi * k / (um * 1e-6) - 37.699) < 0.1
    h2o["optical"]["refractive_index_liquid"] = {
        "kind": "table", "lambda_nm": [round(um * 1000.0, 1) for um, _, _ in SEGELSTEIN],
        "n": [n for _, n, _ in SEGELSTEIN], "k": [k for _, _, k in SEGELSTEIN],
        "src": "segelstein1981", "check": U,
        "note": "liquid water near room temperature; k is the imaginary index (alpha = 4 pi k / lambda)",
    }
