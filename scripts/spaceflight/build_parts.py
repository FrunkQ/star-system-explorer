#!/usr/bin/env python3
"""RS2 part 1: real launch vehicle and engine parts for Sol today, from GCAT (CC BY 4.0).

Usage: python3 -I scripts/spaceflight/build_parts.py <dir with GCAT engines.tsv, stages.tsv, lvs.tsv> <out dir>

Writes <out dir>/parts.json in the field names of SSE4-atomic-rockets-plan.md section 11.3
(engine definition: fuels[], thrust_n, exhaust_km_s, energy_from, mass_t ...). Exhaust speed is
Isp times g0, with g0 read from the master data sheet (packs/base-reference), never typed here.
Propellants are named by their master data sheet substance ids; a propellant the sheet does not
carry yet is listed under "sheet_missing" and kept by GCAT's own name. Every figure GCAT does not
give (engine mass, sea-level thrust and Isp, electric input) is listed in "to_read" for the craft
that need it, to be read from a primary source in RS2 part 2: nothing is guessed.
"""
import json, os, sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
SHEET = json.load(open(os.path.join(ROOT, "packs/base-reference/pack.json"), encoding="utf-8"))["provides"]["reference"]
G0 = {c["id"]: c["value"]["v"] for c in SHEET["constants"]}["g0"]
SUBSTANCES = {s["id"] for s in SHEET["substances"]}

# GCAT propellant names to sheet substance ids. Mixtures list their parts.
PROP = {
    "LOX": ["o2"], "LH2": ["h2"], "N2O4": ["n2o4"], "MON": ["n2o4"], "MON-3": ["n2o4"],
    "Hydrazine": ["n2h4"], "N2H4": ["n2h4"], "MMH": ["mmh"], "UDMH": ["udmh"],
    "AZ-50": ["n2h4", "udmh"], "Methane": ["ch4"], "Xe": ["xe"],
    "Alcohol": ["ethanol", "h2o"],  # the A-4 burned 75 percent ethanol in water
    "Kero": ["rp-1"], "Kerosene RP-1": ["rp-1"],
    "PBAN": ["pban-ap-al"], "Solid": ["solid-composite"],
}

# The launch vehicles of the first slice (design section 8), as GCAT names them (LV, variant).
VEHICLES = [
    ("Saturn V", "-", "Apollo 11"),
    ("Space Shuttle", "-", "STS-1"),
    ("Falcon 9", "FT5", "Falcon 9 booster landings"),
    ("Titan IIIE", "-", "Voyager 2"),
    ("A-4", "-", "V-2"),
]
# Engines on the craft themselves, by GCAT name (those GCAT does not list are in MISSING).
CRAFT_ENGINES = {
    "Apollo 11": ["AJ10-137"], "STS-1": ["AJ10-190"], "Voyager 2": ["Star 37E"],
    "Cassini-Huygens": ["R-4D-11"], "Juno": ["Leros 1b"], "Dawn": ["NSTAR-30"],
}
MISSING = {
    "Apollo 11": ["LM descent engine (TRW LMDE)", "LM ascent engine (Bell)"],
    "Curiosity": ["Mars Lander Engine (Aerojet MR-80B)"],
    "SNAP-10A": ["SNAPSHOT ion engine test unit", "SNAP-10A reactor (a power source, not an engine)"],
    "James Webb Space Telescope": ["secondary combustion augmented thrusters"],
    "International Space Station": ["Zvezda main engines (KTDU-80)"],
}

def rd(path):
    rows, head = [], None
    for line in open(path, encoding="utf-8", errors="replace"):
        if not line.strip():
            continue
        if head is None:
            head = line.lstrip("#").rstrip("\n").split("\t")
            continue
        if line.startswith("#"):
            continue
        rows.append(dict(zip(head, [x.strip() for x in line.rstrip("\n").split("\t")])))
    return rows

def num(s):
    s = (s or "").strip()
    if s in ("", "-", "?"):
        return None
    try:
        v = float(s)
    except ValueError:
        return None
    return int(v) if v == int(v) else v

src = sys.argv[1]
engines = {r["Name"]: r for r in rd(os.path.join(src, "engines.tsv"))}
stages = {r["Stage_Name"]: r for r in rd(os.path.join(src, "stages.tsv"))}
lvs = rd(os.path.join(src, "lvs.tsv"))
missing_sheet = set()

def fuels(r):
    out = []
    for name in (r.get("Oxidizer", "-"), r.get("Fuel", "-")):
        name = name.strip()
        if name in ("", "-", "IPS"):  # IPS: GCAT's "ion propulsion system", not a propellant
            continue
        ids = PROP.get(name, [name.lower()])
        for i in ids:
            if i not in SUBSTANCES:
                missing_sheet.add(i)
            if i not in out:
                out.append(i)
    return out

