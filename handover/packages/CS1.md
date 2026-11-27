# CS1: climate sunlight from the world's own star (27 Nov 2026)

Thread "Fix climate sunlight for other stars and moons", opened by the coordinator from three faults the energy flow thread found while building HV2b (the sunlight half of the energy view). Builds on HV2c (`Body::star_au`, a moon's distance from the star is its planet's), which this branch carries in the same shape so the two merge cleanly.

## The three faults and what fixes each

1. **Sun-like star assumed.** `climate::conditions` took 1361 W/m² / d² for every world. Now `sse_surface::starlight::starlight_w_m2` gives each star's own light over the world's distance squared through the formula block `sse.climate.insolation`:
   - a generated world (`Climate::Star`): the star's life line at the world's age, plus every companion (`OtherLight`), the same light its envelope temperature already used;
   - a catalogued or imported world (`Climate::Fixed` with an `Anchor`): the star's measured luminosity, new `Anchor::luminosity_lsun`, worked out in `adopt.rs` from the star's own catalogued radius and temperature by the new formula block `sse.star.luminosity` (L = 4πR²σT⁴), carried through time by the life line's ratio L(t) / L(measured). Without both facts, the life line's model star;
   - nothing known about the star (no life line), or no physics in force (outside a kernel call): 1361 W/m² / d², and `assumed` says "sunlight from a sun-like star at the orbit (the star's light is not known)".
   The spectrum: the energy balance is broadband, so a red star's light is not yet reflected differently (fudge register row, with the replacement named).
2. **A moon's year was its month.** `starlight::star_place` gives the world's orbit round its star: a planet's own, a moon's its planet's (`Body::star_au` from HV2c, new `Body::star_e` set by `adopt.rs` from the planet's eccentricity, the star's mass from the life line). The year is Kepler's period round the star with everything that goes round with it (`sse.orbit.period`, star + planet + the world). The climate's eccentricity is the planet's. A locked moon's day is the period round what it orbits (`starlight::orbit_days`), so it stays its month. Eclipses by the planet are not modelled (fudge register: share asin(R_p/a)/π, about 5 % for Io, needs the moon's inclination and the planet's radius on the Body).
3. **Default albedo on adopted worlds.** `adopt.rs` read a catalogued `body.albedo` for its equilibrium temperature and then dropped it, so the climate took 0.30. New `Body::bond_albedo` carries the measured or typed Bond albedo; the climate reads it before anything else (Moon 0.11, Earth 0.294, NSSDCA as in base-sol). A world with nothing measured takes the surface pack's default, and `assumed` now says so ("Bond albedo 0.3 (the surface pack's default; none measured)"). No Moon or Sun branch anywhere: every world goes through the same three reads.

## Reads through the physics in force (ER)

- New reads go through the door: blocks from `sse_physics::now().blocks()` (the two blocks read have no settings, so no shipped defaults are taken), the Sun's luminosity and the masses by `sse_physics::constant("L_sun" | "M_sun" | "M_earth")`. AU and the day are units.
- Files edited that are on the ledger: `crates/sse-surface/src/climate.rs` (G now `sse_core::constants::g()`, line shrunk 5 to 4) and `crates/sse-kernel/src/adopt.rs` (unchanged count). Not converted, and why: `climate.rs`'s R_earth and M_earth turn a Body's Earth units into SI, and `conditions` also runs outside any kernel call (sse-weather and sse-surface tests, the scenario goldens) where nothing is in force, so the door has no value to give; `adopt.rs`'s block read is `sse.interior.rocky_radius`, a Rust block with settings, which needs ER2's `physics::run`; its mass constants are module constants used across the file. Asked of the coordinator.

## What moved in the goldens, and why

- `tests/golden/contract.jsonl`, two lines, both `surface_surveyed` (the lines PR 678 touched), text only: no number in the core golden moves.
  - line 161, the Earth-like fixture survey (no orbit, no star): the climate's `assumed` and `why` now say the year was assumed ("a year of 1 AU round a sun-like star (no orbit)"), why the light is sun-like ("the star's light is not known, or no data sheet is in force") and that the Bond albedo is the pack's default. Earth's values are unchanged.
  - line 164, a generated world (kestrel, sys 40, body 2): this core case loads `base-physics` alone, with no data sheet, so there is no Sun's luminosity or mass to read; the climate falls back exactly as before and now says so ("year by Kepler's law in AU and years (no data sheet in force)" and the sunlight line above). With the sheet loaded (every real stack) the same world would read its own brighter star, about 3,930 W/m² arriving over the sphere where the fallback gives 1,471; that was seen in a first bless before the reads moved to the door.
- `tests/golden/contract-reach.txt`: one new line for the new block `sse.star.luminosity`.
- `tests/golden/surface-cover.json`: two `layers` hashes (Earth and the Moon fixtures), from the new `assumed` text only; the climate keys (numbers) are unchanged.

## Checks

- `crates/sse-surface/src/starlight_tests.rs`, five tests. The four that test the fix were run against the old `climate.rs` and seen red (all four failed), then green: a 0.5 solar-mass star's world gets that star's light (absolute, to 1e-6) and a year √2 longer; a star measured at 0.1 L_sun gives a tenth of the solar constant (absolute) and the Sun as measured gives 1361 ± 0.5; the Moon's year is 365.26 ± 0.2 days, its eccentricity 0.0167, its day 27.3 ± 0.3 days and its light 1361 ± 0.5; a measured 0.11 albedo wins and the default is declared. The fifth checks that with nothing in force the climate falls back and says so.
- `crates/sse-kernel/src/sol_tests.rs` `catalogued_worlds_get_their_own_sunlight_and_albedo`: the catalogued Earth and Moon surveys get 330 to 350 W/m² over the sphere and their own Bond albedos (within 0.03), with no sun-like or default-albedo assumption. Not separately run against main (main gives the Moon 0.30 and about 10^8 W/m², so it would fail).
- `sse-atoms` first-wave atom count 178 to 179 (P1 log).
- Figures unverified where marked: the paper search is out until 1 Dec 2026.

## Found on the way (not in this package)

- The kernel tests' Sol loader (`open_sol` in `sol_tests.rs`) wrote base-sol's degrees as radians, so Earth's tilt was 23.44 rad and its climate ran upside down (equator 279 K, poles 305 K, a sea to 322 K). The app's loaders convert. The fix (`open_sol` through `reference::sol_batch`, and a test that Earth's equator is warmest and its sunlight averages 330 to 350 W/m²) is a patch handed to the life thread, `sse4/patches/open-sol-radians.patch` in project files, because life's Earth test was calibrated on the upside-down climate and goes red with it. With it, Earth's annual zonal means are 288 K mean, 303 K equator, 249 and 253 K poles.
- The model Sun on its life line (Hurley fits, z 0.0142, 1 solar mass) gives 1.134 L_sun at 4,567 Myr, about 13 % above the measured 1.0. Catalogued worlds now read the measured light; generated worlds round Sun-like stars and the young-Sun ratio still use the life line. For the star ageing owner (unverified figures).

## Next

- Pass the survey's epoch into `conditions` so a past survey gets the young star's light (today's light is used now; fudge register).
- Eclipses by the planet once the Body carries the moon's inclination and the planet's radius.
- Bond albedo estimated from the surface and air for unmeasured worlds.
