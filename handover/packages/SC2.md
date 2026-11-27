# SC2: the gallery on the formal star classes

"Classify every star and remnant" (PR 642) put the `star` scheme in the base atlas: 61 classes in ten families, each
class's first example carrying its life-line sample (`star.sample_mass_msun`, `star.sample_age_yr`,
`star.sample_feh`). Classes and Make a system already read it through `tilesFromScheme` (that thread wired them and
moved `tests/ui/star-gallery.e2e.mjs` to the real ids).

## Done here
- `STAND_IN` is gone, with the phase-aged sample (`sampleAge`) only it used: a tile's sample is the atlas's.
- Reference › Classes offers the star button only for an atlas with a star scheme (the SSE 3 classic atlas has
  none), so there is no stand-in page.
- Make a system › Choose the star: the stack's scheme, else the shipped atlas; if neither can be read it says so and
  points back to giving a mass, rather than offering made-up kinds.
- The cards already show `describe_star`'s `now.class.code` (O6 V, M5 Ia, DC 10...).

## Checks (27 Nov 2026)
- `packages/ui-contract/test/star-gallery.test.ts` now reads the shipped atlas: every class with a sample (39 of 52
  tiles) is read back by `describe_star`'s classifier as **its own family and kind**, is drawn (remnants: not drawn
  yet, with why), and is no older than the universe; the 13 without a sample (G and K supergiants, an M subgiant,
  supermassive black holes...) say why. **Seen red**: moving the K dwarf's sample to 1.2 M☉ fails it ("reads back
  as F8 V").
- `tests/ui/star-gallery.e2e.mjs` and `tests/ui/reference.e2e.mjs` pass (desk and phone).
- Screenshots: project files `sse4/handover-screens/SC1-screens/sc2-classes-*.png`.
