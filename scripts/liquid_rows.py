"""Sea liquids (package MP32, asked by the climate thread for sea colour and
waves by liquid): optical absorption and refractive index of the liquid.

Part 1: liquid water's optics, the one sea liquid with readable primary tables:
  absorption_liquid   absorption coefficient of pure liquid water, 380 to 700 nm
                      (Pope and Fry 1997, integrating cavity) and 750 to 2000 nm
                      (Hale and Querry 1973), on substance h2o's optical block
  refractive_index_liquid  real index n at 0.5 to 2 micron (Segelstein 1981)
  scattering_liquid   pure water's molecular scattering b at 450 and 550 nm, its
                      lambda^-4.3 law and backscatter share 1/2 (Morel and Prieur 1977)

The values were read on 24 Nov 2026 from the Oregon Medical Laser Center's
copies of the authors' tables (omlc.org/spectra/water/data/), printed in 1/cm
and turned to 1/m here. Unverified until checked against the papers.

Part 2: saturated_liquid on each sea liquid's phases block: a table of
temperature, vapour pressure, density, viscosity and surface tension from the
triple point to 0.95 of the critical temperature, read through CoolProp 8.1.0
(the open-source implementation of the reference correlations the NIST WebBook
uses) into handover/packages/MP32-data/sat-liquids.csv by make_sat_liquids.py
there. The builder reads only the CSV. Each column cites its correlation's
paper. Ethane and propane get substance rows of their own here.

Cold-liquid absorption (methane, ethane) still waits for a readable table
(handover/packages/MP32-sea-liquids.md).
"""
import csv
from pathlib import Path

SAT_CSV = Path(__file__).resolve().parent.parent / "handover/packages/MP32-data/sat-liquids.csv"

SOURCES = {
    "morel-prieur1977": ("Morel, A. and Prieur, L. 1977, Limnology and Oceanography 22, 709: analysis of variations in ocean color; Table 1 pure-water scattering b_w, the wavelength exponent -4.3 (p. 712, after Morel 1974), backscatter share 1/2 for molecular scattering (p. 711, eq. 3) and seawater about 30 % higher (p. 714). Read by the climate and weather thread from a scan on 24 Nov 2026", None, None, "unverified"),
    "pope-fry1997": ("Pope, R. M. and Fry, E. S. 1997, Applied Optics 36, 8710: absorption spectrum (380-700 nm) of pure water. II. Integrating cavity measurements; table on a 2.5 nm grid in 1/cm (OMLC copy pope97.txt)", "10.1364/AO.36.008710", "https://omlc.org/spectra/water/data/pope97.txt", "unverified"),
    "hale-querry1973": ("Hale, G. M. and Querry, M. R. 1973, Applied Optics 12, 555: optical constants of water in the 200 nm to 200 micron wavelength region; absorption in 1/cm (OMLC copy hale73.txt)", "10.1364/AO.12.000555", "https://omlc.org/spectra/water/data/hale73.txt", "unverified"),
    "segelstein1981": ("Segelstein, D. J. 1981, The complex refractive index of water, M.S. thesis, University of Missouri-Kansas City; real and imaginary index against wavelength (OMLC copy segelstein81_index.txt)", None, "https://omlc.org/spectra/water/data/segelstein81_index.txt", "unverified"),
}

