# TM6a: each desk remembers its depth and its view (track TM, TM6 part 1)

Status: done 27 Nov 2026, thread "Build the three desks". Design: `handover/design/SSE4-three-modes-design.md` section 10 and question 9 (accepted 25 Nov 2026): "Remembered per desk on the device: the view choices, the depth and the ruler's span. The key is the desk and the level."

## What it does

- **Depth** is kept per desk (`desk.<desk>.depth`). Coming back to a desk brings back its depth.
- **The view choice** (View style, Seen from, Scale, HUD, Band and the composite) is kept per desk and per level (`desk.<desk>.<level>.view`, the levels of `DESK_LEVELS`). Coming back to a desk at a level brings back what it had there. The Workshop has no level and keeps the plain key.
- **A first visit** to a desk, or to a level on a desk, starts from the latest choice made anywhere (the plain `view` and `depth` keys, which every choice also writes). So nothing jumps on first use, and devices that kept choices before TM6 carry them over.
- A view that opens on its own scale (the size comparison at Real) keeps doing so; the recalled scale is what comes back when it is left.
- `desk.ts`: `deskLevelOf(scale)`, `recallForDesk(load, desk, what, level)`, `keepForDesk(desk, what, level)`. The shell reads and writes through them; no desk is tested by name.

## Not in this part (TM6b)

- **The ruler's span by desk** (top-level 12.1): Explore at a system or world opens on human time with "Whole life ›", Build on the full ruler, both remembered per desk and level. The ruler is the chart room's `TimeRuler`; it needs an opening span and a callback, agreed with the chart room.
- **Run's first-visit True** on a capable device (needs the quality tiers probe).

## Checks

- `desk.test.ts`: the desk's own value first, the latest anywhere otherwise, the Workshop on the plain key, both keys written.
- New `tests/ui/desk-memory.e2e.mjs` (added to `ci.yml` and `npm run ui:e2e`): Explore at 3D tilted and Science; Build's first visit starts from that; Build set to the flat chart and Curious; Explore brings back 3D tilted and Science; Build its own; a reload keeps both. Seen red with the shell's change removed (Explore stayed on the flat chart), green with it.
- `npm run ui:check` clean.
