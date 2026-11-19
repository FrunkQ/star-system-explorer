# SSE4 planetary surface design (part of the physics suite)

Companion to `SSE4-design-handoff.md`, `SSE4-galaxy-design.md` and `SSE4-life-design.md`. Date 18 Nov 2026.

**Read with `SSE4-surfaces-and-rendering.md`** (the research report from a separate thread). The two were written in parallel and are reconciled here. This file owns *how the surface is generated from history*. The research report owns *rendering, hardware scaling and software choices*. Shared terms, used by both from now on: the kernel product is the **surface atlas**; a landing's data is the **descent scene**; a rendered view's style document is a **render recipe** made of **passes**; the kernel is exact down to the **fact floor** and anything finer is decoration.

Goal **[Alex, 18 Nov]**: every world gets a detailed, deterministic surface built from its own history: plates, volcanoes, impacts, weather wearing it down, and the stranger mechanisms proposed in research. One height map per world is the **single data source** for every surface picture, from a line of landing telemetry to a photoreal descent. It resolves when a world is surveyed. People can land on it and walk it.

Same rules as everything else: no heavy simulation, formulas and small stepped models only; deterministic; a cloud until looked at; every feature can answer "why is this here?"; published methods are named for builders to check against their sources.

---

## 1. One data source: the surface model

```
SurfaceModel(world, time) =
  L1  tectonic sketch      coarse global structure with a history        (computed once per world, cheap, stored)
  L2  height function      h(lat, lon, time) at any resolution           (evaluated on demand, never stored whole)
  L3  cover                rock, regolith, ice, liquid, vegetation, colour (from climate and life)
  L4  features             named, dated, addressable things              (ranges, volcanoes, craters, rivers, seas)
  L5  built layer          settlements, bases, ports, aerostats          (rules or GM)
```

Everything visual reads L2 to L5 through one tile service. No view has its own terrain. A value seen in a text readout ("landing zone altitude 2,140 m, slope 4°") is the same number the 3D view draws.

**Tiles (the surface atlas)**: the globe is a cube-sphere quadtree, addressed `/body:<path>/shell:surface@<epoch>/tile:<face>/<level>/<x>/<y>`. A tile's contents are a pure function of the world's address, the surface model and the tile key, so any device can produce the same tile, any tile can be built without its neighbours, and tiles can be sent as data to weak devices. A child tile must downsample to its parent within a stated tolerance. Height is stored in integer units against the world's reference radius. Layers per sample are listed in the research report, section 3.1 (elevation, surface age, material, flow and channels, liquid and ice cover, feature tags, slope and roughness, biome and structures).

**The fact floor**: the kernel produces tiles, bit-identical everywhere, down to about 100 m anywhere on demand and about 10 m inside a declared landing zone, base site or walked area (pack-configurable; tune by measurement). Everything finer (rocks, ripples, grass) is decoration made on the GPU from the same seeds. It may differ slightly between graphics cards and nothing in play depends on it: hazards, routes, slopes and explanations read kernel tiles only.

**Survey and disclosure** (answering "a height map generated on surveyed worlds, used by rendering": yes, exactly that):

| State | What exists | What players can be shown |
|---|---|---|
| Unexamined | the world's envelope, plus a cheap regime-based sketch so the globe looks right from orbit | colour and brightness from afar |
| Scanned | the sketch at global level (level 0: six faces of 512 × 512, about 20 km per sample on an Earth-sized world), detail withheld | albedo and rough relief |
| Surveyed | the full history run (L1) and the atlas resolved globally; refinement tiles on demand down to the fact floor; features named; cover and resources mapped | everything the data view allows |
| Landed / walked | tiles around the site resolved to the 10 m floor and recorded; finer ground is decoration | the ground under their feet |

The full tectonic and erosion history is a heavy block: capable devices run it (seconds to tens of seconds, estimated), weak devices receive finished tiles from the GM's machine, and the UI shows which happened. Resolution is itself a disclosure item in player data views (handoff 6.7): the GM's survey action literally sharpens the map. Resolving is a recorded event, so a landing site's facts are identical every time anyone returns.

Honest labelling: L1 is modelled from the planet's physics; relief down to the fact floor follows from it deterministically; ground finer than the floor is decoration.

---

## 2. L1: the tectonic sketch

### 2.1 Which kind of planet is this?