V = "unverified"
CP_NOTE = "; read through CoolProp 8.1.0 (handover/packages/MP32-data/sat-liquids.csv)"
SOURCES.update({
    "coolprop": ("Bell, I. H., Wronski, J., Quoilin, S. and Lemort, V. 2014, Ind. Eng. Chem. Res. 53, 2498: pure and pseudo-pure fluid thermophysical property evaluation and the open-source thermophysical property library CoolProp (version 8.1.0 used, scratch only, to evaluate the reference correlations cited on each row)", "10.1021/ie4033999", "https://coolprop.org", V),
    "mulero2012": ("Mulero, A., Cachadina, I. and Parra, M. I. 2012, J. Phys. Chem. Ref. Data 41, 043105: recommended correlations for the surface tension of common fluids" + CP_NOTE, "10.1063/1.4768782", None, V),
    "wagner-pruss2002": ("Wagner, W. and Pruss, A. 2002, J. Phys. Chem. Ref. Data 31, 387: the IAPWS Formulation 1995 for the thermodynamic properties of ordinary water substance" + CP_NOTE, "10.1063/1.1461829", None, V),
    "huber2009-water": ("Huber, M. L. et al. 2009, J. Phys. Chem. Ref. Data 38, 101: new international formulation for the viscosity of H2O" + CP_NOTE, "10.1063/1.3088050", None, V),
    "setzmann-wagner1991-eos": ("Setzmann, U. and Wagner, W. 1991, J. Phys. Chem. Ref. Data 20, 1061: a new equation of state and tables of thermodynamic properties for methane from the melting line to 625 K at pressures up to 1000 MPa" + CP_NOTE, "10.1063/1.555898", None, V),
    "sotiriadou2025-methane": ("Sotiriadou, S. G., Antoniadis, K. D., Assael, M. J., Martinek, V. and Huber, M. L. 2025, Int. J. Thermophys. 47, 18: correlation for the viscosity of methane from the triple point to 625 K and pressures to 1000 MPa" + CP_NOTE, "10.1007/s10765-025-03690-7", None, V),
    "buecker-wagner2006": ("Buecker, D. and Wagner, W. 2006, J. Phys. Chem. Ref. Data 35, 205: a reference equation of state for the thermodynamic properties of ethane from the melting line to 675 K and pressures up to 900 MPa" + CP_NOTE, "10.1063/1.1859286", None, V),
    "friend1991-ethane": ("Friend, D. G., Ingham, H. and Ely, J. F. 1991, J. Phys. Chem. Ref. Data 20, 275: thermophysical properties of ethane" + CP_NOTE, "10.1063/1.555881", None, V),
    "gao2023-ammonia": ("Gao, K., Wu, J., Bell, I. H., Harvey, A. H. and Lemmon, E. W. 2023, J. Phys. Chem. Ref. Data 52, 013102: a reference equation of state with an associating term for the thermodynamic properties of ammonia" + CP_NOTE, "10.1063/5.0128269", None, V),
    "monogenidou2018-ammonia": ("Monogenidou, S. A., Assael, M. J. and Huber, M. L. 2018, J. Phys. Chem. Ref. Data 47, 023102: reference correlation for the viscosity of ammonia from the triple point to 725 K and up to 50 MPa" + CP_NOTE, "10.1063/1.5036724", None, V),
    "span2000-nitrogen": ("Span, R., Lemmon, E. W., Jacobsen, R. T., Wagner, W. and Yokozeki, A. 2000, J. Phys. Chem. Ref. Data 29, 1361: a reference equation of state for the thermodynamic properties of nitrogen from 63.151 to 1000 K and pressures to 2200 MPa" + CP_NOTE, "10.1063/1.1349047", None, V),
    "huber2024-nitrogen": ("Huber, M. L., Perkins, R. A. and Lemmon, E. W. 2024, Int. J. Thermophys. 45, 146: reference correlation for the viscosity of nitrogen from the triple point to 1000 K and pressures up to 2200 MPa" + CP_NOTE, "10.1007/s10765-024-03440-1", None, V),
    "ortizvega2023-helium": ("Ortiz-Vega, D. O., Hall, K. R., Holste, J. C., Harvey, A. H. and Lemmon, E. W. 2023, NIST IR 8474: an equation of state for the thermodynamic properties of helium" + CP_NOTE, "10.6028/NIST.IR.8474", None, V),
    "arp1998-helium": ("Arp, V. D., McCarty, R. D. and Friend, D. G. 1998, NIST Technical Note 1334 (revised): thermophysical properties of helium-4 from 0.8 to 1500 K with pressures to 2000 MPa" + CP_NOTE, None, None, V),
    "span-wagner1996-co2": ("Span, R. and Wagner, W. 1996, J. Phys. Chem. Ref. Data 25, 1509: a new equation of state for carbon dioxide from the triple point to 1100 K at pressures up to 800 MPa" + CP_NOTE, "10.1063/1.555991", None, V),
    "laesecke-muzny2017-co2": ("Laesecke, A. and Muzny, C. D. 2017, J. Phys. Chem. Ref. Data 46: reference correlation for the viscosity of carbon dioxide" + CP_NOTE, "10.1063/1.4977429", None, V),
    "tegeler1999-argon": ("Tegeler, Ch., Span, R. and Wagner, W. 1999, J. Phys. Chem. Ref. Data 28, 779: a new equation of state for argon from the melting line to 700 K at pressures up to 1000 MPa" + CP_NOTE, "10.1063/1.556037", None, V),
    "lemmon-jacobsen2004": ("Lemmon, E. W. and Jacobsen, R. T 2004, Int. J. Thermophys. 25, 21: viscosity and thermal conductivity equations for nitrogen, oxygen, argon and air" + CP_NOTE, "10.1023/B:IJOT.0000022327.04529.f3", None, V),
    "lemmon2009-propane": ("Lemmon, E. W., McLinden, M. O. and Wagner, W. 2009, J. Chem. Eng. Data 54, 3141: thermodynamic properties of propane III, a reference equation of state from the melting line to 650 K and pressures up to 1000 MPa" + CP_NOTE, "10.1021/je900217v", None, V),
    "vogel1998-propane": ("Vogel, E., Kuechenmeister, C., Bich, E. and Laesecke, A. 1998, J. Phys. Chem. Ref. Data 27, 947: reference correlation of the viscosity of propane" + CP_NOTE, "10.1063/1.556025", None, V),
})
# CoolProp's BibTeX key for each correlation, as written into the CSV's comment lines.
BY_KEY = {
    "Wagner-JPCRD-2002": "wagner-pruss2002", "Huber-JPCRD-2009": "huber2009-water", "Mulero-JPCRD-2012": "mulero2012",
    "Setzmann-JPCRD-1991": "setzmann-wagner1991-eos", "Sotiriadou-IJT-2025-methane": "sotiriadou2025-methane",
    "Buecker-JPCRD-2006": "buecker-wagner2006", "Friend-JPCRD-1991": "friend1991-ethane",
    "Gao-JPCRD-2023": "gao2023-ammonia", "Monogenidou-JPCRD-2018-ammonia": "monogenidou2018-ammonia",
    "Span-JPCRD-2000": "span2000-nitrogen", "Huber-IJT-2024-nitrogen": "huber2024-nitrogen",
    "OrtizVega-NISTIR-2023": "ortizvega2023-helium", "Arp-NIST-1998": "arp1998-helium",
    "Span-JPCRD-1996": "span-wagner1996-co2", "Laesecke-JPCRD-2017-CO2": "laesecke-muzny2017-co2",
    "Tegeler-JPCRD-1999": "tegeler1999-argon", "Lemmon-IJT-2004": "lemmon-jacobsen2004",
    "Lemmon-JCED-2009": "lemmon2009-propane", "Vogel-JPCRD-1998": "vogel1998-propane",
}
NEW_SUBSTANCES = {
    "c2h6": ({"c": 2, "h": 6}, "ethane"),
    "c3h8": ({"c": 3, "h": 8}, "propane"),
}


