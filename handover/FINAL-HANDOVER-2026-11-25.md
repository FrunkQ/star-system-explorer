# Final handover, 25 Nov 2026 (old account closing)

This file was written on 25 Nov 2026 around 23:00 UTC, as the old account's usage ran out. Work on the old account
was frozen at that point, and the project moves to a new account with the repo and Alex's PC. Start from
`handover/START-HERE.md`. This file records what was in flight at the freeze.

## 1. Open PRs at the freeze (FrunkQ/star-system-explorer)

Checks are as read at 22:50 UTC. "Green" means every CI job passed or was skipped by the change filter.

| PR | Branch | What | Checks | Next step |
|---|---|---|---|---|
| 589 | claude/project-thread-ftsszw | Natural outlines and smooth tile edges (VP3, detail ladder) | green at df402ff; one more commit (Land map in measured colour) re-running at 45c55d3 | merge when green |
| 588 | claude/project-thread-r28d1d | Light track: Mie scattering, forbidden lines, line colours, light in a world's shadow | **ui job red** | fix the ui failure, then merge |
| 585 | claude/project-thread-iyae31 | W-synoptic: a day's clouds instead of the season's average (climate) | **merged** 26 Nov (eb4ad5c0) | renderer: raise AIR_N in `true-air.ts` from 24 to 48 for worlds with ground (64 at close-up), checking the cost on the speed probe; re-shoot Earth beside the Blue Marble |
| 584 | claude/project-thread-uz7sb8 | Editing: the Makeup box, class marks on the Mass strip | **merged** 22:49 (02a0e8a6) | done |
| 581 | claude/project-thread-z535je | GZ5 parts 1 and 1b: Galaxies nearby in 3D | no check runs on head | merge main into it so CI runs |
| 580 | claude/project-thread-rn77dw | PC6 part 1: postcard sets, base set's first cards, the script | **merged** 22:49 (02a6dcdf) | done |
| 559 | claude/project-thread-ri0k6m | XO1 part 2, XO18 parts 2 to 4, XO20: gravity blocks, hazard_map, Hazard map, clocks | no check runs on head | merge main into it so CI runs (golden: core lane) |
| 553 | claude/project-thread-5j6k6e | SH8 part 1: the spaceflight front door opens Sol in Roll call | no check runs on head | merge main into it so CI runs |
| 495 | claude/project-thread-7xs7xp | RK3: ship sizing as named Equations; ship_power radiators (golden) | **ui job red** (24 Nov) | merge main, rebless, fix ui |
| 452 | claude/project-thread-qax0fc | V8 and RK5: party ship logged and fuelled, players in step, slow spirals | **ui job red** | merge main, fix ui |

Branches with work and no PR: `l1e9pw` (renderer, V14i), `3jxtpq` (TRUE4 part 1b, ready), `7g5v90` (R6d, after 589), `bvlyq1` (Local Arm, WIP), `dzkz4p` (kernel K18 and GZ0, ready, area golden only), `h9y77o` (IG1 ice ground, WIP); all `claude/project-thread-*`.

## 2. Thread reports at the freeze (was doing, left, next)

Eleven threads were cut off by the billing lapse and asked to push and report. "No report received" means
take its state from its branch, its PR above and its `handover/status/` file.

- **Postcards and hold backdrops (PC)**: branch rn77dw, PR 580, green.
  - Doing: PC6 part 1. This is the sse.postcards/1 sets, the wall rows and held-screen grounds, and `packs/base-presentation/postcards.json` with 3 True cards, made by `scripts/build-postcard-set.mjs`. Alex said yes (13:22) to listing it in the matter audit's FIXTURE_FILES.
  - Left: 580 merged. Then PC6 part 2: about 24 cards with the trailer thread, a System 3D step, and the wall opening on a universe pack's own set first. PC8 (Explorers) needs Alex's yes.
  - Next: add places to PLACES in the script and rerun it after `scripts/build-shell.sh`. The galaxy, cosmic web and surface cards wait on the renderer's PC2 stills. Log `packages/PC6.md`, status `status/postcards.md`.
- **Detail ladder and world textures (VP3, VP6)**: branch ftsszw at 45c55d3, PR 589.
  - Doing: natural outlines (Alex: "a bit bitmap"). In ui-contract `globe.ts`: sampleNatural, softenGround, edgeNoise, naturalEdge, naturalDirection; smooth coarse tiles in True. PRs 540 and 572 merged today.
  - Left: merge 589. The time-lapse thread wires the same functions into `renderer/src/timelapse.ts`. The canvas-tier Body globe has no natural outlines yet.
  - Next: VP3 part 2b (clouds as their own shell), part 3 (finer tiles on Body), and roughness below the fact floor. That waits on kernel facts roughness_m, grain_m and crater_cover. Two kernel asks are with proven core: icy crusts made of ice, and young craters from SW1. Status `status/detail-ladder.md`, log `packages/VP3.md`.
