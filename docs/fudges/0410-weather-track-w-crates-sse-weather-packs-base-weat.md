
### The one climate (W7a, `crates/sse-climate`, `packs/base-climate`)

Moved here from the rows above: the 12 m/s Rhines wind, Held and Hou's contrast and tropopause, and `weather.supply` now
live under `climate.circulation` and `climate.supply`, with the same values; `weather_formations.t_contrast_k` and the
surface's own `cover.contrast_k`, `wear.rain_m_yr` and `wear.lapse_k_m` are gone (cover, wear and weather read the
energy balance). Details in `handover/packages/W7a.md`.

