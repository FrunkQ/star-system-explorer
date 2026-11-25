# SH3 The count of people in space (track SH)

Design: `handover/design/SSE4-spaceflight-history-design.md` section 4.5.

## Done (25 Nov)

- `packages/shell/src/rollcall/CountStrip.svelte`: one stepped line of people in space through time, from the pack's
  own `in_space` steps (or the roster's `countSteps` in a campaign), over the source's whole span (from the first loss
  or the first flight, whichever is earlier, to the last). The day in view is an accent line; every day of a loss in
  In memory is a small hollow mark in ink below the line (not red, no symbol) that goes to that day. Hover reads a
  date and a count (`sayNumber`); pressing goes there; arrow keys step a hundredth of the span.
- It sits in the Roll call panel under the sentence. `tests/ui/rollcall.e2e.mjs` presses it and reads it back.
- Screenshot: project files `sse4/handover-screens/SH-screens/SH3-count-strip.png`.

## Open seam (for the new account)

- **Along the ruler, as designed.** The ruler footer's row is a fixed 96 px in the shared shell (`App.svelte`
  `.app` grid rows), so a strip inside it would squeeze the time ruler. The coordinator agreed (25 Nov 09:43 UTC) an
  optional `rulerStrip` slot on the ruler footer; it was not built because of that height. Building it means App
  giving the ruler row `auto` height when a screen fills the slot, and the strip following the ruler's own span
  rather than the pack's. Until then the panel's strip is the count.
