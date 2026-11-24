# ST-gen: stations and land constructs built from their description

Status: done (24 Nov 2026), branch `claude/project-thread-l1e9pw`. Asked by Alex on 24 Nov: generate stations from their description, about two dozen shapes, the same shape at every size with bolt-ons, docking points by size, a cut-off where it becomes a megaconstruct, and a self-contained builder that can be swapped later.

## What was built
- **Builder** (`packages/renderer/src/station-design.ts`): `designStation(record, rules, shapes, limits, recipe)` returns a `StationLayout` of plain axis-aligned parts (tube, torus, sphere, box, panel). Its other outputs are the spin, the berths per class, the power, the mass, a `mega` verdict and notes. It reads nothing else and writes nothing, so a better builder can replace it behind the same record and recipe.
  - `pickShape` chooses the shape from the construct's words (template explain, tags, modules, name).
  - `stationPackData` reads the inputs from the packs.
  - `narrateStation` describes the station in words.
  - `stationViews` draws the plans from above and from the side, with call-outs.
- **Pack data** (`packs/base-ships`, 0.2.0):
  - `station_design` holds the sizing rules.
  - `station_shapes` holds 28 shapes: 19 in space (9 of them spun) and 9 on the ground, in the air or at sea. The derelict ship is one of them.
  - `station_skin` holds the 3D look.
  - The spin rate and the strength ladder are read from base-tech's figures `spin_rpm` and `strength_for_weight`, which come from the sheet.
- **Sizing:**
  - **Volume:** people × the living rule. A working post is 50 m³ a head, from the ship rule; a home is 6,900 m³ a head, the Stanford torus.
  - **Spun shapes:** the radius is at least g/ω² at the comfortable spin.
  - **Power and heat:** the record's power, or 10 kW a head. Solar panels supply it; in space, radiators at 300 K shed it.
  - **Berths:** one per 500 people, doubled at a port. Each class up has half as many. The biggest class takes ships up to half the station's size. On the ground the berths are landing pads.
  - **Bolt-ons:** each listed `construct.modules` entry becomes a labelled bolt-on, with more added per decade of people.
- **Megastructure line (physics, never hand-set):**
  - **Spun hull:** needs strength over density of g·R × (1 + load) × safety. Beyond steel wire (the best built material on the ladder) it is a megastructure, naming the material it would need. Beyond the carbon bond it is marked fictional.
  - **Tower:** stands only to the height strength/(ρ g) allows.
  - **Own gravity:** when the escape speed from its own gravity passes 1 m/s, it is a body.
- **3D:**
  - `station-mesh.ts` builds one buffer per surface. `core_bounds` leaves out the power arrays.
  - `gpu-station.ts` (`startStation`) uses the ship's panel texture.
- **Views:**
  - `views/ship/src/StationPlans.svelte` shows the plans or the model. It has a Shape choice and Remix, uses the shared view controls (Seen from reframes), and opens closed in on the station with the power arrays left out. `locked` hides Shape and Remix.
  - `StationPicture.svelte` builds the record from a construct.
  - On the Ships and stations card, a station has a **Picture** button. On the Build desk a change is kept as `construct.design` (a JSON recipe); other desks are locked.
- `scripts/build-shell.sh` now copies base-tech for the builder.

## Checks
- `packages/renderer/test/station-design.test.ts` (8 tests):
  - every shape builds, meshes and draws;
  - 30 and 20 million people give the same ring, about 300× bigger;
  - the spin rule;
  - the megastructure line at 20 million, and fictional past the carbon bond;
  - a tower's height, and a body by its own gravity;
  - the words pick the shape;
  - the same recipe gives the same station.
- `tests/ui/stations.e2e.mjs`: Traveller templates pick a disc (highport), a hollowed rock (corsair base) and a truss (research station). It also switches to a ring and draws the 3D model.
- Screens are in project files, `sse4/handover-screens/ST-gen-screens/`.
- Fudge rows: "Station sizing rules", "Station megastructure line", "Station shapes".

## Next
- The manifest thread's `Pictures` component (PR 391) can host StationPlans beside the ship views.
- Ground placement: the record's `place` and the world's gravity are not yet read from where the construct sits. Today every station is in space unless the caller says otherwise.
- Power from the construct's own plant, which waits on the RK power books. A fusion port needs radiators, not solar panels.
