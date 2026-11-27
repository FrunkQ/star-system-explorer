# TM2d: See as on the Run desk, three kinds only (track TM)

Status: done 27 Nov 2026, thread "Build the three desks". Design: `handover/design/SSE4-three-modes-design.md` section 3.1 (the See as row: "Three kinds only: Game master, The table, each role. 'What do they see?' is the one lens question asked at the table"). TM2b left this open.

## What it does

- On **Run**, See as offers **Game master** (the Running lens, named for who is looking), **As the table sees it**, and one **As <role> sees it** for each role that keeps its own knowledge. Data views ("Through …"), the other job lenses, packs' lenses, your own and "Make your own…" stay on Explore and Build.
- The L key cycles only through what the desk offers.
- A lens kept for Run from before (say Basic) that is not one of the three kinds gives way to Game master. The choice stays on the device, so nothing is lost if the design changes.
- New: `tableLenses(all)` in `ui-contract/src/lens.ts`. The `see-as` shell part gets two controls, `every-lens` and `make` (desks: Explore, Build), which the shell asks with `partShows`. The rail-open note is corrected: a phone folds the rail too (TM9b).

## Checks

- `desk.test.ts`: `tableLenses` (Game master, the table, each role, and no data view) and the see-as controls by desk.
- `tests/ui/stepper.e2e.mjs` (in CI): Explore offers Basic, Building and Make your own. Run offers Game master first, then only the table's lenses. Choosing The table and then Game master changes the stage's lens. Seen red with the shell offering every lens on Run, green with the change.
- Chromium: stepper, lenses, shell, walk, desk-memory, time, table and switchboard pass. `ui:check` is clean and the unit tests pass (1183).
