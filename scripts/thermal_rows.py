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
    "nist-janaf": ("Chase, M. W. 1998, NIST-JANAF Thermochemical Tables, 4th edition, J. Phys. Chem. Ref. Data Monograph 9: reference-state tables (ref) for Al, Ti, Fe, Nb, Mo, Ta, W, Pb, C (graphite) and the C(g) table; Cp and H - H(298.15 K) at 298.15 K and at each transition row", None, "https://janaf.nist.gov/", "unverified"),
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
            top = r.get("melting_point_k", r.get("solidus_k"))["v"]
            assert abs(top - r["latent_fusion"]["at_k"]) < 30.0, (r["id"], top)
        if "boiling_point_k" in r:
            assert r["boiling_point_k"]["v"] > r["latent_fusion"]["at_k"], r["id"]
            assert r["latent_vaporisation"]["v"] > r["latent_fusion"]["v"], r["id"]
    return new
