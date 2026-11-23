# SSE4 real spaceflight pack: what is feasible

Track RS (23 Nov 2026, thread "Build real spaceflight history pack"). Status: ACCEPTED 23 Nov 22:01 (Alex: "yes to all - very sensible approach"), with question 10 as he amended it.

Alex's ask (23 Nov 20:14): everything that covers spaceflight until now, all stations and all craft that have left Earth, simple 3D models of our own, used to test our systems from Shuttle entry to modern reusable craft, the transfers to other planets, maybe a store pack with every mission on our timeline. Additions: every craft is assembled from its real engines, fuels and other parts, sensors included (20:14); SSE3 constructs are the starting point and import point, so everything is additive (20:15); SSE3's flight logs let you rewind and the ship is where it was, SSE4 needs that and it is what makes the historical pack work (20:16); simplify and approximate, only the big burns, stable orbits between, no station corrections (20:17, a ruling); it is an extension of SSE3's constructs re-engineered for SSE4, done now because the needs of 4.0 and later (future tech, 4.1 life and economy, 4.2 RPG) are clear (20:17).

Short answer: **yes, feasible, and smaller than it sounds, if we split "everything" into a wide thin catalogue and a deep showcase.** The catalogue (every launch, every station, everything that left Earth orbit) is a few megabytes of public data. Deep reconstructions (flown paths, every burn, parts-built models) are worth doing for about 10 craft first and perhaps 40 in the end. Every Cassini course change is possible for Cassini, and is the showcase, not the rule.

Figures marked ~ are my estimates from memory, to be firmed up in package RS1.

## 1. What data exists, and its terms

| Source | What it gives | Terms (guidance, not legal advice) | Use |
|---|---|---|---|
| Jonathan McDowell's GCAT (planet4589.org/space/gcat) | every orbital and deep-space launch since 1957, every catalogued object, launch vehicles, **stages and engines** with masses, thrust and Isp | free use with credit (stated on its acknowledgements page; confirm the exact licence wording in RS1) | backbone of the catalogue and the launch-vehicle parts |
| Wikidata | spacecraft, COSPAR ids, launch dates, masses, operators, links between items | CC0 | cross-check and fill gaps; safe to bulk-load |
| NASA NSSDCA master catalog | per-spacecraft description, launch mass, power, **the instrument list per craft** | US government work, no copyright in the US; credit NASA | instruments, masses, mission summaries |
| JPL Horizons (ssd.jpl.nasa.gov/horizons) | reconstructed trajectories for ~150 to 200 spacecraft (negative ids), sampled at any step | NASA/JPL, public, credit JPL | the flown path of deep-space craft |
| NAIF SPICE kernels (naif.jpl.nasa.gov, PDS archive) | the full reconstructed ephemerides, attitude, instrument frames, for NASA missions and some ESA/JAXA | public; ESA's SPICE service has its own free-with-credit terms | the source behind Horizons; burn times can be read off the kinks |
| Mission press kits, NASA Technical Reports Server, mission design papers | burns (Δv, dates), masses by stage, engines, entry and landing figures, instrument specs | NASA public; papers are cited, never copied | parts figures and the event lists |
| Space-Track / CelesTrak TLEs | current orbits of Earth satellites | Space-Track's user agreement limits redistribution | **not used**; GCAT's own orbit records are enough for history |
| Wikipedia | everything, quickly | CC BY-SA, share-alike | spot checks only; cite the primary source it cites, never bulk-copy |
| Russian, Chinese, Indian, Japanese agency material, RussianSpaceWeb, Encyclopedia Astronautica | Soviet and Chinese figures | facts free, the pages are copyright | individual cited figures only |

Facts (a mass, a date, a thrust) are not copyrightable; a compiled database can carry database right in the UK and EU, which is why the backbone is GCAT with credit, Wikidata (CC0) and NASA (public), not scraped sites.

## 2. How many things

| Group | ~Count | Size as data |
|---|---|---|
| Orbital launch attempts, 1957 to now | ~7,000 | ~2 MB raw, a few hundred KB packed |
| Catalogued payloads in Earth orbit | ~25,000, more than half of them one constellation | carried as a list plus constellation shells, never 10,000 drawn Starlinks |
| Crewed flights | ~370 (135 Shuttle) | small |
| Space stations | ~15 (Salyut 1 to 7, Skylab, Mir, ISS, Tiangong 1 and 2, the Chinese station, Genesis I and II, BEAM as a module) | small, but ISS is ~40 dated module joins |
| Craft that left Earth orbit | ~300 (Moon ~150 including Apollo, Venus ~45, Mars ~50, Mercury 3, outer planets ~12, small bodies ~20, Sun and Lagrange points ~30) | the heart of the pack |
| Launch vehicle families | ~150, with ~400 distinct stages | parts |
| Engines | ~500 that have flown | parts |
| Instruments | ~2,000 across the deep-space craft | parts, thinnest data (section 4) |

