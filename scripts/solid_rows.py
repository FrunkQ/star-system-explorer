"""Solid rocket propellants and RP-1's heating value on the master data sheet
(asked for by the real spaceflight thread, track RS, for the engines in
store-packs/sol-today/parts.json, 24 Nov 2026).

The solids are bulk mixtures: their ingredients (ammonium perchlorate, the
PBAN and HTPB binders) are not sheet substances yet, so the make-up by mass is
given as `make_up` in plain words beside an empty recipe. Values were read
from the fetched text of the reports on 24 Nov 2026 and are unverified until
a page-image check. Percent and g/cm3 are as printed and converted here.
"""

SOURCES = {
    "jpl-79-29": ("Anderson, F. A. and West, W. R. 1979, Alternate propellant program phase I final report, JPL Publication 79-29 (NASA-CR-158815): TP-H1148 (Shuttle SRB) 69.60% ammonium perchlorate, 16.00% aluminium, 14.00% PBAN binder, 0.40% iron oxide, cured with an epoxy (Table 2-1); Shuttle PBAN baseline density 1.7714 g/cm3 (Table 3-1); theoretical Isp 262.3 s at sea level (1000 psia, optimum) and 276.7 s in vacuum (expansion ratio 7.16), flame temperature 3471 K (Table 2-2)", None, "https://ntrs.nasa.gov/api/citations/19790020174/downloads/19790020174.pdf", "unverified"),
    "kibbey-star48": ("Kibbey, T. P., New solid propellant additives: evaluation for NASA mission use cases, NASA Marshall Space Flight Center (JANNAF), Table 1 (STAR-48BV): aluminium 18% by mass, polymer 22% by volume, theoretical bulk density 1.813 g/cm3, propellant Isp 294.2 s at expansion ratio 54.8", None, "https://ntrs.nasa.gov/api/citations/20240015535/downloads/JANNAF%209871%20Propellants%20for%20Space%20Kibbey.pdf", "unverified"),
    "nbs-tn937": ("Nuttall, R. L. and Armstrong, G. T. 1977, Estimation of net enthalpies of combustion of some aviation fuels expressed in SI units, NBS Technical Note 937, Table 2: kerosine mean net enthalpy of combustion 43.3020 MJ/kg (18616.3 Btu/lb)", None, "https://nvlpubs.nist.gov/nistpubs/Legacy/TN/nbstechnicalnote937.pdf", "unverified"),
}


def rows(val):
    U = "unverified"
    pban = {
        "id": "pban-ap-al", "names": {"physics": "PBAN composite propellant (TP-H1148, Shuttle SRB)", "plain": "Shuttle booster propellant", "story": "booster propellant"},
        "recipe": {}, "by": "mass",
        "make_up": {"ammonium perchlorate": val(0.696, "1", "jpl-79-29", U, note="69.60%"),
                    "aluminium": val(0.16, "1", "jpl-79-29", U, note="16.00%"),
                    "PBAN binder with epoxy cure": val(0.14, "1", "jpl-79-29", U, note="14.00%"),
                    "iron oxide catalyst": val(0.004, "1", "jpl-79-29", U, note="0.40%")},
        "density": val(1771.4, "kg m-3", "jpl-79-29", U, note="1.7714 g/cm3, the Shuttle PBAN baseline (86% solids, 16% aluminium)"),
        "flame_temperature_k": val(3471.0, "K", "jpl-79-29", U, at_pa=6.9e6, note="theoretical, adiabatic, at 1000 psia; Table 3-1 gives 3515 K"),
        "isp_theoretical": val(276.7, "s (Isp)", "jpl-79-29", U, note="vacuum, expansion ratio 7.16; 262.3 s at sea level, optimum expansion from 1000 psia"),
        "note": "the solid booster propellant of the Space Shuttle (and its RSRM successor)",
    }
    htpb = {
        "id": "htpb-ap-al", "names": {"physics": "HTPB composite propellant (ammonium perchlorate, aluminium)", "plain": "solid rocket propellant", "story": "solid propellant"},
        "recipe": {}, "by": "mass",
        "make_up": {"aluminium": val(0.18, "1", "kibbey-star48", U, note="18%; the ammonium perchlorate share is not printed"),},
        "density": val(1813.0, "kg m-3", "kibbey-star48", U, note="theoretical bulk density, 1.813 g/cm3 (STAR-48BV)"),
        "isp_theoretical": val(294.2, "s (Isp)", "kibbey-star48", U, note="the STAR-48BV propellant at expansion ratio 54.8"),
        "note": "the common upper-stage and kick-motor solid (the STAR motor family); polymer is 22% by volume",
    }
    for m in (pban, htpb):
        assert sum(x["v"] for x in m["make_up"].values()) <= 1.0 + 1e-9, m["id"]
    rp1_add = {"heating_value_net": val(round(43.3020 * 1e6, 0), "J kg-1", "nbs-tn937", U, note="the kerosine mean of NBS TN 937 (43.3020 MJ/kg net), not a measurement of RP-1 itself")}
    return [pban, htpb], rp1_add
