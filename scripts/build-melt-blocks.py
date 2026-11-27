"""Write packs/base-physics/formulas/melt.json: the melt and lid blocks of the
melt design (track HF, package HF4b.0; handover/design/SSE4-melt-and-heat-design.md
sections 3 and 4, block list in 8.2).

Each block is a formula with its cited source and a worked example. The worked
examples are worked here, in Python, from the same equation the block holds, and
the block loader runs every example again in Rust, so a slip in either fails.

Rock numbers are inputs (the sheet's mantle-rock and basalt rows feed them,
scripts/melt_rows.py); a block's own parameter is only a model choice the
design names as a setting (a_rh, the melt-fraction exponent, the Andrade
switch, the rigidity floor). Nothing reads these blocks yet: HF4b.1 and HF4b.2
wire them into the thermal history, HF4c.1 the tidal response.

The two heavy blocks of the design (sse.melt.zone, a 48-point sum, and
sse.tides.layered_response) are Rust and land with the packages that read them.

Every source was written from memory on 27 Nov 2026 while the paper search was
out (until 1 Dec 2026): unverified until read at source.

Run: python3 scripts/build-melt-blocks.py   (--check to compare only)
"""

import json
import math
import sys
from pathlib import Path

OUT = Path(__file__).resolve().parent.parent / "packs/base-physics/formulas/melt.json"
R_GAS = 8.314462618
G = 6.6743e-11

ABOUT = ("Physics blocks whose maths is a formula (ND3b): each keeps its id, ports and results, "
         "and its maths is the Equation below, with a name that says what happens.")


def block(id_, name, shown, formula, inputs, outputs, source, refs, curious, science, examples,
          params=(), constants=None, positive=()):
    letters_in = [k for k in inputs if not (constants and k in constants) and k not in {p[1] for p in params}]
    a = {
        "id": id_,
        "version": "1.0.0",
        "cost": "trivial",
        "inputs": [{"port": inputs[k][3], "letter": k} for k in letters_in],
        "outputs": [{"port": outputs[k][3], "letter": k} for k in outputs],
        "constants": dict(constants or {}),
        "params": [{"param": p[0], "letter": p[1], "default": p[2], "range": p[3], "description": p[4]}
                   for p in params],
        "positive": [{"port": inputs[k][3], "says": inputs[k][2]} for k in positive],
        "references": refs,
        "shown": shown,
        "equation": {
            "name": name,
            "formula": formula,
            "inputs": {k: {"kind": v[0], "unit": v[1], "name": v[2]} for k, v in inputs.items()},
            "outputs": {k: {"kind": v[0], "unit": v[1], "name": v[2]} for k, v in outputs.items()},
            "source": source,
            "explain": {"curious": curious, "science": science},
            "examples": examples,
        },
    }
    return a


def ex(inputs, outputs):
    d = dict(inputs)
    d.update(outputs)
    return d


# ------------------------------------------------------------------ sources
HIRSCHMANN = "Hirschmann, M. M. 2000, Geochemistry, Geophysics, Geosystems 1, 1042: mantle solidus. doi:10.1029/2000GC000070"
KATZ = "Katz, R. F., Spiegelman, M. and Langmuir, C. H. 2003, Geochemistry, Geophysics, Geosystems 4, 1073: a new parameterization of hydrous mantle melting. doi:10.1029/2002GC000433"
MORSCH = "Morschhauser, A., Grott, M. and Breuer, D. 2011, Icarus 212, 541: crustal recycling, mantle dehydration and the thermal evolution of Mars. doi:10.1016/j.icarus.2010.12.028"
DAVAILLE = "Davaille, A. and Jaupart, C. 1993, Journal of Fluid Mechanics 253, 141: transient high-Rayleigh-number thermal convection with large viscosity variations. doi:10.1017/S0022112093001740"
SOLOMATOV = "Solomatov, V. S. and Moresi, L.-N. 2000, JGR 105, 21795: scaling of time-dependent stagnant lid convection. doi:10.1029/2000JB900197"
TURCOTTE = "Turcotte, D. L. and Schubert, G. 2014, Geodynamics, 3rd edition, Cambridge University Press. doi:10.1017/CBO9780511843877"
OREILLY = "O'Reilly, T. C. and Davies, G. F. 1981, Geophysical Research Letters 8, 313: magma transport of heat on Io, a mechanism allowing a thick lithosphere. doi:10.1029/GL008i004p00313"
MOORE_WEBB = "Moore, W. B. and Webb, A. A. G. 2013, Nature 501, 501: heat-pipe Earth. doi:10.1038/nature12473"
MILLER = "Miller, K. J., Zhu, W., Montesi, L. G. J. and Gaetani, G. A. 2014, Earth and Planetary Science Letters 388, 273: permeability of partially molten mantle rock. doi:10.1016/j.epsl.2013.12.003"
BIERSON = "Bierson, C. J. and Nimmo, F. 2016, JGR Planets 121, 2211: a test for Io's magma ocean. doi:10.1002/2016JE005005"
MEI = "Mei, S., Bai, W., Hiraga, T. and Kohlstedt, D. L. 2002, Earth and Planetary Science Letters 201, 491: influence of melt on the creep behaviour of olivine-basalt aggregates. doi:10.1016/S0012-821X(02)00722-0"
COSTA = "Costa, A., Caricchi, L. and Bagdassarov, N. 2009, Geochemistry, Geophysics, Geosystems 10, Q03010: rheology of partially molten rocks. doi:10.1029/2008GC002138"
WHITE = "White, S. M., Crisp, J. A. and Spera, F. J. 2006, Geochemistry, Geophysics, Geosystems 7, Q03010: long-term volumetric eruption rates and magma budgets. doi:10.1029/2005GC001002"
CASTILLO = "Castillo-Rogez, J. C., Efroimsky, M. and Lainey, V. 2011, JGR 116, E09008: the tidal history of Iapetus, Andrade rheology. doi:10.1029/2010JE003664"
EFROIMSKY = "Efroimsky, M. 2012, ApJ 746, 150: tidal dissipation compared to seismic dissipation, in small bodies, earths and super-earths. doi:10.1088/0004-637X/746/2/150"
HENNING = "Henning, W. G., O'Connell, R. J. and Sasselov, D. D. 2009, ApJ 707, 1000: tidally heated terrestrial exoplanets. doi:10.1088/0004-637X/707/2/1000"
LAINEY09 = "Lainey, V. et al. 2009, Nature 459, 957: strong tidal dissipation in Io and Jupiter from astrometric observations. doi:10.1038/nature08108"
MURRAY = "Murray, C. D. and Dermott, S. F. 1999, Solar System Dynamics, Cambridge University Press. doi:10.1017/CBO9781139174817"

