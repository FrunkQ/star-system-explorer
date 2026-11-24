# Status: Editing planets, stars and systems (track ED)

Owner thread: "Build planet and star editing". Covers: ED0 to ED13 and ED0b, from `handover/design/SSE4-editing-design.md` (accepted by Alex 24 Nov 2026 15:21; the package list is its section 15). Kernel asks K1 to K17 are the kernel thread's ("Build kernel resolve and rules"), ordered with this thread on 24 Nov: K13 (pairs) first, then K1 sweep, K2 and K3, K17, K15, K16, K4, K12, K5, K7 and K8, K9, K10, K11 and K14.

Only the owner thread edits this file. Logs are in `handover/packages/`. Index and standing rules: `handover/START-HERE.md`.

| Package | State | Log |
|---|---|---|
| ED1 part 1 the handle sheet (`ui-contract/src/handles.json`: six handles for a world, mass and spin for a star, the results "what followed" watches) and the strip model in `handles.ts` (range from reference worlds and the value, log or linear, "none" at the left end for air and seas, stretch for a typed value, arrow steps; reference dots from packs/base-sol; what followed from the kernel's surface entry) | done | packages/ED1.md |
| ED1a part 1 the units engine: `say` and `readFigure` in `ui-contract/src/figures.ts` over the "How figures are said" table (`figures-said.json`), every factor from the sheet's Units table; readings a click steps through, remembered per kind and scale class (`withChoice`, `chosenReading`) | done | packages/ED1a.md |
| ED1a part 2 **next**: every existing figure onto `say`; click a unit on screen; choices saved with the campaign; "How figures are said" on Reference; the editing-grammar check that fails a screen formatting its own numbers | next | |
| ED0 the Solar System with pairs (Pluto and Charon, Earth and Moon, the Sun's wobble) and the checked list of structural edits; family tree in the ghost's panel and the selector | waits on kernel BC1 and the rest of K13 | |
| ED1 part 2 the handles row and the strip on the body card (`packages/views/editing/`: Handles, Strip), mounted in BodyCard for the GM off the Run desk; Build moves it (drag, arrows, typing in any unit), Explore shows it read-only; letting go is one set_value through `commitEdit`, one Undo; what followed from `resolved_system` surfaces before and after; e2e `tests/ui/editing.e2e.mjs` on desk and phone | done | packages/ED1.md |
| ED1 part 3 exact figures on a pause through `resolved_system {branch}`; Titan on the Air strip; walk step "Change a world" | waits on kernel BC1 | |
| ED2 the sweep behind the strip: tipping points, pencil figures, "what followed" | queued (K1) | |
| ED3 dragging on the chart: orbit size, shape, readout, magnetism, cancel | queued | |
| ED4 to ED13, ED10a, ED13a | queued, in the design's order | |
| ED0b the Solar System experiment for real, against `packs/base-sol`, log in project files | after ED1 to ED13 | |

## Decisions and agreements that live only in threads (written down 24 Nov for the handover)

- Kernel ask order (agreed with "Build kernel resolve and rules", 24 Nov 15:28): K13 pairs (BC1, host checks, rehost, withdraw choices, preview_tree, Alpha Centauri and Kepler-16), then K1 sweep, K2 unset as a read, K3 makeup through and radius from mass, K17 conventions on the way in, K15, K16, K4, K12, K5, K7 and K8, K9, K10, K11 and K14.
- The kernel's BC1 PR (in the golden lane, 24 Nov) brings: `resolved_system {branch}` (asked by this track for the strip's exact answer on a pause: open a branch, `branch_set` the held value, resolve with the branch, discard); `orbits_at` `sys_m` and `centre_m`; `bad_host` on write; `rehost`; `withdraw` with `others` leave (the default, Alex's yes 24 Nov 16:41: moons stay round the host, moving as they were), take or hand; `preview_tree`; `packs/base-starters` (Alpha Centauri, Kepler-16; same sheet shape as base-sol). The UI's Undo of an add or paste sends `withdraw` with `others: "take"`.
- Units the sheet lacks: the data sheet thread agreed (24 Nov 15:29) to add min, kyr, r_moon, m_moon, ocean_earth, tonne, litre, m3, km3, m2, km2, s_earth, arcmin, percent, ppm, tnt_tonne and the imperial set (mi, ft, lb, degf, psi, mph, lb_ft3, ft3, gal, sq_mi) to `UNITS` in scripts/build-base-reference.py. When they land, strike them off `WAITING` in `packages/ui-contract/test/figures.test.ts` and un-skip the "23d 26m" test.
- "Can't find it?" in the Air and Liquids boxes (ED8): the agents thread's hook (PR 412, merged): shell `askOrac(request)` in App.svelte, or an action `{"dialog": "ask", "orac": "<request>"}`; show it only when `linked(loadAgents(localStorage), "building")` (packages/agents/src/agents.ts), otherwise link the data sheet editor. Wording: "Add the gas <name> to the data sheet for <world>". Details in handover/packages/A2.md part 5.
- Titan is not yet in packs/base-sol (BC1 adds it): add "titan" to the Air handle's `moon_refs` once it is.
