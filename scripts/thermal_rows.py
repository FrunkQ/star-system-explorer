"""Heat, melt and boil values for hull skins and hot-structure metals on the
master data sheet (package MP27, for track RK's RK12 burn-through block: the
energy to heat, melt and boil a skin, plan section 13).

Each row gets, per kilogram:
  heat_capacity        Cp at 298.15 K (J kg-1 K-1). Cp climbs with
                       temperature (tungsten's doubles by its melting point),
                       so a burn-through sum should prefer the two integrated
                       heats below.
  heat_to_melt         H(solid at Tm) - H(298.15 K): the sensible heat from room
                       temperature to the start of melting, through any solid
                       phase changes (J kg-1). This is the integral of c dT.
  latent_fusion        the step in H at Tm (J kg-1).
  heat_to_boil         H(liquid at Tb) - H(liquid at Tm): heating the melt to
                       the boiling point (J kg-1).
  boiling_point_k      where the vapour's fugacity reaches 1 bar.
  latent_vaporisation  the step in H at Tb (J kg-1).
Graphite does not melt at 1 bar; it gets latent_sublimation to carbon atoms.

Every molar value is read from the NIST-JANAF tables (Chase 1998) as printed,
in kJ/mol or J/K/mol, and turned into per-kilogram values here with the
element's IUPAC atomic weight from the sheet's elements table, so no
per-kilogram number is typed. The latent heats are differences of the printed
H - H(298.15) column between the two rows at a transition; JANAF prints no
delta-H column. Read 24 Nov 2026 from the fetched text, so all are unverified
until checked against the tables themselves. Densities are from NIST SRD 126
Table 1.
"""

SOURCES = {
    "nist-janaf": ("Chase, M. W. 1998, NIST-JANAF Thermochemical Tables, 4th edition, J. Phys. Chem. Ref. Data Monograph 9: reference-state tables (ref) for Al, Ti, Fe, Nb, Mo, Ta, W, Pb, C (graphite) and the gas tables W(g) W-005, C(g) C-003, C2(g) C-113, C3(g) C-138, C4(g) C-143, C5(g) C-148; Cp and H - H(298.15 K) at 298.15 K and at each transition row", None, "https://janaf.nist.gov/", "unverified"),
    "kim1975": ("Kim, C. S. 1975, Thermophysical properties of stainless steels, ANL-75-55, Argonne National Laboratory, section II: type 304L melting range 1670-1730 K (1700 K used), heat of fusion 64.0 cal/g, solid enthalpy H - H298 = -34.885 + 0.1122 T + 1.611e-5 T^2 cal/g, liquid H - H298 = 0.1900 T - 56.588 cal/g, solid cp = 0.1122 + 3.222e-5 T cal/g K, boiling point 3080 K, heat of vaporisation 1770 cal/g, density 7.894 g/cm3 at 300 K (Table 7)", None, "https://inldigitallibrary.inl.gov/Reports/ANL-75-55.pdf", "unverified"),
    "ati-304": ("ATI 2014, ATI 302/304/304L/305 technical data sheet (v1, 17 Feb 2014), page 4: melting range 2550-2590 F (1399-1421 C), specific heat 0.12 Btu/lb F (500 J/kg K) over 0-100 C, density 0.285 lb/in3 (7.90 g/cm3); typical values", None, "https://www.atimaterials.com/Products/Documents/datasheets/stainless-specialty-steel/austenitic/ati_302_304_304l_305_tds_en1_v1.pdf", "unverified"),
    "cezairliyan1977": ("Cezairliyan, A., McClure, J. L. and Taylor, R. 1977, J. Res. Natl. Bur. Stand. 81A, 251: thermophysical measurements on 90Ti-6Al-4V alloy above 1450 K; melting point 1943 K; cp = 1.3833 - 9.943e-4 T + 3.745e-7 T^2 J/g K over 1450-1900 K", None, "https://nvlpubs.nist.gov/nistpubs/jres/81A/jresv81An2-3p251_A1b.pdf", "unverified"),
    "nist-trc-cryo": ("NIST Cryogenic Technologies Group, Material properties pages: specific heat fits log10 cp = sum a_k (log10 T)^k, J/kg K, 4-300 K, 5 % fit error; 6061-T6 aluminium (a..i = 46.6467, -314.292, 866.662, -1298.3, 1162.27, -637.795, 210.351, -38.3094, 2.96344)", None, "https://trc.nist.gov/cryogenics/materials/6061%20Aluminum/6061_T6Aluminum_rev.htm", "unverified"),
    "plante-szwarc1966": ("Plante, E. R. and Szwarc, R. 1966, J. Res. Natl. Bur. Stand. 70A, 175: vapor pressure and heat of sublimation of rhenium (Langmuir, 2350-3050 K); third-law heat of sublimation at 298 K 185.9 kcal/mol", None, "https://nvlpubs.nist.gov/nistpubs/jres/070/2/V70.N02.A07.pdf", "unverified"),
    "nist-srd126-t1": ("Hubbell, J. H. and Seltzer, S. M., X-ray mass attenuation coefficients, NIST Standard Reference Database 126, Table 1 (elemental media): densities C (graphite) 1.700, Al 2.699, Ti 4.540, Fe 7.874, Nb 8.570, Mo 10.22, Ta 16.65, Re 21.02 g/cm3", None, "https://physics.nist.gov/PhysRefData/XrayMassCoef/tab1.html", "unverified"),
}