- **Planet and star editing (ED)**: branch uz7sb8, PR 584, green.
  - Doing: PRs 542 to 574 merged (sheet and CSV, Could be, figures ledger down to 202, sweep behind the Orbit and Mass strips). 584 adds the Makeup box and class marks, and the sweep now reads at the campaign's now.
  - Left: 584 merged. Then ED9 part 2: the cutaway wedge (needs kernel K9 layers per step), dragging the bar's joins, and heat and redox strips. Then ED2 part 2b: pencil figures from sweep steps, and Reach it and Could be on the sweep.
  - Next: ED6 part 2, the star chooser, is unblocked: K18 describe_star merged in PR 592 (1c354fa0), though `status/editing.md` still says it waits on K18. Then ED8 part 2b (K7, K8), ED10 part 2 (K14), ED11, and ED0b, the Solar System test.
  - Gotcha: kernel ops that edit a scratch branch (sweep, expects) must pass at_s = the campaign's now. Otherwise a main-line edit at now, an Undo included, hides the swept value.
- **Photoreal renderer**: branch l1e9pw at 19511767, no open PR. PRs 567 (MATCH1 notes) and 577 (LIT1) merged.
  - Doing: V14i class gallery pictures from the kernel's class_sample world in True (`renderer/src/true-still.ts`, `views/reference/src/class-picture.svelte.ts`, session classSample, a Node test and a reference e2e). WIP commit, works locally.
  - Left: the kernel fix it waited on is on main (PR 592, 23:54 UTC). Merge main into the branch, run reference.e2e, and open the PR. Samples with no radius stay skipped until kernel K3. Then the MATCH1 re-shoot (after 572, 573, 417), full face and side-lit, labelled by tier and device, on Alex's GPU if he agrees.
  - Next: `status/renderer.md` Next items 2 to 8 (PC2 part 3 stills, VP2 Exposure in other views, nebula lines, lander lamps). After W-synoptic lands, raise AIR_N to 48 or 64 for worlds with ground.
- **Shared climate and weather (W)**: branch iyae31, PR 585, green at 462e52a9.
  - Doing: W-synoptic. Earth's clouds become weather systems: fronts, comma heads and dry slots, mid-level decks, trade cumulus, whole-cloud optical depths. It changes two core contract.jsonl lines (weather_read: the Titan field and the Earth report), so it waits for a core golden slot.
  - Left: merge main, re-bless (`SSE_BLESS=1 cargo test --workspace`), contract test and typecheck, push once, merge in the core slot. Then the renderer raises AIR_N in `true-air.ts` to 48 (64 at close-up) and re-shoots MATCH1.
  - Next: rain drop size in the report, eccentricity and libration tides, W7c part 2d, W8 part 3 (with S17a), W10 to W12 (`design/SSE4-weather-review.md`), MF8. See `status/weather.md`, `packages/W-synoptic.md`.
- **Light and nebulae**: branch r28d1d at fc7d3ea3, PR 588.
  - Doing: K-e Mie scattering (op read_scattering), O4 part 1 forbidden lines, VP-L1 line colours, VP-L3 light in a world's shadow (op read_shadow_light).
  - Left: one ui failure. `tests/ui/aurora.e2e.mjs:54` expects `/O 557\.7 nm at about 1\d\d km/`, but the caption now reads "[O I] 557.7 nm", so the regex must accept the bracketed name. Then merge main, re-bless (sse-atoms, sse-nebula, sse-kernel contract, release), push, and merge in a core slot.
  - Next: MF9 2c (aurora lead angle, Saur et al. 2013 A&A 552 A119), ND2 2c and 2d, RK16, O4 part 2. See `status/light.md`.
- **Plumes, frosts and hot glow (TRUE)**: branch 3jxtpq at b076225, no PR yet. PRs 449, 492, 541, 544 merged.
  - Doing: TRUE4 part 1b, commit 21ab6c16. Earth got a full tholin coat from 2 ppm methane, so this adds the Trainer et al. 2006 haze threshold (`THOLIN_MIN_CH4_PER_OX` 0.1) with a test, fudge row and citation. check.sh green, no golden diff.
  - Left: open the PR and merge when green. SO2 frost and sulphur have no colour yet; it is on the data sheet thread's queue (Hapke 1981, Gradie 1980, Moses and Nash 1991).
  - Next: TRUE4 part 2 per-tile stains; VP-P1, P2, P4 (wait on VP1, VP6, K-e); heat and plumes in Lineup3D; Gallery checks on Sol. See `status/true-surface.md`.
