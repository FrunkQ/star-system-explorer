"""Nuclear reactions on the master data sheet (track RK, package RK1 part 2):
fusion reactions as rows of the sheet's reactions table with kind "nuclear",
and U-235's fission energy split on its isotope row.

A fusion reaction's energy release Q is worked out here from the sheet's own
AME2020 atomic masses (electrons balance in every reaction listed, so atomic
masses serve), and how it divides between the two products from momentum:
for reactants at rest, a product gets Q times the other product's mass over
the sum. Nothing here is typed except the cross-check values from the
sources, which the builder compares against.
"""

SOURCES = {
    "fitzpatrick-fusion": ("Fitzpatrick, R., Plasma physics lecture notes, Nuclear fusion reactions, University of Texas at Austin: D-T releases 17.59 MeV, the neutron carrying 14.05 MeV and the alpha 3.541 MeV", None, "https://farside.ph.utexas.edu/teaching/plasma1/Fusionhtml/node6.html", "unverified"),
    "sher-beck1981": ("Sher, R. and Beck, C. 1981, Fission energy release for 16 fissioning nuclides, EPRI NP-1771, Stanford University, Table VI (U-235, thermal): 202.48 MeV in all, fragments 169.12, prompt neutrons 4.79, prompt gammas 6.97, delayed betas 6.50, delayed gammas 6.33, neutrinos 8.75 MeV per fission", None, "https://www.osti.gov/servlets/purl/6614291", "unverified"),
}

# id, name, reactants, products (nuclide ids; "neutron" is the free neutron).
FUSION = [
    ("d-t", "deuterium-tritium fusion", ["h2", "h3"], ["he4", "neutron"]),
    ("d-d-n", "deuterium-deuterium fusion, helium-3 branch", ["h2", "h2"], ["he3", "neutron"]),
    ("d-d-p", "deuterium-deuterium fusion, tritium branch", ["h2", "h2"], ["h3", "h1"]),
    ("d-he3", "deuterium-helium-3 fusion", ["h2", "he3"], ["he4", "h1"]),
]
# What the sources print, to check the worked-out values against (MeV).
CHECKS = {"d-t": {"q_mev": 17.59, "neutron_mev": 14.05, "src": "fitzpatrick-fusion"}}


def reactions(isotopes, const, val):
    """Rows for the reactions table, worked out from isotope masses (u) and constants (SI)."""
    mass = {i["id"]: i["mass"]["v"] for i in isotopes if "mass" in i}
    mass["neutron"] = const["m_n"] / const["m_u"]
    mev_per_u = const["m_u"] * const["c"] ** 2 / const["e"] / 1e6
    out = []
    for rid, name, ins, outs in FUSION:
        q = (sum(mass[x] for x in ins) - sum(mass[x] for x in outs)) * mev_per_u
        m1, m2 = mass[outs[0]], mass[outs[1]]
        terms = {}
        for x in ins:
            terms[x] = terms.get(x, 0) - 1
        for x in outs:
            terms[x] = terms.get(x, 0) + 1
        products = {outs[0]: val(round(q * m2 / (m1 + m2), 4), "MeV", "ame2020", "unverified", note="share by momentum, reactants at rest"),
                    outs[1]: val(round(q * m1 / (m1 + m2), 4), "MeV", "ame2020", "unverified", note="share by momentum, reactants at rest")}
        row = {"id": rid, "kind": "nuclear", "name": name, "terms": [[k, v] for k, v in terms.items()],
               "q": val(round(q, 4), "MeV", "ame2020", "unverified", note="worked out from the AME2020 atomic masses on this sheet"),
               "product_energy": products}
        if "neutron" in outs:
            row["neutron_share"] = val(round(m1 / (m1 + m2), 4), "1", "ame2020", "unverified", note="the neutron's share of Q; it carries this energy out of a magnetic bottle and into the shield")
        else:
            row["neutron_share"] = val(0.0, "1", "ame2020", "unverified", note="no neutron in the main branch; side reactions add some")
        chk = CHECKS.get(rid)
        if chk:
            assert abs(q - chk["q_mev"]) < 0.01, (rid, q)
            assert abs(products["neutron"]["v"] - chk["neutron_mev"]) < 0.01, (rid, products["neutron"]["v"])
            row["q"]["also"] = [chk["src"]]
        out.append(row)
    return out


def fission_u235(val):
    """U-235 thermal fission energy per fission, by where it goes (Sher and Beck 1981)."""
    s, U = "sher-beck1981", "unverified"
    parts = {
        "fragments": val(169.12, "MeV", s, U), "prompt_neutrons": val(4.79, "MeV", s, U),
        "prompt_gammas": val(6.97, "MeV", s, U), "delayed_betas": val(6.50, "MeV", s, U),
        "delayed_gammas": val(6.33, "MeV", s, U), "neutrinos": val(8.75, "MeV", s, U, note="leave the reactor unseen"),
    }
    total = val(202.48, "MeV", s, U, note="all of it, neutrinos included")
    assert abs(sum(p["v"] for p in parts.values()) - total["v"]) < 0.1
    return {"total": total, "split": parts,
            "recoverable": val(round(total["v"] - parts["neutrinos"]["v"], 2), "MeV", s, U, note="worked out: the total less the neutrinos")}
