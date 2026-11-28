# WS3b: a giant's deck renewed by rising air

Note: `handover/design/SSE4-cloud-renewal-rising-air-note.md`. Alex chose "add renewal by rising air" on 27 Nov 2026 (23:33 UTC). The note's five questions are built as it recommends. Needs WS3. Status row: `handover/status/true-air.md`.

## Blocks (`packs/base-physics/formulas/weather.json`)

- **`sse.weather.overturn_time`** (new):
  - q = x M_g / M
  - Δm = p_b (1 − e^(−h)) / g
  - r = F_c / (L q Δm)
  - τ = τ_fall / (1 + τ_fall r)
  - Inputs:
    - the heat the plumes carry, F_c;
    - the deepest deck condensate's latent heat L, its share of the air x, and its molar mass M_g;
    - the air's molar mass M;
    - the top deck's base pressure p_b and its thickness h in scale heights;
    - gravity g and the fall-out time τ_fall.
  - It adds the two losses as rates, and with no heat from below it gives τ_fall exactly.
- **`sse.weather.plume_count`** 1.1.0: a second output, F_c = s F_int, so the plume share is held in one place.
- New port kinds in `sse-atoms`: `weather.convected_flux`, `weather.deep_mole_fraction`, `weather.deck_scale_heights`, `weather.overturn_rate`, `material.latent_heat`.

## Pack row

- `weather_giants.stirring.fallback_deep_mole_fraction`: 8.4e-4. This is solar water's share by molecules, from Asplund et al. 2009 O/H in hydrogen and helium (unverified). It is used where the world's air holds none of the deepest deck's gas. That includes every fixture giant, whose air lists no water.

## Code (`crates/sse-weather/src/eddies.rs`)

- `renewal_s` now combines fall-out (`fall_out_s`, the WS3 code unchanged) with the overturn through the new block.
- `convected_flux` reads the plume block's second output.

## Choices (the note's questions)

1. There is one τ per giant. A per-band τ is in the fudge row's "to improve" column.
2. Two generations are kept for WS5, and the detail will be measured on Sol's Jupiter.
3. The deep abundance comes from the air where it is held. Otherwise the registered fallback is used.
4. Only internal heat counts; sunlight is left until the energy view gives absorption by depth.
5. The plume count is unchanged.

## Numbers (fixtures, unverified)

| World | Fall-out alone | Renewed in | Condensate carrying the heat |
|---|---|---|---|
| Jupiter | 150 days | 32 days | water (fallback share) |
| Uranus | 22.3 days | 22.3 days (a hair under) | hydrogen sulphide (fallback share) |

Jupiter comes out a little under the note's hand range of 40 to 70 days. The fixture has no water, so the fallback's solar share is used, a third to a half of the note's 2 to 3 times solar. Each kilogram of rising air then carries less heat, so more air must cross the deck to carry the same heat.

## Checks (`eddies_tests.rs`)

- `ws3b_rising_air_renews_jupiters_deck_in_one_to_three_months`: τ is between 30 and 100 days (absolute) and under 0.8 of the fall-out time.
- `ws3b_no_heat_from_below_renews_by_fall_out_alone`: exactly equal.
- `ws3b_uranus_barely_overturns`: within 5 per cent of fall-out, and still under it.
- With the overturn term removed, the Jupiter and Uranus checks go red (seen).

The heat is carried by the deepest deck whose condensate the climate knows. An ice giant's ammonium hydrosulphide is not one, so Uranus uses its hydrogen sulphide deck.

## Goldens

- `tests/golden/weather-giants.json`: the flow digests move, because the flow table carries `renewal_days`. This is an area-only re-bless.
- Reach lines for the new block and for `plume_count` 1.1.0 are under `tests/golden/reach/`.
- `contract.jsonl` is unchanged.
