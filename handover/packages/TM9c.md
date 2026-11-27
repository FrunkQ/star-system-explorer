# TM9c: the start sheet's lines and verbs checked against the desks (track TM)

Status: done 27 Nov 2026, thread "Build the three desks". Design: `handover/design/SSE4-three-modes-design.md` section 5 (the Start sheet row: "A new visitor lands on Explore", "Carry on returns to the desk last used") and TM9; `SSE4-top-level-ui-and-onboarding.md` sections 3.1 and 16. Reference: Alex's recovered mock-up `sse4/mockups/top-level-interface-and-onboarding.html` in project files (board 1, the start sheet).

## What was checked

- **The 42 opening lines and the ten "What it can do" verbs** (`packages/shell/src/documents/start.lines.json`) against what each desk now shows. Every line still says something true of the desk it opens on: Build opens for "Build a world" and "No excuse for bad physics"; Run for "Run a game", the players' screen lines and the pirates line; what-if lines ("What if Jupiter were closer?", "Settle an argument") open the Solar System on Explore, where a change is tried and "Keep on Build…" keeps it. No text changed.
- **The sheet against the mock-up's board 1**: the same doors, verbs and opening line. "Browse Explorers" and the tours line stay held back until those features are built (they `need` "explorers-shelf" and "tours").
- **Tours**: none are built yet (top-level stage 6), so there is nothing to check. The "Run a game" tour opens from the sheet when it exists.

## The one fix

- A verb pressed when the desk last used doesn't have what it opens now lands on a desk that does, Explore first. Before, "Follow life" pressed with Run last used opened Life on Run, where it is not on the rail, and "See in other light" did nothing because Light is off Run.
- "Carry on" and "Open the Solar System" keep the desk last used. A new visitor still lands on Explore.
- New in ui-contract: `deskFor(desk, onDesk)` in `desk.ts` (the desk in front if it shows the thing, else the first of Explore, Build, Run that does) and `screenOnDesk(doc, desk)` in `screen.ts`. The shell's `fromStart` asks them, and runs the action after the desk's parts are drawn (`tick`) so Light is there to press.

## Checks

- `desk.test.ts`: `deskFor` and `screenOnDesk`, including Life from Run landing on Explore, the desk in front keeping a thing it has, and Light being off Run.
- `tests/ui/desk-memory.e2e.mjs` (in CI): a new visitor lands on Explore; Carry on keeps Run; "Follow life" from Run lands on Explore with Life open; "See in other light" from Run lands on Explore with Light open; "Run a game" from Build goes to Run. Seen red with the desk switch removed (Follow life timed out on Run), green with it.
- `tests/ui/start.e2e.mjs` (not in CI) still passes.
