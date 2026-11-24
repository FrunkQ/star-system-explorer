"""Hot-structure, nuclear-fuel and shield materials on the master data sheet
(track RK, package RK1 part 3; Atomic Rockets plan sections 6 and 9).

Each material is a bulk mixture (an empty recipe: its values are its own),
like the load-carrying fibres already on the sheet. The values a rocket needs:
melting point (or solidus and liquidus for an alloy), density, strength cold
and hot with the temperature it was measured at, a service limit where a
source states one, and, for shields, the hydrogen share by mass.

Every value was read from the source named beside it on 23 Nov 2026 from the
fetched text of the report or data sheet; several are OCR scans, so all are
marked unverified until a page-image check. Values printed in Fahrenheit,
inches and pounds are converted here, never typed in SI. Gathered quotes and
the gaps: handover/packages/RK1-data/RK1-sources.md, sections C and D.
"""

SOURCES = {
    "lundberg1985": ("Lundberg, L. B. 1985, Refractory metals in the space age, LA-UR-85-4120 (DE86 003650), Los Alamos National Laboratory, Table 1: melting points W 3683 K, Re 3453 K, Ta 3269 K, Mo 2883 K, Nb 2741 K; tungsten heat shields operate near 2800 K", None, "https://www.osti.gov/servlets/purl/6468954", "unverified"),
    "nasa-sp5035": ("Barth, V. D. and McIntire, H. O. 1965, Tungsten powder metallurgy, NASA SP-5035, Battelle Memorial Institute, Table I: melting points W 6126 F, Re 5720 F, Ta 5428 F, Nb 4474 F", None, "https://apps.dtic.mil/sti/pdfs/ADA399829.pdf", "unverified"),
    "materion-c103": ("Materion, Niobium C-103 alloy plate and sheet products, data sheet DS-AM-621: Nb-10Hf-1Ti, melting point 4262 F (2350 C), density 8.850 g/cm3; sheet up to 0.05 in.: at 70 F yield 40 ksi, ultimate 56 ksi; at 2000 F yield 16 ksi, ultimate 21 ksi", None, "https://hkvxni.files.cmp.optimizely.com/download/assets/Niobium+C-103+Plate+and+Sheet+Data+Sheet/091fbfb6300911ef95efb661c407e4bc", "unverified"),
    "smc-inconel718": ("Special Metals Corporation 2007, INCONEL alloy 718, publication SMC-045: melting range 2300-2437 F, density 0.296 lb/in3 (annealed), used from -423 F to 1300 F; AMS 5596 sheet minimum 180 ksi ultimate and 150 ksi yield; hot-rolled bar aged, at 1200 F, 164.5 ksi ultimate and 145.0 ksi yield", None, "https://www.specialmetals.com/documents/technical-bulletins/inconel/inconel-alloy-718.pdf", "unverified"),
    "popov2000": ("Popov, S. G., Ivanov, V. K., Carbajo, J. J. and Yoder, G. L. 2000, Thermophysical properties of MOX and UO2 fuels including the effects of irradiation, ORNL/TM-2000/351, Oak Ridge National Laboratory, sections 2.3 and 3.1: UO2 melting point 3120 +- 30 K unirradiated, theoretical density 10 970 +- 70 kg/m3 at 273 K", None, "https://rsicc.ornl.gov/fmdp/tm2000-351.pdf", "unverified"),
    "iaea-tecdoc1496": ("IAEA 2006, Thermophysical properties database of materials for light water reactors and heavy water reactors, IAEA-TECDOC-1496, section 6.1.1.1: UO2 melting point 3120 K", None, "https://www-pub.iaea.org/MTCD/Publications/PDF/te_1496_web.pdf", "unverified"),
    "panda-msfc": ("Panda, B., Hickman, R. R. and Shah, S., Solid solution carbides are the key fuels for future nuclear thermal propulsion, NASA Marshall Space Flight Center, NTRS 20060004773: UC melts at 2525 C, UC2 at 2480 C", None, "https://ntrs.nasa.gov/api/citations/20060004773/downloads/20060004773.pdf", "unverified"),
    "nist-srd126": ("Hubbell, J. H. and Seltzer, S. M., X-ray mass attenuation coefficients, NIST Standard Reference Database 126, Tables 1 and 2: densities W 19.30, Pb 11.35, polyethylene 0.930 g/cm3; hydrogen mass fraction of polyethylene 0.143716", None, "https://physics.nist.gov/PhysRefData/XrayMassCoef/tab2.html", "unverified"),
}