blocks = []

# ------------------------------------------------------------ melt fraction
def melt_fraction(T, Ts, Tl, L, c, beta=1.0):
    f = (T - Ts) / (Tl - Ts + L / c)
    return min(max(f, 0.0), 1.0) ** beta

blocks.append(block(
    "sse.melt.fraction", "How much of a rock has melted",
    "F = clamp((T − T_sol) / (T_liq − T_sol + L/c), 0, 1)^β",
    "F = clamp((T - Ts) / (Tl - Ts + L / c), 0, 1) ^ be",
    {"T": ("interior.parcel_temperature", "K", "temperature of the rock", "temperature"),
     "Ts": ("melt.solidus_temperature", "K", "where the rock starts to melt at this depth (solidus)", "solidus"),
     "Tl": ("melt.liquidus_temperature", "K", "where the rock is wholly molten at this depth (liquidus)", "liquidus"),
     "L": ("range.latent_fusion", "J kg-1", "heat it takes to melt a kilogram (latent heat of fusion)", "latent_heat"),
     "c": ("layer.specific_heat", "J kg-1 K-1", "heat it takes to warm a kilogram by one kelvin", "heat_capacity"),
     "be": ("model.melt_fraction_exponent", "1", "how the melt share grows between solidus and liquidus (1: in a straight line)", "")},
    {"F": ("melt.fraction", "1", "share of the rock that is molten", "fraction")},
    {"text": "Morschhauser, A., Grott, M. and Breuer, D. 2011, Icarus 212, 541", "doi": "10.1016/j.icarus.2010.12.028",
     "where": "the linear melt fraction between solidus and liquidus used in parametrised Mars models; the latent term from Katz et al. 2003"},
    [MORSCH, KATZ, HIRSCHMANN],
    "Rock does not melt all at once. It starts at the solidus and is wholly liquid at the liquidus, and between them the molten share grows. Melting soaks up heat, so a rock warmed past its solidus melts less than its temperature alone suggests.",
    "Melt fraction of a parcel at temperature T between the pressure-dependent solidus and liquidus, with the latent heat of fusion in the denominator (L/c ≈ 330 K for rock), so the parcel's cooling as it melts is counted. Linear (β = 1) as in parametrised models; Katz et al. 2003 give β ≈ 1.5 and a kink at clinopyroxene exhaustion. Water lowers the solidus; that enters with the mantle's water budget.",
    [ex({"T": 1900.0, "Ts": 1700.0, "Tl": 2100.0, "L": 4.0e5, "c": 1200.0}, {"F": melt_fraction(1900.0, 1700.0, 2100.0, 4.0e5, 1200.0)}),
     ex({"T": 1600.0, "Ts": 1700.0, "Tl": 2100.0, "L": 4.0e5, "c": 1200.0}, {"F": 0.0})],
    params=[("exponent", "be", 1.0, [1.0, 2.0], "how the melt share grows between solidus and liquidus: 1 in a straight line (Morschhauser et al. 2011), about 1.5 (Katz et al. 2003)")],
    positive=["c"]))

# ------------------------------------------------------- lid base temperature
def lid_base(Tm, E, a=2.24):
    return Tm - a * R_GAS * Tm * Tm / E

blocks.append(block(
    "sse.heat.lid_base_temperature", "Where a world's stiff lid ends and its mantle starts to flow",
    "T_l = T_m − a_rh R T_m² / E",
    "Tl = Tm - a * Rg * Tm * Tm / E",
    {"Tm": ("interior.mantle_temperature", "K", "temperature of the convecting mantle", "mantle_temperature"),
     "E": ("interior.creep_activation_energy", "J mol-1", "how strongly the rock's stiffness depends on temperature (activation energy)", "activation_energy"),
     "Rg": ("constant.R_gas", "J mol-1 K-1", "gas constant", ""),
     "a": ("model.rheological_temperature_scale", "1", "how many viscous temperature scales the lid's base sits below the mantle", "")},
    {"Tl": ("layer.base_temperature", "K", "temperature at the base of the stiff lid", "lid_base_temperature")},
    {"text": "Davaille, A. and Jaupart, C. 1993, Journal of Fluid Mechanics 253, 141", "doi": "10.1017/S0022112093001740",
     "where": "the rheological temperature scale ΔT_rh = R T²/E, and the lid's base about 2.2 of them below the mantle"},
    [DAVAILLE, SOLOMATOV],
    "A cold world's top is too stiff to stir. Below a certain temperature the rock stops flowing with the mantle and joins the lid; this says how far below the mantle's temperature that is.",
    "In stagnant-lid convection the lid's base sits a fixed number a_rh ≈ 2.2 of rheological temperature scales R T_m²/E below the interior temperature, independent of the lid's thickness (Davaille and Jaupart 1993; Solomatov and Moresi 2000).",
    [ex({"Tm": 1700.0, "E": 3.0e5}, {"Tl": lid_base(1700.0, 3.0e5)})],
    params=[("rheological_scale", "a", 2.24, [1.0, 4.0], "how many viscous temperature scales R T²/E the lid's base sits below the mantle (Davaille and Jaupart 1993: about 2.2)")],
    constants={"Rg": "sheet/R_gas"},
    positive=["Tm", "E"]))

# ------------------------------------------------------------- lid growth
def lid_growth(rho, c, Tm, Tl, qb, qm, qi):
    return (qb - qm - qi) / (rho * c * (Tm - Tl))

