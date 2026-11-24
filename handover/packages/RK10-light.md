# RK10, the light thread's share: the band set read from the sheet

RK10 (`handover/packages/RK10.md`, data sheet thread) put one Bands table on the master data sheet.
This share points the light code at it.

- `sse_light::table::LightTable::bands` is read from the sheet's `bands` table (its `light` rows,
  in order of wavelength) when the light pack gives none: `table::bands_from_sheet`. The stack's
  sheet is used when it carries nature's constants, else the base sheet, as for the gases.
- `packs/base-light` no longer carries its own `bands` copy. A pack may still give a `bands` list,
  which overrides the sheet (test `bands_come_from_the_sheet_unless_a_pack_gives_its_own`).
- The builder's agreement check (`band_rows.check_against_light`) is gone: there is nothing left
  to agree with.
- `sse-sense` has no band names of its own: an instrument gives its wavelength range, and RK11's
  `range_bands` are a setting's distance measure, not this table (as RK10.md says).

Golden: no change. Every band edge and name is the same as base-light's copy was, so no light
value changes and the contract golden stays as it was.

Next (other threads): `ui-contract` light.ts and view-controls.ts read `SHEET_BANDS` (renderer and
chart room); an instrument may name a sheet band id instead of its range, when a pack wants it.
