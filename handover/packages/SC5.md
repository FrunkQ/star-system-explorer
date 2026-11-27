# SC5: One star's life

Alex, 26 Nov 2026: "would be cool to see these stars on a timeline - rather than order by size you order by 'same
star'... an alternative view on this page that shows each sequence and order of the lifespan... even better if we
can have it animating between them on that page... and have earth at 1au to show scale".

## What it is
- The star gallery (SC1) gains a second view, **One star's life**, beside **Kinds**; a gallery star's card has
  **Follow its life**. A slider sets the starting mass, between the life line's own walls (deuterium to the fits'
  top), on a log scale.
- The stages are `describe_star`'s `phases` for that mass, in order, never a typed list: a red dwarf's, the Sun's and a
  heavy star's lives differ because the life line says so, and when the life line changes (PR 645: helium white dwarfs
  under 0.25 M☉, a Wolf-Rayet stage for the most massive) the view follows with no change here.
- Each stage is read at ten ages (`star-life.ts` `stageAges`: closer towards a stage's end, where a star changes
  fastest; the last stage in log time over its first 10 billion years). Each age is the life line's star with its
  own surface from `read_activity` (spin, spots, cells), drawn live in True through the same face door as everything
  else (`star-stage.ts` `drawStage`, which now also takes lit bodies and lines).
- **Play its life**: a stage every 4 seconds of screen time; the radius moves smoothly between a stage's ages (log),
  and steps at a stage's end (a giant's envelope gone, a white dwarf left). A scrub bar and a strip of stages jump to
  any point.
- Earth at 1 AU: its orbit as a grey line and Earth as a dot, to the same scale. Scales: **Whole life** (the star at
  its largest, or Earth's orbit if larger, fills the view, so the star is seen to swell and shrink), **Earth's orbit**,
  and **The star** (the star now fills the view, so its face shows). When the star is larger than Earth's orbit the
  line is drawn over it and the words say so.
- Remnants still to be drawn (neutron stars, black holes) say "Not drawn yet", never a stand-in disc.
- `readSurface` was lifted out of `readGalleryStar` so the gallery and the life read a star's surface one way.
- Fixed with it: Reference › Classes on a phone put the star gallery in a second column over the schemes (SC1); the
  wide gallery now takes the one column. `span` says "1 minute", not "1 minutes".

## Findings for other tracks (not changed here)
- The pre-main-sequence stage reads the same radius at every age (0.89 R☉ throughout for 1 M☉): no contraction down
  the Hayashi track. A young star should be several times larger at first (from memory, unverified).
- Massive stars' giant stages still reach 5,300 R☉ (20 M☉) and 12,000 R☉ (40 M☉); see SC1's finding. The view shows
  it plainly: 40 M☉ "Whole life" is 58 AU across.
- A 0.2 M☉ star climbs a giant branch to 240 R☉ on main today; PR 645 removes that.

## Checks (26 Nov 2026)
- `packages/ui-contract/test/star-life.test.ts`: stage ages inside their stage; the Sun's stages are exactly the life
  line's phases, every frame in its stage's phase and drawn; the Sun's largest radius between 0.5 and 2 AU
  (absolute); its white dwarf under 0.03 R☉; no blend across a stage's end (seen red with the old cross-stage blend,
  which gave the giant's last moment a radius on the way to the white dwarf's); scales; a 40 M☉ life ends in a
  remnant not drawn; nothing written.
- `tests/ui/star-gallery.e2e.mjs` (desk and phone): the red giant followed to its life, stages in order ending as a
  white dwarf, the giant's tip at its own size, all three scales, a 20 M☉ life ending in a neutron star marked not
  drawn, play.
- Screenshots: project files `sse4/handover-screens/SC1-screens/` (`life-*`).

## The gallery starved a software-drawn browser (CI, 26 Nov)
- PR 665's first CI run timed out waiting for the gallery's 17 stars to be read (60 s). Measured in headless
  Chromium (software GL): 27 to 37 s to read them with the page drawn, **0.5 s with WebGL turned off**; each kernel call
  took about 100 ms from post to answer while the kernel itself spent under 1 ms. Drawing the whole page of stars on
  every screen refresh (and the life view while still reading) held the browser.
- Now: the gallery redraws only when a face's tenth-of-a-second tick or the layout changes; faces move on within a
  CPU budget per drawing (`FACE_BUDGET_MS`, the longest unchanged first); a page's stars and a stage's ages are asked
  of the kernel all at once; the life view draws only on change and plays once the whole life is read. Reading the
  17 stars now takes 3 to 10 s in the same browser, the life 7 to 10 s.
