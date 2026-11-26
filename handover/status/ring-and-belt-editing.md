# Status: editing rings and belts (track RB)

Owner thread: "Design ring and belt editing" (Fable, design only, 26 Nov 2026) for the note. No builder thread yet. Until one exists, each package below belongs to the thread named in its row, and that thread records it in its own status file too.
Design: `design/SSE4-ring-and-belt-editing-design.md`, **PROPOSED 26 Nov 2026, not accepted**. Its section 11 holds 8 questions for Alex, each with a recommendation; no builder starts until he answers. Readable copy with drawn mock-ups: https://claude.ai/artifact/1YxqDTEaSyXXPqU43Gk81j

What it is, in one line: a ring and a belt are one thing to edit (a crowd of small bodies over a range of distance from a parent, sculpted by moons or planets); one ring per world with its sub-rings as zones inside it; structure is three layers (a base shape picked from plausible alternatives, the sculptors' marks, hand-drawn zones); shaping keeps the mass and only "How much" changes it, booked against a source; a gap wants a cause (Reach it adds the moon, Pin it draws it anyway); colour follows the material, and the app says what a change would take. Test case: a cult paints Saturn's B ring red.

It is the "later note" of the editing design's section 11 and the fix for its finding F2. It builds on, and does not replace: the editing parts (ED), the three desks (TM), the conserved chain (CC), the small bodies track (SB), and the proposed Detail control (RW).

Nothing is built. RB is a code only, never shown in the product.

**Seam agreed 26 Nov 2026 17:48 UTC** with "Give planetary rings real structure": op `ring_structure` (crate `sse-small`, module `ring.rs`), with `variant`, `sculptors`, authored and catalogued `zones` in, and zones with a `cause`, marks and a profile out; the full shape is in the note's section 9. Left to RB1: `branch` if not wired, thickness, the size ladder.

| Package | State | Owner (proposed) |
|---|---|---|
| RB0 the note in `handover/design/`, this file, the row in START-HERE | done, 26 Nov | this thread |
| RB1 kernel: a ring or belt's inputs as authored fields read by `ring_structure`; `branch`; size ladder and thickness; own golden area | proposed, needs the ring structure job merged | the ring structure thread, or kernel |
| RB2 the row and boxes: handle sheet rows for "ring or belt" (Where, How much, Made of, Grains, Stir), walls, typing | proposed, needs RB1 | planet and star editing |
| RB3 the Profile part (an amount along a line), read-only first, then joins and heights that keep the sum | proposed, needs RB1 and question 4 | planet and star editing |
| RB4 "Others like this", Kinds, the Add hoop and edge grips on the chart | proposed, needs RB2, RB3 | planet and star editing, chart room |
| RB5 the books: "How much" offers a moon to break up first (Alex, 26 Nov), then other sources, then "from nowhere" with the overdraft line; Break up and Gather | proposed, needs RB1 and questions 5, 6 | kernel |
| RB6 sculptors: authored zones against moons, Reach it adding the moon, "Could be" records; planets sculpting belts | proposed, needs RB3, RB5 | small bodies, editing |
| RB7 coatings and the "What it would take" card; proven on the red B ring, a generated ring and a belt | proposed, needs RB3, RB5, pigment optics rows | editing, data sheet |
| RB8 the Detail fit: the size ladder with the cut, kept members, "Show finer" | proposed, needs RW4, RW5 | small bodies |
| RB9 `base-sol`: Saturn as one ring with measured zones; the other giants the same | proposed, needs RB1 | data sheet |

## Next
- Alex answers section 11 of the note; his answers go in a section 11A there and in `decisions.md`.
- Then the coordinator opens RB1 once the ring structure job has merged, and RB2 and RB3 after it.
