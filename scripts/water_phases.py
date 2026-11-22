"""Water's phases for the master data sheet (track MP, package MP18; schema MP17, design section 4.3A).

Read by scripts/build-base-reference.py, which puts the output under substances[h2o].phases:

    forms       one row per phase: id, names (physics, plain, story), kind, stability, evidence, props
    boundaries  one row per pair of stable phases that touch: between, curve, evidence, src
    meets       triple points, each written once
    table       the lookup that phase_at answers from, built here from the same curves

Every curve below is the one the table is built from, so the table and the boundaries cannot drift.
Curve forms (all pressures in Pa, temperatures in K):

    iapws2011_ih      p = pt (1 + sum_i a_i (1 - th^b_i)),  th = T/Tt; coeffs [Tt, pt, a1, a2, a3, b1, b2, b3]
    iapws2011_simple  p = pt (1 - a (1 - th^b));                       coeffs [Tt, pt, a, b]
    iapws2011_vii     ln(p/pt) = a1 (1 - 1/th) + a2 (1 - th^5) + a3 (1 - th^22); coeffs [Tt, pt, a1, a2, a3]
    iapws2011_subl    ln(p/pt) = (1/th) sum_i a_i th^b_i;              coeffs [Tt, pt, a1, a2, a3, b1, b2, b3]
    iapws1992_sat     ln(p/pc) = (Tc/T)(a1 t + a2 t^1.5 + a3 t^3 + a4 t^3.5 + a5 t^4 + a6 t^7.5), t = 1 - T/Tc;
                      coeffs [Tc, pc, a1..a6]
    simon             p = p0 + A ((T/T0)^c - 1);                       coeffs [p0, A, T0, c]
    points            [[T, p], ...]: straight between points in T and log10 p
    isotherm, isobar  a line of constant T (p range) or constant p (T range)

Each curve holds for the "t_k" (or "p_pa") range it gives. Evidence grades: measured, computed (from
published simulations, where no laboratory reaches), extrapolated (a measured curve carried past its data),
convention (no phase change happens there; the line is where the names change, as for the critical point).
Python standard library only; the output is deterministic.
"""
import math

TC, PC = 647.096, 22.064e6  # IAPWS critical point
GPA, MPA = 1e9, 1e6

# ---------------------------------------------------------------- curves (the data)
SUBL = [273.16, 611.657, -21.2144006, 27.3203819, -6.10598130, 0.333333333e-2, 1.20666667, 1.70333333]
SAT = [TC, PC, -7.85951783, 1.84408259, -11.7866497, 22.6807411, -15.9618719, 1.80122502]
MELT_IH = [273.16, 611.657, 0.119539337e7, 0.808183159e5, 0.333826860e4, 3.0, 25.75, 103.75]
MELT_III = [251.165, 208.566 * MPA, 0.299948, 60.0]
MELT_V = [256.164, 350.1 * MPA, 1.18721, 8.0]
MELT_VI = [273.31, 632.4 * MPA, 1.07476, 4.6]
MELT_VII = [355.0, 2216 * MPA, 1.73683, -0.544606e-1, 0.806106e-7]
SIMON_VII = [2.17 * GPA, 0.764 * GPA, 355.0, 4.32]  # Frank, Fei and Hu 2004, 3 to 60 GPa


def f_subl(T, c=SUBL):
    th = T / c[0]
    return c[1] * math.exp((c[2] * th ** c[5] + c[3] * th ** c[6] + c[4] * th ** c[7]) / th)


def f_sat(T, c=SAT):
    t = 1 - T / c[0]
    return c[1] * math.exp(c[0] / T * (c[2] * t + c[3] * t ** 1.5 + c[4] * t ** 3 + c[5] * t ** 3.5 + c[6] * t ** 4 + c[7] * t ** 7.5))


def f_melt_ih(T, c=MELT_IH):
    th = T / c[0]
    return c[1] * (1 + c[2] * (1 - th ** c[5]) + c[3] * (1 - th ** c[6]) + c[4] * (1 - th ** c[7]))


def f_simple(T, c):
    return c[1] * (1 - c[2] * (1 - (T / c[0]) ** c[3]))


def f_melt_vii(T, c=MELT_VII):
    th = T / c[0]
    return c[1] * math.exp(c[2] * (1 - 1 / th) + c[3] * (1 - th ** 5) + c[4] * (1 - th ** 22))


def f_simon(T, c=SIMON_VII):
    return c[0] + c[1] * ((T / c[2]) ** c[3] - 1)


def bisect(f, lo, hi, n=200):
    """Root of f in [lo, hi] (f changes sign); fixed step count, so deterministic."""
    flo = f(lo)
    for _ in range(n):
        mid = (lo + hi) / 2
        fm = f(mid)
        if (fm < 0) == (flo < 0):
            lo, flo = mid, fm
        else:
            hi = mid
    return (lo + hi) / 2


def lerp_logp(pts, T):
    """p at T along a points curve, straight in T and log10 p (pts sorted by T)."""
    for (t0, p0), (t1, p1) in zip(pts, pts[1:]):
        if t0 <= T <= t1:
            f = (T - t0) / (t1 - t0)
            return 10 ** (math.log10(p0) + f * (math.log10(p1) - math.log10(p0)))
    raise ValueError(T)


def t_on(pts, p):
    """T at p along a points curve whose p rises (or falls) monotonically with its index."""
    lp = math.log10(p)
    for (t0, p0), (t1, p1) in zip(pts, pts[1:]):
        a, b = math.log10(p0), math.log10(p1)
        if min(a, b) <= lp <= max(a, b):
            return t0 + (t1 - t0) * (lp - a) / (b - a) if b != a else t0
    raise ValueError(p)


# Where the IAPWS ice VII curve hands over to Frank et al.'s Simon fit: the upper of their two crossings
# (they agree to 0.1% near 480 K and again near 590 K; IAPWS's own data stop at 715 K, but its 22nd-power
# term runs away above about 650 K, and Frank et al. measured melting to 39.6 GPa).
T_VII_SWITCH = bisect(lambda T: f_melt_vii(T) - f_simon(T), 560.0, 620.0)

# Triple point A: ice VII, superionic (bcc) ice and fluid. Lin et al. 2005 saw the melting curve break at
# about 35 GPa (and 1040 K); the temperature here is Frank et al.'s melting curve at that pressure, so the
# curves meet. Published placements of this point range from 14.6 GPa, 850 K (Queyroux et al. 2020, quoted
# by Cheng et al. 2021) to about 47 GPa, 1000 K: it is contested, and marked so.
P_A = 35 * GPA
T_A = bisect(lambda T: f_simon(T) - P_A, 600.0, 1500.0)

# Superionic melting, computed: Cheng et al. 2021 give 3250 K at 100 GPa; Millot et al. 2018 reached about
# 5000 K at 190 GPa (recalled, to verify); carried on to 1 TPa at the same slope in log p (extrapolated).
SI_MELT = [[T_A, P_A], [3250.0, 100 * GPA], [5000.0, 190 * GPA]]
_s = (math.log10(190 * GPA) - math.log10(100 * GPA)) / (5000.0 - 3250.0)
SI_MELT.append([5000.0 + (12 - math.log10(190 * GPA)) / _s, 1e12])

