# SH8 The spaceflight front door, part 1 (track SH)

Design: `handover/design/SSE4-spaceflight-history-design.md` section 7 and ruling 3. Seam agreed 25 Nov 09:43 UTC with
the coordinator (the chart room thread is done; App.svelte is shared shell): a minimal App edit, every other path
unchanged.

## Done (25 Nov)

- An address with `?spaceflight=<a "Sol today" folder>` (and no `#place`) skips the start sheet, opens SSE4's own
  Solar System through the existing `openSol()`, and puts Roll call in front. Roll call reads the folder as before
  (`rollcall/feed.ts` `packFolder`), so places now say "At the Moon" and "Around Earth" from Sol's own names.
- `tests/ui/rollcall.e2e.mjs` loads the address with no click and no start-sheet skip.
- Screenshot: project files `sse4/handover-screens/SH-screens/SH8-front-door-sol.png`.

## Next (part 2, open seams)

- A short address (`/spaceflight`) that fills in the Explorers folder of "Sol today": needs the pack's Explorers
  address and a route on the deploy (Cloudflare Workers Builds), so Alex's call.
- Deep links to a date and a person (`#rollcall/1969-07-20`), the rail folded and the panel wide, "Open the whole
  chart room", and a start sheet card. The place hash belongs to the nav thread.
- When Explorers cannot be reached: say so and offer the Solar System without the pack (the panel already says why
  the folder could not be read).
