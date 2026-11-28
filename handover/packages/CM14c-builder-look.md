# CM14c part 1: the builder's parts drawn as what they are (compound ships)

Track CM. Builds on CM14b (the builder to the mock-up, PR 746). Mock-up: `sse4/handover-screens/CM-builder-mockup/` (Alex: the builder is not done until it looks like it). Built 28 Nov 2026 by the thread "Build the snap-together ship builder". No kernel change, no golden moves.

## What it does

- **Each part drawn as what it is** (`packages/renderer/src/builder-shapes.ts`), from its kind and shape at its own length and radius: tanks and stages with domed ends (stages banded), habitat cans with raised ribs and two rows of windows, cargo pods with bands, a docking node as a sphere with four ports, trusses as a lattice, solar wings as blue cells and radiators as flat panels reaching straight out along their clock, dishes as a bowl on a stalk, engines as a bell with a throat and chamber, power plants with fins, bays open along the top with end rings, sensors, shields and legs. Before this every part was a plain cylinder.
- **Looks from the style's skin** (pack data, `base-ships` `ship_styles.*.skin`): white blankets on the modules, foil on the tanks, bare metal truss, dark radiators. Only solar cells, windows and dark openings are fixed in the renderer (the skin has no entry for them yet). Studio lighting: bright sky, key light over the viewer's shoulder, soft fill.
- **The ship's name lettered** along the biggest crew or tank part, on the viewer's side, in ink that stands off its paint.
- **The parts list grouped by use**, as the mock-up does: Crew and rooms, Tanks and stages, Drives, Power and heat, Structure, Sensors and dishes (`paletteByUse`); in each, shapes with no year first, then by first flight. The year slider still fades parts not flown by its year.
- **Fix:** the tanks' propellant read keeps only the latest answer (an older answer arriving late could overwrite it). The builder check waits for the propellant read before its picture, so the first picture shows the real delta-v.

## What it does not do yet

- **Example ships are still the catalogue's four, opened by converting the old generator's layout**, so three of them are long trusses with small parts. Part 2: example ships assembled from parts as the mock-up's are, with the crew capsule, docking port, heat shield and legs they need added to the palette. Their masses are the open question: generic parts' masses are still to be read (tank walls from `sse.ship.tank_wall_mass`, engines have none from GCAT).
- Cutaway, Sensor views and "Build it for me" (later slices). No logo decal. Panels' cells are one texture, not sized to the panel.

## Checks

- `packages/renderer/test/builder-shapes.test.ts` (3 tests): every palette part draws within its own size, a hull panel reaches straight out along its clock, a tank has domed ends, the name goes on the biggest crew or tank part. Seen red with the panel turn reversed and the dome removed; green restored.
- `packages/ui-contract/test/parts.test.ts`: the parts list by use, every part once, ordered by first flight.
- `npm run ui:test` 1266 pass; `ui:check` clean; ship-builder, manifest and voyage e2e pass.
- Pictures: project files `sse4/handover-screens/CM14c-screens/`.

# CM14c part 2: example ships assembled from parts

