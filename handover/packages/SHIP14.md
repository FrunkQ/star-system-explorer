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

## Stage 2: ships designed from their parts

Alex, 23 Nov: design a ship from its components, sizes worked out from physics, styles from hard-science spines to
high tech, assignable to a nation or culture, remixable, overridable; the 2D is "a simple top-down and side view
with rough call-outs", not deck plans. He loves the giant radiators. Coordinator: add crew and cargo fields to the
stock ships with the travel thread (it adds `crew`, `cargo_t` and `fuel_shares` to NavShip and base-nav).

- **Rules** (`packs/base-ships`, `ship_design`, each with a source): habitable volume per person and net to gross,
  cargo stowage density, radiator emissivity and faces, and drive heat by energy source.
- **Needs** (`needsOf`): quarters from crew, hold from cargo, tanks from `tanksOf`, waste heat from jet power, and
  radiator area by Stefan–Boltzmann: A = P / (2 ε σ T⁴). Hand overrides (`crew_m3`, `cargo_m3`, `radiator_m2`) win
  and are said in the words.
- **Styles** (`ship_styles`): "Spine and modules" (open truss, tank rings, radiator wings off the middle) and "Sleek
  hull" (one skin, pointed nose, swept fins). A style arranges and never resizes: the test checks that every
  drawn part holds exactly its need in both styles.
- **Layout** (`designShip`): solids along the ship's axis, kept as a recipe `{ style, seed, overrides }`. `remixes`
  picks inside the style's ranges (tank ring count, tank fineness, drive gap, nose, wing shape) with a seeded
  generator, so a recipe rebuilds the same ship anywhere.
- **Views** (`shipViews`): side and top at one scale, call-outs above and below that never overlap, the plume behind
  the stern, and a "hull" framing that lets kilometre radiators run off the frame. `ShipPlans` (packages/views/ship)
  draws them with a style switch, Remix and "Close in on the hull", inked from theme tokens.
- **Honesty**: a sleek hull on a hot drive says it has nowhere to put the radiators ("its fins would reach 33 km
  out on each side. Ships with drives this hot carry radiators on booms").

Pictures: `SHIP14-screens/design-cool-drives.png` (chemical, nuclear thermal, jump courier in both styles),
`design-hot-drives.png` (torch, cutter, probe framed whole: the radiators dwarf the ship) and `design-hot-close.png`
(closed in on the hull). Checks: `packages/renderer/test/ship-design.test.ts`, 5 tests.

## Stage 3: the 3D model

- `ship-mesh.ts` (pure): `shipMesh(layout, { segments, panel_m })` turns the same layout the plans draw into one
  triangle buffer per surface (hull, tanks, radiators, spine, drive) with normals and texture coordinates in real
  metres over the style's panel size, plus bounds for the whole ship and for its modules alone. `panelTexture`
  makes a tileable 4 × 4 panel texture (seams, shade per panel, wear) from a seed.
- `gpu-ship.ts`: `mountShip` (exported lazily as `startShip`, so three.js loads only when a model is shown) wraps
  the buffers in materials from the style's `skin` (pack data: albedo, roughness, metalness per surface), one star
  and a fill light, a logarithmic depth buffer (kilometre radiators beside metre modules), mipmapped panels, and
  hot radiators that glow with the colour of a body at their temperature. Built once; later draws only turn the
  camera. "hull" framing closes in on the modules.
- `ShipPlans` now carries the shared ViewControls: Chart is the plans (flat), any other "Seen from" turns it into
  the 3D model; Free drags to turn and wheels to zoom. True and Wireframe are listed and say why they wait.
- Fuels from the master data sheet (PR 223): `readPropellants(session, fuels)` reads mixture `fuel-<id>` (shares by
  mass) and each substance's `phases.density_liquid`; antimatter has no row, so its tank stays a marked stand-in.
  Views no longer need typed densities; the tests' NIST fixtures remain only in the pure tests.
- Free view in `ShipPlans`: drag turns and tilts, Shift-drag or two fingers slide, wheel or pinch zooms (the shared
  hint's promise).
- Checks (`ship-mesh.test.ts`, 4 tests): every closed body's mesh volume (divergence theorem) equals its part's
  volume to 1e-6 of the polygon's share; radiator faces add up to exactly the area the waste heat needs; normals are
  unit and agree with winding; bounds match the layout; under 5,000 triangles; textures repeat per seed.
- Picture: `SHIP14-screens/model-3d.png` (chemical, nuclear thermal, jump courier, rescue cutter closed in, and the
  fusion torch closed in and whole: a 15 m crew module on a 785 m spine over 3.8 km² of glowing radiator).

## Stage 4: abilities shape the ship

Alex, 23 Nov: "aerobraking ships need to look like they can", "ships that can land will have aero". The designer
does not judge entry itself; the kernel's `ship_at_world` (travel thread, PR 228) does.

- `ship-abilities.ts`: `fitAbilities(ask, where, ship, layout, hull, { aerobrake, land }, rules)` writes the NavShip
  fields `aero`, `shield` and `landing` from the layout and the ship's abilities, asking the kernel at each step and
  scaling each part by the law its figure follows until it sits at the margin (75%, the kernel's own "would pass"
  mark) of its limit:
  - **shape in air**: frontal area from the layout, drag coefficient by hull (open truss, skin, blunt shield);
  - **heat shield** wherever the ship meets air at speed: nose radius from q ∝ 1/√r_n, the lightest material that
    fits within three hull radii (tiles, then PICA, then carbon phenolic), mass from tiles' areal density or the
    pass's heat load over the ablator's heat of ablation; drawn as a blunt sphere-cone leading the ship, and a
    pointed nose is cut short to carry it;
  - **lift** for a landing world with ground and air when the drive cannot hover there: wing area from v_s ∝ 1/√S
    against the runway speed; a lifting body if the hull's own planform does it, else delta wings (aspect 2.3,
    swept 45°); air too thin for wings that fit says so;
  - **legs** (or wheels under a winged ship) and a **parachute** sized for the legs' touchdown where the air can
    carry one (A = 2 m g / (ρ C_d v²)); where it cannot, the words say the drive brakes the last of the way;
  - **scoop** for a ship that refuels by skimming (base-nav `refuel: scoop`): intake area = propellant / (air
    density × speed × time).
  - Notes in words: "Its radiators must fold behind the shield before it meets the air", shield mass, worlds where
    no part could be made big enough (drawn at its largest; danger is drama, never forbidden).
- `designShip(..., fit)` draws the fitted parts (`shield`, `wings`, `legs`, `chute`, `scoop`) in the plans and the
  model; abilities never resize the systems. The model has a `shield` surface (dark ablator look per style).
- `fittedChecks(result)` picks the kernel's checks for the acts fitted for; `ShipPlans` takes `abilities` (a
  FitResult) and lists them under the words, "past a limit" in the alert colour.
