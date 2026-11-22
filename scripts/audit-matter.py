#!/usr/bin/env python3
"""MP0 audit: every physical number typed into the code.

Walks crates/, packs/ and packages/ (TypeScript and JSON) and reports:

  1. Constants of nature and reference values (G, c, k, the Sun's radius,
     Earth's radii and so on): every literal that matches one, by file and
     line, with the value it holds. Where one quantity is written with more
     than one value, that is a disagreement.
  2. Substance, element and isotope properties: every pack value under a
     key that names a matter property (molar mass, density, triple point,
     latent heat, refractivity, half-life, ...) and every Rust `const` whose
     name does.

Usage:
  scripts/audit-matter.py                 print the summary
  scripts/audit-matter.py --json OUT      also write every hit as JSON
  scripts/audit-matter.py --markdown OUT  write the package-log tables
  scripts/audit-matter.py --retired FILE  fail if any retired constant listed
                                          in FILE (JSON list of quantity ids)
                                          is still typed outside sse-reference
  scripts/audit-matter.py --ledger FILE [--base REF | --base-ledger FILE]
                                          MP22 check: every typed copy must be
                                          a line of the ledger, every ledger
                                          line must still be a typed copy, and
                                          the ledger may not grow against the
                                          base (a git ref, or a file)
  scripts/audit-matter.py --write-ledger FILE
                                          write the ledger from today's scan
                                          (once, for MP22; never to label a
                                          new copy)
  --root DIR                              scan DIR instead of the repo (tests)

Exit codes for --ledger: 0 pass (prints "typed copies left: N"), 1 fail
(an unlabelled copy, a stale ledger line, or the ledger grew), 2 could not
run (bad arguments, unreadable ledger or base).

The scan is textual on purpose (no build needed, runs in CI in a second).
It finds numbers, not meaning: a literal that happens to equal 273.15 in an
unrelated place is still listed, and the table says which quantity it
matched so a reader can judge. Deterministic: files and hits are sorted.
"""
import json
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SCAN_DIRS = ["crates", "packs", "packages"]
SKIP_DIRS = {"target", "node_modules", "dist", "generated", ".git", "golden", "fixtures"}
EXTS = {".rs", ".ts", ".json", ".svelte", ".mjs"}
# The one crate allowed to type constants once MP1c lands.
HOME = os.path.join("crates", "sse-reference")
# Where the sheet itself lives: values here are the source, not copies.
# packs/base-sol is the catalogued Solar System: measured body facts, each
# cited, checked against the sheet by scripts/build-base-sol.py.
SHEET_HOMES = (HOME, os.path.join("packs", "base-reference"), os.path.join("packs", "base-sol"), os.path.join("crates", "sse-core", "src", "sheet.rs"))

