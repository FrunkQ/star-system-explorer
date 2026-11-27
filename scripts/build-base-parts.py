#!/usr/bin/env python3
"""Build packs/base-parts/pack.json: the parts palette (compound ships, CM13;
design handover/design/SSE4-compound-ships-design.md sections 8A.4, 8A.5,
8B.2, 8B.3 and rulings 14A.3 to 14A.6).

Every part uses the one part format (packages/ui-contract/src/parts.ts):
figures with a source and a mark, connection points (place, size, lines,
facing), a shape, a scaling law, and the year it first flew. What it gathers:

  real stages and engines   from the Sol today pack's parts.json (GCAT,
                            Jonathan C. McDowell, CC BY 4.0; credited), with
                            the year each first flew read from the pack's own
                            launch list (the first orbital launch of the
                            vehicle family that used it)
  real power sources,       from the Sol today pack's craft.json (NSSDCA and
  arrays and dishes         each craft's own source), with the craft's launch
                            year from its COSPAR id
  generic primitives        tanks, habitat cans, docking nodes, trusses,
                            panels for arrays and radiators, bays, cargo pods
                            and booms: no year, figures that are geometry only
                            (volume, area); what the kernel reads from them
                            (berths, watts, heat) is worked out, never typed
  port_sizes                the size standard that lets parts from different
                            packs fit (8B.3)

Names are physics names only (ruling 14A.5): this builder refuses a name that
carries the Atomic Rockets brand or a named screen ship, so the palette never
publishes one. The full Atomic Rockets drive set is not in the base palette;
any pack's drive clips in through the same format.

Run after editing, and in check.sh with --check:

    python3 scripts/build-base-parts.py [--check]
"""
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SOL = ROOT / "store-packs/sol-today"
OUT = ROOT / "packs/base-parts/pack.json"

GCAT = "gcat"
NSSDCA = "nssdca"
SOURCES = {
    GCAT: "GCAT: General Catalog of Artificial Space Objects, Jonathan C. McDowell, https://planet4589.org/space/gcat/ , CC BY 4.0 (stage and engine figures, launch dates), via the Sol today pack (store-packs/sol-today/parts.json, catalogue.json).",
    NSSDCA: "NASA Space Science Data Coordinated Archive and each craft's own source as the Sol today pack's craft.json names it (store-packs/sol-today/craft.json).",
    "idss": "International Docking System Standard (IDSS) Interface Definition Document, revision E, 2016: an 800 mm passage. https://www.internationaldockingstandard.com/ (from memory, unverified).",
    "cbm": "Common Berthing Mechanism on the International Space Station: a 50 inch (1.27 m) square passage (from memory, unverified).",
    "authored": "authored for play and pictures; in the fudge register",
    "geometry": "geometry: volume is length times radius squared (a lathe's frustums); outer area its side and ends",
}

# The base palette's names are physics names (ruling 14A.5) and never a named screen ship (house rule; ruling 14A.3).
REFUSED = [r"atomic\s*rockets?", r"projectrho", r"\bliberator\b", r"millennium falcon", r"\bserenity\b", r"rocinante", r"nostromo", r"discovery one"]


def refuse_names(parts):
    bad = [p["name"] for p in parts if any(re.search(rx, p["name"], re.I) for rx in REFUSED)]
    if bad:
        raise SystemExit(f"build-base-parts: names the base palette may not carry (ruling 14A.5): {bad}")


def fig(v, unit, src, mark="cited", **kw):
    f = {"v": v, "unit": unit, "src": src, "mark": mark}
    f.update(kw)
    return f


def axial(size, lines, length):
    """Front and behind points on the axis, for a stage or module."""
    return [
        {"id": "front", "place": "front", "size_m": size, "lines": lines, "at": [length, 0, 0], "facing": [1, 0, 0]},
        {"id": "behind", "place": "behind", "size_m": size, "lines": lines, "at": [0, 0, 0], "facing": [-1, 0, 0]},
    ]


def hull_ports(size, lines, length, radius, n=4):
    """n ports round the middle, evenly; clock 0 is up (+z), 90 towards +y."""
    import math
    out = []
    for k in range(n):
        a = 2 * math.pi * k / n
        y, z = math.sin(a), math.cos(a)
        out.append({"id": f"port{k + 1}", "place": "port", "size_m": size, "lines": lines, "at": [length / 2, round(radius * y, 9), round(radius * z, 9)], "facing": [0, round(y, 12), round(z, 12)]})
    return out


