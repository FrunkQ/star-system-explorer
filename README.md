# Star System Explorer 4

A total rewrite of Star System Explorer: a worldbuilding and space RPG toolkit
whose physics is real, visible and editable. Everything in the universe is a
probability cloud that resolves deterministically when someone looks at it.

Status: early build. Milestone 1 ("proven core") is in progress.

## Layout

| Path | What |
|---|---|
| `crates/` | Rust kernel (deterministic): `sse-core` substrate, `sse-chronicle`, `sse-flow`, `sse-pack`, `sse-kernel` (message API), `sse-wasm`, `sse-cli` |
| `packages/` | TypeScript: contracts (generated types), kernel client, apps |
| `tests/golden/` | golden hashes that every device must reproduce |
| `tests/harness/` | cross-device hash check page (open in any browser) |
| `docs/adr/` | architecture decision records |
| `handover/` | build handover: design copies and progress log. Temporary; delete when no longer wanted |

## Install and run

Needs Rust (via rustup; the pinned toolchain installs itself), Node 22 or later, and once:

```
cargo install wasm-bindgen-cli --version 0.2.130 --locked
```

Then, from the repo root:

```
npm install
npm run harness        # builds everything and serves http://localhost:8080/
```

That page is what exists to run today: the cross-device check. It runs the
pinned maths, encoding and draw cases and every kernel API call through a Web
Worker, and says PASS when this browser matches the native results. Open it
on each of your devices.

To deploy it as a static site, run `npm run build` (writes `dist/`; installs Rust
and wasm-bindgen first if the machine lacks them) and publish `dist/` on any
static host. Cloudflare Workers Builds: build command `npm run build`, deploy
command `npx wrangler deploy` (`wrangler.jsonc` serves `dist/` as static assets),
non-production branch command `npx wrangler versions upload`.
Live at https://v4.starsystemx.com (Cloudflare Worker `star-system-explorer`, auto-deployed from
`claude/project-thread-h9y77o` until a `main` exists).

## Checks

```
cargo test --workspace                       # unit, property and golden tests
cargo run -p sse-cli --release -- harness    # print the cross-device digests
scripts/check.sh                             # fmt, clippy, all Rust tests
scripts/build-harness.sh && node tests/harness/run-node.mjs
npm test                                     # kernel contract through a Node worker
node tests/harness/run-browsers.mjs          # needs Playwright browsers
```

## Licence

Apache-2.0. See `LICENSE` and `NOTICE` (credit to FrunkQ must be kept).
