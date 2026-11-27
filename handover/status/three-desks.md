# Status: every display on the three desks (track TM)

Owner threads: "Design each display in three modes" (Fable, design only, 25 Nov 2026) for the note; **"Build the three desks" (Opus, opened 26 Nov 2026 on the new account) builds TM1, TM2, TM3, TM6, TM7 and TM9** and owns this file from then on. TM4 is "Build planet and star editing"'s, TM5 the chart room's and TM8 each view's owner's; each records it in its own status file too.
Design: `design/SSE4-three-modes-design.md`, **ACCEPTED by Alex 25 Nov 2026 13:30 UTC** (yes to all 15 questions; the names stay Explore, Build, Run). Read its sections 1 and 13A first. Builders may take the packages below. Mockup in project files at `sse4/handover-screens/TM-screens/` (`three-modes-mockup.html` and 12 screens; painted stand-ins, not SSE4 output).

What it is, in one line: Alex asked (25 Nov 12:45 UTC) what each display looks like in his three modes, "Explore, Edit, Play", which are the desks the product calls Explore, Build and Run (which names stand is question 1). Today a desk only arms verbs; the note makes a desk three answers, the same on every display: where a change goes (Explore: a what-if, tried and not kept; Build: the campaign; Run: only the acts of play), what time can do (look; set; the campaign clock in human time), and how much is on screen (every door; every door plus tools; the picture, the place line as a stepper, the five view controls, a clock and Send). Section 3 gives the frame part by part, section 4 the seven levels, section 5 every other instrument, each with its state today.

Seam with the grand zoom (track GZ), agreed by message 25 Nov: this note is the authority for what a desk shows at each level; the grand zoom note for how travel between levels works. The points held in common are in this note's section 9 and the grand zoom note's 5.9.

TM1 (the contract) is built. TM2 parts 1 and 2: the Run desk's rail leaves off the seven instruments of section 5, and its bar, tools and status line are trimmed (depth in More, no Workshop door, Select, Keyboard shortcuts, seat or engine line). TM2 part 3: on Run the rail folds to one button at the head of the view controls. TM3 part 1: on Run the clock (date, + 1 hour/day/week, Run ▸) has the ruler's place. TM3 part 2a: what is due and Jump to a date… on the clock, built against the time design's `happenings` shape (empty until TP6). TM3 part 2b: a look-ahead on Explore is kept with "Keep on Build…". TM6 part 1: each desk remembers its depth, and its view at each level, on the device. TM is a code only, never shown in the product.

| Package | State | Owner |
|---|---|---|
| TM1 the contract: `sse.desk/1`, `desks` on rail entries, shell parts and part controls, `shows()`, per-desk memory keys, the desk-test ledger check | done 26 Nov 2026 (`packages/TM1.md`) | "Build the three desks" |
| TM2 the frame by desk: Run's folded rail, bar, tools and status line; rail entries given their desks | done: part 1 (the rail by desk), part 2 (Run's bar, tools and status line) and part 3 (the folded rail) done 26 Nov 2026 (`packages/TM2a.md`, `TM2b.md`, `TM2c.md`). TM2 is done but for See as on Run (three kinds only), noted under Next | "Build the three desks" |
| TM3 Run's clock in place of the ruler; the ruler's span by desk; "Keep on Build…" | part 1 (the clock in the ruler's place) done 26 Nov 2026 (`packages/TM3a.md`); part 2a (what is due, Jump to a date…) done 26 Nov 2026 (`packages/TM3b.md`), reading TP6's `happenings` when it lands; part 2b (Keep on Build…) done 26 Nov 2026 (`packages/TM3c.md`); the ruler's span by desk moves to TM6 | "Build the three desks", with the chart room |
| TM4 Explore tries at once: handles and grips live, the what-if opening by itself | queued | planet and star editing |
| TM5 the Run card: closed until pressed | queued | chart room |
| TM6 remembered per desk: view choices, depth, ruler span; Run's first-visit True | part 1 (depth per desk, view per desk and level) done 27 Nov 2026 (`packages/TM6a.md`); part 2 next: the ruler's span by desk (with the chart room's ruler), Run's first-visit True | "Build the three desks", quality tiers |
| TM7 the stepper on Run: seven steps drawn, Out and In | queued | "Build the three desks", selector, travel between places |
| TM8 the level pass: each view declares its controls' desks | queued | each view's owner |
| TM9 Fill the screen on Run, the phone's Run frame, tours and start lines checked | queued | "Build the three desks" |

## Next
- TM6 part 2: the ruler's span by desk (top-level 12.1), remembered per desk and level; Run's first-visit True on a capable device. When TP6 lands, hand its `happenings` to the clock (`packages/TM3b.md`); when CLK1 (PR 652) is on main, the clock calls `setRate`. Left from section 3.1 for a later small PR: See as on Run offering three kinds only (Game master, The table, each role). Lower the desk-test ledger as `desk ===` tests go.
- TM4 to "Build planet and star editing", TM5 to the chart room, TM8 to each view's owner, each by message with the note's section named, once those threads exist on this account. TM5 and TM8 read `partShows()` and `PartControl.desks` rather than testing the desk by name.
- From here on, a design note for a new screen gives its Explore, Build and Run rows, and a new control declares its desks (note section 13A).
