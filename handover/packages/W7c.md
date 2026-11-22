# W7c: land, sea and seasons

Weather review (`handover/design/SSE4-weather-review.md`) section 8, package W7c (the accepted Rock 3 W7).
The aim: heat capacity per place, so that coasts are mild and interiors harsh. Also current rules, the ITCZ lag and eyeball ice.

## Part 1 (22 Nov 2026): land and sea columns in each band

### What was built
- **Two columns per band** (`ebm::cycle_two`). A world with a surface liquid and some land runs a land column and a sea column in every band.
  - Land: ground skin plus air. Sea: mixed layer plus air (`ebm::column_capacities`).
  - Diffusion acts on the band's mean (f T_land + (1 − f) T_sea), and its convergence is shared per square metre. Heat crossing the coasts nets to zero over the band.
  - Each step is one implicit 2 × 2 block-tridiagonal solve (`block_thomas`). A band with no land ties the land column to the sea (and the reverse), so the solve keeps one shape.
  - One base albedo keeps the band-weighted Bond albedo (`albedo_two`).
  - Worlds without both (no liquid, or no land) keep the old single-column path, bit for bit.
- **Coupling** (`ebm::land_sea_coupling`), per square metre of band, from the model's own numbers. Nothing in it is fitted.
  - Diffusion: n coasts round the band, blocks alternating. Heat crosses each coast down a gradient reaching into each side by the lesser of half the block and ℓ = R √(D / |B + iωC|). That gives ν = n D R² / (L r).
  - Zonal advection: the air column (c_p p / g) moves with the column-mean wind. That wind is the thermal wind at the thermal depth times `column_wind_ratio` ½, which is the mean of a wind rising linearly with height. The air crosses one windward coast in two and relaxes to the new ground over u τ, with τ = C_air / (ρ c_p C|U| + B). C|U| is the moisture block's bulk exchange.
  - Capped at `max_coupling_w_m2_k` 50, which binds only near the poles.
- **Inputs on `Conditions`**: `land_by_lat` (land fraction in equal latitude strips, south to north, resampled onto the bands by overlap) and `coasts_by_lat`. Both are empty by default: every band then holds the world's land fraction and the pack's `coasts_per_band` 2. Evaporation weights each band by its own sea share when strips are given.
- **Outputs**:
  - `Season::t_land_k` and `t_sea_k`, and `Climate::land`, present only on two-column worlds.
  - `Climate::t_land_at`, `t_sea_at`, `land_range_at` and `sea_range_at`, which fall back on the band's values.
  - A why line naming the band where land and sea differ most.
- **Dated ice-age onsets**: `IceAgeModel::onsets_yr` returns the sample times at which the ice balance turns positive, from the same series `at()` counts. They are asked for by the surface_frames op (proven core with the renderer).
- Pack `climate.land_sea`: `coasts_per_band`, `max_coupling_w_m2_k`, `column_wind_ratio`, `source`.

### Results (Earth fixture with Earth's land in 10° strips)

| | Diffusion only | With advection (built) | Observed |
|---|---|---|---|
| Land range at 52.5° N | 57 K | 32 K | about 30 to 35 K |
| Sea range at 52.5° N | 6.5 K | 9.7 K | about 8 to 10 K |

- The band's range at 50° N is more than twice that at 50° S.
- The global mean holds within 1 K.
- With advection the Antarctic coast bands no longer swing 89 K.

### Tests (`crates/sse-climate/src/land_sea_tests.rs`, plus one in `forcing_tests.rs`)
- Interiors swing more than seas (land more than 2.5 × sea at 42.5°, 52.5° and 62.5°), inside the observed bounds at 52.5° N, with the north more than the south.
- The band is exactly its land and its sea. The global mean is kept.
- More coasts couple land and sea more tightly.
- 36 strips duplicating 18 give the identical climate.
- Mars, Venus and the Moon keep one column. A water world's columns are tied.
- `onsets_yr` gives as many dated onsets as `at()` counts advances, more than half a tilt period apart, and none where the ice never melts.

### Goldens changed (on purpose)
- **Changed:** `weather.json`, `weather-swap.json`, `weather-storms.json`, `surface-cover.json`, `surface-wear.json`, and the survey and weather lines of `contract.jsonl`.
- **Reason:** worlds with both land and a surface liquid (Earth, Titan) now run a land and a sea column per band, as accepted in W7c.
- **Unchanged:** worlds without both keep the single-column path bit for bit. `surface-regimes.json` and the other goldens did not move.

### Not yet (part 2)
- Land and coasts counted from the height map per strip, where the surface fills `land_by_lat` and `coasts_by_lat`.
- A cell's temperature from its distance to the coast, so land cells read the land column and coasts lean to the sea.
- Evaporation from the sea column.
- Current rules: western boundaries warm, eastern upwelling cold, marine decks only over upwelling. A retrograde fixture's currents flip.
- The ITCZ lag from heat capacity.
- Eyeball sea ice on locked worlds.