blocks.append(block(
    "sse.heat.lid_growth", "How fast a world's stiff lid thickens or thins",
    "ρ c (T_m − T_l) dD/dt = q_base − q_m − q_intruded",
    "dD = (qb - qm - qi) / (rho * c * (Tm - Tl))",
    {"rho": ("layer.density", "kg m-3", "density of the rock joining the lid", "density"),
     "c": ("layer.specific_heat", "J kg-1 K-1", "heat it takes to warm a kilogram by one kelvin", "heat_capacity"),
     "Tm": ("interior.mantle_temperature", "K", "temperature of the convecting mantle", "mantle_temperature"),
     "Tl": ("layer.base_temperature", "K", "temperature at the base of the stiff lid", "lid_base_temperature"),
     "qb": ("heat.lid_base_flux", "W m-2", "heat conducted away from the lid's base", "base_flux"),
     "qm": ("heat.mantle_flux", "W m-2", "heat the convecting mantle delivers to the lid's base", "mantle_flux"),
     "qi": ("heat.intruded_flux", "W m-2", "heat of melt freezing inside the lid", "intruded_flux")},
    {"dD": ("heat.lid_growth_rate", "m s-1", "how fast the lid's base moves down (negative: the lid thins)", "growth_rate")},
    {"text": "Turcotte, D. L. and Schubert, G. 2014, Geodynamics, 3rd edition", "doi": "10.1017/CBO9780511843877",
     "where": "the Stefan condition at a moving boundary (section 4.18), here with the mantle's convective flux and intrusions on one side; as used in parametrised stagnant-lid models (Morschhauser et al. 2011)"},
    [TURCOTTE, MORSCH, SOLOMATOV],
    "A world's lid grows when its base loses heat faster than the mantle below supplies it: rock at the base cools, stiffens and joins the lid. When the mantle supplies more, or melt freezing inside the lid warms it, the lid thins.",
    "Energy balance at the lid's base: the rock crossing it changes temperature by T_m − T_l. A young lid (D → 0) conducts a lot and grows as √t, so its surface heat flow falls as 1/√t: a cooling half-space, its heat coming from the rock joining it, not from the mantle. A steady lid has q_base = q_m.",
    [ex({"rho": 3400.0, "c": 1200.0, "Tm": 1700.0, "Tl": 1520.0, "qb": 0.03, "qm": 0.02, "qi": 0.0},
        {"dD": lid_growth(3400.0, 1200.0, 1700.0, 1520.0, 0.03, 0.02, 0.0)})],
    positive=["rho", "c"]))

# ------------------------------------------------ lid conduction with burial
def lid_cond(k, Tl, Ts, D, h, v, kap):
    pe = v * D / kap
    fs = pe / math.expm1(pe) if pe > 0 else 1.0
    q0 = k * (Tl - Ts) / D
    return q0 * fs + h / 2, q0 * fs * math.exp(pe) - h / 2

lc1 = lid_cond(3.0, 1500.0, 220.0, 1.0e5, 0.005, 0.0, 1.0e-6)
lc2 = lid_cond(3.0, 1500.0, 220.0, 1.0e5, 0.005, 2.0e-11, 1.0e-6)
blocks.append(block(
    "sse.heat.lid_conduction_with_burial", "Heat a world's lid conducts, when lava may be burying it",
    "Pe = v D/κ;  q_top = k ΔT/D · Pe/(e^Pe − 1) + h/2;  q_base = k ΔT/D · Pe e^Pe/(e^Pe − 1) − h/2",
    "Pe = v * D / kap\nfs = if Pe > 0 then Pe / expm1(Pe) else 1\nq0 = k * (Tl - Ts) / D\nqs = q0 * fs + h / 2\nqb = q0 * fs * exp(Pe) - h / 2",
    {"k": ("interior.thermal_conductivity", "W m-1 K-1", "how well the lid's rock conducts heat", "conductivity"),
     "Tl": ("layer.base_temperature", "K", "temperature at the base of the lid", "base_temperature"),
     "Ts": ("layer.top_temperature", "K", "temperature at the surface", "surface_temperature"),
     "D": ("layer.thickness", "m", "thickness of the lid", "thickness"),
     "h": ("heat.lid_heating", "W m-2", "heat made inside the lid, per square metre of surface (its crust's radioactivity)", "lid_heating"),
     "v": ("heat.burial_speed", "m s-1", "how fast erupted lava buries the surface", "burial_speed"),
     "kap": ("interior.thermal_diffusivity", "m2 s-1", "how fast heat spreads through the lid's rock", "diffusivity")},
    {"qs": ("heat.lid_surface_flux", "W m-2", "heat conducted out of the surface", "surface_flux"),
     "qb": ("heat.lid_base_flux", "W m-2", "heat conducted away from the lid's base", "base_flux")},
    {"text": "O'Reilly, T. C. and Davies, G. F. 1981, Geophysical Research Letters 8, 313", "doi": "10.1029/GL008i004p00313",
     "where": "steady conduction through a layer advected downward at the resurfacing speed; the h/2 split of heat made inside a layer from Turcotte and Schubert 2014, chapter 4"},
    [OREILLY, MOORE_WEBB, TURCOTTE],
    "Heat leaks up through a world's cold lid. Where lava keeps pouring out and burying the surface, the cold top is pushed down faster than heat can climb, so the surface conducts almost nothing and the lava carries the heat instead: this is how Io holds up mountains while losing heat fast.",
    "Steady conduction in a layer of thickness D moving downward at v: the Péclet number Pe = vD/κ chokes the top flux by Pe/(e^Pe − 1) and raises the base flux by e^Pe. At Pe → 0 both are k ΔT/D. Heat made inside the lid (h, W/m² of surface) splits half up and half down, exact for a uniform source without burial and kept as the first-order split with it.",
    [ex({"k": 3.0, "Tl": 1500.0, "Ts": 220.0, "D": 1.0e5, "h": 0.005, "v": 0.0, "kap": 1.0e-6}, {"qs": lc1[0], "qb": lc1[1]}),
     ex({"k": 3.0, "Tl": 1500.0, "Ts": 220.0, "D": 1.0e5, "h": 0.005, "v": 2.0e-11, "kap": 1.0e-6}, {"qs": lc2[0], "qb": lc2[1]})],
    positive=["k", "D", "kap"]))

