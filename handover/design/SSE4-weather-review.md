# SSE4 weather review: fitting weather to the new geology, as processes not classes

Written 21 Nov 2026 by the thread "Review weather against new geology", at Alex's request ("is weather here too? It needs similar review and redo"). Status: **accepted 21 Nov 2026** (Alex: "yes", with section 9's recommendations as rulings). Packages W7a to W12 are in the build plan; the build thread "Build shared climate and weather" takes W7a first, landing with S0b and S16a. Reviewed against `main` at 7d8670b: `crates/sse-weather` (8,000 lines, W1 to W6), `packs/base-weather/pack.json`, `crates/sse-kernel/src/weather.rs`, and the climate code inside `crates/sse-surface` (`cover.rs`, `wear.rs`).

Read with: `SSE4-weather-design.md` (the authority for weather today), `SSE4-rock3-comparison.md` (accepted: detail levels, strata, wear, caves, section 5.4 climate catch-up), `SSE4-exogeology-gaps.md` (section 5.9, moving ices), and `SSE4-processes-not-classes.md` (the standing rule, accepted 21 Nov).

The moving-ices split was agreed with the exogeology thread on 21 Nov and is written into section 4.

---

## 1. Short answers

- **Is weather covered by the new geology work?** No, not yet. The geology documents ask weather for things it does not give today, and the rule audit (`SSE4-processes-not-classes.md`, end of section 2.2) leaves weather to "its next package". This document is that review.
- **The biggest finding is not about physics.** SSE4 has **four separate climates** that do not talk to each other. Erosion (`wear.rs`) rains on a fixed cosine of latitude with 3 cells on every world. Cover (`cover.rs`) has its own temperature contrast and cell count. The weather rule block has a third temperature model. The zonal energy-balance block has a fourth. **Nothing in geology reads the weather crate at all.** `sse-surface` does not depend on `sse-weather`; the dependency runs the other way. So the rain that cuts the valleys is not the rain the player sees, and the "climate averages for erosion" that W1 publishes have no reader.
- **The weather code is also not wired to real worlds.** The kernel `weather` op runs only on fixtures or on conditions the caller types in. Spin, tilt and air are defaults (known limit). Spin is stored as always positive, so Venus and Uranus cannot spin backwards.
- **Processes, not classes.** The weather code has the same root problem the surface code had: a first-match regime list (airless, banded giant, locked, thin air, slow, fast, Earth-like) that switches whole features on and off. I count **about 25 hard gates** (section 3). About 4 are real edges in nature, the rest become strength blocks.
- **Coverage.** The design names the right Solar System weather, but several famous features are missing or faked: Venus's super-rotation, Mars's CO2 pressure cycle, Pluto and Triton's nitrogen air, ice giants' retrograde equators for the right reason, Saturn's storm clock for the right reason, dust on wet worlds, hot-Jupiter heat redistribution and atmospheric collapse on locked worlds (section 5).
- **What to keep.** Most of it. The `WeatherField` contract and the swap (W6) are good and stay unchanged. So do the pins, the descent column and sound hooks, Mastin plume heights, ballistic plumes, Burger-number poles, giant band layout and the energy-balance block, which becomes the core of the one climate.
- **The redo** is six packages, W7a to W12, in section 8. The first one, "one climate", matters most because every erosion, cave and glacier package reads from it.

---

## 2. What is built, and how it is joined up

| Piece | Where | What it does | Reads | Read by |
|---|---|---|---|---|
| Conditions | `sse-weather/conditions.rs` | spin, tilt, flux, pressure, gases, ocean; `from_surface` fills gaps with defaults | surface envelope, an `Extra` | weather only |
| Circulation | `circulation.rs` | Held and Hou Hadley edge, Rhines cell count with **a fixed wind of 12 m/s**, regime = first pack rule that matches | Conditions | weather only |
| Condensation | `circulation.rs` | Clausius-Clapeyron from a triple point for water, CO2, CH4, NH3, N2; **only the lowest cloud base wins** | pack substances | weather only |
| Climate averages | `circulation.rs` | bands of cloud, rain, wind; energy-limited rain; `rain_cells` "for S3" | the above | **nobody outside weather** |
| Formations | `formations.rs` | storm tracks, tropical cyclones, dust, locked shield, polar vortex, orography, lightning, snow, report | regime string | kernel op |
| Giants | `giants.rs` | bands, ovals, strings, dark spots, outbreaks, Burger-number poles, polygon jets | Conditions | kernel op |
| Aerosols | `aerosols.rs` | plumes, ash, vog, veils, impacts, fire, smog, spray, haze | caller-given sources | kernel op |
| Zonal EBM | `zonal.rs` | North 1975 diffusive energy balance, seasons, ice albedo, moisture, frost cloud | Conditions | swap test only |
| Surface climate | `sse-surface/cover.rs` | own mean, contrast (`contrast_k` 120 K over a transport number), own Rhines cells (U 50 m/s), lapse | envelope | cover, deposits |
| Erosion rain | `sse-surface/wear.rs` | `0.5 + 0.5 cos(2 × rain_cells × lat)`, `rain_cells` = 3 for every world, scaled by sqrt(oceans) | pack | stream power, glaciers, dunes |
| Kernel op | `sse-kernel/weather.rs` | `want` = climate, systems, field, report, column, sound | fixture name or typed conditions | renderer, UI |