# Where ice VII or X becomes superionic (protons start to flow): from A to about 2000 K at 100 GPa (Cheng et
# al. 2021: T_s about 2000 K; Millot et al. 2019: superionic from 2000 K at 100 to 400 GPa), then flat.
SI_ONSET = [[T_A, P_A], [2000.0, 100 * GPA], [2000.0, 1e12]]

# Ice VII to ice X (the hydrogen bond turns symmetric): about 62 GPa at 100 K (VII-VIII-X triple point),
# 60 GPa at 300 K (Goncharov et al. 1999, quoted by Lin et al. 2005), falling with temperature (Lin et al.
# 2005: a negative Clapeyron slope, aimed at their 35 GPa, 1040 K break) until it meets the superionic onset.
VII_X_DIR = [[300.0, 60 * GPA], [1040.0, 35 * GPA]]


def _vii_x_p(T):
    if T <= 300.0:
        return 10 ** (math.log10(62 * GPA) + (T - 100.0) / 200.0 * (math.log10(60 * GPA) - math.log10(62 * GPA)))
    (t0, p0), (t1, p1) = VII_X_DIR
    return 10 ** (math.log10(p0) + (T - t0) / (t1 - t0) * (math.log10(p1) - math.log10(p0)))


def _onset_t(p):
    if p >= 100 * GPA:
        return 2000.0
    if p <= P_A:
        return T_A
    return t_on(SI_ONSET[:2], p)


# Triple point B: ice VII, ice X and superionic ice, where the VII-X line meets the superionic onset.
T_B = bisect(lambda T: _onset_t(_vii_x_p(T)) - T, 300.0, 2000.0)
P_B = _vii_x_p(T_B)

# bcc to fcc superionic ice: Cheng et al. 2021, bcc stable below about 75 GPa (computed); Millot et al. 2019
# saw fcc (ice XVIII) at 100 to 400 GPa.
P_BCC_FCC = 75 * GPA
T_C = _onset_t(P_BCC_FCC)  # ice X, bcc and fcc superionic
T_D = t_on(SI_MELT, P_BCC_FCC)  # bcc, fcc and ionic fluid

# Fluid water turns ionic (molecules split, the fluid conducts): a gradual crossover, drawn at 2000 K above
# 20 GPa (recalled from Mattsson and Desjarlais 2006 and Redmer et al. 2011; to verify).
T_IONIC, P_IONIC = 2000.0, 20 * GPA
P_E = lerp_logp(SI_MELT, T_IONIC)  # superionic (bcc) melting at 2000 K: bcc, supercritical and ionic fluid

# Solid-solid triple points (T K, p Pa) and the straight boundaries between them.
TP = {
    "ih_ii_iii": (238.45, 212.9 * MPA),
    "ii_iii_v": (248.85, 344.3 * MPA),
    "ii_v_vi": (218.0, 620 * MPA),
    "ii_vi_xv": (130.0, 800 * MPA),
    "vi_viii_xv": (130.0, 1500 * MPA),
    "vi_vii_viii": (278.0, 2100 * MPA),
    "vii_viii_x": (100.0, 62 * GPA),
    "ih_ii_xi": (73.4, 70 * MPA),
    "vap_ih_xi": (72.0, None),  # p from the sublimation curve
}
T_LO, T_HI = 10.0, 1e4


def _line(a, b, T):
    """p at T on the boundary from a to b: straight in T and log10 p, as every points curve is read."""
    (t0, p0), (t1, p1) = a, b
    return 10 ** (math.log10(p0) + (math.log10(p1) - math.log10(p0)) * (T - t0) / (t1 - t0))


def p_ih_ii(T):  # also XI-II below 73.4 K (extrapolated)
    return _line(TP["ih_ii_xi"], TP["ih_ii_iii"], T)


def p_ih_iii(T):
    return _line(TP["ih_ii_iii"], (251.165, 208.566 * MPA), T)


def t_ii_iii(p):  # the II-III line as T(p), straight in T and log10 p
    (t0, p0), (t1, p1) = TP["ih_ii_iii"], TP["ii_iii_v"]
    return t0 + (t1 - t0) * (math.log10(p) - math.log10(p0)) / (math.log10(p1) - math.log10(p0))


def p_iii_v(T):
    return _line(TP["ii_iii_v"], (256.164, 350.1 * MPA), T)


def p_ii_v(T):
    return _line(TP["ii_v_vi"], TP["ii_iii_v"], T)


def p_v_vi(T):
    return _line(TP["ii_v_vi"], (273.31, 632.4 * MPA), T)


def p_ii_vi(T):  # also II-XV below 130 K (extrapolated)
    return _line(TP["ii_vi_xv"], TP["ii_v_vi"], T)


def p_vi_viii(T):
    return _line(TP["vi_viii_xv"], TP["vi_vii_viii"], T)


def p_vi_vii(T):
    return _line(TP["vi_vii_viii"], (355.0, 2216 * MPA), T)


def t_vii_viii(p):  # VII-VIII: from (278 K, 2.1 GPa) to (100 K, 62 GPa), straight in T and log p
    return t_on([list(TP["vi_vii_viii"]), list(TP["vii_viii_x"])], p)


def p_melt(T):
    """Pressure above which water is solid at temperature T (T >= 251.165 K), and that solid's family."""
    if T < 256.164:
        return f_simple(T, MELT_III)
    if T < 273.31:
        return f_simple(T, MELT_V)
    if T < 355.0:
        return f_simple(T, MELT_VI)
    if T < T_VII_SWITCH:
        return f_melt_vii(T)
    if T < T_A:
        return f_simon(T)
    if T > SI_MELT[-1][0]:
        return math.inf  # past the last point (1 TPa): fluid everywhere the table reaches
    return lerp_logp(SI_MELT, T)


def fluid(T, p):
    if T >= T_IONIC and p >= P_IONIC:
        return "h2o.ionic_fluid"
    if T >= TC and p >= PC:
        return "h2o.supercritical"
    if T >= TC or (T >= 273.16 and p < f_sat(T)):
        return "h2o.vapour"
    return "h2o.liquid"


def t_ih_xi(p):  # Ih-XI: from (72 K, the vapour pressure) to the Ih-II-XI point (73.4 K, 70 MPa)
    lp0, lp1 = math.log10(f_subl(72.0)), math.log10(TP["ih_ii_xi"][1])
    return 72.0 + (TP["ih_ii_xi"][0] - 72.0) * (math.log10(max(p, 1e-300)) - lp0) / (lp1 - lp0)


