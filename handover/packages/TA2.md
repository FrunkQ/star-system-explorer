# TA2: colour from what particles are made of, and shape from spin (kernel blocks)

Track TA (`handover/status/true-air.md`). Golden-changing in its own area only (reach report lines, and the giants area).

## What landed

- **`sse.light.particle_albedo`** (`packs/base-physics/formulas/light.json`): one particle's extinction and absorption efficiencies and its single-scattering albedo from its radius, the wavelength and its material's refractive index n + ik, by anomalous diffraction (van de Hulst 1957, sections 11.22 and 11.23). Below an absorption size of 10⁻³ the closed form's series is used (model number); a particle with n = 1 scatters nothing.
- **`sse.light.slab_reflectance`**: two-stream reflectance and transmission of a layer of particles (Meador and Weaver 1980, with Bohren 1987's coefficients so a conservative layer gives the same (1 − g)τ / (2 + (1 − g)τ) as the renderer's `weather.ts`).
- **`sse_light::haze`**: layers of particles (a sheet material, radius, optical depth at a wavelength, asymmetry), bottom first over a floor, stacked by the adding method wavelength by wavelength through the two blocks, then the eye (`colour`). Reads `optical.refractive_index` (`table`: `lambda_nm`, `n`, `k`) from the substances or mixtures table, the shape agreed with the data sheet thread (MP33). A material with no index is grey and absorbs nothing, and that is said; a table that does not cover the eye's range uses its nearest row, said.
- **`light_through_gas`** passages take an optional `particles` list and `floor`; the reply then carries `reflected` (reflectance per wavelength, the colour, `reflectance_rgb` for a renderer to multiply by, what was said, references). No new op.
- **`sse.shape.spin_flattening`** (`packs/base-physics/formulas/shape.json`, family `sse.shape`) and flow **`sse.base.body-shape`**: Darwin-Radau flattening from mass, mean radius, rotation period and the moment of inertia factor C (Murray and Dermott 1999, section 4.6); the spin parameter uses the equatorial radius, found in one step from the mean radius. Built here at the exotic stars thread's request (XO "shape from spin and a companion's pull"); the later tidal block `sse.shape.spin_and_pull` takes its output. C comes from the proven core's coming fact `body.moment_of_inertia_factor`.
- Quantity kinds `particle.*`, `slab.*`, `shape.*` described; atom count 122 to 125 (P1.md).

## Tests

- `sse-light` `haze_tests`: a material absorbing blue more than red sends back orange; one with no index is grey and said; a thick layer hides a bright floor; the slab block meets Bohren's conservative limit and R + T = 1, a thick absorbing slab lets nothing through; a large non-absorbing sphere extinguishes twice its cross-section; the series and closed form agree at the switch; index tables are checked and read between rows.
- Flow `sse.base.body-shape` tests: Jupiter (0.06487 measured, tol 0.005), Saturn (0.09796, tol 0.008), Earth (0.003353, tol 0.0002).
- Goldens change on purpose, after the golden split (PR 536): the new area `tests/contract/areas/giants.json` with its golden `tests/golden/areas/giants.jsonl` (an ammonia deck under sunlight through `light_through_gas` with `particles`), and three new block lines in `tests/golden/contract-reach.txt` (sse.light.particle_albedo, sse.light.slab_reflectance, sse.shape.spin_flattening). The contract runs on the frozen base-physics fixture, so `contract.jsonl` does not move. No world's values changed.
- Fudge: none new (the grey fallback is said in the reply; the renderer's "Air in True" row covers its use).

## TA6 in the same pull request: giants drawn flattened

- `packages/renderer/src/true-shape.ts`: `readTrueShape` runs the flow `sse.base.body-shape` through `derive` on the
  world's own `resolved_system` facts (`body.mass`, `body.radius`, `body.rotation_period`,
  `body.moment_of_inertia_factor`). A world missing any is drawn round and the reason names the fact. `flattenScale`
  keeps the drawn volume.
- `GpuItem.flatten` and `GpuItem.pole` (optional), `TrueBody.flatten`: the globe is drawn shorter along its pole
  (its frame's z, or `pole` when there is no frame) and wider round it; clouds follow the same shape.
- The system chart's True reader and the size comparison read the shape with the face; the HUD note says
  "flattened by its spin: the pole 6.5% shorter than the equator (Darwin-Radau)".
- A world without `body.moment_of_inertia_factor` is drawn round and says so. Since the proven core's PR 417 (MOI1,
  25 Nov) Sol's worlds flatten by themselves: Jupiter 6.3%, Saturn 10%, Uranus 1.7%, Neptune 1.6%, Earth 0.33%.
- Test `packages/renderer/test/true-shape.test.ts`: the kernel flattens Jupiter and Saturn by about their measured
  amounts from their facts; a world without C is round and says which fact is missing; the volume is kept.
