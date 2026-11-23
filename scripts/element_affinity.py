"""Where each element goes (element inventory part 1; matter pack design
sections 4.1 and 5): its 50% condensation temperature and host in a cooling
disc, the formation material it rides in, its share in CI chondrites, and
its concentrations in Earth's core, silicate Earth and continental crust.

The sheet stores the cited concentrations only. How an element splits
between core, mantle, crust, sea and air in another world is worked out by a
block from these (weighted by how far that world separated), never typed as
shares here.

Data: handover/packages/MP-affinity-data/ (README there gives every source,
the copy read, and each flag). Every value is a machine-readable
reproduction cross-checked against a second copy where one exists, so every
value is marked unverified.
"""
import csv
from pathlib import Path

DATA = Path(__file__).resolve().parent.parent / "handover/packages/MP-affinity-data"

# Lodders 2003's host phase, in words, to the formation material the element
# rides in (sse-formation's six groups; the sheet's formation-* mixtures).
METAL = ("fe alloy", "refractory metal alloy", "schreibersite")
WATER = ("h2o", "nh3")
SULFIDE = ("troilite", "fes")
# Carbon: equilibrium puts it in methane ice, but chondrites carry it as
# organics, which is formation's carbon group.
RIDES_OVERRIDE = {"c": ("formation-carbon", "equilibrium condensation puts carbon in methane ice below 40 K; chondrites carry it as organics, formation's carbon group")}
# Elements the core column holds wrongly in the copy read (see the README).
CORE_FIX = {"co": (2500.0, "printed 0.25 ppm in the copy read; 0.25 wt% meant (McDonough 1998 gives 2500 ug/g)")}
CORE_SKIP = {"ti": "printed 0.03 ppm in the copy read, unit unsure: left out"}


def rides_in(sym, host, tc):
    h = host.lower()
    if sym in RIDES_OVERRIDE:
        return RIDES_OVERRIDE[sym]
    if not tc:
        return ("gas", "no 50% condensation temperature: stays in the gas")
    if h.startswith(("ne ice", "he ice")):
        return ("gas", "condenses only below any disc's temperature")
    if any(k in h for k in WATER) or "6h2o" in h.replace("·", ""):
        return ("formation-water", None)
    if any(h.startswith(k) for k in METAL) or (h.startswith("fe alloy") and "feldspar" in h):
        return ("formation-metal", None)
    if any(k in h for k in SULFIDE) and "enstatite" not in h:
        return ("formation-metal", "in iron sulfide, which formation counts with the metal")
    return ("formation-rock", None)


def _f(x):
    x = (x or "").strip()
    return float(x) if x else None


def _share(ppm):
    """A concentration in ppm as a mass share, to the copy's own six figures."""
    return float(f"{ppm * 1e-6:.6g}")


def load():
    """symbol (lower case) -> dict of sheet values, built with the caller's val()."""
    out = {}
    for r in csv.DictReader(open(DATA / "lodders2003_tc50.csv", newline="")):
        s = r["symbol"].strip().lower()
        out.setdefault(s, {})["tc"] = (_f(r["tc50_k"]), r["host_phase_as_written"].strip())
    for r in csv.DictReader(open(DATA / "earth_core_mantle.csv", newline="")):
        s = r["symbol"].strip().lower()
        d = out.setdefault(s, {})
        d["core"], d["silicate"] = _f(r["core_ppm"]), _f(r["silicate_earth_ppm"])
    for r in csv.DictReader(open(DATA / "crust_rudnick_gao.csv", newline="")):
        out.setdefault(r["symbol"].strip().lower(), {})["crust"] = _f(r["bulk_crust_ppm"])
    for r in csv.DictReader(open(DATA / "ci_mcdonough_sun1995.csv", newline="")):
        out.setdefault(r["symbol"].strip().lower(), {})["ci"] = (_f(r["ci_mass_fraction"]), r["note"])
    return out


def apply(row, data, val):
    """Add the affinity values for one element row (in place)."""
    d = data.get(row["symbol"].lower())
    if not d:
        return
    sym = row["symbol"].lower()
    if "tc" in d:
        tc, host = d["tc"]
        mat, why = rides_in(sym, host, tc)
        cond = {"host_phase": host, "rides_in": mat, "src": "lodders2003", "check": "unverified"}
        if tc:
            cond["tc50_k"] = val(tc, "K", "lodders2003", "unverified", at_pa=10.0, note="50% condensed from a gas of solar make-up at 1e-4 bar (Table 8)")
        if why:
            cond["note"] = why
        row["condensation"] = cond
    if d.get("ci"):
        v, note = d["ci"]
        if v is not None:
            row["ci_mass_fraction"] = val(v, "1", "mcdonough-sun1995", "unverified", note=f"CI chondrite by mass; {note}")
    earth = {}
    core = d.get("core")
    if sym in CORE_FIX:
        core, cnote = CORE_FIX[sym]
    else:
        cnote = None
    if core is not None and sym not in CORE_SKIP:
        earth["core"] = val(_share(core), "1", "mcdonough2003", "unverified", **({"note": cnote} if cnote else {}))
    if d.get("silicate") is not None:
        earth["silicate"] = val(_share(d["silicate"]), "1", "mcdonough-sun1995", "unverified", note="primitive mantle (pyrolite)")
    if d.get("crust") is not None:
        earth["crust"] = val(_share(d["crust"]), "1", "rudnick-gao2003", "unverified", note="bulk continental crust")
    if earth:
        row["earth_mass_fraction"] = earth
