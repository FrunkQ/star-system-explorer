# START HERE

Last updated: 24 Nov 2026 (handover to a new Claude account). Live site v4.starsystemx.com; `main` is the default branch.

## New to this project? Read this section first

On 25 Nov 2026 Alex (GitHub `FrunkQ`, the owner) moves SSE4 to a new Claude account. The old account's project chat,
threads and project memory do not come with it. **Everything a new Claude needs is in this repo**, mainly in `handover/`.
Read in this order:

1. `/CLAUDE.md` (repo rules), then the rest of this page.
2. `handover/memory/MEMORY.md`: the old project memory index, copied in full on 24 Nov 2026. The other files in
   `handover/memory/` are the detail memories it points to (one subject each). Treat them as dated notes: check
   anything that names a file, PR or function against main before relying on it.
3. `handover/handover-2026-11-24.md`: work streams, per-track status checked against main and open PRs, the golden
   lane queue, open questions for Alex, known caveats.
4. Your track's file in `handover/status/`, then the log of its last package in `handover/packages/`.

### What SSE4 is (one paragraph)
Star System Explorer 4: a total rewrite of Alex's Star System Explorer (V3 is live at starsystemx.com and is the one
for real use; SSE4 is ALPHA at v4.starsystemx.com). A procedural, deterministic universe from Big Bang to heat death
whose physics is real, visible and editable as cited node flows ("lean into the physics, never fake it"). An RPG and
storytelling toolkit first, astrophysics education second. Kernel in Rust (compiled to wasm), UI in TypeScript and
Svelte 5. Apache-2.0. Agents (Orac designer, Minerva storytelling, Tars support) are bring-your-own-key; keys never
leave the browser.

### How Alex wants work done (standing rules)
- **Design and mockups on Fable** (limited weekly allowance; never used for code). **Coding on Opus.**
- **One builder thread per screen section** (chart room, object selector and menus, Run desk and players' side,
  planet and star editing, ...) so contexts stay small and threads do not collide. `shell/src/` is split into section
  folders for this.
- **PRs into `main` merge when their checks pass** ("main is entirely yours"). Every merge deploys the live site.
- **CI workflow or merge-check edits need Alex's own yes.** So do database, storage or user-data changes on the
  Explorers site (repo `FrunkQ/starsystemx-creator-hub`). The converter (`FrunkQ/starsystem-convert`) merges when green.
- **One golden-changing PR open at a time** (`tests/golden/` hashes change only on purpose, reason in the package log).
- **No temporary files in the repo**: screenshots and working artefacts go to project files
  (`sse4/handover-screens/<PKG>-screens/`), never committed. **No history rewrites** (old screenshots stay in history).
- **Questions as one numbered list**, each with Claude's own recommendation; Alex answers like "yes to 1, 2 and 4".
  At most one list a day unless something cannot be undone or blocks a stage. No release-date pressure.
- One work package at a time; its checks (`scripts/check.sh`) pass before the next starts. Commits as
  `FrunkQ <frunk@frunk.net>`.
- One general mechanism over specials; don't copy V3, do it better; "think wide". Physics over simplification where
  practical, every fudge in the register. Interface is "dynamic minimalism": common things a click away, rare things
  in menus with context, newcomer friendly; guide the user, never leave them alone.

### Where everything lives
| What | Where |
|---|---|
| Repo rules | `/CLAUDE.md`, `clippy.toml`, `scripts/check.sh` |
| Design notes (accepted unless marked) | `handover/design/SSE4-*.md`; start with `SSE4-build-plan.md` sections 1 and 2; authority `SSE4-design-handoff.md`; join-up order `SSE4-coming-together.md` |
| Rulings during the build | `handover/decisions.md`, plus the "Standing rulings" in `handover/memory/MEMORY.md` |
| Track status | `handover/status/<track>.md` (table below) |
| Package logs and evidence | `handover/packages/<PKG>.md` |
| Fudge register / citations | `docs/fudges.md` / `docs/citations.md` |
| Interface style, top-level UI | `handover/interface-style.md`, `handover/design/SSE4-top-level-ui-and-onboarding.md` |
| Old project memory | `handover/memory/` |
| Screenshots, mockups, trailer clips | Not in the repo. Old account's project files `sse4/` (see the handover note for the list); Alex's PC |
| Alex's PC (AC-ProArt) | `C:\Development`: `sse4` checkout, `star-system-explorer-v2` (V3 source, code in `star-system-generator`), `sse4-real-saves` (real V3 saves), `sse4-trailer-gpu` (trailer clips) |
| Sister sites | Explorers `FrunkQ/starsystemx-creator-hub`; converter `FrunkQ/starsystem-convert` |


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

