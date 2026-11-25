# Status: Climate and weather

Owner thread: "Build shared climate and weather". Covers: W.

Only the owner thread edits this file. One row per package: update it in the same pull request as the package. Logs are in `handover/packages/`. Index and standing rules: `handover/START-HERE.md`. Stage order: `handover/design/SSE4-coming-together.md`.

| Package | State | Log |
|---|---|---|
| W1 weather contract (swappable `WeatherBlock`, field on atlas tiles, systems, pins), circulation regime, what condenses, climate averages | done (weather track), crate `sse-weather`, pack `packs/base-weather` | packages/W1.md |
| W2 formations on rocky and ocean worlds: storm tracks, tropical cyclones, dust, locked-world shields, marine decks, mountain cloud, lightning, snow; drawn storms with tracks; weather report | done (weather track) | packages/W2.md |
| W5 weather hooks: kernel op `weather` (climate, systems, field, report, column, sound), descent column with hazards, sound cues, GM pins, clear and calm orders, triggered storms | done (weather track) | packages/W5.md |
| W3 giants: bands as S6, storm families (great ovals, strings, dark spots, planet-circling outbreaks), polar regime by Burger number (cyclone ring, single vortex, diffuse cap), polygon jets by a stated rule (Saturn's hexagon); wired into the rule block and the `weather` op | done (weather track) | packages/W3.md |
| W4 smoke, ash, haze, smog and spray from sources (eruption plumes by Mastin et al. 2009 and stratification, ballistic plumes on airless worlds, ash streaks and ash fall, dated sulphate veil cooling, impacts, fires gated by oxygen, smog over industry) | done (weather track); sources are caller inputs until S7, S12 and L1c publish them | packages/W4.md |
| W6 the swap, proven: second block `sse.weather.zonal-ebm` (diffusive energy balance), save manifest, swap report with tolerance, pins carried, switch back restores exactly; kernel `weather` op takes `block` | done (weather track) | packages/W6.md |
| W7a one shared climate: new crate `sse-climate` (seasonal energy balance with heat capacity, Siler moisture transport, Held and Hou and Rhines circulation, keyed Glance and Orbit results and a cache) under both `sse-surface` and `sse-weather`; wear, cover and both weather blocks read it; signed spin; kernel `weather` takes a `system` and `body` | done (weather redo) | packages/W7a.md |
| W7b circulation as process strengths (`sse-climate::strengths`: thermal Rossby number, super-rotation, baroclinic growth, day-night contrast, collapse, collisional air, regime lengths; wind scale from the thermal wind; label read off strengths); spin and tilt from formation and V3 imports ("authored (V3)") | done (weather redo) | packages/W7b.md |
| W9 part 1 supply to geology (`sse-climate::water`: rain, snow, melt, evaporation, runoff and ice balance at any place and height, per-area Köppen labels; `sse-climate::saltation`, `sse.wind.saltation`, no pressure cut); wear's rivers carry runoff, glaciers follow the ice balance, dunes follow saltation | done (weather redo) | packages/W9.md |
| W9 part 2 insolation history and ice ages (`sse-climate::forcing`: spin axis by Zeebe 2022's precession equation under T1's secular orbit track, Huybers summer energy, linear ice response; kernel `ice_ages.rs` builds the track from sibling planets; wear's ice share and glacial advances per cell) | done (weather redo) | packages/W9.md |
| W8 part 1 one species table (matter pack MP3: `sse.volatile.species` is the climate's view of every sheet substance with a triple point and a vapour curve; Fray and Schmitt 2009 ice curves on `packs/base-reference` via `scripts/build-base-reference.py`; the climate pack's `substances` retired; frost and cloud read one curve) | done (weather redo) | packages/W8.md |
| W8 part 2 moving ices (`sse.volatile.exchange`: frost pins the ground at the main gas's frost point inside the seasonal cycle, pressure solved from the inventory; Mars's CO2 cycle, Pluto's buffered N2, ice migration by tilt; lofting and the survey's `volatile` summary read it) | done (weather redo) | packages/W8.md |
| W7c part 1 land and sea columns (`sse-climate::ebm::cycle_two`: a land and a sea column per band with their own heat capacities, sharing the band's transport, coupled across the coasts by diffusion and zonal advection; `Conditions::land_by_lat` and `coasts_by_lat`; `Climate::t_land_at`, `land_range_at`, `sea_range_at`; dated ice-age onsets `IceAgeModel::onsets_yr`). Part 2a: land and coasts per strip from the height map (`sse_surface::climate::land_strips`, filled by `survey_world` before the climate) | done (weather redo); rest of part 2 next | packages/W7c.md |
| W-sea tides and wind waves (Alex 23 Nov): `sse.tides.ocean_range`, `sse.tides.period`, `sse.ocean.wind_waves` formula blocks; kernel `sea.rs`; `survey.climate.sea` on the card and `sea` in the weather report; moons listed flat by imports counted as moons | done (weather redo) | packages/W-sea.md |
| W-wind10 the 10 m wind as one distribution (`sse-climate::wind`: the prevailing wind as its mean, storms as a Gaussian spread, Monahan 2006); the sea card's waves are the year's mean over it | done (weather redo) | packages/W-wind10.md |
| W-sea2 sea colour from the liquid: absorption, scattering and refractive index from the data sheet, deep colour 0.33 b_b/(a + b_b) | built (deep colour on main once data sheet PR 475 lands) | packages/W-sea2.md |
| W-sea3 waves by liquid: density, viscosity, surface tension from the sheet; drive scaling, Pierson-Moskowitz trains with capillary-gravity dispersion, ripple cut-off, superfluid line. Helium test chain and its owners in the package log | built; helium end to end waits on the data sheet (lambda and critical point), the climate's liquid-only volatile path (this thread, next) and surface cover's liquid choice | packages/W-sea2.md |
| W-sea4 whitecaps and froth: whitecap fraction (Stramska and Petelski 2003) on the driven wind, foam albedo 0.22, streak direction | built | packages/W-sea2.md |
| W7c part 2b (currents as rules: upwelling decks, warm western boundaries, flip with the spin) and 2c (sea evaporates at the sea column's temperature) | done, in PR 418 with W-wind10 (open, golden lane after PR 400) | packages/W7c.md |
| W10a giant chromophore and Titan haze depth: a `chromophore` aerosol over ammonia decks, thicker over belts (Sromovsky et al. 2017), and Titan's haze 3 to 9.6 at 500 nm (Huygens DISR fits) | done, in PR 418 | packages/W10a.md |
| K-a and K-b for the visual polish design (`design/SSE4-visual-polish-design.md` section 12, Alex yes 24 Nov 18:57). K-a: the weather op's `climate` read of a banded giant gains `jets` {lat_deg (band centres), jet_speed_m_s, jet_shear_m_s, eddy_speed_m_s, rhines_scale_m, deformation_radius_m, source}; the jet count now reads the local eddy speed (17 m/s gas, 100 ice) and counts round(½ sqrt(aΩ/U)) prograde jets, after the citation check's correction 3 (Jupiter 9 to 13, Saturn to 12, ice giants stay 3). K-b: every weather field cell on a world with a surface carries `wind10_m_s`, the mean 10 m wind (absent on giants and airless worlds). Package log `packages/K-a.md` | done, in PR 418 (added to it to save a golden-lane turn) | packages/K-a.md |
| K-c particle sizes: `particle_radius_um` on weather aerosols and layers (ammonia ice deck 1.0 µm, chromophore 0.15 to 0.29, Titan haze 0.72, dust 1.5); water, methane and sulphuric acid clouds still open | done, in PR 418 | packages/W10a.md |
| W-heights (asked 24 Nov by the giant bands thread for VP-A4): `base_m` and `top_m` on every weather field layer. Rocky worlds use the descent column's bands; giants put each deck's base where its vapour saturates on the adiabat (Jupiter's ammonia 0.73 bar, Uranus's methane 1.28, no tuning). Next: the low base at the lifting condensation level | built, goes in the PR after 418 | packages/W-heights.md |
| NEXT, in order: helium as a liquid-vapour volatile (climate, for the helium sea test), W7c part 2d (a cell's temperature from its distance to the coast: land cells read the land column, coasts lean to the sea; cover and wear), then W8 part 3 (with S17a), W10 to W12 (`design/SSE4-weather-review.md`), MF8 (air loss by channel; shielding is an effect, never a switch) | next; owner "Build shared climate and weather" | design/SSE4-weather-review.md |

## Sea drawing contract (agreed 24 Nov with "Build surface time-lapse", the drawing side)

As built (W-sea2 to W-sea4): the optics and liquid keys (`bands_nm`, `absorb_per_m`, `backscatter_per_m`, `refractive_index`, `deep_rgb`, `density_kg_m3`, `viscosity_pa_s`, `surface_tension_n_m`, `ripple_cutoff_m`, `min_phase_speed_m_s`, `superfluid`, `liquid_note`, `foam_albedo`, `drive`) sit once on `data.sea` and on `survey.climate.sea`, not repeated per band; each card band carries `trains`, `whitecap_fraction` and `streak_dir_deg`, and the site report carries them for the site. Details in `packages/W-sea2.md`.
Site: the weather op's `report` gives `data.sea`. Globe: `survey.climate.sea.bands[]` gives one entry per climate band, null over land. Both use the same keys. Every key is additive, and a missing key means not sent yet.
- W-sea2 colour:
  - `bands_nm` [650, 550, 450]
  - `absorb_per_m` [R,G,B] and `backscatter_per_m` [R,G,B]
  - `refractive_index`
  - `deep_rgb`, computed by the kernel: R∞ ≈ 0.33 b_b/(a + b_b)
  - The shader does the shallow fade exp(−2az) and Fresnel.
- W-sea3 waves:
  - `trains`, at most 6, each {wavelength_m, amplitude_m, period_s, dir_deg}, from the kernel's spectrum with ω² = gk + (σ/ρ)k³
  - `ripple_cutoff_m`
  - `superfluid` (bool) and `liquid_note`
  - `wave_m` and `period_s` stay.
- W-sea4 froth: `whitecap_fraction`, `foam_albedo`, `streak_dir_deg`.