- Call-outs: a row too long for the frame splits into two staggered lines.
- Rules and their sources are pack data (`base-ships` `ship_design.abilities`), all marked unverified; fudge
  register row "base-ships `abilities`". `scripts/audit-matter.py` lists the scoop's skimmed air density as a
  designer setting, not a material number.
- Checks (`ship-abilities.test.ts`, 5 tests, against the kernel through `nodeKernel`): no kernel call when nothing is
  asked; a Mars lander's shield keeps aerocapture and entry heating inside the margin and weighs its areal density
  over its face; wings land exactly at the margin of the runway speed; a cutter that can hover gets no wings, and its
  scoop is the propellant over density, speed and time; the fitted parts appear in plans, call-outs and the model.
- Pictures: `SHIP14-screens/abilities-plans.png` (a Mars lander, a winged surface shuttle, the jump courier landing
  at Earth and Mars, the fusion torch aerobraking at Jupiter with a 27 m carbon phenolic shield),
  `abilities-3d.png` (the same in 3D) and `abilities-card.png` (ShipPlans with the kernel's verdicts).

## Stage 4b: on the Voyages card, and the aerobraking pass

- `ShipAtWorldView` (views/ship) replaces the drawn ship on the Voyages "At world" card (`AtWorld.svelte`): the ship
  designed from its record through `ShipPlans`, and "Fit it for {world}" running `fitAbilities` for what the world
  asks (aerobrake where there is an aerocapture check, land where there is entry or a powered landing). The card keeps
  its own single `shipAtWorld` call per ask; fitting asks the kernel again only when pressed.
- "Watch the pass" shows when the fitted ship carries a shield and the kernel gives an aerocapture peak heating. It
  opens the 3D model in pass mode (`mountShip(..., { pass })`, framed on the front: `ShipMesh.front_bounds`, the
  modules without spine, radiators or drive): radiators folded away, the shield glowing at its face temperature,
  the bow shock ahead of its nose.
- `passGlow(q, ε)`: the face at radiative balance, T = (q/(εσ))^¼ with ε from the pack (`abilities.shield_emissivity`
  0.85); the shock stands off 0.143 nose radii and curves at 1.143 (Billig 1967, sphere at high Mach); its glow
  (colour of a body at 1.6 T, opacity by heating up to 1 MW/m²) is drawn for the eye.
- The shell copies `packs/base-ships/pack.json` to `public/packs/base-ships.json` for the view to fetch.
- Checks: `ship-abilities.test.ts` gains the balance test (εσT⁴ equals q, hotter for more heating, Billig's numbers);
  `tests/ui/voyage.e2e.mjs` checks the plans on the card and presses "Fit it for", and watches the pass when the
  destination has air to brake in. The test drive's Kestrel worlds carry no air, so the browser run fits without a
  pass there; the pass is pictured from the kernel's own fits.
- Pictures: `SHIP14-screens/pass-mars-lander.png` (a Mars lander's PICA shield at about 1,440 K) and
  `pass-torch-jupiter.png` (the fusion torch's carbon phenolic shield at about 4,270 K aerobraking at Jupiter).

## Next

- **Sheet**: `density_liquid` for hydrogen, oxygen, deuterium and helium-3, and a substance for each base-nav fuel
  (asked of the matter data sheet thread). Mixture ratios in the ship record (galaxy and travel thread).
- A True plume from the exhaust's own emission, and the model in the True exposure chain.
- **Stage 5**: styles assigned per nation or culture, inherited down the tree like name schemes.
- Abilities on the ship record (where it must aerobrake and land) once the travel thread's ship editor carries them;
  until then the owner passes them in the recipe (`DesignRecipe.abilities`).
- **Uploads**: GLB, OBJ and STL hulls with credit fields once packs can carry files.