The interior model (heat flow from formation and radioactive elements, which come from the galaxy design's ledger; mass; water; surface temperature; tides) classifies the **tectonic regime** over time. Regime changes are chronicle events. Regimes are pack data; each has its own surface grammar.

| Regime | Seen or proposed for | Surface it writes |
|---|---|---|
| Mobile lid (plate tectonics) | Earth | drifting plates, mountain belts at collisions, ocean ridges and trenches, volcanic arcs, continents and ocean basins (two-level height distribution) |
| Stagnant lid | Mars, Mercury, the Moon | one global plate; giant shield volcanoes over fixed plumes; old cratered highlands; contraction scarps; rift systems |
| Episodic lid | proposed for Venus | long quiet, then global volcanic resurfacing; few craters, all young |
| Plume and squishy lid | Venus alternatives | coronae, plateaus, rift belts, no plate boundaries |
| Heat pipe | Io; proposed for early Earth and hot super-Earths | constant eruption, burial and sinking; lava plains, paterae, isolated block mountains |
| Ice shell tectonics | Europa, Enceladus, Ganymede | bands, double ridges, chaos terrain, fracture stripes, plumes; ice volcanism on Triton- and Pluto-like worlds |
| Tidal lock, hot | lava worlds | day-side magma ocean, night-side crust, a shore between |
| Ocean world | water-rich planets | sea floor relief only, perhaps under high-pressure ice; no land |
| Hydrocarbon cycle | Titan | dune seas, methane rivers and lakes, eroded icy highlands |
| Carbon-rich | proposed for high C/O systems | graphite and carbide crust, tar seas (labelled speculative) |
| Super-Earth, uncertain | research is divided | a dial: plates or stagnant lid |
| Gas and ice giants | no solid surface | section 6 |

### 2.2 The plate sketch (mobile-lid worlds)

A coarse kinematic model on the sphere, in the spirit of published procedural tectonics work (Cortial et al. 2019, "Procedural Tectonic Planets"; builders check the paper). It is moved geometry, not fluid dynamics.

- The sphere carries about 10,000 to 40,000 cells. Plates are groups of cells grown from keyed seed points. Each plate has a rotation pole and rate (keyed draws scaled by the planet's heat flow).
- History is stepped in coarse intervals (default 25 Myr) from the time plate tectonics starts to the time in question. Per step: plates rotate; boundaries are classed as spreading, colliding or sliding from relative motion; new ocean crust is stamped with its birth date at ridges; old ocean crust is consumed at trenches; continent meets continent and raises a mountain belt stamped with its date; arcs and back-arc basins form above trenches; continents rift where they stretch.
- Reorganisations (a plate splitting, a pole jumping, a supercontinent assembling or breaking) are keyed events at a rate tied to heat flow. They are in the chronicle and editable, like galaxy mergers.
- Mantle plumes are fixed points in the deep frame (keyed). Plates moving over them leave age-ordered volcano chains. Rare very large eruptions flood whole provinces (these are also extinction triggers for the life suite).
- Output per cell and time step: plate, crust kind (continental or oceanic), crust age, crust thickness, list of mountain-building episodes with dates, distance and kind of nearest boundary, volcanic history.

Cost: tens of thousands of cells times a few hundred steps. Trivial once, and stored. The plate positions at every step are kept, so the deep-time ruler shows continents drifting, and a surface can be asked for at any date.

Other regimes replace this box with their own sketch (plume fields and resurfacing events; shell fracture patterns from tidal stress fields; and so on) writing the same output shape.

### 2.3 Things every solid world gets

- **Impacts**: craters are drawn from the system's impact history (heavy early, declining, with spikes from the chronicle such as a stellar encounter's comet shower). Each has a date, size and position. Crater shape follows size and gravity (bowl, central peak, ringed basin). What survives depends on how often the surface is renewed: an Earth keeps few, a Mercury keeps them all. The largest basins are L1 features that shape everything after.
- **Global asymmetries**: a hemispheric split in crust thickness (as on Mars and the Moon) from a keyed giant impact or early convection pattern, where the formation history allows.
- **Tidal and rotational shape**: flattening, tidal bulge, and stress patterns for locked or fast-spinning worlds.

---

## 3. L2: the height function

Height at a point and time is a sum of terms, each a pure function. Any resolution can be asked for; coarse terms come from L1, fine terms from keyed noise whose character is set by the coarse terms.

1. **Floating level**: crust floats on the mantle. Continents ride high; ocean floor sinks as it ages (depth grows roughly with the square root of age, the classic plate-cooling result). This alone gives ridges, abyssal plains and continental shelves.
2. **Mountain belts**: for each mountain-building episode near the point, a ridge profile whose height depends on how hard and how recently the collision happened, lowered by erosion since (term 6). Young belts are high and sharp; ancient ones are worn stumps. Maximum height is capped by gravity and rock strength, so low-gravity worlds get taller mountains.
3. **Volcanic constructs**: shields, cones, calderas, plateaus, island chains, flood provinces. Size limited by gravity and by how long the source stayed under one spot (a stagnant lid grows giants).
4. **Rifts, trenches, faults, scarps**.
5. **Craters and basins** from 2.3, with ejecta, rims, central peaks and later infill.
6. **Wearing down**, applied as functions of age and climate rather than a fluid simulation:
   - *Rivers*: on a coarse grid, water is routed downhill from rainfall to find drainage basins, main rivers, lakes and deltas (a one-pass flow routing, done once per time step and stored as L1-level data). Valleys are then carved into the fine terrain along those lines with depth from the stream-power rule (erosion grows with drainage area and slope). Fine detail uses noise shaped to look river-cut where it is wet and old.
   - *Ice*: where the climate history had ice sheets, U-shaped valleys, fjords, scoured shields, moraines.
   - *Wind*: dune fields where there is sand and wind and little water; yardangs.
   - *Coast*: sea level is not fixed; it follows ocean volume and basin shape over time, so shelves flood and drain. Cliffs, beaches, reefs (if the life suite supplies builders).
   - *Mass wasting and smoothing*: slopes relax with age; airless worlds soften by micrometeorite gardening instead.
   - *Sediment*: what is worn off high ground fills basins and builds plains; the totals balance at the coarse grid.
7. **Fine texture**: keyed fractal detail down to metres, with roughness, direction and style chosen by terrain kind and age (fresh lava, old highland, dune sea, chaos ice).

Checks: the distribution of heights matches the regime (two-peaked for Earth-like, single-peaked for Venus-like, split for Mars-like); slopes and river networks obey their scaling laws; fixture worlds built from real inputs (Earth, Mars, Venus, Moon, Europa, Titan) are recognisably of their kind against real data statistics, not replicas.

---

## 4. L3 to L5: cover, features and the built layer

**Cover (L3)**: per tile, what the surface is made of and how it looks: rock type from crust and volcanic history; regolith depth; ice and snow from the climate bands and height (temperature falls with altitude); liquid bodies; wetness and rain shadow from wind bands crossing mountains (the number of wind bands follows from the rotation rate); vegetation and its colour from the life suite; resource deposits (ores at old arcs and rifts, buried carbon where life put it, ice in shadowed craters). Colour and albedo maps come from cover, so the globe from orbit and the ground underfoot agree.

**Features (L4)**: ranges, peaks, volcanoes, basins, craters, rifts, canyons, rivers, lakes, seas, ice caps, dune seas, plains become addressable features under the world (the address tree), each with a kind, extent, date, cause (links to chronicle events) and a generated name from the campaign's naming rules. **Names can be changed, exactly like a planet's** **[Alex, 18 Nov]**: click the name on the chart and type. A rename is an authored event with a date, so it never affects the feature's address, physics or history, and a feature can carry several names (what the first survey called it, what the colonists call it now, a name per culture or language from a universe pack). The generated name stays on record as the original and can be restored. The GM can also name things the generator left unnamed (a pass, a bay, a landing site), and can rename by audience, so a seat's map shows its own people's names.

Sifters pick out the remarkable ones (highest peak, deepest trench, a crater that is also a lake, a canyon visible from orbit).

**Built layer (L5)**: settlements, bases, ports, mines, roads. Placed by rules from a game-system or universe pack (near water, flat ground, resources, a port near the equator) or by the GM's hand. **Aerostats** float where the atmosphere suits them: the altitude band with tolerable pressure and temperature, drifting with that band's wind, so a Venus-like world's cloud city has a real altitude, a real day length as it circles, and real hazards below.

GM editing: pin or sculpt anything. Raise a range, drop a crater, flood a basin, place a base. Each is an authored event with a date; stamped impossible if it contradicts the physics. The model flows around pins the same way resolve honours pins everywhere else.

## 4A. Rock, strata, flourishes, living cover, landmarks and countries **[Alex, 18 Nov: not all day one, but the design must reach it]**

All of this is possible with the machinery already described, because it is the same idea again: a cloud at coarse scale that resolves into particular things when someone looks closer. It is staged after the basic surface (stages S9 to S13 below).

### 4A.1 Rock columns: what the ground is made of, in layers

Each coarse cell of the tectonic sketch already knows its environment at every time step (under a shallow sea, a deep basin, a desert, a river plain, an ice sheet, beside a volcano, inside a collision). That history *is* its rock record. From it the kernel keeps a **rock column envelope** per cell:

```
RockColumn { basement: kind,                       granite-like, basalt-like, ice, and pack-defined alien classes
             layers: [{ kind, thickness (distribution), laid_down: time range,
                        hardness, solubility, jointing, colour }],       oldest at the bottom
             structure: { tilt, fold wavelength and amplitude, fault density, metamorphic grade },
             cut_by: [intrusions, dykes, impact melt] }
```

- Layer kinds follow where the cell was: limestone from warm shallow seas (and from reef builders if the life suite supplies them), sandstone from deserts and rivers, shale from quiet deep water, salt from dried seas, coal-like beds where life was buried, lava flows and ash near volcanoes, glacial till, and so on.
- Structure follows what happened to it: flat far from any collision, tilted and folded near one (tighter the closer and the harder the collision), faulted in rifts, baked and recrystallised deep in mountain roots.
- It is a cloud: thicknesses and exact sequence are distributions until a tile is looked at, then a keyed draw fixes the stack for that tile (consistent with its neighbours through shared coarse values), recorded like any resolve.

### 4A.2 Different rocks wear differently

The wearing-down terms (section 3, term 6) take the hardness of whatever layer is exposed. That one change produces a great deal of recognisable scenery with no special cases:

- hard caps over soft beds: mesas, buttes, cliff-and-bench staircases down a canyon wall;
- tilted beds: long asymmetric ridges; folded beds: parallel ridge-and-valley country;
- soluble rock: sinkholes, caves, towers and disappearing rivers;
- soft clays: badlands; massive granite: domes and tors; cooled lava: columns; resistant headlands: cliffs, stacks.

**A canyon over millions of years**: an uplift event raises a plateau across a river; the river keeps its course and cuts down at a rate set by its flow, the slope and the layers it meets, fast through soft beds, slow through hard. The channel depth is tracked per time step at the coarse grid, so the time ruler shows the canyon deepening and widening, and the fine terrain shows the layered walls. This works at stage S9; a plain canyon without visible layering already works at S3.

### 4A.3 Flourishes: particular formations that resolve when you get close

A **landform grammar** in a pack: rules of the form *where this rock, this structure, this climate and this process have had this long, expect this many of that formation per square kilometre*. Examples: sea arches and stacks on jointed resistant coasts; natural bridges; hoodoos; balanced rocks; slot canyons; waterfalls where a hard bed crosses a river; cave mouths; dune forms; ice caves; lava tubes with skylights; crater-lake islands; alien classes from research and from fictional packs.

- Far away a flourish is only a density. On approach (inside a landing zone or walked area) it resolves "now" into an object: kind, position, size and shape numbers. That object is a kernel fact, identical for everyone, addressable, nameable, and able to answer why it is there. Its fine mesh is drawn from a parametric recipe.
- Flourishes can be short-lived. An arch has a collapse chance per century; over deep time unrecorded flourishes are redrawn per era, exactly as unsurveyed stars are (galaxy design 8.3). A recorded one persists, and its collapse can be drawn as an event or triggered by the GM for the story.

### 4A.4 Living cover

Plants and microbes are shown; animals are not **[decided]**. The life suite's producer guilds (and mat- and crust-forming microbes) become cover: where each grows from climate, soil, water and light; density and height; colour from their traits. Up close they are drawn from parametric plant-form recipes driven by those traits (frond, trunk, mat, crust, the tallest height gravity allows), as decoration. Things life builds that are landforms (reefs, mounds, peat, stromatolite-like terraces) are features in their own right.

### 4A.5 Tech landmarks

The built layer (L5) gains a landmark library, also parametric recipes in packs: settlements with a footprint and later a street plan, ports and pads, towers, mines, dams, roads and rail, farms and domes, beacons, wrecks, the ground anchor of a space elevator, terraforming works. Each is an addressable, nameable feature placed by rules or by hand, with a date. **Ruins** fall out for free: a landmark from an earlier civilisation in the chronicle is the same object with age applied, weathered and buried by the same wearing-down terms. A GM's own images or models can be attached to a landmark as media.

### 4A.6 Countries and political maps

Polities are a region layer over the surface: areas with an owner, borders, provinces, claims, capitals, each with dates so borders move on the time ruler. They can be drawn by hand on the chart, or grown by rule from capitals outward over travel cost so that borders settle on rivers, ranges and coasts. Roads follow least-cost paths between settlements. With the Political lens (handoff 8.6) and seats per polity (8.7), this is the "RPG country app": a continent or a single kingdom charted at the 100 m fact floor, with real terrain reasons behind its borders, towns, passes and resources. It needs nothing new in the architecture; a pack could even open straight onto one world's surface and never mention space.

---

## 5. Time

A surface exists at a date. Scrubbing the ruler: plates drift (stored steps, interpolated), mountains rise and wear, seas open and close, ice advances, craters accumulate, sea level moves, life greens the land. The surface at a date is L1 at that step plus the height function evaluated with ages measured to that date.

Surveyed detail is recorded at its date. Moving a campaign a million years leaves landforms in place; moving it a hundred million gives a new geography with the old one as its recorded past (fossil mountain roots, buried craters).

---

## 6. Worlds without a solid surface

- **Weather, clouds, smoke and haze** on every kind of world are specified in the companion `SSE4-weather-design.md` (19 Nov), which extends the cloud deck model below and supplies the weather that the hazard layer and descent scene read.
- **Gas and ice giants**: the "surface" is a cloud deck model: band count and widths from rotation and heat, zonal wind speeds, long-lived keyed storms that drift and merge, colour from cloud chemistry by temperature. Depth layers by pressure give the aerostat band. The tile service serves cloud-top height and colour so the same views work.
- **Ice-shell ocean worlds**: the shell is the surface (section 2 regimes); the ocean floor is a second surface model below.
- **Lava worlds, ocean worlds**: as their regimes.
- **Small bodies**: shape models from keyed lumps and craters, not spheres.

---

## 7. Landing and being there

A **landing** is a NAV product (handoff 6.8), the **descent scene** (research report section 4): from orbit, site and vehicle, the kernel computes the descent: entry speed, heating, deceleration, whether parachutes work in this air or engines must do it all, winds, final approach, slope and roughness at the site, hazards, a difficulty with reasons. It is a timed sequence of events plus a path over the real terrain tiles.

Every view renders that same sequence and the same tiles. The theme decides the look:

| Rendering | What it is | Device tier |
|---|---|---|
| Telemetry lines | timed text: "T−00:42 altitude 3.1 km, descent 41 m/s, crosswind 27 km/h WNW, thrusters 42%" | any |
| Line terrain | the style of Alex's reference clip: ridgelines of the real terrain ahead drawn as stacked profile lines, a grid globe with the landing zone marked in the approach, status column and countdown | canvas |
| Wireframe / contour 3D | mesh or contour terrain, hologram or green-phosphor theme | basic GPU |
| Chart | a topographic chart with the track inked on it (GM default) | any |
| Photoreal | lit terrain with real sun angle, atmosphere scattering in this air's colour, clouds, cover textures, the right sky (handoff section 10) | high GPU |

The reference clip (ridgeline terrain, orbit grid with red landing zones, status sidebar reading landing zone coordinates, air density, wind, hull and heat shield state, a countdown) is reproducible exactly as a screen made of the Surface view in line-terrain style plus Readout and Text views, driven by a landing sequence. Nothing in it is decoration: every number would come from the kernel.

**On the ground**: the Surface view at eye height, moved by intents ("walk to the ridge", "drive to the base"), with the true horizon distance for the planet's size, true sky, true sun or suns, weather from the climate model. A remote probe is the same thing with a camera mast and a signal delay. Players command; nobody steers a physics-driven vehicle by hand.

New and changed views for the stack: **Surface** (terrain from orbit to eye height, with styles line, wireframe, contour, shaded, photoreal), **Terrain chart** (topographic map and cross-sections; part of Chart), and Body (globe) now reads the same tiles.

### 7.1 Not five renderings: a pipeline of styles that Orac can evolve **[Alex, 18 Nov]**

The table above is only examples. The Surface view (and in time every view) is a **render pipeline** whose look is a declarative **style document**, called a **render recipe** for rendered views: a graph of passes from a finite, tested pass library, each pass with its own parameter schema, tiers and cost model (research report section 5, which is the authority on the mechanics: degrade order, the frame-time governor, the quality badge, Orac's preview loop, and the rule that shared recipes are data-only unless the user allows code). A style is data, so there can be any number of them, they travel in presentation packs, and Orac can create and refine them.

```
StyleDoc { id, label, extends?: StyleId, tiers: [...],
  source:    which tiles, levels, time, sequence, overlays of data (hazard, slope, cover, knowledge)
  geometry:  how terrain becomes shapes: mesh | profile-lines | contours | points | voxels | hachures | text-grid | none
             plus its parameters (line spacing, count, exaggeration, simplification, draw distance, culling of hidden lines)
  marks:     how features, sites, paths, vehicles and labels are drawn
  shading:   flat | by-height | by-slope | by-cover | lit (sun, sky, atmosphere) | emissive wire, with palettes from theme tokens
  camera:    orbit | chase | cockpit | top-down | cinematic rails keyed to sequence events; shake, lag, field of view
  staging:   what happens at each sequence event (cut to grid globe, show tunnel frames, flash a warning, start countdown)
  text:      which readouts, wording layer, cadence
  post:      filter chain (scanlines, glow, grain, chromatic split, fog, dither, posterise)
  sound?:    cue names
  params:    every number above, each with a name, range, default, unit and one-line meaning }
```

Design rules:

- **Lots of hanging configuration, on purpose.** Each stage exposes many parameters, far more than a person would want to face. They are described in a machine-readable schema with ranges and plain meanings so that Orac can reason about them: "make the ridgelines denser near the landing zone and add a slow roll during the grid-globe phase" becomes a small diff to a style document.
- **People get a short list.** A style *promotes* a handful of its parameters as friendly sliders ("line density", "glow", "camera drama"). Everything else is reachable through Orac or the raw document.
- **Styles inherit.** "Green phosphor landing" extends "line terrain" and changes twenty values. A universe pack's bridge look is a family of styles sharing a base.
- **Evolving a look**: Orac proposes variants side by side from one sentence, the GM picks, Orac refines. Every variant is a validated document (schema, tier limits, cost class), so nothing Orac writes can break a session or exceed a device.
- **Styles never touch data.** Same tiles, same sequence, same numbers. A style that hides a hazard is a presentation choice the GM made, not a different truth.
- **Stages are plug points.** New geometry kinds, shaders and filters are view-side modules registered by id; a style refers to them by id. Adding one does not touch the kernel.
- The same style mechanism should be adopted by the other views in the stack (System 3D, Sky, Map), so the handoff's "skins" become: theme tokens plus per-view style documents.

---

## 8. NASA and ESA open tools, Space Engine, and what the browser can do

Answered in depth, with sources, in `SSE4-surfaces-and-rendering.md` sections 1, 2, 6 and 7. That report is the authority. In brief, and replacing my earlier first view:

- The NASA and ESA open tools cannot be embedded (OpenSpace and Gaia Sky are desktop programs) and the browser globes (WebWorldWind, CesiumJS) show measured tiles; none invents a world. What is usable is the layer beneath: published techniques, permissively licensed browser implementations, and public elevation data.
- **Renderer base: three.js `WebGPURenderer`** with its own planet layer and an open atmosphere implementation (Hillaire's method), falling back to WebGL2. Not CesiumJS (this withdraws my earlier suggestion to evaluate it).
- Expect near parity with Space Engine in **stills** on any GPU device through a progressive `still` tier with a "take photograph" action, close to it in real time on WebGPU desktops, and stylised views at full quality on phones.
- Real measured terrain is used for real bodies in the Milky Way pack, as pins, with procedural fill below its resolution.
- Still worth doing from my first view: export the atlas in standard open formats (height map pyramids, GeoTIFF, 3D Tiles) so desktop tools and mapping software can open an SSE world.
- Where SSE4 wins: every landform has a date and a reason, surfaces change through time, and one data source drives text, charts, retro displays and photoreal.

---

## 9. Stages and checks (surface and rendering tracks, merged with the research report's section 9)

| Stage | Delivers | Needs | Verified by |
|---|---|---|---|
| S0 | Surface envelope on every world; regime classification over time; cheap regime sketch for unexamined and scanned worlds | P1, P4 | fixture worlds classed correctly; regime change events have causes; sketch globe is statistically of its kind |
| S1 | Surface atlas: cube-sphere addressing, tile layers, parent-child consistency, caching, disclosure by resolution | K2, K9 | any tile builds alone with identical hashes on all harness browsers; children downsample to parents within tolerance |
| S2 | Tectonic regimes and the plate sketch (section 2), uplift histories, drift on the time ruler | P1, S1, T1 | Earth-like fixture forms continents, ridges, arcs with two-peaked heights; stagnant-lid fixture forms none; crust ages symmetric about ridges; replays bit-identically |
| S3 | Wearing down (section 3 term 6): stream-power rivers with climate-driven strength, ice, wind, coast, sea level through time, sediment balance | S2 | drainage networks dendritic and obeying scaling laws; sediment balance closes at the coarse grid; no rivers uphill in 1,000 random worlds; an old dried world keeps softened valleys |
| S4 | Impacts and volcanism from the chronicle, crater degradation by age, refinement to the fact floor, slots for alien-mechanism atoms | S3 | crater density against surface age matches the chronicle; every feature answers "why is this here" |
| S5 | Hazard layer (slope, roughness, weather) and the descent scene from NAV | S4, T3 | descent physics against known cases (thin-air Mars landing needs engines; Titan parachute descent is long); the reference clip's content reproduced from a fixture |
| S6 | Other regimes: ice shells, episodic and plume lids, heat pipe, lava, hydrocarbon cycle, giants' cloud decks, small bodies | S2 | one fixture per regime, statistically of its kind |
| S7 | Cover, named features with causes, resources, built-layer rules, aerostats; GM reshape tools as authored events | S4, L1a | cover agrees between globe and ground; aerostat band matches the atmosphere profile; the terrain honours pins |
| S8 | Standard-format export; Milky Way pack with measured terrain as pins | S1, G7 | export opens in an external open tool; measured worlds match their data to its resolution |
| R1 | Pass library, render recipes, degrade order, governor, quality badge (the rendering form of handoff 8.9) | V1, V11 | a recipe runs on every tier it declares; governor holds its budget on a named low-end phone; invalid recipe rejected |
| R2 | Body and descent views on basic and high GPU tiers, atmosphere, clouds; terrain chart and the line-terrain, contour, wireframe and shaded recipes | R1, S1 | visual fixtures for an Earth-like, a Mars-like and a Titan-like world; the reference-clip screen rebuilt as a recipe |
| R3 | `still` tier and "take photograph" to Media | R2 | a still on a phone in under 10 s (target) |
| R4 | Orac recipe editing with preview frames; walking and probe intents on the surface | R1, A2 | Orac adjusts a recipe from a written request within the device budget; a recipe change never alters the state hash |
| S9 | Rock columns from each cell's history; hardness-aware wearing down; canyon incision through layers on the time ruler | S3 | hard-over-soft fixture forms mesas and benches; tilted-bed fixture forms asymmetric ridges; soluble fixture forms sinkholes; a scripted uplift-and-river run cuts a layered canyon and replays from its manifest |
| S10 | Landform grammar and flourishes resolving on approach; lifetimes and collapse | S9, S4 | densities match the rules over large samples; a resolved flourish is identical on every device and every return; every flourish answers why |
| S11 | Living cover from producer and microbial guilds; plant-form recipes; life-built landforms | S7, L1c | cover follows climate and traits; no cover where the energy budget is zero; no animals rendered |
| S12 | Landmark library, placement rules, ruins by ageing | S7 | a landmark dated in the past appears weathered by the same erosion terms; attached media round-trips |
| S13 | Polities, borders through time, rule-grown borders, least-cost roads; a surface-first pack entry point | S7, U9, V10 | grown borders follow terrain cost on fixtures; border history scrubs on the ruler; a pack can open directly on a surface chart |

S9 to S13 are later stages, not part of the first surfaces milestone.

S0 to S3 belong before the life suite's later stages, because habitats, nutrients (exposed rock) and catastrophes depend on them.
