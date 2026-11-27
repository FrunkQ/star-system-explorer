# WS3: plumes from the heat below, and how fast a deck is renewed

Design: `handover/design/SSE4-stirred-clouds-design.md` sections 3.4 and 3.5. Accepted by Alex on 27 Nov 2026. Needs WS2. Status row: `handover/status/true-air.md`.

## Blocks (`packs/base-physics/formulas/weather.json`)

- **`sse.weather.plume_count`**: N = s F_i 4πa² / (F_p π r_p²), with r_p = L_d / 2.
  - F_i is the heat from below only (not absorbed sunlight), a the radius, L_d the deformation radius at 45°.
  - F_p, the heat carried inside one plume, is about 3,000 W/m² (Gierasch et al. 2000, from memory, unverified). s is a share, 1.
- **`sse.weather.particle_fall_time`**: Stokes' fall speed with the Cunningham slip correction (Allen and Raabe 1985 constants, unverified), and τ = dz / v.
  - The air's viscosity is hydrogen's, μ = 8.9e-6 Pa s × (T / 300 K)^0.68, held as two block parameters because the data sheet has no gas viscosities yet. Fudge row "A giant's plumes and cloud renewal".
- New port kinds described in `sse-atoms`. Citations `gierasch2000`, `allen1985`, `stokes1851` added, unchecked.

## Pack row

- `weather_giants.stirring.fallback_particle_um`: 5. Used when a deck has no grain size in `clouds.particle_radius_um`.

## Code (`crates/sse-weather/src/eddies.rs`, `giants.rs`)

- **Plumes.** `plumes_between` draws plumes per window of one life (one turn of an eddy their own size), stream `weather.giant_plumes.v1`, keyed so any date or span gives the same plumes. Each is placed on the cyclonic side of a band (up to 8 tries), drifts with the zonal wind, and fades in and out as sin².
- **On the face.** `GiantPrepared.plumes` holds the day's plumes; `giant_cell` paints fresh cover on the top deck under each (cumulonimbus where the plume is over 0.05) and adds to the cell's turbulence. This moves the field digests in `tests/golden/weather-giants.json` (re-blessed); the core golden does not move.
- **Renewal.** `renewal_s` reads the top deck's grain size and the ice's density off the data sheet in force (the shipped sheet outside a call), the deck's middle pressure and temperature, and runs the block over one deck thickness.
- **Flow table.** `want: "flow"` now also carries `plumes` and `renewal_days`. `flow_table` takes the climate settings.

## Numbers (fixtures, unverified)

| World | Plumes alive | L_d at 45° | Top deck | Renewed in |
|---|---|---|---|---|
| Jupiter | 67 | 1,451 km | ammonia ice | 150 days |
| Saturn | 3.3 | 3,294 km | ammonia ice | 790 days |
| Uranus | 0 (no heat from below) | 2,790 km | methane ice | 22 days |
| Neptune | 0.4 | 1,987 km | methane ice | 13 days |

- Jupiter's 67 is about what Juno and Voyager see as active plumes at once (order of tens; unverified).
- Saturn's renewal is slow because its deck is thick and its gravity low; the ice giants' methane decks are thin and their grains large.

## Checks

- `ws3_plumes_alive_average_the_count_their_heat_gives`: over 200 dates Jupiter's plumes alive average the block's count within 20%, and any date gives the same plumes as a span.
- `ws3_no_heat_from_below_no_plumes`: went red when sunlight was counted as heat from below.
- `ws3_a_deck_is_renewed_in_months`: Jupiter's deck between 10 and 1,000 days (absolute).
- Contract: two `cannot_follow` reach lines added; `contract.jsonl` unchanged.
