"""Rocket and thruster propellants on the master data sheet (track RK, package
RK1 part 1; Atomic Rockets plan section 6): storable and cryogenic propellants
as substances with boiling and freezing points, liquid density and heat of
formation, and the critical points of the gases electric thrusters store
supercritical.

Every value was read from the source named beside it on 23 Nov 2026 (the
fetched text of the page or report; several reports are OCR scans, so every
value is marked unverified until a page-image check). Molar masses are never
typed: the sheet works them out from each formula. Gathered notes and quotes:
handover/packages/RK1-data/RK1-sources.md.
"""

SOURCES = {
    "nist-webbook-hydrazine": ("NIST Chemistry WebBook, SRD 69: hydrazine (phase change: Tboil 386.65 K, Tipton, Stone et al. 1989; Tfus 274.65 K, Lempe, Neuhaeuser et al. 1966; liquid enthalpy of formation 50.63 kJ/mol, Chase 1998)", None, "https://webbook.nist.gov/cgi/cbook.cgi?ID=C302012&Units=SI&Mask=7", "unverified"),
    "nist-webbook-mmh": ("NIST Chemistry WebBook, SRD 69: methylhydrazine (Ttriple 220.79 K, Aston, Finke et al. 1951; liquid enthalpy of formation 54.14 kJ/mol, Cole and Gilbert 1951)", None, "https://webbook.nist.gov/cgi/cbook.cgi?ID=C60344&Units=SI&Mask=7", "unverified"),
    "nist-webbook-udmh": ("NIST Chemistry WebBook, SRD 69: 1,1-dimethylhydrazine (Tboil 337.1 K, Weast and Grasselli 1989; Tfus 216 K, McMillan 1967; liquid enthalpy of formation 48.3 +- 3.6 kJ/mol, Donovan, Shomate and McBride 1960 as reanalysed by Cox and Pilcher 1970)", None, "https://webbook.nist.gov/cgi/cbook.cgi?ID=C57147&Units=SI&Mask=7", "unverified"),
    "nist-webbook-n2o4": ("NIST Chemistry WebBook, SRD 69: dinitrogen tetroxide (liquid enthalpy of formation -19.56 kJ/mol, Chase 1998, NIST-JANAF)", None, "https://webbook.nist.gov/cgi/cbook.cgi?ID=C10544726&Units=SI&Mask=7", "unverified"),
    "nist-webbook-h2o2": ("NIST Chemistry WebBook, SRD 69: hydrogen peroxide (Tfus 272.26 K, Cuthbertson, Matheson and Maass 1928)", None, "https://webbook.nist.gov/cgi/cbook.cgi?ID=C7722841&Units=SI&Mask=7", "unverified"),
    "marshall-deans2013": ("Marshall, W. M. and Deans, M. C. 2013, Recommended figures of merit for green monopropellants, NASA Glenn Research Center (NTRS 20140009573), Table 1: hydrazine 1004 kg/m3 at 300 K", None, "https://ntrs.nasa.gov/api/citations/20140009573/downloads/20140009573.pdf", "unverified"),
    "uscg-chris-dmh": ("US Coast Guard CHRIS hazardous chemical data, 1,1-dimethylhydrazine (DMH), June 1999: specific gravity 0.791 at 20 C, boiling point 336.5 K", None, "https://cameochemicals.noaa.gov/chris/DMH.pdf", "unverified"),
    "uscg-chris-nox": ("US Coast Guard CHRIS hazardous chemical data, nitrogen tetroxide (NOX), June 1999: boiling point 294 K, freezing point 262 K, specific gravity 1.45 at 20 C", None, "https://cameochemicals.noaa.gov/chris/NOX.pdf", "unverified"),
    "nasa-rp1013": ("Smith, I. D. and Dhooge, P. M. 1977, Triple point determinations of monomethylhydrazine and nitrogen tetroxide, NASA RP-1013 (JSC-12876): MMH normal boiling point 360.1 K, normal freezing point 220.7 K; N2O4 triple point 261.9 K at 18,637 Pa", None, "https://ntrs.nasa.gov/api/citations/19780005291/downloads/19780005291.pdf", "unverified"),
    "nrc-aegl2000": ("National Research Council 2000, Acute exposure guideline levels for selected airborne chemicals, volume 1, National Academies Press, chapter 3, Table 3-2: monomethylhydrazine specific gravity 0.874 at 25 C", None, "https://www.nationalacademies.org/read/10043/chapter/6", "unverified"),
    "evonik-h2o2": ("Evonik Active Oxygens, hydrogen peroxide physico-chemical properties (manufacturer's table): 100% density 1.450 g/ml at 20 C, boiling point 150 C at 1013 mbar", None, "https://www.evonik.com/en/company/businesslines/ao/products-and-services/hydrogen-peroxide/general-information/physico-chemical-properties.html", "unverified"),
    "nistir6646": ("Magee, J. W. et al. 2007, Thermophysical properties measurements and models for rocket propellant RP-1: phase I, NISTIR 6646: density about 799 kg/m3 near 23 C; the 14-component surrogate has the approximate formula C11.8H23.0", None, "https://nvlpubs.nist.gov/nistpubs/Legacy/IR/nistir6646.pdf", "unverified"),
    "setzmann-wagner1991": ("Setzmann, U. and Wagner, W. 1991, J. Phys. Chem. Ref. Data 20, 1061: a new equation of state and tables of thermodynamic properties for methane (values read from the NIST WebBook fluid tables, which use it: saturated liquid 422.36 kg/m3 at the normal boiling point, 111.67 K)", "10.1063/1.555898", None, "unverified"),
    "sifner-klomfar1994": ("Sifner, O. and Klomfar, J. 1994, J. Phys. Chem. Ref. Data 23, 63: thermodynamic properties of xenon from the triple point to 800 K (critical point 289.734 K, 5.840 MPa; normal boiling point 165.03 K)", "10.1063/1.555958", None, "unverified"),
    "theeuwes-bearman1970": ("Theeuwes, F. and Bearman, R. J. 1970, J. Chem. Thermodyn. 2, 179: the p, V, T behaviour of dense fluids, krypton (critical point 209.46 K, 55.2019 bar; read from the NIST WebBook)", "10.1016/0021-9614(70)90081-9", None, "unverified"),
    "tegeler1999": ("Tegeler, Ch., Span, R. and Wagner, W. 1999, J. Phys. Chem. Ref. Data 28, 779: a new equation of state for argon (critical point 150.687 K, 4.863 MPa; normal boiling point 87.303 K)", "10.1063/1.556037", None, "unverified"),
}


