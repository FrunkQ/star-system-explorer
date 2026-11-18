# START HERE

Last updated: 18 Nov 2026, after U0 and U1. Milestone M1 complete, pending Alex's review of the K0 documents. M2 runs as parallel tracks (see Branches).

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
| U0 one client API, every edit an intent (rule for all U packages) | done | packages/U0.md |
| U1 shell, chart room and night chart tokens, chart grammar | done, screenshots in packages/U1-screens/ for Alex | packages/U1.md |
| U2 system chart editor | next on the UI track; needs P1 | |
| M2 "One system": P1, P3, P4, U2, G0, P2 (build plan section 4) | in progress on parallel tracks | |

## Branches (parallel tracks from the M1 commit 3461036)
| Branch | Holds |
|---|---|
| `claude/project-thread-h9y77o` | M1 (K0 to K6); base for the tracks below |
| `claude/project-thread-adgixl` | UI track: U0, U1 (pull request against the M1 branch) |
| other track branches | physics atoms (P1, P3, P4) and kernel K7 to K10, from their own threads |

UI work stays in `packages/ui-contract`, `packages/skins`, `packages/chart-grammar`, `packages/views`, `packages/shell`, `tests/ui`. It does not touch `/crates`; if the client API needs a kernel op, say so in the package log.

## How to resume
1. Read `/CLAUDE.md`, then this file, then the log of the last package.
2. `cargo test --workspace` must pass before you change anything.
3. Take the first package marked "next"; write its log in `packages/` as you go (evidence packet format: build plan section 2).
4. At the end of a package: `scripts/check.sh`, update this table, re-sync `design/`, commit as FrunkQ, push.

## Environment
Rust 1.97.0 (rust-toolchain.toml, wasm32 target), wasm-bindgen-cli 0.2.130, Node 22 (`npm install`), Playwright for browser checks. Install and run: see README. UI: Svelte 5.58, Vite 8.3; `npm run ui:check`, `npm run ui:test`, `scripts/build-shell.sh`, `node tests/ui/shell.e2e.mjs`.