# ------------------------------------------------- boundary layer speed
def bl_speed(kap, D, k, dT, q):
    d = k * dT / q
    return kap * D / (d * d)

blocks.append(block(
    "sse.heat.boundary_layer_speed", "How fast a world's mantle turns over",
    "δ = k ΔT/q;  u = κ D/δ²",
    "d = k * dT / q\nu = kap * D / (d * d)",
    {"kap": ("interior.thermal_diffusivity", "m2 s-1", "how fast heat spreads through the rock", "diffusivity"),
     "D": ("interior.mantle_depth", "m", "depth of the convecting mantle", "mantle_depth"),
     "k": ("interior.thermal_conductivity", "W m-1 K-1", "how well the rock conducts heat", "conductivity"),
     "dT": ("heat.boundary_layer_temperature_drop", "K", "temperature drop across the mantle's top boundary layer", "temperature_drop"),
     "q": ("heat.mantle_flux", "W m-2", "heat the convecting mantle delivers through that layer", "mantle_flux")},
    {"u": ("interior.convection_speed", "m s-1", "typical speed of the mantle's flow", "speed")},
    {"text": "Turcotte, D. L. and Schubert, G. 2014, Geodynamics, 3rd edition", "doi": "10.1017/CBO9780511843877",
     "where": "boundary-layer theory of convection (section 6.21): a boundary layer of thickness δ grows for δ²/κ while the flow crosses the depth D"},
    [TURCOTTE, MORSCH],
    "A mantle's slow churning brings fresh rock up to where it can melt. How fast it churns follows from how much heat it carries: a thin, steep top layer means a fast flow.",
    "The boundary layer's thickness δ follows from the heat it conducts, q = k ΔT/δ; it grows for a time δ²/κ while the flow crosses the mantle's depth D, so u = κD/δ². No new constant: the speed the heat flow already implies.",
    [ex({"kap": 1.0e-6, "D": 1.7e6, "k": 3.0, "dT": 100.0, "q": 0.02}, {"u": bl_speed(1.0e-6, 1.7e6, 3.0, 100.0, 0.02)})],
    positive=["kap", "D", "k", "dT", "q"]))

# ------------------------------------------------------------ melt supply
def supply(u, F, V, x, D):
    return u * F * V * x / D

blocks.append(block(
    "sse.melt.supply", "Melt made as the mantle's flow carries rock through its melting zone",
    "Π = u F̄ V_zone x / D",
    "P = u * F * V * x / D",
    {"u": ("interior.convection_speed", "m s-1", "typical speed of the mantle's flow", "speed"),
     "F": ("melt.zone_mean_fraction", "1", "mean molten share across the melting zone", "mean_fraction"),
     "V": ("melt.zone_volume", "m3", "volume of the melting zone under the lid", "zone_volume"),
     "x": ("melt.fertile_share", "1", "share of the rock that can still melt (falls as basalt is taken)", "fertile_share"),
     "D": ("interior.mantle_depth", "m", "depth of the convecting mantle", "mantle_depth")},
    {"P": ("melt.production_rate", "m3 s-1", "melt made, volume per second", "production")},
    {"text": "Morschhauser, A., Grott, M. and Breuer, D. 2011, Icarus 212, 541", "doi": "10.1016/j.icarus.2010.12.028",
     "where": "crust production from the convective speed through the melt zone (they divide by the planet's radius; the mantle's depth here, registered as a fudge)"},
    [MORSCH],
    "A mantle melts only where rock rises high enough under the lid. The faster the flow carries rock through that zone, the more melt it makes, and once rock has given up its basalt it gives little more.",
    "Volume of melt made per second by convective supply: the flow sweeps the melt zone's volume once per D/u, each pass extracting the zone's mean melt fraction from the share x of fertile rock. The length the mantle turns over on (D here, the planet's radius in Morschhauser et al.) is registered as a fudge.",
    [ex({"u": 1.0e-9, "F": 0.1, "V": 1.0e19, "x": 1.0, "D": 1.7e6}, {"P": supply(1.0e-9, 0.1, 1.0e19, 1.0, 1.7e6)})],
    positive=["D"]))

# -------------------------------------------------------- melt in place
def in_place(N, s, rho, L, c, Tm, Tl):
    return s * max(N, 0.0) / (rho * (L + c * (Tm - Tl)))

blocks.append(block(
    "sse.melt.in_place", "Melt made where extra heat lands inside the melting zone",
    "Π = share · max(N, 0) / (ρ_melt (L + c (T_m − T_l)))",
    "P = s * max(N, 0) / (rho * (L + c * (Tm - Tl)))",
    {"N": ("heat.surplus_power", "W", "heat the mantle gains beyond what it loses (only a gain melts rock)", "surplus"),
     "s": ("melt.heat_share", "1", "share of that heat landing in the melting zone (its volume share; all of a tide's heat)", "share"),
     "rho": ("melt.density", "kg m-3", "density of the melt", "melt_density"),
     "L": ("range.latent_fusion", "J kg-1", "heat it takes to melt a kilogram", "latent_heat"),
     "c": ("layer.specific_heat", "J kg-1 K-1", "heat it takes to warm a kilogram by one kelvin", "heat_capacity"),
     "Tm": ("interior.mantle_temperature", "K", "temperature of the convecting mantle", "mantle_temperature"),
     "Tl": ("layer.base_temperature", "K", "temperature at the base of the lid", "lid_base_temperature")},
    {"P": ("melt.production_rate", "m3 s-1", "melt made, volume per second", "production")},
    {"text": "Moore, W. B. and Webb, A. A. G. 2013, Nature 501, 501", "doi": "10.1038/nature12473",
     "where": "heat beyond what conduction can carry leaves as melt (the heat-pipe balance); graded here by the share of the heat landing in the melting zone"},
    [MOORE_WEBB, BIERSON],
    "When a world is heated faster than it can lose heat, the part of that heat landing in rock already at its melting point does not warm it further: it melts more of it. That is why a tidally squeezed world like Io erupts instead of boiling.",
    "Melt made in place by a heat surplus N: its share inside the melt zone (the zone's volume share for radiogenic and core heat; 1 for tidal heat, made where the rock is soft) pays the latent heat plus warming from the lid's base to the mantle. With the whole mantle above its solidus this is HF1's venting at the solidus.",
    [ex({"N": 1.0e12, "s": 0.1, "rho": 2800.0, "L": 4.0e5, "c": 1200.0, "Tm": 1700.0, "Tl": 1520.0},
        {"P": in_place(1.0e12, 0.1, 2800.0, 4.0e5, 1200.0, 1700.0, 1520.0)})],
    positive=["rho", "L"]))