# Quantity catalogue: id, name, canonical value, unit, source, relative
# tolerance for a match, and the unit scales the code might write it in.
# Canonical values: CODATA 2018 (Tiesinga et al. 2021), IAU 2015 B3
# nominal values (Prsa et al. 2016), NIST WebBook, IUPAC 2021.
Q = [
    # constants of nature
    ("G", "gravitational constant", 6.67430e-11, "m3 kg-1 s-2", "CODATA 2018", 2e-3, [1]),
    ("c", "speed of light", 299792458.0, "m/s", "CODATA 2018 (exact)", 1e-3, [1, 1e-3]),
    ("h", "Planck constant", 6.62607015e-34, "J s", "CODATA 2018 (exact)", 1e-3, [1]),
    ("hbar", "reduced Planck constant", 1.054571817e-34, "J s", "CODATA 2018", 1e-3, [1]),
    ("k_B", "Boltzmann constant", 1.380649e-23, "J/K", "CODATA 2018 (exact)", 1e-3, [1]),
    ("sigma_SB", "Stefan-Boltzmann constant", 5.670374419e-8, "W m-2 K-4", "CODATA 2018 (derived)", 1e-3, [1]),
    ("N_A", "Avogadro constant", 6.02214076e23, "1/mol", "CODATA 2018 (exact)", 1e-3, [1]),
    ("R_gas", "molar gas constant", 8.314462618, "J mol-1 K-1", "CODATA 2018 (derived)", 1e-3, [1]),
    ("e", "elementary charge (and the electronvolt in J)", 1.602176634e-19, "C", "CODATA 2018 (exact)", 1e-3, [1]),
    ("m_e", "electron mass", 9.1093837015e-31, "kg", "CODATA 2018", 1e-3, [1]),
    ("m_p", "proton mass", 1.67262192369e-27, "kg", "CODATA 2018", 5e-4, [1]),
    ("m_u", "atomic mass constant", 1.66053906660e-27, "kg", "CODATA 2018", 5e-4, [1]),
    ("a_rad", "radiation constant", 7.5657e-16, "J m-3 K-4", "CODATA 2018 (derived)", 1e-3, [1]),
    ("b_wien", "Wien displacement constant", 2.897771955e-3, "m K", "CODATA 2018", 1e-3, [1, 1e6]),
    ("R_inf", "Rydberg constant", 1.0973731568160e7, "1/m", "CODATA 2018", 1e-4, [1]),
    ("alpha", "fine-structure constant", 7.2973525693e-3, "1", "CODATA 2018", 1e-4, [1]),
    ("alpha_inv", "inverse fine-structure constant", 137.035999084, "1", "CODATA 2018", 1e-4, [1]),
    ("eps0", "vacuum permittivity", 8.8541878128e-12, "F/m", "CODATA 2018", 1e-3, [1]),
    ("mu0", "vacuum permeability", 1.25663706212e-6, "N/A2", "CODATA 2018", 1e-3, [1]),
    ("sigma_T", "Thomson cross-section", 6.6524587321e-29, "m2", "CODATA 2018", 1e-3, [1]),
    # units and conventions
    ("au", "astronomical unit", 149597870700.0, "m", "IAU 2012 B2 (exact)", 1e-3, [1, 1e-3]),
    ("pc", "parsec", 3.0856775814913673e16, "m", "IAU 2015 B2", 1e-3, [1, 1e-3]),
    ("ly", "light year", 9.4607304725808e15, "m", "IAU (Julian year x c)", 1e-3, [1, 1e-3]),
    ("yr_julian", "Julian year", 31557600.0, "s", "IAU", 1e-4, [1]),
    ("yr_sidereal", "sidereal year", 31558149.8, "s", "IERS", 2e-7, [1]),
    ("day", "day", 86400.0, "s", "SI", 1e-9, [1]),
    ("atm", "standard atmosphere", 101325.0, "Pa", "SI (exact)", 1e-4, [1, 1e-3, 1e-5]),
    ("g0", "standard gravity", 9.80665, "m/s2", "CGPM 1901 (exact)", 1e-3, [1]),
    ("T0C", "0 degrees Celsius", 273.15, "K", "SI (exact)", 1e-6, [1]),
    # reference objects
    ("GM_sun", "solar mass parameter", 1.3271244e20, "m3/s2", "IAU 2015 B3", 1e-3, [1, 1e-9]),
    ("M_sun", "solar mass", 1.9884099e30, "kg", "IAU 2015 B3 GM over CODATA 2018 G", 3e-3, [1, 1e3]),
    ("R_sun", "solar radius (nominal)", 6.957e8, "m", "IAU 2015 B3", 2e-3, [1, 1e-3]),
    ("L_sun", "solar luminosity (nominal)", 3.828e26, "W", "IAU 2015 B3", 2e-3, [1, 1e7]),
    ("T_sun", "solar effective temperature (nominal)", 5772.0, "K", "IAU 2015 B3", 2e-3, [1]),
    ("S_sun", "solar constant at 1 au (nominal)", 1361.0, "W/m2", "IAU 2015 B3", 2e-3, [1]),
    ("GM_earth", "Earth mass parameter", 3.986004e14, "m3/s2", "IAU 2015 B3", 1e-3, [1, 1e-9]),
    ("M_earth", "Earth mass", 5.9722e24, "kg", "IAU 2015 B3 (GM/G)", 2e-3, [1, 1e3]),
    ("R_earth_eq", "Earth equatorial radius", 6.3781e6, "m", "IAU 2015 B3", 5e-5, [1, 1e-3]),
    ("R_earth_mean", "Earth mean radius", 6.371e6, "m", "IUGG (mean radius)", 5e-5, [1, 1e-3]),
    ("R_earth_pol", "Earth polar radius", 6.3568e6, "m", "IAU 2015 B3", 5e-5, [1, 1e-3]),
    ("GM_jup", "Jupiter mass parameter", 1.2668653e17, "m3/s2", "IAU 2015 B3", 1e-3, [1, 1e-9]),
    ("M_jup", "Jupiter mass", 1.89813e27, "kg", "IAU 2015 B3 (GM/G)", 3e-3, [1, 1e3]),
    ("R_jup_eq", "Jupiter equatorial radius", 7.1492e7, "m", "IAU 2015 B3", 5e-5, [1, 1e-3]),
    ("R_jup_pol", "Jupiter polar radius", 6.6854e7, "m", "IAU 2015 B3", 5e-5, [1, 1e-3]),
    ("R_jup_mean", "Jupiter mean radius", 6.9911e7, "m", "NSSDCA", 5e-5, [1, 1e-3]),
    ("M_moon", "Moon mass", 7.346e22, "kg", "NSSDCA Moon fact sheet (DE440 GM over G gives 7.3458e22)", 3e-3, [1]),
    ("R_moon", "Moon mean radius", 1.7374e6, "m", "NSSDCA", 5e-4, [1, 1e-3]),
    ("a_moon", "Moon semi-major axis", 3.844e8, "m", "NSSDCA", 1e-3, [1, 1e-3]),
    ("Msun_Mearth", "Sun-to-Earth mass ratio", 332946.0487, "1", "IAU 2015 B3 GM ratio", 2e-3, [1]),
    ("Msun_Mjup", "Sun-to-Jupiter mass ratio", 1047.348644, "1", "IAU 2015 B3 GM ratio", 2e-3, [1]),
    ("au_Rsun", "astronomical unit in solar radii", 215.0320, "1", "IAU au over nominal R_sun", 1e-3, [1]),
    ("M_ch", "Chandrasekhar mass", 1.44, "M_sun", "textbook (mu_e = 2)", 1e-3, [1]),
    # substances (only distinctive values; round ones are found by key instead)
    ("T3_h2o", "water triple point", 273.16, "K", "NIST WebBook", 1e-6, [1]),
    ("P3_h2o", "water triple-point pressure", 611.657, "Pa", "NIST WebBook", 2e-3, [1]),
    ("M_h2o", "water molar mass", 0.0180153, "kg/mol", "IUPAC 2021", 2e-4, [1, 1e3]),
    ("M_co2", "carbon dioxide molar mass", 0.0440095, "kg/mol", "IUPAC 2021", 3e-4, [1, 1e3]),
    ("M_n2", "nitrogen molar mass", 0.0280134, "kg/mol", "IUPAC 2021", 2e-4, [1, 1e3]),
    ("M_co", "carbon monoxide molar mass", 0.0280101, "kg/mol", "IUPAC 2021", 2e-5, [1, 1e3]),
    ("M_o2", "oxygen molar mass", 0.0319988, "kg/mol", "IUPAC 2021", 2e-4, [1, 1e3]),
    ("M_ch4", "methane molar mass", 0.0160425, "kg/mol", "IUPAC 2021", 3e-4, [1, 1e3]),
    ("M_nh3", "ammonia molar mass", 0.0170305, "kg/mol", "IUPAC 2021", 3e-4, [1, 1e3]),
    ("M_ar", "argon molar mass", 0.039948, "kg/mol", "IUPAC 2021", 2e-4, [1, 1e3]),
    ("M_air", "dry air molar mass", 0.0289644, "kg/mol", "US Standard Atmosphere 1976", 1e-3, [1, 1e3]),
    ("T3_co2", "carbon dioxide triple point", 216.58, "K", "NIST WebBook", 1e-4, [1]),
    ("T3_ch4", "methane triple point", 90.694, "K", "NIST WebBook", 5e-4, [1]),
    ("T3_n2", "nitrogen triple point", 63.151, "K", "NIST WebBook", 5e-4, [1]),
    ("T3_nh3", "ammonia triple point", 195.495, "K", "NIST WebBook", 5e-4, [1]),
    ("Lv_h2o", "water latent heat of vaporisation (0 C)", 2.501e6, "J/kg", "NIST WebBook", 1e-3, [1]),
    ("Lf_h2o", "water latent heat of fusion", 3.34e5, "J/kg", "NIST WebBook", 3e-3, [1]),
    ("cp_air", "dry air heat capacity", 1004.0, "J kg-1 K-1", "textbook", 1e-3, [1]),
    ("Rd_air", "dry air gas constant", 287.05, "J kg-1 K-1", "R / M_air", 1e-3, [1]),
    # isotopes
    ("t_u238", "U-238 half-life", 4.468e9, "yr", "Ruedas 2017", 2e-3, [1]),
    ("t_u235", "U-235 half-life", 7.04e8, "yr", "Ruedas 2017", 2e-3, [1]),
    ("t_th232", "Th-232 half-life", 1.405e10, "yr", "Ruedas 2017", 2e-3, [1]),
    ("t_k40", "K-40 half-life", 1.248e9, "yr", "Ruedas 2017", 2e-3, [1]),
    ("t_al26", "Al-26 half-life", 7.17e5, "yr", "Ruedas 2017", 2e-3, [1]),
]

