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
