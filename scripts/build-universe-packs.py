#!/usr/bin/env python3
"""Build the two universe packs written for SSE4 (tags design TG13).

- packs/near-future/pack.json (`sse.near-future`): a generic near-future hard
  science fiction setting. Slow ships on real drives, a few powers, ports at
  the worlds people can live near and fuel depots at the giants.
- packs/space-opera/pack.json (`sse.space-opera`): an original space opera
  setting, "the Meridian Reach", with a crown, a league of free worlds and a
  syndicate. Fictional drives, marked so.

Each pack holds only existing pack contents (tags design 5.7): tag
declarations, templates, labelling rules (rule tags), dressing rules (TG12:
a world like this gets one of these) and a political lens. Nothing in either
pack reaches physics: dressing adds authored objects, labels are labels.
Every name here is written for SSE4; neither pack copies a published setting.

Conditions read the facts a world carries under either of the two namings in
use (SI facts from the catalogue and formation: `body.mass`, `body.radius`,
`orbit.semi_major_axis`; or the chart room's: `mass_earth`, `a_au`,
`surface_k`), so the same rule dresses an adopted Sol and the Kestrel Reach.
Edit this script, never the JSON.
"""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
# Unit factors for writing thresholds in both namings, read from the master
# data sheet (packs/base-reference), never typed here (MP22).


def _sheet_value(id_):
    ref = json.loads((ROOT / "packs/base-reference/pack.json").read_text())["provides"]["reference"]

    def find(o):
        if isinstance(o, dict):
            if o.get("id") == id_ and isinstance(o.get("value"), dict):
                return o["value"]["v"]
            for v in o.values():
                r = find(v)
                if r is not None:
                    return r
        elif isinstance(o, list):
            for v in o:
                r = find(v)
                if r is not None:
                    return r
        return None

    v = find(ref)
    if v is None:
        raise SystemExit(f"base-reference has no {id_}")
    return v


EARTH_KG = _sheet_value("M_earth")
AU_M = _sheet_value("au")


# ---- Conditions in both namings -------------------------------------------

def mass_at_least(earths, prefix=""):
    return {"if": "any", "of": [
        {"if": "range", "field": f"{prefix}body.mass", "min": earths * EARTH_KG},
        {"if": "range", "field": f"{prefix}mass_earth", "min": earths},
    ]}


def mass_between(lo, hi, prefix=""):
    return {"if": "any", "of": [
        {"if": "range", "field": f"{prefix}body.mass", "min": lo * EARTH_KG, "max": hi * EARTH_KG},
        {"if": "range", "field": f"{prefix}mass_earth", "min": lo, "max": hi},
    ]}


def orbit_between(lo_au, hi_au):
    return {"if": "any", "of": [
        {"if": "range", "field": "orbit.semi_major_axis", "min": lo_au * AU_M, "max": hi_au * AU_M},
        {"if": "range", "field": "a_au", "min": lo_au, "max": hi_au},
    ]}


def around_a_star():
    """A planet: its parent has no planet-sized mass of its own (it is a star or the system)."""
    return {"if": "not", "p": mass_at_least(0.0001, "parent.")}


def around_a_giant():
    return mass_between(10, 4000, "parent.")


# Stars carry a mass too: every world condition first asks "not a star".
NOT_STAR = {"if": "all", "of": [
    {"if": "not", "p": {"if": "has", "field": "star.effective_temperature"}},
    {"if": "not", "p": {"if": "has", "field": "mass_sun"}},
]}
# A giant: 10 Earth masses (below Uranus, the lightest giant in Sol) up to about 13 Jupiter masses, where
# deuterium burning starts (brown dwarfs are not giants for these labels).
GIANT = {"if": "all", "of": [NOT_STAR, mass_between(10, 4000)]}
ROCK = {"if": "all", "of": [NOT_STAR, mass_between(0.05, 10)]}
TEMPERATE = {"if": "any", "of": [
    {"if": "range", "field": "surface_k", "min": 240, "max": 330},
    {"if": "all", "of": [{"if": "not", "p": {"if": "has", "field": "surface_k"}}, orbit_between(0.8, 1.3)]},
]}
COLD = {"if": "any", "of": [
    {"if": "range", "field": "surface_k", "max": 120},
    {"if": "all", "of": [{"if": "not", "p": {"if": "has", "field": "surface_k"}}, orbit_between(5.0, 60.0)]},
]}