# Keys (packs) and const-name fragments (Rust) that name a matter property.
PROP_KEY = re.compile(
    r"(molar|molar_mass|molecular_weight|density|rho\b|triple|critical|latent|"
    r"melt|boil|condens|sublim|refractiv|king|heat_capacity|\bcp\b|cp_|"
    r"conductiv|half_life|halflife|decay|heat_w_kg|w_per_kg|abundance|"
    r"atomic_weight|vapou?r|freez|frost|ice_|_ice|viscos|strength|albedo_ice|"
    r"enthalpy|gibbs|free_energy|delta_g|delta_h|solubil)",
    re.I,
)
# Rust constant names that hold a constant of nature, a unit, a reference
# object or a matter property (curated from the crates, 21 Nov 2026).
PHYS_NAME = re.compile(
    r"^(G|G_SI|G0|G_SUN|G_KPC|G_PC3|C|C_M_S|C_PC_MYR|H|K_B|BOLTZMANN|SIGMA|SIGMA_SB|"
    r"STEFAN_BOLTZMANN|R_GAS|AMU|M_PROTON|M_H|MU0|MU_H|E_OVER_ME|E_ION_H|H_ALPHA_J|WIEN_B|"
    r"WIEN_NM_K|N_LOSCHMIDT|PHOTON_550|KEV_M|AU|AU_M|AU_KM|AU_RSUN|AU_IN_RSUN|PC|PC_M|MPC_M|"
    r"DAY|DAY_S|HOUR|YEAR|YEAR_S|YEAR_D|YEAR_DAYS|YR|YR_S|JULIAN_YEAR|JULIAN_YEAR_S|GYR_S|MYR_S|"
    r"KMS_\w+|GM_\w+|M_SUN\w*|MSUN\w*|SOLAR_MASS\w*|R_SUN|RSUN\w*|L_SUN\w*|T_SUN|TEFF_SUN|"
    r"OMEGA_SUN|M_EARTH\w*|MEARTH|ME_MSUN|R_EARTH\w*|EARTH_\w+|I_EARTH|V_ESC_EARTH|V_KEP_1AU|"
    r"M_JUP\w*|MJ\w*|R_JUPITER\w*|M_CH|L_EDD_PER_MSUN|RHO_\w+|M_SO2|NACL_MOLAR_MASS|"
    r"G_PER_MOL_C|T_MELT_ICE_K|ICE_K|K_ICE_651|SIGMA_ICE_PA|SIGMA_ROCK_PA|WATER_KF|AL26_\w+|"
    r"EXOSPHERE_K)$"
)
CONST_RE = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?const\s+([A-Z][A-Z0-9_]*)\s*:\s*f64\s*=\s*(.+?);")
STR_RE = re.compile(r'"(?:\\\\.|[^"\\\\])*"')
NUM_RE = re.compile(r"(?<![\w.])-?(?:\d[\d_]*\.?[\d_]*|\.\d[\d_]*)(?:[eE][+-]?\d+)?(?:_?f64|_?f32)?(?![\w.])")


