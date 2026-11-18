# V3 colour coding, for the V12 symbology

Source: Alex's Star System Explorer 3 (FrunkQ/star-system-generator, `beta`, commit 53dd177, v3.1.80). Read only.
The data is in [`v3-colours.json`](v3-colours.json); this note says how V3 uses it.

## What colours a body

`getClassColor` in `src/lib/rendering/colors.ts` picks one colour per object, first match wins:

1. Stars by spectral key (O to Y, WD, NS, magnetar, BH). A red giant takes the K colour at 3700 K or hotter, else M.
2. Belts.
3. A habitability tag (`habitability/earth-like` or `habitability/human`): the habitable blue.
4. A biosphere: green.
5. Brown dwarf, then ice giant, then gas giant, by class.
6. Everything else: terrestrial orange.

With true-colour mode on, a body that is not an override shows its derived `apparentColorHex` instead. Constructs
use their own `icon_color`, falling back to the construct yellow.

The other domain colours (zones, orbit bands, the six-step hazard ramp, temperature bands, habitability tiers, fixed
vs derived data, magnetic fields) are CSS tokens in `tokens.css`, listed in the JSON. One mismatch in V3: the
red-giant token defaults to `#8b0000` but the renderer uses the temperature rule.

## The colour-coding screen

`/palette` ("Appearance") edits chrome groups (Surfaces, Text, Accent, Semantic) and domain groups (Body types, Star
spectral class, Zone bands, Orbital placement, Hazard ramp, Temperature band, Habitability tier, Data provenance).

## Where custom colours live

- **Not in the save.** Palette overrides are a `{ token: hex }` map in localStorage key `sse-palette-overrides`,
  applied as CSS variables on `:root`. Skins (`modern`, `classic`, `colourblind`/Clarity, `nebula`, `daylight`,
  `terminal`) and custom skins are in `sse-skin`, `sse-custom-skins` and `sse-float-skin`. They belong to the viewer,
  so in V4 they map to view settings, not campaign data.
- **In the save:** `coiCategories`, holding only tag categories the GM changed from the shipped set, each with its
  `color` and `textColor`. Per-tag colours are not in the save (the saved tag shape has no colour field).
- **Per object, in the save:** fluid layer and life layer `colorHex`, biosphere `colorHex`/`lightHex`, construct
  `icon_color`, engine `exhaust_color_hex` (all authored), and `apparentColorHex`/`apparentColor` (derived; V4
  recomputes it). See [`v3-save-fields.md`](v3-save-fields.md) for how the importer classes these.