def all_of(*ps):
    return {"if": "all", "of": list(ps)}


def tag(t):
    return {"if": "tag", "tag": t}


# ---- Shared declarations (identical in every pack that uses them) ---------

SHARED_TAGS = [{"id": "construct", "label": "construct"}, {"id": "ship", "label": "ship"},
               {"id": "station", "label": "station"}, {"id": "site", "label": "site"}]


def construct(id_, kind, label, explain, *, facts=None, tags=()):
    """A template. No kept values: crew aboard, readiness and discipline are
    a game system's record kinds (the Traveller pack declares them), so a
    universe pack carries only facts and stacks under any game system."""
    return {"id": id_, "kind": kind, "label": label, "facts": {**(facts or {}), "construct.source": "written for this pack"},
            "tags": ["construct", kind, *tags], "explain": explain}


def lens(id_, label, description, fields):
    return {"schema": "sse.lens/0", "id": id_, "label": label, "description": description,
            "cards": [{"subject_kind": "*", "fields": fields, "density": "compact"}],
            "layers": ["orbits", "zones", "labels", "moons", "key"], "tools": ["undo"], "cost": "light"}


def pack(id_, label, about, prefix, groups, tags, templates, rules, dressing, lenses, fictional):
    declared = SHARED_TAGS + [{"id": t, "label": l, **({"group": g} if g else {}), "about": a, "source": s, "tier": tier}
                              for (t, l, g, a, s, tier) in tags]
    return {
        "format": "sse.pack/1",
        "id": id_,
        "version": "0.1.0",
        "kind": ["universe"],
        "kernel_range": "^0.1",
        "licence": "Apache-2.0",
        "provides": {"other": {
            "about": {"label": label, "text": about, "fictional": fictional,
                      "note": "Written for SSE4 (tags design TG13). Rules and plain descriptions only; no published setting is copied."},
            "tags": {"about": "Tags this pack names (tags design, TG1): labels only, never an input to physics.",
                     "groups": groups, "tags": declared},
            "templates": {"templates": templates},
            "rules": rules,
            "dressing": {"about": f"{label}: which worlds get a port, a base or a faction's presence, and why.", "rules": dressing},
            "lenses": lenses,
        }},
    }


# ---- Near future (hard science fiction) -----------------------------------