def solid(T, p):
    """The stable ice at (T, p), given that water is solid there. Low pressure first."""
    if T < 273.16:
        up = p_ih_ii(T) if T < 238.45 else (p_ih_iii(T) if T < 251.165 else f_melt_ih(T))
        if p < up:
            return "h2o.ice_xi" if T < t_ih_xi(min(p, TP["ih_ii_xi"][1])) else "h2o.ice_ih"
    if p >= P_A or T >= T_A:
        if T >= _onset_t(p) or (T >= T_A and p < P_A):
            return "h2o.ice_xx" if p < P_BCC_FCC else "h2o.ice_xviii"
    if p >= P_B or (p >= 50 * GPA and p >= _vii_x_p(T)):
        if p >= _vii_x_p(T):
            return "h2o.ice_x"
    if T < 100.0 and p >= 62 * GPA:
        return "h2o.ice_x"
    # ice VII and VIII
    if (T >= 355.0) or (T >= 278.0 and p >= p_vi_vii(T)) or (130.0 <= T < 278.0 and p >= p_vi_viii(T)) or (T < 130.0 and p >= 1500 * MPA):
        if p >= 2100 * MPA and T < t_vii_viii(min(p, 62 * GPA)):
            return "h2o.ice_viii"
        if T < 278.0 and p < 2100 * MPA:
            return "h2o.ice_viii"
        return "h2o.ice_vii"
    # ice VI and XV
    if (T >= 273.31) or (218.0 <= T and p >= p_v_vi(T)) or (T < 218.0 and p >= p_ii_vi(T)):
        return "h2o.ice_xv" if T < 130.0 else "h2o.ice_vi"
    # ice V
    if (T >= 256.164) or (248.85 <= T and p >= p_iii_v(T)) or (218.0 <= T < 248.85 and p >= p_ii_v(T)):
        return "h2o.ice_v"
    # ice III, II, Ih, XI
    if T >= 238.45 and p >= min(p_ih_iii(T), 212.9 * MPA) and T >= t_ii_iii(p):
        return "h2o.ice_iii"
    return "h2o.ice_ii"


def phase_of(T, p):
    if T < 273.16:
        if p < f_subl(T):
            return "h2o.vapour"
        if T < 251.165:
            return solid(T, p)
        if p < f_melt_ih(T):
            return solid(T, p)
        return "h2o.liquid" if p < p_melt(T) else solid(T, p)
    if T < TC and p < f_sat(T):
        return "h2o.vapour"
    if T >= TC and p < PC:
        return "h2o.vapour"
    return fluid(T, p) if p < p_melt(T) else solid(T, p)


# ---------------------------------------------------------------- forms
def _d(v, src, check="verified", **extra):
    out = {"v": v, "unit": "kg m-3", "src": src}
    if check != "verified":
        out["check"] = check
    out.update(extra)
    return out


def _ev(grade, t_k, p_pa, src, **extra):
    out = {"grade": grade, "t_k": t_k, "p_pa": p_pa, "src": src}
    out.update(extra)
    return out


LSBU_NOTE = "experimental density at the phase's own conditions (LSBU structural table, column a)"