def first_flights():
    """The first orbital launch year of each vehicle family in the Sol today launch list."""
    cat = json.loads((SOL / "catalogue.json").read_text())
    col = cat["launch_columns"]
    iv, idate = col.index("vehicle"), col.index("date_utc")
    first = {}
    for row in cat["launches"]:
        v, y = row[iv], int(row[idate][:4])
        if v not in first or y < first[v]:
            first[v] = y
    return first


def vehicle_year(vehicle, first):
    """A parts.json vehicle id to its family's first orbital launch ("Falcon 9 FT5" is listed as "Falcon 9")."""
    if vehicle in first:
        return first[vehicle], "first orbital launch of the vehicle in GCAT's launch list"
    best = None
    for name, y in first.items():
        if vehicle.startswith(name) and (best is None or len(name) > len(best[0])):
            best = (name, y)
    if best:
        return best[1], f"first orbital launch of the {best[0]} family in GCAT's launch list (this variant may be later)"
    return None, None


# Stages that never reached orbit are not in the launch list; their first flight is given here, marked.
SUBORBITAL = {"V-2": (1942, "first successful flight, 3 October 1942 (from memory, unverified)")}


def slug(s):
    return re.sub(r"[^a-z0-9]+", "-", s.lower()).strip("-")


def real_parts():
    parts_json = json.loads((SOL / "parts.json").read_text())
    craft = {c["id"]: c for c in json.loads((SOL / "craft.json").read_text())["craft"]}
    first = first_flights()
    stage_year = {}
    for v in parts_json["vehicles"]:
        y, why = vehicle_year(v["id"], first)
        for s in v["stack"]:
            if y is not None and (s["stage"] not in stage_year or y < stage_year[s["stage"]][0]):
                stage_year[s["stage"]] = (y, why)
    out = []
    engine_host = {}
    for s in parts_json["stages"]:
        L, D = s["length_m"], s["diameter_m"]
        p = {
            "id": f"stage-{slug(s['id'])}",
            "name": f"{s['id']} stage",
            "kind": "stage",
            "alone": True,
            "figures": {
                "wet_t": fig(s["wet_t"], "t", GCAT),
                "dry_t": fig(s["dry_t"], "t", GCAT),
                "propellant_t": fig(round(s["wet_t"] - s["dry_t"], 6), "t", GCAT, "derived", why="wet less dry"),
                "engine_count": fig(s["engine_count"], "", GCAT),
                "burn_s": fig(s["burn_s"], "s", GCAT),
            },
            "points": axial(D, ["propellant", "power"], L),
            "shape": {"hint": "lathe", "length_m": L, "radius_m": D / 2},
            "scale": {"law": "fixed", "stretch": False, "widen": False, "src": "real hardware comes in the size it flew"},
            "maker": s["maker"],
            "engine": f"engine-{slug(s['engine'])}",
            "credit": GCAT,
        }
        if s["id"] in stage_year:
            p["year"], p["year_src"] = stage_year[s["id"]][0], stage_year[s["id"]][1]
        elif s["id"] in SUBORBITAL:
            p["year"], p["year_src"] = SUBORBITAL[s["id"]]
        out.append(p)
        engine_host.setdefault(s["engine"], (s, p.get("year")))
    craft_engines = {}
    for name, c in parts_json["craft"].items():
        for e in c.get("engines", []):
            craft_engines.setdefault(e, name)
    year_of_craft = {}
    for c in craft.values():
        if c.get("cospar"):
            year_of_craft[c["name"]] = int(c["cospar"][:4])
    for e in parts_json["engines"]:
        figures = {"exhaust_km_s": fig(e["exhaust_km_s"], "km s-1", GCAT, "derived", why="GCAT's specific impulse times the master data sheet's standard gravity"), "isp_s": fig(e["isp_s"], "s", GCAT)}
        if e.get("thrust_n") is not None:
            figures["thrust_n"] = fig(e["thrust_n"], "N", GCAT)
        host = engine_host.get(e["id"])
        # The bell's size is not in the catalogue: drawn to fit its stage's cluster (estimated), or small on a craft.
        if host:
            s, y = host
            y = y if y is not None else SUBORBITAL.get(s["id"], (None,))[0]
            r = 0.8 * (s["diameter_m"] / 2) / max(1, s["engine_count"]) ** 0.5
            year, ysrc = y, "first flight of the stage it flew on"
        else:
            flew = craft_engines.get(e["id"], "")
            year = next((y for n, y in sorted(year_of_craft.items()) if flew and n.startswith(flew)), None)
            r, ysrc = 0.25, "launch of the craft it flew on (COSPAR id)"
        p = {
            "id": f"engine-{slug(e['id'])}",
            "name": f"{e['id']} engine",
            "kind": "engine",
            "alone": False,
            "figures": figures,
            "points": [{"id": "mount", "place": "front", "size_m": round(2 * r, 9), "lines": ["propellant", "power"], "at": [2.2 * r, 0, 0], "facing": [1, 0, 0]}],
            "shape": {"hint": "lathe", "length_m": round(2.2 * r, 9), "radius_m": round(r, 9), "profile": [[0, 1], [0.7, 0.45], [1, 0.35]], "shape_src": "estimated: a bell sized to fit its stage's cluster; the real size is to be read"},
            "scale": {"law": "fixed", "stretch": False, "widen": False, "src": "real hardware comes in the size it flew"},
            "maker": e["maker"],
            "fuels": e["fuels"],
            "energy_from": e["energy_from"],
            "to_read": e.get("to_read", []),
            "credit": GCAT,
        }
        if year is not None:
            p["year"], p["year_src"] = year, ysrc
        out.append(p)
    # Power sources, arrays and dishes from the craft that flew them.
    for c in craft.values():
        year = int(c["cospar"][:4]) if c.get("cospar") else None
        for pw in c.get("power", []):
            n = pw.get("count", 1)
            figures = {"electric_w": fig(pw["electric_w"] / n, "W", NSSDCA, why=f"each of {n}; {pw.get('electric_w_is', 'as the source states it')}") if n > 1 else fig(pw["electric_w"], "W", NSSDCA)}
            if pw.get("mass_t") is not None:
                figures["mass_t"] = fig(pw["mass_t"], "t", NSSDCA)
            if pw["kind"] == "solar-array":
                span = pw.get("panel_m", [None, None])[1]
                width = pw.get("panel_m", [None, None])[0]
                if span is None or width is None:
                    continue  # no size given: wait until it is read
                figures = {"electric_w_at_1_au": fig(pw["electric_w"] / n, "W", NSSDCA, why=f"each of {n} wings, at 1 au"), "area_m2": fig(width * span, "m2", NSSDCA, "derived", why="width times span")}
                p = {"id": f"array-{slug(c['id'])}", "name": f"solar array wing (as on {c['name']})", "kind": "solar", "alone": False, "figures": figures,
                     "points": [{"id": "root", "place": "hull", "size_m": 0.8, "lines": ["power"], "at": [width / 2, 0, 0], "facing": [0, -1, 0]}],
                     "shape": {"hint": "panel", "length_m": width, "radius_m": 0.05, "span_m": span},
                     "scale": {"law": "area", "stretch": True, "widen": True, "with_area": ["electric_w_at_1_au", "area_m2"], "src": "geometry: an array's output at a given sunlight follows its area"}}
            else:
                size = pw.get("size_m")
                if not size:
                    p = None
                else:
                    p = {"id": f"power-{slug(pw['id'])}", "name": f"{'radioisotope generator' if pw['kind'] == 'radioisotope' else pw['kind'].replace('-', ' ')} (as on {c['name']})", "kind": "power", "alone": False, "figures": figures,
                         "points": [{"id": "base", "place": "hull", "size_m": 0.8, "lines": ["power"], "at": [size["length"] / 2, 0, -size["diameter"] / 2], "facing": [0, 0, -1]}],
                         "shape": {"hint": "lathe", "length_m": size["length"], "radius_m": size["diameter"] / 2},
                         "scale": {"law": "fixed", "stretch": False, "widen": False, "src": "real hardware comes in the size it flew"}}
                # Its output law (decay, sunlight) stays in the Sol today pack's craft record, where the kernel reads it.
            if p:
                if year:
                    p["year"], p["year_src"] = year, "launch of the craft it flew on (COSPAR id)"
                p["credit"] = NSSDCA
                out.append(p)
        dish = c.get("dimensions", {}).get("high_gain_dish_m")
        if dish:
            p = {"id": f"dish-{slug(c['id'])}", "name": f"high-gain dish, {dish} m (as on {c['name']})", "kind": "dish", "alone": False,
                 "figures": {"aperture_m": fig(dish, "m", NSSDCA)},
                 "points": [{"id": "base", "place": "hull", "size_m": 0.8, "lines": ["power"], "at": [0, 0, 0], "facing": [-1, 0, 0]}],
                 "shape": {"hint": "dish", "length_m": round(dish / 4, 9), "radius_m": dish / 2},
                 "scale": {"law": "fixed", "stretch": False, "widen": False, "src": "real hardware comes in the size it flew"},
                 "credit": NSSDCA}
            if year:
                p["year"], p["year_src"] = year, "launch of the craft it flew on (COSPAR id)"
            out.append(p)
    bay = craft.get("sts-1-columbia", {}).get("payload_bay_m")
    if bay:
        p = {"id": "bay-sts-1-columbia", "name": f"payload bay, {bay[0]} by {bay[1]} m (as on Columbia)", "kind": "bay", "alone": False,
             "figures": {"bay_radius_m": fig(bay[0] / 2, "m", NSSDCA, "derived", why="half the stated width"), "bay_length_m": fig(bay[1], "m", NSSDCA)},
             "points": [{"id": "mouth", "place": "bay", "size_m": bay[0], "lines": ["power"], "at": [bay[1] / 2, 0, bay[0] / 2], "facing": [0, 0, 1]}],
             "shape": {"hint": "lathe", "length_m": bay[1], "radius_m": bay[0] / 2},
             "scale": {"law": "fixed", "stretch": False, "widen": False, "src": "real hardware comes in the size it flew"},
             "year": 1981, "year_src": "launch of the craft it flew on (COSPAR id)", "credit": NSSDCA}
        out.append(p)
    return out