NF = "nf"
NF_GROUPS = [
    {"id": "nf.label", "label": "Near future: worlds", "about": "How the near-future powers speak of a world.", "applies_to": ["body"]},
    {"id": "nf.faction", "label": "Near future: powers", "about": "Who runs a place.", "applies_to": ["construct", "body"], "exclusive": True},
    {"id": "nf.port", "label": "Near future: ports", "about": "Where a ship can dock, refuel or refit.", "applies_to": ["construct"]},
]
NF_TAGS = [
    ("nf.label.homeworld", "home-type world", "nf.label", "rocky, temperate and around its star: where people can live outside a habitat", "derived", "obvious"),
    ("nf.label.volatile-giant", "volatile giant", "nf.label", "a giant whose air holds hydrogen to skim for fusion fuel", "derived", "obvious"),
    ("nf.label.ice-moon", "ice moon", "nf.label", "a moon of a giant: water ice and shelter from the giant's belts behind rock", "derived", "scanned"),
    ("nf.label.frontier", "frontier rock", "nf.label", "a rocky world far and cold from its star, worth a claim", "derived", "scanned"),
    ("nf.faction.accord", "Terran Accord", "nf.faction", "the inner worlds' treaty government and its navy", "authored", "obvious"),
    ("nf.faction.consortium", "Jovian Consortium", "nf.faction", "the fuel and ice combines of the giants", "authored", "obvious"),
    ("nf.faction.haulers", "Free Haulers' Union", "nf.faction", "independent crews and claim-stakers", "authored", "obvious"),
    ("nf.port.yard", "orbital yard", "nf.port", "docking, refit and refuelling in orbit", "authored", "obvious"),
    ("nf.port.fuel", "fuel depot", "nf.port", "refined fusion fuel for sale", "authored", "obvious"),
    ("nf.military", "military", None, "armed and under a navy's orders", "authored", "scanned"),
    ("nf.research", "research", None, "a science post", "authored", "obvious"),
]
NF_TEMPLATES = [
    construct("nf_orbital_yard", "station", "Orbital yard", "a large station for docking, refit and refuelling, spun for gravity",
              facts={"construct.mass_kg": 4.0e8, "construct.length_m": 600.0, "construct.crew_max": 4000.0}, tags=("nf.port.yard", "nf.port.fuel")),
    construct("nf_skimmer_depot", "station", "Skimmer depot", "a fuel station in low orbit of a giant, fed by skimmer craft diving into its air",
              facts={"construct.mass_kg": 6.0e7, "construct.crew_max": 300.0, "construct.cargo_t": 50000.0}, tags=("nf.port.fuel",)),
    construct("nf_patrol_station", "station", "Patrol station", "a navy picket: sensors, a hangar of cutters and a brig",
              facts={"construct.mass_kg": 9.0e7, "construct.crew_max": 800.0}, tags=("nf.military",)),
    construct("nf_ice_station", "site", "Ice-moon station", "a research and water-mining base dug under the ice",
              facts={"construct.crew_max": 120.0, "construct.power_mw": 40.0}, tags=("nf.research",)),
    construct("nf_claim_outpost", "site", "Claim outpost", "a pressurised hab and a drill rig staking a mineral claim",
              facts={"construct.crew_max": 20.0, "construct.power_mw": 2.0}),
    construct("nf_patrol_cutter", "ship", "Patrol cutter", "a fast fusion-drive cutter for inspection and rescue",
              facts={"construct.mass_kg": 1.2e6, "construct.length_m": 70.0, "construct.crew_max": 12.0}, tags=("nf.military",)),
    construct("nf_ice_hauler", "ship", "Ice hauler", "a slow tug pushing a block of water ice sunward",
              facts={"construct.mass_kg": 3.0e6, "construct.cargo_t": 200000.0, "construct.crew_max": 6.0}),
]
NF_RULES = [
    {"id": "nf.label.homeworld.v1", "tag": "nf.label.homeworld", "applies_to": ["body"], "when": all_of(ROCK, TEMPERATE, around_a_star())},
    {"id": "nf.label.volatile-giant.v1", "tag": "nf.label.volatile-giant", "applies_to": ["body"], "when": all_of(GIANT, around_a_star())},
    {"id": "nf.label.ice-moon.v1", "tag": "nf.label.ice-moon", "applies_to": ["body"], "when": all_of(NOT_STAR, around_a_giant(), mass_between(0.001, 0.1))},
    {"id": "nf.label.frontier.v1", "tag": "nf.label.frontier", "applies_to": ["body"], "when": all_of(ROCK, COLD, around_a_star())},
]
NF_DRESSING = [
    {"id": "nf.yard.v1", "applies_to": ["body"], "when": tag("nf.label.homeworld"), "place": "nf_orbital_yard", "where": "high-orbit",
     "tags": ["nf.faction.accord"], "faction": "nf.faction.accord",
     "explain": "a home-type world has an Accord orbital yard above it: the port where inner-system traffic docks"},
    {"id": "nf.patrol.v1", "applies_to": ["body"], "when": tag("nf.label.homeworld"), "place": "nf_patrol_station", "where": "L4",
     "tags": ["nf.faction.accord"], "faction": "nf.faction.accord",
     "explain": "the Accord keeps a patrol station at the leading Lagrange point of every home-type world"},
    {"id": "nf.depot.v1", "applies_to": ["body"], "when": tag("nf.label.volatile-giant"), "place": "nf_skimmer_depot", "where": "low-orbit",
     "tags": ["nf.faction.consortium"], "faction": "nf.faction.consortium",
     "explain": "every volatile giant has a Consortium skimmer depot: hydrogen is skimmed from its air and refined to fuel"},
    {"id": "nf.ice-station.v1", "applies_to": ["body"], "when": tag("nf.label.ice-moon"), "place": "nf_ice_station", "where": "surface",
     "chance": 0.5, "tags": ["nf.faction.consortium"], "faction": "nf.faction.consortium",
     "explain": "about half of a giant's ice moons have a Consortium station under the ice (drawn from the seed)"},
    {"id": "nf.claim.v1", "applies_to": ["body"], "when": tag("nf.label.frontier"), "place": "nf_claim_outpost", "where": "surface",
     "chance": 0.4, "tags": ["nf.faction.haulers"], "faction": "nf.faction.haulers",
     "explain": "some frontier rocks carry a Haulers' Union claim (drawn from the seed)"},
]
NF_LENSES = [lens("nf.politics", "Near future: who holds what",
                  "Each place's power and port first: the political map of the near-future setting.",
                  ["name", "dressing.faction", "dressing.where", "construct.crew_max"])]