## 3. Trajectory detail: four levels

1. **Catalogue**: launched when, from where, on what, where it ended (orbit, landed, impacted, escaped, re-entered). Every object.
2. **Events**: the dated mission script: launch, departure burn, each flyby (body, date, closest distance), orbit insertion, landing, end. ~10 to 50 events per mission. Between events, SSE4's own physics flies the leg (Lambert and patched conics already in sse-nav). This is the level that **tests our transfers against reality**: we know where Voyager 2 was on 25 Aug 1989; does our solver get it there from its departure burn? For all ~300 deep-space craft.
3. **Flown track**: a sampled reconstructed path from Horizons, stored as compact fits (Chebyshev pieces, like SPICE does). Cassini's 20 years at a daily step is ~7,000 states, ~100 KB packed, loaded only when opened. For the ~20 to 40 showcase craft. With this, every course change is visible as a kink and can be listed as an event.
4. **Full SPICE**: attitude, instrument pointing, minute steps. Not needed; level 3 is enough to draw and test.

**Alex's ruling (20:17): big burns only, stable orbits between, no station-keeping or small corrections.** So level 2 is the standard for every craft, and level 3 tracks are kept only as the yardstick to measure our error against, not as what the chart plays. One care point: a craft's path also changes at flybys with no burn at all (Cassini's tour was shaped by ~120 Titan flybys, Voyager's by four giants), so flybys are events in the log alongside burns, or the approximation drifts off by whole planets.

**The fudge, stated and logged:** "Big burns only: trim manoeuvres, station-keeping and drag are left out; each leg is solved to meet its recorded next event, so a craft is right at every logged event and off only in between." Estimated size: deep-space trims total a few to tens of m/s per mission, so pinning each leg to its next recorded event keeps mid-leg error to roughly thousands to tens of thousands of km on legs of hundreds of millions (well under 0.1%). In low Earth orbit a station held on a fixed orbit is closer to truth than one decaying without its reboosts, but its phase along the orbit drifts, so where it is in its orbit at a given minute is only good to a fraction of an orbit after a few days. RS3 measures both against the Horizons tracks and writes the real numbers into the fudge register.