# ------------------------------------------------------------ melt retained
def retained(P, A, eta, b, dr, g, n, C, pc):
    return min((P / A * eta * C / (b * b * dr * g)) ** (1.0 / n), pc)

r1 = retained(1.0, 1.0e13, 10.0, 0.01, 500.0, 3.7, 2.6, 58.0, 0.4)
r2 = retained(5.2e4, 4.2e13, 10.0, 0.01, 500.0, 1.8, 2.6, 58.0, 0.4)
blocks.append(block(
    "sse.melt.retained", "How much melt stays in the rock while it seeps out",
    "φ = min( (Π/A · η_melt C / (b² Δρ g))^(1/n), φ_c )",
    "phi = min((P / A * eta * C / (b * b * dr * g)) ^ (1 / n), pc)",
    {"P": ("melt.production_rate", "m3 s-1", "melt made, volume per second", "production"),
     "A": ("melt.zone_top_area", "m2", "area of the melting zone's top", "area"),
     "eta": ("melt.viscosity", "kg m-1 s-1", "how runny the melt is (its viscosity)", "melt_viscosity"),
     "b": ("melt.grain_size", "m", "size of the rock's grains", "grain_size"),
     "dr": ("melt.density_contrast", "kg m-3", "how much denser the rock is than its melt", "density_contrast"),
     "g": ("layer.gravity", "m s-2", "gravity where the melt is", "gravity"),
     "n": ("melt.permeability_exponent", "1", "how fast the rock's permeability grows with its melt share", "permeability_exponent"),
     "C": ("melt.permeability_constant", "1", "the permeability law's constant: permeability is b² φⁿ / C", "permeability_constant"),
     "pc": ("melt.lockup_share", "1", "melt share at which the grains stop touching", "lockup_share")},
    {"phi": ("melt.retained_share", "1", "share of the rock that is melt waiting to leave", "retained")},
    {"text": "Miller, K. J., Zhu, W., Montesi, L. G. J. and Gaetani, G. A. 2014, Earth and Planetary Science Letters 388, 273", "doi": "10.1016/j.epsl.2013.12.003",
     "where": "the permeability law k = b² φⁿ / C (n ≈ 2.6, C ≈ 58), in Darcy's law for buoyant melt as in Bierson and Nimmo 2016"},
    [MILLER, BIERSON],
    "Melt seeps up between the grains of the rock like water through sand. A world making little melt drains it as fast as it is made and holds only a trace; a world making a lot, like Io, holds more, and that is what softens it.",
    "Darcy percolation at a speed (b² φⁿ / C) Δρ g / η_melt through the melt zone's top; melt crosses the zone far faster than a step, so the share held is the one at which that flux carries the melt being made. Capped at lock-up, where the mush stops being a solid. A negative Δρ (water under ice) drains down instead; the same law with the sign. Grain size is the number this is most sensitive to.",
    [ex({"P": 1.0, "A": 1.0e13, "eta": 10.0, "b": 0.01, "dr": 500.0, "g": 3.7, "n": 2.6, "C": 58.0, "pc": 0.4}, {"phi": r1}),
     ex({"P": 5.2e4, "A": 4.2e13, "eta": 10.0, "b": 0.01, "dr": 500.0, "g": 1.8, "n": 2.6, "C": 58.0, "pc": 0.4}, {"phi": r2})],
    positive=["A", "eta", "b", "dr", "g", "n", "C"]))

# ------------------------------------------------------------ softening
def soften(phi, pc, a, eta0, mu0, fl=0.05):
    return eta0 * math.exp(-a * min(phi, pc)), mu0 * max(1.0 - phi / pc, fl)

s1 = soften(0.05, 0.4, 26.0, 1.0e19, 6.5e10)
blocks.append(block(
    "sse.melt.softening", "How melt between the grains softens a rock",
    "η = η₀ e^(−a φ);  μ = μ₀ max(1 − φ/φ_c, floor)",
    "eta = eta0 * exp(-a * min(phi, pc))\nmu = mu0 * max(1 - phi / pc, fl)",
    {"phi": ("melt.retained_share", "1", "share of the rock that is melt", "retained"),
     "pc": ("melt.lockup_share", "1", "melt share at which the grains stop touching", "lockup_share"),
     "a": ("melt.weakening_exponent", "1", "how strongly melt weakens the rock's flow", "weakening"),
     "eta0": ("interior.viscosity", "kg m-1 s-1", "the rock's viscosity with no melt", "viscosity"),
     "mu0": ("interior.rigidity", "kg m-1 s-2", "the rock's rigidity with no melt", "rigidity"),
     "fl": ("model.rigidity_floor", "1", "the least share of its rigidity a mush keeps before lock-up", "")},
    {"eta": ("interior.viscosity", "kg m-1 s-1", "viscosity of the partly molten rock", "softened_viscosity"),
     "mu": ("interior.rigidity", "kg m-1 s-2", "rigidity of the partly molten rock", "softened_rigidity")},
    {"text": "Mei, S., Bai, W., Hiraga, T. and Kohlstedt, D. L. 2002, Earth and Planetary Science Letters 201, 491", "doi": "10.1016/S0012-821X(02)00722-0",
     "where": "viscosity falls as exp(−a φ), a ≈ 26 (diffusion creep); the rigidity law is a linear stand-in going to zero at lock-up (Costa et al. 2009), registered as a fudge until a measured form is read at source"},
    [MEI, COSTA, BIERSON],
    "A little melt between the grains makes rock flow much more easily: five per cent melt makes it hundreds of times runnier. It also makes the rock less springy, until at about forty per cent the grains stop touching and it flows like a thick liquid.",
    "Melt-weakened viscosity η₀ exp(−a φ) (Mei et al. 2002) and a linear rigidity loss to zero at the lock-up share φ_c, floored. The same softened viscosity feeds the convection scaling and the tidal response: one viscosity, read in both places. The rigidity law is a stand-in, flagged in the fudge register.",
    [ex({"phi": 0.05, "pc": 0.4, "a": 26.0, "eta0": 1.0e19, "mu0": 6.5e10}, {"eta": s1[0], "mu": s1[1]})],
    params=[("rigidity_floor", "fl", 0.05, [0.0, 1.0], "the least share of its rigidity a partly molten rock keeps below lock-up (a stand-in)")],
    positive=["pc", "eta0", "mu0"]))