FORMS = [
    # id, (physics, plain, story), kind, stability, evidence, props
    ("h2o.vapour", ("water vapour", "steam", "water vapour"), "gas", "stable",
     _ev("measured", [T_LO, T_HI], [1e-10, PC], "iapws2011-meltsub", also=["iapws1992-sat"]), {}),
    ("h2o.liquid", ("liquid water", "water", "water"), "liquid", "stable",
     _ev("measured", [251.165, TC], [611.657, 20 * GPA], "iapws1992-sat", also=["iapws2011-meltsub"]),
     {"density": _d(999.789, "iapws1992-sat", at_k=273.16, at_pa=611.657, note="saturated liquid at the triple point (the release's check value)"),
      "sigma_elec": {"v": 5.5e-6, "unit": "S m-1", "src": "recalled", "check": "unverified", "at_k": 298.15, "note": "pure water, from its own ions; any dissolved salt raises it by orders of magnitude"}}),
    ("h2o.supercritical", ("supercritical water", "supercritical water: neither liquid nor gas", "a hot dense fluid with no surface between sea and sky"), "supercritical fluid", "stable",
     _ev("measured", [TC, T_HI], [PC, 1e12], "iapws1992-sat", note="above the critical point there is no phase change; the name changes by convention"), {}),
    ("h2o.ionic_fluid", ("ionic water", "water split into ions", "a hot fluid that conducts electricity, deep inside ice giants"), "ionic fluid", "stable",
     _ev("computed", [T_IONIC, T_HI], [P_IONIC, 1e12], "redmer2011", also=["mattsson2006"], note="a gradual crossover; the 2000 K, 20 GPa edge is recalled from these papers' figures and must be checked"),
     {"sigma_elec": {"v": 1e4, "unit": "S m-1", "src": "redmer2011", "check": "unverified", "note": "order of magnitude, recalled"}}),
    ("h2o.ice_ih", ("ice Ih", "ordinary ice", "ice"), "solid", "stable",
     _ev("measured", [72.0, 273.16], [0.0, 212.9 * MPA], "iapws2011-meltsub", also=["chaplin-lsbu"]),
     {"density": _d(916.7, "chaplin-lsbu", at_k=273.15, at_pa=101325.0, note="LSBU phase diagram page, 0.9167 g/cm3")}),
    ("h2o.ice_xi", ("ice XI", "ordered ice (below 72 K)", "the coldest form of ordinary ice"), "solid", "stable",
     _ev("measured", [T_LO, 72.0], [0.0, 70 * MPA], "chaplin-lsbu", note="hydrogen-ordered partner of ice Ih; forms very slowly"),
     {"density": _d(920.0, "chaplin-lsbu", note=LSBU_NOTE)}),
    ("h2o.ice_ii", ("ice II", "ice II", "dense ice"), "solid", "stable",
     _ev("measured", [T_LO, 248.85], [15 * MPA, 1050 * MPA], "chaplin-lsbu", note="hydrogen-ordered with no disordered partner; never in contact with liquid water"),
     {"density": _d(1170.0, "chaplin-lsbu", note=LSBU_NOTE)}),
    ("h2o.ice_iii", ("ice III", "ice III", "dense ice"), "solid", "stable",
     _ev("measured", [238.45, 256.164], [208.566 * MPA, 350.1 * MPA], "iapws2011-meltsub", also=["chaplin-lsbu", "journaux2020"]),
     {"density": _d(1140.0, "chaplin-lsbu", note=LSBU_NOTE),
      "compression": {"form": "journaux2020-mie-gruneisen", "v0_m3_mol": 15.49e-6, "k0_pa": 9.9 * GPA, "k0_prime": 6.0, "gamma0": 1.0, "q": 1.0, "src": "journaux2020", "note": "Table 3; V0, K0 and K0' at 1 bar and 0 K"}}),
    ("h2o.ice_v", ("ice V", "ice V", "dense ice"), "solid", "stable",
     _ev("measured", [218.0, 273.31], [344.3 * MPA, 632.4 * MPA], "iapws2011-meltsub", also=["chaplin-lsbu", "journaux2020"]),
     {"density": _d(1230.0, "chaplin-lsbu", note=LSBU_NOTE),
      "compression": {"form": "journaux2020-mie-gruneisen", "v0_m3_mol": 14.48e-6, "k0_pa": 13.2 * GPA, "k0_prime": 6.0, "gamma0": 1.1, "q": 1.0, "src": "journaux2020", "note": "Table 3; V0, K0 and K0' at 1 bar and 0 K"}}),
    ("h2o.ice_vi", ("ice VI", "ice VI", "the dense ice floor under an icy moon's ocean"), "solid", "stable",
     _ev("measured", [130.0, 355.0], [620 * MPA, 2216 * MPA], "iapws2011-meltsub", also=["chaplin-lsbu", "journaux2020"]),
     {"density": _d(1310.0, "chaplin-lsbu", note=LSBU_NOTE),
      "compression": {"form": "journaux2020-mie-gruneisen", "v0_m3_mol": 13.62e-6, "k0_pa": 15.2 * GPA, "k0_prime": 6.5, "gamma0": 1.4, "q": 1.0, "src": "journaux2020", "note": "Table 3; V0, K0 and K0' at 1 bar and 0 K"}}),
    ("h2o.ice_xv", ("ice XV", "ordered ice VI (below 130 K)", "very cold dense ice"), "solid", "stable",
     _ev("measured", [T_LO, 130.0], [800 * MPA, 1500 * MPA], "chaplin-lsbu", note="hydrogen-ordered partner of ice VI; the 130 K edge is approximate"),
     {"density": _d(1300.0, "chaplin-lsbu", note=LSBU_NOTE)}),
    ("h2o.ice_vii", ("ice VII", "ice VII: ice that stays solid when hot", "ice so squeezed it stays solid at hundreds of degrees"), "solid", "stable",
     _ev("measured", [100.0, T_A], [2100 * MPA, 62 * GPA], "iapws2011-meltsub", also=["frank2004", "lin2005"], note="found as inclusions in diamonds"),
     {"density": _d(1500.0, "chaplin-lsbu", note=LSBU_NOTE)}),
    ("h2o.ice_viii", ("ice VIII", "ordered ice VII (below about 278 K)", "cold squeezed ice"), "solid", "stable",
     _ev("measured", [T_LO, 278.0], [1500 * MPA, 62 * GPA], "chaplin-lsbu", note="hydrogen-ordered partner of ice VII"),
     {"density": _d(1460.0, "chaplin-lsbu", note=LSBU_NOTE + "; the LSBU phase-diagram page gives 1490")}),
    ("h2o.ice_x", ("ice X", "ice X: hydrogen shared evenly between oxygens", "ice crushed until its molecules lose their shape"), "solid", "stable",
     _ev("measured", [T_LO, 2000.0], [P_B, 1e12], "lin2005", also=["chaplin-lsbu"], note="above about 60 GPa; measured at room temperature and below, its high-temperature edge is computed"),
     {"density": _d(2510.0, "chaplin-lsbu", note=LSBU_NOTE)}),
    ("h2o.ice_xx", ("superionic ice (bcc, ice XX)", "superionic ice: solid oxygen, flowing hydrogen", "ice that is solid and liquid at once: the oxygen holds still while the hydrogen flows through it"), "superionic solid", "stable",
     _ev("computed", [T_A, T_D], [P_A, P_BCC_FCC], "cheng2021", also=["millot2019"], note="seen in diamond-anvil and shock experiments; its boundaries are computed and contested"),
     {"sigma_elec": {"v": 1e4, "unit": "S m-1", "src": "millot2019", "note": "simulations in Millot et al. 2019: ionic conductivity above 100 S/cm"}}),
    ("h2o.ice_xviii", ("superionic ice (fcc, ice XVIII)", "superionic ice: solid oxygen, flowing hydrogen", "the ice thought to fill the deep insides of Uranus and Neptune"), "superionic solid", "stable",
     _ev("measured", [2000.0, T_HI], [P_BCC_FCC, 1e12], "millot2019", also=["cheng2021", "millot2018"], note="seen for billionths of a second in laser-shock experiments at 100 to 400 GPa and 2000 to 3000 K; above 400 GPa an hcp form is computed to take over (Cheng et al. 2021), not drawn here"),
     {"sigma_elec": {"v": 1e4, "unit": "S m-1", "src": "millot2019", "note": "simulations in Millot et al. 2019: ionic conductivity above 100 S/cm"}}),
]

# Metastable forms: they exist, but are not what forms on a slow path; listed rows, off the map unless asked.
METASTABLE = [
    ("h2o.ice_ic", "ice Ic", "cubic ice", 930.0, "made by vapour deposition or from amorphous ice between about 130 and 220 K; turns to Ih near 240 K", None),
    ("h2o.ice_iv", "ice IV", "ice IV", 1270.0, "metastable inside ice V's field; made from high-density amorphous ice near 810 MPa", None),
    ("h2o.ice_ix", "ice IX", "ordered ice III", 1160.0, "hydrogen-ordered ice III, made by cooling ice III below about 165 K; metastable in ice II's field", None),
    ("h2o.ice_xii", "ice XII", "ice XII", 1290.0, "metastable inside ice V's field, from liquid near 260 K and 0.55 GPa", None),
    ("h2o.ice_xiii", "ice XIII", "ordered ice V", 1230.0, "hydrogen-ordered ice V, needs acid doping", None),
    ("h2o.ice_xiv", "ice XIV", "ordered ice XII", 1290.0, "hydrogen-ordered ice XII, needs acid doping", None),
    ("h2o.ice_xvi", "ice XVI", "empty clathrate ice", 810.0, "made by pumping neon out of a clathrate at 142 K; the least dense crystalline ice", "salzmann2019"),
    ("h2o.ice_xvii", "ice XVII", "empty hydrogen-clathrate ice", 850.0, "made by pumping hydrogen out of a filled ice below 120 K", "salzmann2019"),
    ("h2o.ice_xix", "ice XIX", "another ordered ice VI", 1320.0, "a second hydrogen-ordered form of ice VI, from acid-doped ice VI cooled at about 2 GPa", None),
    ("h2o.amorphous_lda", "low-density amorphous ice", "glassy ice", 925.0, "a glass made by vapour deposition below about 120 K; the frost on very cold comets and outer-system grains; turns crystalline above about 130 to 136 K", None),
    ("h2o.amorphous_hda", "high-density amorphous ice", "dense glassy ice", 1170.0, "made by squeezing ice Ih at 77 K (about 1.6 GPa)", "salzmann2019"),
    ("h2o.amorphous_vhda", "very-high-density amorphous ice", "densest glassy ice", 1250.0, "made by heating high-density amorphous ice to about 160 K at 1 to 2 GPa (1.26 at 77 K in another table)", None),
]


