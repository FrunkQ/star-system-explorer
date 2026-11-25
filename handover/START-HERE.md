# START HERE

Last updated: 24 Nov 2026 (handover to a new Claude account). Live site v4.starsystemx.com; `main` is the default branch.

## New to this project? Read this section first

On 25 Nov 2026 Alex (GitHub `FrunkQ`, the owner) moves SSE4 to a new Claude account. The old account's
window closes at **7 pm London time on 25 Nov 2026** (19:00 UTC; Alex, 01:31 UTC 25 Nov); work still open then
is picked up from this repo. The old account's project chat,
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
- **Golden changes by area** (`tests/golden/` hashes change only on purpose, reason in the package log): see the golden lane rule below. PRs in different areas may merge in parallel; within one area, or for the shared core, one at a time.
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
| Fudge register / citations | `docs/fudges.md` / `docs/citations.md`; the Consensus run-through of every cited paper and its master list `handover/citations/` (README there) |
| Interface style, top-level UI | `handover/interface-style.md`, `handover/design/SSE4-top-level-ui-and-onboarding.md` |
| Old project memory | `handover/memory/` |
| Screenshots, mockups, trailer clips | Not in the repo. Alex's PC: `C:\Development\sse4-project-files\` (13 zips of the old project files `sse4/`, 24 Nov snapshot) |
| Alex's PC (AC-ProArt) | `C:\Development`: `sse4` checkout, `star-system-explorer-v2` (V3 source, code in `star-system-generator`), `sse4-real-saves` (real V3 saves), `sse4-trailer-gpu` (trailer clips) |
| Sister sites | Explorers `FrunkQ/starsystemx-creator-hub`; converter `FrunkQ/starsystem-convert` |


## Standing rulings of 24 Nov 2026 (afternoon and evening)

- **CI test lines, add only** (Alex 18:07 UTC: "yes - if it makes your job easier - do it!"): a builder may add lines to `.github/workflows/ci.yml` so its new tests run automatically. Never remove, skip or loosen a check. Confirmed as a standing rule on 25 Nov 02:00 UTC (Alex: "in future yes to 'any thread may add its own test line to ci.yml, nothing else'"): a thread may add only its own test line, with no need to ask. Every other CI workflow or merge-check edit still needs Alex's own yes.
- **Frozen physics for the golden** (Alex 18:07 UTC): the shared contract golden may run on a frozen copy of the base-physics pack, with the live pack's loading tested separately. The kernel thread builds this after the barycentre PR.
- **Golden lane (by area, 25 Nov, the coordinator's call on Alex's "split the golden by area"):** the contract golden is split.
  - **Areas.** New contract cases go in `tests/contract/areas/<area>.json`, never in `tests/contract/requests.json`. Each area file runs on a fresh kernel, starting with its own `load_stack` (`"$pack:base-physics"` and so on) and `open_universe`. Its golden is `tests/golden/areas/<area>.jsonl`. One area per owning thread; create the file if your area has none. Bless with `SSE_BLESS=1 cargo test -p sse-kernel contract`.
  - **Core.** `requests.json` (closed at 380 cases, `CORE_CASES` in `crates/sse-kernel/src/tests.rs`; kernel PR 536) and `tests/golden/contract.jsonl` are closed: the test `the_core_sequence_is_closed` fails on an appended case. Their lines still move when an op's output really changes (a physics change, a new field), and that is a core change: one at a time, in the coordinator's order.
  - **Parallel.** A PR that changes only its own area's golden (and its own crate's goldens, such as `nav.json`) may merge as soon as it is green, in parallel with other areas. After merging main it re-blesses; only its own area file should show in the diff. If core lines move too, it queues in the core lane.
  - **Frozen physics.** The contract runs on the frozen `tests/fixtures/packs/base-physics.json`, so editing the live `packs/base-physics` text does not move it. Formula blocks are compiled in, though, so a changed or new formula still moves the manifest and stack hashes in core.
  - **Masked lines.** The op lists (hello, unknown op) are masked and checked against `ops()`. The reach report's lists are one line per block in `tests/golden/contract-reach.txt`.
  - **Areas so far:** `kernel`, `small-bodies`. Map of areas to threads: `handover/packages/GS2-golden-split-part-two.md`.