- **Surface time-lapse**: branch 7g5v90 at 31c34ad, no PR. PRs 436, 476, 579, 583 merged.
  - Doing: R6d, natural outlines in the time-lapse (`drawnCellOf` in `renderer/src/timelapse.ts`, using the shared naturalDirection). ui:test 1003/1003 locally.
  - Left: none. R6d merged as PR 593 (26 Nov, 3446d53).
  - Next: the globe's sea per climate band once W-sea2 to W-sea4 kernel numbers land (agree with the renderer). Parked by Alex: air and oceans that evolve through the frames.
- **Kernel hand-offs (Local Arm; galaxy arm fix and atmosphere advice)**: branch bvlyq1, WIP commit b3530b1f, no PR.
  - Doing: the home galaxy's arms from measured positions (Reid et al. 2019), Local Arm included, as one measured-arms list in `packs/base-cosmology`, plus chart arm_names. ATMOS check: the hydrogen_per_carbon ratio was upside down (fixed), and the escape constant is credited to Hunten and Strobel 1974.
  - Left (branch now f76116ce): the centre-over-anticentre glow check is fixed. The next check fails: the midplane lane is deeper in Ks (0.86) than in V (0.43), because the arms' young stars fill the V midplane. It needs a physics look at the young-star scale height and the dust under the arms, not tuning. The core slot was released. Then re-bless, check.sh, UI checks, and open the PR.
  - Next: core lane (galaxy chart, sky and glow lines move). Log `packages/GH-local-arm.md`, status `status/kernel-handoffs.md`.
