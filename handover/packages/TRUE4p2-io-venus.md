# TRUE4 part 2: Io's sulphur and Venus's cloud droplets from measured spectra (26 Nov 2026)

Asked by Alex ("finally getting some colour on our planets now?"), scoped by the coordinator: make Io's yellow ground and
the colour of Venus's cloud droplets come from the data sheet rows PR 611 (MP36) added, through the sheet's Look up, in
the real drawing paths.

## What changed

- **Stains can be measured spectra** (`packages/renderer/src/stain-look.ts`). `stainHue` reads a row's
  `optical.reflectance` (MP35's shape) when it has one, at the three bands True draws, the same way bare ground is read
  (`ground-optics.ts` `bandsOf`). Such a stain is laid on by areal mixing, brightness and all: (1 − share) × ground +
  share × the stain's reflectance. Rows with only n + ik keep the old hue-on-the-ground's-brightness path (tholin, Mars
  dust). The `sulfur` row (alpha S8, USGS GDS94) is the kernel's stain id for Io's volcanic sulphur
  (`crates/sse-surface/src/stains.rs`), so nothing else needed naming.
- **One reader for stains** (`readStains`) and **`stainTiles`**, which bakes stains into a tile's `ground_rgb` for views
  that colour straight from tiles.
- **Venus's droplets** (`true-air.ts`, `cloud-rows.json`). The weather op names Venus's deck "sulphuric acid"; no sheet
  substance had a colour for it, so it was drawn white by default. Now `readCloudRows` asks the kernel's
  `light_through_gas` (particles, sse-light haze.rs: anomalous diffraction and the adding method) for a thick deck of
  1.05 µm droplets of `sulfuric-acid-75`, and `sheetColours` uses that. The answer is white (0.997 in each channel):
  the acid does not absorb visible light. So Venus looks the same as before, but now for a measured reason. Its real
  pale yellow comes from the unknown ultraviolet absorber, which has no sheet row.

## Drawing paths covered

| Path | Io's sulphur | Venus's droplets |
|---|---|---|
| System True (3D chart in True, the size comparison, the class gallery: `readTrueFace`) | yes | yes |
| Body globe (`views/body/src/surface.ts`) | yes (stained tiles) | no: the Body globe's weather pass (`weather.ts`) draws every cloud white by its own rule, which for acid matches the measured answer |
| Land (`views/surface/src/Surface.svelte`) | yes: the globe tiles and the finer patches near the site | not applicable (no deck drawn from the ground) |

## What it shows, and why Io is only faintly yellow

The kernel gives Io a sulphur coat of **21%**, because its heat flow is 0.43 W/m² there (stains.rs: share =
(q/(q + 0.5))²). Io's measured heat flow is about 2 to 3 W/m² (Veeder et al. 2012), which the same rule turns into
about 70 to 80%. The renderer now draws whatever share the kernel gives from the measured spectrum; the shortfall is
the kernel's tidal heating for Io, not the colour. That is the next thing to fix (proven core, heat flow), not a number
to tune here.

Before and after (project files `sse4/handover-screens/TRUE4p2-screens/`, Chromium with SwiftShader, desk):
- `io-system-true-before.png`: lunar grey (mature highland soil, the sulphur stain "not drawn: no optics").
- `io-system-true-after.png`: brighter and faintly yellow-khaki (21% alpha S8 over the grey).
- `venus-system-true-before.png`, `venus-system-true-after.png`: the same, as expected.
- `io-venus-before-after.png`: all four side by side.

## Still honest gaps (no values invented)

- Io's white SO2 frost, its red and orange short-chain sulphur, and beta S8: no rows (MP36's searched list). They draw
  nothing of their own.
- Jupiter's brown chromophore: still the weather pack's colour (no measured k on the sheet).
- Venus's yellow: the unknown UV absorber has no row.
- Europa's trailing-side darkening (Io's sulphur reaching it) is not done; the kernel would need to give Europa a
  sulphur stain, and the renderer would draw it with no further change.

## Checks

- `packages/renderer/test/stains.test.ts`: Io's sulphur from the sheet's spectrum is bright and yellow, laid on whole
  (absolute bounds on the reflectance); stained tiles carry it, the sea is untouched. Seen red with the reflectance
  branch removed (2 of 6 failed), green with it.
- `packages/renderer/test/true-air.test.ts`: cloud-rows.json names rows with n,k on the sheet; the kernel request names
  the row and droplet size; "sulphuric acid" is coloured only through the droplets.
- `crates/sse-light/src/haze_tests.rs`: a thick deck of the sheet's 75% acid droplets is white (each channel > 0.99).
- No golden moves: no kernel code or pack changed.
- Class gallery: renderer files changed, so the pre-drawn globes' look fingerprint no longer matches and the gallery
  draws live until `node scripts/build-class-pictures.mjs` is run again on a GPU machine.

Fudge register: "Measured-reflectance stains in True" and "`renderer/src/cloud-rows.json`". Citation: Hansen and
Hovenier 1974 in `docs/citations/surfaces.md`, unverified (from memory; Consensus out of allowance until 1 Dec).