def read_sat():
    """Rows by substance and the per-fluid correlation keys from the CSV."""
    meta, rows = {}, {}
    with open(SAT_CSV) as f:
        lines = f.read().splitlines()
    for ln in lines:
        if ln.startswith("# ") and " eos=" in ln:
            parts = ln[2:].split()
            kv = dict(p.split("=", 1) for p in parts[2:])
            meta[parts[0]] = kv
    for r in csv.DictReader(ln for ln in lines if not ln.startswith("#")):
        rows.setdefault(r["substance"], []).append([float(r[k]) for k in ("t_k", "p_pa", "density_kg_m3", "viscosity_pa_s", "surface_tension_n_m")])
    return meta, rows


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


def apply(substances, pi, val):
    """Add the liquid optics and saturated-liquid tables to the substance rows
    in place; append the new substance rows (ethane, propane)."""
    U = "unverified"
    by_id = {s["id"]: s for s in substances}
    h2o = by_id["h2o"]
    meta, sat = read_sat()
    for sid, (formula, name) in NEW_SUBSTANCES.items():
        assert sid not in by_id, sid
        row = {"id": sid, "formula": formula, "names": {"physics": name, "plain": name, "story": "liquid " + name}, "phases": {}}
        substances.append(row)
        by_id[sid] = row
    for sid, rows in sat.items():
        m = meta[sid]
        eos, visc, st = BY_KEY[m["eos"]], BY_KEY[m["viscosity"]], BY_KEY[m["surface_tension"]]
        ts = [r[0] for r in rows]
        assert ts == sorted(ts) and all(r[2] > 0 and r[3] > 0 and r[4] > 0 for r in rows), sid
        assert all(a[2] > b[2] and a[4] > b[4] for a, b in zip(rows, rows[1:])), sid  # density and surface tension fall as it warms
        ph = by_id[sid].setdefault("phases", {})
        ph["saturated_liquid"] = {
            "kind": "table", "columns": ["t_k", "p_pa", "density", "viscosity", "surface_tension"],
            "units": ["K", "Pa", "kg m-3", "Pa s", "N m-1"], "rows": rows,
            "src": eos, "also": [visc, st, "coolprop"], "check": U,
            "src_by_column": {"p_pa": eos, "density": eos, "viscosity": visc, "surface_tension": st},
            "note": "liquid along its boiling curve, from just above the triple point (%s K) to 0.95 of the critical point (%s K); waves on a sea of it read density, viscosity and surface tension here" % (m["t_triple"], m["t_crit"]),
        }
        if sid in NEW_SUBSTANCES:
            ph["triple_point_k"] = val(float(m["t_triple"]), "K", eos, U)
            ph["critical_point_k"] = val(float(m["t_crit"]), "K", eos, U)
            ph["density_liquid"] = val(rows[0][2], "kg m-3", eos, U, at_k=rows[0][0], note="saturated liquid just above the triple point")
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
    # Pure water's own (molecular) scattering, for the sea's deep colour R = 0.33 b_b / (a + b_b).
    h2o["optical"]["scattering_liquid"] = {
        "kind": "table", "lambda_nm": [450.0, 550.0],
        "b": {"v": [0.0035, 0.0015], "unit": "m-1", "src": "morel-prieur1977", "check": U},
        "exponent": {"v": -4.3, "unit": "1", "src": "morel-prieur1977", "check": U, "note": "b scales as lambda to this power between and beyond the table's points"},
        "backscatter_share": {"v": 0.5, "unit": "1", "src": "morel-prieur1977", "check": U, "note": "b_b / b for molecular scattering, which is symmetric fore and aft"},
        "seawater_factor": {"v": 1.3, "unit": "1", "src": "morel-prieur1977", "check": U, "note": "seawater scatters about 30 % more than pure water (dissolved salts)"},
        "src": "morel-prieur1977", "check": U,
        "note": "pure liquid water; the 650 nm entry of Table 1 was unreadable in the scan, so use the exponent beyond 550 nm",
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
