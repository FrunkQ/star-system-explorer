---
name: weather-track
description: State of SSE4 weather track W (crates sse-climate and sse-weather, kernel op `weather`) and its open follow-ups
metadata:
  type: project
---

W1 to W6 on main since 20 Nov (PR 25). Redo, owner thread "Build shared climate and weather":
- W7a merged 21 Nov (PR 84): crate `sse-climate` + `packs/base-climate` is the ONE climate under wear, cover and both weather blocks; kernel `weather` takes system+body.
- W7b merged 21 Nov (PR 96): circulation as process strengths (`sse-climate::strengths`), Rhines wind from the thermal wind, regime label read off strengths, tested-range "extrapolation" marks; Body.spin_hours/obliquity_deg/spin_source from formation and V3 ("authored (V3)"); rain belt follows the season's energy peak (tilted worlds); Kepler seasons with Conditions.perihelion_lon_deg (year phase still from the spring equinox).
- W9 part 1 (PR 105, 21 Nov): `sse-climate::water` (rain, snow, melt, PET, Fu runoff, ice balance at any lat+height; Köppen area labels as pack rules `climate.areas`) and `sse-climate::saltation` (sse.wind.saltation, no pressure cut); wear rivers carry runoff, glaciers = ice balance > 0, dunes = Lancaster mobility; Wear gains runoff/snow/ice_balance/dune_activity/areas; survey climate summary gains `places`. Chart room U17 (PR 104) reads summary lat_deg/t_k/precip_m_yr arrays: keep that shape; it spots extrapolation lines by text.
- W9 part 2 (PR 109, 21 Nov): `sse-climate::forcing` (Zeebe 2022 spin axis over T1 secular track, Huybers summer energy, linear ice response); kernel `ice_ages.rs` (sibling planets, drawn inclinations); Wear ice_share and glacial_advances_per_myr; survey `ice_ages`.
- W8 part 1 (PR 115, 22 Nov): MP3 species table: Fray-Schmitt ice curves added via scripts/build-base-reference.py (sources fray2009, lisse2021; kernel thread agreed to keep them); climate pack `substances` retired; ebm::p_sat/frost_k read one curve.
- W8 part 2 (exchange, `sse-climate::exchange`, Climate.exchange, Conditions.ground_ice_kg_m2, survey `volatile`): Mars swing 29% two minima; Pluto buffered low-lat ice vs 30° tilt polar caps. Part 3 (with S17a): ground inventories on surveyed bodies, polar hood, CO2 snow, minor ices; giants' decks onto curves with W11.
- W7c part 1 (22 Nov, land and sea columns per band, `ebm::cycle_two`, Conditions.land_by_lat/coasts_by_lat, coupling = diffusion + zonal advection; Earth land 32 K vs sea 10 K range at 52N; `IceAgeModel::onsets_yr` for surface_frames). Part 2: surface fills land/coasts from height map, coast distance per cell, currents, ITCZ lag, eyeball ice.
- W-sea (PR 346, merged 24 Nov): tides `sse.tides.ocean_range`/`period`, waves `sse.ocean.wind_waves` (CEM); kernel sea.rs; survey climate.sea, weather report sea. Caveat: climate mid-lat surface wind ~2x high so card waves 4.8 m vs 2.5-3 observed; no swell. V3 imports list moons flat; ice_ages now finds them by host mass.
- PR 418 MERGED 25 Nov 06:17 UTC; W-helium PR 538 MERGED 25 Nov 09:08 UTC: W-wind10, W7c 2b/2c, K-a/b/c, W10a, W-heights, W-sea2 to 4, W-giant-air (worlds read their catalogued air; Sol giants' 1 bar temps; Saturn ammonia deck + chromophore). State in handover/status/weather.md.
- surface_frames (renderer video): proven core assembles the op; climate side = world_climate per epoch with that epoch's envelope and luminosity.
- Queued by Alex 24 Nov after W7c part 2, in order: W-sea2 sea colour from the liquid's absorption (sheet + light code), W-sea3 waves by liquid properties (density ratio, viscosity, surface tension, capillary cut-off), W-sea4 whitecaps/froth; kernel hands a few numbers per region, renderer draws cheaply in shaders. Helium is one more liquid in W-sea3, with a "superfluid below 2.17 K" card line; no creeping film. Acceptance: rogue planet in a void with a helium sea works end to end.
- Next: W7c part 2, W8 part 3, W10, W11, W12. Also queued MF8 air loss by channel (magnetic fields design, Alex 22 Nov 01:06 "agree to all": shielding is an effect, never a switch).

Coordinator decided (not Alex) yes to per-area labels in W9 and a plain climate Why? readout (done, U17); matter pack design thread started from his periodic-table question.

Open: weather formations still gate on the regime label (W10); Lindzen-Farrell 0.31 unverified; formation spins primordial; surface Body has no perihelion longitude; Titan's flat MSE makes the rain belt hop.

Related: [[sse4-weather-review]], [[traveller-pack-track]]
