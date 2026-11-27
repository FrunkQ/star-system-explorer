# WS2: a giant's eddies from the heat it sends out

Design: `handover/design/SSE4-stirred-clouds-design.md` sections 3.2, 3.3 and 6. Accepted by Alex on 27 Nov 2026. Needs WS1. Status row: `handover/status/true-air.md`.

## Blocks (`packs/base-physics/formulas/weather.json`, family `sse.weather`)

- **`sse.weather.eddy_power`**: ε = η F g / p₀.
  - F is the heat the air sends out (internal plus absorbed sunlight), g is gravity, and p₀ is the weather layer's pressure.
  - η is the block's setting, editable through a pack (ER2).
- **`sse.weather.cascade_speed`**: u = C (ε L)^(1/3), capped at the isothermal speed of sound √(R T / M). C is a setting (3).
- **`sse.weather.torn_by_shear`**: Moore and Saffman's steady ellipse, ζ/ω = λ(λ − 1)/(λ + 1).
  - Below ζ/ω = 2√2 − 3 the eddy is torn, in one shear time 1/|ζ|.
  - `giants::sheared_aspect` (PR 646's ovals) now runs through this block, so the law lives in one place.
  - That also fixes a latent bug: for ζ/ω ≤ −3 − 2√2 the old code returned a negative aspect.
- The new port kinds are described in `sse-atoms` `KIND_DESCRIPTIONS`. The new references are on the citation list (`kraichnan1967`, `salyk2006`, unchecked).

## Pack rows (`packs/base-weather` `weather_giants.stirring`)

- `weather_layer_bar`: 1.
- `eddy_area_share_per_octave`: 0.25.
- `eddy_life_turns`: 2.
- All are stand-ins, `unverified`.

## The eddies (`crates/sse-weather/src/eddies.rs`)

- **Bands.** Every band of each hemisphere (equal area, edges evenly spaced in sin latitude).
- **Sizes.** Size classes run one doubling at a time from the deformation radius at the band's middle up to the band's width. Where the deformation radius is wider than the band, there is a single class at the band's width.
- **Windows and counts.** Each class is drawn per window of one life (two turns of its middle eddy at its edge). The count has the class's mean: share × band area / eddy area, rounded at random.
- **Each eddy.**
  - Width: log-uniform in its class.
  - Place: uniform by area in its band.
  - Sense: either, at even odds.
  - Speed: from the block.
  - Shape: aspect and whether it is torn come from the block.
  - Life: one window, or one shear time if torn.
  - Strength: rises and falls as sin² over its life.
  - Drift: carried by the zonal wind at its centre.
- **Keying.** Draws are keyed by band, class, window and slot in the world's stream `weather.giant_eddies.v1`, so any date or span gives the same eddies.
- **Not in the systems list.** They are too many and too brief to name. Jupiter has about 2,800 alive at once; Saturn, Uranus and Neptune far fewer.
- **`flow_table` (weather op `want: "flow"`, request field `span_days`, 0 to 60).**
  - The jets by latitude at 0.25°, and the polar jet with its polygon.
  - The storms on the first day, and every eddy alive in the span.
  - The eddy power, and a "why" line.
  - Like `glint`, `flow` is left out of the refusal's list of wants, so the core golden does not move.

## The fit, and where it differs from the design

- **η is 0.0068, not the design's 0.04.**
  - Worked from the Jupiter fixture: F = 13.66 W/m², g = 24.79 m/s², band at 45° 7,896 km wide, giving ε = 2.30e-5 W/kg and u = 17.0 m/s.
  - With η = 0.04, Jupiter's eddies would run at 31 m/s. The design's hand figure looks like an arithmetic slip, but the fit itself is the rule.
- **Saturn comes out at 8.4 m/s.** The design said about 8.
- **Uranus and Neptune come out at 6.2 and 6.7 m/s, not the design's "about 3".**
  - Their three bands (kept by the older 100 m/s eddy fudge) are about 21,000 km wide, and the law gives this.
  - It is still sixty times slower than their 400 m/s jets, so the "calm" reading holds.
  - WS8, one eddy speed for the jet count too, is the cure for the two numbers.

## Tests

`eddies_tests.rs`:
- Jupiter at 17 m/s (absolute).
- Saturn within a tenth of 8.
- Every world on the cube-root law from Jupiter's fit, within 2%.
- An eddy against the shear is gone within one shear time. This test went red with the tearing removed.
- Torn eddies sit at the limit's aspect and held ones above it.
- The same eddies whichever date or span asks.
- No heat, no eddies.

`sse-kernel` `sol_tests::a_giants_flow_table_carries_its_eddies`:
- The op on catalogued Jupiter gives the table.
- Earth is refused with `not_a_giant`.

## Goldens

- `tests/golden/weather-giants.json` (crate): new lines only per world, the flow table's digest, its eddy count over five days and the eddy power. The other digests do not move, since the ovals' aspect comes through the block bit for bit.
- `tests/golden/contract-reach.txt` gains three lines for the new blocks. Per the coordinator, this is not a core lane turn.
- `tests/golden/contract.jsonl` does not move.

## Fudges

`docs/fudges.md`: "A giant's eddies, fitted and drawn (WS2)".