- **Data sheet UI copy:** after any change to the data sheet or `scripts/build-ui-reference.py`, merge main, run `python3 scripts/build-ui-reference.py`, then `--check`, before merging.
- **"Call me if there is a stop"** (Alex): if a stage is blocked on him, ask him; don't wait silently.
- **Figures go through one door (ED1a, accepted; check lands with the editing thread's PR):** a screen never formats its own numbers. Use `packages/ui-contract/src/figures.ts` (`say`, `sayText`, `sayFact`, `sayNumber`, `roundTo`). `tests/ui/figures-ledger.test.ts` counts `toFixed`, `toPrecision` and `toLocaleString` per file under views, shell, chart-grammar, ui-contract and renderer against `tests/ui/figures-ledger.json`: a file's count may go down, never up, and a new file starts at zero. After removing calls, rewrite the ledger with `WRITE_LEDGER=1 node --test tests/ui/figures-ledger.test.ts`.
- **Consensus for research** (Alex 18:47 UTC): the Consensus research connector is there for deep research; on the new account Claude has it directly, so use it yourself rather than writing briefs for Alex.
- Other rulings of the day (Composite band, Hazard map, seas, pulsar glitches, CoolProp only in scratch with his yes) are in `handover/memory/sse4-rulings-24nov-evening.md`.

## Standing rule: real physics, documented fudges (Alex, 19 Nov 2026)
In his words: "the physics is WAY more interesting than faking it. But understand a browser has limited capability. Fall on physics rather than simplification where practical. If we decide to "fudge" it is a known and documented fudge with a damn fine reason to exist."

- Every deliberate simplification, stand-in or tuned constant goes in `docs/fudges.md` (where it lives, why it exists, what real physics would replace it) in the same PR that adds it.
- Every reference, published relation or quoted constant goes in `docs/citations.md` with its check status (verified, corrected, unverified). Check a relation against its source before relying on it.
- **Master citation list (from PR 471, thread "Verify every cited paper"):** a new citation in a pack or in the code fails the checks until it is added to the master list with `python3 scripts/citations.py --add`. The list (about 1,475 papers, tiered, with statuses) and how papers are verified (with Consensus, which the new account has directly) are in `handover/citations/README.md`. Still also add the citation to `docs/citations.md` with its status.

## Standing rule: physics in the chain blocks, each with a view (Alex, 19 Nov 2026)
In his words: "try and do everything in the physics chain blocks... other than Big Blocks. And have a UI element that makes sense to visualise".

- New physics goes into the editable, cited flows of small blocks (atoms and boxes) by default.
- "Big Blocks" (heavy engine or simulator blocks: the classifier, weather, geology, stellar evolution and the like) only for what truly cannot be expressed as a flow, and each says why in `docs/fudges.md`.
- Each block or Big Block has a UI element that makes sense to visualise what it does.
- Logic hidden in Rust that could be a flow is listed in `docs/fudges.md` (section "Candidates to move into flows") until it moves.

## Banked ideas (not accepted, nothing queued)

Design notes kept for the future. They are **not** rulings and no builder starts on them without Alex's own yes.

- `design/SSE4-outside-models-design.md` (track OM, banked 24 Nov 2026): a scientist's own model feeding a physics flow through recorded answers, and the kernel as a published module. Alex: "find a proper use case first". Revisit when a real use case appears.

## Where we are

Since 23 Nov 2026 the state of each track is its own small file in `handover/status/`, edited only by the thread that owns it (Alex's yes to question 5 of `design/SSE4-coming-together.md`). This page is the index and the standing rules. **Do not add package rows here.**

The order in which tracks are joined up into the app is the six stages of `design/SSE4-coming-together.md` (accepted 23 Nov): 1 test drive, 2 one system told in full, 3 one language underneath, 4 the galaxy has a past, 5 run a game, 6 show it. Its section 4 gives one lane to each change that touches many blocks; read it before a sweep or a pull request that changes golden hashes.

| Status file | Owner thread (24 Nov 2026, 18:15 UTC) | Covers |
|---|---|---|
| `status/kernel.md` | Build kernel resolve and rules | K, J, C, T1, T2, G, L, E, the kernel side of Y, TG, GH, DM2 to DM5, ND3, ND3b, ND6, ED kernel asks K1 to K17, the golden split |
| `status/kernel-handoffs.md` | Build galaxy arm fix and atmosphere advice | kernel packages handed off by the kernel thread: arm sign fix, GH3 leading arm, ATMOS, V3 bary nodes on import |
| `status/proven-core.md` | Build SSE4 proven core | P, F, S, SF, B, T3, S5, the source side of MF (MF1 to MF3, MF10), MP19, MP20, ND1 |
| `status/chart-room.md` | Make the SSE4 test drive work end to end (was "Rebuild the chart room's top level") | U (not U6, U8), V (not V8, V9), N, the screen side of MP10, MF14b, ND5, ND7 |
| `status/navigation.md` | Build object selector and menus (now "Build look transitions for screens") | NV (selector part 2, object menu, find by tag, notes); AT15a screen delivery (how a screen arrives and leaves) |
| `status/run-desk.md` | Build Run desk playbook and players' side | the players' page, `shell/src/run/`, Bookmarks, Playbook and Switchboard (RD, PB) |
| `status/landing.md` | Build landing run-through (from the chart room) | LR: the landing run-through milestone, the whole journey as one browser check |
| `status/editing.md` | Build planet and star editing | ED (handles, strips, chart dragging, the units engine, pairs, the Solar System built through the UI) |
| `status/galaxy-and-travel.md` | Build galaxy charts and travel screens | V8, voyages, the flight log, travel panels, HW |
| `status/galaxy.md` | Build galaxy charts and histories (split from travel) | U6, U8, V9 (not the 3D web cube), GH8, V3 gaps 17 and 39 |
| `status/cosmic-web.md` | Build cosmic web view | the web cube in 3D and True, light from haloes (V9w follow-ups) |
| `status/light.md` | Build light and nebulae | O, the light side of Y, MF6, MF7, MF9, ND2 |
| `status/weather.md` | Build shared climate and weather | W, seas (colour, waves, whitecaps) |
| `status/data-sheet.md` | Build the master matter data sheet | MP data waves, MP22, SOL, DM1, DM6, RK1, RK2, RK10, citation checks |
| `status/renderer.md` | Build photoreal renderer | R, the drawing side of Y5 and MF14a, V14 |
| `status/quality-tiers.md` | Build quality tiers and speed probe (split from the renderer) | VP4, VP5: the `gpu-ultra` tier, the speed probe and placement by measured speed, heat watch, Quality choice, drawing bench rows; recipes and the governor on every True view; sharpen at rest |
| `status/detail-ladder.md` | Build detail ladder and world textures (split from the renderer) | VP6 the detail ladder, VP3 texture ladder and relief normals |
| `status/star-look.md` | Build star glow and flares | SG (glow past the edge, churn, lumpy giants, flares, ejections) |
| `status/aurora.md` | Build aurorae and night lights | the drawing side of VP-AU (aurora shells, curtains, ground view, bands, airglow drawn), VP12 (city lights, lightning), construct and drive lights; facts stay with the light thread (MF9 part 2) |
| `status/exotic-objects.md` | Build exotic stars and black holes (design: "Design exotic stars and black holes") | XO (light bending, discs, pulsars, stripped stars, Composite band, Hazard map, pulsating stars, ship clocks); XO is a code only, never shown in the product |
| `status/true-air.md` | Build giant bands, clouds and hazes | TA, third V3 audit R1 to R8, R2b part 2 |
| `status/true-surface.md` | Build plumes, frosts and hot glow (was "... and small bodies") | third V3 audit items 4, 5, 6, 10: plumes, coloured frosts, hot glow, small bodies and comets, lava tube openings |
| `status/small-bodies.md` | Build asteroids and rings up close (took small bodies from the plumes thread) | SB: belt and ring members resolved up close, notable bodies (a shattered world's core), small body kinds for the Add list, comets whole (takes VP-P3), VP-R1 to VP-R5 |
| `status/surface-timelapse.md` | Build surface time-lapse (R6 part 2 from the renderer) | R6 time-lapse, the drawing side of W-sea2 to W-sea4 |
| `status/names-and-life.md` | Build names gallery and life board | rest of NM6 (Classes, names gallery), U7 Life board |
| `status/naming.md` | no live owner (NM design done) | NM |
| `status/agents.md` | Build Orac, Minerva and Tars | A, ND4, ND5, ND9, SV7, AT11, AT16 |
| `status/traveller.md` | Build Traveller pack | X3, TG8, TG9, TG11, TG13, MP16 |
| `status/manifest.md` | Build ship manifest and crew roster | MC |
| `status/atomic-rockets.md` | Build ship flow and power views | RK13 Flow view and Power card, RK12 rings, RK10, RK3 ship sizing |
| `status/paper-spacecraft.md` | Build Paper spacecraft pack | RK6 (plan: Atomic Rockets plan section 6) |
| `status/citations.md` | Verify every cited paper | CR: the Consensus check of every cited paper, the master list in `handover/citations/` and its check |
| `status/spaceflight.md` | Build real spaceflight history pack | RS |
| `status/spaceflight-history.md` | none yet: to be opened as "Build roll call and In memory" (design: "Design spaceflight history view and memorial") | SH: Roll call (who was in space on any day), the count on the ruler, In memory, the real memorials where they are; accepted 25 Nov; SH is a code only, never shown in the product |
| `status/performance.md` | no live owner (thread finished) | PF |
| `status/atlas.md` | Build star atlas and editions (design: "Design star atlas and reports") | AT: the Atlas view, editions, print, margin notes, letterhead and ASCII art, interface kits and the four starters; AT is a code only, never shown in the product |
| `status/postcards.md` | Build postcards and hold backdrops (design: "Design saved views and hold backdrops") | PC: postcards (a bookmark that can travel and is drawn live as a still, with the reasons on its back), grounds and quotes on hold screens. ACCEPTED by Alex 25 Nov 01:27 UTC; builder thread "Build postcards and hold backdrops" (Opus) owns PC1, PC3, PC4, PC6; PC is a code only |
| `status/from-here.md` | none yet: proposed as "Build views and sounds from inside worlds" (design: "Design views and sounds from inside worlds") | FH: the standing view at any depth in any fluid (down through a gas giant), the "As lit" exposure, and the Aids (range markers, colour card, light meter, Listen). PROPOSED 25 Nov 2026, not yet accepted: 14 questions in the note's section 17; FH is a code only |
| `status/across-tracks.md` | each row names its owners | umbrella rows for MF, Y, GH, ND, TG, the converter module |

Added after 18:15 UTC: "Verify every cited paper" (`status/citations.md`, PR 471; master citation list, daily Consensus batches), "Build quality tiers and speed probe" (`status/quality-tiers.md`, PR 470), "Build detail ladder and world textures" (`status/detail-ladder.md`, PR 467), "Build aurorae and night lights" (`status/aurora.md`, PR 469). Designs: "Design visual polish and flourishes" (accepted, note in `handover/design/`); the exotic objects note's section 15 adds pulsating stars (XO19) and clocks (XO20); outside models and the kernel as a module (track OM, PR 484) is banked, not accepted.

Design (Fable) threads take design questions for their notes: "Bring Atomic Rockets into SSE4" (RK), "Design exotic stars and black holes" (XO). Every status file named here is on main (25 Nov 11:30 UTC); `handover/status-on-open-prs.md` says which PR brought each. Threads marked "none yet" were asked to start a file.

A package that does not fit a file goes in its owner thread's file. A new track gets a new file and one line in the table above. Milestone M1 "Proven core" (K0 to K6) is complete.

**Since 19 Nov (Alex): `main` is the default branch.** New work branches from `main` and arrives as a pull request
into `main`; its checks pass before merge. The old track branches are kept, not deleted.

Each track keeps to its own crates and folders. Integration points are listed at the end of each package log. UI work stays in `packages/ui-contract`, `packages/skins`, `packages/chart-grammar`, `packages/views`, `packages/shell`, `tests/ui`; it does not touch `/crates`, and a missing kernel op is reported in the package log.

## Deployment
https://v4.starsystemx.com: Cloudflare Worker `star-system-explorer` (Workers Builds, set up by Alex 18 Nov 2026).
Build `npm run build` (scripts/build-site.sh, writes dist/), deploy `npx wrangler deploy` (wrangler.jsonc),
production branch: switch to `main` in Cloudflare (was `claude/project-thread-h9y77o`); other branches get preview versions.
There is no deploy workflow in GitHub Actions: `ci.yml` only tests (and never cancels runs on `main`). Cloudflare
builds every merge to `main` (4 to 10 minutes each). When merges come faster than that, a waiting build can show as
Skipped, but the next build on `main` contains it, so the site catches up. Alex checked on 25 Nov 01:36 UTC and found
every build Ready; nothing needs changing in Cloudflare and no GitHub deploy workflow is wanted.
The site is the GM shell (chart room) at `/`, the players page at `/players.html` and the cross-device check page at `/harness/`.

## How to resume
1. Read `/CLAUDE.md`, then this file, then the log of the last package. Any view or renderer package also follows `rendering-rules.md` (no precision jiggle: local orbits, nested frames, camera-relative numbers).
2. `cargo test --workspace` must pass before you change anything.
3. Open your track's file in `status/` and take the first package marked "next"; write its log in `packages/` as you go (evidence packet format: build plan section 2).
   Screenshots are not committed (they had grown to 73 MB of the repo). Write them to `handover/packages/<PKG>-screens/` as before, which git now ignores, and copy that folder to the project files at `sse4/handover-screens/<PKG>-screens/`, where every earlier package's screenshots now live. Links in package logs such as `U16-screens/` mean that folder.
4. At the end of a package: `scripts/check.sh`, update your track's file in `status/`, re-sync `design/`, commit as FrunkQ, push.

## Environment
Rust 1.97.0 (rust-toolchain.toml, wasm32 target), wasm-bindgen-cli 0.2.130, Node 22 (`npm install`), Playwright for browser checks. Install and run: see README. UI: Svelte 5.58, Vite 8.3; `npm run ui:check`, `npm run ui:test`, `scripts/build-shell.sh`, `node tests/ui/shell.e2e.mjs`.
