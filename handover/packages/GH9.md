# GH9: every collision leaves its mark

Track GH (`design/SSE4-galaxy-histories-design.md` section 9.4), built 23 Nov 2026 by the renderer thread after GH3
(PR 250) put dated marks on a looked-at galaxy. Alex, 22 Nov: "each collision should leave its mark. High speed ones
can literally punch a hole. slower ones will stay around forever orbiting".

## What

- `ui-contract/src/galaxy.ts`: `GalaxyChart.assembly` (GH3's tally) with `GalaxyMark` and `GalaxyAssembly` types.
- `renderer/src/galaxy-marks.ts` (pure): `scarsAt(marks, t, disc_kpc)` gathers the marks alive at the chart's time
  (`markStrengthAt` mirrors the kernel's `Mark::strength_at`) into a disc shape: warp, ripples, outer tilt, thick
  disc, expanding rings with emptied middles, lopsided and driven two-armed brightness, knots of new stars, and the
  other-band marks (gas trail in radio, warm side in X-rays) kept for words. `heightAt` and `brightnessAt` are the
  formulas over the disc; `faceOnPaths` and `edgeOnPaths` give Chart paths; `scarWords` names the strongest scars
  with how many encounters made them and since when.
- Views (galaxy chart, agreed with the galaxy and travel thread): `GalaxyScars` is a layer inside the face-on disc
  (the visible edge pushed out by lopsided and driven brightness, dashed; rings in the accent colour round their
  emptied middles; knots), and `GalaxyEdgeOn` is the disc seen edge-on under it (mid-plane, thin and thick discs,
  heights 5 times) with the words. Scrubbing the chart's time shows the disc damaged and healing.
- Marks overlap: the strongest live mark of a kind sets its size and direction; ripples from several passes share
  one union so they stay a few hundred parsecs high. Shapes are stand-ins (fudge register "galaxy-marks.ts
  SCAR_SHAPES").

## Checks

- `packages/renderer/test/galaxy-marks.test.ts`, 7 tests: strength rule; a warp is an integral sign (opposite
  sides opposite, flat inside its start, rise within its cap); ripples from 30 passes stay within 0.3 kpc; a ring
  expands at its set speed (from the sheet's year and parsec) round an emptied middle and fades; scars heal except
  the thick disc; paths fit their frames; the kernel's Milky-Way-like galaxy carries marks the words name.
- `tests/ui/galaxy.e2e.mjs` checks the scar layer and the edge-on caption.

Pictures: `GH9-screens/scars-over-time.png` (the Milky-Way-like galaxy at 1.6, 3 and 13.7 Gyr: punched rings and a
strong warp early, healing to ripples, a mild warp and a thick disc today) and `galaxy-night.png`.

## Next

- True style: draw the stars and gas themselves bent and lit by the same formulas (V9 True makeover), satellites
  drawn as small galaxies from their own G3 results, the gas trail in the radio band and the warm side in X-rays.
- Each mark's direction from the satellite's orbit, once the kernel writes one into the mark's shape numbers.
