# SSE4 weather design: convincing skies from rules, with room for a real simulation later

Written 19 Nov by the design thread at Alex's request. A companion to `SSE4-surface-design.md` (climate bands, cloud decks on giants, hazard layer, descent scene) and `SSE4-sound-design.md`. Authority for clouds, storms, haze, smoke and the weather a lander or a player meets.

Alex's rulings (19 Nov): a simple weather system that gives convincing cloud formations in the right places; volcanic smoke, ash and smog; polar features on giants (the hexagon and its relatives); and the whole thing an easy swap for a real weather simulation later, **without breaking saves**.

---

## 1. Principles

1. **Rules from physics already in hand.** Spin, sunlight, air density and gas mix, oceans, terrain, tilt and season, tidal locking, and for giants internal heat. No fluid simulation at this level. Every formation answers "Why is it here?".
2. **Climate is fact, weather is a drawing of it.** The kernel publishes a **weather field**: where systems are, how they move, how thick the cloud is, what falls. Individual storms are keyed draws from climate statistics, so they are the same for everyone and replay exactly, but they are labelled **drawn** (handoff evidence labels). Fine cloud texture below the field's resolution is GPU decoration.
3. **Honest about what is a rule.** A polar hexagon appears because a published condition is met, not because a simulation grew one. Explain mode says so.
4. **One field, many readers.** The renderers' cloud passes, the hazard layer and descent scene, sound (wind, thunder, rain), surface wetness and snow, erosion strength, the life suite's light and water, and player reports all read the same field.
5. **Swappable.** The weather block sits behind one contract (section 6). A simulation can replace it without touching readers or saves.

## 2. Rocky and ocean worlds: what forms where

**Step 1, the circulation regime**, from spin rate, size, air mass and heating (after Kaspi and Showman 2015, *Astrophysical Journal* 804, "Atmospheric dynamics of terrestrial exoplanets over a wide range of orbital and atmospheric parameters" **[existence checked]**):

| Regime | When | Pattern |
|---|---|---|
| Slow rotator | long day, or small world | one overturning cell per hemisphere reaching the poles; weak jets; a broad cloud band; Venus-like if the air is thick |
| Earth-like | moderate spin | tropical cell, mid-latitude storm track, polar cell: three belts per hemisphere |
| Fast rotator | short day, large world | more, narrower belts and several jets |
| Tidally locked | one face to the star | a permanent cloud shield and storm under the star, clearer night side, twin night-side gyres; the shield matters for habitability (Yang, Cowan and Abbot 2013, *Astrophysical Journal Letters* 771, L45 **[existence checked]**) |
| Thin air | pressure too low for weather to organise | dust devils, thin ice cloud, fog in basins, seasonal polar frost, global dust storms where there is dust |

The number and width of belts scale with spin and wind speed (the Rhines scale); the tropics' width shrinks as spin rises.

**Step 2, formations placed by rule:**

