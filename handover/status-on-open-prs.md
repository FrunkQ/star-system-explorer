# Status files that exist only on open PR branches (copied 24 Nov 2026 18:15 UTC)

Each is the owner thread's own status file as it stands on an open PR branch, copied here so it survives if the PR
is still open at the account move. Once that PR merges, the file in `handover/status/` is the truth and the copy
here is stale. Rows changed on open PRs for files that already exist on main are not copied; read the PR.

---

## `handover/status/true-surface.md` from PR 449 (branch `claude/project-thread-3jxtpq`)

### Status: True looks like the real thing (surfaces and small bodies)

Owner thread: "Build plumes, frosts and small bodies". Covers the third V3 audit's items 4, 5, 6 and 10 (R9 to R12, R15, R16, R18, R32, R33): plumes above the limb, tholins and coloured frosts, hot things glowing in Visible, small bodies and comets, lava tube openings on the ground.

Only the owner thread edits this file. Files agreed on 24 Nov: `crates/sse-surface/src/plumes.rs`, a later `shapes.rs`, the tholin and frost cover in `cover.rs` (with "Build SSE4 proven core"); new files `packages/renderer/src/plumes.ts`, `ices.ts`, `hot-glow.ts`, `small-bodies.ts` and their GpuItem fields, filled in the renderer's `trueLook` (with "Build photoreal renderer"). The frost and tholin pass runs after the ground and before "Build giant bands, clouds and hazes"'s air layer. Material colours only through the data sheet's Look up (MP33 titan-tholin, MP34 SO2, sulphur, N2 and CH4 ice, CO2 frost).