def forms():
    out = []
    for fid, (ph, pl, st), kind, stab, ev, props in FORMS:
        out.append({"id": fid, "names": {"physics": ph, "plain": pl, "story": st}, "kind": kind, "stability": stab,
                    "evidence": ev, "props": props})
    for fid, ph, pl, rho, how, src in METASTABLE:
        kind = "glass" if "amorphous" in fid else "solid"
        out.append({"id": fid, "names": {"physics": ph, "plain": pl, "story": pl}, "kind": kind, "stability": "metastable",
                    "evidence": {"grade": "measured", "src": src or "chaplin-lsbu", "also": ["wikipedia-ice-phases"] if src is None else ["chaplin-lsbu"], "note": how},
                    "props": {"density": _d(rho, "chaplin-lsbu", note=LSBU_NOTE)}})
    return out


# ---------------------------------------------------------------- boundaries
def _pts(*pairs):
    return [[t, p] for t, p in pairs]


# ---------------------------------------------------------------- curves as the kernel reads them (MP17)
# The kernel checks two forms against the table: iapws_melt (pi = 1 + sum a (1 - th^b), or
# ln pi = sum a (1 - th^b) when log) and points ([[p_pa, t_k], ...]). Every other curve is written as points
# sampled from its equation, with the equation kept beside it under "fit".
N_SAMPLES = 24


def _curve_fn(c):
    f = c["form"]
    if f == "iapws2011_subl":
        return lambda T: f_subl(T, c["coeffs"])
    if f == "iapws1992_sat":
        return lambda T: f_sat(T, c["coeffs"])
    if f == "iapws2011_ih":
        return lambda T: f_melt_ih(T, c["coeffs"])
    if f == "iapws2011_simple":
        return lambda T: f_simple(T, c["coeffs"])
    if f == "iapws2011_vii":
        return lambda T: f_melt_vii(T, c["coeffs"])
    if f == "simon":
        return lambda T: f_simon(T, c["coeffs"])
    raise ValueError(f)


def _g(x):
    return float("%.9g" % x)


def _sample(c):
    if c["form"] == "points":
        return [[_g(p), _g(t)] for t, p in c["points"]]
    if c["form"] == "isotherm":
        return [[_g(c["p_pa"][0]), c["t_k"]], [_g(c["p_pa"][1]), c["t_k"]]]
    if c["form"] == "isobar":
        return [[_g(c["p_pa"]), _g(c["t_k"][0])], [_g(c["p_pa"]), _g(c["t_k"][1])]]
    if c["form"] == "segments":
        out = []
        for part in c["parts"]:
            pts = _sample(part)
            out += pts if not out else pts[1:]
        return out
    fn = _curve_fn(c)
    lo, hi = c["t_k"]
    # log spacing in T, so a curve spanning 10 K to 273 K is sampled evenly on the diagram
    return [[_g(fn(T)), _g(T)] for T in (lo * (hi / lo) ** (k / (N_SAMPLES - 1)) for k in range(N_SAMPLES))]


def kernel_curve(c):
    f = c["form"]
    if f == "iapws2011_ih":
        tt, pt, a1, a2, a3, b1, b2, b3 = c["coeffs"]
        return {"form": "iapws_melt", "tn": tt, "pn": pt, "log": False, "terms": [[a1, b1], [a2, b2], [a3, b3]], "valid_k": c["t_k"]}
    if f == "iapws2011_simple":
        tt, pt, a, b = c["coeffs"]
        return {"form": "iapws_melt", "tn": tt, "pn": pt, "log": False, "terms": [[-a, b]], "valid_k": c["t_k"]}
    if f == "iapws2011_vii":
        tt, pt, a1, a2, a3 = c["coeffs"]
        return {"form": "iapws_melt", "tn": tt, "pn": pt, "log": True, "terms": [[a1, -1.0], [a2, 5.0], [a3, 22.0]], "valid_k": c["t_k"]}
    if f == "simon":
        p0, a, t0, e = c["coeffs"]
        # p = p0 + A ((T/T0)^c - 1)  is  pi = 1 + (-A/p0)(1 - th^c) with pn = p0, tn = T0
        return {"form": "iapws_melt", "tn": t0, "pn": p0, "log": False, "terms": [[-a / p0, e]], "valid_k": c["t_k"],
                "note": "Simon fit p = p0 + A((T/T0)^c - 1) written in the iapws_melt form"}
    return {"form": "points", "points": _sample(c)}


