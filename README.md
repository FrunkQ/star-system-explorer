# Star System Explorer 4

A total rewrite of Star System Explorer: a worldbuilding and space RPG toolkit
whose physics is real, visible and editable. Everything in the universe is a
probability cloud that resolves deterministically when someone looks at it.

Status: early build. Milestone 1 ("proven core") is in progress.

## Layout

| Path | What |
|---|---|
| `crates/` | Rust kernel (deterministic): `sse-core` substrate, and later chronicle, flows, packs, kernel API, wasm surface, CLI |
| `packages/` | TypeScript: contracts (generated types), kernel client, apps |
| `tests/golden/` | golden hashes that every device must reproduce |
| `tests/harness/` | cross-device hash check page (open in any browser) |
| `docs/adr/` | architecture decision records |
| `handover/` | build handover: design copies and progress log. Temporary; delete when no longer wanted |

## Checks

```
cargo test --workspace                       # unit, property and golden tests
cargo run -p sse-cli --release -- harness    # print the cross-device digests
scripts/build-harness.sh && node tests/harness/run-node.mjs
node tests/harness/run-browsers.mjs          # needs Playwright browsers
```

## Licence

Apache-2.0. See `LICENSE` and `NOTICE` (credit to FrunkQ must be kept).