# element: (JANAF table, Cp at 298.15 J/K/mol, Tm K, H solid at Tm, H liquid at Tm,
#           Tb K or None, H liquid at Tb, H gas at Tb), H in kJ/mol above 298.15 K.
JANAF = {
    "al": ("Al-001", 24.209, 933.450, 17.982, 28.693, 2790.812, 87.665, 381.667),
    "ti": ("Ti-001", 25.238, 1939.000, 55.394, 69.540, 3630.956, 149.463, 559.447),
    "fe": ("Fe-003", 25.094, 1809.000, 58.645, 72.452, 3133.345, 133.404, 482.989),
    "nb": ("Nb-001", 24.694, 2750.000, 77.011, 103.914, 5130.824, 183.605, 874.064),
    "mo": ("Mo-001", 23.933, 2896.000, 89.885, 125.867, 4951.969, 203.287, 793.918),
    "ta": ("Ta-001", 25.295, 3258.000, 93.564, 130.132, 5778.070, 235.572, 974.285),
    "w": ("W-001", 24.295, 3680.000, 118.383, 153.780, None, None, None),
    "pb": ("Pb-001", 26.836, 600.600, 8.499, 13.273, 2019.022, 54.758, 232.340),
}
SOLID_STEPS = {
    "ti": "includes the alpha to beta change at 1166 K (4.172 kJ/mol)",
    "fe": "includes the magnetic change near 1042 K and the alpha to gamma (1184 K, 0.900 kJ/mol) and gamma to delta (1665 K, 0.837 kJ/mol) changes",
}
# The sheet rows each element's values go on.
ROW_OF = {"al": "aluminium", "ti": "titanium", "fe": "iron", "nb": "niobium",
          "mo": "molybdenum", "ta": "tantalum", "w": "tungsten", "pb": "lead"}
# Graphite: Cp from C-002 (graphite, ref); C(g) table C-003 gives delta-f H at
# 298.15 K, which is the heat to turn graphite into carbon atoms.
GRAPHITE_CP, C_GAS_DFH = 8.517, 716.670
DENSITY_G_CM3 = {"graphite": 1.700, "aluminium": 2.699, "titanium": 4.540, "iron": 7.874, "rhenium": 21.02}
T_ROOM = 298.15
CAL_J = 4.184  # thermochemical calorie, exact by definition

# W(g), table W-005: (T, delta-f H kJ/mol, log Kf) for its last two rows; above
# 3680 K the reference is liquid tungsten, so delta-f H is the heat of
# vaporisation and log Kf the log of the vapour pressure in bar. The table ends
# at 6000 K, below the boiling point, so Tb comes from a + b/T through these.
W_GAS = ((5900.0, 837.175, -0.310), (6000.0, 838.834, -0.186))
W_LIQ_CP, W_LIQ_H_6000 = 35.564, 236.289  # W-001 at 6000 K, J/K/mol and kJ/mol
# Carbon vapour over graphite: log Kf of Cn(g) is log10 of its pressure in bar;
# delta-f H in kJ/mol of Cn. Tables C-003, C-113, C-138, C-143, C-148; graphite
# H - H(298) from C-002, kJ/mol.
C_VAPOUR = {  # n: {T: (delta-f H, log Kf)}
    1: {4000.0: (706.719, -1.207), 4100.0: (706.173, -0.982)},
    2: {4000.0: (805.652, -1.073), 4100.0: (804.375, -0.817)},
    3: {4000.0: (738.464, -0.317), 4100.0: (735.676, -0.082)},
    4: {4000.0: (916.075, -2.266), 4100.0: (913.580, -1.974)},
    5: {4000.0: (919.927, -1.922), 4100.0: (917.111, -1.629)},
}
GRAPHITE_H = {4000.0: 88.646, 4100.0: 91.432}


