# SSE4 build plan: for the coordinator who starts the builders

Date 17 Nov 2026. Audience: a coordinating session planning the work, and the building sessions it starts. Design only has been done so far; no V4 code exists.

## 0. Read in this order

1. `SSE4-design-handoff.md` : the architecture, contracts, decisions. The authority.
2. `SSE4-galaxy-design.md`, `SSE4-life-design.md`, `SSE4-surface-design.md`, `SSE4-classification-design.md`, `SSE4-system-formation-design.md`, `SSE4-sound-design.md`, `SSE4-weather-design.md` `SSE4-nebulae-and-light-design.md` and `SSE4-record-logic-programs-design.md` : the domain designs in detail. `SSE4-surfaces-and-rendering.md` is the research report on browser photorealism, hardware scaling and software choices; it is the authority for rendering and must be read before any R or V package.
3. The "SSE4 mockups" canvas : every screen. Chart room board is the reference for the GM look.
4. "SSE4 decisions for Alex" : the reasoning trail, if a decision seems odd.
5. On Alex's machine under `C:\Development`: the V3 source (for parity and import), `ovid-next` (agent system to reuse), `evolution-lab` and `evolution-lab-design` (contracts, UI components, experiments).

The design sessions did **not** read the V3 or Ovid source. Work package K0 fixes that before anything else is trusted.

## 1. Standing rules

- Never push without Alex's word. Commits are authored as FrunkQ <frunk@frunk.net>. Real user files stay outside the repo. V3 is not touched.
- Licence: Apache-2.0 with a NOTICE file crediting Alex (FrunkQ), for SSE4 and Evolution Lab **[decided 17 Nov]**.
- Repo: https://github.com/FrunkQ/star-system-explorer. Ask Alex for the branch or folder convention before the first commit.
- Deterministic paths contain no wall clock, no unordered iteration, no platform maths, no LLM output.
- Each new physics atom starts as a GitHub issue with its spec (declaration, formula, validity, citations, golden cases), then tests, then code.
- If the design is wrong or silent, write an ADR in `/docs/adr`, say so in the evidence packet, and ask Alex when it changes what users see. Do not quietly diverge.
- Published relations named in the design must be checked against their source before they are relied on; record the citation. If a named relation turns out to be misremembered, replace it and note it.

## 2. How a work package runs

One package, one builder session, one branch, one pull request. A package is done only when its **evidence packet** is in the pull request:

```
What was built (three lines)
Design sections implemented, and any deviation with its ADR
Commands to reproduce every check
Results: pass/fail per acceptance check, with numbers
Golden hashes added or changed, and why
Known limits and what was not done
Decisions needed from Alex (at most three, each with a recommendation)
```

The coordinator reviews the packet, not the builder's summary. Checks must be runnable by `sse-cli` or CI, not by eye, except UI reviews which need screenshots at desktop and phone widths in both GM modes.

**Device edges** **[decided 17 Nov]**: weak devices do less; capable devices compute internally and work better offline. Every kernel operation declares a cost class. On first run a device measures itself in deterministic work units and gets a capability tier. An operation above the tier is either requested from the GM's machine as recorded data when connected, or offered in a reduced form, and the UI always shows where the edge is and why (on charts: an "unsurveyed beyond this device's range" marking with a reason and what would lift it). Every package that adds an operation must declare its cost class and its edge behaviour.

## 3. Tracks and packages

Tracks can run in parallel once their dependencies are met. `needs` lists package ids. Section numbers refer to the handoff (H), galaxy (G) and life (L) documents.

### K: kernel foundations

