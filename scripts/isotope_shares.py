"""Protosolar isotope shares on the master data sheet's isotope rows (asked
for by MP8c, giant envelopes, on 24 Nov 2026): each stable isotope's share
of its element by number, 4.567 Ga ago.

Read from Lodders 2021, Space Sci. Rev. 217, 44, Table 4 ("Proto-Solar
Isotopic Composition of the Elements ..."), column 4, "Isotopic composition
atom %", in the NSF public-access copy (https://par.nsf.gov/servlets/purl/10642789)
on 24 Nov 2026. Lodders adopts most compositions from Meija et al. 2016 and
gives the Sun's own where it differs from Earth's (C, N, O, Ar). Deuterium is
"essentially absent in the sun" today: these are protosolar shares, before
the Sun burned its deuterium into helium-3. Values are in percent as printed;
the builder divides by 100, and never types the fraction.
"""

# isotope id: atom percent of its element (Lodders 2021, Table 4, column 4).
ATOM_PERCENT = {
    "h1": 99.9980, "h2": 0.00197,
    "he3": 0.0166, "he4": 99.9834,
    "li6": 7.589, "li7": 92.411,
    "be9": 100.0,
    "b10": 19.827, "b11": 80.173,
    "c12": 98.965, "c13": 1.035,
    "n14": 99.774, "n15": 0.226,
    "o16": 99.777, "o17": 0.035, "o18": 0.188,
    "f19": 100.0,
    "ne20": 93.1251, "ne21": 0.2236, "ne22": 6.6513,
}


def apply(isotopes, val):
    """Put abundance_protosolar on each isotope row the sheet holds; return the ids it has no row for yet."""
    have = {i["id"]: i for i in isotopes}
    missing = []
    for iid, pct in ATOM_PERCENT.items():
        if iid not in have:
            missing.append(iid)
            continue
        have[iid]["abundance_protosolar"] = val(round(pct / 100.0, 12), "atom fraction of the element", "lodders2021", "unverified",
                                                note=f"{pct} atom % (Table 4)")
    # Shares of one element never pass the whole.
    by_el = {}
    for i in isotopes:
        if "abundance_protosolar" in i:
            by_el[i["element"]] = by_el.get(i["element"], 0.0) + i["abundance_protosolar"]["v"]
    for el, t in by_el.items():
        assert t <= 1.0 + 1e-6, (el, t)
    return missing
