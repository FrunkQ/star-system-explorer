# W-wind10: the 10 m wind as one distribution, and waves over it

On 24 Nov 2026 the coordinator, relaying Alex's "can we hurry things along?", asked for the high waves on Earth's card (4.8 m, against 2.5 to 3 m measured) to be fixed at their cause. Owner: "Build shared climate and weather".

## The cause
The wave card read `Climate::wind_m_s`. That is the thermal wind's near-ground share, 0.3 × u_T: about 12 m/s, flat from 15° to 55° and never easterly. It sets the scale for storm steering and land-sea heat transport, but it is not the wind at 10 m. The climate had no 10 m wind with storms in it. Saltation's 10 m wind (W9) is half that same field.

## What was built
- **`sse-climate::wind`**: the 10 m wind per band, as an isotropic Gaussian vector wind (Monahan 2006).
  - The mean is the circulation's prevailing wind: easterly trades and westerlies in the eddy-driven cells.
  - Storms spread each component by σ = `wind10.gust_share` × the eddy wind × the eddies' share of the heat transport. That share is 1 − w, where w is the moisture block's Hadley share exp(−x²/σ_H²). σ also falls with the cube root of the air's density over Earth's and scales with collisionality.
  - `expect(u, σ, f)` averages any function of speed over the distribution, by a 10 × 10 Gauss-Hermite rule (Abramowitz and Stegun table 25.10). `mean_speed` uses it.
- **`Climate`** gains `wind10_m_s` and `gust_m_s` per band, plus `wind10_at` and `speed10_at`. The doc of `wind_m_s` now says what it is.
- **The sea card**: each band's waves are the year's mean of `sse.ocean.wind_waves` over that band's 10 m winds, not the waves of one mean wind. Each band also reports its mean `wind_m_s`.
  - Below the block's 0.5 m/s floor the sea is glassy.
  - Above its 30 m/s top, the growth is held at its top value.
- Pack `climate.wind10`: `gust_share` 0.5, fitted to a storm-track spread of about 5.7 m/s on Earth.

## Results (V3 Earth, 36 bands)
| | Built | Measured |
|---|---|---|
| Mean 10 m speed at 50° | 8.3 m/s | about 9 (north) to 11 (south) |
| Mean 10 m speed in the trades (17°) | 5.9 to 6.5 m/s | about 7 |
| Mean 10 m speed at the equator | 1 to 3 m/s | about 4 to 5 (doldrums; tropical gusts not modelled) |
| Waves at 45° to 55° | 2.4 to 2.7 m | about 3 (north) to 4.5 (south), swell included |
| Waves over all the sea | 1.6 m, 5.3 s | 2.5 to 3 m with swell; wind sea alone is a part of that |

## Not changed
Blown sand still reads W9's 10 m wind (`saltation.ten_metre_share`, `gust_share`). On the new wind, Mars's dunes stop. Thin air's strong gusts come from the convective boundary layer, and that is not in the spread yet. Both 10 m winds are logged in `docs/fudges.md` until W10 makes gusts a strength.

## Tests
- The existing sea test (`adopt_age_and_read_back_a_system`) holds: the typical wave height is between 0.3 and 8 m.

## Goldens changed (on purpose)
- **Changed:** `surface-cover.json` (two `layers` hashes: the serialised climate gains `wind10_m_s` and `gust_m_s`, nothing else in the cover moves). In `contract.jsonl`: the survey line (the sea card's waves and the climate's new fields) and two weather climate lines (the new fields).
- **Unchanged:** every other golden, including wear and weather: no number outside the sea card changed.
