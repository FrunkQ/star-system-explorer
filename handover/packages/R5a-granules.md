# R5a part 2: stars boil (granules from the O10 convection cells)

Light's O10 convection (PR 76) gives every star cooler than about 6,900 K a `convection` field in `read_activity`: granule
and supergranule sizes and lives, speeds, `granules_across` and `network_cells`. The Body screen's star now draws those
granules.

## What was built
- `packages/renderer/src/star.ts`:
  - `granuleCount(across)`: cells over the sphere, from its area over a hexagon's.
  - `granulesShow(across, disc_px)`: a granule must be at least `GRANULE_MIN_PX` (6) wide on screen, and the star must
    need no more than 60,000 cells.
  - `granulesFor(address, across, epoch)`: deterministic centres per cell generation (a jittered Fibonacci lattice),
    with a lookup grid. The pattern's own mean and spread are measured so the picture keeps the plain disc's mean light.
  - `granuleDeltaT`: the cell pattern as a temperature change (bright domed upflows, dark lanes); rms `GRANULE_DT` = 3.1%
    of Teff, set from the Sun's measured 14% contrast (Wedemeyer-Bohm and Rouppe van der Voort 2009).
  - `drawStar` takes `granules`: each channel changes by d(ln B)/d(ln T) times dT, linear, so the mean light is unchanged.
  - `narrateGranules`: size, life, speed, and whether this picture shows them.
- `packages/views/body/src/StarDisc.svelte`: reads `activity.convection`; draws the generation alive at the star's present
  moment (epoch = now / granule life); a "Granules" line, `data-granules` (cell count, 0 when not drawn) and
  `data-granule-words`; the Why line names the stand-in.

## Evidence
- `node --test packages/renderer/test/star.test.ts`: 6 of 6. New: the Sun's granules (1,034 across) are not drawn on a
  300 px disc and a 39-across supergiant's are; a 39-across star has about 5,500 cells; same star and generation give the
  same cells, a new generation new ones; dT has mean about 0 and rms within 10% of 3.1% Teff; a supergiant's disc is
  mottled (spread up) with its mean light within 3% of the plain disc's; words for drawn, too small and too hot.
- `tests/ui/render.e2e.mjs`: the Sun on the Body screen says "Granules about 1,346 km wide ... too small to draw" and
  draws 0 cells; the state hash is unchanged.
- Picture: `R5-screens/supergiant-granules.png` (3,600 K, 39 granules across, 368 px disc).

## Fudges (docs/fudges.md)
GRANULE_DT one contrast for all stars; cell places, no motion within a life, constant contrast to the limb, no network.

## Next
Supergranule network faculae near the limb and in the X-ray view; spots placed in the network (O10 notes spot sizes are
not yet tied to supergranules); cell drift over a life when the star view animates time.
