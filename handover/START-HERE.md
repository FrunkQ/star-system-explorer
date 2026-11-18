# START HERE

Last updated: 18 Nov 2026, after F1 and G0 (physics and kernel tracks) and U0 to U3, V1, U5 (UI track). Milestone M1 complete (K0 documents with Alex for review). Milestone 2 running as parallel tracks.

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
| F2 Classic Accrete block | next on the physics track (System Lab source on Alex's PC) | |
| K7 envelopes, partitioning, resolve, feasibility | done (kernel track) | packages/K7.md |
| K8 rules, classification, sifters, significance | done (kernel track) | packages/K8.md |
| K9 disclosure tiers, data views, scene building | done (kernel track) | packages/K9.md |
| K10 suite socket and coordinator | done (kernel track) | packages/K10.md |
| U0 one client API, every edit an intent (rule for all U packages) | done | packages/U0.md |
| U1 shell, chart room and night chart tokens, chart grammar | done, screenshots in packages/U1-screens/ for Alex | packages/U1.md |
| U2 system chart editor | done: real base-physics values, drag and typed edits, undo, why | packages/U2.md |
| U3 flow editor, explain lenses, change requests | done: real flows, kernel-validated drafts, adopt, badge; kernel gaps listed in the log | packages/U3.md |
| U4 time ruler with history, branches, look-ahead | waits for T2 | |
| V1 view contract, player scenes, Chart, Card, Table, Text, Readout | done (scenes from K9 fixtures until the kernel serves a scene op) | packages/V1.md |
| U5 packs screen, screens composer | done; data views screen waits for the kernel `scene` op | packages/U5.md |
| G0 default Milky-Way-like patch envelope | done (kernel track) | packages/G0.md |
| Kernel API additions: stack adopt, manifest, scene, scan_toggle | done (kernel track) | packages/K6-additions.md |
| C1, C2 classification | next (kernel track) | |

## Parallel tracks (18 Nov, Alex: build in parallel, integrate later)
| Track | Packages | Branch |
|---|---|---|
| Physics | P1, P3, P4, then F0 to F2 and F6 | `claude/project-thread-h9y77o` (this file) |
| Kernel | K7 to K10 and G0 done; C1, C2 next | `claude/project-thread-dzkz4p` (PR #1 into the physics branch; physics branch merged in at e0a9701) |
| Chart room shell and player views | U0 to U3, V1, U5 done; next V2 themes or U5 data views once `scene` lands (U4 waits for T2) | `claude/project-thread-adgixl`, PR #2 into the physics branch (kept merged up to date with it) |

The kernel now registers the real atom library and loads the base-physics pack (see P1 log, integration notes). The kernel track has merged that in.

Kernel track integration points (K7 to K10):
- New crates only: `sse-resolve`, `sse-sift`, `sse-observe`, `sse-suite` (path dependencies; root `Cargo.toml` untouched). No K1 to K6 interface changed.
- Shared files touched: `crates/sse-wasm` (adds `engines_run()`), `scripts/build-harness.sh`, `tests/harness/{harness.js,index.html,run-node.mjs,run-browsers.mjs}` (the harness now also checks the four engine golden scenarios in Node and every browser).
- Kernel API additions (log `packages/K6-additions.md`): `stack.adopt` events when a stack is loaded into an open universe, plus `manifest`, `scene` and `scan_toggle` ops, and the `provides.other.observe` pack key.
- Not yet in the kernel message API (H9.1): `resolve`, `query_feasible`, `sift`, `subscribe`. Wire them when the first consumer needs them (U2 for resolve and scenes, U5/V1 for data views, U8 for sift); each engine already has a pure function and a cost class (`ops()`) ready.
- G0 (`sse-cosmos`, pack `packs/base-cosmos`): the default patch envelope F0 starts from (`SystemBirth`, fixture `tests/fixtures/cosmos/solar-patch.json`). It added three things to K7: broken power laws, given weights, and `CountSpec::Fill`. G0's log has a determinism note on JSON floats that applies to every track.
- Pack content keys these engines read: `provides.other.rules`, `.axes`, `.classes`, `.sifters` (K8). Data views and disclosure maps are plain serde types (K9); a pack key for them is for U5 to choose.

Each track keeps to its own crates and folders. Integration points are listed at the end of each package log. UI work stays in `packages/ui-contract`, `packages/skins`, `packages/chart-grammar`, `packages/views`, `packages/shell`, `tests/ui`; it does not touch `/crates`, and a missing kernel op is reported in the package log.

## How to resume
1. Read `/CLAUDE.md`, then this file, then the log of the last package.
2. `cargo test --workspace` must pass before you change anything.
3. Take the first package marked "next"; write its log in `packages/` as you go (evidence packet format: build plan section 2).
4. At the end of a package: `scripts/check.sh`, update this table, re-sync `design/`, commit as FrunkQ, push.

## Environment
Rust 1.97.0 (rust-toolchain.toml, wasm32 target), wasm-bindgen-cli 0.2.130, Node 22 (`npm install`), Playwright for browser checks. Install and run: see README. UI: Svelte 5.58, Vite 8.3; `npm run ui:check`, `npm run ui:test`, `scripts/build-shell.sh`, `node tests/ui/shell.e2e.mjs`.
