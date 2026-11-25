# TRUE6: lava tube openings on the ground

Track TRUE-surface, third V3 audit ("seeing a lava tube opening"). Going inside is parked with interior sites (about 4.2); this package draws the holes in the ground and says where they are.

## What
- `packages/renderer/src/cave-mouths.ts`: the kernel's cave survey at a landing site (S14a, op `survey_caves`, sse-surface caves.rs) read as openings. Every segment of tube whose roof has fallen (the kernel labels the run a skylight, a pit chain or a rille) is an opening in the site's frame (x east, y north), of the kernel's width, its floor the kernel's passage floor. `caveCut` lowers the ground inside an opening to that floor, with the broken roof edge as a steep wall; `nearestOpening` and `openingLine` name the nearest one with its size, depth, distance and bearing.
- `ui-contract` `Session.surveyCaves` (GM, or a seat while the table watches that world's landing, as for the ground's tiles), and the same read on `LandingGround` so players' Land screens cut the same holes on their own kernel.
- `views/surface` Surface.svelte: on landing, the caves within 25 km (the kernel's widest window) are read; within the 2 km walk patch each opening is cut into the ground (you can walk to the rim and down into it), the HUD names the nearest opening, and the ground note says how many lie near.
- Fudge row: "Lava tube openings on the ground (TRUE6)" in docs/fudges.md, Light, rendering and sound.

## What the kernel says about the real Moon
On the sample Moon every tube field's roof is gone ("0% of the roof still stands: craters deeper than the 37 m roof have breached 100% of it"), so its openings are long rilles, and tube fields are sparse: a site needs to be near a field's centre to find one within 25 km. Earth's plains and young shields keep more roof, so skylights and pit chains are found there. Whether the Moon should keep some roof (the Marius Hills and Mare Tranquillitatis skylights, Haruyama et al. 2009, are real) is a question for the surface owner: the kernel's crater breaching takes every roof in 4 Gyr.

## Checks
`packages/renderer/test/cave-mouths.test.ts` (only fallen roof opens; the cut reaches the floor inside, not over standing roof or past the rim; the nearest opening line); `packages/ui-contract/test/landing.test.ts` "lava tube openings near a landing site on the sample Moon" (a site at each tube field's centre, openings with widths and depths, the same openings on a second look, players refused).