def boundaries():
    s_vap_xi = f_subl(72.0)
    B = []

    def add(a, b, curve, grade, src, **extra):
        row = {"between": sorted([a, b]), "curve": kernel_curve(curve), "evidence": {"grade": grade}, "src": src}
        if curve["form"] not in ("points", "iapws2011_ih", "iapws2011_simple", "iapws2011_vii", "simon"):
            row["fit"] = curve  # the equation itself; the curve above samples it
        row.update(extra)
        B.append(row)

    add("h2o.ice_ih", "h2o.vapour", {"form": "iapws2011_subl", "coeffs": SUBL, "t_k": [72.0, 273.16]}, "measured", "iapws2011-meltsub",
        note="the release holds it from 50 K; the data reach 130 K and below that it rests on Feistel and Wagner 2007's model")
    add("h2o.ice_xi", "h2o.vapour", {"form": "iapws2011_subl", "coeffs": SUBL, "t_k": [T_LO, 72.0]}, "extrapolated", "iapws2011-meltsub",
        note="ice Ih's sublimation curve carried below 50 K and onto ice XI; the vapour pressure there is below 1e-15 Pa")
    add("h2o.liquid", "h2o.vapour", {"form": "iapws1992_sat", "coeffs": SAT, "t_k": [273.16, TC]}, "measured", "iapws1992-sat")
    add("h2o.ice_ih", "h2o.liquid", {"form": "iapws2011_ih", "coeffs": MELT_IH, "t_k": [251.165, 273.16]}, "measured", "iapws2011-meltsub", uncertainty="2% in p (k=2)")
    add("h2o.ice_iii", "h2o.liquid", {"form": "iapws2011_simple", "coeffs": MELT_III, "t_k": [251.165, 256.164]}, "measured", "iapws2011-meltsub", uncertainty="3% in p (k=2)")
    add("h2o.ice_v", "h2o.liquid", {"form": "iapws2011_simple", "coeffs": MELT_V, "t_k": [256.164, 273.31]}, "measured", "iapws2011-meltsub", uncertainty="3% in p (k=2)")
    add("h2o.ice_vi", "h2o.liquid", {"form": "iapws2011_simple", "coeffs": MELT_VI, "t_k": [273.31, 355.0]}, "measured", "iapws2011-meltsub", uncertainty="3% in p (k=2)")
    add("h2o.ice_vii", "h2o.liquid", {"form": "segments", "parts": [
        {"form": "iapws2011_vii", "coeffs": MELT_VII, "t_k": [355.0, T_VII_SWITCH], "src": "iapws2011-meltsub"},
        {"form": "simon", "coeffs": SIMON_VII, "t_k": [T_VII_SWITCH, TC], "src": "frank2004"}]},
        "measured", "iapws2011-meltsub", also=["frank2004"], uncertainty="7% in p (k=2) for the IAPWS part",
        note="IAPWS to where it meets Frank et al.'s Simon fit near 590 K (they agree there to 0.1%); measurements disagree by up to about 80 K at 13 GPa (Lin et al. 2004)")
    add("h2o.ice_vii", "h2o.supercritical", {"form": "simon", "coeffs": SIMON_VII, "t_k": [TC, T_A]}, "measured", "frank2004", note="Frank et al.'s fit holds 3 to 60 GPa")
    add("h2o.ice_xx", "h2o.supercritical", {"form": "points", "points": [p for p in SI_MELT if p[0] <= T_IONIC] + [[T_IONIC, P_E]], "t_k": [T_A, T_IONIC]}, "computed", "cheng2021", also=["lin2005"])
    add("h2o.ice_xx", "h2o.ionic_fluid", {"form": "points", "points": [[T_IONIC, P_E], [T_D, P_BCC_FCC]], "t_k": [T_IONIC, T_D]}, "computed", "cheng2021")
    add("h2o.ice_xviii", "h2o.ionic_fluid", {"form": "points", "points": [[T_D, P_BCC_FCC]] + [p for p in SI_MELT if p[1] > P_BCC_FCC], "t_k": [T_D, SI_MELT[-1][0]]}, "computed", "cheng2021",
        also=["millot2018"], check="unverified", note="3250 K at 100 GPa is Cheng et al.'s; 5000 K at 190 GPa is recalled from Millot et al. 2018 and must be checked; beyond 190 GPa extrapolated")
    add("h2o.liquid", "h2o.supercritical", {"form": "isotherm", "t_k": TC, "p_pa": [PC, f_simon(TC)]}, "convention", "iapws1992-sat")
    add("h2o.supercritical", "h2o.vapour", {"form": "isobar", "p_pa": PC, "t_k": [TC, T_HI]}, "convention", "iapws1992-sat")
    add("h2o.ionic_fluid", "h2o.supercritical", {"form": "segments", "parts": [
        {"form": "isotherm", "t_k": T_IONIC, "p_pa": [P_E, P_IONIC]},
        {"form": "isobar", "p_pa": P_IONIC, "t_k": [T_IONIC, T_HI]}]}, "computed", "redmer2011", also=["mattsson2006"], check="unverified",
        note="a smooth crossover, not a phase change; the edge is recalled and must be checked")
    add("h2o.ice_ih", "h2o.ice_xi", {"form": "points", "points": [[72.0, s_vap_xi], [TP["ih_ii_xi"][0], TP["ih_ii_xi"][1]]]}, "measured", "chaplin-lsbu",
        note="gas-Ih-XI at about 72 K (LSBU 72.15 K); Ih-II-XI given as 73.4 K, 70 MPa or 75.05 K, 89.6 MPa")
    add("h2o.ice_ih", "h2o.ice_ii", {"form": "points", "points": _pts(TP["ih_ii_xi"], TP["ih_ii_iii"])}, "measured", "chaplin-lsbu", note="straight between the two triple points")
    add("h2o.ice_ii", "h2o.ice_xi", {"form": "points", "points": _pts((T_LO, p_ih_ii(T_LO)), TP["ih_ii_xi"])}, "extrapolated", "chaplin-lsbu", note="the Ih-II line carried below 73.4 K")
    add("h2o.ice_ih", "h2o.ice_iii", {"form": "points", "points": _pts(TP["ih_ii_iii"], (251.165, 208.566 * MPA))}, "measured", "chaplin-lsbu")
    add("h2o.ice_ii", "h2o.ice_iii", {"form": "points", "points": _pts(TP["ih_ii_iii"], TP["ii_iii_v"])}, "measured", "chaplin-lsbu")
    add("h2o.ice_iii", "h2o.ice_v", {"form": "points", "points": _pts(TP["ii_iii_v"], (256.164, 350.1 * MPA))}, "measured", "chaplin-lsbu")
    add("h2o.ice_ii", "h2o.ice_v", {"form": "points", "points": _pts(TP["ii_v_vi"], TP["ii_iii_v"])}, "measured", "chaplin-lsbu",
        note="II-V-VI is given as about 218 K, 620 MPa (one study 201.9 K, 670.8 MPa)")
    add("h2o.ice_v", "h2o.ice_vi", {"form": "points", "points": _pts(TP["ii_v_vi"], (273.31, 632.4 * MPA))}, "measured", "chaplin-lsbu")
    add("h2o.ice_ii", "h2o.ice_vi", {"form": "points", "points": _pts(TP["ii_vi_xv"], TP["ii_v_vi"])}, "measured", "chaplin-lsbu")
    add("h2o.ice_ii", "h2o.ice_xv", {"form": "points", "points": _pts((T_LO, p_ii_vi(T_LO)), TP["ii_vi_xv"])}, "extrapolated", "chaplin-lsbu", note="the II-VI line carried below 130 K")
    add("h2o.ice_vi", "h2o.ice_xv", {"form": "isotherm", "t_k": 130.0, "p_pa": [800 * MPA, 1500 * MPA]}, "measured", "chaplin-lsbu", note="about 130 K; approximate")
    add("h2o.ice_vi", "h2o.ice_viii", {"form": "points", "points": _pts(TP["vi_viii_xv"], TP["vi_vii_viii"])}, "measured", "chaplin-lsbu")
    add("h2o.ice_viii", "h2o.ice_xv", {"form": "isobar", "p_pa": 1500 * MPA, "t_k": [T_LO, 130.0]}, "extrapolated", "chaplin-lsbu")
    add("h2o.ice_vi", "h2o.ice_vii", {"form": "points", "points": _pts(TP["vi_vii_viii"], (355.0, 2216 * MPA))}, "measured", "chaplin-lsbu",
        also=["haldemann2020"], note="AQUA's fit (Haldemann et al. 2020, Eq. 22) runs about 30 K warmer at 2.1 GPa; the triple points are kept")
    add("h2o.ice_vii", "h2o.ice_viii", {"form": "points", "points": _pts(TP["vi_vii_viii"], TP["vii_viii_x"])}, "measured", "chaplin-lsbu",
        check="unverified", note="straight in T and log p between the two triple points; the real line is flatter near 270 K at moderate pressure")
    add("h2o.ice_viii", "h2o.ice_x", {"form": "isobar", "p_pa": 62 * GPA, "t_k": [T_LO, 100.0]}, "extrapolated", "chaplin-lsbu")
    add("h2o.ice_vii", "h2o.ice_x", {"form": "points", "points": [[100.0, 62 * GPA], [300.0, 60 * GPA], [T_B, P_B]]}, "measured", "lin2005",
        note="60 GPa at 300 K (Goncharov et al. 1999, quoted by Lin et al. 2005), then the negative slope Lin et al. propose, until the superionic onset")
    add("h2o.ice_vii", "h2o.ice_xx", {"form": "points", "points": [[T_A, P_A], [T_B, P_B]]}, "computed", "cheng2021", also=["lin2005"])
    add("h2o.ice_x", "h2o.ice_xx", {"form": "points", "points": [[T_B, P_B], [T_C, P_BCC_FCC]]}, "computed", "cheng2021")
    add("h2o.ice_x", "h2o.ice_xviii", {"form": "points", "points": [[T_C, P_BCC_FCC], [2000.0, 100 * GPA], [2000.0, 1e12]]}, "computed", "cheng2021", also=["millot2019"])
    add("h2o.ice_xviii", "h2o.ice_xx", {"form": "isobar", "p_pa": P_BCC_FCC, "t_k": [T_C, T_D]}, "computed", "cheng2021",
        note="Cheng et al.: bcc below about 75 GPa; Millot et al. 2019 saw bcc turn fcc on compression and heating")
    return B


