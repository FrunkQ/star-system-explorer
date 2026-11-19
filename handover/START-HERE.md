# START HERE

Last updated: 19 Nov 2026: PR #2 (UI track) and PR #3 (kernel T1, T2) merged into the integration branch, which became `main`. Live site v4.starsystemx.com. Milestone M1 complete (K0 documents with Alex for review). Milestone 2 running as parallel tracks.

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
| U9 lenses: Basic, Advanced, Building, Running, every player data view as a lens, make your own, lens packs | done | packages/U9.md |
| N1 sound engine: sound scenes, recipes, mixer, tiers, off by default, captions, orbit bed | done | packages/N1.md |
| O1 light switch (UI side): Eye, Camera, SHO and HOO palettes, single bands, legend chip and honesty label, stubbed spectra marked | UI done on stubbed light; real light facts wait on the kernel | packages/O1-ui.md |
| V13 make a system: import a V3 save (report and preview) and grow a new system (formation run and preview) | part 1 done: previews; part 2 adopts a preview into the campaign | packages/V13.md |
| A1 agents shell, bring your own key | part 1 done: providers, keys on the device only, ledger and cap, Tars working, Orac and Minerva panels; part 2 reconciles with Ovid's source | packages/A1.md |
| G0 default Milky-Way-like patch envelope | done (kernel track) | packages/G0.md |
| G1 cosmology, cosmic web, epochs | done (kernel track) | packages/G1.md |
| G2 haloes per cell, growth, spin, mergers | done (kernel track) | packages/G2.md |
| G3 galaxy reservoirs, rings, element groups, what died here | done (kernel track) | packages/G3.md |
| G4 patch envelopes from a galaxy, green regions, outcome-first | done (kernel track) | packages/G4.md |
| G5 stars drawn per co-rotating cell: IMF, ages, metals, multiplicity, orbits through time, pins, formation input | done (kernel track) | packages/G5.md |
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
| C3 reverse placement in a system | done (kernel track) | packages/C3.md |
| T1 epoch stepping, ageing star, secular orbits, symplectic windows | done (kernel track); giant phase sketch awaits F7 | packages/T1.md |
| T2 look-ahead branch, commit, age the Solar System | done (kernel track); with Alex for review | packages/T2.md |

## Parallel tracks (18 Nov, Alex: build in parallel, integrate later)
| Track | Packages | Branch |
|---|---|---|
| Physics | P1, P3, P4, F0 to F6 done; pipeline, F7, F9, S0 to S4 done; F8 waits on G6; next S6 (other regimes), S7 | `main` via PRs |
| Kernel | K7 to K10, G0 to G5, C1 to C3, L0, L1a to L1d, T1, T2 (also through the API), the U3 gaps, the V10 grant gaps and the UI asks (`adopt`, look-ahead write-back, `resolved_system`; K6-additions.md) done; next G6 or L2 (life feedback on the planet; needs T1) | `main` via PRs |
| Chart room shell and player views | U0 to U3, V1, U5, V2, V12, V3, V11, V4, V5, planet chart, V10, V6 System 3D, U9 lenses, N1 sound, U4 (what-if and look-ahead), A1 part 1, O1 light switch (UI), V13 parts 1 and 2 (import and grow, add to the campaign, read back and age with write-back), V6 Cutaway and Body globe (Body screen), phone screen menu done; next U7 biosphere (waits on the kernel's `life_view`), browser campaign save (waits on `export_chronicle`), V7 | `main` via PRs |

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