def sig_digits(lit):
    s = lit.lower().replace("_", "").lstrip("-")
    s = re.sub(r"f(32|64)$", "", s)
    # Trailing zeros are not counted even after a point: "44.0" and
    # "300000000.0" are round numbers, not precise ones.
    mant = s.split("e")[0].replace(".", "").strip("0")
    return len(mant)


def to_float(lit):
    s = lit.replace("_", "")
    s = re.sub(r"f(32|64)$", "", s)
    try:
        return float(s)
    except ValueError:
        return None


def _tracked():
    """The repo's files, tracked or new but not ignored (so built, gitignored
    copies are never read),
    or None outside a git checkout."""
    import subprocess
    try:
        r = subprocess.run(["git", "-C", ROOT, "ls-files", "-z", "--cached", "--others", "--exclude-standard", "--"] + SCAN_DIRS, capture_output=True)
    except OSError:
        return None
    if r.returncode != 0 or not os.path.isdir(os.path.join(ROOT, ".git")) and not r.stdout:
        return None
    return set(r.stdout.decode("utf-8").split("\0")) - {""}


def walk():
    tracked = _tracked()
    for path in _walk_all():
        if tracked is None or path.replace(os.sep, "/") in tracked:
            yield path


def _walk_all():
    for top in SCAN_DIRS:
        base = os.path.join(ROOT, top)
        for d, dirs, files in os.walk(base):
            dirs[:] = sorted(x for x in dirs if x not in SKIP_DIRS)
            for f in sorted(files):
                if os.path.splitext(f)[1] in EXTS:
                    yield os.path.relpath(os.path.join(d, f), ROOT)


def match_quantity(v, lit):
    if v == 0:
        return None
    best = None
    for qid, name, val, unit, src, tol, scales in Q:
        for s in scales:
            ref = val * s
            rel = abs(v - ref) / abs(ref)
            if rel <= tol:
                if best is None or rel < best[1]:
                    best = ((qid, s), rel)
    if best is None:
        return None
    (qid, s), rel = best
    q = next(x for x in Q if x[0] == qid)
    # Round numbers only count when the catalogue value is itself round
    # (86400, 273.15) or the literal carries real precision.
    if sig_digits(lit) < 3 and qid not in ("M_ch",):
        return None
    return qid, s, rel


def is_comment(line, ext):
    t = line.strip()
    return ext in (".rs", ".ts", ".mjs", ".svelte") and (t.startswith("//") or t.startswith("*") or t.startswith("/*"))


def json_props(path):
    """Pack values under matter-property keys, with their JSON path."""
    try:
        data = json.load(open(os.path.join(ROOT, path), encoding="utf-8"))
    except (ValueError, UnicodeDecodeError):
        return []
    out = []

    def rec(o, p):
        if isinstance(o, dict):
            for k in sorted(o):
                v = o[k]
                kp = p + "/" + k
                if isinstance(v, (int, float)) and not isinstance(v, bool) and PROP_KEY.search(k):
                    out.append((kp, k, v))
                elif isinstance(v, list) and v and all(isinstance(x, (int, float)) and not isinstance(x, bool) for x in v) and PROP_KEY.search(k):
                    out.append((kp, k, v))
                else:
                    rec(v, kp)
        elif isinstance(o, list):
            for i, v in enumerate(o):
                ident = v.get("id") or v.get("name") or v.get("gas") or v.get("formula") if isinstance(v, dict) else None
                rec(v, p + "/" + (str(ident) if ident else str(i)))

    rec(data, "")
    return out


def scan():
    consts, props, rust_consts = [], [], []
    for path in walk():
        ext = os.path.splitext(path)[1]
        try:
            text = open(os.path.join(ROOT, path), encoding="utf-8").read()
        except UnicodeDecodeError:
            continue
        in_test = False
        for n, line in enumerate(text.split("\n"), 1):
            if ext == ".rs" and re.match(r"\s*#\[cfg\(test\)\]", line):
                in_test = True
            if is_comment(line, ext):
                continue
            code = line.split("//")[0] if ext in (".rs", ".ts", ".mjs", ".svelte") else line
            # Numbers inside strings are prose (paper volumes, page numbers,
            # explanations), not values the code computes with.
            code = STR_RE.sub('""', code)
            for m in NUM_RE.finditer(code):
                lit = m.group(0)
                v = to_float(lit)
                if v is None:
                    continue
                hit = match_quantity(abs(v), lit)
                if hit:
                    qid, s, rel = hit
                    test = in_test or "/tests/" in path or path.endswith("_tests.rs") or path.endswith("tests.rs") or "/test/" in path
                    consts.append({"q": qid, "file": path, "line": n, "literal": lit, "value": abs(v), "scale": s, "test": test})
            if ext == ".rs":
                cm = CONST_RE.match(line)
                if cm and PHYS_NAME.match(cm.group(1)):
                    rust_consts.append({"file": path, "line": n, "name": cm.group(1), "expr": cm.group(2).strip()})
        if ext == ".json" and path.startswith("packs"):
            for kp, k, v in json_props(path):
                props.append({"file": path, "path": kp, "key": k, "value": v})
    return consts, props, rust_consts


def disagreements(consts):
    by = {}
    for h in consts:
        by.setdefault(h["q"], []).append(h)
    out = []
    for q in Q:
        hs = by.get(q[0], [])
        vals = sorted({float("%.12g" % (h["value"] / h["scale"])) for h in hs})
        if len(vals) > 1:
            spread = (vals[-1] - vals[0]) / abs(q[2])
            if spread > 1e-9:
                out.append((q, vals, hs))
    return out


