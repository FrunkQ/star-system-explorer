# TM3c: "Keep on Build…" for a look ahead on Explore (track TM, TM3 part 2b)

Status: done 26 Nov 2026, thread "Build the three desks". Design: `handover/design/SSE4-three-modes-design.md` section 7.1 ("Keeping it is a Build act: the button reads Keep on Build…, switches desk and offers the two named accepts").

## What it does

- On **Explore** a look-ahead can be run, scrubbed, asked why and thrown away, as before. Its two keeps ("Keep as history", "Keep and move now to …") give way to one button, **Keep on Build…**, which switches to the Build desk with the look-ahead still open. On **Build** the panel offers the two keeps, unchanged.
- `LookAhead.svelte` takes `canKeep` and `onbuild`; the shell passes `canKeep` from the desk record (`writes !== "try"`), not a desk test.
- The keeps keep their present words. The design's two named accepts ("Make this the campaign's now", "Make this system this old, today", top-level 12.2) need the kernel change of top-level question 1; when it lands they replace these two on Build.

## Tests followed the design

- `lookahead.e2e.mjs` and `adopt.e2e.mjs` kept a look-ahead on Explore. Each now checks there is no keep on Explore, presses "Keep on Build…", checks the desk is Build and the look-ahead still open, then keeps as before.

## Not in this part

The ruler's span by desk (top-level 12.1) goes with TM6 (remembered per desk), since the span is remembered per desk and both change what the ruler opens on.

## Checks

- `npm run ui:check` clean.
- Chromium: lookahead and adopt pass.
