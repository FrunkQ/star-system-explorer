# V3 gap item 14, part 1: ship hulls and drive plumes from physics

V3 gap list (`/mnt/project-files/sse4/SSE4-v3-gaps.md`) item 14: "3D hulls and drive plumes". V3 let a user upload a
GLB, STL or OBJ and scaled a plume to thrust. Alex chose on 23 Nov: **hulls built from each ship's physics now, file
uploads later** (packs cannot carry files yet, and the rendering design says assets are generated, not shipped).

## What was built

`packages/renderer/src/ship.ts`, exported from the renderer's index. It reads the same ship record that plans a
voyage (base-nav `ships`, ui-contract `NavShip`) plus the fuels it lists.

- **Drive** (`driveFacts`): the relativistic rocket (Ackeret 1946), which is Tsiolkovsky's at chemical speeds.
  Thrust is wet mass × accel_g × g0 (base-nav quotes accel_g at full tanks). Mass flow F/(γ v_e), jet power
  ṁ(γ−1)c², change in speed c·tanh((v_e/c) ln(m0/m1)), full-thrust burn time, and the energy each atomic mass unit
  of exhaust carries, with the shortest light that energy can make (h c / E). Constants come from the sheet.
- **Hull** (`hullFromShip`): one tank per stored part of each fuel, volume = mass / stored density, so liquid
  hydrogen's tank dwarfs liquid oxygen's even at a sixth of the mass. A crew module whose skin is the shield
  (shield_g_cm2 × area), and an engine bell. The hull radius is the one at which the tanks make a cylinder four
  diameters long. A fuel the sheet cannot size is drawn at a stand-in size and the words say so.
- **Outline** (`hullProfile`): (along, radius) pairs for three.js `LatheGeometry`, the GPU tiers' hull.
- **Chart** (`shipPaths`, `plumeShape`): side-view SVG paths (hull, part dividers, plume). The plume glyph's
  length reads jet power on a log scale. Inks come from the view's theme.
- **Words** (`narrateShip`): for example "Fusion torch: 79 m long and 15 m across, built round 11,568 m³ of tanks.
  Thrust 20 MN; the jet carries 20 TW and the tanks last 28 hours at full thrust, for 1,386 km/s of change in speed.
  Each hydrogen atom's weight of exhaust carries 21 keV: where the jet stops, that is enough for light up to X-rays."

- **View** (`packages/views/ship`, `ShipPicture({ ship, fuels, height?, words? })`): the Chart side view and its
  words in the theme's inks (`accent` plume, `ground-raised` hull, `ink` outline). The galaxy and travel thread
  mounts it on the Voyages ship card and the route card's ship line.
- **Fuel shares**: an optional `fuel_shares` on the record (the travel thread is adding it to base-nav, with cited
  reasons) splits the propellant; without it, fuels share equally.
- `constant`, `constantOf`, `constantIds` and `SHEET_PACK` are now exported from the ui-contract index, so packages
  outside it reach the sheet through the public API.

Densities are passed in by the view from the sheet's substance rows (`phases.density_liquid`); nothing is typed.

## Checks (`packages/renderer/test/ship.test.ts`, 5 tests, all six base-nav ships)

- Chemical ship: thrust, mass flow, jet power ½Fv, Δv = v_e ln(m0/m1) and burn time to 1e-9.
- Antimatter probe: Δv is c tanh(...), below c and below the classical answer; momentum flux equals thrust; its
  exhaust can make gamma rays, the fusion torch's X-rays.
- Fuel shares from the record split the mass (deuterium and helium-3 at 2.014 : 3.016).
- Every tank holds its propellant at its density; parts run end to end; shield mass is σ × skin; the jump courier
  carries its jump tank; antimatter is a marked stand-in.
- The outline builds a lathe as long and wide as the hull. Chart paths fit their frame; the words carry the facts.

Picture: `SHIP14-screens/ships-chart.png` (all six ships, Chart style, with densities from NIST as test fixtures).

## Fudge register

Three rows: hull proportions (HULL_SHAPE), equal fuel shares and the stand-in density, and the Chart plume glyph.

## Next

- **Sheet**: `density_liquid` for hydrogen, oxygen, deuterium and helium-3, and a substance for each base-nav fuel
  (asked of the matter data sheet thread). Mixture ratios in the ship record (galaxy and travel thread).
- **Part 2**: the GPU hull (lathe, lit like a body) and a True plume from the exhaust's own emission; mounting on the
  ship card and Voyages screen (galaxy and travel thread's screens).
- **Uploads**: GLB, OBJ and STL hulls with credit fields once packs can carry files.