def verdict(lit_value, lit, canon):
    """'rounded' if the literal is the canonical value at its own precision,
    else 'differs'."""
    d = sig_digits(lit)
    rel = abs(lit_value - canon) / abs(canon)
    return "rounded" if rel <= 0.5 * 10 ** (1 - d) * 1.0001 or rel < 1e-12 else "differs"


def fmt(v):
    return ("%.10g" % v) if isinstance(v, float) else str(v)


def markdown(consts, props, rust_consts):
    L = []
    by = {}
    for h in consts:
        by.setdefault(h["q"], []).append(h)
    L.append("## A. Constants of nature, conventions and reference objects\n")
    L.append("One row per quantity: the canonical value, how many places type it (outside tests / in tests), the distinct values written, and where. A crate's own `consts` module counts as a typed place.\n")
    L.append("| Quantity | Canonical | Unit | Source | Typed (code / tests) | Distinct values written | Places (code) |")
    L.append("|---|---|---|---|---|---|---|")
    for qid, name, val, unit, src, tol, scales in Q:
        hs = by.get(qid, [])
        if not hs:
            continue
        code = [h for h in hs if not h["test"]]
        test = [h for h in hs if h["test"]]
        vals = sorted({float("%.12g" % (h["value"] / h["scale"])) for h in hs})
        files = sorted({h["file"] + ":" + str(h["line"]) for h in code})
        shown = ", ".join("`%s`" % f for f in files[:6]) + (" and %d more" % (len(files) - 6) if len(files) > 6 else "")
        L.append("| %s (`%s`) | %s | %s | %s | %d / %d | %s | %s |" % (name, qid, fmt(val), unit, src, len(code), len(test), ", ".join(fmt(x) for x in vals), shown or "tests only"))
    L.append("\n## B. Disagreements (one quantity, more than one value)\n")
    L.append("A value is **rounded** when it is the canonical value at the precision it is written with, and **differs** when it is not (a different definition, an older measurement or a slip).\n")
    L.append("| Quantity | Canonical | Values written (in the catalogue unit) | Where each value is |")
    L.append("|---|---|---|---|")
    for q, vals, hs in disagreements(consts):
        where = []
        for v in vals:
            mine = [h for h in hs if abs(h["value"] / h["scale"] - v) <= 1e-11 * abs(v)]
            fs = sorted({h["file"] + ":" + str(h["line"]) for h in mine})
            vd = verdict(v, mine[0]["literal"], q[2]) if mine else "?"
            where.append("%s (%s): %s" % (fmt(v), vd, ", ".join("`%s`" % f for f in fs[:4]) + (" +%d" % (len(fs) - 4) if len(fs) > 4 else "")))
        L.append("| %s (`%s`) | %s | %s | %s |" % (q[1], q[0], fmt(q[2]), ", ".join(fmt(v) for v in vals), "; ".join(where)))
    L.append("\n## C. Matter properties in packs\n")
    L.append("Every pack number under a key that names a matter property (molar mass, density, phase points, latent heat, optics, half-life, thermochemistry).\n")
    L.append("| Pack file | JSON path | Value |")
    L.append("|---|---|---|")
    for p in props:
        v = p["value"]
        vs = ", ".join(fmt(x) for x in v) if isinstance(v, list) else fmt(v)
        L.append("| `%s` | `%s` | %s |" % (p["file"], p["path"], vs))
    L.append("\n## D. Matter-property constants in Rust\n")
    L.append("| File | Line | Name | Value as written |")
    L.append("|---|---|---|---|")
    for r in rust_consts:
        L.append("| `%s` | %d | `%s` | `%s` |" % (r["file"], r["line"], r["name"], r["expr"].replace("|", "\\|")))
    return "\n".join(L) + "\n"


# ---------------------------------------------------------------------------
# MP22: the ledger of labelled typed copies.
#
# One line per typed copy, tab separated:  path  row  pattern
#   path     the file, from the repo root
#   row      the sheet row it should read: constant:<id>, substance:<id>/<prop>,
#            mixture:<id>/<prop>, isotope:<id>/<prop>, element:<id>/<prop>, or
#            new:<what> where the sheet has no row for it yet
#   pattern  code: the source line, trimmed, runs of spaces made one;
#            pack property: json:<JSON path>
# Lines are matched as a multiset, so a line typed three times is listed three
# times. Line numbers are left out on purpose, so edits elsewhere in a file do
# not touch the ledger.

# Catalogue id -> sheet row.
Q_ROW = {
    "R_earth_eq": "constant:R_earth_equatorial", "R_earth_pol": "constant:R_earth_polar",
    "R_earth_mean": "constant:R_earth_mean", "R_jup_eq": "constant:R_jup_equatorial",
    "R_jup_pol": "constant:R_jup_polar", "R_jup_mean": "constant:R_jup_mean",
    "alpha_inv": "constant:alpha (inverse)", "yr_sidereal": "new:sidereal year",
    "Msun_Mearth": "constant:GM_sun/GM_earth", "Msun_Mjup": "constant:GM_sun/GM_jup",
    "au_Rsun": "constant:au/R_sun", "M_ch": "new:Chandrasekhar mass",
    "T3_h2o": "substance:h2o/phases.triple_point_k", "P3_h2o": "substance:h2o/phases.triple_point_pa",
    "M_h2o": "substance:h2o/molar_mass", "M_co2": "substance:co2/molar_mass",
    "M_n2": "substance:n2/molar_mass", "M_co": "substance:co/molar_mass",
    "M_o2": "substance:o2/molar_mass", "M_ch4": "substance:ch4/molar_mass",
    "M_nh3": "substance:nh3/molar_mass", "M_ar": "substance:ar/molar_mass",
    "M_air": "mixture:dry-air/molar_mass", "T3_co2": "substance:co2/phases.triple_point_k",
    "T3_ch4": "substance:ch4/phases.triple_point_k", "T3_n2": "substance:n2/phases.triple_point_k",
    "T3_nh3": "substance:nh3/phases.triple_point_k",
    "Lv_h2o": "substance:h2o/phases.latent_vaporisation", "Lf_h2o": "new:h2o latent heat of fusion",
    "cp_air": "new:dry-air heat capacity", "Rd_air": "mixture:dry-air/gas_constant (R_gas over molar mass)",
    "t_u238": "isotope:u238/half_life_s", "t_u235": "isotope:u235/half_life_s",
    "t_th232": "isotope:th232/half_life_s", "t_k40": "isotope:k40/half_life_s",
    "t_al26": "isotope:al26/half_life_s",
}

