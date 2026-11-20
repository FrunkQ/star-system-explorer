# START HERE

Last updated: 19 Nov 2026: PR #2 (UI track) and PR #3 (kernel T1, T2) merged into the integration branch, which became `main`. Live site v4.starsystemx.com. Milestone M1 complete (K0 documents with Alex for review). Milestone 2 running as parallel tracks.

## Standing rule: real physics, documented fudges (Alex, 19 Nov 2026)
In his words: "the physics is WAY more interesting than faking it. But understand a browser has limited capability. Fall on physics rather than simplification where practical. If we decide to "fudge" it is a known and documented fudge with a damn fine reason to exist."

- Every deliberate simplification, stand-in or tuned constant goes in `docs/fudges.md` (where it lives, why it exists, what real physics would replace it) in the same PR that adds it.
- Every reference, published relation or quoted constant goes in `docs/citations.md` with its check status (verified, corrected, unverified). Check a relation against its source before relying on it.

## Standing rule: physics in the chain blocks, each with a view (Alex, 19 Nov 2026)
In his words: "try and do everything in the physics chain blocks... other than Big Blocks. And have a UI element that makes sense to visualise".

- New physics goes into the editable, cited flows of small blocks (atoms and boxes) by default.
- "Big Blocks" (heavy engine or simulator blocks: the classifier, weather, geology, stellar evolution and the like) only for what truly cannot be expressed as a flow, and each says why in `docs/fudges.md`.
- Each block or Big Block has a UI element that makes sense to visualise what it does.
- Logic hidden in Rust that could be a flow is listed in `docs/fudges.md` (section "Candidates to move into flows") until it moves.

## Where we are
Milestone M1 "Proven core" = packages K0 to K6 (build plan section 3/4).