| Package | State | Log |
|---|---|---|
| TRUE1 plumes part 1: the kernel's plume facts (ocean, polar cracks, speeds, height, rate) on `resolved_system` surface records | in review | packages/TRUE1.md |
| TRUE2 plumes part 2: drawn above the limb in System True | next | |
| TRUE3 hot things glow in Visible (Planck against the world's own daylight) | module written, drawing next | |
| TRUE4 tholins and coloured frosts | waits on data sheet MP33, MP34 | |
| TRUE5 small bodies and comets | not started | |
| TRUE6 lava tube openings on the ground (from `survey_caves`) | not started | |

#### Live checks (audit R3, per body)
Not yet run. Note: the default Solar System (`packs/base-sol`) has no Enceladus, Io, Europa, Titan, Triton or Charon yet, so those lines cannot be judged on the live site until the big moons are added.

---

## `handover/status/atomic-rockets.md` from PR 447 (branch `claude/project-thread-7xs7xp`)

### Status: Atomic Rockets (track RK)

Owner thread: "Build ship flow and power views" (from 24 Nov 2026). Covers: the RK packages handed on by the chart room
and renderer threads: RK13 (Flow view and Power card), RK12 (range rings as a HUD overlay), RK10 (remaining band
strings), RK3 (ship sizing formulas as named Equations). Other RK packages stay rows in their owners' files: RK9 in
`galaxy-and-travel.md`, RK6 in the Paper spacecraft pack's thread.

Only the owner thread edits this file. One row per package: update it in the same pull request as the package. Logs are
in `handover/packages/`. Plan: `handover/design/SSE4-atomic-rockets-plan.md` (accepted in full, questions 1 to 25).

| Package | State | Log |
|---|---|---|
| RK13: the general Flow view (`ui-contract/flow.ts`, `views/flow` Flow) and the Power card on a ship's card (three columns, "Nowhere" dashed band on the GM seat only, five readings, live starlight and load switches) | done 24 Nov; next: `reject_k` on the kernel's heat rows (golden lane); later users of Flow: a world's energy budget (weather), an element's journey (matter) | packages/RK13-flow-and-power.md |
| RK12: range rings (kernel `effect_at_range` done) as a HUD overlay: `ui-contract/range-rings.ts`, one component each in system-chart and system-3d; the chart room thread mounts them | next | |
| RK10: band strings in ui-contract `light.ts`, `view-controls.ts` (and the renderer's) read `SHEET_BANDS` | queued | |
| RK3: ship sizing formulas as named Equations in `packs/base-ships` (may change goldens: golden lane) | queued | |

---

## `handover/status/true-air.md` from PR 444 (branch `claude/project-thread-nrzllv`)

### Status: Air in True (giant bands, clouds and hazes)

Owner thread: "Build giant bands, clouds and hazes". Covers: TA, the third V3 audit's atmosphere items (`handover/design/SSE4-v3-gaps.md`, third audit: R1 giant bands, R2 storms, R3 polar vortices, R4 oblateness, R6 cloud decks, R7 limb, R8 coloured hazes) and R2b part 2 (the cloud shell at height on the Body globe, handed over by the renderer thread on 24 Nov).

Only the owner thread edits this file. Logs are in `handover/packages/`. Files: `packages/renderer/src/true-air.ts` (and its test); hooks in `SystemChart.svelte` and the renderer's index are a few marked lines. Agreed 24 Nov: the renderer thread owns `packages/renderer`; this track owns `true-air.ts` and the coming `gpu-cloud-shell.ts`, may touch `gpu-globe.ts` and `gpu-air.ts` for R2b part 2 and add optional fields to `gpu.ts`; the plumes thread's `trueIces` tints the ground before the air is laid on.

| Package | State | Log |
|---|---|---|
| TA1: air in system True. Every world's True face is read through one `readTrueFace` (ground tiles, then today's clouds and haze from the weather op's field); a giant is its cloud decks, so its bands, storms and polar cyclones show; each deck coloured by its condensate's sheet colour, white where the sheet has none (said in the HUD) | done | packages/TA1.md |
| TA2: colour from what particles are made of: blocks `sse.light.particle_albedo` (anomalous diffraction) and `sse.light.slab_reflectance` (two-stream), `sse_light::haze`; and `sse.shape.spin_flattening` (Darwin-Radau) for oblateness. Golden-changing: waits for its slot | next (code written, in the thread's working tree until the slot) | |
| TA3: Titan orange and Jupiter's brown from the sheet's n,k rows (data sheet MP33: titan-tholin, jupiter-chromophore, nh4sh, h2so4) and the giants' chromophore haze (weather W10a) | waits on MP33 and W10a | |
| TA4: methane above the decks through `light_through_gas` (Uranus and Neptune blue) | waits on the light thread's fix: its methane rows read about 10⁴ too strong (reported 24 Nov) | |
| TA5: the cloud shell at height (R2b part 2) and the limb of air in system True | next after TA2 | |
| TA6: giants visibly flattened in True from TA2's block (GpuItem flatten along the body's frame) | after TA2 | |

#### Looks like the real thing (audit R3 checklist), as of TA1

Not yet run on the live site: the TA1 check is in its log (local build, WebGL2). Rerun on v4.starsystemx.com after each merge and record the date and the main commit here.

---

## `handover/status/paper-spacecraft.md` from PR 438 (branch `claude/project-thread-evvtvp`)

### Status: Paper spacecraft pack (track RK, RK6)

Owner thread: "Build Paper spacecraft pack". Plan: `design/SSE4-atomic-rockets-plan.md` section 6. Other RK rows live in the files of the threads that own them (data sheet: RK1, RK2, RK10; light: RK10, RK11, RK12; galaxy and travel: RK9; kernel: RK9 part 2).

| Package | State | Log |
|---|---|---|
| RK6 part 1: store pack `store-packs/paper-spacecraft` (7 ships from 6 studies, 9 engines, 8 not-yet-built advances, 3 tours as data), builder `scripts/build-paper-spacecraft.py`, test `crates/sse-kernel/tests/paper_spacecraft.rs` | done (PR 438) | packages/RK6.md |
| RK6 next | 1 source check by the data sheet thread; 2 Longshot interstellar test; 3 Daedalus, DRA 5.0 NTR, EMPIRE, JIMO from primary text; 4 launch loops and skyhooks; 5 tours play after PB2b | packages/RK6.md, Next |
| RK8 name, credit, links | waits on the Atomic Rockets author's written reply to Alex; do not start | design section 9 |

---

## `handover/status/landing.md` from PR 437 (branch `claude/project-thread-zcty3l`)

### Status: landing run-through (track LR)

Owner thread: "Build landing run-through" (from 24 Nov 2026, taken over from the chart room). Covers Alex's milestone of
23 Nov 2026: "landing sequence and end to end run through ... the GM plotting the Players experience of the events".
The journey: the party's ship flies a voyage to a world, arrives, the world is revealed, the GM picks a site, the
ship descends and lands, the party looks about; players on their own seats see each step the GM sends.

Owned here: this file, `tests/ui/run-through.e2e.mjs` (the whole journey, GM and a player side by side), the hand-off
from Voyages to Land, and the Land screen's glue (`packages/views/surface/src/Surface.svelte` controls; the renderer
thread owns its drawing). Not owned: `shell/src/run/`, `Players.svelte`, `landing-table.e2e`, `party.e2e` (Run desk
thread, status/run-desk.md); `shell/src/chart/` (chart room); `nav/` (selector); the drawing in `renderer/` (renderer).
Only the owner thread edits this file. Earlier landing rows stay where they were written: chart-room.md ("What the table
is shown", "The table watches a landing", "Players see the party's ship"), galaxy-and-travel.md (party ship, blocker 4),
renderer.md (SL1, V10g, V10i, V10j, V10k), run-desk.md (RD1 reveal on arrival).

#### The journey on main 9a5912c (24 Nov 2026, walked by `run-through.e2e`)

| Step | GM | Players | State |
|---|---|---|---|
| Party's ship picked (Ships and stations) | yes | header "Aboard Beowulf, at Earth" | works |
| Voyage Earth to Mars, "Fly Beowulf there" | yes, At-world card with dangers | header follows to Mars | works |
| Arrival reveal | RD1 slip (PR 421) | through set_known | in PR 421 |
| Approach and orbit | At-world card (aerocapture, entry, powered landing) | nothing until sent | gap LR4 |
| Pick a site (Land here, landmark or anywhere) | yes, party's ship as lander | nothing until sent | works |
| Send this view | yes | Landing screen, same site and verdict on own kernel | works (send race fixed in RD1) |
| Descend | flown from the kernel's log | flies too, a few seconds behind | works; verdict wrong, gap LR1 |
| Land, look about, walk | yes | follows the GM's pose | works; night and 3D ground, gaps LR2, LR3 |
| Take off | yes | back to the site | works |

#### Gaps and packages

| Package | Owner | State | Notes |
|---|---|---|---|
| LR0 this file and `run-through.e2e` (the whole journey as one check) | here | done | not in CI (workflow edits need Alex's yes); in `npm run ui:e2e` |
| LR1 a ship that can fly down flies down: plan_descent with a capable engine (thrust over weight, delta-v enough, as ship_at_world's powered-landing check) plans a powered descent from orbit instead of a ballistic entry, picking the better of the two and saying which and why. Today a Traveller far trader at Mars "burns up" (aerobrake only, 7 W/cm² against a 1 W/cm² hull, 2392 m/s touchdown) while the At-world card says "Powered landing from orbit: within limits" | here (kernel thread agreed, sse-nav descent) | next | may move golden plan_descent cases: ask the coordinator for a lane slot first |
| LR2 choosing the hour: the Land screen's "Hours from now" slider binds degrees (±180) under an hours label; label it in hours and add Dawn, Midday, Dusk and Night presses on the world's own clock, so the GM can land in daylight | here (renderer agreed) | next | Surface.svelte only |
| LR3 the ground at night and in 3D: True at night is black with no stars, 3D is a flat grey slab (headless WebGL2) | renderer | asked 24 Nov | |
| LR4 approach and orbit for the table: between arrival and the site the players see only the header; a "Beowulf in orbit of Mars" view (the world turning below, the At-world card's chosen act) for Send this view | here, with the Run desk thread for the preset | queued | |
| LR5 a landing preset for the Playbook (PB3 "A landing"): an exported pure helper `landingFor(world, site) -> LandingShow`, so a preset stores {world, site} and builds the landing when fired | here, for the Run desk thread | queued | |
| Gap 10, Mass on the Run desk card at Curious | Run desk thread (asking Alex) | open | |

#### Next
LR1, then LR2, then LR5 (the Run desk needs it for PB3), then LR4.

---

## `handover/status/surface-timelapse.md` from PR 436 (branch `claude/project-thread-7g5v90`)

### Status: Surface time-lapse and sea drawing

Owner thread: "Build surface time-lapse" (handed R6 part 2 by "Build photoreal renderer", 24 Nov 2026). Covers the
drawing of the kernel's `surface_frames` on the Body globe (`packages/renderer/src/timelapse.ts` and its test, the
time-lapse parts of `packages/views/body`: Body.svelte's `lapse` API and BodyTimeline.svelte, and the `frames()`
plumbing in `packages/renderer/src/runtime.ts`), and the drawing side of the sea items W-sea2, W-sea3 and W-sea4
(a sea shading file of its own). The renderer thread keeps the rest of `packages/renderer`; the kernel side of the
sea is "Build shared climate and weather" (`status/weather.md`), the frames op is proven core's (SF).

Only the owner thread edits this file. Logs are in `handover/packages/`.

| Package | State | Log |
|---|---|---|
| R6c: the time-lapse reads PR 394's frames: a dry world (sea_level_m null) has no sea and its basins are bare rock; the kernel's ice-age onsets, which all fall in the last few million years, get a strip of their own under the main scrubber with a fine range over the orbit-history window, a line each ("Ice age 3 of 9 in the last 200 thousand years, begun 20 thousand years ago: ...") and their why on the card; times inside the window read in thousands of years; the frame's own kernel line on pick; a Solar System Earth test (31 ice ages dated in 500 thousand years, each frame's climate on its own land) | done (this PR) | packages/R6c-timelapse.md |
| R6c part 2: each ice age's spread on the globe, from the cells the kernel names at its onset (reader done; waits on proven core sending `cells` on "ice-age" events) | waiting on the kernel | packages/R6c-timelapse.md |
| W-sea2 drawing: sea colour from the liquid's own absorption (deep colour, shallow fade over the floor, Fresnel with the liquid's index) | next | |
| W-sea3 drawing: waves as the kernel's wave trains (Gerstner sum), ripple detail to the kernel's cut-off | after W-sea2 | |
| W-sea4 drawing: whitecaps and streaks of froth from the kernel's whitecap fraction | after W-sea3 | |

#### The sea contract (agreed with the climate thread, 24 Nov 2026)
Same keys at the site (weather `report`, `data.sea`) and on the globe (`survey.climate.sea.bands[]`, one entry per
climate band, null over land). Every key is additive; a missing key means "not sent yet" and the drawing says so.
- W-sea2: `bands_nm` [650, 550, 450]; `absorb_per_m` [R,G,B]; `backscatter_per_m` [R,G,B]; `refractive_index`;
  `deep_rgb` (R∞ ≈ 0.33 b_b/(a+b_b), Gordon and Morel, computed by the kernel). The drawing does the shallow fade
  exp(−2az) over the floor and the Fresnel with that index.
- W-sea3: `trains`, up to 6 of {wavelength_m, amplitude_m, period_s, dir_deg} from the kernel's spectrum and its
  capillary-gravity dispersion; `ripple_cutoff_m`; `superfluid` and `liquid_note` for the card; `wave_m` and
  `period_s` kept as before.
- W-sea4: `whitecap_fraction`, `foam_albedo`, `streak_dir_deg`.

---

## `handover/status/galaxy.md` from PR 431 (branch `claude/project-thread-z535je`)

### Status: Galaxy charts and histories

Owner thread: "Build galaxy charts and histories" (split from "Build galaxy charts and travel screens" on 24 Nov 2026; travel keeps V8, voyages, the flight log and the travel panels). Covers: U6, U8, V9, GH8, V3 gap 39.

Files: `packages/views/galaxy-chart`, `packages/views/galaxy-3d`, `packages/views/foam-map`, `packages/views/cluster-map`, `packages/views/story-finder`, `packages/ui-contract/src/galaxy.ts`, `tests/ui/galaxy*.e2e.mjs`, `tests/ui/foam*`, `tests/ui/cluster*`, `tests/ui/stories*`. The True picture (GH9, `GalaxyScars`, renderer galaxy files) is the renderer thread's; the 3D and True web cube (`foam-map/src/Web3D.svelte`, renderer `web-cube.ts`) is "Build cosmic web view"'s.

Only the owner thread edits this file. One row per package: update it in the same pull request as the package. Logs are in `handover/packages/`. Index and standing rules: `handover/START-HERE.md`.

| Package | State | Log |
|---|---|---|
| U6 galaxy and patch charts (part 1: Galaxy under the Explore tab, from the `galaxy_chart` op) | parts 1, 2a (inked history preview) and 2c (what other galaxies brought, GH3) done; 24 Nov: the stellar halo row is `data-stellar-halo` (it clashed with the halo picker's `data-halo`); next: part 2b, patch survey, find a home, save inked history | packages/U6.md |
| V3 gap 39: system hover summary (GM only: counts of stars, planets, moons, minor bodies, ships, any life, anything remarkable; the selector of gap 37 reads the same summary) | part 1 done 24 Nov: pure reader `ui-contract/src/system-summary.ts` (`systemSummary`, `summaryWords`; reads scene subjects or picker chips; life only from life/biosphere tags or a Life chip, never guessed), test `system-summary.test.ts`; the navigation thread puts it on the picker's system rows; the hover on Your systems done with gap 17 part 1 | packages/GAP17-your-systems.md |
| GH8 leading-arm mark in the chart's history (design SSE4-galaxy-histories-design.md 9.4; rare, about 1 in 100, a cited editable number) | chart side done 24 Nov (`leadingArmWords`, the line and mark name in "Built from other galaxies"), hidden until the kernel's GH3 `leading_arm` mark merges (golden lane, "Build galaxy arm fix and atmosphere advice"); then an e2e | packages/GH8.md |
| V3 gap 17: Your systems, the campaign's systems on one map (U6 part 2b) | part 1 done 24 Nov: screen `systems` at the galaxy scale, view `sse.view.your-systems`; systems where their own V3 map puts them, the rest listed; hover says what each holds; press opens; loader `readCampaignSystems`; next: routes, Add system here, the home star cell behind | packages/GAP17-your-systems.md |
| V9 map views (Cosmic web foam under Explore from `web_slice` with time scrub and Play; cluster map "Galaxies nearby" on `halo_cell`) | parts 1 and 2 done | packages/V9.md |
| U8 story finder (Stories under Explore, on the `sift` op; starter sifters as data, hits with evidence, the GM's own sifter, a storyteller brief) | part 1 done; 24 Nov: "Tell it with Minerva" on each hit (ctx.describe, GM machine), the brief still copies for any storyteller; next: rarity from resolve clouds, event starters, a story pack | packages/U8.md |

---

## `handover/status/traveller.md` from PR 400 (branch `claude/project-thread-mzdt73`)

### Status: Traveller pack

Owner thread: "Build Traveller pack". Covers: X3a to X3d, TG8, TG9, TG11, TG13, MP16.

Only the owner thread edits this file. One row per package: update it in the same pull request as the package. Logs are in `handover/packages/`. Index and standing rules: `handover/START-HERE.md`. Stage order: `handover/design/SSE4-coming-together.md`.

| Package | State | Log |
|---|---|---|
| X3a: code sheets (`sse-codes`) and the Traveller pack's world codes (UWP, PBG, trade codes, zones, bases) from classification rules; units, Imperial calendar (`ui-contract/calendar.ts`), words; kernel op `codes`; world data sheet view (`sse.view.world-codes`) | done (Traveller pack track); lens, reports and sifters next | packages/X3a.md, packages/X3a-sheet.md |
| X3b part 1: a whole system from a coded world (pins to a formation request, codes win, impossible values marked with reasons, make physically sound, counts searched) | done (Traveller pack track); kernel one-shot op and moons next | packages/X3b.md |
| X3c: hex map view (`sse.view.hex-map`); sector formats (`sse-sector-io`); import through the code sheet (kernel op `import_sector`, preview or authored events); Travellermap fetch through the site Worker (`/tmap/*`) | done (Traveller pack track); GM screens in X3e | packages/X3c.md, packages/X3c-io.md |
| X3d parts 1 to 3: gas mixes and what to wear outside; sensors, range bands and the knowledge ladder as triggers and grants (to each ship's own crew); stock ships, stations and sites as J7 templates | done (Traveller pack track); add from templates (gap 13), readiness (TG9), jump measure (MP16) next | packages/X3d.md |
| X3e: chart room screens: Sector map (galaxy scale: hex map, import with preview, export, sifters, reach) and Codes (world scale); kernel op `sector_map`; rail entries that `need` a pack key | done (Traveller pack track); the stage 5 walk as an e2e next | packages/X3e-screens.md |
| X3f: Ships and stations screen (V3 gap 13): constructs in a system, Add here from the stacked templates, cards by depth, edit in place, take out of play | done (Traveller pack track); constructs on the chart and flight profiles next | packages/X3f-constructs.md |
| TG9: `ship.readiness` and `ship.discipline` kept values with template defaults; `status.derelict` and `status.damaged` act only through visible pack triggers | done (Traveller pack track); programs that read them, and kept values on the card, next | packages/TG9-readiness.md |
| TG13: two universe packs written for SSE4 (`sse.near-future`, `sse.space-opera`: templates, factions, labels, dressing rules, a political lens); Traveller ports and bases as dressing rules; both on the Packs screen | done (Traveller pack track); ghost markers are TG14 (chart room) | packages/TG13-universe-packs.md |
| TG8 part 1: `access.*` labels in a base hooks pack (`sse.base-hooks`); the optional intrigue pack (`sse.intrigue`, seeded, GM-only, off by default) | done (Traveller pack track); part 2, `resource.*` over the TG11 inventory, waits on TG11 | packages/TG8-access-intrigue.md |
| V3 gap 16 part 1: the ship's log on the construct card, read from the history and the record (`shipLog`, session `record`) | done (Traveller pack track); g-force and dose on the voyage card (travel thread), drift (V8), and which facts an edit changed, next | packages/G16-ships-log.md |
| V3 gap 26: a seat's view inside a virtual tabletop (`&embed=1`, exact origin list on the seat, refusal with a reason, a ready and focus message bridge) | done (Traveller pack track); a "subject clicked" message when a tabletop asks | packages/G26-embed.md |
| Sector map on the shared view controls (V10a): flat chart spec, HUD strips labels and overlays and keeps the grid; whole hex takes a press; stage 5 walk start as an e2e | done (Traveller pack track); the grid as a pack overlay on other charts (gap 18), Layers names | packages/X3e-hud.md |
| V3 gap 25 part 1: a seat's brand (name and small raster logo) on its screens; join, waiting and away words from the packs' vocabulary (`vocabulary.join`), Traveller wording | done (Traveller pack track); part 2, quick overrides on the Seats panel and the size coming | packages/G25-brand-join.md |
| V3 gap 23 part 1: Describe on the body card hands Minerva one subject and its neighbours for one audience; a passed text kept as a narrative description note in the record (`session.addNote`) | done (Traveller pack track); the description on the card, and players seeing it through the record channel, next | packages/G23-describe.md |
| V3 gap 25 parts 1 and 2: a seat's brand (name and small raster logo) on its screens; join, waiting and away words from the packs' vocabulary (`vocabulary.join`), Traveller wording; quick overrides (`Seat.live`, Hide labels) from the Seats panel, live, never saved | done (Traveller pack track); Hide orbit lines, the single table and sealed seats, and the size coming, next | packages/G25-brand-join.md |
| V3 gap 18 part 1: the Traveller sector grid as pack data (`provides.other.overlays`, the hex map's own grid) on Voyages, placed on a star, under the overlay slot of PR 271 | done (Traveller pack track); imported worlds tied to the stars under their hexes next | packages/G18-sector-grid-overlay.md |
| Example packs 4.1: the Traveller standard ships (12 starships, 6 small craft) as templates and as pack ships the voyage planner flies and the designer draws (`nav_ships`, `packShips`); Mongoose's notice word for word (`provides.other.notice`, shown on Packs); the World hooks tag set (`sse.world-hooks`, 13 prompts) | done (Traveller pack track); pack ships in the voyage pickers (travel thread), tidal lock hooks | packages/X3g-standard-ships.md |
| A game system's difficulty numbers beside SSE4's words: `provides.other.difficulty` (dice, ladder, typical operator, words, skills), ui-contract `gameDifficulty` from odds or words; Traveller's ladder and skills from the Cepheus Engine SRD | done (Traveller pack track); shown on the At world card and route console (travel thread) next | packages/X3h-difficulty.md |
| Traveller sensors as instruments (RK11 pack share): each of the twelve kinds carries an sse-sense instrument (method, band, aperture, noise) so `detect` computes its reach; V3 ranges kept as authored | done; triggers asking `detect` (kernel lane), fits per tech level (FT8) next | packages/X3i-sensor-instruments.md |
| Resources on the books (TG11 kept half): `resource.<kind>.extracted` and `.stock` kept on any address, held = natural - extracted + stock in `read_inventory`, new op `transfer` that conserves | done; mining step and `resource.*` labels (X3k) next | packages/X3j-resource-books.md |

#### Open now

- **PR 400** (X3j, resources on the books). Green, and it changes `tests/golden/contract.jsonl`. It merges in the golden lane after the travel flight log (PR 403) and the kernel's barycentre PR. To land it: merge main, run `SSE_BLESS=1 cargo test -q -p sse-kernel contract_matches_golden`, check, push, merge on green.

#### Next packages, in order

1. **X3k: mining and resource labels** (TG11 program step, TG8 part 2).
   - A ship's program moves ore at a computed rate by `transfer`.
   - `resource.*` labels are derived from `held_kg`, so a mined-out rock loses its label by itself.
   - Packs name their making and consuming effects (a refinery).
   - Check: helium-3 on all four giants (MP8c: kind with matter `he3`, reach `envelope`).
2. **Sector routes as links.** Import the routes in sector files (trade, courier, jump lanes) through the travel thread's `set_link` (PR 377), with `source` set to the import and `audience` and `tier` for routes players don't know. `plan_voyage` already uses crew-known links.
3. **RK12, the Traveller mapping** (Atomic Rockets plan, accepted in full): weapons as "effect at range" (in range, marginal or out of range; nothing fired), with the game's table beside the physics. Then the RK16 pack side: stealth and countermeasure gear onto the counters list. Design questions go to the Atomic Rockets track.
4. **MP16 counted measures.** "2 jumps" and a "Jump-2" drive rating (never "J2"). The default count is ceil(route pc / rating), and the GM's count overrules it. This needs the kernel's counted-measure declarations, and links carry `counted` in `nav`.
5. **WBH extended profile** as one swappable "big block".
   - Read the physical profile off SSE4's physics; never roll a physical digit. Then add a seeded, GM-only social layer.
   - Use Cepheus Engine SRD (OGL) numbers for the basic social rolls, and keep WBH tables out (Mongoose copyright).
   - Port ideas, not code, from Roy Martin's TravellerSystemGenerator (GPL-3.0, so its code cannot enter this Apache repo). Credit it in sources and the log according to its licence, and say nothing about permission.
6. **Smaller items:**
   - Triggers that ask `detect` for the pair instead of reading `range_m` (kernel lane).
   - Fits per tech level, and the Traveller TL table as a tech ladder (FT8, after FT1 and FT2).
   - SV7: the pack names its own "Known to" rungs.
   - Switch `shipLog` to `crewAboard` (PR 318).
   - Gap 25 rest: hide orbit lines, sealed seats.
   - Gap 18 rest: imported worlds tied to the stars under their hexes.
   - Tidal-lock hooks once a lock fact exists.
   - OV10 pack half: allegiance and travel zones as hex map layers.

#### Alex's rulings for this track (23 and 24 Nov)

- Never the TAS logo or other trademarked art. Ship no copyrighted rules text, and bundle no official setting data.
- The Traveller pack data alone is CC BY-NC 4.0 (`packs/traveller/LICENCE.md`, which quotes Mongoose's fair use notice word for word); the repo stays Apache-2.0. Alex emails Mongoose (sales@mongoosepublishing.com) himself.
- Traveller is the 4.0 showcase. Other game packs (Mothership game side, Eclipse Phase, 2300AD) wait for 4.2.
- SSE4 never rolls dice or decides. A game pack may show its own difficulty ladder beside SSE4's words: odds through the system's dice first, then the word table; GM screens only; the skill shown when the pack knows the game's skills (X3h).
- Sensors must "just work", and stay tweakable by those who care (X3i).
- The World Builder's Handbook way of making worlds is wanted. It ships as one "big block" a GM can grab and change (see next package 5).