def meets():
    s72 = f_subl(72.0)
    M = [
        (["h2o.ice_ih", "h2o.liquid", "h2o.vapour"], 273.16, 611.657, "iapws2011-meltsub"),
        (["h2o.ice_ih", "h2o.ice_iii", "h2o.liquid"], 251.165, 208.566 * MPA, "iapws2011-meltsub"),
        (["h2o.ice_iii", "h2o.ice_v", "h2o.liquid"], 256.164, 350.1 * MPA, "iapws2011-meltsub"),
        (["h2o.ice_v", "h2o.ice_vi", "h2o.liquid"], 273.31, 632.4 * MPA, "iapws2011-meltsub"),
        (["h2o.ice_vi", "h2o.ice_vii", "h2o.liquid"], 355.0, 2216 * MPA, "iapws2011-meltsub"),
        (["h2o.ice_ih", "h2o.ice_ii", "h2o.ice_iii"], *TP["ih_ii_iii"], "chaplin-lsbu"),
        (["h2o.ice_ii", "h2o.ice_iii", "h2o.ice_v"], *TP["ii_iii_v"], "chaplin-lsbu"),
        (["h2o.ice_ii", "h2o.ice_v", "h2o.ice_vi"], *TP["ii_v_vi"], "chaplin-lsbu"),
        (["h2o.ice_ii", "h2o.ice_vi", "h2o.ice_xv"], *TP["ii_vi_xv"], "chaplin-lsbu"),
        (["h2o.ice_vi", "h2o.ice_viii", "h2o.ice_xv"], *TP["vi_viii_xv"], "chaplin-lsbu"),
        (["h2o.ice_vi", "h2o.ice_vii", "h2o.ice_viii"], *TP["vi_vii_viii"], "chaplin-lsbu"),
        (["h2o.ice_vii", "h2o.ice_viii", "h2o.ice_x"], *TP["vii_viii_x"], "chaplin-lsbu"),
        (["h2o.ice_ih", "h2o.ice_ii", "h2o.ice_xi"], *TP["ih_ii_xi"], "chaplin-lsbu"),
        (["h2o.ice_ih", "h2o.ice_xi", "h2o.vapour"], 72.0, s72, "chaplin-lsbu"),
        (["h2o.ice_vii", "h2o.ice_xx", "h2o.supercritical"], T_A, P_A, "lin2005"),
        (["h2o.ice_vii", "h2o.ice_x", "h2o.ice_xx"], T_B, P_B, "cheng2021"),
        (["h2o.ice_x", "h2o.ice_xviii", "h2o.ice_xx"], T_C, P_BCC_FCC, "cheng2021"),
        (["h2o.ice_xviii", "h2o.ice_xx", "h2o.ionic_fluid"], T_D, P_BCC_FCC, "cheng2021"),
        (["h2o.ice_xx", "h2o.ionic_fluid", "h2o.supercritical"], T_IONIC, P_E, "redmer2011"),
        (["h2o.liquid", "h2o.supercritical", "h2o.vapour"], TC, PC, "iapws1992-sat"),
        (["h2o.ice_vii", "h2o.liquid", "h2o.supercritical"], TC, f_simon(TC), "frank2004"),
    ]
    notes = {
        "h2o.ice_vii h2o.ice_xx h2o.supercritical": "contested: placed from 14.6 GPa, 850 K to about 47 GPa, 1000 K by different studies; here at Lin et al. 2005's 35 GPa, on Frank et al.'s melting curve",
        "h2o.liquid h2o.supercritical h2o.vapour": "the critical point: the end of the boiling curve, where the convention lines start",
        "h2o.ice_vii h2o.liquid h2o.supercritical": "not a true triple point: where ice VII's melting curve crosses the critical temperature",
    }
    out = []
    for ph, t, p, src in M:
        row = {"phases": sorted(ph), "t_k": round(t, 6), "p_pa": float("%.9g" % p), "src": src}
        n = notes.get(" ".join(sorted(ph)))
        if n:
            row["note"] = n
        out.append(row)
    return out


# ---------------------------------------------------------------- the lookup table
TABLE_LOGP = (-10.0, 12.0, 4401)  # 0.005 dex
TABLE_LOGT = (1.0, 4.0, 1501)     # 0.002 dex (0.46% in T)
DIGITS = "0123456789abcdefghijklmnopqrstuvwxyz"


def b36(n):
    out = ""
    while True:
        out = DIGITS[n % 36] + out
        n //= 36
        if not n:
            return out


def table():
    """The phase_at lookup: a grid in log10 p by log10 T, run-length encoded by row.

    rle: rows (one per pressure, low first) separated by "/"; each row is runs separated by spaces; a run is
    the phase's base-36 digit (index into phases) followed by its length in base 36. Temperature runs low to
    high within a row. Decoded, row i holds n_t cells and cell (i, j) is phases[digit].
    """
    ids = [f[0] for f in FORMS]
    index = {fid: i for i, fid in enumerate(ids)}
    lp0, lp1, n_p = TABLE_LOGP
    lt0, lt1, n_t = TABLE_LOGT
    temps = [10 ** (lt0 + (lt1 - lt0) * j / (n_t - 1)) for j in range(n_t)]
    rows = []
    for i in range(n_p):
        p = 10 ** (lp0 + (lp1 - lp0) * i / (n_p - 1))
        runs, prev, n = [], None, 0
        for T in temps:
            c = index[phase_of(T, p)]
            if c == prev:
                n += 1
            else:
                if prev is not None:
                    runs.append(DIGITS[prev] + b36(n))
                prev, n = c, 1
        runs.append(DIGITS[prev] + b36(n))
        rows.append(" ".join(runs))
    return {"log10_p_pa": {"lo": lp0, "hi": lp1, "n": n_p}, "log10_t_k": {"lo": lt0, "hi": lt1, "n": n_t},
            "phases": ids, "order": "pressure outer (low first), temperature inner (low first)",
            "encoding": "rle: rows split by '/', runs by ' '; run = phase digit (base 36) then length (base 36)",
            "built_by": "scripts/water_phases.py", "rle": "/".join(rows)}


