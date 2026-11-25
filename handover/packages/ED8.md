# ED8: the Air and Liquids boxes (editing design section 11A)

## Part 1 (25 Nov): the Air box's mix

- `packages/ui-contract/src/air.ts`: `airMix(facts)` reads `atmosphere.composition.<gas>` (volume fractions, the fields
  the kernel's SOL5 reads), largest first. `retrade(mix, gas, share)` sets one gas and scales the rest to keep their
  proportions (a new gas joins the same way; sum one). `mixEdits`, `formulaOf` ("co2" to CO₂), and
  `substanceNames(packs)` (plain names from `provides.reference.substances` on the stack; `AirBox` fetches
  `packs/base-reference.json` when the sheet is not stacked). Test: `packages/ui-contract/test/air.test.ts`.
- `packages/ui-contract/src/edit.ts`: `commitFields` sends several fields of one body as one `set_values` (one authored
  event); the undo stack holds it under the field `FIELDS` with `{field: value}` records, and Undo sends `set_values`
  back. On a what-if it is one `branch_set` per field.
- `packages/views/editing/src/AirBox.svelte`, shown under the Air handle's strip (Handles): the mix bar (gases of 1% or
  more, labelled from 8%), trace chips in ppm, and a gas's own strip (share on a log scale from 1e-8 to 1) when pressed.
  `Handles.commitMix` refreshes the card and fills "What followed".
- Honest note in the box: the mix does not change the climate yet. Checked: Mars at 50% CO₂ changes no result the card
  watches. The surface chain does not read an authored mix (kernel ask K7).
- e2e (`tests/ui/editing.e2e.mjs`, desk): Mars's bar and chips; typing 50% for CO₂ gives N₂ 27% and Ar 20% (proportions
  kept), then one Undo. Screen: project files `sse4/handover-screens/ED8-screens/air-box-desk.png`.

## Part 2a (25 Nov): Kinds for air, Add a gas

- `airKinds(sol)`: every body of `packs/base-sol` with a measured surface pressure and mix gives a kind, "Like Venus",
  "Like Earth", "Like Mars" or "Like Pluto", thickest first, with its source. None of the figures are typed. The pack
  has no Titan air, so there is no Titan-like kind until base-sol carries one. `airKindEdits` sets the pressure and
  the kind's gases, and sets any gas the body has that the kind lacks to nothing. It is one change.
- `gasNames(packs, also)`: what "Add a gas" offers. It takes the sheet's substances that are not ions and have phase
  data placing them in an air (a triple point, critical point, vapour pressure or saturated liquid), and leaves out
  anything boiling above 300 K, which drops rocket fuels and peroxide. It adds the gases any catalogued air holds. A
  gas joins at 1,000 ppm (`retrade`) and its strip opens.
- e2e: "Like Earth" on Mars gives N₂ 78% and "Like Earth" in What followed, then Undo. Adding methane gives a CH₄ chip
  at 1,000 ppm, then Undo.

## Part 2b (queued)

- Add a gas in three groups (what the rock gives off, what it can hold, what escapes, with the time it takes).
- Drag a join of the bar to trade two neighbours; tipping points on a gas's strip (fires burn, toxic, freezes out).
- Kinds for air (Earth-like, Mars-like, Venus-like, Titan-like, first hydrogen air) and "Reset to what the science
  expects" (the atmosphere advice thread's "Suggest an atmosphere").
- The Liquids box (phase bars, Add a liquid) after kernel K8.