- **Kernel resolve and rules**: branch dzkz4p at 08283765, no PR. PRs 468, 536, 546 merged.
  - Doing: two pushed commits, full local checks green. K18 describe_star (the editing star chooser); a fix for grown systems coming out 13,783 Gyr old (the galaxy's now in Myr read as Gyr, which caused the wasm out-of-memory); `systems` at any depth; class_sample addresses carrying branch and class (the renderer waits on this). GZ0 kernel checks: no campaign system gets a site event history today, and a halo's two places are a median 7.2 Mpc apart. Logs `packages/K18-describe-star.md`, `packages/GZ0-kernel-checks.md`.
  - Left: open the PR (merge main, re-bless). It moves only `tests/golden/areas/kernel.jsonl`, so no core slot is needed. Merge on green, then tell the editing, galaxy charts and renderer threads.
  - Next (`status/kernel.md` queue 2 and 2b): GZ12 (core slot), GZ13, GZ2, GZ17, GZ15, GZ14; then K16, K4, K12, K5, K7 to K11, K14; K3 for small bodies; life_view dials.
- **SSE4 proven core**: branch h9y77o, WIP head c2d6d0c4, no PR. PR 417 merged 13:18 UTC.
  - Doing: IG1 ice ground, three cited blocks in `packs/base-physics/formulas/surface.json` (from `scripts/build-weathering-blocks.py`; crate code in `sse-atoms/src/ice_ground.rs`, `sse-surface/src/ice_ground.rs`). Icy moons, Pluto and Triton get ice ground; the Moon and Ceres lose open ice; albedo comes from the sheet's optics. It also fixes the sol_tests helper that put every moon at body:0.
  - Left: Europa is too dark (0.32, resurfacing not wired); Titan needs measured air and temperature in base-sol; Io needs sulphur; remove the ignored zz_probe_ground test; fudge rows, log, status row, bless, full checks. Core golden slot (pack hash lines move).
  - Next: craters and basins (rim and floor relief, leading-side lopsiding, Zahnle 2001; oblique craters, Bottke 2000; young craters' dates for rays).

## 3. Questions waiting on Alex

1. Postcards question 1: may the universe seed travel with a sent postcard? Recommended yes.
2. Quality tiers: approve the thread's PC connection, so it can read `C:\Development\sse4\docs\perf-reports\ac-proart\sse4-bench-device.json`. Recommended yes.
3. Open "Build the three desks" (TM1) now, or on the new account? Recommended: on the new account.
4. RK8 (Atomic Rockets name, credit and links): waits on the author's written reply.
5. Trailer filming: paused by Alex. The daily re-shoot task is switched off (section 5).
6. PC8 (postcards into Explorers): needs Alex's own yes.
7. MATCH1 re-shoot on Alex's GPU: does he agree?
8. Older parked items, raised when work reaches them: class thresholds, hycean class, colour per class, Night chart accent.
9. Postcards questions 2 to 7, 9 and 11 to 14 (note `design/SSE4-postcards-design.md`), and citation batches through Consensus. These were open on 25 Nov 01:21; some may since have been answered in the PRs.

## 4. Planned next steps not started, by thread

- **From here (FH) builder**: not opened. Design `design/SSE4-from-here-design.md`, status `status/from-here.md`.
- **Build travel between places (GZ Stage C)**: not opened. Design `design/SSE4-grand-zoom-design.md`.
- **Build the three desks (TM1)**: not opened (question 3). Design `design/SSE4-three-modes-design.md`.
- **Atlas and editions (AT)**, **Spaceflight history (SH)**: accepted designs with status files; packages beyond those merged are listed there.
- Every other track: the rows marked "next" or "queued" in its `handover/status/<track>.md`, plus the reports in section 2.

## 5. Kept outside the repo

- **Cloud environment** "Default" (env_01VdpwuzZCJMF6NTdL6CEGAE, personal):
  - Network: limited, with package managers and MCP servers allowed.
  - Allowed hosts: planet4589.org, ssd.jpl.nasa.gov, nssdc.gsfc.nasa.gov, query.wikidata.org, images-api.nasa.gov, images-assets.nasa.gov, photojournal.jpl.nasa.gov, assets.science.nasa.gov, esahubble.org, cdn.esahubble.org, esawebb.org, cdn.esawebb.org, chandra.harvard.edu.
  - Setup script: none. Environment variables saved by the user: none listed. Network secrets: none.
  - Recreate it on the new account. Note that v4.starsystemx.com is not on the list, so cloud sessions can't fetch the live site.
- **Saved keys**: none in the environment. Deploy credentials live in Cloudflare (Workers Builds, project `star-system-explorer`). Agent keys are bring-your-own in the app.
- **Scheduled tasks** (project):
  - "SSE4 trailer: daily re-shoot" (cron `23 21 * * *`, switched off 25 Nov 02:07). Its prompt is in `handover/memory/` and `project-files/trailer/`.
  - "Final project-files top-up to PC" (one-shot): ran at 22:52 UTC 25 Nov. 821 files changed since 21:10 UTC 24 Nov went to `C:\Development\sse4-project-files\` as `sse4-project-files-final-part01..17.zip`. One file was too big for the copy and is only in project files: `sse4/match/pillars-of-creation/ref.jpg` (27 MB, a NASA/ESA reference photo, re-downloadable).
- **Deploy**: Cloudflare Workers Builds deploys every merge to `main` (START-HERE, Deployment).
- **Project files** (`/mnt/project-files`): `converter/`, `explorers/`, `traveller/`, `sse4/` (design notes, citations batches, `match/` photo poses, `*-screens/` screenshots), `handover-export/` zips. Copies are on Alex's PC at `C:\Development\sse4-project-files\`: the 17:14 UTC 24 Nov snapshot, the late top-up to 21:10, and the final top-up if it ran.
- **Device**: Alex's PC AC-ProArt, approved folder `C:\Development`.

## 6. Project instructions and Context tab (copied in full)

- **Project instructions**: the field is empty (no text set).
- **Context tab**:
  - Repositories: frunkq/star-system-explorer, frunkq/starsystemx-creator-hub, frunkq/starsystem-convert.
  - Shared project files folder `/mnt/project-files`.
  - Approved device folder `C:\Development` on AC-ProArt.
  - Default environment env_01VdpwuzZCJMF6NTdL6CEGAE.
  - No default model for threads.
  - Project memory enabled.
  - Scheduled tasks switched off at project level.
- **Project memory**: copied in full to `handover/memory/` (index `handover/memory/MEMORY.md`), refreshed at the freeze. It is the nearest thing to standing instructions.

## 7. Could not capture

- Galaxy charts and histories (PR 581) and exotic objects (PR 559) were not among the cut-off threads and sent no report.
- Design notes: every `SSE4-*.md` in project files matches its `handover/design/` copy, except surfaces-and-rendering, where the repo copy is the newer. Citations batches, results and corrections are copied to `handover/citations/batches/`. The `match/` photo poses and credits (not the 53 reference photos, 80 MB), the atlas notes and the exogeology notes are copied to `handover/project-files/`.
- Screenshots and clips are never committed. They are in project files and the PC zips.