| Id | Package | Needs | Acceptance checks |
|---|---|---|---|
| K0 | Read V3, Ovid Next and Evolution Lab source; write `docs/v3-parity.md` (every V3 feature: keep, replace, drop), `docs/ovid-agents.md` (what to reuse), `docs/evolab-reuse.md` | none | documents reviewed by Alex; list of V3 save-file fields for the importer |
| K1 | Workspace, CI, lint, canonical encoding, SHA-256 content addressing, pinned maths, cross-device hash harness (H3, H12) | none | harness runs natively and in Chrome, Firefox, Safari, iOS Safari; identical hashes for 10,000 maths and encoding cases |
| K2 | Addresses, quantities and units with dimension checks, keyed draws (H5.1 to 5.3) | K1 | property tests: draws independent of call order; unit mismatch rejected at validation; hashes stable |
| K3 | Chronicle, stores, checkpoints, branches, run manifest (H5.4 to 5.6) | K2 | fold of events equals checkpoint; branch then discard leaves state hash unchanged; manifest round-trips |
| K4 | Flow documents, validation, scheduler, Solve nodes, explain traces (H6.2, 6.5) | K2 | a ten-atom toy flow evaluates, explains every output, rejects cycles and dimension errors |
| K5 | Pack manifest, stack resolution, one-publisher rule, evidence and fictional labels (H7) | K4 | conflicting packs rejected with a readable report; declared override works; fictional label propagates |
| K6 | Kernel message API, worker binding, typed client, cancellation and progress in work units, cost classes and capability tiers (H9.1; section 2 above) | K3, K4 | every API call covered by a contract test run both natively and through the worker |
| K7 | Envelopes, partitioning, resolve, feasibility queries, generic (H6.1, 6.4) | K3 | order independence; budget closure with pins; "not found within budget" path |
| K8 | Rules, classification, sifters, significance (H6.6) | K4 | a rule pack produces tags with hysteresis; sifter hits carry evidence |
| K9 | Disclosure tiers, data views, scene building (H6.7) | K6 | filtered scenes for the mockup's Tarn before and after scan match fixtures |
| K10 | Suite socket and coordinator at time boundaries (H11) | K3, K7 | a stub suite exchanges fluxes; coupled run replays bit-identically |

### P: physics suite

| Id | Package | Needs | Acceptance checks |
|---|---|---|---|
| P1 | Atom library, first wave: star properties from mass, age, metallicity (table lookup); orbits; planet bulk; insolation; equilibrium and surface temperature; atmosphere basics; interior shells; tides | K4 | each atom has golden cases from cited sources; Solar System fixture values within stated tolerance |
| P2 | System formation: now track F, stages F0 to F10 exactly as `SSE4-system-formation-design.md` section 8, one package each. Includes porting SSE 3's Accrete (System Lab) as the Classic block | P1, K7 | as listed there |
| P3 | V3 importer (one-way) | K0, P1 | a set of real V3 systems (Alex supplies, kept outside the repo) import; derived values match V3 or each difference is explained in a report |
| P4 | Regions, layers, features on, in and above worlds: plates, oceans, sub-surface oceans, biomes, settlements, aerostats | P1 | Tarn fixture reproduces the Strata mockup's stack |
| P5 | Second wave atoms as issues arise, each by the issue-spec-test-code rule | P1 | per atom |

### T: time and travel

| Id | Package | Needs | Acceptance checks |
|---|---|---|---|
| T1 | Epoch stepping, stellar evolution over time, secular orbit changes, detailed windows with a fixed-step symplectic integrator (H6.3) | P1, K3 | energy drift bounds; epoch stepping against the integrator on reference systems with error reported |
| T2 | Branch look-ahead and commit; "age the Solar System" scenario end to end | T1 | scripted run to the white dwarf stage: engulfment outcomes, orbit expansion, cooling colours, each explained; replay from manifest |
| T3 | NAV: route candidates, hazards, difficulty, drift states (H6.8) | P1 | fixture transfers match analytic solutions; "ship not under power" returns ways out |
| T4 | SE and US2 converters | P3 | round-trip fixtures |

### U: GM interface (chart room)

