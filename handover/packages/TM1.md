# TM1: the three desks' contract (track TM)

Status: done 26 Nov 2026, thread "Build the three desks" (Opus). Design: `handover/design/SSE4-three-modes-design.md` section 10, accepted by Alex 25 Nov 2026 13:30 UTC (section 13A). **No screen changes**: everything below is data and checks; what the GM sees on each desk is exactly as before.

## What it does

- **`sse.desk/1`: a desk is a record** (`packages/ui-contract/src/desk.ts`). `{ schema, id, label, key, hint, line, writes, time, lens? }`, with `writes` one of try, campaign, play and `time` one of look, set, clock (design section 1's three answers). The three shipped desks are `SHIPPED_DESKS`; `checkDesk` checks one word by word. `DESKS` and `Desk` now live here (re-exported from `actions.ts` as before) and the object menu's `DESK_WORDS` come from the records.
- **The desk switch reads its words from the records.** `App.svelte` builds the switch's options, the lens each desk opens with (`lens`; left out, the device's default) and whether the chart is armed for editing (`deskKeeps`: `writes === "campaign"`) from them, replacing three local tables. Renaming a desk is now a change of data. The words, keys and hints are unchanged.
- **One answer to "is this on screen?": `shows(thing, desk)`.** Anything with a `desks` list (left out: all three) is on those desks. `checkDesks` refuses an empty list, an unknown desk or a desk listed twice; `deskWords` says the desks in words ("Build and Run").
- **`desks` on rail entries** (`RailEntry.desks`). `checkScreen` refuses a bad one; `railOf` takes an optional desk and leaves out an entry not on it. No entry declares desks yet and the shell does not pass the desk yet, so the rail is unchanged; TM2 gives the entries their section 5 desks and passes the desk.
- **`desks` on part controls** (`PartControl.desks`, the AT13 shape). The parts catalogue line (`controlLines`, read by the Reference and Orac) adds "on Build only" when a control is not on every desk; lines for controls without desks are unchanged.
- **The shell's own parts** (`SHELL_PARTS`): place line, subtitle, desk switch, See as, depth, Ask, More, rail, view controls, the stage's tools (with Select, Undo, Redo, Copy, Add here, Cut and Keyboard shortcuts as controls), card, the "Known to" eyes, the what-if stamp, ruler, Light, Sound, status line. **Each row says what the part does today** (design section 2), not the section 3.1 target, so the table tells the truth while nothing on screen moves: every part is on all three desks except the eyes (Build and Run) and the tools' Undo, Redo, Copy, Add here and Cut (Build; Undo and Redo also show on another desk while a what-if is open, which is the shell's state, not a desk). `partShows(part, desk, control?)` reads it and throws on an unknown id. The clock (7.3) joins in TM3.
- **The eyes read their desks from the table.** `eyesShow` (`known.ts`) now asks `partShows("eyes", desk)` instead of naming Build and Run; the answer is the same on every desk.
- **Per-desk memory keys** (question 9): `deskMemoryKey(desk, what, level?)` gives "desk.run.system.view", "desk.build.galaxy.ruler-span" and "desk.run.depth" for `loadPref`/`savePref`. The view choice (the whole `ViewChoice`: View style, Seen from, Scale, HUD, Band, Motion, Aids) and the ruler's span are per desk and level, depth per desk (`DESK_MEMORY`). The seven levels are `DESK_LEVELS` (web, cluster, galaxy, local, system, world, site: `PLACE_SCALES` plus the two new steps). Nothing reads or writes these keys yet; TM6 does. The device's existing "desk" and "desk-lens" keys are untouched.
- **The desk-test ledger** (`tests/ui/desk-ledger.test.ts`, `desk-ledger.json`): like the figures ledger, it counts per file every comparison of a desk-named value with `===` or `!==` outside ui-contract (views, shell, chart-grammar, renderer, skins, agents). A file may go down, never up; a new file starts at none. **30 today**, App.svelte 10 of them (it was 12 before this package). Seen to go red: one extra `desk === "run"` in `Stage.svelte` fails it with the file and counts named.

## Checks

- `packages/ui-contract/test/desk.test.ts` (9 tests): the records and their answers, `checkDesk`, `shows` and `checkDesks`, the shell parts as today (the eyes and the Build tools are the only rows with desks), `eyesShow` unchanged on each desk, the control line's desk words, the rail filtered by desk and a bad `desks` refused by `checkScreen`, the memory keys.
- `npm run ui:check` clean. `npm run ui:test` passes (the kernel-backed tests need the wasm build, as in CI). `scripts/check.sh` passes. No goldens change (nothing in the kernel moves).

## Note for the chart room

The ruler, card and stage tools are rows in `SHELL_PARTS`. TM2 moves the frame's rows to their section 3.1 desks one small PR at a time; TM3 adds the clock, TM5 (the Run card) is the chart room's and should read `partShows("card", desk)` rather than test the desk by name. The old account's chart room thread did not move to this account, so this note stands in for the message.

## Next

- TM2: the frame by desk. The shell's parts read `shows()`; Run's folded rail, bar, tools and status line; the rail entries given their section 5 desks, and `railOf` given the desk.