- **Six example ships built from the palette's parts**, as the mock-up's are (pack data, `packs/base-parts/pack.json` `provides.other.examples`, built by `scripts/build-base-parts.py`, which checks every part exists and every clip comes after its carrier): Kestrel (a 1970s capsule, service tank and two solar wings), Harbour station (modules on a node), Highland lander (capsule over a squat tank, strap-on tanks, four legs), Long-hauler (crew forward, long spine, shadow shield, nuclear rocket), Bulk frame (sixteen painted cargo pods on a spine) and Wayfarer (a survey carrier with its lander in a bay). Each is only a recipe: open it from Example ships, under "Assembled from parts", and pull it apart. The catalogue's four stay below, under "From the catalogue (converted from the old layout)"; the builder still opens on the first of them, since only those have masses yet.
- **New generic parts** they need: crew capsule (a habitat drawn as a capsule), docking port (the docking standard's passage), heat shield, radiation shadow shield and landing leg. Geometry figures only; each names what is still to be read. `PartShape.look` is a picture hint (capsule, port) no figure reads.
- **No mass is invented** (coordinator's ruling 28 Nov 18:50, from Alex's "we dont fudge unless we HAVE to"). A build with a part that has no mass, and nothing carried over standing in, says "needs part masses" for delta-v, mass and push, names the parts, and "What can it do?" waits for them. The examples type no structure mass. Part 3 reads real masses: tank walls from `sse.ship.tank_wall_mass`, engines from a cited published table in the reference pack.
- Fixes: a bay is open along its top (it was open on the far side); a landing leg's foot is at the stern end.

## Checks (part 2)

- `builder-shapes.test.ts` gains: every example places all its parts, has a part to letter its name on, types no structure mass and says it needs part masses; a bay is open along its top (seen red with the old opening, green restored).
- `ship-builder.e2e.mjs` opens Kestrel from Example ships: eight or more parts at once, "needs part masses" shown, "What can it do?" disabled. Picture `example-*.png`.
- `ui:test` 1271 pass; `ui:check` clean; `build-base-parts.py --check`; ship-builder, manifest and voyage e2e pass.

# CM14c part 3a: Cutaway

- **Cutaway** (the mock-up's chip, now live): bays and habitats open on the viewer's side, so what they hold shows: Wayfarer's lander in its bay, a habitat's deck. Geometry only (the shell is drawn with a gap, no clipping planes, so it works the same on WebGPU, WebGL2 and software drawing). The name is not lettered on a cut-away habitat.
- Check: `builder-shapes.test.ts` "cut away, a bay and a habitat open on the viewer's side" (seen red with the cut ignored); the e2e opens Wayfarer, presses Cutaway and takes `cutaway-*.png`.

# CM14c part 3b: tank walls and engine masses

Coordinator's rulings 28 Nov 18:50 and 19:08 (no structure mass authored; tank walls from the wall block; engines from a cited table; any part still without a mass says "needs part masses"; the app says where each mass came from).

- **Tank walls from the kernel.** New flow `sse.base.ship-tank-wall` (`packs/base-physics/flows/ship-tank-wall.json`) wraps the block `sse.ship.tank_wall_mass` (m = φ p V ρ / σ, Roark's thin-wall shells), with two golden cases (the block's own worked example, and a long cylinder against a sphere). base-physics gains one unit, `m3`, which the built-in table lacked (`sse_atoms::units()`); the contract runs on the frozen fixture, so `contract.jsonl` does not move.
- **What a tank's wall reads**, all cited and editable: the tank part's design pressure (40.0 psia, the Centaur oxygen tank's structural limit, NASA TN D-6876) and design factors (ultimate 1.4, yield 1.25, NASA-STD-5001B Table 1) in the parts pack; its wall material, the new data sheet row `al-2219-t87` (density 2840 kg/m3, ultimate 476 MPa, yield 393 MPa, Aluminum Association typical values) in the reference pack. The allowed stress is the lower of ultimate over 1.4 and yield over 1.25 (314 MPa here). A 4 m tank's wall comes to 501 kg. `renderer/ship-walls.ts` reads them and asks the flow once per tank shape (a stretched tank asks again).
- **Engine masses** from a cited published table (Wikipedia's comparison of orbital rocket engines, unverified): F-1 8,391 kg, J-2 1,438, SSME (RS-25) 3,526, Merlin 1D-5 470, AJ10-190 118. GCAT's engine list as served here gives no liquid engine's mass, and the RL10A-3 is not on the table, so the examples' RL10A-3 still needs a mass.
- **Where a mass came from**: the selected part says it ("Mass: 501 kg, wall from the thin-wall pressure block ..." or "1.4 t, dry mass from wiki-engines (the page's J-2 row), not yet checked against the source"), or that it is not known yet.
- Fudge register: F-CM14c-wall-shape (every tank weighed as a long cylinder, so heavy rather than light), F-CM14c-tank-pressure (one design pressure for every tank), F-CM14c-engine-mass (a comparison table, not makers' sheets). Each row names the real physics that closes it.

# CM14c part 3c: mass by kind

- **One cited, editable row per kind** (`packs/base-parts/pack.json` `palette.mass_by_kind`, built by `scripts/build-base-parts.py` `MASS_BY_KIND`): a generic part with no mass of its own weighs what a flown example of its kind weighs per unit of the same figure the part carries. Solar panel 3.3 kg/m2 (a Dawn wing, 63 kg over 8.3 by 2.3 m); radiator 14.2 kg/m2 (an ISS heat rejection radiator, 1,122.64 kg over 23.3 by 3.4 m); habitat and crew capsule 117.6 kg/m3 (Destiny, 14,515 kg in a can 8.5 m by 4.3 m); node and docking port 148.9 kg/m3 (Node 1, 11,895 kg, 5.5 m by 4.3 m); cargo pod 37.7 kg/m3 (the MPLM empty, 4,082 kg, 6.6 m by 4.57 m). All unverified until the citation run-through. They live in the parts pack beside the parts' other figures, not the reference pack, which keeps materials; a pack can replace any row.
- The selected part says which row it used and the flown example behind it.
- A bare truss now needs a mass too (it was silently counted as nothing).
- Register: F-CM14c-kind-mass, with the loop (a sizing block per kind).

## What it does not do yet (parts 3b and 3c)

- No row yet for shields, legs, dishes, bays or trusses, and no cited mass for the RL10A-3 or the tech drives, so every example still says "needs part masses" (each names what it lacks). Sources not found in this pass; the next is a truss segment, a flown dish and the Apollo heat shield.
- Solid motors: GCAT's masses for them are loaded masses, so they need a dry and propellant split before they count.

## Checks (part 3b)

- `cargo test -p sse-atoms base_physics`: the flow's golden cases pass (seen red with a wrong expected mass).
- `packages/renderer/test/ship-walls.test.ts` (2 tests): a 4 m tank's wall through the flow from its cited figures and the sheet's alloy, absolute 501 kg, counted in the totals with its source and asked only once; a J-2 counts its 1.438 t and a habitat still needs a mass. Seen red with the wall left out of the totals.
- `ship-walls.test.ts` third test: a habitat and a panel weigh by their kinds' rows (absolute 11.89 t), say so, and need nothing more. Seen red with the kind rows ignored.
- `ship-builder.e2e.mjs`: picks Kestrel's tank from the build tree and sees its wall mass from the block. Picture `tank-mass-*.png`.
