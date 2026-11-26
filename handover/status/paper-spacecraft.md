# Status: Paper spacecraft pack (track RK, RK6)

Owner thread: "Build Paper spacecraft pack". Plan: `design/SSE4-atomic-rockets-plan.md` section 6. Other RK rows live in the files of the threads that own them (data sheet: RK1, RK2, RK10; light: RK10, RK11, RK12; galaxy and travel: RK9; kernel: RK9 part 2).

| Package | State | Log |
|---|---|---|
| RK6 part 1: store pack `store-packs/paper-spacecraft` (7 ships from 6 studies, 9 engines, 8 not-yet-built advances, 3 tours as data), builder `scripts/build-paper-spacecraft.py`, test `crates/sse-kernel/tests/paper_spacecraft.rs` | done (PR 438) | packages/RK6.md |
| RK6 part 2: NASA DRA 5.0 nuclear thermal ship (193 days to Mars against 180); Longshot flown between the stars (313 years single-stage against 100 staged) | done (PR 489) | packages/RK6.md, part 2 |
| RK6 part 3: flown again after slow spirals (PR 452): HOPE NEP 1,064 days and the fission fragment ship 1,629, both via the Main Belt; the old low-thrust gap row is replaced by a narrower belt-leg row | done (PR 598) | packages/RK6.md, part 3 |
| RK6 next | 1 source check by the data sheet thread; 2 stages on ship records for NAV (with track RS); 3 Daedalus, EMPIRE, JIMO from primary text; 4 skyhooks and launch loops once open primary text is found; 5 tours play after PB2b; 6 when the travel thread makes belt legs spiral, add test bands for HOPE NEP and the fission fragment ship and retire the belt-leg row in docs/fudges.md | packages/RK6.md, Next |
| RK8 name, credit, links | waits on the Atomic Rockets author's written reply to Alex; do not start | design section 9 |