So the weather *drawing* is rich and well cited, but the weather *climate* underneath it is three or four disconnected guesses, and the one that shapes the ground is the crudest.

---

## 3. Audit against "processes, not classes"

Same verdicts as the surface audit: **Edge** is real physics, kept but computed. **Smooth** becomes a strength block, a logistic in log(ratio) centred on the critical value, width from the spread in the cited measurements or a factor of 2 by default. **Fudge** stays for now, registered, until the stage that replaces it.

### 3.1 The regime list (`weather.rules`, first match wins)

| Today | Verdict | Replacement physics |
|---|---|---|
| `airless`: p ≤ 1e-6 bar | Edge with width | Collisional air stops where the mean free path exceeds the scale height (Knudsen number about 1). That gives an exosphere, not a wind field. It is computed from pressure, temperature and molecule size, so Pluto's 10 µbar stays collisional (it has haze layers and winds) and the Moon does not. |
| `banded-giant`: no solid surface | Edge | Kept: no surface is a fact. But giants and rocky worlds then run the **same** circulation processes, read at the 1 bar level, as the surface audit asks for the `cloud-deck` rule. |
| `tidally-locked`: star_locked | Smooth | Locked is a spin fact, but its consequences are strengths. Day-night contrast is the ratio of the radiative time to the wave or advection time (Komacek and Showman 2016). Collapse comes from night-side temperature against the frost point of the main gas (Wordsworth 2015). The pattern (substellar convection, Rhines-scale jets, superrotation) follows from the Rossby deformation and Rhines lengths against the radius (Haqq-Misra et al. 2018). Slow rotators that are not locked then sit on the same curves. |
| `thin-air`: p ≤ 0.03 bar | Smooth | There is no physical edge at 30 mbar. Replace with three strengths: the radiative time against the day (how large the daily swing is), the fraction of the air that freezes out each year (pressure cycle, section 4) and saltation (dust, section 3.2). Mars becomes "thin, cold, dusty, seasonal" because of numbers, not because of a label. |
| `slow-rotator`: Hadley edge ≥ 60° | Smooth | The thermal Rossby number Ro_T = g H (Δθ/θ) / (Ω a)² already sits inside Held and Hou's formula. Hadley extent is sqrt(5/3 · Ro_T). Super-rotation turns on as Ro_T passes about 1 (Mitchell and Vallis 2010). One number, no class. |
| `fast-rotator`: cells ≥ 4 | Smooth | The Rhines cell count is already continuous. Only the label is a cut. |
| `earth-like`: fallback | Label only | Read off the strengths. |
| Rhines wind `wind_m_s` 12 m/s for every world | Fudge, then smooth | The wind scale must come from the forcing: thermal wind balance U ≈ g H Δθ / (θ f a) in fast rotators, cyclostrophic balance in slow ones, capped by the energy available. Then jets, cells, storm sizes, dust lifting and wave heights all follow from the world, not from Earth. This is the single biggest physics fix. |
| Hadley contrast `delta_h` 1/3 for every world | Smooth | From the energy balance block's own equator-to-pole contrast (it already computes one). |
| Cell layout `room = 90 − 8 × (cells − 1)` | Fudge | Geometry for drawing; registered. |

### 3.2 Gates inside the formations, giants and aerosols

