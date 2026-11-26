# Status: editing rings and belts (track RB)

Owner thread: "Design ring and belt editing" (Fable, design only, 26 Nov 2026) for the note. No builder thread yet. Until one exists, each package below belongs to the thread named in its row, and that thread records it in its own status file too.
Design: `design/SSE4-ring-and-belt-editing-design.md`, **ACCEPTED by Alex 26 Nov 2026 18:12 UTC** (yes to all 8 questions; rulings in the note's section 11A). Builders may take the packages below. Readable copy with drawn mock-ups: https://claude.ai/artifact/1YxqDTEaSyXXPqU43Gk81j

What it is, in one line: a ring and a belt are one thing to edit (a crowd of small bodies over a range of distance from a parent, sculpted by moons or planets); one ring per world with its sub-rings as zones inside it; structure is three layers (a base shape picked from plausible alternatives, the sculptors' marks, hand-drawn zones); shaping keeps the mass and only "How much" changes it, booked against a source; a gap wants a cause (Reach it adds the moon, Pin it draws it anyway); colour follows the material, and the app says what a change would take. Test case: a cult paints Saturn's B ring red.

It is the "later note" of the editing design's section 11 and the fix for its finding F2. It builds on, and does not replace: the editing parts (ED), the three desks (TM), the conserved chain (CC), the small bodies track (SB), and the proposed Detail control (RW).

Nothing is built. RB is a code only, never shown in the product.

**Seam agreed 26 Nov 2026 17:48 UTC** with "Give planetary rings real structure": op `ring_structure` (crate `sse-small`, module `ring.rs`), with `variant`, `sculptors`, authored and catalogued `zones` in, and zones with a `cause`, marks and a profile out; the full shape is in the note's section 9. Left to RB1: `branch` if not wired, thickness, the size ladder.

| Package | State | Owner (proposed) |
|---|---|---|
| RB0 the note in `handover/design/`, this file, the row in START-HERE | done, 26 Nov | this thread |
| RB1 kernel: a ring or belt's inputs as authored fields read by `ring_structure`; `branch`; size ladder and thickness; own golden area | queued, needs the ring structure job merged | the ring structure thread, or kernel |
| RB2 the row and boxes: handle sheet rows for "ring or belt" (Where, How much, Made of, Grains, Stir), walls, typing | queued, needs RB1 | planet and star editing |
| RB3 the Profile part (an amount along a line), read-only first, then joins and heights that keep the sum | queued, needs RB1 | planet and star editing |
| RB4 "Others like this", Kinds, the Add hoop and edge grips on the chart | queued, needs RB2, RB3 | planet and star editing, chart room |
| RB5 the books: "How much" offers a moon to break up first (Alex, 26 Nov), then other sources, then "from nowhere" with the overdraft line; Break up and Gather | queued, needs RB1 | kernel |
| RB6 sculptors: authored zones against moons, Reach it adding the moon, "Could be" records; planets sculpting belts | queued, needs RB3, RB5 | small bodies, editing |
| RB7 coatings and the "What it would take" card; proven on the red B ring, a generated ring and a belt | queued, needs RB3, RB5, pigment optics rows | editing, data sheet |
| RB8 the Detail fit: the size ladder with the cut, kept members, "Show finer" | queued, needs RW4, RW5 (not yet accepted) | small bodies |
| RB9 `base-sol`: Saturn as one ring with measured zones; the other giants the same | queued, needs RB1 | data sheet |

## Next
- RB1 opens when "Give planetary rings real structure" has merged `ring_structure`. RB2 and RB3 follow it and can run side by side (RB2 is handle sheet rows and boxes, RB3 is the new Profile part).
- RB9 (Saturn as one ring in `base-sol`) can start as soon as that job has merged, if that job has not already done it.
- Watch for collisions: the Add list is the editing track's (RB4's Kinds and Add hoop go through it), and the asteroid and comet designer (RW16a) is being built on the Build desk.