def rows(val):
    """New substance rows and additions to existing ones, built with the caller's val()."""
    U = "unverified"
    new = [
        ("n2h4", {"n": 2, "h": 4}, "hydrazine", "hydrazine", {
            "phases": {
                "boiling_point_k": val(386.65, "K", "nist-webbook-hydrazine", U, at_pa=101325.0),
                "melting_point_k": val(274.65, "K", "nist-webbook-hydrazine", U),
                "density_liquid": val(1004.0, "kg m-3", "marshall-deans2013", U, at_k=300.0),
            },
            "chemical": {"states": {"liquid": {"dhf": val(50630.0, "J mol-1", "nist-webbook-hydrazine", U, at_k=298.15)}}},
        }),
        ("mmh", {"c": 1, "h": 6, "n": 2}, "monomethylhydrazine", "MMH", {
            "phases": {
                "boiling_point_k": val(360.1, "K", "nasa-rp1013", U, at_pa=101325.0, note="NIST lists 364 K (Majer and Svoboda 1985); the two disagree by about 4 K"),
                "melting_point_k": val(220.7, "K", "nasa-rp1013", U, note="triple point 220.79 K (NIST)"),
                "density_liquid": val(874.0, "kg m-3", "nrc-aegl2000", U, at_k=298.15, note="specific gravity 0.874 at 25 C"),
            },
            "chemical": {"states": {"liquid": {"dhf": val(54140.0, "J mol-1", "nist-webbook-mmh", U, at_k=298.15)}}},
        }),
        ("udmh", {"c": 2, "h": 8, "n": 2}, "1,1-dimethylhydrazine", "UDMH", {
            "phases": {
                "boiling_point_k": val(337.1, "K", "nist-webbook-udmh", U, at_pa=101325.0, note="CHRIS gives 336.5 K"),
                "melting_point_k": val(216.0, "K", "nist-webbook-udmh", U),
                "density_liquid": val(791.0, "kg m-3", "uscg-chris-dmh", U, at_k=293.15, note="specific gravity 0.791 at 20 C"),
            },
            "chemical": {"states": {"liquid": {"dhf": val(48300.0, "J mol-1", "nist-webbook-udmh", U, at_k=298.15, note="+- 3.6 kJ/mol")}}},
        }),
        ("n2o4", {"n": 2, "o": 4}, "dinitrogen tetroxide", "nitrogen tetroxide", {
            "phases": {
                "boiling_point_k": val(294.0, "K", "uscg-chris-nox", U, at_pa=101325.0),
                "triple_point_k": val(261.9, "K", "nasa-rp1013", U),
                "triple_point_pa": val(18637.0, "Pa", "nasa-rp1013", U),
                "density_liquid": val(1450.0, "kg m-3", "uscg-chris-nox", U, at_k=293.15, note="specific gravity 1.45 at 20 C"),
            },
            "chemical": {"states": {"liquid": {"dhf": val(-19560.0, "J mol-1", "nist-webbook-n2o4", U, at_k=298.15)}}},
        }),
        ("h2o2", {"h": 2, "o": 2}, "hydrogen peroxide", "hydrogen peroxide", {
            "phases": {
                "boiling_point_k": val(423.15, "K", "evonik-h2o2", U, at_pa=101300.0, note="150 C at 1013 mbar, pure"),
                "melting_point_k": val(272.26, "K", "nist-webbook-h2o2", U),
                "density_liquid": val(1450.0, "kg m-3", "evonik-h2o2", U, at_k=293.15, note="pure (100%)"),
            },
        }),
    ]
    add = {
        "ch4": {"phases": {"boiling_point_k": val(111.67, "K", "setzmann-wagner1991", U, at_pa=101325.0)}},
        "o2": {"phases": {"boiling_point_k": val(90.188, "K", "schmidt-wagner1985", U, at_pa=101325.0)}},
        "xe": {"phases": {"critical_point_k": val(289.734, "K", "sifner-klomfar1994", U), "critical_point_pa": val(5.840e6, "Pa", "sifner-klomfar1994", U),
                          "boiling_point_k": val(165.03, "K", "sifner-klomfar1994", U, at_pa=101325.0)}},
        "kr": {"phases": {"critical_point_k": val(209.46, "K", "theeuwes-bearman1970", U), "critical_point_pa": val(5.52019e6, "Pa", "theeuwes-bearman1970", U)}},
        "ar": {"phases": {"critical_point_k": val(150.687, "K", "tegeler1999", U), "critical_point_pa": val(4.863e6, "Pa", "tegeler1999", U),
                          "boiling_point_k": val(87.303, "K", "tegeler1999", U, at_pa=101325.0)}},
    }
    mixtures = [
        {"id": "rp-1", "names": {"physics": "RP-1 rocket kerosene", "plain": "rocket kerosene", "story": "kerosene"},
         "recipe": {}, "by": "mass",
         "density": val(799.0, "kg m-3", "nistir6646", U, at_k=296.45, note="measured near 23.3 C; the report's own digits to be checked against the PDF"),
         "note": "a refinery cut, not one molecule: the report's 14-component surrogate has the approximate formula C11.8H23.0 (hydrogen to carbon 1.95)"},
    ]
    return new, add, mixtures
