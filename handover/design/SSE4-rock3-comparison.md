# SSE4 surfaces against Rock 3: comparison and where to go further

Analysis for Alex, 21 Nov 2026. Companion to `SSE4-surface-design.md` (authority on how surfaces come from history), `SSE4-surfaces-and-rendering.md` (rendering and hardware) and `SSE4-weather-design.md`. Status: **accepted 21 Nov 2026** (Alex: "agree with all proposals in the current geology doc. tweak after review - but looks solid"). Every recommendation in section 7 is a ruling, and the stages in section 8 are in the build plan. The exogeology research (thread "Research exogeology features we're missing") will add a section here; until it lands, its additions are proposals.

Sources: Rock 3's feature list as posted on Reddit (pasted by Alex), its Steam page (Insight Games, free, released 13 Nov 2022, 88% positive of 127 reviews), and SSE4's built packages S0 to S7, W1 to W6 (`handover/packages/`). Rock 3's source is not public, so its methods are read from its own description only. References marked *(check)* are named for builders to verify against the source before use, as the citations rule requires.

---

## 1. Short answers

- **Are we missing anything Rock 3 has?** A few things, all fixable: transform-fault relief, continental rifts, a land and sea temperature contrast (coasts moderated by the ocean), a finer tectonic grid (Rock 3 92,160 cells, ours 24,576), biome maps, map export, and above all the *feel*: Rock 3 regenerates a world live as you move a slider. Section 3.
- **Where are we already ahead?** Time (every landform has a date and the surface exists at any date), the other twelve kinds of world, impact science, rivers by stream power with a closing sediment balance, determinism across devices, "why is this here?" on every feature, named and renameable features, deposits, settlements, landings, and detail down to 10 m tiles instead of one 2048 × 1024 (or GPU-limited 8K) image. Section 2.
- **Can we go further than Rock 3 in a browser?** Yes. Nothing Alex asked for (strata, wear times, lava tubes, caves) needs heavy simulation. They are bookkeeping on the plate grid we already step, plus objects that resolve only where someone looks. The one real constraint is self-imposed: facts must be bit-identical on every device, so the GPU cannot compute them. The answer is a fast coarse preview plus the full run in a worker, not the GPU. Section 6.
- **Skipped worlds must still look good at once (Alex's requirement).** Detail levels (5.8): a coarse but real history in under a second for any world on screen, the full run only on survey, rock layers and caves only at sites. The coarse world pins the detailed one, so they are the same world.
- **Is Rock 3's approach worth copying?** Its geology list is a good checklist and its live sliders are the right experience. Its method (GPU, one snapshot, Earth-like only, no history) is not what SSE4 is.

---

## 2. Feature by feature

Legend: **built** (in `main` with tests), **designed** (in a design doc, stage named), **gap** (neither), **beyond** (SSE4 does more than Rock 3 claims).

### 2.1 Geology

| Rock 3 | SSE4 today | Verdict |
|---|---|---|
| Variable density plates for plateaus | Continental and oceanic crust, Airy thickening at collisions, crust age and sea-floor cooling (GDH1) (S2) | built, partly. Plateaus come only from thickening. No buoyant oceanic plateaus (Ontong Java), no delamination uplift (Tibet, Altiplano). Proposal 5.1 |
| Subduction uplift: long ranges along shorelines | Trench plus volcanic arc on the overriding plate; arcs thicken ocean crust into continent (S2) | built |
| Continental convergence: creased, wide ranges | Dated mountain-building episodes, worn by their own e-folding time, height capped by gravity and rock strength (S2) | built, but belts are one or two 156 km cells wide; the creases come from noise, not the collision. Proposal 5.1 |
| Transform boundaries: perpendicular rippling ridges | Sliding boundaries are classified, but write no relief | **gap**. Proposal 5.1 |
| All other boundary pairs (convergence, divergence, transform) | Ridges, trenches, arcs, sutures, plate splits. No continental rift valleys, failed rifts, passive margins or back-arc basins (S2 known limits) | partly. Proposal 5.1 |
| Hot spots for island chains | 8 fixed plumes leaving age-ordered chains, plus flood basalts at 2 per Gyr (S2) | built, **beyond** (flood basalts feed the life suite's extinctions) |
| Meteor craters you can add | Craters from the system's own impact history: Neukum chronology, gravitational focusing, saturation, survival by renewal, degradation by age, bowl to multi-ring shapes, crater-count ages (S4); GM "drop crater" as a dated event (S7) | built, **far beyond** |
| (nothing on erosion beyond "erosion simulation" on Steam) | Priority-flood routing, stream-power incision (Braun and Willett), sediment routed and balanced exactly, hillslope diffusion, glacial factor, dune flags, sea level refitted through time; Hack's law checked (S3) | built, likely beyond. Rock 3's rivers are not mentioned anywhere |
| Earth-like only | 14 regimes: stagnant lid, episodic lid, heat pipe, ice shell, lava world, ocean world, Titan cycle, carbon world, giants' cloud decks and more (S0, S6) | **beyond** |
| One moment | The surface exists at any date; plates drift on the time ruler; regimes change with dated causes | **beyond** |

### 2.2 Climate

| Rock 3 | SSE4 today | Verdict |
|---|---|---|
| Sunlight absorbed by ocean versus land albedo | Energy balance by latitude (North 1975) with transport from air pressure and gravity; albedo by cover, but temperature does not yet use the local albedo | **gap** (small). Proposal 5.4 |
| Coastlines moderated and cooled by oceans | Annual zonal means; no continentality (S7, W2 known limits) | **gap**. Proposal 5.4 |
| Altitude and air density modulate heat | Lapse rate g/c_p, reduced on wet worlds; none below 0.01 bar (S7) | built |
| Hadley, Ferrel and polar cells | Cell count *from rotation* (Earth 3, Mars 2, Venus and Titan 1), wet rising and dry sinking branches (S7, W1) | built, **beyond** (Rock 3's three cells look fixed) |
| Trade winds, westerlies, easterlies carrying rain | Prevailing wind per band carries moisture inland (S7); seasonal rain belt (W1) | built |
| Windward rain, leeward rain shadow | exp(−barrier upwind / 1500 m) (S7) | built |
| Wind set by spin direction and speed | Speed sets the cells. Spin *direction* (retrograde worlds) not checked | check. Proposal 5.4 |
| Axial tilt | Seasons move the rain belt, storms, frost (W1, W2); the S7 cover climate is still annual mean | partly |
| (not in Rock 3) | Tropical cyclones, dust storms, polar vortices, giants' storms, ash and smog, a swappable weather block (W1 to W6) | **beyond** |

### 2.3 Planet, output and use

| Rock 3 | SSE4 today | Verdict |
|---|---|---|
| Up to 92,160 hexagonal cells | 24,576 tectonic cells, then tiles to 100 m anywhere and 10 m at sites (S1) | finer output, **coarser history grid**. Proposal 5.1 |
| Real time, regenerate as you drag | Seconds per world natively; the full chain is a heavy block | **gap in feel**. Proposal 6.2 |
| Toggle atmosphere and ocean rendering | Render recipes and passes on the Body globe (R1, R2a) | built |
| Spherical or flat projection, choose the centre | Globe built; flat terrain chart designed (V-track "Terrain chart") | designed |
| Save elevation, heat, water, landmass, biome maps (2048 × 1024 per Reddit, up to 8K per Steam) | Export designed as S8 (height pyramids, GeoTIFF, 3D Tiles), not built | **gap**. Proposal 5.6 |
| Biome map | Cover (ice, water, rock, colour) built; biomes wait on the life suite's guilds (S11) | **gap** for now |
| Dedicated GPU required, Windows | Any browser, phones included; weak devices take finished tiles from the GM | **beyond** |

---

## 3. What Rock 3 does better, honestly

1. **Interactivity.** Change tilt, watch rain shadows move. That instant loop is why people love it. SSE4 has the physics but not yet the toy.
2. **Tectonic grid density.** Four times our cells, so ranges and coasts carry structure that ours fake with noise.
3. **Boundary variety.** Transform ripples and the full boundary matrix give it more kinds of mountain country.
4. **Coastal climate.** Land and sea heating differently is the cheapest big win for believable climate maps.
5. **A product you can take away.** One button, six maps.

None of these is hard for us. Points 2 to 5 are pack numbers and one new term each; point 1 is a way of running (6.2).

---

## 4. What SSE4 can do that a snapshot generator cannot

The thing that makes Alex's three directions possible is that **every coarse cell already has a dated life story**: when it was sea floor, when it was in a collision, how much sediment arrived each 100 Myr, when it was glaciated, when lava covered it. Rock 3 makes a shape. We have a history, and strata, wear times and caves are all just *reading that history downwards*.

---

## 5. Proposals

### 5.1 Tectonics: catch up and pass (package S2b)

- **Grid to 128 × 128 a face (98,304 cells, 78 km on an Earth)**, above Rock 3's count. Cost is about 4× the present run, still seconds in a worker. It must be one fixed number in the manifest so every peer agrees.
- **Transform faults and fracture zones**: on sliding boundaries, offset ridge segments, linear fracture-zone ridges and troughs on the sea floor, pull-apart basins at releasing bends and pressure ridges at restraining bends (signs from the relative velocity already computed). *(check: Gerya 2012, Tectonophysics, transform fault origin)*
- **Continental rifts**: where a split runs through continent, a rift valley with flank uplift, then either a new ocean with passive margins on both sides or a failed rift (an aulacogen) if the split stops. Rift lakes and flood basalts follow. This also makes the best salt and limestone basins for strata (5.2).
- **Back-arc basins** behind steep, old subduction.
- **Plateaus**: buoyant oceanic plateaus that resist subduction and accrete as terranes; plateau uplift after a thickened root delaminates.
- **Erosional rebound**: removing rock lets the crust float up (isostasy), so worn ranges keep peaks while their valleys deepen. *(check: Molnar and England 1990, Nature)*. One line in the S3 step using the S2 crust thickness.
- **Fix the known flaw**: ocean crust lives about 280 Myr against Earth's 60 Myr. Retune plate speeds and subduction so the mean falls within a factor of two; that gives more trenches, more arcs, more mountains.
- **Sub-cell boundary lines**: store each boundary as a polyline inside its cells, so a range can be 20 km wide and sharp even on a 78 km grid, and fine ranges follow the collision's own line instead of noise.

### 5.2 Rock types and strata (package S9, already designed, here made concrete)

The design's `RockColumn` stays. How to fill it from what we already compute:

1. **Each 25 Myr step, each cell gets a depositional environment** from state we already have: water depth (sea level versus height), distance to coast, sediment arriving (S3's routed flux), climate band and wetness (S7, W1), volcanic activity nearby (S2 arcs and plumes, S4 plains), ice cover, and life if the life suite says so.
2. **Environment gives rock kind**, as pack data: warm shallow sea and little mud → limestone (more if reef builders exist); quiet deep water → shale; river plain → sandstone and mudstone; desert → dune sandstone; closed basin drying out → salt and gypsum; buried swamp with life → coal; arcs → lava and ash; ice → till; deep ocean → chert and ooze. Lateral shifts between environments stack up in time exactly as real facies do (Walther's law), so coastlines advancing and retreating write the familiar sandwich of layers.
3. **Thickness is the deposited volume**, which S3 already balances to rounding. Nothing is invented: a layer is as thick as the sediment that actually arrived.
4. **Erosion removes from the top**, layer by layer, at a rate set by the exposed layer's hardness (5.3), so an unconformity (a gap in the record) appears wherever a cell was uplifted and worn. These gaps are dated and can be explained.
5. **Structure from events**: tilt and folding from each collision episode by distance and strength; faulting in rifts; metamorphic grade from burial depth and heat (S0 heat flow), so mountain roots come up as schist and gneiss when worn deep enough.
6. **Non-Earth worlds use the same machine with their own tables**: Mars (sulphate and clay layers, polar layered deposits), Titan (organic sands over water-ice bedrock), icy moons (ice of different salt content and age), lava worlds (lava on lava).

**Storage**: merge adjacent layers of the same kind; at about 40 layers a cell × 98,304 cells × 8 bytes this is about 30 MB at worst before compression, computed once per world and cached. Fine on a desktop; a phone takes tiles that already carry the exposed layer.

**What the GM and players get** (each with a view, as the blocks rule asks):
- A **geological map layer**: which rock is at the surface. Outcrop bands, cuestas and hogbacks then appear on their own where tilted hard and soft beds meet the topography.
- A **cross-section tool**: draw a line on the chart, see the layers underneath with their dates, like a road cut. This is the education payoff: the Grand Canyon story told by the world itself.
- **Drill a core** at any point: the column with "laid down 312 to 290 Myr ago in a warm shallow sea".
- Strata feed the life suite (fossil-rich layers, buried carbon) and resources (oil traps under salt, coal, placer gold).

### 5.3 Wear times: different rocks, different clocks (packages S9 and S10)

Alex's "wear times to create interesting features" is the strongest idea in the brief, and it falls out of 5.2 with one change: **every erosion and weathering rate takes the exposed layer's properties**. Order-of-magnitude rates to start from (all *check* before use; pack numbers):

| Process | Rate scale | Feature it makes |
|---|---|---|
| River incision, stream-power K varies with rock by about four orders of magnitude | *(check: Stock and Montgomery 1999, JGR)* | hard caps over soft beds give mesas, buttes and canyon staircases |
| Waterfall (knickpoint) retreat upstream | metres per year to mm per year by rock | waterfalls that migrate upriver over the time ruler, gorges behind them *(check: Crosby and Whipple 2006, Geomorphology)* |
| Limestone dissolution | tens of mm per thousand years, more with soil CO₂ and runoff | karst plains, towers, sinkholes, caves (5.5) |
| Sea-cliff retreat | cm to m per year by rock and wave energy | headlands, arches, stacks, wave-cut platforms |
| Frost shattering | where temperature crosses freezing often (seasons from W1) | talus slopes, tors, block fields |
| Wind abrasion | dry, sandy, windy cells | yardangs, ventifacts, pedestal rocks |
| Chemical weathering | rises with temperature and runoff | deep red soils in the tropics, thin soils at the poles; nutrients for life |
| Micrometeorite gardening (airless) | already in S4 | softened craters, regolith depth |

Two things this gives that no snapshot generator can:
- **Features have lifespans you can watch.** An arch forms when a fin of jointed hard rock is cut from both sides and collapses with a chance per century that grows as it thins. A waterfall walks upstream. A mesa shrinks to a butte, then a pinnacle, then nothing. On the deep-time ruler the land is visibly alive. *(check: Bruthans et al. 2014, Nature Geoscience, on stress-controlled sandstone arches)*
- **Age is readable from the land.** A GM can say "the Pinnacles have stood for 2 Myr and will fall within 0.5 Myr", and it is true in the model.

These features stay a cloud (a density per cell) until someone looks closer, then resolve as flourishes (S10), exactly as designed.

### 5.4 Climate catch-up (to the weather thread, small)

- **Land and sea**: temperature at each cell relaxes toward the zonal value with a heat capacity by surface (ocean high, land low) and a coastal mixing length, so coasts are mild and interiors extreme, both annual mean and seasonal range. That one term gives continental deserts, mild west coasts and harsh interiors.
- **Local albedo in the balance** (ice, desert, forest), with the ice-albedo feedback that the energy-balance model already supports.
- **Ocean currents as rules**: western boundary currents warm east coasts, eastern boundary upwelling cools west coasts and makes coastal deserts and the marine decks W2 already draws (its known limit is that they sit over any subtropical sea). Gyres follow from basin shape and the wind bands we have.
- **Retrograde spin** flips the wind directions; add a test.
- **Ice ages over deep time** from orbit and tilt cycles, writing moraines, sea-level steps and raised beaches into S3.

Rock 3 has the first two. None of them has currents; we would.

### 5.5 Caves and voids: what makes holes (new package S14)

Underground was "later, until the renderer reaches ground level". These mechanisms can be put into the kernel now as envelopes, so that when the renderer gets there the facts already exist. Each is a rule block with a citation and a "why".

| Mechanism | Needs (from what we have) | Where and how big | Worlds |
|---|---|---|---|
| **Lava tubes** | S4 shields and lava plains, S2 plumes and arcs; low-viscosity lava on gentle slopes; flow paths by S3's own downhill routing from each vent | Tubes follow the steepest path away from vents; width scales up as gravity falls, so lunar and Martian tubes can be far wider than Earth's (km scale proposed for the Moon); roof collapse makes skylights, then pit chains, then a sinuous rille when the roof is gone. Preservation: billions of years on airless worlds, filled and eroded on wet ones | Moon, Mars, Earth, Io, any lid world; cryolava tubes on icy moons labelled speculative |
| **Solution caves (karst)** | Limestone, dolomite, gypsum or salt layers from 5.2; a water table; CO₂ in the soil (more where life lives) | Passages form at the water table; when the river cuts down or the sea falls, a new level forms below, so **multi-level caves record the history of uplift and sea level** (dated, explainable) | wet worlds with carbonate or evaporite strata |
| **Acid caves** | Hydrogen sulphide rising from hydrocarbons or volcanoes into limestone | large rooms rather than passages | Earth-like; volcanic carbonate worlds |
| **Titan karst** | organic sediment layers dissolving in methane rain | labyrinth terrain, sinkhole lakes *(check: Cornet et al. 2015, JGR Planets; Malaska et al. 2020, Icarus)* | Titan-like |
| **Ice caves** | glaciers with meltwater (S3 glaciated cells, W1 seasons); sublimation on thin-air worlds | glacier caves (short-lived, seasonal); sublimation pits | Earth, Mars, Pluto-like |
| **Water lenses in ice shells** | S6 chaos terrain, tidal heat | liquid pockets a few km down under chaos terrain *(check: Schmidt et al. 2011, Nature)* | Europa-like |
| **Sea caves** | jointed resistant coast plus wave energy (5.3) | short caves at sea level, arches when two meet | worlds with seas |
| **Tectonic fissures and talus caves** | rifts and fault density from 5.1 and 5.2; big rockfalls | cracks and boulder caves | any solid world |

**How a cave exists in the model** (same cloud-then-resolve rule as everything else):
- **Far away**: per cell, an envelope: expected passage length per km², depth band, typical width, levels, what made them. This is cheap and is what sifters read ("a moon with lava tubes wider than 300 m within 50 km of the equator").
- **On approach** (site or walked area): a keyed draw resolves a **passage graph**: nodes and segments with widths, heights, floor depth, entrances where the graph meets the surface (a skylight, a cave mouth, a sinkhole), all kernel facts, identical for everyone and nameable. A lava tube's graph follows its flow path; a karst graph is a branching network along bedding and joints at each level.
- **Below the fact floor**: wall texture, rubble, formations (stalactites on wet worlds, lava stalactites in tubes) are GPU decoration from the same seeds.
- **Habitats**: the life design already lists "caves and lava tubes" as a habitat kind; this gives it real places.
- **For play**: shelter from radiation and micrometeorites (a key Moon and Mars base site), hazards (collapse, flooding, bad air), exploration with a real map.

**Views**: first a **cave survey map** (the plan-and-profile drawing cavers make), which runs on the canvas tier on every device. Then the 3D walk-through from a signed-distance volume built only for the walked area, when the renderer reaches the ground. The Cutaway view gains a void band.

### 5.6 Map export (move part of S8 forward)

Equirectangular PNGs per layer (height, temperature, rain, land and sea, cover, geology, later biome), 2048 × 1024 by default and up to 8192 × 4096 on desktops, generated from level tiles. Matches Rock 3's one button, and is useful for VTT maps and for the Explorers library. GeoTIFF and 3D Tiles later, as designed.

### 5.7 Where the impactor came from (new package S15) **[Alex, 21 Nov: yes to provenance, with real physics behind what it leaves]**

Today S4 makes each crater from a statistical rain: it has a date, size, position and the chronicle event that caused it (a bombardment spike, a comet shower), but not *what* hit. The formation run already knows much more than that, so each crater can carry its impactor's source, and the source sets real consequences on the ground.

**The sources already exist in the kernel.** Nothing is invented for the purpose:

| Source population | Where it comes from in SSE4 | What it is made of | Typical arrival |
|---|---|---|---|
| Inner belt, stony | F3 rings and F5 `belt` inside the snow line | rock and metal in the belt's element-group books | slow: the escape speed plus a few km/s |
| Outer belt, carbonaceous | F5 belts beyond the snow line; P1's `planetesimal.water` hook (later chondrites wet) | hydrated rock, organics, water | slow to moderate |
| **Debris of a destroyed world** | F4 collisions: shattering and erosion (Leinhardt and Stewart 2012) with **mantle stripped first** (Marcus 2010) leave a `debris-belt` dated to the collision | metal-rich if the core was exposed: iron, nickel, platinum-group metals; a core-mantle mix (like pallasites) | moderate; a dated spike that decays as the belt grinds down |
| A broken moon | F4 impact moons, F7 tides, or a GM edit, where a moon falls inside its planet's breakup distance | that moon's own make-up | ring then rain, mostly on its own planet |
| Trojans, scattered disc, Oort comets | F4 and F5 reservoirs, kicked inward by F4 giant sweeps and G6 stellar passages (comet showers) | ices, organics, dust | fast: comets several times belt speeds |
| **Interstellar visitors** | G5 and G6: the local star density and speeds give a rate; each visitor is drawn from a neighbouring system's own inheritance | another star's chemistry: its own metals and its own r-process share (U, Th) | very fast, on hyperbolic paths |
| Its own ejecta, or a neighbour's | S4 large impacts throwing material to escape; it can land on other worlds of the system (as Martian meteorites reach Earth) | the parent world's crust | at the target's escape speed and up |

**Each crater draws its source** from these populations, weighted by how often each one hits this world at that date. That is the share of each population on orbits crossing the world, times focusing, using the classic crossing-probability method *(check: Öpik 1951; Wetherill 1967)*. So the mix changes over time on its own: a debris belt dominates for a few hundred Myr after its collision, then fades; a stellar passage brings a burst of comets; interstellar visitors are a steady trickle. The background lunar-style chronology stays as the total, and provenance splits it, so S4's crater counts and ages do not change.

**What the source changes on the ground** (all real physics, all explainable):
- **Speed and angle**: speed from the population's orbits; angle from the sin 2θ law already used in F4, so 45° is most common. Below about 15° craters go elliptical with butterfly ejecta *(check: Gault and Wedekind 1978; Bottke et al. 2000)*.
- **Size**: S4's crater scaling (Holsapple 1993) already takes impactor density and speed. So a dense iron fragment and a fluffy comet of the same mass make different craters.
- **What survives of the impactor**:
  - **Slow or oblique hits** leave fragments: iron-nickel masses and strewn fields of meteorites, sometimes a fragment in the melt sheet.
  - **Fast hits** vaporise it, leaving only a chemical trace. That trace is an iridium and osmium enrichment, which a very large impact spreads worldwide as a thin dated layer (the dinosaur-extinction layer is the real case; *check: Alvarez et al. 1980*).
- **What it delivers**:
  - **Iron, nickel and platinum metals** from core fragments. This is the "old planetary core" Alex asked for: a 3 km body that really was the core of a named world destroyed 4.4 Gyr ago. *(check: Elkins-Tanton et al. 2020 on Psyche as a possible exposed core)*
  - **Water and organics** from carbonaceous bodies and comets, added to the world's water and life books. This links to the life suite's delivery of building blocks.
  - **Radioactives** with a real origin:
    - long-lived uranium, thorium and potassium in fragments of a destroyed world's crust;
    - an r-process-rich interstellar body from a system its galaxy cell enriched more;
    - live iron-60 dust fallen from a recent G6 supernova. This is not an impact, but it is the same deposit layer, and it is real *(check: Knie et al. 2004; Wallner et al. 2016)*.
  - **Impact products**: melt sheets (ore bodies form in some, though how much metal comes from the impactor is debated and labelled so), shocked quartz, impact diamonds, tektite strewn fields, and hydrothermal systems in large young craters (a habitat).
- **Deposits are located facts** in S7's deposit layer: a strewn field ellipse downrange of the angle, a buried mass under the rim, a melt-sheet ore body, a global iridium layer in the S9 strata. They are useful for mining, stories and sifters ("a crater whose meteorites are older than the star").

**Answering "where did it come from?"** Every crater's explanation gains a line such as: "Made 1.2 Gyr ago by a 3 km iron body from the debris of Proto-Kel, a world shattered 4.4 Gyr ago at 1.8 AU. It arrived at 18 km/s at 27° from the north-east. About 40,000 t of iron-nickel lies under the south rim." Going the other way, a meteorite found on the ground can be traced to its parent population or named parent body, which F5 resolves deterministically, so "the asteroid this came from" can be found and mined.

**Cost and the cloud rule**:
- Listed craters (S4's global list) and craters resolved at a site get a drawn source.
- Small craters keep only the population mix per cell, as a share. They draw a source only when someone looks closer.
- Cost is a few draws per crater: negligible.

### 5.8 Detail levels: good-looking worlds at once, full history only where it matters (new package S16) **[Alex, 21 Nov: firm requirement]**

**The requirement**: most worlds are glanced at and skipped. They must look good, with terrain worth looking at, almost at once, and must never cost a full tectonic run. Full history, rock layers, caves and impactor sources are worked out only for worlds someone goes close to or asks about. A coarse world and the detailed world it sharpens into must be **the same world**: the continents a player saw from orbit are still there after the survey.

**A problem in what is built today (to fix first)**: as built, the cheap globe for unexamined and scanned worlds (S0's sketch, made from smooth keyed bumps) and the surveyed atlas (S2's plate run) are made separately. They have separate addresses (`shell:sketch@` and `shell:surface@`) and nothing ties their continents together. So a world's continents can jump when it is surveyed. I have read this from the S1 and S2 logs and not yet confirmed it in a run. Either way, S16 removes it.

#### The levels

**Two rules over all the levels** **[Alex, 21 Nov: "best calc is NO calc"]**
1. **Reuse before computing, always.** Before computing anything at any level, look for a result that already exists. Check, in this order:
   - this device's cache;
   - the GM's machine, or a peer that already has it (the tiles are data, so receiving them is cheaper than working them out);
   - a world shipped with a pack or from the Explorers library.

   A result is only reused when it is for the same world key, code version and level, so a reused result is always identical to a fresh one. The quality badge does not need to say which source it came from, because the result is the same either way.
2. **Go up a level only on real need.** That means one of three things:
   - **approach**: the camera goes close, or a landing or route is planned;
   - **stay**: the world stays the focus for a few seconds, counted in frames or ticks of attention, never by the wall clock in the kernel;
   - **explicit request**: survey, Explore, cross-section, or the override.

   A click, a hover, a list or a fly-past never starts D2 or D3. A world that only ever gets a click costs D0, plus D1 if it was actually opened. Higher levels are also never pre-computed "just in case". The one exception is a GM marking a world for D2 ahead of a session.


| Level | When | What is computed | What it shows | Cost (estimates, to measure) | What "why is this here?" can honestly say |
|---|---|---|---|---|---|
| **D0 Glance** | a dot in a list, the system chart, a world far away | envelope and regime (S0), already done for every world | colour, brightness, bands on giants, no heights | microseconds | the kind of world and why ("a plate world: heat flow 1.6 times the critical value") and nothing about any place on it |
| **D1 Orbit** | Body view opened and kept open, or the camera close enough that the world fills the view | the **same plate, wear, crater and climate code** on a coarse grid: 32 × 32 cells a face (6,144 cells, about 310 km on an Earth), 100 Myr steps; only the largest craters (S4's global list); cover and climate bands; a few big named features | a real globe: continents, the main ranges, ocean ridges, the big basins, ice caps, deserts and rain shadows, large craters, all dated. Detail below the cell size is noise shaped by the terrain kind | about 16 times less work than today's full run (6,144 cells × 45 steps against 24,576 × 176). Target: under 0.2 s in a worker | dated causes for anything continent-sized ("this range: a collision 310 Myr ago, worn for 280 Myr"). For finer detail: "texture typical of an old worn collision belt, not yet resolved. Survey to see its history." It never invents a cause for decoration |
| **D2 Survey** | the GM's survey action, Explore, a planned landing, or the user's override | the fine run **guided by D1** (below): 128 × 128 a face, 25 Myr steps, full wear, every crater with impactor sources (S15), the full feature register | tiles down to the 100 m fact floor, rivers, named features, deposits | seconds in a worker (S2b's grid). Weak devices fetch it from the GM | full dated causes for every listed feature |
| **D3 Site** | landing, a walked area, a base, a cross-section or core asked for | rock columns (S9) **only for the cells asked about**, caves and voids (S14), flourishes (S10) | the 10 m site floor, the geology under the feet, cave survey maps | milliseconds to a second per site, because rock columns are worked out per cell from the stored history and need no neighbours | everything, down to the layer and the passage |

Disclosure (what an audience may see) and detail (what has been worked out) stay separate. A GM may resolve a world to D2 and still show players only a scanned view. A player view never triggers work beyond its own disclosure.

#### How the coarse world stays the same world

The rule is the one SSE4 uses everywhere: **the coarser level is a pin for the finer one**. The fine run is not a second, independent simulation. It refines the coarse one.

- **Shared plates.** Plates, their rotation poles and speeds, and every reorganisation event (splits, collisions, pole jumps) are drawn once at D1 from the world's key. D2 uses the same plates and the same dated events.
- **Refinement, not a rerun.** Each D1 cell splits into 16 D2 cells. A D2 cell starts with its parent's plate, crust kind and age. Plate boundaries are placed where D1 put them and then roughened by keyed noise inside one coarse cell, using S2b's sub-cell boundary lines. The fine 25 Myr steps run within the coarse 100 Myr ones, so belts, arcs and rifts get their true narrow width without moving.
- **Stated tolerances, tested the way S1 already tests tiles.**
  - D2 heights averaged back to the D1 grid match D1 within a stated tolerance.
  - Land and sea agree on at least a stated share of the area (proposed 95%), and a coastline never moves by more than one coarse cell.
  - Every feature named at D1 keeps its address and name at D2.
  - Sea level, ice caps and climate bands agree within tolerance.
- **The fudge, stated** (it goes in `docs/fudges.md`): the fine run inherits where D1 put the boundaries, so fine plate geometry is limited by the coarse placement to about one coarse cell. Why we accept it: a fine run left free would drift away from what players already saw, because plate histories are chaotic, and honesty to the shown world matters more. What would replace it: nothing in a browser. A free fine run is still available as a GM choice that makes a new world.
- **Rerunning gives the same result.** Same key, same code, same level, same world, on every device. Nothing needs storing to stay consistent.

This is also the honest form of the live preview in 6.2: dragging a slider reruns D1 (well under a second), and D2 follows when the slider stops.

#### The resolution control

- **Automatic by default.** The level follows attention (D0 in lists and system charts, D1 when a world fills the view, D2 on survey or Explore, D3 at sites) and is capped by the device tier (a phone stops at D1 locally and fetches D2 and D3 from the GM's machine, as the design already says).
- **A user override per world**: a small four-step control on the Body view (Glance, Orbit, Survey, Site), plus a device setting "work out detail up to: Orbit / Survey / Site". The quality badge always says which level a view is showing and why ("Orbit detail: this world has not been surveyed").
- **Campaign setting for the GM**: worlds that start at D1 (default all of them) and worlds that start at D2 (the home world, plot worlds).

#### Memory budget per device

Everything at every level can be recomputed from the world's key, so caches can always be discarded safely. The only things kept for good are the small records SSE4 already keeps (resolves, names, GM edits).

| Device | Surface cache budget (proposed default, user-adjustable) | Roughly holds |
|---|---|---|
| Phone | 64 MB | D1 for about a hundred worlds (well under 1 MB each) plus tiles around one site |
| Laptop or tablet | 256 MB | D1 for every world in play, D2 for a few, D3 for their sites |
| GM desktop | 1 GB | D2 for dozens of worlds, plus the tiles it streams to weak devices |

- **Space at each level**:
  - D2 keeps the fine final state, about 3 MB. Its drift snapshots for the time ruler are kept at D1 resolution only.
  - Rock columns are kept only for the cells asked about. The earlier 30 MB is the worst case for a whole world's geology map at D2, and it is only built if someone asks for one.
- **Eviction**:
  - Least recently used first, counted by use and never by the clock, as S1's tile cache already works.
  - Worlds with recorded sites are evicted last.
  - Caches are kept in browser storage (OPFS) across sessions, within the budget.

---

## 6. Can a browser carry all this?

### 6.1 Budget

| Piece | Cost | Where it runs |
|---|---|---|
| Plate history at 98,304 cells, ~180 steps | a few seconds natively (4× today's run); wasm somewhat slower | worker, once per world, cached by epoch |
| Wear plus strata bookkeeping | same order as S3 today, plus layer appends | worker, once |
| Strata store | tens of MB at worst, before compression | desktop cache; phones receive tiles |
| Caves | envelopes per cell are tiny; graphs only for sites and walked areas | anywhere |
| Tiles with geology layer | one more byte a sample | anywhere |
| Cave 3D volume | only the walked area | gpu-high; survey map everywhere |

Nothing here is a fluid simulation. The heavy parts run once per world and are cached; weak devices take results from the GM, as already designed.

### 6.2 Getting Rock 3's live feel without breaking determinism

The GPU cannot compute facts because floating point differs between GPUs. Instead:
1. **A preview run**: the same code on a 16-a-face grid with 100 Myr steps, well under a second, re-run on every slider move. It is deterministic too, just coarse, and it is labelled "preview".
2. **The full run** starts in a worker when the slider rests, and the globe sharpens when it lands. The quality badge says which you are looking at.
3. **Parallel workers**: steps split by fixed cell ranges stay deterministic; with wasm SIMD this could bring the full run near interactive on a desktop.
4. **What-if branches**: the time ruler's look-ahead already gives "change one thing, compare".

### 6.3 What we should not try, and why

- 3D mantle convection, global metre-scale erosion, or a full general circulation model: not in a browser, and not needed. Each would be a swappable big block (as weather already is), and the rule blocks stay the default.
- GPU-computed facts: breaks determinism and peer play. GPU stays for decoration below the floor.

So: no reason we cannot go well beyond Rock 3. The limits are ones we chose, and each has a stated way around it.

---

## 7. Rulings (accepted 21 Nov; each was a question with a recommendation, and the recommendation stands)

1. **Tectonic grid size.** 64 a face today; Rock 3's count is about 92,000. *Recommendation*: 128 a face (98,304) as the one default, with phones fetching results. Higher in a "detailed survey" pack setting.
2. **Order of the new work.** *Recommendation*:
   0. **S16a detail levels D0 and D1**, the resolution control and the cache budget, before anything else, because every later package runs inside them and the sketch mismatch should be fixed now. **S16b** (the guided fine run) is built together with S2b.
   1. **S14a lava tubes** first among the new features. They need only S3 routing and S4 volcanism, which are built, and they are Alex's favourite.
   2. **S15a impactor provenance**. Everything it reads (F4 debris, F5 belts and reservoirs, G6 showers, S4 craters, S7 deposits) is built, so it lands cleanly and soon.
   3. **S2b** tectonics (5.1).
   4. **S9** strata with hardness-aware wear (5.2, 5.3). **S15b** (global impact layers) goes with it. Reefs, coal, salt and glacial deposits all arrive in this step too.
   5. **S14b** karst and acid caves, which need strata and a water table.
   6. **S10** flourishes with lifespans.

   Climate catch-up (5.4) goes to the weather thread in parallel. Map export (5.6) is small and can go any time.
3. **Live preview.** *Recommendation*: the coarse deterministic preview (6.2), not a GPU path.
4. **Caves as facts or decoration.** *Recommendation*: passage graphs are kernel facts (a base built in a lava tube must be the same tube for everyone); only wall detail is decoration.
5. **Water table.** Karst needs one. *Recommendation*: a simple rule block where the water table follows the land surface, damped where rain is low and rock is permeable *(check: Haitjema and Mitchell-Bruker 2005, Ground Water)*, rather than a groundwater flow model. Swappable later.
6. **Impactor provenance: drawn per crater, or a share only.** *Recommendation*: listed and site-resolved craters draw a named source. Small craters keep a per-cell population share until someone looks closer. Provenance only splits S4's total and never changes its counts, so existing goldens stay put except for the added fields.
7. **Detail levels.** *Recommendation*: coarse grid 32 a face at 100 Myr for D1; automatic by attention and device with a per-world override; at least 95% land-and-sea agreement between levels; the budgets above as defaults. Every world gets D0. D1 only when a world is opened or approached, and higher levels only on real need, after checking for an existing result (Alex's two rules above).
8. **Which thread owns it.** *Recommendation*: the physics thread ("Build SSE4 proven core") owns S16, S2b, S9, S10, S14 and S15 as kernel work; the renderer thread owns the cave survey map and later the 3D volume; the weather thread owns 5.4.

---

## 8. Proposed build plan rows

| Stage | Delivers | Needs | Verified by |
|---|---|---|---|
| S2b | 128-a-face plate grid; transform faults and fracture zones; continental rifts, passive margins and failed rifts; back-arc basins; plateaus and terranes; erosional rebound; ocean crust age retuned | S2, S3 | Earth fixture mean ocean crust age within 2× of 60 Myr; fracture zones perpendicular to ridges; a rifting continent opens an ocean with matching margins; replay identical |
| S9 (as designed, made concrete) | depositional environment per cell per step; rock kind tables per world kind; thickness from S3 volumes; unconformities; structure from events; geological map layer; cross-section and core | S3, S2b | sediment volume equals layer volume; hard-over-soft fixture forms mesas; tilted-bed fixture forms cuestas; a canyon fixture shows dated layered walls |
| S10 (extended) | flourishes with formation time and lifespan by rock and process; knickpoint retreat; arch and stack life cycles | S9 | waterfalls retreat upstream at the pack rate; arch counts in equilibrium on a long run; every flourish answers why and when it will go |
| S14a | lava tubes: envelopes from vents and flow paths, gravity-scaled widths, skylights and pit chains by age, rilles; passage graphs on approach; cave survey map | S3, S4 | tubes only on lava units and gentle slopes; lunar fixture widths larger than Earth's by the gravity scaling; a site resolves the same graph on every device |
| S14b | karst and acid caves from strata and water table; levels dated from base-level changes; ice caves; Titan karst; ice-shell lenses; sea caves | S9, S14a | cave levels match the fixture's dated base-level drops; no karst without soluble rock; every void answers why |
| S15a | impactor provenance: source populations from F4, F5 and G6 with crossing probabilities through time; speed and angle per source; elliptical craters; impactor survival (fragments or vapour); delivered metal, water, organics and radioactives as located deposits; crater explanations with their source; meteorite-to-parent tracing | S4, S7, F4, F5, G6 | crater counts and ages unchanged from S4; a debris-belt spike's share decays with the belt; iron sources make smaller craters than comets of equal energy, as the scaling says; strewn fields lie downrange; every deposit names its impactor |
| S15b | global impact layers (iridium and osmium, spherules, shocked quartz) and supernova iron-60 as dated strata markers | S9, S15a, G6 | a large-impact fixture writes one worldwide layer at its date; the layer shows in cross-sections and cores |
| S16a | detail levels D0 and D1 on the coarse grid (one model at both levels, the sketch no longer a separate world); reuse before compute (cache, GM or peer, pack); levels raised only on approach, stay or request; the resolution control and quality-badge wording, cache budgets per device, why-answers labelled by level | S2, S3, S4, S7 | D1 under the stated time on the harness; a scanned world's continents are the surveyed world's continents; why at D1 never names a cause for sub-cell detail; caches evicted and rebuilt give identical tiles; a scripted session that clicks through 50 worlds runs no D2 or D3 work; a result already held by the GM or a pack is fetched, not recomputed |
| S16b | the D2 run guided by D1 (16 children a cell, shared plates and events, sub-cell boundaries) | S16a, S2b | D2 averaged to D1 within tolerance; at least 95% land and sea agreement; every D1 feature keeps its address and name; replay identical |
| S8a | equirectangular map export per layer | S1 | exported height map round-trips to tile heights within tolerance |
| W7 (weather thread) | land and sea heat capacity, local albedo, current rules, retrograde winds, ice-age cycles | W1, S7 | coastal versus interior seasonal range on the Earth fixture in the observed sense; a retrograde fixture's winds flip |

---

## 9. Sources

- Rock 3, Insight Games, Steam store page: https://store.steampowered.com/app/1892520/Rock_3/ (read 21 Nov 2026). Feature list from the developer's Reddit post as pasted by Alex.
- SSE4 package logs S0 to S7 and W1 to W6 in `handover/packages/`.
- Named for builders to check: Stock and Montgomery 1999 (JGR, stream power and lithology); Crosby and Whipple 2006 (Geomorphology, knickpoints); Molnar and England 1990 (Nature, erosional rebound); Gerya 2012 (Tectonophysics, transform faults); Bruthans et al. 2014 (Nature Geoscience, sandstone arches); Sauro et al. 2020 (Earth-Science Reviews, lava tubes on Earth, Moon and Mars); Blair et al. 2017 (Icarus, lunar lava tube stability); Haruyama et al. 2009 (GRL, Marius Hills skylight); Keszthelyi 1995 (JGR, lava tube thermal budget); Palmer 1991 (GSA Bulletin, limestone caves); Cornet et al. 2015 (JGR Planets, Titan karst); Malaska et al. 2020 (Icarus, Titan labyrinth terrain); Schmidt et al. 2011 (Nature, Europa chaos and shallow water); Haitjema and Mitchell-Bruker 2005 (Ground Water, water table); Walther 1894 (facies law); Öpik 1951 and Wetherill 1967 (collision probabilities); Gault and Wedekind 1978 and Bottke et al. 2000 (oblique impacts, elliptical craters); Alvarez et al. 1980 (Science, iridium layer); Elkins-Tanton et al. 2020 (JGR Planets, Psyche); Knie et al. 2004 and Wallner et al. 2016 (iron-60 from nearby supernovae).