PORTS = [
    {"size_m": 0.8, "name": "crew passage (docking standard)", "src": "idss", "check": "unverified"},
    {"size_m": 1.27, "name": "berthing passage", "src": "cbm", "check": "unverified"},
    {"size_m": 2.0, "name": "small module or pod", "src": "authored"},
    {"size_m": 4.0, "name": "module or pod", "src": "authored"},
    {"size_m": 10.0, "name": "large module, pod or hangar", "src": "authored"},
]


def generic_parts():
    """Primitives with no year: geometry only, sizes on the standard, stretch and widen by their law."""
    vol = lambda: {"law": "volume", "stretch": True, "widen": True, "with_volume": ["volume_m3"], "src": "geometry"}
    out = []
    for size in (2.0, 4.0, 10.0):
        r = size / 2
        L = 2 * size
        V = round(3.141592653589793 * r * r * L, 6)
        tag = f"{size:g} m"
        out.append({"id": f"tank-{tag.replace(' ', '')}", "name": f"propellant tank, {tag} across", "kind": "tank", "alone": False,
                    "figures": {"volume_m3": fig(V, "m3", "geometry", "derived", why="length times radius squared")},
                    "points": axial(size, ["propellant"], L) + hull_ports(0.8, ["propellant"], L, r, 2),
                    "shape": {"hint": "lathe", "length_m": L, "radius_m": r}, "scale": vol(),
                    "to_read": ["wall and insulation mass from the thin-wall pressure block (CM13 part 2)"]})
        out.append({"id": f"habitat-{tag.replace(' ', '')}", "name": f"habitat can, {tag} across", "kind": "habitat", "alone": True,
                    "figures": {"volume_m3": fig(V, "m3", "geometry", "derived", why="length times radius squared")},
                    "points": axial(min(size, 1.27) if size < 4 else 1.27, ["air", "people", "power"], L) + hull_ports(0.8, ["air", "people", "power"], L, r, 4),
                    "shape": {"hint": "lathe", "length_m": L, "radius_m": r}, "scale": vol(),
                    "to_read": ["berths from the room per person (base-ships crew rule), worked out by the kernel"]})
        out.append({"id": f"pod-{tag.replace(' ', '')}", "name": f"cargo pod, {tag} across", "kind": "pod", "alone": True,
                    "figures": {"volume_m3": fig(V, "m3", "geometry", "derived", why="length times radius squared")},
                    "points": [{"id": "clamp", "place": "port", "size_m": size, "lines": [], "at": [L / 2, 0, -r], "facing": [0, 0, -1]}],
                    "shape": {"hint": "box", "length_m": L, "radius_m": r, "span_m": size},
                    "scale": {"law": "fixed", "stretch": False, "widen": False, "src": "standard sizes, so a frame freighter's pods fit any frame (ruling 14A.6)"}})
    out.append({"id": "node-4", "name": "docking node, four ports", "kind": "node", "alone": True,
                "figures": {"volume_m3": fig(round(3.141592653589793 * 1.0 * 1.0 * 2.5, 6), "m3", "geometry", "derived", why="length times radius squared")},
                "points": axial(0.8, ["air", "people", "power"], 2.5) + hull_ports(0.8, ["air", "people", "power"], 2.5, 1.0, 4),
                "shape": {"hint": "lathe", "length_m": 2.5, "radius_m": 1.0},
                "scale": {"law": "fixed", "stretch": False, "widen": False, "src": "authored"}})
    out.append({"id": "truss", "name": "truss or spine", "kind": "truss", "alone": False, "figures": {},
                "points": axial(1.27, ["power", "propellant"], 10.0) + hull_ports(1.27, ["power", "propellant"], 10.0, 0.5, 4),
                "shape": {"hint": "truss", "length_m": 10.0, "radius_m": 0.5},
                "scale": {"law": "volume", "stretch": True, "widen": True, "src": "geometry"},
                "to_read": ["mass from strength for weight (base-tech materials) and the load it carries"]})
    out.append({"id": "boom", "name": "boom (sensors clear of the ship)", "kind": "truss", "alone": False, "figures": {},
                "points": axial(0.8, ["power"], 5.0),
                "shape": {"hint": "truss", "length_m": 5.0, "radius_m": 0.1},
                "scale": {"law": "volume", "stretch": True, "widen": False, "src": "geometry"}})
    for kind, name, line in (("solar", "solar array panel", "power"), ("radiator", "radiator panel", "power")):
        out.append({"id": f"{kind}-panel", "name": name, "kind": kind, "alone": False,
                    "figures": {"area_m2": fig(20.0, "m2", "geometry", "derived", why="length times span, one face")},
                    "points": [{"id": "root", "place": "hull", "size_m": 0.8, "lines": [line], "at": [1.0, 0, 0], "facing": [0, -1, 0]}],
                    "shape": {"hint": "panel", "length_m": 2.0, "radius_m": 0.05, "span_m": 10.0},
                    "scale": {"law": "area", "stretch": True, "widen": True, "with_area": ["area_m2"], "src": "geometry"},
                    "to_read": ["watts (arrays: sunlight at the ship) and heat shed (radiators: their temperature) worked out by the kernel"]})
    out.append({"id": "bay", "name": "bay", "kind": "bay", "alone": False,
                "figures": {"volume_m3": fig(round(3.141592653589793 * 2.0 * 2.0 * 8.0, 6), "m3", "geometry", "derived", why="length times radius squared")},
                "points": [{"id": "mouth", "place": "bay", "size_m": 4.0, "lines": ["power"], "at": [8.0, 0, 0], "facing": [1, 0, 0]}] + axial(4.0, ["power", "propellant"], 8.0),
                "shape": {"hint": "lathe", "length_m": 8.0, "radius_m": 2.0}, "scale": vol()})
    out.append({"id": "dish", "name": "dish antenna", "kind": "dish", "alone": False,
                "figures": {"aperture_m": fig(2.0, "m", "geometry", "derived", why="its diameter")},
                "points": [{"id": "base", "place": "hull", "size_m": 0.8, "lines": ["power"], "at": [0, 0, 0], "facing": [-1, 0, 0]}],
                "shape": {"hint": "dish", "length_m": 0.5, "radius_m": 1.0},
                "scale": {"law": "area", "stretch": False, "widen": True, "with_area": [], "src": "geometry"},
                "to_read": ["aperture sized from the link it must carry (the link budget block, CM13 part 2)"]})
    return out