| Today | Verdict | Replacement physics |
|---|---|---|
| Storm-track cyclones only in some regimes | Smooth | Baroclinic growth rate σ = 0.31 f (∂U/∂z) / N (Lindzen and Farrell 1980) against the radiative damping rate. Weak on Venus, strong on Earth and Mars, no regime list. |
| Tropical cyclones: sea above 26 °C, 5° from the equator, regimes `earth-like` or `fast-rotator` only, water only | Smooth | A genesis potential index: absolute vorticity, mid-level humidity, potential intensity and wind shear, multiplied (Emanuel and Nolan 2004; Camargo, Emanuel and Sobel 2007). Potential intensity comes from sea temperature against outflow temperature, so it works for any surface liquid and its latent heat. Methane cyclones over Titan's seas become a labelled, debated option. The 26 °C line falls out on Earth instead of being typed in. |
| Dust: only `thin-air`, dry, solid | Smooth | Saltation strength = friction speed over threshold friction speed (Iversen and White 1982), which depends on grain size and density, gravity and air density. Supply = loose fines from geology's cover. Runs on every world: Sahara dust, Titan's equinox dust storms, Pluto's dunes. **Shared with geology's dune flag** (section 4). |
| Global dust storm: a fixed chance per year and a fixed season | Fudge | No closed published relation exists; it is a radiative-dynamic feedback found in models (Newman, Lewis and Read 2002). Keep as a drawn event whose chance scales with the dust-lifting strength and perihelion heating. Season from the world's own perihelion, not `season_phase` 0.69. Registered. |
| Polar vortex only if tilt > 5° | Smooth | Winter-pole cooling from insolation and radiative time; vortex strength from the resulting temperature gradient. |
| Lightning only in "cumulonimbus" cells, Earth flash curves | Smooth | Flash rate proportional to convective energy times precipitation (Romps et al. 2014), with a mixed-phase factor (charge needs ice and graupel or the substance's equivalent). Jupiter's water clouds get lightning for the right reason; Venus is a labelled option. |
| Snow below 273.15 K | Edge with width | A phase edge, but per substance and with a melting layer width (wet-bulb). |
| Fog when cover > 0.9 stratocumulus | Smooth | Saturation at the ground from temperature against dew point. |
| Wildfire below 15% oxygen: none | Smooth | Ignition and spread probability rise steeply with oxygen; no sustained fire below about 15% (Belcher and McElwain 2008). Use their curve, times fuel dryness from the rain block. |
| Photochemical deck (SO2 ≥ 1e-5, flux ≥ 100, p ≥ 1 bar), organic haze (CH4 ≥ 0.005), sulphur haze | Smooth | Haze is production against removal. Production from the UV photon flux and the parent gas column (the same block as geology's `sse.tholin.production`). Removal by settling and rain-out. Organic haze on CO2-rich air needs CH4/CO2 near 0.1 or more (Trainer et al. 2006), so early Earth can be orange. |
| Condensation: only the lowest cloud base wins | Smooth | Every condensable that saturates makes its own layer, weighted by its mass. Mars gets water and CO2 cloud, Titan methane and ethane, giants already do this. |
| Rain: `latent_share` 0.5, `cover_max` 0.75 and similar | Fudge, then smooth | Rain = evaporation from the surface energy balance (the zonal block's bulk formula), moved by moist transport, capped by energy. Whether rain reaches the ground comes from drop evaporation on the way down (Loftus and Wordsworth 2021), which gives Venus's acid virga and Titan's light methane rain. |
| Ice albedo below 263 K | Edge | The sea's freezing point from its salt, shared with geology's `sea_ice_k`. |
| ITCZ follows the sun by 0.35 over ocean, 0.6 over land | Smooth | From surface heat capacity (seasonal lag); monsoons as a regime transition of the overturning (Bordoni and Schneider 2008). |
| Ice giant if mass < 50 Earth masses (retrograde equator) | Smooth | Equatorial jet direction from large-scale latent heating: low water gives super-rotation (Jupiter, Saturn), high water gives a retrograde equator (Uranus, Neptune) in the same model (Lian and Showman 2010). Reads envelope water from the formation track. |
| Planet-circling outbreaks only below 150 K | Smooth | Moist convection in hydrogen air is held back when water's mixing ratio passes a critical value, and the storm period is set by the radiative cooling time of the layer above (Li and Ingersoll 2015). Saturn's 30-year clock falls out; Jupiter's water abundance keeps it quieter. |
| Polar regime by Burger number with three hard cut-offs; diffuse cap below 0.1 W/m² | Smooth | Already the right physics (Brueshaber, Sayanagi and Dowling 2019; O'Neill, Emanuel and Flierl 2015). Turn the cuts into logistics in log Bu, with width from the 30 to 40 gap in a/L_d. Convective forcing as a strength. |
| Polygon: latitude ≥ 70°, jet ≥ 100 m/s, regime single vortex | Smooth | Polygon strength from jet narrowness and strength against the latitude circle; keep the side-count rule. |
| Plume ballistic below 1e-4 bar | Edge with width | Buoyant against ballistic is set by whether the air can entrain the plume (plume momentum against air column). Keep a width. |
| Spray above 3 m/s wind | Smooth | Whitecap fraction against wind speed. |

**Net**: of about 25 gates, four are real edges (no surface, phase changes for snow and sea ice, the collisional edge), one is drawing geometry, two stay as registered fudges (global dust storm odds, cell layout), and the rest become strength blocks named `sse.weather.<process>`, as the exogeology thread proposed.

---

## 4. What geology needs from weather, and what weather needs from geology

Agreed by message with the exogeology thread on 21 Nov.

### 4.1 Weather gives geology

| Geology needs | For | Today | Proposal |
|---|---|---|---|
| Rain and snow per cell, averaged over each epoch | S3 stream power, S9 strata (facies need wetness), S14b karst and water table, land ice mass balance | fixed cosine in `wear.rs` | `ClimateAverages` per cell from the one climate (W7a), with snow and rain split |
| Runoff (rain minus evaporation) | karst dissolution rate, water table, cave levels | none | from the same balance |
| Temperature per cell, season range | chemical weathering rate, frost shattering, permafrost, periglacial rules (M6) | own model in `cover.rs` | one climate, with land and sea (W7c) |
| Wind near the ground, friction speed | dunes and yardangs, dust; replaces `wear.dune_pressure_bar` (it would forbid Pluto's dunes) | none | `sse.wind.saltation`, one shared block |
| Prevailing wind direction | rain shadows, dune orientation, ash fall | own `easterly()` in cover | from the circulation block, signed spin |
| Insolation history over orbit and tilt cycles | ice ages, moraines, raised beaches, polar layered deposits | none | W9, from the T1 orbit history |
| Volatile transport (N2, CH4, CO, CO2) | caps, glaciers, Sputnik's fill | none | `sse.volatile.exchange`, below |
| Haze production and fallout | tholin colour on the ground (S19) | haze is a fixed optical depth | one production rate; weather draws the sky, geology colours the ground |
| Lightning and fire | charcoal and fire layers in strata, fulgurites | drawn only | yearly rates per cell |

### 4.2 Geology gives weather

Height map and land fraction (orographic rain, basins for smog and fog); surface cover and its albedo, heat capacity and roughness; loose fines and sediment (dust supply); volcanic sources and eruption events (W4 sources stop being caller inputs); ice caps and their albedo; ground volatiles (what can sublime); sea salt and composition (freezing point).

### 4.3 The moving-ices block, designed once

- **S17 (exogeology, built by the physics thread) owns the ice on the ground**: caps, basin traps, glaciers, convection cells, pits, penitentes, layered deposits.
- **One species table**, `sse.volatile.species`, in base physics: Fray and Schmitt 2009 vapour-pressure fits, latent heat, density, and the N2 flow law. Weather's cloud condensation (W1) switches to it, so ground frost and cloud can never disagree.
- **One exchange block**, `sse.volatile.exchange`, owned by weather: per latitude band and season it returns the mass flux between ground ice and air, and the surface pressure set by the coldest ice (vapour-pressure buffering). S17 calls it orbit-averaged at D1 and seasonal at D2. Weather reads the same output for Mars's CO2 pressure cycle, Pluto's and Triton's pressure changes, the condensation flow from the summer pole to the winter pole, polar hood cloud and CO2 snow.
- **Weather publishes back** its band winds and heat transport, which the exchange uses instead of assuming.
- **Haze**: S19's `sse.tholin.production` feeds W4's haze layer. **Dust colour** from S19 feeds the dust storms.

---

## 5. Coverage: real weather in the Solar System and in theory

Status of each feature in the built code. "Processes" says which strength block makes it after the redo.

| World or case | What is real | Today | After the redo |
|---|---|---|---|
| Earth | three cells, storm tracks, monsoons, tropical cyclones, marine decks, coasts mild and interiors harsh, ice ages | cells, tracks, cyclones, decks; no land and sea contrast, decks on any subtropical sea, no ice ages | land and sea (W7c), currents as rules, insolation cycles (W9), genesis index |
| Venus | slow retrograde spin, air super-rotating about 60 times faster than the ground, world-wide acid deck, polar vortices | "slow rotator" with a deck by gas thresholds; spin cannot be retrograde; **no super-rotation** | super-rotation strength from Ro_T (Mitchell and Vallis 2010; Gierasch-type mechanism, Sánchez-Lavega et al. 2017); signed spin; deck from production and removal; acid virga |
| Mars | CO2 pressure swings about 25 to 30% a year as caps freeze and sublime, CO2 snow, dust devils, regional and global dust storms, water ice clouds | thin-air dust storms on a fixed calendar; **no pressure cycle**; one condensate | `sse.volatile.exchange`, saltation, dust devils, multiple condensates, perihelion season from the orbit |
| Titan | methane cycle, equatorial storms near equinox, polar lakes fed by rain, orange haze, dust storms | methane condensation and haze by thresholds | multiple condensables, raindrop survival, haze production, saltation (dust at equinox, Rodriguez et al. 2018), cyclones over seas as a debated option |
| Pluto, Triton | N2 air in vapour-pressure balance with surface ice, pressure tripling since 1988, layered haze, Triton's geyser plumes | "thin air" at 10 µbar would be treated as nearly airless; nothing seasonal | exchange block with S17 (Forget et al. 2017; Bertrand and Forget 2016), haze from production; geyser plumes through W4 |
| Io | patchy SO2 air that freezes out at night and in eclipse, volcanic plumes | ballistic plumes (good) | exchange block with SO2 as the species |
| Jupiter, Saturn | belts and zones, super-rotating equator, deep jets, great ovals, String of Pearls, polar cyclone ring and hexagon, Saturn's 30-year storms, lightning | all drawn, with good citations; outbreaks by a 150 K cut | Li and Ingersoll storm clock, smooth Burger poles, lightning from convective energy |
| Uranus, Neptune | retrograde equator, prograde high-latitude jets, dark spots, extreme tilt seasons on Uranus | retrograde by a mass cut; Uranus's 98° tilt not used | jet direction from latent heating (Lian and Showman 2010); tilt seasons through the same insolation block |
| Locked temperate worlds | substellar convection, cloud shield, day-night jets, eyeball oceans, possible collapse | permanent shield and twin gyres drawn by rule | day-night contrast by timescales, collapse by frost point, eyeball sea ice from the energy balance with ocean heat transport as a strength (Hu and Yang 2014) |
| Hot Jupiters | super-rotating equatorial jet, hot spot shifted east, day-night contrast rising with temperature | banded giant only | the same timescale ratio (Komacek and Showman 2016); a labelled hot-spot offset |
| Lava worlds | rock-vapour air, mineral rain, collapse 90 to 100° from the substellar point | mineral cloud by temperature (giants only) | exchange block with rock vapours (shared with S17 and M9) |
| Brown dwarfs | banded, patchy clouds, variability | treated as a giant | same giant processes once stars and planets are one body |
| Thin and collapsing air in general | air that freezes out entirely in cold seasons or on night sides | not modelled | exchange block plus collapse strength |

---

## 6. Detail levels and browser cost

Weather follows the accepted Rock 3 rules: reuse before computing, go up a level only on approach, stay or request, and the coarse result pins the fine one.

| Level | Weather computed | Cost (estimate, measure in W11) |
|---|---|---|
| **D0 Glance** | climate scalars from orbit and air: mean temperature, Ro_T, day-night ratio, which species condense, haze yes or no, sky colour hint. Strengths give the label | microseconds |
| **D1 Orbit** | the one climate: zonal energy balance, 36 bands × 12 seasons (built, `zonal.rs`); band winds, cloud bands, rain bands for the globe; volatile exchange orbit-averaged; the epoch averages erosion reads | about a millisecond per epoch; cached by world key, code version and epoch |
| **D2 Survey** | land and sea per cell on the D1 grid (North, Mengel and Short 1983, 2D energy balance), orographic rain on terrain, seasonal volatile exchange, drawn storm systems by date, field tiles | tens of milliseconds per epoch on the 6,144-cell grid, in a worker |
| **D3 Site** | the report, the descent column, sound, local fog, valley and katabatic winds, cave air | per request, already built for the column and report |

Deep time: erosion runs in 100 Myr (D1) and 25 Myr (D2) steps, so climate is needed per epoch, not per day. One climate per epoch is cheap. Ice-age cycles are an average over the insolation history within the epoch, not stepped. Day-to-day weather is never stored; it is redrawn from keys, as now.

---

## 7. The redo: what is kept and what changes

**Kept as is**: the `WeatherField` contract, `WeatherBlock` trait, the swap and its report (W6), the save manifest, pins and GM orders, the kernel op shape, the descent column and sound hooks, Mastin and Morton-Taylor-Turner plume heights, ballistic plumes, ash streaks and fall, veil timing, giant band layout, the polygon side-count rule, all W tests that check behaviour rather than thresholds.

**Kept and promoted**: the zonal energy-balance block becomes the core of the one climate, under the rule block's formations instead of beside them. The rule block stops having its own temperature model.

**Changed**:
1. One climate below both geology and weather (a new crate, `sse-climate`, so the surface can depend on it without a cycle).
2. Weather wired to the body object: spin (signed), tilt, air from the physics thread; defaults only where a fact is truly missing, and listed.
3. Regime list replaced by strength blocks `sse.weather.<process>`; `regime` kept one version as a derived label.
4. Wind scale from the forcing, not 12 m/s.
5. The shared blocks: `sse.volatile.species`, `sse.volatile.exchange`, `sse.wind.saltation`, haze production with `sse.tholin.production`.

**Each block gets a fitting UI element** (in the Workshop, per the top-level UI design): a strength gauge with its ratio and critical value ("super-rotation 0.92: thermal Rossby number 9.4 against 1"), a small band chart for climate blocks (temperature, rain and wind by latitude, scrubbable by season), and a pressure-through-the-year line for the exchange block. Each answers "why is this here?" with the ratio and the paper.

---

## 8. Build packages, in priority order

Order follows what geology needs first: S3, S9 and S14b need rain and runoff; S17a needs the exchange block; S16 needs the levels.

| Package | Contents | Needs | Tests (evidence) |
|---|---|---|---|
| **W7a One climate** | `sse-climate` crate: Conditions, energy balance (from `zonal.rs`), circulation; `cover.rs` and `wear.rs` read `ClimateAverages` per cell and epoch instead of their own models; signed spin; body address in the op; D0 and D1 keyed cache | B1 one body (spin, tilt, air) or its interim facts | Earth, Mars, Venus, Titan: cover, wear and weather read the same rain and temperature to the last bit; no Rhines or contrast code outside the crate; retrograde Venus flips its winds; erosion goldens change once, on purpose, with S0b |
| **W7b Circulation as strengths** | Ro_T, super-rotation, baroclinic growth, day-night timescale ratio, collapse, collisional edge; wind scale from thermal wind; label derived | W7a | fixture labels unchanged for the Solar System; a spin sweep from 1 h to locked changes every output smoothly (no step larger than a stated amount between neighbouring samples); Venus super-rotates and Earth does not; a cold locked world collapses and a warm one does not; Pluto stays collisional, the Moon does not |
| **W9 Supply to geology** | rain, snow and runoff per cell and epoch; `sse.wind.saltation` shared with the dune flag; insolation history from T1 orbit cycles; land-ice mass balance supply | W7a, T1 | Pluto dunes allowed at 10 µbar, none on the Moon; Earth's driest and wettest bands rank as observed; an obliquity-cycle fixture writes the expected number of glacial advances |
| **W8 Moving ices (with S17a)** | `sse.volatile.species` (switch W1 to it), `sse.volatile.exchange`, condensation flow, polar hood and CO2 snow, collapse | W7a; built alongside S17a | Mars pressure cycle with two minima and an amplitude of 25 to 30%; Pluto's pressure follows the N2 frost temperature; Triton gets polar caps and Pluto low-latitude ice with only the orbit changed (Bertrand et al. 2024's test) |
| **W7c Land, sea and seasons** | the accepted Rock 3 W7: heat capacity per cell, coastal mixing, current rules (western boundary warm, eastern upwelling cold), ITCZ lag from heat capacity, eyeball ice on locked worlds | W7a, S7 | Earth's coastal versus interior seasonal range in the observed sense; marine decks only over upwelling coasts; a retrograde fixture's currents flip |
| **W10 Storms, aerosols and giants as strengths** | genesis index cyclones, saltation dust everywhere, dust devils, lightning from convective energy, fire by Belcher and McElwain, haze from production and removal, every condensable its own layer, raindrop survival, Lian and Showman jets, Li and Ingersoll storm clock, smooth Burger poles | W7b, W9; S19 for haze | no cyclones without a liquid; Earth's genesis peaks in late summer basins; dust on Earth over deserts and on Titan at equinox; Saturn's storm period about one Saturn year and Jupiter's longer; every gate in section 3 has a test that a world just either side of the old cut gets nearly the same answer |
| **W11 Levels and cost** | D0 to D3 mapping, reuse cache shared from the GM, measured costs on a phone | W7a | D1 under 0.2 s for all epochs on the reference phone; D2 averaged back to D1 within tolerance; same results on every device |
| **W12 Blocks in the Workshop** | gauges, band charts, pressure lines; "Why?" for each | W7b, UI shell | every process shows its ratio and paper; changing a pack number moves the gauge and the globe |

---

## 9. Rulings (accepted 21 Nov 2026: each recommendation below is the ruling)

1. **Where should the one climate live?** *Recommendation*: a new crate `sse-climate` that both `sse-surface` and `sse-weather` depend on. Keeping it in `sse-weather` would force geology to depend on the weather drawing.
2. **Which block is the default after the swap?** *Recommendation*: they stop being rivals. The energy-balance block becomes the climate core. The rule block becomes the weather drawn on top of it. The swap stays for both layers, so a real circulation model can later replace the core and a simulation can replace the drawing.
3. **Who builds it?** *Recommendation*: a new Opus build thread, "Build weather processes", that takes W7a to W12, coordinating with the physics thread on the shared blocks and on golden timing. The old weather build thread is finished.
4. **Golden hashes.** *Recommendation*: change them once, in W7a, together with S0b, so the surface and weather goldens move in the same package with this document as the reason.
5. **Spin and tilt before the one-body object lands.** *Recommendation*: W7a takes them from the formation run where it has them (it does for formed systems) and from V3 imports, and lists any default it still uses, as `from_surface` does now. It does not wait for B1. Alex (21 Nov): V3's default data is V3's own, authored over time, and may be wrong. So values taken from a V3 import carry the evidence mark "authored (V3)", are never treated as measured, and where our own physics disagrees by more than a stated tolerance the "Why?" panel shows both and flags the difference rather than silently overriding either.
6. **Global dust storms have no closed physics.** *Recommendation*: keep a drawn event whose odds scale with the dust-lifting strength, registered as a fudge, with the season taken from the world's own perihelion.
7. **Debated weather** (Venus lightning, cyclones on Titan's seas, hot-spot offsets): *Recommendation*: labelled options, default on only where our own blocks produce them, as question 13 of the exogeology review rules for geology.
8. **Ice ages where there is no orbit history.** *Recommendation*: use T1's history where it exists; otherwise draw quasi-periodic cycles keyed on the orbit, labelled drawn.

---

## 10. Sources

Status: **found** means I confirmed the paper exists, with its title and venue, by search on 21 Nov. **Pack** means it is already cited in `packs/base-weather` and was checked by the citations track (`handover/packages/W-citations/`). **Recalled** means not checked this session. I read none of these in full here; the numbers quoted are from abstracts, the pack or memory, and the citations thread should check each relation before a package relies on it. DOIs given from memory are marked so.

**Must be read before entering the citations register.** No paper new in this review has been read in full. Each package's builder (or the citations track) reads the ones its relations rest on, checks the relation and the numbers quoted here, adds the DOI, and only then registers it:

| Package | Read first |
|---|---|
| W7a | none new (Held and Hou, Rhines, North 1975 are pack-checked) |
| W7b | Mitchell and Vallis 2010 (Ro_T threshold near 1); Lindzen and Farrell 1980 (the 0.31 factor); Komacek and Showman 2016 (timescale ratio); Haqq-Misra et al. 2018 (regime lengths); Wordsworth 2015 (collapse condition) |
| W7c | North, Mengel and Short 1983 (heat capacities, mixing); Bordoni and Schneider 2008; Hu and Yang 2014 |
| W9 | Iversen and White 1982 and Telfer et al. 2018 (shared with S17; read once by whichever lands first) |
| W8 | Forget et al. 2017; Fray and Schmitt 2009 and Bertrand et al. 2024 (exogeology checked these via Consensus; re-check the coefficients) |
| W10 | Emanuel and Nolan 2004 and Camargo, Emanuel and Sobel 2007 (index form and constants); Romps et al. 2014; Belcher and McElwain 2008 (the oxygen curve, not just the 15% floor); Trainer et al. 2006 (the CH4/CO2 ratio); Loftus and Wordsworth 2021; Lian and Showman 2010; Li and Ingersoll 2015 (critical mixing ratio); Newman, Lewis and Read 2002; Rodriguez et al. 2018; find the DOI for Brueshaber, Sayanagi and Dowling 2019 |

"Found" papers above are confirmed to exist; "recalled" ones (Lindzen and Farrell, Hu and Yang, Bordoni and Schneider, Emanuel and Nolan, Rodriguez) must also have their existence and venue confirmed.

| Paper | Used for | Status |
|---|---|---|
| Held and Hou 1980, J. Atmos. Sci. 37, 515, doi:10.1175/1520-0469(1980)037<0515:NASCIA>2.0.CO;2 | Hadley extent, Ro_T | pack |
| Rhines 1975, J. Fluid Mech. 69, 417, doi:10.1017/S0022112075001504 | jets and cells | pack |
| Kaspi and Showman 2015, ApJ 804, 60, doi:10.1088/0004-637X/804/1/60 | trends across spin and air | pack |
| Mitchell and Vallis 2010, JGR 115, E12008, "The transition to superrotation in terrestrial atmospheres", https://arxiv.org/abs/1008.1996 | super-rotation from Ro_T | found |
| Sánchez-Lavega et al. 2017, Space Sci. Rev. 212, 1541, doi:10.1007/s11214-017-0389-x | Venus dynamics | pack |
| Lindzen and Farrell 1980, J. Atmos. Sci. 37, 1648, "A simple approximate result for the maximum growth rate of baroclinic instabilities" | baroclinic growth | recalled |
| Komacek and Showman 2016, ApJ 821, 16, "Atmospheric circulation of hot Jupiters: dayside-nightside temperature differences", https://arxiv.org/abs/1601.00069 | day-night timescale ratio | found |
| Haqq-Misra et al. 2018, ApJ 852, 67, "Demarcating circulation regimes of synchronously rotating terrestrial planets within the habitable zone", https://arxiv.org/abs/1710.00435 | locked circulation by Rossby and Rhines lengths | found |
| Wordsworth 2015, ApJ 806, 180, "Atmospheric heat redistribution and collapse on tidally locked rocky planets", https://arxiv.org/abs/1412.5575 | collapse | found |
| Yang, Cowan and Abbot 2013, ApJL 771, L45, doi:10.1088/2041-8205/771/2/L45 | cloud shield | pack |
| Showman and Polvani 2011, ApJ 738, 71, doi:10.1088/0004-637X/738/1/71 | locked super-rotation | pack |
| Hu and Yang 2014, PNAS 111, 629, "Role of ocean heat transport in climates of tidally locked exoplanets around M dwarf stars" | eyeball ice | recalled |
| North 1975, J. Atmos. Sci. 32, 2033, doi:10.1175/1520-0469(1975)032<2033:TOEBCM>2.0.CO;2 | energy balance core | pack |
| North, Mengel and Short 1983, JGR 88, 6576, "Simple energy balance model resolving the seasons and the continents" | land and sea, seasons | found |
| Bordoni and Schneider 2008, Nature Geosci. 1, 515, "Monsoons as eddy-mediated regime transitions of the tropical overturning circulation" | monsoons, ITCZ | recalled |
| Emanuel and Nolan 2004, genesis potential index (AMS 26th Hurricanes conference) | cyclone genesis | recalled |
| Camargo, Emanuel and Sobel 2007, J. Climate 20, 4819, "Use of a genesis potential index to diagnose ENSO effects on tropical cyclone genesis", https://www.ldeo.columbia.edu/~suzana/papers/camargo_emanuel_sobel_jclim07.pdf | cyclone genesis | found |
| Emanuel 2003, Annu. Rev. Earth Planet. Sci. 31, 75, doi:10.1146/annurev.earth.31.100901.141259 | potential intensity | pack |
| Iversen and White 1982, Sedimentology 29, 111, doi:10.1111/j.1365-3091.1982.tb01713.x | saltation threshold | from the processes-not-classes audit |
| Telfer et al. 2018, Science 360, 992, doi:10.1126/science.aao2975 | Pluto dunes at 10 µbar | from the processes-not-classes audit |
| Newman, Lewis and Read 2002, JGR 107(E12), "Modeling the Martian dust cycle" parts 1 and 2, https://oro.open.ac.uk/4508 | dust lifting, storm feedback | found |
| Zurek and Martin 1993, JGR 98, 3247, doi:10.1029/92JE02936 | global dust storm years | pack |
| Rodriguez et al. 2018, Nature Geosci., "Observational evidence for active dust storms on Titan at equinox" | Titan dust | recalled |
| Romps et al. 2014, Science 346, 851, "Projected increase in lightning strikes in the United States due to global warming", https://ui.adsabs.harvard.edu/abs/2014Sci...346..851R | lightning from convective energy times rain | found |
| Price and Rind 1992, JGR 97, 9919, doi:10.1029/92JD00719 | current lightning curves | pack |
| Belcher and McElwain 2008, Science 321, 1197, "Limits for combustion in low O2 redefine paleoatmospheric predictions for the Mesozoic" | fire against oxygen | found |
| Trainer et al. 2006, PNAS 103, 18035, "Organic haze on Titan and the early Earth", https://pmc.ncbi.nlm.nih.gov/articles/PMC1838702 | haze needs CH4/CO2 near 0.1 | found |
| Mitchell and Lora 2016, Annu. Rev. Earth Planet. Sci. 44, 353, doi:10.1146/annurev-earth-060115-012428 | Titan climate | pack |
| Loftus and Wordsworth 2021, JGR Planets 126, "The physics of falling raindrops in diverse planetary atmospheres", https://arxiv.org/abs/2102.09570 | rain reaching the ground, any substance | found |
| Forget et al. 2017, Icarus 287, 54, "A post-New Horizons global climate model of Pluto including the N2, CH4 and CO cycles", https://ui.adsabs.harvard.edu/abs/2017Icar..287...54F | Pluto air and ices | found |
| Bertrand and Forget 2016, Nature 540, 86 | Sputnik's nitrogen | via exogeology review |
| Bertrand et al. 2024 (same model gives Pluto and Triton) | exchange test | via exogeology review (Consensus research) |
| Fray and Schmitt 2009, Planet. Space Sci. 57, 2053 | vapour-pressure fits | via exogeology review |
| Lian and Showman 2010, Icarus 207, 373, "Generation of equatorial jets by large-scale latent heating on the giant planets", https://ui.adsabs.harvard.edu/abs/2010Icar..207..373L/abstract | equatorial jet direction from water | found |
| Li and Ingersoll 2015, Nature Geosci. 8, 398, "Moist convection in hydrogen atmospheres and the frequency of Saturn's giant storms", https://www.nature.com/articles/ngeo2405 | storm clock | found |
| Vasavada and Showman 2005, Rep. Prog. Phys. 68, 1935 | giant bands and storms | pack (text) |
| Brueshaber, Sayanagi and Dowling 2019, Icarus, "Dynamical regimes of giant planet polar vortices" | Burger-number poles | pack (DOI still to find) |
| O'Neill, Emanuel and Flierl 2015, Nature Geosci. 8, 523, "Polar vortex formation in giant-planet atmospheres due to moist convection" | polar regimes | pack (text) |
| Adriani et al. 2018, Nature 555, 216, doi:10.1038/nature25491 | Jupiter's cyclone rings | pack |
| Mastin et al. 2009, J. Volcanol. Geotherm. Res. 186, 10, doi:10.1016/j.jvolgeores.2009.01.008 | plume heights (kept) | pack |

Sources searched this session: [Komacek and Showman 2016](https://arxiv.org/abs/1601.00069v2), [Haqq-Misra et al. 2018](https://arxiv.org/abs/1710.00435v4), [Li and Ingersoll 2015](https://www.nature.com/articles/ngeo2405), [Lian and Showman 2010](https://ui.adsabs.harvard.edu/abs/2010Icar..207..373L/abstract), [Mitchell and Vallis 2010](https://arxiv.org/pdf/1008.1996.pdf), [Belcher and McElwain 2008](https://physicstoday.aip.org/news/new-limits-on-prehistoric-oxygen-levels), [Wordsworth 2015](https://ar5iv.arxiv.org/html/1412.5575), [Romps et al. 2014](https://ui.adsabs.harvard.edu/abs/2014Sci...346..851R), [Loftus and Wordsworth 2021](https://web3.arxiv.org/abs/2102.09570), [Forget et al. 2017](https://ui.adsabs.harvard.edu/abs/2017Icar..287...54F), [North, Mengel and Short 1983](https://earthref.org/ERR/46393/), [Camargo, Emanuel and Sobel 2007](https://www.ldeo.columbia.edu/~suzana/papers/camargo_emanuel_sobel_jclim07.pdf), [Trainer et al. 2006](https://pmc.ncbi.nlm.nih.gov/articles/PMC1838702), [Newman, Lewis and Read 2002](https://oro.open.ac.uk/4508).
