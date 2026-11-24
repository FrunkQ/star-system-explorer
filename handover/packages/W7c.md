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

## Part 2a (22 Nov 2026): land and coasts from the height map

### What was built
- **`sse_surface::climate::land_strips(grid, elevation_m, strips)`** reads land and coasts off a height map, per strip of latitude.
  - Land is the share of the strip's cells above sea level.
  - Coasts is the mean number a line round the strip crosses. Each land-sea pair of neighbours holds a piece of coast about a cell long, at right angles to the step between them. Its north-south extent is the cell size times the step's eastward share. The crossings sum those extents over the strip's width (Cauchy-Crofton). Nothing in it is fitted.
- **`with_land(x, grid, elev, strips)`** puts the strips on `Extra`, which gains `land_by_lat` and `coasts_by_lat`. It also sets `ocean_fraction` from the map, so the climate's sea is the map's sea.
- **`land_extra(s, env, seed, epoch_gyr, oceans, x, orbit_run)`** reads the strips off the **Orbit-level** (S16 guide) plate run's pre-wear heights (`wear::base_heights`, which covers both plate and sketch worlds). It places sea level for the water and gives `x` back with the strips.
  - Every level (a quick survey, a full survey) and the weather op read the same strips, so they keep one climate key.
  - `survey_world`'s chain hands over its own run when it already is the Orbit level. A full survey runs the coarse guide once more for the climate.
  - A dry world, or a giant, keeps one column.
- **Kernel `weather` op**: `body_conditions` calls `land_extra` with the universe's seed (the survey's) and keeps the result for 8 bodies and dates. A re-read, or a tile of the live weather layer, does not re-run the plates.
- `conditions()` carries the strips only for a world with a surface liquid.
- The surface_frames op (proven core) calls `land_strips` per frame with that frame's heights.

### Tests (`crates/sse-surface/src/land_tests.rs`)
- A world with land east of one meridian has land 0.5 and coasts 2 in every strip.
- A polar-cap world has no land and no coasts at the equator.
- A surveyed Earth carries 36 strips. Its ocean fraction comes from the map and it runs both columns.
- `sse-weather::the_ground_and_the_sky_read_one_climate`: the weather's conditions, with `land_extra`, still equal the survey's for Earth, Mars, Venus and Titan.

### Goldens changed (on purpose)
- **Changed:** the survey lines of `contract.jsonl`.
- **Reason:** a surveyed world with seas now takes its land and sea from its own height map instead of one global share.
- **Unchanged:** the surface goldens. They build their climates directly, without the survey chain.

## Part 2b (24 Nov 2026): currents as rules

### What was built
- **Eastern-boundary upwelling** (`formations::upwelling`, `weather_formations.currents`). Under the subtropical high, a sea place's nearness to a coast lying to the spin's east is exp(−d / `upwelling_km`), with d read along the parallel. That nearness cools the place by `upwelling_cooling_k` and is the only place the marine deck forms (Klein and Hartmann 1993). The open ocean and the far shore get no deck.
- **Western boundary currents**: within `gyre_half_width_deg` of the Hadley edge, a sea place near a coast lying to the spin's west is warmed by `boundary_warming_k` × exp(−d / `boundary_km`).
- On a backward-spinning world, east and west swap, so the upwelling and the warm currents move to the other coasts.
- Worlds with no height map keep the old deck over every subtropical sea.

### Tests
- `sse-weather::marine_decks_sit_over_upwelling_coasts_and_flip_with_the_spin`: one continent from 0° to 40° E.
  - Off its west coast: a deck and cold water.
  - Off its east coast and mid-ocean: no deck.
  - The warm current is off the east coast.
  - A backward spin swaps the two coasts.

## Part 2c (24 Nov 2026): evaporation from the sea column
- `ebm::diagnose_two` passes the sea column's temperature to the moisture block. On a two-column world, each band's evaporation is now ρ C|U| q_sat(T_sea)(1 − rh), times the band's sea share. Single-column worlds keep the old path bit for bit.
- Earth fixture: global evaporation and precipitation go from 0.86 to 0.84 m a year (about 1.0 measured). The summer sea is cooler than the band.

### Goldens changed (parts 2b and 2c, on purpose; they ship in PR 418 alongside W-wind10)
- **Changed:** `weather.json`, `weather-swap.json`, `weather-storms.json`, `surface-wear.json`, `surface-cover.json`, and the survey and weather lines of `contract.jsonl`.
- **Reason:** on two-column worlds the sea now evaporates at its own temperature, which moves rain and therefore wear and cover. Marine decks now sit only over upwelling coasts where there is a map.
- **Unchanged:** single-column worlds keep the old evaporation path bit for bit.

### Not yet (rest of part 2)
- A cell's temperature from its distance to the coast, so land cells read the land column and coasts lean to the sea.
- The ITCZ lag from heat capacity.
- Eyeball sea ice on locked worlds.
