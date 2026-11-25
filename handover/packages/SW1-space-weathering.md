# SW1: space weathering of airless ground

Asked by the coordinator on 25 Nov 2026, from Alex's note that the Moon looks very brown in True ("in SSE3 we had bleaching of airless worlds"). On an airless world, real space weathering darkens and reddens soil; it does not bleach it. What stands out bright are young craters and their ejecta. The kernel gave ground an albedo but no weathering, and True drew regolith in a typed beige.

## What changed

- **Two cited blocks** in `packs/base-physics/formulas/surface.json` (new family `sse.surface`). Generator: `scripts/build-weathering-blocks.py`. Their Rust mirror is `sse-atoms/src/weathering.rs`, and test `weathering_blocks_match_their_sums` holds the two together.
  - `sse.surface.space_weathering`: maturity M = 1 − e^(−t (f_w w o + 1 − f_w)/τ). Here t is the exposure age, w the star's wind flux against 1 AU, o whether the wind reaches the ground, and f_w the wind's share (0.5). τ is about 300 Myr at 1 AU (Grier et al. 2001; Pieters and Noble 2016).
  - `sse.surface.weathered_reflectance`: each band's factor is the mature-over-fresh ratio raised to M. The ratios are 0.62, 0.55 and 0.48 at 650, 550 and 450 nm (Hapke 2001; Lucey and Riner 2011). The factor is exactly 1 on fresh ground.
- **`sse-surface/src/weathering.rs`**: a world's `Weathering`, with these fields:
  - `active`: false when its air is 1e-4 bar or more;
  - `wind_reaches_ground`: false under a running dynamo;
  - `wind_flux_ratio`: from the distance to its star (a moon uses its planet's distance);
  - `tau_myr`;
  - `mature_factor_rgb`;
  - `why`.
- **Tiles** (atlas.rs) gain `maturity` (u8, 0 to 255) on every sample at every level, but only while weathering is active.
  - Bare ground (cover "none", not ice) matures with its exposure age. That age is the younger of `surface_age_myr` and the age of the youngest crater whose bowl or continuous ejecta cover it.
  - Every other sample gets 0.
  - The tile `albedo` is multiplied by the 550 nm factor, and `colour` by all three factors.
- **The world record**: `survey_surface` returns `survey.weathering`, and each entry in `resolved_system` `surfaces[]` carries `weathering`, only where something from space reaches the ground (asked by the detail ladder thread: an absent record, never zeros, on worlds with real air).
- **`SurveyOptions.star_distance_au`** is new. The kernel fills it from the world's orbit, or for a moon from its planet's orbit (`adopt::star_distance_au`). The Orbit atlas cache key carries it.
- `packages/ui-contract/src/globe.ts`: `SurfaceTile.maturity?`.

## Seam (agreed with the detail ladder thread, 25 Nov)

The renderer draws bare ground as R_fresh(band) × mature_factor_rgb^(maturity/255). R_fresh comes from the data sheet's reflectance rows (Look up). When only a mature row exists, R_fresh = R_mature / mature_factor. The renderer stops reading `albedo` for bare ground once it uses the band reflectance.

## Results

| World | Weathering |
|---|---|
| Moon | Active, the wind reaches the ground, τ about 300 Myr. Old ground is fully mature (mean maturity 254 of 255), and young craters are fresher (the least mature sample is 2). |
| Mercury | Active, τ about 78 Myr (6.7 times the wind flux). Its field is off in our model (a known caveat), so the wind reaches the ground. |
| Earth | Air stops it: there is no weathering record, and its tiles carry no maturity. |

The sample Sol's tile albedo is still a flat placeholder (0.30 on the Moon). The fresh reflectance should come from the data sheet's rows.

## Checks

- `weathering_blocks_match_their_sums` (sse-atoms) and `weathering::tests::only_airless_ground_weathers_and_fresh_ground_is_unchanged` (sse-surface).
- `sol_tests::airless_ground_weathers_with_exposure_age`.
- Atom count 130. The base-physics pack and its fixture were re-blessed for the new blocks.
- Golden: re-blessed on purpose. Survey lines carry `weathering`, and tile `albedo`, `colour` and `maturity` change on airless worlds.
- Fudge rows: five, all marked "Stand-in (SW1)".

## Next

- SW1 part 2: ray systems out to about 10 crater radii.
- Per-material weathering from the data sheet's optical constants.
- The wind's open share from the field read rather than the dynamo switch.
- Icy moons' own weathering, such as Europa's sulphur.