def apply(mixtures, weights, val):
    """Add the values to the mixture rows, making the four new skin rows.

    weights: element id -> IUPAC atomic weight (the sheet's elements table).
    Returns the list of new rows; existing rows are updated in place."""
    U, J = "unverified", "nist-janaf"
    by_id = {m["id"]: m for m in mixtures}
    new = []

    def row(mid, physics, plain, story, note):
        r = {"id": mid, "names": {"physics": physics, "plain": plain, "story": story},
             "recipe": {}, "by": "mass", "note": note}
        new.append(r)
        by_id[mid] = r
        return r

    row("aluminium", "aluminium metal", "aluminium", "aluminium", "pure metal; hull alloys (2xxx, 6xxx, 7xxx series) melt a little lower")
    row("titanium", "titanium metal", "titanium", "titanium", "pure metal; Ti-6Al-4V is the usual airframe alloy")
    row("iron", "iron metal", "iron", "iron", "pure iron; steels melt lower and have their own heats")
    row("graphite", "graphite (carbon)", "graphite", "carbon", "does not melt at 1 bar; it turns straight to vapour")

    for el, (tab, cp, tm, hs, hl, tb, hlb, hgb) in JANAF.items():
        r = by_id[ROW_OF[el]]
        kg = weights[el] * 1e-3  # kg per mol
        jk = lambda kj: round(kj * 1000.0 / kg, -2)  # noqa: E731
        r["heat_capacity"] = val(round(cp / kg, 1), "J kg-1 K-1", J, U, at_k=T_ROOM, note="%s, %.3f J/K/mol; climbs with temperature" % (tab, cp))
        r["heat_to_melt"] = val(jk(hs), "J kg-1", J, U, note="%s: H - H(298.15) of the solid at %.0f K, %.3f kJ/mol%s" % (tab, tm, hs, "; " + SOLID_STEPS[el] if el in SOLID_STEPS else ""))
        r["latent_fusion"] = val(jk(hl - hs), "J kg-1", J, U, at_k=tm, note="%s: %.3f kJ/mol, liquid minus solid H at %.0f K" % (tab, hl - hs, tm))
        if "melting_point_k" not in r and "solidus_k" not in r:
            r["melting_point_k"] = val(tm, "K", J, U, note=tab)
        if tb is None:
            r["note"] = r.get("note", "") + ("; " if r.get("note") else "") + "boiling point not in %s, so no boiling values yet" % tab
            continue
        r["heat_to_boil"] = val(jk(hlb - hl), "J kg-1", J, U, note="%s: liquid H from %.0f K to %.0f K, %.3f kJ/mol" % (tab, tm, tb, hlb - hl))
        r["boiling_point_k"] = val(round(tb, 1), "K", J, U, note="%s: liquid to ideal gas, fugacity 1 bar" % tab)
        r["latent_vaporisation"] = val(jk(hgb - hlb), "J kg-1", J, U, at_k=round(tb, 1), note="%s: %.3f kJ/mol, gas minus liquid H at Tb" % (tab, hgb - hlb))

    g, kc = by_id["graphite"], weights["c"] * 1e-3
    g["heat_capacity"] = val(round(GRAPHITE_CP / kc, 1), "J kg-1 K-1", J, U, at_k=T_ROOM, note="C-002 (graphite), %.3f J/K/mol" % GRAPHITE_CP)
    g["latent_sublimation"] = val(round(C_GAS_DFH * 1000.0 / kc, -3), "J kg-1", J, U, at_k=T_ROOM,
                                  note="C-003: delta-f H of C(g) at 298.15 K, %.3f kJ/mol, graphite to single atoms; real carbon vapour is mostly C3, which takes less; no sublimation temperature sourced yet" % C_GAS_DFH)

    # Tungsten's boiling point: log Kf = a + b/T through W-005's last two rows.
    import math
    (t1, h1, k1), (t2, h2, k2) = W_GAS
    b = (k2 - k1) / (1.0 / t2 - 1.0 / t1)
    a = k2 - b / t2
    tb = -b / a
    assert t2 < tb < t2 + 300.0, tb
    w, kw = by_id["tungsten"], weights["w"] * 1e-3
    hl_tb = W_LIQ_H_6000 + W_LIQ_CP * 1e-3 * (tb - t2)
    lv_tb = h2 + (h2 - h1) / (t2 - t1) * (tb - t2)
    tm, hl_tm = JANAF["w"][2], JANAF["w"][4]
    ext = "extrapolated %.0f K past the table's end at 6000 K" % (tb - t2)
    w["boiling_point_k"] = val(round(tb, 0), "K", J, U, note="W-005: where log Kf of W(g) over liquid tungsten reaches 0, fitting log Kf = a + b/T through the 5900 and 6000 K rows; " + ext)
    w["heat_to_boil"] = val(round((hl_tb - hl_tm) * 1000.0 / kw, -2), "J kg-1", J, U, note="W-001: liquid H from %.0f K (%.3f kJ/mol) to Tb, with the table's constant liquid Cp %.3f J/K/mol past 6000 K" % (tm, hl_tm, W_LIQ_CP))
    w["latent_vaporisation"] = val(round(lv_tb * 1000.0 / kw, -2), "J kg-1", J, U, at_k=round(tb, 0), note="W-005: delta-f H of W(g) over the liquid, %.1f kJ/mol, " % lv_tb + ext)
    w["note"] = w["note"].replace("; boiling point not in W-001, so no boiling values yet", "").replace("boiling point not in W-001, so no boiling values yet", "") or "tungsten metal"

    # Graphite's sublimation point: total carbon vapour (C1 to C5) reaches 1 bar.
    def p_total(t):
        return sum(10.0 ** C_VAPOUR[n][t][1] for n in C_VAPOUR)
    ta, tz = 4000.0, 4100.0
    la, lz = math.log10(p_total(ta)), math.log10(p_total(tz))
    assert la < 0.0 < lz, (la, lz)
    f = -la / (lz - la)
    ts = ta + f * (tz - ta)
    # Heat per mole of carbon to make the vapour mix at Ts: sum p_n dH_n / sum n p_n.
    def mix(t):
        num = sum(10.0 ** C_VAPOUR[n][t][1] * C_VAPOUR[n][t][0] for n in C_VAPOUR)
        den = sum(n * 10.0 ** C_VAPOUR[n][t][1] for n in C_VAPOUR)
        return num / den
    lsub = mix(ta) + f * (mix(tz) - mix(ta))
    c3 = 10.0 ** C_VAPOUR[3][tz][1] / p_total(tz)
    hs = GRAPHITE_H[ta] + f * (GRAPHITE_H[tz] - GRAPHITE_H[ta])
    g["sublimation_point_k"] = val(round(ts, 0), "K", J, U, note="where the carbon vapour over graphite, C1 to C5 (C-003, C-113, C-138, C-143, C-148), reaches 1 bar; log of the total pressure interpolated between the 4000 and 4100 K rows; the vapour there is about %.0f %% C3 by pressure" % (100.0 * c3))
    g["heat_to_sublime"] = val(round(hs * 1000.0 / kc, -3), "J kg-1", J, U, note="C-002: H - H(298.15) of graphite at Ts, %.2f kJ/mol" % hs)
    g["latent_sublimation"] = val(round(lsub * 1000.0 / kc, -3), "J kg-1", J, U, at_k=round(ts, 0),
                                  note="heat per mole of carbon to make the equilibrium C1 to C5 vapour at Ts, %.1f kJ/mol; to single atoms at 298.15 K it would be %.3f kJ/mol (C-003)" % (lsub, C_GAS_DFH))

    # Stainless steel 304L (Kim 1975, ANL-75-55), cal/g turned to J/kg here.
    def cal_g(x):
        return round(x * CAL_J * 1000.0, -2)
    ss = row("stainless-304l", "stainless steel type 304L", "stainless steel", "steel", "austenitic Cr-Ni steel; ANL-75-55 gives one set of heats for 304L and 316L")
    h_s = lambda t: -34.885 + 0.1122 * t + 1.611e-5 * t * t  # noqa: E731
    h_l = lambda t: 0.1900 * t - 56.588  # noqa: E731
    assert abs(h_s(1700.0) + 64.0 - h_l(1700.0)) < 0.05
    ss["solidus_k"] = val(1670.0, "K", "kim1975", U, note="melting range 1670-1730 K; the type 304 data sheet gives 1399-1421 C", also=["ati-304"])
    ss["liquidus_k"] = val(1730.0, "K", "kim1975", U)
    ss["density"] = val(7894.0, "kg m-3", "kim1975", U, at_k=300.0, note="Table 7, 304L; ATI's 304 sheet gives 7.90 g/cm3", also=["ati-304"])
    ss["heat_capacity"] = val(round((0.1122 + 3.222e-5 * 300.0) * CAL_J * 1000.0, 1), "J kg-1 K-1", "kim1975", U, at_k=300.0, note="eq. 5, 0.1122 + 3.222e-5 T cal/g K; ATI gives 500 J/kg K over 0-100 C")
    ss["heat_to_melt"] = val(cal_g(h_s(1700.0)), "J kg-1", "kim1975", U, note="eq. 1 at 1700 K, the melting temperature the report uses: %.2f cal/g" % h_s(1700.0))
    ss["latent_fusion"] = val(cal_g(64.0), "J kg-1", "kim1975", U, at_k=1700.0, note="64.0 cal/g")
    ss["heat_to_boil"] = val(cal_g(h_l(3080.0) - h_l(1700.0)), "J kg-1", "kim1975", U, note="eq. 9, liquid from 1700 K to 3080 K")
    ss["boiling_point_k"] = val(3080.0, "K", "kim1975", U)
    ss["latent_vaporisation"] = val(cal_g(1770.0), "J kg-1", "kim1975", U, at_k=3080.0, note="1770 cal/g")

    # Ti-6Al-4V: melting point and hot specific heat only so far.
    ti64 = row("ti-6al-4v", "titanium alloy Ti-6Al-4V", "titanium alloy", "titanium", "the usual airframe and pressure-tank titanium alloy; room-temperature heats not sourced yet")
    ti64["melting_point_k"] = val(1943.0, "K", "cezairliyan1977", U)
    ti64["heat_capacity_hot"] = val(826.0, "J kg-1 K-1", "cezairliyan1977", U, at_k=1850.0, note="Table 2, 0.826 J/g K; fit valid 1450-1900 K")

    # Aluminium 6061-T6: heat capacity at 300 K from the NIST cryogenic fit.
    coef = (46.6467, -314.292, 866.662, -1298.3, 1162.27, -637.795, 210.351, -38.3094, 2.96344)
    x = math.log10(300.0)
    cp61 = 10.0 ** sum(c * x ** k for k, c in enumerate(coef))
    al61 = row("al-6061", "aluminium alloy 6061-T6", "aluminium alloy", "aluminium", "the usual structural aluminium alloy; melting and latent heats not sourced yet, so the pure-metal row stands in")
    al61["heat_capacity"] = val(round(cp61, 0), "J kg-1 K-1", "nist-trc-cryo", U, at_k=300.0, note="the 4-300 K fit at its top end, 5 % fit error")
    assert 800.0 < cp61 < 1000.0, cp61

    # Rhenium: heat of sublimation at 298 K (Plante and Szwarc 1966).
    re_ = by_id["rhenium"]
    re_["latent_sublimation"] = val(round(185.9 * CAL_J * 1e6 / weights["re"], -3), "J kg-1", "plante-szwarc1966", U, at_k=T_ROOM,
                                    note="185.9 kcal/mol, solid to vapour at 298 K, third law; no melting or boiling heats sourced yet")

    for mid, d in DENSITY_G_CM3.items():
        if "density" not in by_id[mid]:
            by_id[mid]["density"] = val(round(d * 1000.0, 1), "kg m-3", "nist-srd126-t1", U,
                                        note="nominal; pyrolytic and nuclear graphites run 1.7 to 2.2 g/cm3" if mid == "graphite" else None)
            if by_id[mid]["density"]["note"] is None:
                del by_id[mid]["density"]["note"]

    # Inconel 718: Special Metals data sheet, Table 2 (source already on the sheet).
    by_id["inconel-718"]["heat_capacity"] = val(435.0, "J kg-1 K-1", "smc-inconel718", U, at_k=294.15, note="0.104 Btu/lb F at 70 F, printed as (435) J/kg C at 21 C; no latent heat in the data sheet")

    for r in by_id.values():
        if "heat_to_melt" in r:
            assert r["latent_fusion"]["v"] > 0 and r["heat_to_melt"]["v"] > 0, r["id"]
            at = r["latent_fusion"]["at_k"]
            if "melting_point_k" in r:
                assert abs(r["melting_point_k"]["v"] - at) < 30.0, (r["id"], at)
            else:
                assert r["solidus_k"]["v"] <= at <= r["liquidus_k"]["v"], (r["id"], at)
        if "boiling_point_k" in r:
            assert r["boiling_point_k"]["v"] > r["latent_fusion"]["at_k"], r["id"]
            assert r["latent_vaporisation"]["v"] > r["latent_fusion"]["v"], r["id"]
    return new
