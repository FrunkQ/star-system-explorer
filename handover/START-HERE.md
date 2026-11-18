# START HERE

Last updated: 18 Nov 2026, after F2 and the kernel track's L0. Milestone M1 complete (K0 documents with Alex for review). Milestone 2 running as parallel tracks.

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
| P1 atom library, first wave | done | packages/P1.md |
| P3 V3 importer | built; real-save check pending on Alex's PC | packages/P3.md |
| P4 regions and features on worlds | done | packages/P4.md |
| F0 inheritance, stars, life lines, formation history | done | packages/F0.md |
| F1 disc and material bands | done | packages/F1.md |
| F2 Classic Accrete block | done | packages/F2.md |
| F6 early worlds and hand-off | done (ahead of F2 to F5; reads a plain `Protoplanet`) | packages/F6.md |
| K7 envelopes, partitioning, resolve, feasibility | done (kernel track) | packages/K7.md |
| K8 rules, classification, sifters, significance | done (kernel track) | packages/K8.md |
| K9 disclosure tiers, data views, scene building | done (kernel track) | packages/K9.md |
| K10 suite socket and coordinator | done (kernel track) | packages/K10.md |
| G0 default Milky-Way-like patch envelope | done (kernel track) | packages/G0.md |
| Kernel API additions: stack adopt, manifest, scene, scan_toggle | done (kernel track) | packages/K6-additions.md |
| Numbers: exact float parse, JSON numbers decode by value (cross-track) | done (kernel track); other tracks merge and re-baseline | packages/numbers.md |
| C1 axis registry, class records, schemes, SSE 3 classic | done (kernel track) | packages/C1.md |
| C2 base physical atlas, habitability, gas mixes | done (kernel track); thresholds and hycean open for Alex's review | packages/C2.md |
| L0 life suite sockets: interfaces, habitat graphs, empty suite | done (kernel track) | packages/L0.md |
| C3 reverse placement in a system | next (kernel track) | |

## Parallel tracks (18 Nov, Alex: build in parallel, integrate later)
| Track | Packages | Branch |
|---|---|---|
| Physics | P1, P3, P4, F0 to F2 and F6 done; F3 to F5 and F7 not started (await assignment) | `claude/project-thread-h9y77o` (this file) |
| Kernel | K7 to K10, G0, C1, C2, L0 and the U3 gaps done; C3 next | `claude/project-thread-dzkz4p` (PR #1 into the physics branch; physics branch merged in at 788aea1) |
| Chart room shell | U1 | its own branch from the M1 state; see that thread |

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
- Pack content keys these engines read: `provides.other.rules`, `.axes`, `.classes`, `.sifters` (K8). Data views and disclosure maps are plain serde types (K9); a pack key for them is for U5 to choose.

Each track keeps to its own crates and folders. Integration points are listed at the end of each package log.

## How to resume
1. Read `/CLAUDE.md`, then this file, then the log of the last package.
2. `cargo test --workspace` must pass before you change anything.
3. Take the first package marked "next"; write its log in `packages/` as you go (evidence packet format: build plan section 2).
4. At the end of a package: `scripts/check.sh`, update this table, re-sync `design/`, commit as FrunkQ, push.

## Environment
Rust 1.97.0 (rust-toolchain.toml, wasm32 target), wasm-bindgen-cli 0.2.130, Node 22 (`npm install`), Playwright for browser checks. Install and run: see README.
