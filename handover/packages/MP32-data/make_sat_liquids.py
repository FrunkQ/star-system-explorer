"""Make sat-liquids.csv: saturated-liquid density, viscosity and surface tension
of the sea liquids, read through CoolProp (open-source implementation of the
reference correlations NIST's WebBook uses). Run on a machine with CoolProp
installed; the sheet builder reads only the CSV, never CoolProp.

    python make_sat_liquids.py > sat-liquids.csv

Temperatures run from the triple point to 0.95 of the critical temperature in
eight equal steps (nine rows a fluid). Values are printed to 6 significant
figures. The correlation behind each column is named per fluid by CoolProp's
BibTeX keys, written into the CSV's first comment lines.
"""
import CoolProp
import CoolProp.CoolProp as CP

FLUIDS = [("h2o", "Water"), ("ch4", "Methane"), ("c2h6", "Ethane"), ("nh3", "Ammonia"),
          ("n2", "Nitrogen"), ("he", "Helium"), ("co2", "CarbonDioxide"), ("ar", "Argon"),
          ("c3h8", "Propane")]
STEPS = 8


def g(x):
    return float("%.6g" % x)


print("# CoolProp %s; columns: substance,t_k,p_pa,density_kg_m3,viscosity_pa_s,surface_tension_n_m" % CoolProp.__version__)
for sid, f in FLUIDS:
    keys = [CP.get_BibTeXKey(f, k) for k in ("EOS", "VISCOSITY", "SURFACE_TENSION")]
    print("# %s %s eos=%s viscosity=%s surface_tension=%s t_triple=%s t_crit=%s" % (
        sid, f, keys[0], keys[1], keys[2], g(CP.PropsSI("Ttriple", f)), g(CP.PropsSI("Tcrit", f))))
print("substance,t_k,p_pa,density_kg_m3,viscosity_pa_s,surface_tension_n_m")
for sid, f in FLUIDS:
    t0, tc = CP.PropsSI("Ttriple", f), CP.PropsSI("Tcrit", f)
    t0 = max(t0, CP.PropsSI("Tmin", f))
    for i in range(STEPS + 1):
        t = round(t0 + (0.95 * tc - t0) * i / STEPS, 2)
        if i == 0:
            t = round(t0 + 0.005, 2)  # just above the triple point, inside every correlation's range
        row = [CP.PropsSI(k, "T", t, "Q", 0, f) for k in ("P", "D", "V", "I")]
        print(",".join([sid, repr(t)] + [repr(g(x)) for x in row]))
