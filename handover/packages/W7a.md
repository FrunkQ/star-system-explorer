# W7a: one shared climate (21 Nov 2026)

Weather review (`handover/design/SSE4-weather-review.md`) section 8, package W7a, under rulings 2 and 5 of its section 9.
The finding it fixes: erosion (S3 wear), cover (S7) and the two weather blocks each had their own climate, so the rain
that cut a valley was not the rain on the weather map. Now there is one.

## What was built
- **New crate `crates/sse-climate`**, below both `sse-surface` and `sse-weather`, with pack key `climate` in the new
  `packs/base-climate` (falls back to the base pack when a stack has none).
  - `conditions.rs`: `Conditions` (moved from weather) and `Extra` (optional facts a caller knows: spin, tilt, flux, air,
    liquid and so on). Spin is **signed**: `rotation_hours < 0` is retrograde; `omega()` is signed, `spin_rate()` is |Ω|,
    `solar_day_hours()` handles both (Venus gives 116.75 days).
  - `ebm.rs`: the core. North 1975 diffusive energy balance, Budyko linear OLR, A set so the global mean is the world's
    own temperature, base albedo set so the Bond albedo is kept. New this package:
    - a **seasonal cycle with heat capacity** (North and Coakley 1979): C = sea × ρc × 50 m + land × I sqrt(year/π) + the
      air column c_p p / g; implicit steps, 12 seasons × 4 substeps, 6 spin-up years from the annual steady state. Earth's
      poles now swing a sensible few kelvin over the sea and tens over land and thin air (Mars poles 102 to 248 K),
      where the steady-season version gave 188 to 355 K.
    - **moisture transport after Siler, Roe and Armour 2018**: eddies diffuse moisture down-gradient; the Hadley cell
      carries the share w = exp(−x²/σ²) of the energy flux, its lower branch moisture toward the equator,
      F_q = −F_HC L q / (h_T − h). Earth: rain belt about 3.8 m a year at the equator, dry subtropics, storm tracks near
      1 m a year, global mean about 0.85 m a year (observed about 1).
    - a thermal wind from the temperature gradient, with the signed Coriolis parameter.
  - `circulation.rs`: Held and Hou edge, Rhines cell count, cell layout, storm tracks and the regime label (first-match
    rules moved from the weather pack), plus `prevailing_wind` and `easterly`, which flip on a backward spin. The giant
    jet count, band edges and polar jet (Rhines) moved here too, so the S6 cloud deck and the giant weather share them.
  - `climate.rs`: detail levels **Glance (D0)** (circulation and a mean only) and **Orbit (D1)** (the full seasonal
    climate); each result is keyed `address|climate-v1|level|epoch|hash of conditions`, and `Cache` keeps recent results
    by key (reuse before compute). Survey and Site will refine these in W11.
- **Surface** (`sse-surface`): new `climate.rs` builds a body's `Conditions` once (Kepler year, locked spin, defaults
  listed in `assumed`), and `WorldClimate` adds the lapse rate. `wet_climate` gives a world wet only in the past its
  water back. **Wear** reads rain and glacier temperature from it; **cover** reads temperature, wetness and winds from
  it (its own contrast and cell wind are gone). `survey_world` and the Orbit chain (`orbit_world`, S16a part 2) compute
  the climate once and keep it on the `Survey`.
- **Weather** (`sse-weather`): `circulation::core` is the one climate; `averages` reads its bands; the rule block's cells
  read rain, cloud and temperature from the season; the zonal block's season is the climate's season. The two blocks
  now read the same climate (ruling 2: the energy balance is the core, the rule block is the drawing on top, the swap
  stays). The swap report test now proves a different climate is caught (latent share × 0.3).
- **Kernel**: `weather` takes `system` + `body` (read at `at_s` like `survey_surface`); the conditions come from the same
  surface function, at the same epoch, so the weather op and the survey give the same climate key, temperatures and
  rain. `want: climate` returns the climate itself as `core` and keeps it in an 8-entry cache. `survey_surface` returns
  a summary of the climate (key, regime, cells, Hadley edge, bands of temperature and rain, lapse, assumed, why).
- Fixture: Venus spins at −5832.5 h.

## Tests
- `sse-weather::tests::the_ground_and_the_sky_read_one_climate`: Earth, Mars, Venus and Titan; the survey's climate and
  the weather layers' climate are equal, the conditions are equal, both blocks' band rain equals the ground's rain and the
  rule block's season temperatures equal the ground's, to the last bit.
- `sse-climate`: Earth warm and wet in the tropics with trades and westerlies; a backward spin flips the winds only;
  the cache reuses; Glance matches Orbit's circulation; the airless Moon; references; and
  `no_second_climate_outside_the_crate`, which scans every other crate's non-test code for the old contrast, cell wind,
  rain-lapse, Hadley and Rhines code.
- `sse-weather::a_backwards_spin_turns_the_weather_winds`; `sse-kernel` survey test now also asks `weather` for the same
  body and checks key, temperature and rain against the survey, and that a re-read is identical.

## Goldens changed (once, on purpose)
`surface-cover.json`, `surface-wear.json`, `weather.json`, `weather-storms.json`, `weather-swap.json` and the survey and
weather lines of `contract.jsonl`. Reason: erosion, cover and weather now read the one seasonal energy-balance climate
(the accepted W7a), instead of four separate models. PR 71 (S16a part 2, the Orbit-level chain) landed first; this
package merged main and re-blessed once for both. `surface-regimes.json` is unchanged (the moved giant jet code is
bit-identical). PR 82 (S0b icy bodies) then landed too; main was merged again and only `contract.jsonl`'s survey and weather lines needed one more bless, for the same reason.

## Known limits (all in `docs/fudges.md`)
- Locked worlds are still one zonal climate, not day and night (W7b).
- One ground thermal inertia; one mixed layer; Siler's σ scaled by our own Hadley edge; h_T 1.06 is Earth's.
- The Rhines wind is still one 12 m/s (W7b takes it from the forcing).
- Spin and tilt default to 10 h and 20° when a body does not publish them; formation runs and V3 imports do not hand
  them to the surface `Body` yet (next, with the "authored (V3)" mark of ruling 5).
- A moon's year is its own orbit; flux defaults to a sun-like star at the orbit.
- Titan's surface fixture has no surface liquid, so no methane rain yet (W8 moving ices).
- Mars's poles get colder than the CO₂ frost point (about 148 K) because the climate does not yet condense CO₂ (W8).
- The aerosols' dry season still reads the rule block's wetness drawing (W10).
- The zonal weather block's other calls (systems, field) recompute the climate rather than read the kernel cache; it is
  a few milliseconds a call (W11 detail levels and cost).

## Next
W7b: circulation as strength blocks (Hadley, Rhines and super-rotation strengths, the wind scale from the forcing,
day-night on locked worlds), with spin and tilt carried from formation and V3 imports.