# ---- The Meridian Reach (original space opera) ----------------------------

SO_GROUPS = [
    {"id": "so.label", "label": "Meridian Reach: worlds", "about": "How the Reach speaks of a world.", "applies_to": ["body"]},
    {"id": "so.faction", "label": "Meridian Reach: powers", "about": "Who rules, trades or hides here.", "applies_to": ["construct", "body"], "exclusive": True},
    {"id": "so.port", "label": "Meridian Reach: ports", "about": "Where a starship can put in.", "applies_to": ["construct"]},
]
SO_TAGS = [
    ("so.label.throne-world", "throne world", "so.label", "a temperate rocky world fit for a court", "derived", "obvious"),
    ("so.label.forge-giant", "forge giant", "so.label", "a giant whose moons feed the Reach's shipyards", "derived", "obvious"),
    ("so.label.rimward", "rimward world", "so.label", "a cold rocky world on the edge of a system, beyond easy patrol", "derived", "scanned"),
    ("so.label.shadow-moon", "shadow moon", "so.label", "a small moon of a giant, easy to hide behind", "derived", "scanned"),
    ("so.faction.crown", "Meridian Crown", "so.faction", "the old throne and its household fleet", "authored", "obvious"),
    ("so.faction.league", "League of Free Worlds", "so.faction", "the merchant republics that keep the starlanes open", "authored", "obvious"),
    ("so.faction.syndicate", "Ashen Syndicate", "so.faction", "smugglers and fixers who answer to no flag", "authored", "surveyed"),
    ("so.port.free", "free port", "so.port", "a port open to any ship that pays its dues", "authored", "obvious"),
    ("so.port.hidden", "hidden berth", "so.port", "a port that does not show on the charts", "authored", "surveyed"),
    ("so.military", "military", None, "armed and under a fleet's orders", "authored", "scanned"),
    ("so.fictional_drive", "fictional drive", None, "carries a starlane drive: a setting rule, not physics", "authored", "obvious"),
]
SO_TEMPLATES = [
    construct("so_free_port", "station", "Free port", "a ring station where any ship may berth, trade and refuel",
              facts={"construct.mass_kg": 2.0e10, "construct.crew_max": 60000.0}, tags=("so.port.free",)),
    construct("so_citadel", "station", "Crown citadel", "a fortress station with a household fleet's berths and a throne room",
              facts={"construct.mass_kg": 6.0e10, "construct.crew_max": 40000.0}, tags=("so.military",)),
    construct("so_shipyard", "station", "Moon-forge yard", "a starship yard fed by the mines of a giant's moons",
              facts={"construct.mass_kg": 3.0e10, "construct.crew_max": 20000.0}, tags=("so.port.free",)),
    construct("so_smugglers_den", "station", "Smugglers' den", "a hollowed rock with an unlisted berth and no questions asked",
              facts={"construct.mass_kg": 5.0e8, "construct.crew_max": 400.0}, tags=("so.port.hidden",)),
    construct("so_watch_beacon", "site", "Watch beacon", "a League listening post and starlane marker on a cold world",
              facts={"construct.crew_max": 30.0}),
    construct("so_lancer", "ship", "Household lancer", "a Crown warship, fast and proud",
              facts={"construct.mass_kg": 4.0e7, "construct.length_m": 320.0, "construct.crew_max": 300.0}, tags=("so.military", "so.fictional_drive")),
    construct("so_lane_trader", "ship", "Lane trader", "a League merchantman that runs the starlanes",
              facts={"construct.mass_kg": 1.5e7, "construct.cargo_t": 8000.0, "construct.crew_max": 40.0}, tags=("so.fictional_drive",)),
]
SO_RULES = [
    {"id": "so.label.throne-world.v1", "tag": "so.label.throne-world", "applies_to": ["body"], "when": all_of(ROCK, TEMPERATE, around_a_star())},
    {"id": "so.label.forge-giant.v1", "tag": "so.label.forge-giant", "applies_to": ["body"], "when": all_of(GIANT, around_a_star())},
    {"id": "so.label.rimward.v1", "tag": "so.label.rimward", "applies_to": ["body"], "when": all_of(ROCK, COLD, around_a_star())},
    {"id": "so.label.shadow-moon.v1", "tag": "so.label.shadow-moon", "applies_to": ["body"], "when": all_of(NOT_STAR, around_a_giant(), mass_between(0.0001, 0.05))},
]
SO_DRESSING = [
    {"id": "so.free-port.v1", "applies_to": ["body"], "when": tag("so.label.throne-world"), "place": "so_free_port", "where": "high-orbit",
     "tags": ["so.faction.league"], "faction": "so.faction.league",
     "explain": "the League keeps a free port above every throne world, by old treaty"},
    {"id": "so.citadel.v1", "applies_to": ["body"], "when": tag("so.label.throne-world"), "place": "so_citadel", "where": "low-orbit",
     "tags": ["so.faction.crown"], "faction": "so.faction.crown",
     "explain": "a throne world has a Crown citadel in low orbit"},
    {"id": "so.forge.v1", "applies_to": ["body"], "when": tag("so.label.forge-giant"), "place": "so_shipyard", "where": "high-orbit",
     "chance": 0.6, "tags": ["so.faction.crown"], "faction": "so.faction.crown",
     "explain": "most forge giants hold a Crown moon-forge yard (drawn from the seed)"},
    {"id": "so.den.v1", "applies_to": ["body"], "when": tag("so.label.shadow-moon"), "place": "so_smugglers_den", "where": "surface",
     "chance": 0.35, "tags": ["so.faction.syndicate"], "faction": "so.faction.syndicate",
     "explain": "the Syndicate hides dens on some small moons of giants (drawn from the seed)"},
    {"id": "so.beacon.v1", "applies_to": ["body"], "when": tag("so.label.rimward"), "place": "so_watch_beacon", "where": "surface",
     "tags": ["so.faction.league"], "faction": "so.faction.league",
     "explain": "the League marks every rimward world with a watch beacon"},
]
SO_LENSES = [lens("so.politics", "Meridian Reach: who holds what",
                  "Each place's power and port first: the political map of the Reach.",
                  ["name", "dressing.faction", "dressing.where", "construct.crew_max"])]


def write(rel, doc):
    out = ROOT / rel
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(doc, indent=1, ensure_ascii=False) + "\n")
    o = doc["provides"]["other"]
    print(f"wrote {out}: {len(o['templates']['templates'])} templates, {len(o['rules'])} labels, {len(o['dressing']['rules'])} dressing rules")


write("packs/near-future/pack.json", pack(
    "sse.near-future", "Near future",
    "A generic near-future hard science fiction setting: fusion drives and no faster-than-light travel, a treaty government on the "
    "inner worlds, fuel combines at the giants, and independent crews staking claims on the cold rocks.",
    NF, NF_GROUPS, NF_TAGS, NF_TEMPLATES, NF_RULES, NF_DRESSING, NF_LENSES, False))
write("packs/space-opera/pack.json", pack(
    "sse.space-opera", "The Meridian Reach",
    "An original space opera setting: an old crown, a league of merchant republics and a syndicate in the shadows, joined by starlane "
    "drives that are a setting rule, not physics (marked fictional).",
    "so", SO_GROUPS, SO_TAGS, SO_TEMPLATES, SO_RULES, SO_DRESSING, SO_LENSES, True))
