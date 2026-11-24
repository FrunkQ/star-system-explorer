# TA1: air in system True (giant bands, storms, polar cyclones, clouds and haze)

Track TA (`handover/status/true-air.md`), owner "Build giant bands, clouds and hazes". Audit: `handover/design/SSE4-v3-gaps.md`, third audit, items R1 to R3 and R6; top-ten items 1 and 2.

## What landed

- `packages/renderer/src/true-air.ts`:
  - `readTrueFace(session, system, body)`: one reader for every True view (the system view now, the size comparison next through the renderer's `trueLook`). Ground from the level-0 tiles (`trueSurface`, as before), then the weather op's field for today (six faces at level 0, 24 cells across each, about 0.4 s a world) laid over it. A world whose tiles are refused is drawn from its cloud decks alone.
  - `withAir` and `trueGiant`: each cell's three decks are two-stream slabs (cover times Bohren's reflectance, g 0.85), each tinted by its condensate's colour; the haze is one slab of the summed aerosol depth (g 0.7) in its own colour; the slabs stack bottom up by the adding method, as `weather.ts` does on the Body globe. Pixels blend the cells within three half-cell widths with a Gaussian, so the coarse cells do not show. Bands, storms (ovals, dark spots, outbreaks) and polar cyclones and polygons come out because the weather op already puts them in its cells.
  - `sheetColours`: condensate names from the weather op ("ammonia ice", "hydrogen sulphide ice") are matched to the sheet's substances by their names, phase words dropped and sulph/sulf alike, so a new sheet row is used with no list here. Colour is `optical.condensed_colour_rgb`.
  - `gasAbove` and `gasTint`: the gas above the decks as one `light_through_gas` passage (the light thread's recipe: n = p x / kT over one scale height, path factor 2) and the factor per channel it gives. Written and tested, **not wired yet** (TA4): the op's methane rows read about 10⁴ too strong (reported to the light thread 24 Nov; Jupiter would come out deep blue).
- `SystemChart.svelte`: the True reader calls `readTrueFace`; what a face could not show (a colour the sheet lacks, a giant's dark deep air) is said once in the HUD note.
- Fudge row "Air in True" in `docs/fudges.md`.

## What it looks like now (local build, WebGL2, 24 Nov)

Screens in project files `sse4/handover-screens/TRUE-atmos-screens/` (TA1-*). Jupiter and Saturn show bands and storm spots in grey-white; rocky worlds carry their clouds. This is physically right for what the sheet holds today: NH₃ and NH₄SH are nearly non-absorbing, so grey-white. Jupiter's brown and Saturn's gold come from a chromophore haze over the ammonia deck (Sromovsky et al. 2017), which the weather op does not yet make (W10a, queued on the climate thread) and whose colour needs the data sheet's n,k row (MP33). Titan's haze is the weather pack's display orange over a thin haze (optical depth 3, below the measured 5 to 10), so the ground still shows through; W10a moves it to the measured range.

Audit R3 lines for the bodies touched, on the local build: Jupiter bands yes, Great Red Spot no (an oval is drawn white: the chromophore), polar cyclones present in the field, flattening no (TA6); Saturn bands yes, gold no, hexagon in the field; Venus overcast yes, cream-yellow no (white: H₂SO₄ has no sheet colour yet); Earth clouds yes; Titan haze yes, orange weak and ground visible (TA3); Uranus and Neptune not blue (TA4).

## Tests

- `packages/renderer/test/true-air.test.ts`: names match the sheet whatever the phase word or spelling, and a sheet row without a colour gives none; a giant's zones are brighter than its belts, the bands mirror about the equator and ammonia's colour is warm; a haze is laid in its own colour and clear air leaves the ground as it was; no cells keeps the ground and gives a giant no face.
- `npm run ui:test` renderer files (142 pass), `npm run ui:check` (0 errors, 0 warnings).
- Browser: local `npm run build`, Sol, True, Readable and zoomed; every world read, no page errors on WebGL2 (headless WebGPU on SwiftShader fails to create buffers, unrelated).

## Integration

- The renderer's `trueLook` (`true-body.ts`, PR 442) should take the face `readTrueFace` returns; the size comparison (Lineup3D) can call `readTrueFace` itself so both views agree.
- Weather (W10a): a `chromophore` aerosol on giants and Titan's haze depth; naming the sheet id of an aerosol's material would let TA3 look up its n,k without a name match.