# Eras of spaceflight (Alex, 27 Nov 2026 11:52: "lets not do them by age but by eras... it means we can have a few
# 'future eras'", and "they can vary in length"). Named by what changed, of any length; a flown part falls in the era
# its first-flight year does, the picker still reads the exact year. Future eras are where the tech drives that have
# not flown sit; `level` is the base-tech ladder level an era reaches. Pack data: a setting or store pack renames,
# moves or adds eras. Boundaries authored (the events named are well known; the years are from memory, unverified).
ERAS = [
    {"id": "pioneers", "name": "Rocket pioneers", "to": 1956, "level": 2, "about": "the V-2 and the first sounding rockets"},
    {"id": "space-race", "name": "Space race", "from": 1957, "to": 1972, "level": 3, "about": "Sputnik to the last Moon landing"},
    {"id": "stations", "name": "Stations and shuttles", "from": 1973, "to": 1999, "level": 3, "about": "Skylab, Mir, the Shuttle and Voyager"},
    {"id": "reusable", "name": "Reusable and commercial", "from": 2000, "to": 2029, "level": 4, "about": "the ISS, landing boosters and ion-driven probes"},
    {"id": "inner-system", "name": "Inner system", "from": 2030, "to": 2079, "level": 4, "future": True, "about": "nuclear-thermal ships, space reactors, big electric drives and depots"},
    {"id": "fusion", "name": "Fusion", "from": 2080, "to": 2199, "level": 5, "future": True, "about": "fusion torches and large sails"},
    {"id": "interstellar", "name": "Interstellar", "from": 2200, "level": 6, "future": True, "about": "relativistic drives"},
]
ERA_SRC = "authored: the era a tech drive first flew in, from memory and unverified; a drive that has not flown sits in the first future era its ladder level reaches"
# Tech drives that have flown, by the era they first flew in; the rest go to the first future era at their level.
TECH_FLOWN = {"chemical-rocket": "pioneers", "fission-reactor": "space-race", "rtg": "space-race", "hall-thruster": "space-race", "arcjet": "stations", "ion-thruster": "stations", "light-sail": "reusable"}