| Id | Package | Needs | Acceptance checks |
|---|---|---|---|
| U0 | Rule for all U packages (H8.8): the GM workspace is assembled from view modules and a screen document; every edit is an intent through the same client API as the player app; no private path to the kernel | K6 | lint or test proving the shell imports only the client API; an edit made through a GM view produces the same event as the same intent from a player seat with auto-approve |
| U1 | Shell, chart room design tokens for light and dark modes, chart grammar components (ground, grid, ink, hatch, stamps, cards, time ruler) | K6 | screenshots match the Chart room board in spirit; keyboard operable; phone layout |
| U2 | System chart editor: edit in place, cards pinned to bodies, why on every number | U1, P1 | edit Tarn's orbit by drag and by typing; undo; explain opens |
| U3 | Flow editor and explain mode with three lenses; Orac change-request panel (without agent) | U1, K4, K5 | open a real flow, change a parameter, see diff and badge |
| U4 | Time ruler with history, branches, look-ahead | U1, T2 | scrub, branch, compare, commit |
| U5 | Packs screen, data views screen, screens composer | U1, K5, K9 | reproduce the PlayerData and Compose boards against live data |
| U6 | Galaxy and patch charts, history inking, find tool | U1, G3 to G6 | reproduce the GalaxyHistory and PatchSurvey boards against live data |
| U7 | Biosphere view, life timeline, life flow | U1, L1 | reproduce the Biosphere and LifeFlow boards against live data |
| U9 | Lenses (H8.6): lens documents, switcher, card and layer configuration, save and reuse, the shipped defaults, every player data view as a lens | U2, K9 | the Lenses board reproduced against live data; switching lens leaves the state hash unchanged; a lens saved, exported in a pack and reloaded is identical; honesty stamps cannot be hidden on the GM view but can be on player views |
| U8 | Story finder | U1, K8 | hits listed with evidence; send to Minerva |

### V: views, players, peer-to-peer

