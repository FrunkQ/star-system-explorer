# RK10 part 2: the interface's band strings read the sheet

Part 1 (`handover/packages/RK10.md`) put one Bands table on the master data sheet (`packs/base-reference`, exported to
the interface as `SHEET_BANDS`). This part repoints ui-contract at it.

## What changed
- `packages/ui-contract/src/light.ts`: `BANDS` is the sheet's light rows (id, name, from_m, to_m), no longer a typed
  list. The hand-written `BAND_OF` map (light band to view band) is read off the sheet: a light row's `view` names a
  view row, whose `view_id` is the switch's own id.
- The radio edge found in part 1 is gone: radio now runs to 10 km as on the sheet and in `packs/base-light` (the typed
  list stopped at 10 m, so a recipe channel at 100 m read as Visible).
- `view-controls.ts` already took its band words from the sheet; `VIEW_BANDS` stays a typed list because it is a
  TypeScript type, and `view-controls.test.ts` checks it equals the sheet's view rows.

- Renderer (agreed with the renderer thread): `galaxy-true.ts` `GalaxyBand` is the view bands less Mag and
  `GALAXY_BANDS` is the sheet's view rows that some light row names (its per-band drawing branches are kept);
  `ship.ts` `bandOf` reads the light rows and says each row's plain words (so a 0.1 mm photon is "far infrared", where
  the typed edges called anything up to 1 mm "infrared").

## Tests
`packages/ui-contract/test/light-bands.test.ts`: BANDS equals the sheet's light rows end to end; each light band is
seen in the view band its row names; a long radio channel reads as Radio.

## Left
- `star.ts` (line 227) and `live-star.ts` (lines 127, 131) type `light` as "visible" | "xray": they belong to "Build
  star glow and flares" now; a one-line retype to `ViewBand` when that thread is free.
- The light thread's `packs/base-light` copy (their row in part 1).