# Exact by definition (international yard and pound, 1959).
INCH_M = 0.0254
POUND_KG = 0.45359237


def rows(val, const):
    """Bulk mixtures for the materials table, built with the caller's val()."""
    U = "unverified"
    t0, g0 = const["T0C"], const["g0"]

    def c_k(c):
        return round(c + t0, 2)

    def f_k(f):
        return round((f - 32.0) * 5.0 / 9.0 + t0, 2)

    def ksi(k):
        return round(k * 1000.0 * POUND_KG * g0 / INCH_M ** 2, -5)  # 1000 pounds-force per square inch

    def lb_in3(x):
        return round(x * POUND_KG / INCH_M ** 3, 0)

    def mat(mid, physics, plain, story, note=None, **fields):
        row = {"id": mid, "names": {"physics": physics, "plain": plain, "story": story}, "recipe": {}, "by": "mass"}
        row.update(fields)
        if note:
            row["note"] = note
        return row

    L, SP = "lundberg1985", "nasa-sp5035"
    out = [
        # Refractory metals: hot structure, nozzles, radiators, reactor parts.
        mat("tungsten", "tungsten metal", "tungsten", "tungsten",
            melting_point_k=val(3683.0, "K", L, U, note="the text says 3685 K; NASA SP-5035 gives 6126 F, %.0f K" % f_k(6126.0)),
            density=val(19300.0, "kg m-3", "nist-srd126", U),
            service_limit_k=val(2800.0, "K", L, U, note="vacuum-furnace heat shields run near this; thermionic emitters run for very long times at 1700 K")),
        mat("molybdenum", "molybdenum metal", "molybdenum", "molybdenum",
            melting_point_k=val(2883.0, "K", L, U, note="TZM and Mo-Hf-C keep high strength from 1145 to 1475 K; must be kept from air when hot")),
        mat("niobium", "niobium metal", "niobium", "niobium",
            melting_point_k=val(2741.0, "K", L, U, note="NASA SP-5035 gives 4474 F, %.0f K" % f_k(4474.0))),
        mat("tantalum", "tantalum metal", "tantalum", "tantalum",
            melting_point_k=val(3269.0, "K", L, U, note="NASA SP-5035 gives 5428 F, %.0f K" % f_k(5428.0))),
        mat("rhenium", "rhenium metal", "rhenium", "rhenium",
            melting_point_k=val(3453.0, "K", L, U, note="NASA SP-5035 gives 5720 F, %.0f K; the two disagree by about 20 K" % f_k(5720.0))),
        mat("niobium-c103", "niobium alloy C-103 (Nb-10Hf-1Ti)", "niobium rocket-nozzle alloy", "nozzle alloy",
            note="the usual alloy for radiation-cooled thruster nozzles and skirts",
            melting_point_k=val(c_k(2350.0), "K", "materion-c103", U, note="4262 F (2350 C)"),
            density=val(8850.0, "kg m-3", "materion-c103", U),
            yield_strength=val(ksi(40.0), "Pa", "materion-c103", U, at_k=f_k(70.0), note="40 ksi, sheet up to 0.05 in."),
            tensile_strength=val(ksi(56.0), "Pa", "materion-c103", U, at_k=f_k(70.0), note="56 ksi, sheet up to 0.05 in."),
            yield_strength_hot=val(ksi(16.0), "Pa", "materion-c103", U, at_k=f_k(2000.0), note="16 ksi at 2000 F"),
            tensile_strength_hot=val(ksi(21.0), "Pa", "materion-c103", U, at_k=f_k(2000.0), note="21 ksi at 2000 F")),
        mat("inconel-718", "nickel superalloy Inconel 718", "nickel superalloy", "superalloy",
            note="turbopumps, injectors and engine structure; works from liquid hydrogen to red heat",
            solidus_k=val(f_k(2300.0), "K", "smc-inconel718", U, note="melting range 2300-2437 F"),
            liquidus_k=val(f_k(2437.0), "K", "smc-inconel718", U),
            density=val(lb_in3(0.296), "kg m-3", "smc-inconel718", U, note="0.296 lb/in3, annealed; 0.297 aged"),
            yield_strength=val(ksi(150.0), "Pa", "smc-inconel718", U, note="AMS 5596 sheet minimum, 150 ksi"),
            tensile_strength=val(ksi(180.0), "Pa", "smc-inconel718", U, note="AMS 5596 sheet minimum, 180 ksi"),
            yield_strength_hot=val(ksi(145.0), "Pa", "smc-inconel718", U, at_k=f_k(1200.0), note="145.0 ksi at 1200 F, hot-rolled bar, aged"),
            tensile_strength_hot=val(ksi(164.5), "Pa", "smc-inconel718", U, at_k=f_k(1200.0), note="164.5 ksi at 1200 F, hot-rolled bar, aged"),
            service_limit_k=val(f_k(1300.0), "K", "smc-inconel718", U, note="used from -423 F to 1300 F")),
        # Nuclear fuels: the reactor's hottest part sets a nuclear rocket's exhaust speed.
        mat("uo2-fuel", "uranium dioxide fuel (UO2)", "uranium oxide fuel", "reactor fuel",
            note="formula UO2; reactor ceramic, unirradiated",
            melting_point_k=val(3120.0, "K", "popov2000", U, note="+- 30 K, unirradiated", also=["iaea-tecdoc1496"]),
            density=val(10970.0, "kg m-3", "popov2000", U, at_k=273.0, note="theoretical density, +- 70 kg/m3")),
        mat("uc-fuel", "uranium monocarbide fuel (UC)", "uranium carbide fuel", "carbide fuel",
            melting_point_k=val(c_k(2525.0), "K", "panda-msfc", U, note="2525 C")),
        mat("uc2-fuel", "uranium dicarbide fuel (UC2)", "uranium dicarbide fuel", "carbide fuel",
            melting_point_k=val(c_k(2480.0), "K", "panda-msfc", U, note="2480 C; late Rover and NERVA fuels used (U,Zr)C solid solutions to run hotter")),
        # Shields: hydrogen slows neutrons; heavy metals stop gammas.
        mat("polyethylene", "polyethylene (CH2)n", "polyethylene plastic", "plastic",
            note="hydrogen-rich neutron shield; NIST gives one generic grade, not high-density polyethylene",
            density=val(930.0, "kg m-3", "nist-srd126", U),
            hydrogen_mass_fraction=val(0.143716, "1", "nist-srd126", U, note="the (CH2)n formula gives the same")),
        mat("lead", "lead metal", "lead", "lead", note="gamma shield",
            density=val(11350.0, "kg m-3", "nist-srd126", U)),
    ]
    for m in out:
        for k in ("yield_strength", "tensile_strength"):
            if k + "_hot" in m:
                hot = m[k + "_hot"]
                assert "at_k" in hot and hot["v"] < m[k]["v"], (m["id"], k)
        if "solidus_k" in m:
            assert m["solidus_k"]["v"] < m["liquidus_k"]["v"], m["id"]
        if "service_limit_k" in m:
            top = m.get("melting_point_k", m.get("solidus_k"))
            assert m["service_limit_k"]["v"] < top["v"], m["id"]
    return out