# Rust const names whose value no catalogue literal matched (round, derived
# or not yet catalogued) -> sheet row.
NAME_ROW = [
    (r"^(M_SUN\w*|MSUN\w*|SOLAR_MASS\w*)$", "constant:M_sun"),
    (r"^(M_EARTH\w*|MEARTH|ME_MSUN)$", "constant:M_earth"),
    (r"^(M_JUP\w*|MJ\w*)$", "constant:M_jup"),
    (r"^(R_JUPITER\w*)$", "constant:R_jup_equatorial"),
    (r"^(R_EARTH\w*)$", "constant:R_earth_mean"),
    (r"^(RSUN\w*|R_SUN)$", "constant:R_sun"),
    (r"^(L_SUN\w*)$", "constant:L_sun"), (r"^(T_SUN|TEFF_SUN)$", "constant:T_sun"),
    (r"^(G_SUN)$", "constant:GM_sun/R_sun^2"),
    (r"^(G|G_SI|G_KPC|G_PC3)$", "constant:G"), (r"^(GM_SUN)$", "constant:GM_sun"),
    (r"^(GM_EARTH)$", "constant:GM_earth"), (r"^(GM_JUP\w*)$", "constant:GM_jup"),
    (r"^(C|C_M_S|C_PC_MYR)$", "constant:c"), (r"^(H)$", "constant:h"),
    (r"^(K_B|BOLTZMANN)$", "constant:k_B"), (r"^(SIGMA|SIGMA_SB|STEFAN_BOLTZMANN)$", "constant:sigma_SB"),
    (r"^(R_GAS)$", "constant:R_gas"), (r"^(AMU)$", "constant:m_u"), (r"^(M_PROTON)$", "constant:m_p"),
    (r"^(M_H)$", "element:h/atomic_mass"), (r"^(MU0)$", "constant:mu0"),
    (r"^(AU|AU_M|AU_KM)$", "constant:au"), (r"^(AU_RSUN|AU_IN_RSUN)$", "constant:au/R_sun"),
    (r"^(PC|PC_M|MPC_M)$", "constant:pc"), (r"^(KMS_\w+)$", "constant:pc and yr_julian (km/s in pc per Myr)"),
    (r"^(DAY|DAY_S)$", "constant:day"), (r"^(HOUR)$", "unit:h"),
    (r"^(YEAR\w*|YR\w*|JULIAN_YEAR\w*|GYR_S|MYR_S)$", "constant:yr_julian"),
    (r"^(WIEN_B|WIEN_NM_K)$", "constant:b_wien"), (r"^(N_LOSCHMIDT)$", "new:Loschmidt constant (p0 over k_B T0)"),
    (r"^(E_OVER_ME)$", "constant:e/m_e"), (r"^(KEV_M)$", "constant:h c / keV"),
    (r"^(AL26_\w+)$", "isotope:al26"), (r"^(RHO_ICE)$", "substance:h2o/phases.density_solid"),
    (r"^(RHO_WATER)$", "substance:h2o/phases.density_liquid"),
    (r"^(G_PER_MOL_C)$", "element:c/atomic_weight"), (r"^(M_SO2)$", "substance:so2/molar_mass"),
    (r"^(NACL_MOLAR_MASS)$", "new:NaCl molar mass"),
    (r"^(T_MELT_ICE_K)$", "substance:h2o/phases.triple_point_k"),
]

# Pack keys the property scan finds that are not a material number: facts of
# one named body, model settings, shapes. Listed so the ledger holds only real
# copies. (file fragment, JSON path regex, reason)
NOT_MATTER = [
    ("packs/", r"/examples/[^/]+/values/(body\.bulk_density|surface\.ice_fraction)$", "a named body's measured fact"),
    ("packs/sse3-classic/", r"body\.bulk_density$", "a named body's fact (V3 import)"),
    ("packs/base-light/", r"/lines/[^/]+/strength$", "an absorption line's shape"),
    ("packs/base-cosmos/", r"density_pc3$", "a star density in a galaxy model"),
    ("packs/base-cosmology/", r"/(midplane_density|triple_share)$", "a galaxy model setting"),
    ("packs/base-nav/", r"/(top_density|parking_radii)$", "a navigation setting"),
    ("packs/base-life/", r"/decay_methane_(oxic|anoxic)$", "a model rate"),
    ("packs/base-climate/", r"/(delta_h|latent_scale_w_m2|frost_emissivity|lowest_frost_k|t_ice_k|"
     r"super_rotation_critical|latent_share|ddf_ice_mm)$", "a climate model setting"),
    ("packs/base-surface/", r"/(ice_dry_k|land_ice_k|shadow_ice_max_bar|ice_min_km2|impactor_density|"
     r"ice_giant_mass_earth|wind_ice_m_s)$", "a surface model setting"),
    ("packs/base-weather/", r"/(equator_ice|equatorial_bands_ice|ice_giant_mass_earth|wind_ice_m_s|"
     r"strength_u|strength_m_s)$", "a weather model setting"),
    ("packs/base-weather/", r"/impact/density_kg_m3$", "an assumed impactor, a model setting"),
]