Two lines on the chart, never one: **flown** (recorded, the honesty mark "recorded") and **SSE4 says** (computed from the craft's parts and the event script). The difference is the test result, shown on the GM's Engine depth.

## 4. Parts: how much is open, and where it is thin

Every craft is assembled from parts with cited figures, as Alex asked, never one lump.

| Part family | Coverage | Thin spots |
|---|---|---|
| Engines and stages (thrust vac and sea level, Isp, mass, propellant, mixture, restarts, throttle) | good: GCAT, NASA, manufacturers, papers | Soviet and Chinese upper stages, some commercial engines (estimates widely quoted, few primary sources) |
| Propellants | complete: these are substances already on the master data sheet | none |
| Tanks and structure | stage dry and wet masses are good; tank volumes are derived | individual tank masses rarely published, so tank mass is computed and logged |
| Power: RTGs, solar arrays, fuel cells, batteries | good for deep-space craft (RTG output and decay, array watts at 1 AU) | military and commercial satellites |
| Heat shields and entry | good for Apollo, Shuttle, Mars landers, Huygens, Orion, Dragon | Soviet capsules partial |
| Radiators | ISS, Shuttle bay doors good; most probes have no separate figure | usually computed from power, as base-ships already does |
| Sensors and instruments | a list exists for every NASA and ESA craft (NSSDCA); the figures SSE4 needs (band, aperture, field of view, resolution) are in instrument papers, uneven | the thinnest family. Spy satellites unknown. Plan: real figures for the showcase craft, "listed, not sized" for the rest |
| Crew and life support | good for crewed craft | none for 4.0 |

**Electric power feeds engines (Alex, 23 Nov 20:21).** Power is a general input an engine can draw, not only propellant: reactors, RTGs, solar arrays and fuel cells make power, losses become waste heat for the radiators, and an electric engine (Dawn's ion engines, Hall thrusters, SNAP-10A's own ion test) declares the power it draws and its efficiency, so jet power = efficiency x electrical power and the rest is heat. SSE3 had `systems.power_plants` and `powerDraw_MW` on engines but never connected them. The model itself is being settled in the "Bring Atomic Rockets into SSE4" thread together with the future-tech jet power balance, so there is one model; these parts only record the measured figures it needs (input power, thrust, exhaust speed, efficiency, specific mass of the power source). Working shape (Atomic Rockets plan section 11.3, proposed as its question 12; the travel thread confirms names): construct `power[]` {id, kind, electric_w, thermal_w?, reject_k, mass_t, law: constant | sunlight {ref_flux_w_m2} | decay {half_life_y, since_s} | fuel {fuel}}, `engines[]` {engine, count}, `loads[]` {name, electric_w, reject_k}; engine definitions {fuels[], thrust_n, exhaust_km_s, exhaust_range_km_s? [low, high], energy_from: fuel | electric | beam | sunlight, electric_w, efficiency, reject_k, mass_t}. Confirmed by the travel thread 23 Nov. Sources' outputs are given at a stated reference (1 au flux for arrays, a date for RTGs). Instruments use the same plan's section 12 shape: band by id, aperture_m, resolution, method; no typed ranges. Check: Dawn's NSTAR at about 92 mN, 31 km/s exhaust and 2.5 kW in gives 0.5 x 0.092 x 31,000 / 2,500, about 57%, matching the expected ~60% [figures from memory, verified in RS2].

Where a figure is missing, the part carries an "estimated" mark with its method and goes in the fudge register, never a quiet invention. The "Bring Atomic Rockets into SSE4" thread (track RK, not accepted) also proposes engine tables; real flown engines should come from primary sources here, and RK can point at the same parts.

## 5. SSE3 as the starting and import shape (additive)

SSE3 already builds constructs from parts, which is the right start:

- a construct node (`kind: "construct"`) with `engines: [{engine_id, quantity}]`, `fuel_tanks: [{fuel_type_id, capacity_units, current_units}]`, `sensors: [{definition_id, name, range_km}]`, `systems.power_plants: [{type, output_MW}]`, `systems.life_support`, `crew`, `physical_parameters {dimensionsM, massKg, cargoCapacity_tonnes, can_aerobrake, thermal_protection_type}`, `attachedTo` (docking), `model`, `flight_log`;
- libraries `engineDefinitions` (`thrust_kN`, `efficiency_isp`, `fuel_type_id`, `powerDraw_MW`, `atmo_efficiency`, `drive_tags`), `fuelDefinitions` (`density_kg_per_m3`), `sensorDefinitions` (`range_km`), and `constructTemplates`.

The importer (`crates/sse-import-v3`) already counts constructs. Additive plan: keep every SSE3 field and meaning, so an SSE3 save still imports unchanged; add optional fields and new part kinds alongside:

- engine: `thrust_sl_kN`, `isp_sl_s` (SSE3's `atmo_efficiency` becomes a derived reading), `mass_kg`, `mixture_ratio`, `throttle_min`, `restarts`, `src`;
- fuel: points at the master data sheet substance instead of its own density (the density stays as written for old saves);
- sensor: `band`, `aperture_m`, `fov_deg`, `resolution`; SSE3's `range_km` stays as a written override and otherwise is computed;
- new kinds in the same list style: `stages`, `power` (RTG with decay, arrays with distance), `heat_shield`, `radiators`, `instruments`;
- every figure with `src` and a mark (cited, estimated, authored).

This is data shape only; the SSE4 sizing and physics remain its own, not SSE3's design.

## 6. What SSE4's systems need

1. **A parts catalogue** as pack data (section 4), built on base-ships' sizing rules, so real hardware is made from SSE4's own parts and numbers, not special-cased.
2. **Split and join at dated events**, one mechanism: stages drop, a lander leaves its orbiter, a probe releases Huygens, modules dock to build ISS, Shuttle sheds its tank. A construct can become two constructs, or two one, at an event. SSE3's `attachedTo` is the join half.
3. **A mission script**: the events of section 3 level 2, held as dated data on the construct (SSE3's `flight_log` is its import shape).
4. **Recorded tracks**: an optional flown path (level 3), loaded on demand, marked "recorded".
5. **Launch from the ground**: ascent through Earth's air (gravity turn, max-q, staging). New kernel physics; entry and landing already exist (sse-nav entry.rs, `ship_at_world`).
6. **The timeline**: missions are spans on the one universe clock; a mission list on the ruler lets you jump to any mission and play it.
7. **Models from parts**: the renderer's ship designer already builds hulls from parts. For real hardware it needs a few **shape hints** (a lathe profile, a wing planform, a truss, a dish, a flat panel, a box) placed by the craft record, and five procedural finishes (white blanket, gold foil, solar cell blue, black tile, bare metal). Our own work; no agency meshes or artwork.

## 6A. A flight log on every construct

**What SSE4 has today (checked on main, 23 Nov):** `plan_voyage` (sse-nav `voyage.rs`, contract `ui-contract/src/voyage.ts`) plans legs with a departure time (`depart_s`), courses (Hohmann, Lambert, torch), waits and Δv, and the record-logic design (section 6.2) already says a ship on a course needs no simulating, because where it is on a leg is a closed-formula look-up. But **nothing stores a committed voyage as a dated history on the construct**, so nothing can say where a ship was last Tuesday. The V3 importer keeps SSE3's `flight_log`, `scheduled_journeys` and `draft_transit_plan` only as raw authored data, unread. **So the flight log needs building in the travel code** (sse-nav, with the kernel thread for the record), as a general construct feature, not a special for history.

**SSE3's shape (the import point):** `scheduled_journeys` hold the transit legs, and `flight_log` holds `ConstructLogEvent`s at the stops: `{id, journeyId, atSec, kind: depart|arrive|load|unload|mine|refuel|loiter|stuck|disengage, text, placeId, resourceKey, tonnes, durationSec, fuelKg, plannedAtSec, fromConstructId}`, in universe seconds, scrubbable by time, totals derived by reducing over the events.

**SSE4's flight log, proposed: one dated list of pieces on any construct, each piece a closed formula for where it is.**

- **State pieces**, which tile the construct's life without gaps: `orbit` (body, elements, epoch: stable between events, per the ruling), `course` (from, to, depart, arrive, kind: Hohmann, Lambert, torch, jump: the same courses plan_voyage already makes), `landed` (body, site), `docked` (to a construct, SSE3's `attachedTo`), `track` (optional, a recorded flown path for the yardstick), `gone` (re-entered, impacted, lost, escaped).
- **Events** at the joins: `burn` (Δv, engine, propellant used), `flyby` (body, closest distance), `split` and `join` (stages, landers, probes, modules: the mechanism of section 6 point 2), `launch`, `land`, plus SSE3's own kinds unchanged (depart, arrive, load, unload, mine, refuel, loiter, stuck, disengage).
- **Where is it at time t** = find the piece covering t, evaluate it. Rewind and fast-forward are a look-up, never a re-run, so scrubbing the ruler across sixty years of spaceflight costs nothing heavy.
- **Who writes it**: a committed voyage appends its legs; the GM can write pieces by hand; a pack ships them (the historical craft); later the 4.1 economy's autopilots and the 4.2 game layer append the same pieces and events. One log, many writers, each entry stamped with its writer and honesty mark (recorded, planned, authored, estimated).
- **Import**: SSE3 `scheduled_journeys` become `course` pieces, SSE3 `flight_log` events come across as events with every field kept; nothing SSE3 wrote is dropped or reinterpreted.
- **Built for later, built now only as 4.0 needs**: 4.0 needs the pieces, the events, the look-up, voyage commit, import and pack-shipped logs. Cargo totals by reducing over load and unload (SSE3's Totals) come along cheaply. Economy rates, encounters and estimated long jumps (record-logic 6.2) use it later without changing its shape.

**Real missions in it:** each real craft ships a log of a few dozen pieces: launch, parking orbit, departure burn, course to the next flyby, flyby, course, orbit insertion, orbit pieces between big burns, landing or end. A dated event list in a pack, a few KB per mission; the optional Horizons track is a separate file used only to measure the error.

## 7. Serving the future-tech test (FT improvement 6)

The future-tech design wants about ten real craft reproduced at "Sol today" within a stated tolerance. This pack is that test's data: each craft's parts go in, the tech blocks size the craft, and the result is compared with the real mass, Δv and dimensions. The first slice below is picked so each craft tests something different.

## 8. A first slice: twelve craft

| Craft | Tests |
|---|---|
| Saturn V with Apollo 11 (CSM and LM) | staging, ascent, trans-lunar burn, lunar landing and ascent, split and join |
| Space Shuttle, STS-1 | winged entry and landing, tank drop, boosters |
| Falcon 9 with a booster landing | propulsive landing, reuse |
| Voyager 2 | the flyby chain past four giants, RTG decay |
| Cassini-Huygens | gravity assists, orbit insertion, a probe released, every trim burn at level 3 |
| Juno | solar panel area and output at 5 AU (swapped in for New Horizons at the future-tech thread's request; RTGs are covered by Voyager, Cassini and Curiosity) |
| Dawn | ion drive, two orbits round two bodies |
| Curiosity (Mars Science Laboratory) | Mars entry, parachute, sky crane |
| James Webb Space Telescope | a halo orbit at L2, sunshield as a radiator |
| International Space Station | a station assembled over 13 years from ~40 dated joins |
| V-2 (A-4) | the future-tech "one level short" check against a modern engine: thrust, engine mass, exhaust speed at sea level and vacuum |
| SNAP-10A (Snapshot, 1965) | the only US fission reactor flown: reactor power for its mass |

So the slice is twelve, agreed with the future-tech thread (its note section 15.6). Beyond parts, mass, Δv and dimensions, each craft carries these measured figures where a source gives them, with source and uncertainty (tolerances are the future-tech test's own, per row, in FT2): heat rejection load, radiator area and temperature (ISS, JWST); peak heating and total heat load at entry (Shuttle, Apollo command module, Curiosity); power source output and mass, and for solar the panel area and output at a stated distance; engine thrust, engine mass and exhaust speed; for ISS, supplies per person per day, the fraction of water and air recycled, and shielding in g/cm².

Each with its parts, its event script, a flown track where Horizons has one, its own generated model, and a test of "SSE4 says" against "flown".

## 9. Questions for Alex, each with my recommendation

1. **Wide and thin, then deep.** The catalogue level for everything (all launches, all stations, all crewed flights, everything that left Earth orbit), with Earth-orbit satellites as a list and constellations as shells rather than each drawn. Recommended: yes.
2. **Trajectory depth, per your ruling.** Flight logs with the big burns and flybys for all ~300 craft that left Earth orbit, stable orbits between, the approximation logged as a fudge with its measured size; Horizons tracks kept only as the yardstick for the ten showcase craft. Recommended: yes.
3. **Sources.** GCAT with credit, Wikidata (CC0) and NASA/JPL (public) as the backbone; agency and paper figures cited one by one; Wikipedia and Space-Track never bulk-copied. Recommended: yes.
4. **Additive on SSE3's construct shape** (section 5): every SSE3 field kept with its meaning, real-parts detail added as optional fields and new part kinds. Recommended: yes.
5. **Split and join as one mechanism** for stages, landers, probes and station modules. Recommended: yes; the kernel thread builds it.
6. **Two lines, flown and SSE4 says**, with the difference reported as the future-tech tolerance test. Recommended: yes.
7. **Models from parts plus shape hints**, five procedural finishes, all our own. Recommended: yes; the renderer thread builds the hints.
8. **Launch from the pad** (ascent physics) in the first slice, since launch is half of "Shuttle to reusable". Recommended: yes, for Saturn V, Shuttle and Falcon 9.
9. **Gaps stay visible**: missing Soviet, Chinese, commercial or sensor figures are marked "estimated" with their method and logged as fudges. Recommended: yes.
10. **The pack is "Sol today", on Explorers, not built in** (Alex, 23 Nov 22:01: "Lets not have it built it and have it part of the onboarding/tutorial... to let folks play"). Free, never paid. The onboarding tour teaches loading a pack from the store with it: find it on Explorers, add it, then play a mission (Apollo 11 or Voyager 2) on the ruler. Base data (catalogue and parts) in the pack; flight logs and yardstick tracks as files loaded on demand, so the first download stays small. If Explorers cannot be reached, the tour says so and offers to carry on without it. ANSWERED.
11. **A flight log on every construct** (section 6A): dated pieces with a closed formula each, events at the joins, SSE3's journeys and log imported unchanged, built in the travel code and used by voyages, the GM, packs and later the economy and game layer. Recommended: yes.
12. **The twelve craft of section 8** as the first slice, agreed with the future-tech thread as its test set. Recommended: yes.

## 10. Packages, once accepted

| Package | What | Owner |
|---|---|---|
| RS1 | Catalogue import script (GCAT, Wikidata, NSSDCA) to pack data; licence wording confirmed; counts firmed up | this thread |
| RS2 | Parts catalogue, additive schema, the ten craft's parts with cited figures | this thread, with the data sheet thread for substances |
| RS3 | Flight logs for the ten; Horizons tracks as the yardstick; the big-burns fudge measured | this thread |
| RS4 | The flight log on every construct (section 6A): pieces, events, look-up at any time, voyage commit, SSE3 import; split and join at dated events | travel thread (sse-nav) with the kernel thread |
| RS5 | Ascent from the ground | travel thread (owns entry and landing) |
| RS6 | Shape hints and finishes for real hardware | renderer thread |
| RS7 | Missions on the ruler; flown and SSE4-says lines | chart room thread |
| RS8 | Flight logs for all ~300 departed craft, stations and crewed flights | this thread |
