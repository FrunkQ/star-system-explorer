"""Melting inside worlds: the rock rows the melt and lid blocks read (track HF,
package HF4b.0; design handover/design/SSE4-melt-and-heat-design.md section 8.1).

Rows added to two mixtures:
  mantle-rock  phases (solidus and liquidus fits, latent heat of fusion, the
               solidus rise of depleted rock), rigidity, creep (reference
               viscosity, temperature and activation energy, the dry factor,
               melt weakening, Andrade exponent), melt (lock-up share, grain
               size, permeability law) and thermal (conductivity, diffusivity,
               heat capacity, expansivity)
  basalt       liquid density, liquid viscosity and solid density

Every value here was written from memory on 26 Nov 2026, while the paper
search was out (until 1 Dec 2026): all rows are "unverified" until read at
source. Values that the thermal history already used as typed constants
(core.rs CoreSettings) keep exactly those values, so moving the reads to the
sheet changes no result.
"""

U = "unverified"

SOURCES = {
    "hirschmann2000": ("Hirschmann, M. M. 2000, Geochemistry, Geophysics, Geosystems 1, 1042: mantle solidus, experimental constraints and the effects of peridotite composition (dry peridotite solidus fit, 0 to 10 GPa)", "10.1029/2000GC000070", None, U),
    "katz2003": ("Katz, R. F., Spiegelman, M. and Langmuir, C. H. 2003, Geochemistry, Geophysics, Geosystems 4, 1073: a new parameterization of hydrous mantle melting (lherzolite liquidus fit)", "10.1029/2002GC000433", None, U),
    "andrault2011": ("Andrault, D. et al. 2011, Earth and Planetary Science Letters 304, 251: solidus and liquidus profiles of chondritic mantle (deep liquidus, Simon form)", "10.1016/j.epsl.2011.02.006", None, U),
    "morschhauser2011": ("Morschhauser, A., Grott, M. and Breuer, D. 2011, Icarus 212, 541: crustal recycling, mantle dehydration and the thermal evolution of Mars (solidus rise of depleted mantle)", "10.1016/j.icarus.2010.12.028", None, U),
    "hirth-kohlstedt2003": ("Hirth, G. and Kohlstedt, D. 2003, in Inside the Subduction Factory, AGU Geophysical Monograph 138, 83: rheology of the upper mantle and the mantle wedge (creep laws, water weakening)", "10.1029/138GM06", None, U),
    "mei2002": ("Mei, S., Bai, W., Hiraga, T. and Kohlstedt, D. L. 2002, Earth and Planetary Science Letters 201, 491: influence of melt on the creep behaviour of olivine-basalt aggregates (melt weakening exponent)", "10.1016/S0012-821X(02)00722-0", None, U),
    "jackson-faul2010": ("Jackson, I. and Faul, U. H. 2010, Physics of the Earth and Planetary Interiors 183, 151: grainsize-sensitive viscoelastic relaxation in olivine (Andrade exponent)", "10.1016/j.pepi.2010.09.005", None, U),
    "costa2009": ("Costa, A., Caricchi, L. and Bagdassarov, N. 2009, Geochemistry, Geophysics, Geosystems 10, Q03010: a model for the rheology of particle-bearing suspensions and partially molten rocks (rheological lock-up)", "10.1029/2008GC002138", None, U),
    "bierson-nimmo2016": ("Bierson, C. J. and Nimmo, F. 2016, JGR Planets 121, 2211: a test for Io's magma ocean, modeling tidal dissipation with a partially molten mantle (grain size, porous flow)", "10.1002/2016JE005005", None, U),
    "miller2014": ("Miller, K. J., Zhu, W., Montesi, L. G. J. and Gaetani, G. A. 2014, Earth and Planetary Science Letters 388, 273: experimental quantification of permeability of partially molten mantle rock (k = d^2 phi^n / C)", "10.1016/j.epsl.2013.12.003", None, U),
    "stolper-walker1980": ("Stolper, E. and Walker, D. 1980, Contributions to Mineralogy and Petrology 74, 7: melt density and the average composition of basalt", "10.1007/BF00375484", None, U),
    "giordano2008": ("Giordano, D., Russell, J. K. and Dingwell, D. B. 2008, Earth and Planetary Science Letters 271, 123: viscosity of magmatic liquids, a model", "10.1016/j.epsl.2008.03.038", None, U),
}


def apply(mixtures, val):
    """Add the melt rows to mantle-rock and basalt, in place."""
    by_id = {m["id"]: m for m in mixtures}
    mr = by_id["mantle-rock"]
    mr["phases"] = {
        "solidus": {"fit": "T = a P^2 + b P + c, T in degC, P in GPa", "coeffs": [-5.104, 132.899, 1120.661],
                    "valid_gpa": [0.0, 10.0], "src": "hirschmann2000", "check": U, "note": "dry peridotite"},
        "liquidus": {"fit": "T = a P^2 + b P + c, T in degC, P in GPa", "coeffs": [-2.0, 45.0, 1780.0],
                     "valid_gpa": [0.0, 10.0], "src": "katz2003", "check": U, "note": "dry lherzolite"},
        "liquidus_deep": {"fit": "T = T0 (P / P0 + 1)^(1 / c), T in K, P in GPa", "coeffs": [1940.0, 29.0, 1.9],
                          "src": "andrault2011", "check": U, "note": "chondritic mantle; the base of a magma ocean"},
        "latent_fusion": val(4.0e5, "J kg-1", "turcotte2014", U, note="6e5 is also quoted"),
        "solidus_rise_depleted": val(150.0, "K", "morschhauser2011", U, note="at 20% melt taken"),
    }
    mr["rigidity"] = val(6.5e10, "Pa", "turcotte2014", U, note="shear modulus of the upper mantle")
    mr["creep"] = {
        "viscosity_ref": val(1.0e21, "Pa s", "hirth-kohlstedt2003", U, note="wet upper mantle at t_ref; calibrated on Earth"),
        "t_ref": val(1600.0, "K", "hirth-kohlstedt2003", U),
        "activation": val(3.0e5, "J mol-1", "hirth-kohlstedt2003", U),
        "dry_factor": val(100.0, "1", "hirth-kohlstedt2003", U, note="dry over wet; water weakens olivine 10 to 100 times"),
        "melt_weakening": val(26.0, "1", "mei2002", U, note="viscosity falls as exp(-a phi)"),
        "andrade_exponent": val(0.3, "1", "jackson-faul2010", U),
    }
    mr["melt"] = {
        "lockup_share": val(0.4, "1", "costa2009", U, note="melt share above which the mush flows as a liquid"),
        "grain_size": val(1.0e-2, "m", "bierson-nimmo2016", U, note="1 mm to 1 cm argued"),
        "permeability_exponent": val(2.6, "1", "miller2014", U),
        "permeability_constant": val(58.0, "1", "miller2014", U, note="k = d^2 phi^n / C"),
    }
    th = mr.setdefault("thermal", {})
    th["conductivity"] = val(3.0, "W m-1 K-1", "turcotte2014", U)
    th["diffusivity"] = val(1.0e-6, "m2 s-1", "turcotte2014", U)
    th["heat_capacity"] = val(1200.0, "J kg-1 K-1", "turcotte2014", U)
    th["expansivity"] = val(1.5e-5, "K-1", "turcotte2014", U)
    b = by_id["basalt"]
    b["density"] = val(2900.0, "kg m-3", "turcotte2014", U)
    b["density_liquid"] = val(2800.0, "kg m-3", "stolper-walker1980", U)
    b["viscosity_liquid"] = val(10.0, "Pa s", "giordano2008", U, note="10 to 100 Pa s")
