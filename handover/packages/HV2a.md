# HV2a: the survey publishes the climate's sunlight budget

Thread "Show a planet's energy flow over time", 26 Nov 2026. Status: `handover/status/heat-screen.md`.

Alex (26 Nov, 23:18 UTC): sunlight cannot be left out of a world's energy flow; show how much arrives, is reflected and is absorbed. The coordinator placed this kernel read with this thread (routine, no new physics).

## The change

- `sse_climate::Climate::sunlight()` returns a `Sunlight` (W/m², averaged over the sphere and the year): `arriving_w_m2` (S/4 times the year's mean (a/r)², 1/√(1 − e²)), `absorbed_w_m2` (the annual season's band values, area-weighted), `reflected_w_m2` (arriving less absorbed), `bond_albedo` (reflected over arriving, the albedo the bands give after ice, cloud and land), `internal_w_m2` (as the climate takes it), and `emitted_w_m2` (absorbed plus internal, the steady state).
- `survey_surface` puts it in the summary as `survey.climate.sunlight`.

## Check

`crates/sse-climate/src/sunlight_tests.rs`: Earth arrives at 340 W/m² (measured about 340) with a Bond albedo of 0.30 (measured about 0.29), held as limits 330 to 350 and 0.2 to 0.4; the books close to 1e-9 (absolute).

## Golden

Core lane: two `surface_surveyed` lines of `tests/golden/contract.jsonl` gain the `sunlight` object and nothing else (checked line by line against main).