def engine(name):
    r = engines[name]
    e = {"id": name, "maker": r["Manufacturer"], "fuels": fuels(r)}
    t, isp = num(r["Thrust"]), num(r["Isp"])
    if t:
        e["thrust_n"] = round(t * 1000.0, 3)
    if isp:
        e["isp_s"] = isp
        e["exhaust_km_s"] = round(isp * G0 / 1000.0, 4)
    e["energy_from"] = "electric" if "xe" in e["fuels"] else "fuel"
    if num(r["Duration"]):
        e["burn_s"] = num(r["Duration"])
    flags = {k: r[k] for k in ("TFlag", "IspFlag") if r.get(k, "").strip()}
    e["mark"] = "estimated" if any(v == "?" for v in flags.values()) else "cited"
    e["src"] = "gcat"
    need = ["mass_t"]
    if e["energy_from"] == "fuel" and not name.endswith("Vac"):
        need.append("thrust_sl_n")
    if e["energy_from"] == "electric":
        need += ["electric_w", "thrust_n", "efficiency"]
    # GCAT gives one thrust and one Isp per engine without saying at sea level or in vacuum
    # (its F-1 row is close to the sea-level figure, its J-2 row the vacuum one), so the
    # condition is read with the missing figures.
    need.append("condition_of_thrust_and_isp")
    e["to_read"] = need
    return e

out_engines, out_stages, out_vehicles = {}, {}, []
for lv, variant, used_by in VEHICLES:
    stack = [r for r in lvs if r["LV_Name"] == lv and r["LV_Variant"] == variant]
    rows = []
    for r in stack:
        sn = r["Stage_Name"]
        if sn not in stages or sn.endswith("Interstage") or sn == "Fairing":
            continue
        s = stages[sn]
        st = {"id": sn, "maker": s["Stage_Manufacturer"], "length_m": num(s["Length"]),
              "diameter_m": num(s["Diameter"]), "wet_t": num(s["Launch_Mass"]),
              "dry_t": (num(s["Dry_Mass"]) or 0) / 1000.0 or None, "engine": s["Engine"],
              "engine_count": num(s["NEng"]), "burn_s": num(s["Duration"]),
              "mark": "estimated" if "?" in (s["Launch_Mass_Flag"] + s["Dry_Mass_Flag"]) else "cited", "src": "gcat"}
        out_stages[sn] = {k: v for k, v in st.items() if v is not None}
        if s["Engine"] in engines:
            out_engines[s["Engine"]] = engine(s["Engine"])
        rows.append({"stage": sn, "order": r["Stage_No"], "count": num(r["Multiplicity"])})
    out_vehicles.append({"id": f"{lv} {variant}".replace(" -", ""), "used_by": used_by, "stack": rows, "src": "gcat"})
craft = {}
for c, names in CRAFT_ENGINES.items():
    for n in names:
        out_engines[n] = engine(n)
    craft.setdefault(c, {})["engines"] = names
for c, names in MISSING.items():
    craft.setdefault(c, {})["engines_to_read"] = names

doc = {
    "about": "Sol today parts, RS2 part 1: launch vehicles, their stages and engines, and the engines on the first slice's craft, from GCAT. Field names follow SSE4-atomic-rockets-plan.md section 11.3. Generated by scripts/spaceflight/build_parts.py; do not edit by hand. Figures GCAT does not give are listed under to_read and come from primary sources in RS2 part 2.",
    "notes": ["GCAT books a stage's engines where they fire from: the Shuttle's SSMEs appear on its ET stage although they ride on the orbiter (OV). The construct record puts them on the orbiter; the stage keeps GCAT's row."],
    "credit": "Stage and engine data from GCAT, Jonathan C. McDowell, https://planet4589.org/space/gcat/ , CC BY 4.0. Exhaust speed is GCAT's Isp times the master data sheet's g0.",
    "sheet_missing": sorted(missing_sheet),
    "engines": [out_engines[k] for k in sorted(out_engines)],
    "stages": [out_stages[k] for k in sorted(out_stages)],
    "vehicles": out_vehicles,
    "craft": {k: craft[k] for k in sorted(craft)},
}
os.makedirs(sys.argv[2], exist_ok=True)
with open(os.path.join(sys.argv[2], "parts.json"), "w", encoding="utf-8") as f:
    json.dump(doc, f, ensure_ascii=False, indent=1)
    f.write("\n")
print(len(doc["engines"]), "engines", len(doc["stages"]), "stages", len(out_vehicles), "vehicles; sheet missing", doc["sheet_missing"])