def tech_era(a):
    if a["id"] in TECH_FLOWN:
        return TECH_FLOWN[a["id"]]
    return next(e["id"] for e in ERAS if e.get("future") and e["level"] >= a["level"])


def tech_parts():
    """Drives and power plants from the future tech pack's advances (ruling 14A.5: each era ships its own drives).
    The figures are not copied: a part names its advance, and the kernel reads that advance's figures at the
    setting's level, so a drive improves along the ladder (CM15 fits the trends between)."""
    tech = json.loads((ROOT / "packs/base-tech/pack.json").read_text())["provides"]["other"]["tech"]
    out = []
    for a in tech["advances"]:
        part = a.get("part", "")
        if part not in ("drive", "electric drive", "reactor", "power plant"):
            continue
        drive = "drive" in part
        out.append({"id": f"tech-{a['id']}", "name": a["name"].lower() if drive else a["name"], "kind": "engine" if drive else "power", "alone": False, "figures": {},
                    "advance": a["id"], "level": a["level"], "era": tech_era(a), "era_src": ERA_SRC,
                    "points": [{"id": "mount", "place": "front" if drive else "hull", "size_m": 2.0, "lines": ["propellant", "power"] if drive else ["power"], "at": [2.0, 0, 0] if drive else [1.0, 0, -1.0], "facing": [1, 0, 0] if drive else [0, 0, -1]}],
                    "shape": {"hint": "lathe", "length_m": 2.0, "radius_m": 1.0, "shape_src": "estimated: a stand-in size until its sizing block draws it from its output"},
                    "scale": {"law": "fixed", "stretch": False, "widen": False, "src": "its size follows its output through the advance's figures, worked out by the kernel"},
                    "to_read": [f"figures from the base-tech advance {a['id']} at the setting's level"]})
    return out