# ------------------------------------------------------------ processed
def processed(rho, P, dt, F, M):
    p = rho * P * dt / (F * M)
    return p, math.exp(-p)

pr = processed(2800.0, 1.0, 3.156e13, 0.1, 4.9e23)
blocks.append(block(
    "sse.melt.processed", "How much of a mantle has had its melt taken",
    "processed = ρ Π dt / (F̄ M_mantle);  kept = e^(−processed)",
    "p = rho * P * dt / (F * M)\nk = exp(-p)",
    {"rho": ("melt.density", "kg m-3", "density of the melt", "melt_density"),
     "P": ("melt.production_rate", "m3 s-1", "melt made, volume per second", "production"),
     "dt": ("time.step", "s", "length of the step", "step"),
     "F": ("melt.zone_mean_fraction", "1", "mean molten share of the rock that melted", "mean_fraction"),
     "M": ("interior.mantle_mass", "kg", "mass of the mantle", "mantle_mass")},
    {"p": ("melt.processed_share", "1", "share of the mantle that passed through the melting zone this step", "processed"),
     "k": ("melt.kept_factor", "1", "share of the mantle's fertile rock, water and heat-making elements left behind", "kept")},
    {"text": "Morschhauser, A., Grott, M. and Breuer, D. 2011, Icarus 212, 541", "doi": "10.1016/j.icarus.2010.12.028",
     "where": "mantle depletion and dehydration by crust production: the mass that melted over its mean melt fraction is the mass processed"},
    [MORSCH],
    "Melt carries away what rock gives up most easily: its basalt, its water and its radioactive elements. A world that melts a lot dries out and runs low on fuel for more melt; one that barely melts stays wet.",
    "Mass of mantle passed through the melt zone in a step is the melt mass over the mean melt fraction. Fertile share, water (incompatible almost wholly) and U, Th and K are each multiplied by exp(−processed) per step, which conserves mass and keeps them positive. This replaces a fixed degassing time.",
    [ex({"rho": 2800.0, "P": 1.0, "dt": 3.156e13, "F": 0.1, "M": 4.9e23}, {"p": pr[0], "k": pr[1]})],
    positive=["F", "M"]))

# ------------------------------------------------------- erupted share
def erupted(P, A, f=0.2):
    return f * P, (1.0 - f) * P, f * P / A

e1 = erupted(52.0, 4.17e13)
blocks.append(block(
    "sse.melt.erupted_share", "How much melt reaches the surface, and how fast it buries it",
    "Π_erupted = f_ext Π;  Π_intruded = (1 − f_ext) Π;  v = f_ext Π / A",
    "Pe = f * P\nPi = (1 - f) * P\nv = f * P / A",
    {"P": ("melt.production_rate", "m3 s-1", "melt made, volume per second", "production"),
     "A": ("body.surface_area", "m2", "area of the world's surface", "surface_area"),
     "f": ("model.erupted_share", "1", "share of the melt that reaches the surface", "")},
    {"Pe": ("melt.erupted_rate", "m3 s-1", "melt erupted at the surface, volume per second", "erupted"),
     "Pi": ("melt.intruded_rate", "m3 s-1", "melt frozen inside the lid, volume per second", "intruded"),
     "v": ("heat.burial_speed", "m s-1", "how fast erupted lava buries the surface", "burial_speed")},
    {"text": "White, S. M., Crisp, J. A. and Spera, F. J. 2006, Geochemistry, Geophysics, Geosystems 7, Q03010", "doi": "10.1029/2005GC001002",
     "where": "intrusive to extrusive ratios of about 5 to 1 in long-term magma budgets; one editable share in the fudge register until the magma's own head decides it (HF8)"},
    [WHITE, MOORE_WEBB],
    "Most melt never reaches the surface: it freezes on the way up, inside the lid. About one part in five erupts as lava, and that lava slowly buries the ground below it.",
    "Erupted and intruded melt volumes from one share f_ext (about 0.2; White et al. 2006), registered as a fudge: the physics that should set it is the melt's buoyancy against the column it rises through. The erupted volume over the surface area is the burial speed that chokes the lid's conduction.",
    [ex({"P": 52.0, "A": 4.17e13}, {"Pe": e1[0], "Pi": e1[1], "v": e1[2]})],
    params=[("erupted_share", "f", 0.2, [0.0, 1.0], "share of the melt that reaches the surface (White et al. 2006: about 1 in 5)")],
    positive=["A"]))

# ----------------------------------------------------------- heat carried
def heat_carried(rho, Pe, Pi, L, c, Tm, Tl, Ts):
    hm = rho * (Pe + Pi) * (L + c * (Tm - Tl))
    he = rho * Pe * (L + c * (Tm - Ts))
    hi = rho * Pi * (L + c * (Tm - Tl))
    return hm, he, hi

