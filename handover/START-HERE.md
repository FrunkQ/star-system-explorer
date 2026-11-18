# START HERE

Last updated: 18 Nov 2026, after P1. Milestone M1 complete (K0 documents with Alex for review). Milestone 2 running as parallel tracks.

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
| P4 regions and features on worlds | next on the physics track | |

## Parallel tracks (18 Nov, Alex: build in parallel, integrate later)
| Track | Packages | Branch |
|---|---|---|
| Physics | P1, P3, P4 | `claude/project-thread-h9y77o` (this file) |
| Kernel | K7 to K10 | its own branch from the M1 state; see that thread |
| Chart room shell | U1 | its own branch from the M1 state; see that thread |

Each track keeps to its own crates and folders. Integration points are listed at the end of each package log.

## How to resume
1. Read `/CLAUDE.md`, then this file, then the log of the last package.
2. `cargo test --workspace` must pass before you change anything.
3. Take the first package marked "next"; write its log in `packages/` as you go (evidence packet format: build plan section 2).
4. At the end of a package: `scripts/check.sh`, update this table, re-sync `design/`, commit as FrunkQ, push.

## Environment
Rust 1.97.0 (rust-toolchain.toml, wasm32 target), wasm-bindgen-cli 0.2.130, Node 22 (`npm install`), Playwright for browser checks. Install and run: see README.
