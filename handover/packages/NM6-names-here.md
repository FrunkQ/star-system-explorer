# NM6 part 5: "Names here" for a whole system

Status: done, 24 Nov 2026 (thread "Build names gallery and life board").

## What it does

- `views/reference/src/NamesHere.svelte`, exported from `views/reference` as `NamesHere`: a self-contained block for
  any card that shows a system or place as one thing. Props: `session`, `workspace`, `address`, `label`, `depth`.
- It reads the place's own names (`read_names`) and lists each layer (the campaign's default layers, the place's
  layers, always Designation and Common) with the place's name and where it comes from.
- On the Build desk (GM): per layer, "Name with…" a scheme or "As set above" sends `set_naming` for the place and
  everything beneath. At Science and Engine, "For" limits it to one kind (planets, moons, stars, small bodies:
  `of_kind`) and "rename names already kept" sets `rename_fixed`. Typed names are never touched. A player seat's
  change goes to the GM, as every intent does.
- Mounted first on Reference, Names, under "This campaign", for the system the GM is in (when a pack carries names).
  The galaxy charts thread is asked to mount it on the sector and galaxy charts' system card (one line, its views),
  and the selector thread may mount it from the place line.

## Checks

- `tests/ui/reference.e2e.mjs` (desk, Build): Kestrel's block shows its names; common names for planets set to
  Oluen, then put back to "As set above", each with its note.
- Screen: project files `sse4/handover-screens/NM6-screens/names-here-desk.png`.
