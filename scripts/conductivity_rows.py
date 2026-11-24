"""Electrical conductivity for the magnetic field ledger (package MP30, asked by
proven core for MF2: the ledger reads eta = 1/(mu0 sigma) from the sheet).

Three bulk mixtures and one field on the iron row:
  earth-core-liquid  liquid iron with silicon and oxygen at Earth's core-mantle
                     and inner-core boundaries (Pozzo et al. 2012, Table 1)
  iron               sigma_elec_liquid: pure liquid iron at the same conditions
  giant-envelope     Jupiter's hydrogen-helium envelope (model J11-8a): a
                     profile of sigma against depth, pressure, density and
                     temperature, and where conduction turns metal-like
                     (French et al. 2012, Table 2 and section 6.2)
  seawater           standard seawater at practical salinity 35 (PSS-78:
                     C(35,15,0) as Sea-Bird and Culkin and Smith print it, the
                     temperature ratio r_T and the salinity definition)

All values were read on 24 Nov 2026 from fetched text and are unverified until
checked against the papers. Pressures are printed in GPa and turned to Pa here;
densities in g/cm3 to kg/m3; mS/cm to S/m.
"""

SOURCES = {
    "pozzo2012": ("Pozzo, M., Davies, C., Gubbins, D. and Alfe, D. 2012, Nature 485, 355: thermal and electrical conductivity of iron at Earth's core conditions (arXiv:1203.4970), Table 1: sigma at the inner-core and core-mantle boundaries, Fe with 10 % Si and 8 % O (pure Fe in parentheses), for a density jump of 0.6 g/cm3", "10.1038/nature11031", "https://arxiv.org/abs/1203.4970", "unverified"),
    "french2012": ("French, M. et al. 2012, ApJS 202, 5: ab initio simulations for material properties along the Jupiter adiabat (model J11-8a), Table 2 (linear transport properties) with Table 1 (thermodynamic properties); section 6.2: metal-like conduction (sigma >= 2e4 S/m) from about 0.9 Jupiter radii, 0.5 Mbar", "10.1088/0067-0049/202/1/5", None, "unverified"),
    "unesco37": ("UNESCO 1981, Background papers and supporting data on the Practical Salinity Scale 1978, Unesco Technical Papers in Marine Science 37 (Joint Panel on Oceanographic Tables and Standards): practical salinity S = 0.0080 - 0.1692 K^0.5 + 25.3851 K + 14.0941 K^1.5 - 7.0261 K^2 + 2.7081 K^2.5, K = C(S,15,0)/C(KCl,15,0), valid for S 2 to 42; Culkin and Smith's C(35,15,0) = 42.914 mmho/cm", None, "https://www.vliz.be/imisdocs/publications/ocrd/270529.pdf", "unverified"),
    "lewis1980": ("Lewis, E. L. 1980, IEEE J. Oceanic Eng. OE-5, 3: the Practical Salinity Scale 1978 and its antecedents, eq. 8: r_T = C(35,T,0)/C(35,15,0) = c0 + c1 T + c2 T^2 + c3 T^3 + c4 T^4, T in deg C (c0..c4 = 6.766097e-1, 2.00564e-2, 1.104259e-4, -6.9698e-7, 1.0031e-9)", None, "https://www.vliz.be/imisdocs/publications/ocrd/29440.pdf", "unverified"),
    "seabird-an14": ("Sea-Bird Electronics, Application note 14: 1978 Practical Salinity Scale (rev. January 1989): C(35,15,0) = 42.914 mmho/cm (Culkin and Smith, UNESCO 37 p. 23; Poisson used 42.933); mmho/cm equal mS/cm", None, "https://pscfiles.apl.uw.edu/woodgate/BeringStraitArchive/BeringStraitMooringData/BeringStraitMoorings2007to2009IPY_versionMar10/SeasoftForWavesProcessingSoftware/website/application_notes/AN14.htm", "unverified"),
}

GPA = 1e9
# French et al. 2012 Tables 1 and 2, model J11-8a: R (Jupiter radii), rho g/cm3, T K, P GPa, sigma S/m.
JUPITER = [
    (0.0923, 4.42, 19500.0, 4180.0, 3.39e6),
    (0.196, 3.99, 18000.0, 3410.0, 3.05e6),
    (0.350, 3.39, 16000.0, 2460.0, 2.48e6),
    (0.478, 2.80, 14000.0, 1640.0, 1.97e6),
    (0.584, 2.26, 12000.0, 1030.0, 1.48e6),
    (0.680, 1.60, 10000.0, 600.0, 1.09e6),
    (0.770, 1.20, 8000.0, 300.0, 7.20e5),
    (0.852, 0.824, 6000.0, 120.0, 3.60e5),
    (0.890, 0.638, 5000.0, 64.0, 1.46e5),
]
# The next shallower Table 2 row (R 0.930, 23 GPa) has sigma 1.5e3 S/m.
JUPITER_SHALLOW = (0.930, 23.0, 1.5e3)
R_T = (6.766097e-1, 2.00564e-2, 1.104259e-4, -6.9698e-7, 1.0031e-9)
PSS78 = (0.0080, -0.1692, 25.3851, 14.0941, -7.0261, 2.7081)
C35_15_MS_CM = 42.914