# Pack key -> property on the sheet (the substance or mixture is the key's
# parent in the JSON path when the sheet has it).
KEY_PROP = [
    (r"molar|molecular_weight", "molar_mass"), (r"^king", "optical.king_factor"),
    (r"refractiv", "optical.refractivity_589nm"), (r"cp_|heat_capacity", "thermal.heat_capacity"),
    (r"density", "density"), (r"half_life", "half_life_s"), (r"albedo", "optical.albedo"),
    (r"sea_ice", "freezing_point (sea water)"),
]
PARENT_SUB = {
    "H2O": "substance:h2o", "water": "substance:h2o", "air": "mixture:dry-air",
    "water-ice sand": "substance:h2o/phases.density_solid", "methane-ice sand": "substance:ch4/phases.density_solid",
    "CO2": "substance:co2", "CO2 (thin)": "mixture:mars-air", "N2-O2": "mixture:dry-air",
    "earth": "mixture:dry-air", "H2-He": "mixture:giant-air", "N2-CH4": "mixture:titan-air",
    "rock sand": "mixture:surface-rock", "clouds": "mixture:giant-air",
    "rings": "isotope:k40",
}


def _norm(line):
    return re.sub(r"\s+", " ", line.strip())


def _q_row(qid, ids):
    if qid in Q_ROW:
        return Q_ROW[qid]
    return "constant:" + qid if qid in ids else "new:" + qid


def _name_row(name):
    for rx, row in NAME_ROW:
        if re.match(rx, name):
            return row
    return "new:" + name.lower()


def _prop_row(path, key, sheet_subs, sheet_mix=()):
    row = _prop_row1(path, key, sheet_subs)
    m = re.match(r"mixture:([^/]+)", row)
    return "new:" + row if m and m.group(1) not in sheet_mix else row


def _prop_row1(path, key, sheet_subs):
    parts = path.strip("/").split("/")
    parent = parts[-2] if len(parts) > 1 else ""
    prop = next((p for rx, p in KEY_PROP if re.search(rx, key, re.I)), None)
    owner = PARENT_SUB.get(parent)
    if owner is None and parent.lower() in sheet_subs:
        owner = "substance:" + parent.lower()
    if owner and "/" in owner:
        return owner
    if owner and prop:
        return owner + "/" + prop
    if owner:
        return owner + "/" + key
    return "new:" + (parent + " " if parent and not parent.isdigit() else "") + key


def _is_test(path):
    return "/tests/" in path or path.endswith("tests.rs") or "/test/" in path


def _has_typed_number(expr):
    code = STR_RE.sub('""', expr)
    for m in NUM_RE.finditer(code):
        v = to_float(m.group(0))
        if v is not None and abs(v) not in (0.0, 1.0, 2.0):
            return True
    return False


def ledger_entries(consts, props, rust_consts):
    """Every typed copy outside the sheet and outside tests, as ledger lines."""
    sheet_ids, sheet_subs, sheet_mix = set(), set(), set()
    try:
        ref = json.load(open(os.path.join(ROOT, "packs", "base-reference", "pack.json"), encoding="utf-8"))
        sheet_ids = {c["id"] for c in ref["provides"]["reference"]["constants"]}
        sheet_subs = {c["id"] for c in ref["provides"]["reference"]["substances"]}
        sheet_mix = {c["id"] for c in ref["provides"]["reference"]["mixtures"]}
    except (OSError, ValueError, KeyError, TypeError):
        pass
    lines_of = {}

    def src(path, n):
        if path not in lines_of:
            lines_of[path] = open(os.path.join(ROOT, path), encoding="utf-8").read().split("\n")
        return _norm(lines_of[path][n - 1])

    out = []
    seen_lines = set()
    for h in consts:
        if h["test"] or h["file"].startswith(SHEET_HOMES):
            continue
        out.append((h["file"], _q_row(h["q"], sheet_ids), src(h["file"], h["line"])))
        seen_lines.add((h["file"], h["line"]))
    for r in rust_consts:
        if r["file"].startswith(SHEET_HOMES) or _is_test(r["file"]) or (r["file"], r["line"]) in seen_lines:
            continue
        if r.get("test") or not _has_typed_number(r["expr"]):
            continue
        out.append((r["file"], _name_row(r["name"]), src(r["file"], r["line"])))
    for p in props:
        if p["file"].startswith(SHEET_HOMES):
            continue
        if any(p["file"].startswith(f) and re.search(rx, p["path"]) for f, rx, _ in NOT_MATTER):
            continue
        out.append((p["file"], _prop_row(p["path"], p["key"], sheet_subs, sheet_mix), "json:" + p["path"]))
    return sorted(out)


LEDGER_HEAD = """# MP22 ledger: every constant of nature, reference value and material number
# still typed outside the master data sheet (packs/base-reference and
# crates/sse-reference), outside tests. Design: SSE4-matter-pack-design.md 3A.
#
# Columns, tab separated: file, the sheet row it should read, the line as
# written (code) or json:<path> (a pack property). "new:" means the sheet has
# no row for it yet; add the row when the copy is moved.
#
# This file may only shrink. When you edit a file listed here, move its copies
# to the Look up block or the reader and delete their lines in the same change.
# scripts/audit-matter.py --ledger fails on a copy not listed here, on a line
# here that no longer matches a copy, and on a ledger that grew.
"""