hc = heat_carried(2800.0, 10.0, 42.0, 4.0e5, 1200.0, 1700.0, 1520.0, 220.0)
blocks.append(block(
    "sse.melt.heat_carried", "Heat that melt carries out of a world's mantle",
    "mantle: ρ Π (L + c(T_m − T_l));  surface: ρ Π_e (L + c(T_m − T_s));  lid: ρ Π_i (L + c(T_m − T_l))",
    "hm = rho * (Pe + Pi) * (L + c * (Tm - Tl))\nhe = rho * Pe * (L + c * (Tm - Ts))\nhi = rho * Pi * (L + c * (Tm - Tl))",
    {"rho": ("melt.density", "kg m-3", "density of the melt", "melt_density"),
     "Pe": ("melt.erupted_rate", "m3 s-1", "melt erupted at the surface", "erupted"),
     "Pi": ("melt.intruded_rate", "m3 s-1", "melt frozen inside the lid", "intruded"),
     "L": ("range.latent_fusion", "J kg-1", "heat it takes to melt a kilogram", "latent_heat"),
     "c": ("layer.specific_heat", "J kg-1 K-1", "heat it takes to warm a kilogram by one kelvin", "heat_capacity"),
     "Tm": ("interior.mantle_temperature", "K", "temperature of the convecting mantle", "mantle_temperature"),
     "Tl": ("layer.base_temperature", "K", "temperature at the base of the lid", "lid_base_temperature"),
     "Ts": ("layer.top_temperature", "K", "temperature at the surface", "surface_temperature")},
    {"hm": ("heat.melt_mantle_loss", "W", "heat the mantle loses to the melt", "mantle_loss"),
     "he": ("heat.melt_erupted", "W", "heat erupted lava gives the surface", "erupted_heat"),
     "hi": ("heat.melt_intruded", "W", "heat frozen melt gives the lid", "intruded_heat")},
    {"text": "Moore, W. B. and Webb, A. A. G. 2013, Nature 501, 501", "doi": "10.1038/nature12473",
     "where": "the heat-pipe balance: melt carries its latent heat and its warmth above the ground it erupts onto; the split between surface and lid as in the melt design's section 3.8"},
    [MOORE_WEBB, OREILLY],
    "Molten rock is a hot-water bottle on the move: it takes its heat up with it. Lava gives all of it to the surface; melt that freezes underground warms the lid from inside, and that heat leaks out later.",
    "The rows of the melt design's heat table (3.8) for one step. The mantle loses the melt's latent heat plus its warmth above the lid's base; erupted melt delivers latent heat plus its warmth above the surface temperature (the extra c(T_l − T_s) is the lid's buried cooling); intruded melt delivers latent heat plus warmth above the lid's base to the lid. A steady heat pipe's surface total equals the mantle's loss.",
    [ex({"rho": 2800.0, "Pe": 10.0, "Pi": 42.0, "L": 4.0e5, "c": 1200.0, "Tm": 1700.0, "Tl": 1520.0, "Ts": 220.0},
        {"hm": hc[0], "he": hc[1], "hi": hc[2]})],
    positive=["rho"]))

# -------------------------------------------------------- Andrade response
def gamma1p(al):
    z = al + 3.0
    lg = (z - 0.5) * math.log(z) - z + math.log(2 * math.pi) / 2 + 1 / (12 * z) - 1 / (360 * z ** 3) + 1 / (1260 * z ** 5)
    return math.exp(lg) / ((al + 1) * (al + 2))


def andrade(mu, eta, rho, g, R, w, al, s=1.0):
    X = w * eta / mu
    Y = s * gamma1p(al) * X ** (-al)
    j1 = 1 + Y * math.cos(al * math.pi / 2)
    j2 = 1 / X + Y * math.sin(al * math.pi / 2)
    A = 19 * mu / (2 * rho * g * R)
    m2 = j1 * j1 + j2 * j2
    u = 1 + A * j1 / m2
    v = A * j2 / m2
    dd = u * u + v * v
    return 1.5 * v / dd, 1.5 * u / dd


def maxwell(mu, eta, rho, g, R, w):
    A = 19 * mu / (2 * rho * g * R)
    X = w * eta / mu
    D = 1 + (1 + A) ** 2 * X * X
    return 1.5 * A * X / D, 1.5 * (1 + (1 + A) * X * X) / D