Since 23 Nov 2026 the state of each track is its own small file in `handover/status/`, edited only by the thread that owns it (Alex's yes to question 5 of `design/SSE4-coming-together.md`). This page is the index and the standing rules. **Do not add package rows here.**

The order in which tracks are joined up into the app is the six stages of `design/SSE4-coming-together.md` (accepted 23 Nov): 1 test drive, 2 one system told in full, 3 one language underneath, 4 the galaxy has a past, 5 run a game, 6 show it. Its section 4 gives one lane to each change that touches many blocks; read it before a sweep or a pull request that changes golden hashes.

| Status file | Owner thread | Covers |
|---|---|---|
| `status/kernel.md` | Build kernel resolve and rules | K, J, C, T1, T2, G, L, E, the kernel side of Y, TG, GH, DM2 to DM5, ND3, ND3b, ND6 |
| `status/proven-core.md` | Build SSE4 proven core | P, F, S, SF, B, T3, S5, the source side of MF (MF1 to MF3, MF10), MP19, MP20, ND1 |
| `status/chart-room.md` | Make the SSE4 test drive work end to end (was "Rebuild the chart room's top level") | U (not U6, U8), V (not V8, V9), N, the screen side of MP10, MF14b, ND5, ND7 |
| `status/galaxy-and-travel.md` | Build galaxy charts and travel screens | U6, U8, V8, V9, voyages, GH8 |
| `status/light.md` | Build light and nebulae | O, the light side of Y, MF6, MF7, MF9, ND2 |
| `status/weather.md` | Build shared climate and weather | W |
| `status/data-sheet.md` | Build the master matter data sheet | MP data waves, MP22, SOL, DM1, DM6, citation checks |
| `status/renderer.md` | Build photoreal renderer | R, the drawing side of Y5 and MF14a |
| `status/agents.md` | Build Orac, Minerva and Tars | A, ND4, ND5, ND9 |
| `status/traveller.md` | Build Traveller pack | X3a to X3d, TG8, TG9, TG11, TG13, MP16 |
| `status/manifest.md` | Build ship manifest and crew roster | MC (ship manifest and crew roster) |
| `status/naming.md` | the thread that owns track NM | NM |
| `status/performance.md` | Measure performance for minimum specs | PF |
| `status/across-tracks.md` | each row names its owners | the umbrella rows for MF, Y, GH, ND, TG, and the converter module |
| `status/spaceflight.md` | Build real spaceflight history pack | RS |
| `status/navigation.md` | Build object selector and menus | NV (finding your way: selector part 2, object menu, find by tag, notes) |
| `status/exotic-objects.md` | no builder thread yet (design by "Design exotic stars and black holes") | XO (black holes with real light bending, pulsars, stripped stars, Composite band, the approach diagram) |
| `status/editing.md` | Build planet and star editing | ED (handles, strips, chart dragging, the units engine, pairs as edited, the Solar System built through the UI) |
| `status/surface-timelapse.md` | Build surface time-lapse | R6 part 2 (the globe's time-lapse from surface_frames), the drawing side of W-sea2 to W-sea4 |
| `status/run-desk.md` (arrives with PR 421) | Build Run desk playbook and players' side | RD, PB (Playbook and Switchboard) |
| none yet | Build Paper spacecraft pack | RK6 (other RK packages are rows in the files above) |
| none yet | designed only (XO note, PR 420) | XO exotic objects |

A package that does not fit a file goes in its owner thread's file. A new track gets a new file and one line in the table above. Milestone M1 "Proven core" (K0 to K6) is complete.

**Since 19 Nov (Alex): `main` is the default branch.** New work branches from `main` and arrives as a pull request
into `main`; its checks pass before merge. The old track branches are kept, not deleted.

Each track keeps to its own crates and folders. Integration points are listed at the end of each package log. UI work stays in `packages/ui-contract`, `packages/skins`, `packages/chart-grammar`, `packages/views`, `packages/shell`, `tests/ui`; it does not touch `/crates`, and a missing kernel op is reported in the package log.

## Deployment
https://v4.starsystemx.com: Cloudflare Worker `star-system-explorer` (Workers Builds, set up by Alex 18 Nov 2026).
Build `npm run build` (scripts/build-site.sh, writes dist/), deploy `npx wrangler deploy` (wrangler.jsonc),
production branch: switch to `main` in Cloudflare (was `claude/project-thread-h9y77o`); other branches get preview versions.
The site is the GM shell (chart room) at `/`, the players page at `/players.html` and the cross-device check page at `/harness/`.

## How to resume
1. Read `/CLAUDE.md`, then this file, then the log of the last package. Any view or renderer package also follows `rendering-rules.md` (no precision jiggle: local orbits, nested frames, camera-relative numbers).
2. `cargo test --workspace` must pass before you change anything.
3. Open your track's file in `status/` and take the first package marked "next"; write its log in `packages/` as you go (evidence packet format: build plan section 2).
   Screenshots are not committed (they had grown to 73 MB of the repo). Write them to `handover/packages/<PKG>-screens/` as before, which git now ignores, and copy that folder to the project files at `sse4/handover-screens/<PKG>-screens/`, where every earlier package's screenshots now live. Links in package logs such as `U16-screens/` mean that folder.
4. At the end of a package: `scripts/check.sh`, update your track's file in `status/`, re-sync `design/`, commit as FrunkQ, push.

## Environment
Rust 1.97.0 (rust-toolchain.toml, wasm32 target), wasm-bindgen-cli 0.2.130, Node 22 (`npm install`), Playwright for browser checks. Install and run: see README. UI: Svelte 5.58, Vite 8.3; `npm run ui:check`, `npm run ui:test`, `scripts/build-shell.sh`, `node tests/ui/shell.e2e.mjs`.
