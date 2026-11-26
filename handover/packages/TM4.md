# TM4: Explore tries at once (track TM, built by the editing track)

Design: `handover/design/SSE4-three-modes-design.md` question 3 and sections 3.1 and 6 (accepted by Alex 25 Nov 2026 13:30 UTC). Built on TM1 (PR 601).

## Part 1 (26 Nov 2026): the card's handles live on Explore

- **One mechanism, in `edit.ts`:** `commitEdit` and `commitFields` call `tryFirst` first. On a desk whose record says `writes: "try"` (Explore; `deskTries` in desk.ts), with no what-if open and a GM seat, it opens one by itself (label `TRY_LABEL`, "Trying it") and the change goes there. The stamp and dashed frame, Compare, Keep it and Throw away are the ones the ruler already had. `keepWhatIf` sends the kept edits through the inner `sendEdit`, so keeping on Explore does not open a fresh what-if.
- **The handles row** (`views/editing`): editable is now `deskChanges(desk)` (Explore and Build, never Run). The "Try changing it" button and "Change it on the Build desk" are gone (Alex's yes to question 3 replaces them). Before the first change the strip says "A try: it opens a what-if, and nothing is kept unless you press Keep it." Build's own tools stay on Build (`deskKeeps`): Reach it, Could be, the sweep marks, Type it all in.
- **Desk names:** Handles.svelte no longer tests the desk by name (desk-test ledger 3 to 0).
- Desk record hint for Explore: "Look around and try anything: nothing is kept by accident (V)".

## Checks

- `packages/ui-contract/test/explore-tries.test.ts` (new): the first change opens the what-if by itself, later changes join it, the main line's hash does not move, Throw away leaves it exactly as it was, Keep it inks it with one Undo and leaves no what-if, Build writes to the campaign as before. Seen red: with `tryFirst` returning null, the first-change test fails.
- `tests/ui/editing.e2e.mjs` Explore step rewritten with Alex's yes (26 Nov, 15:10 UTC): the strip is live on Explore, typing 1 AU opens "Trying it", the main line does not move, Throw away puts Mars back. Passes on the desk and phone sizes up to the chart-drag step, which **fails on main too** (Mars's hit box is not where the test expects after Undo; not in CI) and is left for the editing track.
- walk, whatif, tactile e2e pass; `ui:test` 1045 pass; `ui:check` clean. No goldens move.

## Part 2 (next): grips, paste and add as a try

Needs the shell (App.svelte), which another thread is working in:
- A viewed system (the Solar System, any system read back from the universe) is resolved without the branch, so the chart would not show a try's change there; the read-back needs the branch id, keeping the main read for Compare.
- The chart's drag is armed by `ws.mode === "edit"`, which the shell sets from `deskKeeps`; on Explore it should be armed as a try (SystemChart can read `deskTries`, once the read-back shows the branch).
- Paste and Add are refused on a what-if ("value edits only"); as a try they need structural edits on a branch, and the paste line "Paste works on the Build desk" in App.svelte goes.
