# MF14b part 1: the Mag layer on the Body screen

Magnetic fields design sections 8 and 9. UI track. Builds on MF14a part 1 (the renderer's tracer, Chart cut and
`fieldFromRead`, PR 166 and 176) and MF7's `read_field` fixture.

## What it does

- `packages/views/body/src/BodyMag.svelte`: a **Mag** toggle on the Body screen, under the globe. Pressing it reads
  `read_field({fixture: "solar-system"})` once (Solar System only, until MF2 publishes fields as facts of every world)
  and matches the body by name.
- The Chart cut (a plane through the magnetic axis and the wind's direction) as SVG: closed lines in two theme inks,
  `field-out` up to each line's farthest point and `field-in` after it (the field leaves the body on one side and
  returns on the other), open lines faint, the wind's bubble dashed, the spin axis solid, the magnetic axis dashed, the
  tilt, and "to the star" on the left. Lines are traced once per change of field, wind or tier (`FIELD_LINES[tier]`),
  never per frame. The text tier draws none and says so.
- Words from `narrateField`. A legend whose every entry opens its why; clicking a line opens the same why.
- Depth: Curious shows the picture, words and whys. Science adds the model, grade and stand-in notes. Engine adds line
  counts, Gauss coefficients, the drawn extent and `FIELD_REFS`.
- A world without field facts (Mars, every generated world) reads "No field facts for this world yet."
- Theme tokens `field-out` and `field-in` in every chart-room and player theme (one pair per theme, checked by the
  theme contrast and one-hue tests).
- The shell stack loads `base-reference` (the data sheet `read_field` needs for its constants); build-shell copies it.

## Checks

- `tests/ui/mag.e2e.mjs` (desk and phone): Sol door, Earth, Mag: lines in two inks, the bubble, the words, every why,
  no source at Curious, source at Science, coefficients at Engine, Mars has no field facts, Jupiter in Night chart
  keeps two inks, the state hash is unchanged (reading writes nothing), no phone overflow, no page errors.
- ui:check, ui:test, full ui:e2e. Screens in `/mnt/project-files/sse4/MF14b-screens/`.

## Not yet

- MF14c: Mag in the band switch, shown when the seat's lens lists a magnetometer (GM only until MF15 and U9).
- The 3D layer and True emissions (renderer, MF14a part 2), held with view styles.
- Fields for every world once MF2 publishes them as facts; the layer then reads them instead of the fixture.

## Also in this PR

- `packages/ui-contract/generated/reference-constants.json` regenerated: DM1 (PR 175) added the cosmos constants to
  base-reference while U23 (PR 174) was in review, so main's `sheet.test.ts --check` went stale when both merged.
  After any change to packs/base-reference, run `python3 scripts/build-ui-reference.py`.
- `tests/ui/setup.e2e.mjs`: the Packs screen now lists base-reference in the shell stack.