def read_ledger(text):
    rows = []
    for raw in text.split("\n"):
        if not raw.strip() or raw.startswith("#"):
            continue
        parts = raw.split("\t")
        if len(parts) != 3:
            raise ValueError("ledger line is not three tab-separated columns: %r" % raw)
        rows.append(tuple(parts))
    return rows


def _multiset(rows):
    c = {}
    for r in rows:
        c[r] = c.get(r, 0) + 1
    return c


def _less(a, b):
    """Entries of multiset a beyond those in b, as a sorted list."""
    out = []
    for k in sorted(a):
        out += [k] * max(0, a[k] - b.get(k, 0))
    return out


def check_ledger(found, ledger, base):
    """Returns (problems, count). found, ledger, base: lists of 3-tuples; base
    may be None (no growth check)."""
    probs = []
    f, l = _multiset(found), _multiset(ledger)
    key = lambda r: (r[0], r[2])  # the row column is a label, not identity
    fk, lk = _multiset([key(r) for r in found]), _multiset([key(r) for r in ledger])
    for path, pat in _less(fk, lk):
        row = next(r[1] for r in found if key(r) == (path, pat))
        probs.append("unlabelled typed copy (move it to the sheet, or read the sheet): %s: %s  [should read %s]" % (path, pat, row))
    for path, pat in _less(lk, fk):
        probs.append("stale ledger line (the copy is gone; delete the line): %s\t%s" % (path, pat))
    if base is not None:
        if len(ledger) > len(base):
            probs.append("the ledger grew from %d to %d lines; it may only shrink" % (len(base), len(ledger)))
        # A changed line (same file, same row) is an edit, not a new copy.
        added = _less(l, _multiset(base))
        removed = _multiset([(r[0], r[1]) for r in _less(_multiset(base), l)])
        for r in added:
            k = (r[0], r[1])
            if removed.get(k, 0) > 0:
                removed[k] -= 1
            else:
                probs.append("ledger line added (label nothing new; read the sheet instead): %s" % "\t".join(r))
    del f
    return probs, len(ledger)


def git_show(ref, path):
    import subprocess
    rel = os.path.relpath(os.path.abspath(path), ROOT).replace(os.sep, "/")
    r = subprocess.run(["git", "-C", ROOT, "show", "%s:%s" % (ref, rel)], capture_output=True, text=True)
    if r.returncode != 0 and ("exists on disk, but not in" in r.stderr or "does not exist in" in r.stderr):
        return None  # the ledger is new in this change: nothing to grow from
    if r.returncode != 0:
        raise OSError("cannot read %s at %s: %s" % (rel, ref, r.stderr.strip()))
    return r.stdout


def _arg(argv, flag):
    if flag not in argv:
        return None
    i = argv.index(flag)
    if i + 1 >= len(argv):
        raise ValueError("%s needs a value" % flag)
    return argv[i + 1]


def main(argv):
    global ROOT
    try:
        root = _arg(argv, "--root")
        ledger_path = _arg(argv, "--ledger")
        write_path = _arg(argv, "--write-ledger")
        base_ref = _arg(argv, "--base")
        base_file = _arg(argv, "--base-ledger")
    except ValueError as e:
        print("audit-matter: %s" % e)
        return 2
    if root:
        ROOT = os.path.abspath(root)
    consts, props, rust_consts = scan()
    if write_path:
        entries = ledger_entries(consts, props, rust_consts)
        with open(write_path, "w", encoding="utf-8", newline="\n") as fh:
            fh.write(LEDGER_HEAD + "".join("\t".join(e) + "\n" for e in entries))
        print("ledger written: %d typed copies" % len(entries))
        return 0
    if ledger_path:
        try:
            ledger = read_ledger(open(ledger_path, encoding="utf-8").read())
            base = None
            if base_file:
                base = read_ledger(open(base_file, encoding="utf-8").read())
            elif base_ref:
                shown = git_show(base_ref, ledger_path)
                base = None if shown is None else read_ledger(shown)
        except (OSError, ValueError) as e:
            print("audit-matter: %s" % e)
            return 2
        probs, n = check_ledger(ledger_entries(consts, props, rust_consts), ledger, base)
        for p in probs:
            print(p)
        print("typed copies left: %d" % n)
        if base is None:
            print("(no base ledger, so the may-only-shrink check did not run)")
        return 1 if probs else 0
    if "--retired" in argv:
        retired = set(json.load(open(argv[argv.index("--retired") + 1])))
        left = [h for h in consts if h["q"] in retired and not h["file"].startswith(HOME)]
        for h in left:
            print("%s:%d typed %s (%s)" % (h["file"], h["line"], h["q"], h["literal"]))
        return 1 if left else 0
    if "--json" in argv:
        json.dump({"constants": consts, "matter_props": props, "rust_consts": rust_consts}, open(argv[argv.index("--json") + 1], "w"), indent=1, sort_keys=True)
    if "--markdown" in argv:
        open(argv[argv.index("--markdown") + 1], "w").write(markdown(consts, props, rust_consts))
    code = [h for h in consts if not h["test"]]
    print("constant literals: %d (%d outside tests) for %d quantities" % (len(consts), len(code), len({h["q"] for h in consts})))
    print("pack matter properties: %d in %d files" % (len(props), len({p["file"] for p in props})))
    print("rust matter-property consts: %d" % len(rust_consts))
    print("disagreements: %d" % len(disagreements(consts)))
    for q, vals, hs in disagreements(consts):
        print("  %-14s %s" % (q[0], ", ".join(fmt(v) for v in vals)))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