| Package | State | Log |
|---|---|---|
| K1 workspace, encoding, hashes, maths, harness | done | packages/K1.md |
| K2 addresses, quantities, units, keyed draws | done | packages/K2.md |
| K3 chronicle, stores, checkpoints, branches, manifest | done | packages/K3.md |
| K4 flows, validation, scheduler, solve, explain | done | packages/K4.md |
| K5 packs, stacks, one-publisher, fictional labels | done | packages/K5.md |
| K6 kernel API, worker, TS client, cancellation, cost classes, tiers | done | packages/K6.md |
| J1 to J9 campaign record, logic blocks, boxes, engine blocks, programs (new track J, decided 19 Nov) | J1 to J5 and J7 done (kernel track, logs `packages/J1.md` to `J7.md`; the chart room still has to switch to the kernel intent registry, see J5.md); J6 (programs) next, after T3 NAV. Read `design/SSE4-record-logic-programs-design.md`, section 9 first (points of care against built code) | |
| K0 V3 / Ovid / Evolution Lab source reading | done, Alex reviewing docs/v3-parity.md, v3-save-fields.md, ovid-agents.md, evolab-reuse.md | packages/K0.md |
| P1 atom library, first wave | done; heat-producing elements and reference links added | packages/P1.md, packages/P1-heat.md |
| P3 V3 importer | built; run on 3 real saves on Alex's PC, one difference open (P3.md) | packages/P3.md |
| P4 regions and features on worlds | done | packages/P4.md |
| F0 inheritance, stars, life lines, formation history | done | packages/F0.md |
| F1 disc and material bands | done | packages/F1.md |
| F2 Classic Accrete block | done; **Alex to confirm**: System Lab (GPL-3.0, his code) ported under Apache-2.0 on his 17 Nov ruling, with credit (F2.md, Source) | packages/F2.md |
| F3 modern growth block | done | packages/F3.md |
| F4 violent phase and upheavals | done | packages/F4.md |
| F5 leftovers: belts, Oort cloud, Trojans, moons, rings | done | packages/F5.md |
| F6 early worlds and hand-off | done (ahead of F2 to F5; reads a plain `Protoplanet`) | packages/F6.md |
| Formation pipeline (stages chained, heat flow per world) | done | packages/F-pipeline.md |
| F7 long life of generated systems; close binaries by rule table | done (giant tables still T1's sketch) | packages/F7.md |
| F9 pins and reconstruction (solve around pinned worlds; imported systems gain a reconstructed history) | done | packages/F9.md |
| S0 surface envelope: kind, regime over time with dated causes, regime sketch globe | done | packages/S0.md |
| S1 surface atlas: cube-sphere tiles, band-limited detail, disclosure and fact floor, cache | done (sketch atlas; S2 adds the surveyed plate atlas) | packages/S1.md |
| S2 plate sketch: moving plates, ridges, trenches, arcs, mountain belts, plumes, drift snapshots, surveyed atlas | done | packages/S2.md |
| S3 wearing down: rivers, lakes, sediment, sea level, hillslopes, worn atlas with flow layer | done | packages/S3.md |
| S4 impacts and volcanism: chronology and spikes, survival by renewal, crater shapes, lava plains, shields, tiles to the fact floor, "why is this here" | done | packages/S4.md |
| S6 other regimes: ice-shell ridges, bands and chaos; heat-pipe paterae and mountains; coronae, chasmata and overturns; contraction scarps and rifts; Titan's dunes and lakes; tar seas; giant cloud decks | done | packages/S6.md |
| S7 cover, named features, built layer, aerostats, GM reshape: climate by latitude and height, wind cells and rain shadow, ice, deposits, colour; feature register with dated, per-audience renames and sifters; built-layer rules; aerostat band from the air profile; dated GM edits tagged when impossible, height pins honoured | done; served by `survey_surface` | packages/S7.md, packages/K-physics-ops.md |
| Kernel ops `import_v3`, `run_formation` and `read_world_stack` (V3 import with report; one system through the pipeline with pins and surfaces; a body's world stack with ladder and physics notes) | done (physics track, for the UI) | packages/K-physics-ops.md |
| K7 envelopes, partitioning, resolve, feasibility | done (kernel track) | packages/K7.md |
| K8 rules, classification, sifters, significance | done (kernel track) | packages/K8.md |
| K9 disclosure tiers, data views, scene building | done (kernel track) | packages/K9.md |
| K10 suite socket and coordinator | done (kernel track) | packages/K10.md |
| U0 one client API, every edit an intent (rule for all U packages) | done | packages/U0.md |
| U1 shell, chart room and night chart tokens, chart grammar | done, screenshots in packages/U1-screens/ for Alex | packages/U1.md |
| U2 system chart editor | done: real base-physics values, drag and typed edits, undo, why | packages/U2.md |
| U3 flow editor, explain lenses, change requests | done: real flows, kernel-validated drafts, adopt, badge; kernel gaps listed in the log | packages/U3.md |
| U4 time ruler with history, branches, look-ahead | done: what-if branches (part 1) and look-ahead (part 2: age the system on a branch, scrub, throw away or keep, move now) | packages/U4.md |
| V1 view contract, player scenes, Chart, Card, Table, Text, Readout | done (scenes from K9 fixtures until the kernel serves a scene op) | packages/V1.md |
| U5 packs screen, data views screen, screens composer | done; player page and composer on live kernel scenes | packages/U5.md |
| V2 themes: tokens, frame parts, filter chains, six player themes | done | packages/V2.md |
| V3 screens documents (data view, bind, layouts) and one screens runtime; composer opens screens from documents | done | packages/V3.md |
| V11 Orac-first configuration: schemas for every view, style documents with inheritance, promoted controls, validation and cost budgets, Styles screen | done | packages/V11.md |
| V12 symbology: diagrammatic and colour-coded grammars, mixing, key, Symbols screen | done, with V3's colour table and the V3 colour import (palette overrides, save tag colours) | packages/V12.md |
| V4 peer-to-peer session: replication by intent, state hash, snapshot, version pin, GM-offline read-only; relay room in the site Worker | done | packages/V4.md |
| V5 player requests and GM approval | done | packages/V5.md |
| Planet chart: moons in their planet's frame (coordinator ask, not in the plan) | done | packages/planet-chart.md |
| V10 seats and audiences | done: plans, fog of war per audience, per-seat links and screens, auto-approve, sealed seats | packages/V10.md |
| V6 System 3D (frames, camera-relative rule, pixel-stability test, GPU and Canvas tiers) | System 3D, Cutaway (from the kernel's world stacks) and Body globe (from S1 surface tiles) done; finer tiles and a GPU tier next | packages/V6.md |
| V14 Size comparison (Sizes screen: true-scale lineup from this system, the campaign and reference objects; Measure on the chart) | done (UI track) | packages/V14.md |
| U7 biosphere view and life timeline (Life screen: epochs, food web, scan tiers, hooks, traits, dials, in three lenses; example worlds until chart worlds carry habitats) | part 1 done (UI track); LifeFlow board next | packages/U7.md |
| V8 Route, the astrogation console (part 1: GM Route screen on plan_route; courses compared, the chosen one charted in the star's frame) | part 1 and 2a (Window, on the new `window_view` op) done (second UI track); next: course requests, player console, drift | packages/V8.md |
| U6 galaxy and patch charts (part 1: Galaxy under the Explore tab, from the new `galaxy_chart` op; also `web_slice` for V9) | part 1 and 2a (inked history preview) done (second UI track); part 2b: patch survey, find a home, save inked history | packages/U6.md |
| V9 map views (part 1: the Cosmic web foam under Explore, from `web_slice`, with time scrub and Play) | part 1 done (second UI track); cluster map next | packages/V9.md |
| U8 story finder (part 1: Stories under Explore, on the new `sift` op; starter sifters as data, hits with evidence, the GM's own sifter, a brief for a storyteller) | part 1 done (second UI track); Minerva and rarity next | packages/U8.md |
| U10 browser campaign save: kept in this browser after every change, Save, Open and Start fresh, players join a reopened campaign | done (UI track) | packages/U10.md |
| U11 human-scale time: one zooming ruler from Gyr to hours, a campaign "now", step and play, Make this now, worlds moving by the day (kernel `orbits_at`), pack calendars | parts 1 and 2 done (UI track); the campaign's now through the intent log (kernel `set_campaign_now` intent) next | packages/U11.md |
| U12 tactility (dynamic minimalism, `handover/interface-style.md`): motion tokens, hover peek, cards and dialogs open from what opened them, ruler glide, reduced motion honoured | parts 1 and 2 done (UI track) | packages/U12.md |
| U9 lenses: Basic, Advanced, Building, Running, every player data view as a lens, make your own, lens packs | done | packages/U9.md |
| N1 sound engine: sound scenes, recipes, mixer, tiers, off by default, captions, orbit bed | done | packages/N1.md |
| O1 light switch (UI side): Eye, Camera, SHO and HOO palettes, single bands, legend chip and honesty label, stubbed spectra marked | UI done on stubbed light; real light facts wait on the kernel | packages/O1-ui.md |
| O1 light (kernel side): light descriptions as spectra (thermal, reflected, power law, accretion disc, named lines), band set and line table as pack data, deterministic band integrals, brightness temperature | done (light track, crate `sse-light`, pack `packs/base-light`); O2 next | packages/O1.md |
| O2 light from bodies: glow, reflected light, stellar wind, magnetopause, auroral radio as atoms and the `body.emission` flow; radiators, transmitters, lamps, discs, coronae; Solar System checked; kernel op `read_light` | done (light track) | packages/O2.md |
| O3 nebulae from causes, first slice: planetary nebulae from stars' deaths (shells, shed-layer rings, lobes from companions), supernova remnants (free expansion, Sedov-Taylor, snowplough), star-forming glows (Strömgren); counts = rate × lifetime; kernel op `read_nebulae` | first slice done (light track); metal lines in O4 | packages/O3.md |
| O7 shifts (kernel side): Doppler and beaming, gravitational and cosmological redshift as exact spectrum transforms (`sse_light::shift`) | done (light track); far-galaxy colours with O6 | packages/O7.md |
| O10 stellar activity, first slice: activity from mass and spin (Rossby number), X-rays, spots, cycles, flares and mass ejections as keyed events with light and hit-or-miss by geometry (`sse-activity`, atoms `sse.activity.*`); kernel op `read_activity` | first slice done (light track); readers in other tracks | packages/O10.md |
| O12 spectrum at an observer (kernel side): components shifted and summed, flux with band coverage, emission and absorption lines where they arrive, readings with Why (temperature from the peak, speeds, doubled lines); kernel op `read_spectrum` | kernel side done (light track); chart in the shell | packages/O12.md |
| Air optics for the renderer: Rayleigh scattering per gas, scale height and optical depth by wavelength and visible slice (atoms `sse.air.*`, gas table in base-light, `sse_light::air`); kernel op `read_air_optics` | done (light track); renderer to switch over | packages/O-air.md |
| O9 sensing, first slice: detection by band against background, dark counts, glare and dust; radio by radiometer; stare needed and detection range; radar seen-versus-heard ranges (`sse-sense`); kernel op `detect` | first slice done (light track); construct sensors, seats and contacts later | packages/O9.md |
| O13 planet finding from afar: transit, wobble (minimum mass), position wobble, direct imaging, pulsar timing, per observer direction, distance, instrument and watch time (`sse-survey`); kernel op `survey_planets` | done (light track); knowledge ladder in O14 | packages/O13.md |
| V13 make a system: import a V3 save (report and preview) and grow a new system (formation run and preview) | part 1 done: previews; part 2 adopts a preview into the campaign | packages/V13.md |
| A1 agents shell, bring your own key | done (parts 1 and 2): providers, keys on the device only and apart from the settings, ledger and cap, loaded-context guard, reconciled with Ovid's source (agents track); **Alex to confirm** the Ovid relicensing line in NOTICE | packages/A1.md |
| A2 Orac: change requests from a sentence, Ovid's draft, check and fix loop; every draft checked by SSE's own validators before it can be accepted | parts 1 and 2 done (agents track): physics (kernel-checked, filed on the Physics screen), lenses, atom issue drafts, styles, player screens, symbologies, data views; part 3 next: sifters, track J boxes and programs, game tables to packs | packages/A2.md |
| A3 Minerva: narrative from one audience's scene, every number checked against her facts, secrets never sent | done for scenes (agents track); sifter hits wait on a kernel `sift` op | packages/A3.md |
| A4 Tars: support from SSE's help passages and the kernel's why?, every answer cites its source, humour setting | done (agents track); works with no key, from the passages | packages/A4.md |
| G0 default Milky-Way-like patch envelope | done (kernel track) | packages/G0.md |
| G1 cosmology, cosmic web, epochs | done (kernel track) | packages/G1.md |
| G2 haloes per cell, growth, spin, mergers | done (kernel track) | packages/G2.md |
| G3 galaxy reservoirs, rings, element groups, what died here | done (kernel track) | packages/G3.md |
| G4 patch envelopes from a galaxy, green regions, outcome-first | done (kernel track) | packages/G4.md |
| G5 stars drawn per co-rotating cell: IMF, ages, metals, multiplicity, orbits through time, pins, formation input | done (kernel track) | packages/G5.md |
| G6 galaxy events: history edits (ink a merger, starburst, quiet nucleus), supernovae, kilonovae, bursts, encounters and arm crossings drawn per cell, cross-patch exposure with travel time, before/after, "ink a merger" manifest | done (kernel track) | packages/G6.md |
| G nuclei: black-hole seeds, Eddington-limited quasar episodes, light and class, unified-model views, hole mergers, tidal disruptions, the nucleus as a body | done (kernel track) | packages/G-nuclei.md |
| K9 additions: grant op, tag-driven grants (V10 seats gaps) | done (kernel track) | packages/K9-grants.md |
| Kernel API additions: stack adopt, manifest, scene, scan_toggle, grant, look_ahead | done (kernel track) | packages/K6-additions.md |
| Numbers: exact float parse, JSON numbers decode by value (cross-track) | done (kernel track); other tracks merge and re-baseline | packages/numbers.md |
| C1 axis registry, class records, schemes, SSE 3 classic | done (kernel track) | packages/C1.md |
| C2 base physical atlas, habitability, gas mixes | done (kernel track); thresholds and hycean open for Alex's review | packages/C2.md |
| L0 life suite sockets: interfaces, habitat graphs, empty suite | done (kernel track) | packages/L0.md |
| L1a life box B: energy and nutrient caps per habitat, closures with reasons | done (kernel track) | packages/L1a.md |
| L1b life boxes C, D, E: origin, capability graph, catastrophes, dated history with causes | done (kernel track) | packages/L1b.md |
| L1c life boxes F, G: guilds with production shares, four lenses, traits from physics, vocabulary | done (kernel track) | packages/L1c.md |
| L1d life boxes I, J: signatures by tier with confidence, false positive and negative, data views, sifters, hooks, life view | done (kernel track) | packages/L1d.md |
| L2 life box H: life's fluxes and the planet's answer (air box), oxygenation by balance | done (kernel track) | packages/L2.md |
| J1 the campaign record: notes, kept values, actors and relations; GM-only, with per-audience disclosures | done (kernel track) | packages/J1.md |
| J2 typed values, logic blocks, conditions shared with K8, dice on the table secret, engine blocks (classify, place) | done (kernel track) | packages/J2.md |
| J3 boxes: documents, nesting, inlining, explain, tests, replace | done (kernel track) | packages/J3.md |
| C3 reverse placement in a system | done (kernel track) | packages/C3.md |
| T1 epoch stepping, ageing star, secular orbits, symplectic windows | done (kernel track); giant phase sketch awaits F7 | packages/T1.md |
| T2 look-ahead branch, commit, age the Solar System | done (kernel track); with Alex for review | packages/T2.md |
| X3a: code sheets (`sse-codes`) and the Traveller pack's world codes (UWP, PBG, trade codes, zones, bases) from classification rules; units, Imperial calendar (`ui-contract/calendar.ts`), words; kernel op `codes`; world data sheet view (`sse.view.world-codes`) | done (Traveller pack track); lens, reports and sifters next | packages/X3a.md, packages/X3a-sheet.md |
| X3b part 1: a whole system from a coded world (pins to a formation request, codes win, impossible values marked with reasons, make physically sound, counts searched) | done (Traveller pack track); kernel one-shot op and moons next | packages/X3b.md |
| X3c: hex map view (`sse.view.hex-map`); sector formats (`sse-sector-io`); import through the code sheet (kernel op `import_sector`, preview or authored events); Travellermap fetch through the site Worker (`/tmap/*`) | done (Traveller pack track); GM screens wait on the screen hook | packages/X3c.md, packages/X3c-io.md |
| X3d parts 1 to 3: gas mixes and what to wear outside; sensors, range bands and the knowledge ladder as triggers and grants (to each ship's own crew); stock ships, stations and sites as J7 templates | done (Traveller pack track); stock ships (J7) and jump-era physics (J6) next | packages/X3d.md |
| S5 descent scene: entry heating, lift, parachutes, powered descent and touchdown; site slope, roughness and liquid from the height map; hazards in one shape; kernel op `plan_descent` | done | packages/S5.md |
| T3 NAV: candidate courses (Hohmann, Lambert grid, torch) with burns, fuel and reserve, dose, heat, grazes, belts, debris, delays and difficulty with reasons; a drifting ship's fate and ways out; kernel op `plan_route` | done | packages/T3.md |
| R1 pass library, render recipes, degrade order, governor, quality badge (the Body globe runs on recipes: GPU, canvas, words) | done (renderer track) | packages/R1.md |

## Parallel tracks (18 Nov, Alex: build in parallel, integrate later)
| Track | Packages | Branch |
|---|---|---|
| Physics | P1, P3, P4, F0 to F6 done; pipeline, F7, F9, S0 to S4, S6 and S7 done, served by the `survey_surface` op; T3 NAV done (`sse-nav`, `plan_route`); S5 done (`plan_descent`); F8 next (G6 is in), S8 on G7; next B1 one body (handover/design/SSE4-star-census.md), then weather W1 to W3 and W5, then S8 or S9 | `main` via PRs |
| Kernel | K7 to K10, G0 to G6, C1 to C3, L0, L1a to L1d, L2 (with E3 competing O2 sinks, `packages/E3.md`), J1 to J5, J7, T1, T2 (also through the API), the U3 gaps, the V10 grant gaps and the UI asks (`adopt`, look-ahead write-back, `resolved_system`; K6-additions.md; `orbits_at`, campaign clock and calendars, `packages/orbits-clock-calendar.md`) galactic nuclei (`packages/G-nuclei.md`) the `sift` op for the agents (`packages/sift-op.md`) and J6 part 1 (programs, `packages/J6.md`) done; next J6 part 2 (NAV travel, pirate and patrol), then G7 | `main` via PRs |
| Renderer (R1 to R4, then exotic objects in bands) | R1, R2a (air from kernel facts, physical lighting, True colour), R3 (still tier, "Take photograph") and R4 part 1 (checked recipe edits, hand editor) done; next R2b clouds and haze from the weather op, R2c magnetic field overlay, R5 light-lens views, R4 part 2 with Orac (A2) | `main` via PRs |
| Chart room shell and player views | U0 to U3, V1, U5, V2, V12, V3, V11, V4, V5, planet chart, V10, V6 System 3D, U9 lenses, N1 sound, U4 (what-if and look-ahead), A1 part 1, O1 light switch (UI), V13 parts 1 and 2 (import and grow, add to the campaign, read back and age with write-back), V6 Cutaway and Body globe (Body screen), phone screen menu, V14 size comparison and chart measuring, U10 browser campaign save, U7 biosphere view part 1, U11 human-scale time parts 1 and 2, U12 tactility parts 1 and 2 done; next the campaign's now through the intent log and the switch to the kernel's intent registry (J5), V7, U7 part 2 (LifeFlow) | `main` via PRs |

**Since 19 Nov (Alex): `main` is the default branch.** New work branches from `main` and arrives as a pull request
into `main`; its checks pass before merge. The old track branches are kept, not deleted.

The kernel now registers the real atom library and loads the base-physics pack (see P1 log, integration notes). The kernel track has merged that in.

Kernel track integration points (K7 to K10):
- New crates only: `sse-resolve`, `sse-sift`, `sse-observe`, `sse-suite` (path dependencies; root `Cargo.toml` untouched). No K1 to K6 interface changed.
- Shared files touched: `crates/sse-wasm` (adds `engines_run()`), `scripts/build-harness.sh`, `tests/harness/{harness.js,index.html,run-node.mjs,run-browsers.mjs}` (the harness now also checks the four engine golden scenarios in Node and every browser).
- Numbers (log `packages/numbers.md`, every track): serde_json `float_roundtrip` is on, and a JSON number that is integral and below 2^63 in size decodes as an integer, so `281` and `281.0` hash the same (limit 2^63, set in C1). Write fixture numbers as JavaScript writes them.
- Kernel API additions (log `packages/K6-additions.md`): `stack.adopt` events when a stack is loaded into an open universe, plus `manifest`, `scene` (with an optional draft `observe` preview), `scan_toggle` and `atoms` ops, flow tests run at load_stack, and the `provides.other.observe` pack key.
- Not yet in the kernel message API (H9.1): `resolve`, `query_feasible`, `sift`, `subscribe`. Wire them when the first consumer needs them (U2 for resolve and scenes, U5/V1 for data views, U8 for sift); each engine already has a pure function and a cost class (`ops()`) ready.
- G0 (`sse-cosmos`, pack `packs/base-cosmos`): the default patch envelope F0 starts from (`SystemBirth`, fixture `tests/fixtures/cosmos/solar-patch.json`). It added three things to K7: broken power laws, given weights, and `CountSpec::Fill`. G0's log has a determinism note on JSON floats that applies to every track.
- C1 (log `packages/C1.md`): pack `packs/sse3-classic` (SSE 3's 72 classes, exact SSE 3 scoring), pack key `provides.other.schemes`, `classify.relabel` events via `sse_sift::relabel::record`, golden `tests/golden/classify.json`.
- C2 (log `packages/C2.md`): pack `packs/base-atlas` (physical, surface, air, habitability schemes; generated by `scripts/build-base-atlas.py`), gates now count in region fit, `Atlas::overlaps`, fixture `tests/fixtures/classify/exoplanets.json`.
- L0 (log `packages/L0.md`): crate `sse-life` (fixed life interfaces, habitat graphs from world stacks, `NoLifeModel`), pack `packs/base-life` (habitat rules), fixture worlds Earth, Mars, Europa, Titan, Venus, golden `tests/golden/life.json`.
- L1a (log `packages/L1a.md`): `sse-life::budget` (box B: energy caps from light and chemical supply, phosphorus caps, closures with reasons, biomass, feeding levels), `provides.other.life_budget` in `packs/base-life`, golden `tests/golden/life-budget.json`, harness engine `life-budget`.
- L1b (log `packages/L1b.md`): `sse-life::history` (origin hazard with a dial, capability graph against opportunity, catastrophes with refuges and recovery, `record` to the chronicle), `provides.other.life_history`, Earth and Europa timelines in `scenario.rs`, golden `tests/golden/life-history.json`, harness engine `life-history`.
- L1c (log `packages/L1c.md`): `sse-life::biosphere` (`resolve`: guilds, relations in four lenses, diversity, world traits from `WorldPhysics`), golden `tests/golden/life-biosphere.json`, harness engine `life-biosphere`.
- L1d (log `packages/L1d.md`): `sse-life::signatures` (`Observables`, `signatures`, `facts`, `disclosure`) and `sse-life::sift` (`epochs`, `hooks`, `view`), golden `tests/golden/life-signatures.json`, harness engine `life-signatures`.
- L2 (log `packages/L2.md`): `sse-life::history::{Boundary, StepFluxes, history_with}` and `sse-life::feedback` (`AirBox`, `CoupledManifest`, `coupled`), pack key `provides.other.life_feedback`, golden `tests/golden/life-feedback.json`, harness engine `life-feedback`.
- J1 (log `packages/J1.md`): crate `sse-record` (notes, kept kinds `provides.other.record.kept`, actors), the kernel's separate GM-only record chronicle with its own hash, scene notes in `sse-observe`, ops `add_note`, `set_kept`, `add_actor`, `record`, `disclosures`. **UI track:** never broadcast the record; send each seat its `disclosures`.
- J2 (log `packages/J2.md`): typed wires in `sse-flow` (`Shape`, `Val`, `NodeDoc.config`, `Atom::eval_typed`, `Env` and `World`; all optional, no hash changed), crate `sse-logic` (logic set, `to_blocks` for K8 predicates, `chance.roll` and `chance.draw` on the table secret with `commitment`, `engine.classify`, `engine.place`). `atoms` lists logic blocks with `class` and port `shape`. **UI track:** draw `class: logic` and `engine` blocks apart from physics; an engine block opens its own editor.
- J3 (log `packages/J3.md`): boxes in `sse-flow::boxes` (`BoxDoc`, `NodeDoc.box`, `Flow::compile_with`, `box_decl`, `run_box_tests`), `provides.boxes` and box `replaces` in `sse-pack`, the kernel inlining at `load_stack`. Inner node ids are `<box node>/<inner>`. **UI track:** show a box collapsed using `Flow::boxes`; ask for a `boxes` op when the editor needs box faces.
- G5 (log `packages/G5.md`): `sse-cosmos::stars` (`Stars`: cells, draws, orbits, state, pins, `birth`), golden `tests/golden/stars.json`, harness engine `stars`.
- C3 (log `packages/C3.md`): crate `sse-place` (reverse placement through the base-physics flows, four outcomes, picker preview, chronicle commit), pack `packs/base-place`, fixture `tests/fixtures/place/systems.json`, golden `tests/golden/place.json`. The base atlas gained `needs`.
- T1 and T2 (logs `packages/T1.md`, `T2.md`): crate `sse-time` (integrator written fresh from Yoshida 1990, not from REBOUND; Laplace-Lagrange; the ageing star on F0's life line with a giant-branch core clock; look-ahead branches, commit, replay), pack `packs/base-time`, fixture `tests/fixtures/time/solar-system.json`, golden `tests/golden/time.json`. One-line K3 fix in `Chronicle::branch_append` (branch cut from an empty chronicle). **Physics track:** F7's giant tables replace the core clock in `sse-time/src/star.rs` (`star_with`, `core_at`).
- Pack content keys these engines read: `provides.other.rules`, `.axes`, `.classes`, `.sifters` (K8). Data views and disclosure maps are plain serde types (K9); a pack key for them is for U5 to choose.

Each track keeps to its own crates and folders. Integration points are listed at the end of each package log. UI work stays in `packages/ui-contract`, `packages/skins`, `packages/chart-grammar`, `packages/views`, `packages/shell`, `tests/ui`; it does not touch `/crates`, and a missing kernel op is reported in the package log.

## Deployment
https://v4.starsystemx.com: Cloudflare Worker `star-system-explorer` (Workers Builds, set up by Alex 18 Nov 2026).
Build `npm run build` (scripts/build-site.sh, writes dist/), deploy `npx wrangler deploy` (wrangler.jsonc),
production branch: switch to `main` in Cloudflare (was `claude/project-thread-h9y77o`); other branches get preview versions.
The site is the GM shell (chart room) at `/`, the players page at `/players.html` and the cross-device check page at `/harness/`.

## How to resume
1. Read `/CLAUDE.md`, then this file, then the log of the last package. Any view or renderer package also follows `rendering-rules.md` (no precision jiggle: local orbits, nested frames, camera-relative numbers).
2. `cargo test --workspace` must pass before you change anything.
3. Take the first package marked "next"; write its log in `packages/` as you go (evidence packet format: build plan section 2).
4. At the end of a package: `scripts/check.sh`, update this table, re-sync `design/`, commit as FrunkQ, push.

## Environment
Rust 1.97.0 (rust-toolchain.toml, wasm32 target), wasm-bindgen-cli 0.2.130, Node 22 (`npm install`), Playwright for browser checks. Install and run: see README. UI: Svelte 5.58, Vite 8.3; `npm run ui:check`, `npm run ui:test`, `scripts/build-shell.sh`, `node tests/ui/shell.e2e.mjs`.