def r_t(t_c):
    return sum(c * t_c ** k for k, c in enumerate(R_T))


def apply(mixtures, val, t0c):
    """Add the rows; return the new mixtures (iron is updated in place)."""
    U = "unverified"
    by_id = {m["id"]: m for m in mixtures}
    new = []

    def row(mid, physics, plain, story, note):
        r = {"id": mid, "names": {"physics": physics, "plain": plain, "story": story}, "recipe": {}, "by": "mass", "note": note}
        new.append(r)
        return r

    core = row("earth-core-liquid", "liquid iron alloy with silicon and oxygen (Earth's outer core)", "liquid core iron", "molten core",
               "Pozzo et al.'s light-element case, 10 % Si and 8 % O; its conductivity changes only about 13 % from top to bottom of the core")
    core["sigma_elec"] = val(1.11e6, "S m-1", "pozzo2012", U, at_k=4186.0, at_place="Earth's core-mantle boundary", note="Table 1, density jump 0.6 g/cm3; first-principles, 2 to 3 times the estimates used before")
    core["sigma_elec_icb"] = val(1.25e6, "S m-1", "pozzo2012", U, at_k=5700.0, at_place="Earth's inner-core boundary", note="Table 1, density jump 0.6 g/cm3")
    fe = by_id["iron"]
    fe["sigma_elec_liquid"] = val(1.36e6, "S m-1", "pozzo2012", U, at_place="Earth's core-mantle boundary", note="pure liquid iron, Table 1 in parentheses; 1.56e6 at the inner-core boundary")

    env = row("giant-envelope", "hydrogen-helium fluid of a gas giant's interior", "gas giant's deep fluid", "the giant's deep sea of hydrogen",
              "along Jupiter's adiabat, model J11-8a; hydrogen turns metal-like with depth, with no sharp edge")
    env["sigma_elec_profile"] = {
        "src": "french2012", "check": U, "model": "Jupiter J11-8a",
        "columns": ["r_rj", "density", "t_k", "p_pa", "sigma_elec"],
        "units": ["Jupiter radii", "kg m-3", "K", "Pa", "S m-1"],
        "rows": [[r, round(rho * 1000.0, 1), t, p * GPA, s] for (r, rho, t, p, s) in JUPITER]
                + [[JUPITER_SHALLOW[0], None, None, JUPITER_SHALLOW[1] * GPA, JUPITER_SHALLOW[2]]],
        "note": "from the centre outwards; the deepest row is at 0.0923 Jupiter radii; the 0.930 row prints no density or temperature in Table 2",
    }
    env["metallic_from"] = {"sigma_elec": val(2e4, "S m-1", "french2012", U), "p_pa": val(0.5e11, "Pa", "french2012", U, note="0.5 Mbar, about 0.9 Jupiter radii (section 6.2)")}
    ps = [r[3] for r in JUPITER]
    assert ps == sorted(ps, reverse=True)

    sea = row("seawater", "standard seawater, practical salinity 35", "seawater", "sea water", "conductivity rises with salt and with warmth")
    sea["salinity_practical"] = val(35.0, "1", "unesco37", U, note="PSS-78 salinity is a pure number from a conductivity ratio")
    s15 = round(C35_15_MS_CM * 0.1, 5)  # 1 mS/cm = 0.1 S/m
    sea["sigma_elec"] = val(s15, "S m-1", "seabird-an14", U, at_k=round(15.0 + t0c, 2), at_place="sea surface", note="C(35,15,0) = 42.914 mmho/cm (Culkin and Smith); Poisson's 42.933 differs by 0.04 %", also=["unesco37"])
    sea["sigma_elec_25c"] = val(round(s15 * r_t(25.0), 4), "S m-1", "lewis1980", U, at_k=round(25.0 + t0c, 2), at_place="sea surface", note="C(35,15,0) times r_T(25 C) = %.5f from eq. 8" % r_t(25.0))
    sea["sigma_elec_t_fit"] = {"fit": "C(35,T,0)/C(35,15,0) = c0 + c1 T + c2 T^2 + c3 T^3 + c4 T^4, T in deg C", "coeffs": list(R_T), "range_c": [-2.0, 35.0], "src": "lewis1980", "check": U}
    sea["salinity_fit"] = {"fit": "S = a0 + a1 K^0.5 + a2 K + a3 K^1.5 + a4 K^2 + a5 K^2.5, K = C(S,15,0)/C(KCl,15,0)", "coeffs": list(PSS78), "range": [2.0, 42.0], "src": "unesco37", "check": U}
    assert abs(r_t(15.0) - 1.0) < 1e-4 and abs(sum(PSS78) - 35.0) < 1e-3
    return new