- **Rising-air belt** near the thermal equator, shifting with season: towers, heavy rain, lightning.
- **Clear subtropical bands** under sinking air: deserts where over land (this already drives the surface design's cover).
- **Mid-latitude storm track**: trains of comma-shaped cyclones with fronts, in number and size set by the regime; they travel with the jet and live for days.
- **Tropical cyclones**: only over warm enough ocean, off the equator, where spin is neither too weak nor too strong; drawn as tracked, named-able storms with seasons.
- **Mountain cloud**: banked on windward slopes with a dry shadow behind (the erosion terms already use this rain shadow); lee-wave clouds; cap clouds on high volcanoes.
- **Coast and sea**: marine layer decks over cold currents, sea-breeze lines, fog.
- **Polar**: vortex, frost and snow line by season; polar-night cloud.
- **Monsoon** where a large land mass sits beside a warm ocean under a seasonal sun.
- **Dust**: lifting where wind is strong and ground is dry and loose; regional or global storms by season.
- **Whatever condenses here**: the cloud and rain are of the substance that reaches saturation in this air: water, carbon dioxide ice, methane and ethane (a Titan-like cycle), sulphuric acid, ammonia, and on very hot worlds mineral cloud. Colour and droplet behaviour follow the substance.
- **Lightning** where convection is deep; it feeds thunder in the sound design and a hazard on descent.

## 3. Giants: belts, storms and poles

The existing cloud deck model (surface design 6) is kept: band count from spin and wind, deck composition by depth and temperature, long-lived storms that drift and merge. Additions:

- **Storm families**: great ovals in shear zones between jets; strings of smaller ovals; periodic planet-circling outbreaks on cooler giants; dark spots on ice giants.
- **Polar regime, chosen by rule.** Shallow-water studies find that a giant's poles fall into regimes controlled mainly by how the storm scale compares with the planet's size (a Burger number) and how energetic the convection is: one strong polar vortex (Saturn-like), a ring of cyclones around a central one (Jupiter-like), or a broad, diffuse cap (ice-giant-like). **[regime picture from Brueshaber, Sayanagi and Dowling, Icarus 2019, and O'Neill, Emanuel and Flierl, Nature Geoscience 2015; from memory, verify titles and thresholds before use]**
  - **Cyclone ring**: Juno found eight cyclones around Jupiter's north pole and five around its south (Adriani et al. 2018, *Nature* 555, "Clusters of cyclones encircling Jupiter's poles", doi:10.1038/nature25491 **[existence checked]**). SSE4 draws the count from the regime, keyed, and keeps the polygon stable.
  - **Polygon jet (the hexagon)**: Saturn's six-sided jet near 78° north has persisted since Voyager (Godfrey 1988, "A hexagonal feature around Saturn's north pole", *Icarus* **[title checked; volume and page to confirm]**). Laboratory and numerical work shows a strong, narrow circumpolar jet can settle into a stable wave whose number of sides depends on the jet's width and strength relative to its latitude circle: Barbosa Aguiar et al. 2010, *Icarus* 206, 755, "A laboratory model of Saturn's North Polar Hexagon" **[existence checked]**; Morales-Juberías et al., "Meandering shallow atmospheric jet as a model of Saturn's north-polar hexagon", *Astrophysical Journal Letters* 806, L18 (2015) **[existence checked]**. SSE4's rule: when a high-latitude jet meets the stated strength and narrowness condition, it carries a polygon with a side count from three to eight given by that relation. So some giants get hexagons, some pentagons or squares, most get none, and the two poles need not match.
- The rule is labelled as a rule. A later simulation block may grow these for real.

## 4. Smoke, ash, haze and smog

All tied to things the design already tracks.

| Source | From | What appears |
|---|---|---|
| **Volcanic plume** | active volcanoes and eruption events from the surface design and chronicle | a plume whose height follows eruption strength, gravity and air density (tall in thin air and low gravity, as on Io; squat in thick air), blown downwind as an ash streak; ash fall darkens ground and ice; a large eruption puts a sulphate veil in the upper air that cools the climate for years, as a dated event |
| **Volcanic gas** | steady outgassing | low haze and acid rain downwind of volcanic provinces |
| **Impact** | chronicle impacts | fireball, ejecta curtain, soot and dust veil, "impact winter" for big ones |
| **Wildfire** | living cover from the life suite, dry season, oxygen level | smoke plumes and regional haze; no fire where oxygen is too low |
| **Photochemical haze** | gas mix and ultraviolet from the star | planet-wide organic haze on methane-rich worlds (Titan-like orange), sulphur hazes |
| **Industrial smog** | the built layer: settlements and industry with their tech level (game or universe pack rules) | brown domes over cities trapped by inversions and basins; visible from orbit as a tell-tale of industry, and filtered by player data views like any other fact |
| **Dust** | section 2 | storms, and a dusty sky colour |
| **Sea spray and salt haze** | wind over ocean | a soft marine haze |

Each has an optical depth and colour that the atmosphere pass uses, so sunsets, visibility and the view from orbit change accordingly. Smoke and ash are also hazards for flight and landing.

## 5. Time, scale and what players meet

- **Time scales**: climate bands change over ages (they already follow the star and the air); seasons follow the orbit and tilt; storm systems live for days and are drawn per day along their tracks; a time scrub at high speed shows belts breathing with the seasons, at human speed shows fronts crossing.
- **Scale**: from orbit, the pattern of belts, spirals and plumes; on descent, the layers the craft passes through with wind, turbulence, icing, lightning and visibility at each height; on the ground, the sky overhead, what is falling, wind, and a short forecast.
- **For the table**: a plain weather report for any place and date ("wind 40 km/h from the west, dust rising, visibility 2 km, storm front in six hours"), usable as a hazard for travel and landing, and as colour for Minerva.
- **GM control**: pin a storm, order clear skies for a scene, or trigger an eruption or fire; all are authored events with the usual marks.
- **Deep time**: unrecorded storms are redrawn per era, exactly like unrecorded surface flourishes; a storm someone has seen and named is recorded and persists for its natural life.

## 6. The swap: one contract, replaceable block

```
WeatherField(body, time) -> {
  grid: coarse cells on the surface atlas's cube-sphere (same addressing as terrain tiles), with height layers,
  per cell and layer: cloud cover, cloud type, condensate, optical depth, precipitation kind and rate,
                      wind vector, temperature offset from climate, turbulence, lightning rate,
                      aerosols: [{ kind: ash | smoke | dust | haze | smog | spray, optical depth, colour }],
  systems: [{ id, address, kind, track, size, strength, born, ends }]      the nameable things: cyclones, ovals, plumes, polar features
  provenance: { block id and version, level: rule | simulated, evidence: drawn | calculated } }
```

- Readers use only this. The rule-based block fills it from rules and keyed draws. A later **simulation block** (shallow-water or a small general circulation model, as a heavy block with results as data, handoff 11 and 14) fills the same structure.
- **Why saves do not break**: a save never stores weather as bulk data. It stores the manifest (which weather block and version, handoff 5), authored weather events, and the short records of systems that were seen and named. Swapping blocks is a manifest change with a shown difference: climate facts stay, unrecorded weather is regenerated by the new block, recorded systems are carried over as pins the new block must honour. Old saves keep their old block until the GM chooses to move.
- Anything that feeds back into slow physics (rainfall for erosion, cloud cover for climate, light for life) uses climate **averages**, which both kinds of block must publish and which must agree within a stated tolerance. That keeps terrain, climate and life stable across a swap.

## 7. Rendering and sound hooks

- Cloud passes in the render recipes (research report section 5) read cover, type and optical depth: flat textured shell on low tiers, layered shells on basic GPU, raymarched volumes on high GPU. Formations come from the field; fine texture is noise on the GPU.
- The atmosphere pass reads aerosols for colour and visibility.
- Sound reads wind, precipitation, lightning and the gas for world sound (sound design N5).
- The descent scene reads the column along the path for telemetry lines and hazards (surface design S5).

## 8. Stages (track W)

| Stage | Contents | Needs | Evidence |
|---|---|---|---|
| W1 | WeatherField contract, circulation regime from the facts, climate averages published for erosion, climate and life | P1, S1 | Earth, Mars, Venus and Titan fixtures land in the expected regimes; averages feed the existing erosion and cover terms unchanged in form |
| W2 | Formations on rocky and ocean worlds (section 2), seasons, drawn storm systems with tracks; weather report | W1, S3 | Earth fixture: storm tracks, the rain belt and tropical cyclone basins fall in the right latitudes; no tropical cyclones on a world without warm ocean; same weather on every device for a given date |
| W3 | Giants: storm families and polar regimes including cyclone rings and polygon jets | W1, S6 | Jupiter-like and Saturn-like fixtures give a ring and a single vortex with a polygon; side counts follow the stated relation; most random giants show no polygon |
| W4 | Smoke, ash, haze, smog and dust tied to volcanism, impacts, fire, chemistry and the built layer; climate effect of large veils | W2, S7, S12, L1c | a scripted eruption produces a plume of the height the rule gives, an ash streak downwind and a dated cooling; smog appears only over industry and respects player data views |
| W5 | Hooks: cloud and atmosphere passes, descent hazards and telemetry, sound, GM pins and triggers | W2, R2, S5, N5 | the same storm is seen from orbit, met on descent and heard on the ground; a pinned storm survives regeneration |
| W6 | The swap, proven: a second, deliberately different block behind the same contract (a small shallow-water model on giants is the natural first) | W3, K5 | switching blocks on a saved campaign changes no climate fact beyond tolerance, keeps recorded systems, shows the difference, and switching back restores the original exactly |

W1 to W5 belong with the surfaces milestone and just after. W6 is the later proof and the door for real weather simulation.