def decode(t):
    """The table's cells as a list of rows of phase ids (for checks and tests)."""
    ids = t["phases"]
    out = []
    for row in t["rle"].split("/"):
        cells = []
        for run in row.split(" "):
            cells += [ids[int(run[0], 36)]] * int(run[1:], 36)
        out.append(cells)
    return out


SOURCES = {
    "iapws2011-meltsub": ("IAPWS R14-08(2011), Revised Release on the Pressure along the Melting and Sublimation Curves of Ordinary Water Substance (Wagner, Riethmann, Feistel and Harvey 2011, J. Phys. Chem. Ref. Data 40, 043103). Coefficients and the six check values read from the release; all six reproduce", "10.1063/1.3657937", "https://iapws.org/documents/release/MeltSub", "verified"),
    "iapws1992-sat": ("IAPWS SR1-86(1992), Revised Supplementary Release on Saturation Properties of Ordinary Water Substance (Wagner and Pruss 1993, J. Phys. Chem. Ref. Data 22, 783). Coefficients read from the NIST reprint; the check values at 273.16 K and 373.1243 K reproduce", "10.1063/1.555926", "https://srd.nist.gov/jpcrdreprint/1.555926.pdf", "verified"),
    "frank2004": ("Frank, M. R., Fei, Y. and Hu, J. 2004, Geochim. Cosmochim. Acta 68, 2781: constraining the equation of state of fluid H2O to 80 GPa using the melting curve, bulk modulus and thermal expansivity of ice VII. Simon fit (P - 2.17)/0.764 = (T/355)^4.32 - 1, P in GPa, 3 to 60 GPa, read from the paper; its Table 2 points reproduce within their errors", None, "https://www.cfa.harvard.edu/~lzeng/papers/frank_fei_hu_2004.pdf", "verified"),
    "lin2005": ("Lin, J.-F., Gregoryanz, E., Struzhkin, V. V., Somayazulu, M., Mao, H.-k. and Hemley, R. J. 2005, Geophys. Res. Lett. 32, L11306: melting behavior of H2O at high pressures and temperatures (a break in the melting curve at about 35 GPa and 1040 K; ice VII to X from 60 GPa at 300 K, quoting Goncharov et al. 1999)", "10.1029/2005GL022499", None, "verified"),
    "journaux2020": ("Journaux, B. et al. 2020, J. Geophys. Res. Planets 125, e2019JE006176: holistic approach for studying planetary hydrospheres, Gibbs representation of ices thermodynamics, elasticity and the water phase diagram to 2,300 MPa (SeaFreeze). Table 3 (ices III, V, VI) read from the arXiv version, 1907.09598", "10.1029/2019JE006176", "https://arxiv.org/abs/1907.09598", "verified"),
    "chaplin-lsbu": ("Chaplin, M., Water Structure and Science (London South Bank University): the water phase diagram page (triple points, with its references) and the ice phases page (structural table: densities, ordering, space groups). A curated secondary source; each triple point cites a primary paper there", None, "https://water.lsbu.ac.uk/water/water_phase_diagram.html", "verified"),
    "wikipedia-ice-phases": ("Wikipedia, Phases of ice (read 22 Nov 2026): table of phases with formation conditions; used only where it agrees with the LSBU tables", None, "https://en.wikipedia.org/wiki/Phases_of_ice", "verified"),
    "salzmann2019": ("Salzmann, C. G. 2019, J. Chem. Phys. 150, 060901: advances in the experimental exploration of water's phase diagram (ices XVI and XVII, amorphous ices, hydrogen order)", "10.1063/1.5085163", None, "verified"),
    "millot2019": ("Millot, M., Coppari, F., Rygg, J. R., Correa Barrios, A., Hamel, S., Swift, D. C. and Eggert, J. H. 2019, Nature 569, 251: nanosecond X-ray diffraction of shock-compressed superionic water ice (fcc superionic ice XVIII at 100 to 400 GPa and 2000 to 3000 K; simulated ionic conductivity above 100 S/cm). Abstract read; body paywalled", "10.1038/s41586-019-1114-6", None, "verified"),
    "millot2018": ("Millot, M. et al. 2018, Nature Physics 14, 297: experimental evidence for superionic water ice using shock compression. Not read; its melting point near 5000 K at 190 GPa is recalled", "10.1038/s41567-017-0017-4", None, "unverified"),
    "cheng2021": ("Cheng, B., Bethkenhagen, M., Pickard, C. J. and Hamel, S. 2021, arXiv:2103.09035 (Nature Physics 17, 1228): predicting the phase behaviors of superionic water at planetary conditions (bcc stable below about 75 GPa and 2500 K; superionic onset near 2000 K; melting 3250 K at 100 GPa). Text read; boundaries only in figures", None, "https://arxiv.org/abs/2103.09035", "verified"),
    "haldemann2020": ("Haldemann, J., Alibert, Y., Mordasini, C. and Benz, W. 2020, A&A 643, A105: AQUA, a collection of H2O equations of state for planetary models (the ice VI-VII fit, Eq. 22 and Table 2, read from arXiv 2009.10098)", None, "https://arxiv.org/abs/2009.10098", "verified"),
    "redmer2011": ("Redmer, R., Mattsson, T. R., Nettelmann, N. and French, M. 2011, Icarus 211, 798: the phase diagram of water and the magnetic fields of Uranus and Neptune. Not read; the ionic-fluid edge and conductivity are recalled from it", "10.1016/j.icarus.2010.08.008", None, "unverified"),
    "mattsson2006": ("Mattsson, T. R. and Desjarlais, M. P. 2006, Phys. Rev. Lett. 97, 017801: phase diagram and electrical conductivity of high energy-density water from density functional theory. Not read; recalled", "10.1103/PhysRevLett.97.017801", None, "unverified"),
    "recalled": ("A value recalled from textbooks and not yet read from a source; listed so the citations track can find it", None, None, "unverified"),
}


def build():
    return {"forms": forms(), "boundaries": boundaries(), "meets": meets(), "table": table()}


if __name__ == "__main__":
    # ASCII map for a quick look: rows are pressure (high at top), columns temperature.
    import sys
    ids = [f[0] for f in FORMS]
    short = {fid: DIGITS[i] for i, fid in enumerate(ids)}
    for lp in [x / 2 for x in range(24, -21, -1)]:
        row = "".join(short[phase_of(10 ** (1 + 3 * j / 99), 10 ** lp)] for j in range(100))
        sys.stdout.write("%6.1f %s\n" % (lp, row))
    sys.stdout.write("       T from 10 K to 10000 K, log; key: " + " ".join("%s=%s" % (short[i], i[4:]) for i in ids) + "\n")
    sys.stdout.write("switch %.2f K  A %.1f K %.3g Pa  B %.1f K %.3g Pa  C %.1f K  D %.1f K  E %.3g Pa\n" % (T_VII_SWITCH, T_A, P_A, T_B, P_B, T_C, T_D, P_E))