def build():
    parts = real_parts() + generic_parts() + tech_parts()
    refuse_names(parts)
    ids = [p["id"] for p in parts]
    dup = sorted({i for i in ids if ids.count(i) > 1})
    if dup:
        raise SystemExit(f"build-base-parts: duplicate part ids {dup}")
    for p in parts:
        for f in p["figures"].values():
            if f["src"] not in SOURCES:
                raise SystemExit(f"build-base-parts: {p['id']} cites {f['src']}, which the pack does not list")
    return {
        "format": "sse.pack/1",
        "id": "sse.base-parts",
        "version": "0.1.0",
        "kind": ["base-physics"],
        "kernel_range": "^0.1",
        "licence": "Apache-2.0; stage, engine and launch figures from GCAT are CC BY 4.0 (credited)",
        "provides": {
            "other": {
                "about": "The parts palette (compound ships, CM13): every part in the one part format (packages/ui-contract/src/parts.ts). Real stages, engines, power sources, arrays, dishes and a bay with the figures they flew with and the year they first flew; generic primitives with geometry only. Physics names only (ruling 14A.5). Built by scripts/build-base-parts.py; do not edit by hand.",
                "palette": {"eras": ERAS, "port_sizes": PORTS, "parts": parts, "credits": [SOURCES[GCAT], SOURCES[NSSDCA]]},
                "sources": SOURCES,
            }
        },
    }


def main():
    text = json.dumps(build(), indent=1, ensure_ascii=False) + "\n"
    if "--check" in sys.argv:
        if not OUT.exists() or OUT.read_text() != text:
            raise SystemExit("build-base-parts: packs/base-parts/pack.json is out of date; run scripts/build-base-parts.py")
        print("base-parts up to date")
        return
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(text)
    print(f"wrote {OUT.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
