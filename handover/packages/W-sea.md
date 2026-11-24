# W-sea: tides and wind waves

Asked by Alex on 23 Nov 2026 ("tides and waves, YES!"). Owner: "Build shared climate and weather". Tides come from the real bodies that raise them, and waves from wind, fetch and gravity. Both are named, cited Equation blocks, shown on a world's card and at a landing site, so the renderer draws waves from the same figures.

## What was built
- **Formula blocks** (`packs/base-physics/formulas`):
  - `sse.tides.ocean_range` (tides.json): the open-sea equilibrium tide range H = 3/2 γ (m/M)(R/d)³ R. Murray and Dermott 1999, ch. 4. Param `diminishing` γ is 0 to 1, default 1.
  - `sse.tides.period` (tides.json): T = 1 / (2 |1/P_spin − 1/P_raiser|), the semidiurnal tide.
  - `sse.ocean.wind_waves` (ocean.json): the significant height and peak period of a wind sea. These use the Coastal Engineering Manual's fetch-limited growth (eq. II-2-36), capped at full development (eq. II-2-37), with the drag coefficient C_D = 0.001 (1.1 + 0.035 U10). Its coefficients are fitted models; gravity is the world's own.
  - New port kinds: `tides.raiser_mass`, `tides.raiser_distance`, `tides.raiser_period`, `tides.equilibrium_range`, `tides.period`, `weather.wind_speed_10m`, `ocean.fetch`, `ocean.significant_wave_height` and `ocean.wave_peak_period`. Atom count 90 → 93 (after MP6).
- **Kernel `sea.rs`** runs those blocks (found in `sse_atoms::formulas::loaded()`, so a pack edit to them changes the figures).
  - Raisers: the body the world orbits (once a year round the sky, or a fixed bulge when locked) and its moons (periods from `sse.orbit.period`).
  - Fetch per band: the band's open sea between its coasts (from W7c part 2a's land strips), held to `climate.sea.storm_fetch_m` 1000 km.
- **Survey card**: `survey.climate.sea` is null without a surface sea. Otherwise it holds:
  - `liquid`;
  - `tides`: `by` per raiser (`raiser`, `range_m`, `period_h`, `fixed_bulge`), `spring_range_m`, `neap_range_m`, `block` and `note`;
  - `waves`: `typical_m`, `typical_period_s`, `roughest_m`, `roughest_lat_deg`, `bands` (per climate band `wave_m`, `period_s`, `fetch_km`, or null over land), `block` and `note`.
- **Landing site**: `weather` op `want: report` adds `sea` with `wave_m`, `period_s`, `wind_m_s`, `fetch_km`, a plain `line` and `tides` (when the body is known). These are the waves under the wind blowing there now.
- **Moons listed flat**: `ice_ages::orbit_inputs` now also counts a sibling whose orbit's host is the world's own mass as a moon. V3 imports list the Moon beside Earth, so the imported Earth had no Moon in its precession (W9) or its tides.
- Pack `climate.sea`: `storm_fetch_m` and `source`.

## Results
| | Built | Reference |
|---|---|---|
| Earth, the Moon's tide | 0.533 m every 12.42 h | 0.535 m equilibrium (Murray and Dermott); 12.42 h M2 |
| Earth, the Sun's tide | 0.245 m every 12.00 h | 0.46 of the Moon's |
| Earth, spring and neap | 0.78 m and 0.29 m | equilibrium values |
| 10 m/s over 300 km | 2.75 m, 6.9 s | CEM worked figures |
| Earth card mean waves | 4.8 m, 9.9 s | about 2.5 to 3 m measured (altimetry) |
| Titan, 1 m/s over 100 km | 0.18 m | |

Earth's card waves run high because the climate's mid-latitude near-surface wind runs high (about 17 m/s, against 8 to 12 observed). The tropics run low because there is no swell yet. Both are in `docs/fudges.md`.

## Tests
- `sse-atoms`: `tides_of_the_moon_and_the_sun_on_earth`, `waves_grow_with_wind_and_fetch_until_the_sea_is_full_grown`.
- `sse-kernel` `adopt_age_and_read_back_a_system`: the imported Earth's card has a host tide and the Moon's (0.535 m, 12.42 h) and a typical wave height in a plausible range; its landing-site report has waves and a spring range.

## Goldens changed (on purpose)
- **Changed:** `packs/base-physics/pack.json` and its copy `tests/fixtures/packs/base-physics.json` (three new atoms). In `contract.jsonl`: the stack-loaded hashes and the manifests that carry them (the pack changed), the survey lines (the new `sea` key on the climate summary), the weather report line (its new `sea`) and the reach line (the new atoms read no constant, so they appear as followers).
- **Unchanged:** everything else.
