# SB4 part 1: when a comet's ices wake

Track SB, thread "Build asteroids and rings up close", 24 Nov 2026.

## What was built
- `crates/sse-small/src/comet.rs`: `activity(member, shares, r_au, host, settings)` solves, for each ice the nucleus holds (water, CO2, CO, their shares from formation's solid groups via `ice_shares`), absorbed sunlight = emissivity sigma T^4 + flux x latent heat, with the flux from Hertz-Knudsen on the sheet's vapour curve (`sse_reference::sublimation_pa`) and the latent heat from that curve's slope (`sublimation_latent_j_mol`). `onset_au` is where sublimation takes half the subsolar sunlight. CO2 and CO leave with the water once it is active, else from under the mantle (fudge SB-comet-mantle). It gives production per ice, gas and dust mass rates, coma scale length, dust and ion tail lengths (fudge SB-comet-coma).
- Pack `comet` block: active fraction, insolation shares, emissivity, phase integral, dust to gas, outflow, lifetimes, tail numbers. Constants (sigma, R, N_A, S_sun, GM_sun, au, day) and molar masses are read from the sheet.
- Kernel: `small_body` takes `r_au` (perihelion when left out, a planet-bound body at its planet's distance); answers for icy bodies, and `make_small_body` answers, carry `activity`.

## Evidence
`cargo test -p sse-small comet`: 4 pass. Onsets in order with nothing typed: water 3.3 AU, CO2 13 AU, CO about 150 AU (0.06 albedo, Sun-like star); water flux at 1 AU 1e20 to 1e22 molecules per m^2 s; a 4.7 km 67P-like nucleus makes about 5e27 water molecules a second at 1.24 AU (Rosetta: of order 1e28); coma tens of thousands of km and a dust tail over a million km at 1 AU, nothing at 500 AU; a rocky body has no activity. Kernel: a made comet nucleus at perihelion 1.05 AU is active, water leading.

Golden: two of this PR's own appended lines (the member and the made contact binary now carry `activity`); no older line changed.

## Next (part 2)
Draw coma and tails in True: brightness from production, the dust tail curved behind the orbit and the ion tail straight away from the star; reuse `plumeBrightness`.