| Id | Package | Needs | Acceptance checks |
|---|---|---|---|
| V1 | View contract, scene descriptions, first views: Chart, Card, Table, Text, Readout (H8.1, 8.2) | K9 | a new view added in a package that does not touch `/crates` |
| V2 | Themes: tokens, frame parts, filter chains; four shipped themes (starship panels, green phosphor, clean high tech, friendly guidebook) plus chart and paper (H8.4) | V1 | one screen renders correctly in all themes; no view contains theme-specific code |
| V12 | Symbology (H8.4A): bindings from facts to channels, the two base grammars (diagrammatic, colour-coded with V3's table), mixing, theme capability rules, generated legend, colour and shading editors ported from V3's colour-coding screen | V1, V2 | the same system renders in both grammars and a mix under every shipped theme; a one-colour theme still distinguishes every planet class; legend matches the marks; no view contains encoding code; state hash unchanged; a V3 save's custom colours import |
| V3 | Screens documents and composer runtime (H8.3) | V1 | the Compose board's screen loads from its document |
| V4 | Peer-to-peer session: manifest, authored events, clock, data-view state, state hash, snapshot, version pinning, GM-offline read-only (H9.2) | K6, K9 | phone and desktop agree by state hash; GM drop gives banner and read-only; version mismatch forces reload |
| V5 | Intents and GM approval | V4 | a player's course request becomes an event only after approval |
| V10 | Seats and audiences (H8.7): per-audience knowledge grants, tag-tied audiences, per-seat join links, several screens per seat, role intents and auto-approve scopes; sealed seats as a second step | V4, V5, K9 | default session shows no seat UI; a three-empire fixture gives each seat only its discoveries by state inspection; a sealed seat's device holds no seed and no unfiltered fact; two windows of one seat stay in step |
| V11 | Orac-first configuration (H8.9): style and settings documents with parameter schemas, inheritance, promoted controls, validation and cost classes, for every view | V1, V2 | each shipped view exposes a schema; an out-of-range or over-budget document is rejected; promoted controls render from the schema; changing a style leaves the state hash unchanged |
| V6 | System 3D, Body, Cutaway views with capability tiers | V1 | each declares tiers; canvas fallback works |
| V7 | Sky view and backgrounds (H10) | G5, G7 | sight-line checks; clicking a sky point opens its chart |
| V8 | Route (astrogation console) and Window views | V5, T3 | reproduce the PlayerPlan and Window boards |
| V9 | Map views for cluster, galaxy, foam | G1, G3 | time scrub animates foam growth |

### G: cosmos suite. Stages G0 to G8 exactly as galaxy document section 12, one package each. G0 early (needs K7); G1 needs K4; then in order.

### S and R: surfaces and rendering. Stages S0 to S13 and R1 to R4 exactly as `SSE4-surface-design.md` section 9 (merged with the research report `SSE4-surfaces-and-rendering.md`), one package each, with the dependencies listed there. Renderer base is three.js `WebGPURenderer` with a WebGL2 fallback.

### L: life suite. Stages L0, L1a, L1b, L1c, L1d, L2, L2b exactly as life document section 14, one package each. L0 needs K10 and P4; L1a needs P1; L2 needs T1.

### C: classification. Stages C1 to C5 exactly as `SSE4-classification-design.md` section 12, one package each. C1 needs K5 and P1; C3 needs K8 and L0; C4 needs U3 and A2; C5 needs G5. The Traveller pack's codes (X3a) are schemes in this machinery, so X3a needs C1 and X3b needs C3.

### O: light and nebulae. Stages O1 to O11 exactly as `SSE4-nebulae-and-light-design.md` section 8, one package each. Light stays a spectrum until a per-view band recipe; nebulae are objects with causes and lifetimes.

### W: weather. Stages W1 to W6 exactly as `SSE4-weather-design.md` section 8, one package each. Rule-based, deterministic, behind a contract so a simulation block can replace it without breaking saves.

### N: sound. Stages N1 to N7 exactly as `SSE4-sound-design.md` section 7, one package each. Off by default, synthesised from data-only recipes, no baked audio.

### J: the campaign record, logic blocks, boxes and programs **[decided 19 Nov]**. Stages J1 to J9 exactly as `SSE4-record-logic-programs-design.md` section 10, one package each. J1 (needs K3, K5, K9), J2 (K4, K8) and J3 (K4, K5) can start now. Read that file's section 9 first: it lists five points of care against code already built (flow document hashes, note filtering in scenes, the intent list moving into the kernel, X3d building on J4 and J7, and record data leaving the peer-to-peer broadcast). Section 8A (secrecy: what each device receives) binds every J package.

### A: agents

| Id | Package | Needs | Acceptance checks |
|---|---|---|---|
| A1 | Provider adapter and key handling lifted from Ovid Next; keys never leave the browser | K0 | works with two providers; app fully usable with no key |
| A2 | Orac: change requests for flows, packs, screens, data views, sifters, atom issue drafts; all validated before display | A1, U3, U5 (J3, J6 for boxes and programs) | invalid candidates never reach the accept button; nothing mutates without accept |
| A3 | Minerva: narrative from sifter hits and explanation bundles, respecting data views | A1, K8, K9 | no number in output that is not in the supplied facts (automated check); will not reveal unscanned data |
| A4 | Tars: support from docs and explanation bundles, humour setting | A1 | answers cite the bundle or doc they came from |

### X: sharing and release

| Id | Package | Needs | Acceptance checks |
|---|---|---|---|
| X1 | Cloudflare deployment, V4 subdomain, signalling worker | V4 | fresh device loads player app from the web and joins a session |
| X2 | Explorers site: V4 entries beside V3 with labels; publish and fetch manifests and packs | K5 | publish, fetch, hash verified |
| X5 | Real-data service (H10, surfaces-and-rendering 3.3): Worker and R2 tile cache for Gaia stars, exoplanet archive, Solar System elevation and imagery; address-scheme tiling; catalogue versions pinned in the run manifest; device cache; offline fallback with the edge shown; licence and attribution register | K5, G7 | two peers with the same manifest get identical pins; a known exoplanet keeps measured values and resolves the rest as tagged fill; no real star is duplicated by the fill; changing catalogue version is an explicit action with a diff; offline run shows the edge |
| X3 | Base content: default presentation pack, a generic game-system pack, Milky Way pack | K5, G7 | pack validation |
| X3a | Traveller pack, rules (H7A parts 1, 8, 9): UWP, PBG, trade codes, zones, bases from classification rules; units, calendar, vocabulary; lenses, reports, sifters | K5, U9 | codes come from rules only; every digit answers why; world to UWP to world keeps every digit |
| X3b | UWP to system (H7A part 2): codes as pins and constraints, constrained resolve, `traveller-generated` tag, impossible values kept and marked, "make physically sound" change request, seeded subsector rolls | X3a, K8 | reference UWP set resolves with every code honoured; impossible cases tagged with reasons, none altered; same seed gives same subsector on every device |
| X3c | Hex map view in the core stack, configured by the pack; importers and exporters: UWP lines, SEC, T5 tab-delimited, sector XML, Travellermap fetch (H7A parts 3, 4) | V1, V12, X3b | a reference subsector imports, displays and exports back to an equivalent file; import is undoable; hex view works with a non-Traveller configuration |
| X3d | Gas mixes, sensors and the knowledge ladder, constructs, jump-era fictional physics pack with jump shadows and refuelling (H7A parts 5, 6, 7), ported from V3's tables | X3a, T3, V5, J4, J7 | jump shadow fixtures match hand calculation; sensor rule moves a body up the ladder at the right range and time; everything downstream of the physics pack carries the fictional label; removing the bundle leaves a valid campaign |
| X4 | Documentation and learning content: every atom's three lenses written, in-app glossary | P1 onward | coverage report: no atom without lenses |

## 4. Milestones (each is a usable thing Alex can try)

| Milestone | Packages | Alex can |
|---|---|---|
| M1 Proven core | K0 to K6 | see identical hashes on his devices; read the parity documents |
| M2 One system | P1, P3, P4, U1, U2, G0, F0 to F2, F6, C1, C2 | import a V3 system, edit it on the chart, ask why |
| M3 Open physics | K5, U3, K8, X3 (generic pack), C3, C4, F3 to F5, F9, F10 | change physics in the flow editor; stack packs |
| M4 Players | K9, V1 to V6, U5, U9, X1 | run a session with themed player screens and the Scanned button |
| M4b Sound | N1 to N3 | turn sound on, hear a system's background tones change from world to world, hear the astrogation console |
| M5 Time and travel | T1 to T3, U4, V8, U8, A1 to A4, F7, O10 (after O1, O2) | age the Solar System; plot courses; agents on his own key |
| M5c Living campaign | J1 to J7, J9 | jot a note on a statue and find it on the timeline; track factions and what they think of the party; set a mining drone's autopilot and look ahead to see what it does |
| M5d Turns (later) | J8 | run a week of downtime or a turn-based game with sealed orders |
| M5b Traveller table | X3a to X3d, V12, O9 (after O1, O2, O8) | import a subsector from Travellermap onto a hex map, open any world as a full system with its impossible values marked, plan a jump with real jump shadows, hand players a UWP sheet |
| M6 Living worlds | K10, L0 to L1d, U7 | open any world's biosphere and history |
| M6b Surfaces | S0 to S7, R1 to R4, V11, N4, N5, W1 to W3, W5, O1, O2 | chart a world's terrain, ask why a range is there, watch continents drift, land on it in several styles |
| M7 Galaxy | K7, G1 to G6, U6, V9, C5, F8, N6, N7, O3 to O8, O11 | chart a galaxy, ink a merger, find a home, survey a patch |
| M6c Ground detail (later) | S9 to S13, W4, W6 | watch a layered canyon cut over millions of years; find a sea arch on approach; walk through alien forest; place landmarks and ruins; chart a country with borders and roads |
| M8 Whole universe | G7, G8, V7, L2, L2b, S8, T4, X2, X3, X5 | real Milky Way skies; coupled living planets; share on the explorers site |

M6 and M7 are independent of each other and of M5 after M2, so they can run in parallel.

## 5. Risks to watch

1. Cross-device floating point: settle in K1 before anything else exists.
2. Frames and promotion (G8.3): prototype in G5 with the identity test before building views on it.
3. Calibration of the galaxy model: accept loose tolerances; do not tune one relation by breaking another; record what was tuned.
4. Life dials: keep them visible; resist hard-coding Earth.
5. Scope: every milestone must be shippable on its own. No milestone depends on a later one.
6. Performance on phones: cost classes and edges from K6 onward, not retrofitted.
7. Shared packs that imitate commercial settings are user content; the shipped packs and themes use original designs and generic names.

## 6. Open items for Alex (none blocks M1)

- Branch or folder convention in the repo for V4.
- A handful of real V3 save files for the importer tests (kept outside the repo).
- The V4 subdomain when X1 is near.
- Which game-system pack to ship first after the generic one (Traveller assumed).