assert abs(gamma1p(0.3) - math.gamma(1.3)) < 1e-6 * math.gamma(1.3)
mx = maxwell(6.5e10, 1.0e13, 3530.0, 1.796, 1821600.0, 4.11e-5)
an0 = andrade(6.5e10, 1.0e13, 3530.0, 1.796, 1821600.0, 4.11e-5, 0.3, 0.0)
assert all(abs(a - b) < 1e-12 * abs(b) for a, b in zip(an0, mx)), (an0, mx)
an = andrade(6.5e10, 1.0e19, 3530.0, 1.796, 1821600.0, 4.11e-5, 0.3)
blocks.append(block(
    "sse.tides.andrade_response", "How much a world gives and lags under a tide, with the rock's slow creep",
    "J = 1/μ − i/(ηω) + s μ^(α−1) η^(−α) (iω)^(−α) Γ(1+α);  k₂ = (3/2) / (1 + 19/(2ρgR J))",
    "X = w * eta / mu\nz = al + 3\nq = z * z\nlg = (z - 1/2) * ln(z) - z + ln(2 * pi) / 2 + (1 - (1 - 2 / (7 * q)) / (30 * q)) / (12 * z)\nY = s * exp(lg) / ((z - 2) * (z - 1)) * X ^ (-al)\nh = al * pi / 2\nj1 = 1 + Y * cos(h)\nj2 = 1 / X + Y * sin(h)\nB = 19 * mu / (2 * rho * g * R) / (j1 * j1 + j2 * j2)\nu = 1 + B * j1\nv = B * j2\ne = 3/2 / (u * u + v * v)\nki = e * v\nkr = e * u",
    {"mu": ("interior.rigidity", "kg m-1 s-2", "how stiff the world's rock or ice is (shear modulus)", "rigidity"),
     "eta": ("interior.viscosity", "kg m-1 s-1", "how slowly the world's insides flow", "viscosity"),
     "rho": ("body.bulk_density", "kg m-3", "mean density", "density"),
     "g": ("body.surface_gravity", "m s-2", "gravity at the surface", "gravity"),
     "R": ("body.radius", "m", "radius of the world", "radius"),
     "w": ("tides.frequency", "s-1", "how fast the tide flexes it (the orbit's mean motion for a locked moon)", "frequency"),
     "al": ("interior.andrade_exponent", "1", "how the rock's slow creep spreads over time scales (Andrade exponent)", "andrade_exponent"),
     "s": ("model.transient_creep", "1", "the rock's slow creep: 1 on, 0 off (a Maxwell body)", "")},
    {"ki": ("tides.im_k2", "1", "share of the tide's work turned to heat (Im k2, the k2/Q of the heating law)", "im_k2"),
     "kr": ("tides.re_k2", "1", "how far the world's shape follows the tide (Re k2)", "re_k2")},
    {"text": "Castillo-Rogez, J. C., Efroimsky, M. and Lainey, V. 2011, JGR 116, E09008", "doi": "10.1029/2010JE003664",
     "where": "the Andrade compliance J = 1/μ − i/(ηω) + μ^(α−1) η^(−α) (iω)^(−α) Γ(1+α), in the homogeneous incompressible body's Love number (Efroimsky 2012); Γ(1+α) by Stirling's series from Γ(α+3), good to a part in 10⁶"},
    [CASTILLO, EFROIMSKY, HENNING],
    "Rock is not only springy and runny: it also creeps slowly in between, the way a shelf sags under books. That slow creep turns a little of every tide into heat, even when the rock is far too stiff to flow, so cold worlds warm a little and warm ones a lot more than the simplest model says.",
    "Uniform incompressible body with an Andrade rheology: Maxwell's elastic and viscous compliances plus a transient term μ^(α−1) η^(−α) (iω)^(−α) Γ(1+α), its strength tied to the viscosity as in Castillo-Rogez et al. 2011. Written as real and imaginary parts. With s = 0 it is the Maxwell block sse.tides.maxwell_response exactly. A layered world needs the layered block.",
    [ex({"mu": 6.5e10, "eta": 1.0e19, "rho": 3530.0, "g": 1.796, "R": 1821600.0, "w": 4.11e-5, "al": 0.3}, {"ki": an[0], "kr": an[1]}),
     ex({"mu": 6.5e10, "eta": 1.0e13, "rho": 3530.0, "g": 1.796, "R": 1821600.0, "w": 4.11e-5, "al": 0.3, "s": 0.0}, {"ki": an0[0], "kr": an0[1]})],
    params=[("transient_creep", "s", 1.0, [0.0, 1.0], "the rock's slow (Andrade) creep: 1 on, 0 off, which gives the Maxwell body")],
    positive=["mu", "eta", "rho", "g", "R", "w"]))

# ---------------------------------------------------- host power ceiling
def ceiling(n, kq, m, Rh, a):
    return n * 1.5 * kq * G * m * m * Rh ** 5 / a ** 6

ce = ceiling(4.11e-5, 1.1e-5, 8.93e22, 7.0e7, 4.217e8)
blocks.append(block(
    "sse.tides.host_power_ceiling", "The most heat a host's spin can feed a moon held in step",
    "P_max = n (3/2) (k2/Q)_host G m² R_host⁵ / a⁶",
    "r5 = Rh * Rh * Rh * Rh * Rh\na6 = a * a * a * a * a * a\nP = n * 3/2 * kq * G * m * m * r5 / a6",
    {"n": ("tides.frequency", "s-1", "the moon's orbital rate (mean motion)", "mean_motion"),
     "kq": ("tides.host_k2_over_q", "1", "how strongly the host flexes and loses energy under the moon's tide (its k2/Q)", "host_k2_over_q"),
     "m": ("body.mass", "kg", "mass of the moon", "mass"),
     "Rh": ("tides.host_radius", "m", "radius of the host", "host_radius"),
     "a": ("orbit.semi_major_axis", "m", "size of the moon's orbit", "semi_major_axis"),
     "G": ("constant.G", "m3 kg-1 s-2", "gravitational constant", "")},
    {"P": ("tides.host_power_ceiling", "W", "the most heat the host's tide can keep feeding the moon", "ceiling")},
    {"text": "Murray, C. D. and Dermott, S. F. 1999, Solar System Dynamics, Cambridge University Press", "doi": "10.1017/CBO9781139174817",
     "where": "the tidal torque of a satellite on its planet (section 4.9) times the orbital rate: the power the host's spin passes into the orbit, the most a moon held in resonance can dissipate for long; Jupiter's k2/Q from Lainey et al. 2009"},
    [MURRAY, LAINEY09],
    "A moon's tidal heat is paid for by its host's spin. The host's own tide passes energy to the moon's orbit, and for a moon held in step by its neighbours that is the most it can turn into heat for long: Io gives off about nine tenths of what Jupiter can feed it.",
    "The tidal torque (3/2)(k2/Q)_host G m² R_host⁵ / a⁶ times the mean motion n. For a moon in a resonance that pumps its eccentricity, its heat is set mostly by the host, and the moon's interior decides the eccentricity it settles at. A check and an explanation, never a clamp.",
    [ex({"n": 4.11e-5, "kq": 1.1e-5, "m": 8.93e22, "Rh": 7.0e7, "a": 4.217e8}, {"P": ce})],
    constants={"G": "constants/G"},
    positive=["n", "kq", "m", "Rh", "a"]))


def main():
    doc = {"format": "sse.formulas/1", "family": "sse.melt", "about": ABOUT, "atoms": blocks}
    text = json.dumps(doc, indent=1, ensure_ascii=False) + "\n"
    if "--check" in sys.argv:
        if OUT.read_text(encoding="utf-8") != text:
            print(f"{OUT} is stale: run python3 scripts/build-melt-blocks.py")
            sys.exit(1)
        print("melt blocks are current")
        return
    OUT.write_text(text, encoding="utf-8")
    print(f"wrote {OUT}: {len(blocks)} blocks")


if __name__ == "__main__":
    main()
