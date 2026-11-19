# X3c part 1: the hex map view

Status: view module done (19 Nov 2026), Traveller pack track. Design: handoff 7A part 3; build plan X3c. Importers and exporters: `X3c-io.md`. Not yet: a GM screen for it and a kernel scene op (see Limits).

## What was built (`packages/views/hex-map`, `sse.view.hex-map`)
- `grid.ts`, pure: a grid configured by data (`GridConfig`): block size (subsector 8 x 10), blocks per sheet (4 x 4), label form (`{c2}{r2}` gives 0101; `{c}.{r}` gives 1.1), which columns sit half a hex lower, what one hex is (1 pc). `label`/`parse`, `blockOf`/`blockHexes`, `cube`, `distance` (offset to cube, Patel's hexagonal grids note), `within` (jump-range hexes), `centre`/`corners` (flat-topped).
- `glyphs.ts`, pure: glyph rules as data (`when` a field is/in/gt/has, `mark`, `at` a spot, optional text from a field, a tone token). `marksFor` gives a world's marks, first rule wins per spot. Colours come from theme tokens only (`--sse-*`, caution and danger tones).
- `HexMap.svelte` and `index.svelte.ts`: the view: hexes with numbers, worlds with their marks and names, zone rings, the selected hex and its range highlighted; text tier lists the worlds as a table. Params: `hex_size`, `tier`.
- The Traveller pack configures it: `provides.other.views["sse.view.hex-map"]` (grid, and glyphs for starport letter, wet/dry world, belt, gas giant, naval/scout/way-station/depot bases, amber and red rings).

## Checks (`node --test tests/ui/hexmap.test.ts`, in `npm run ui:test`; `npm run ui:check` clean)
- The pack's grid equals the shipped default; sheet 32 x 40.
- Labels round trip; subsector letters at the corners (0101 A, 0810 A, 0901 B, 0111 E, 3240 P).
- Distances: neighbours of an upper and a lower column; jump-2 from 0505 is 19 hexes; corner 0101 has 2 neighbours; symmetric.
- Glyph rules give the right marks for a world with an A starport, two giants, naval and scout bases and an amber zone, and a belt.
- A non-Traveller grid (6 x 6 blocks, `{c}.{r}`, odd columns lower, 10 ly hexes) works with the same functions.

## Limits
- Not on a GM screen yet: it registers through `packages/shell/src/screens.more.ts` (MORE_VIEWS, MORE_SCREENS) once that hook lands with PR 26.
- No kernel op builds its scene yet; the importer preview (X3c io) and the `codes` op (X3a part 2) will.
- Routes and allegiance borders are in the scene type but not drawn yet.

# Map sifters (X3a part 9 on the hex map, 19 Nov 2026)
- `packages/views/hex-map/src/sift.ts`, pure:
  - `mains(cfg, worlds, jump, min)` finds chains of worlds a jump apart, largest first.
  - `chokePoints(cfg, worlds, jump)` finds worlds whose loss splits a chain. It uses articulation points with an iterative search.
  - `sift(sifter, worlds)` runs a pack filter (`any`/`all` of the glyph `When` conditions, with an `explain` sentence).
- The Traveller pack's hex map config adds three sifters: good refuelling stops (a gas giant or surface water), refined fuel (class A or B starport) and red zone candidates (not zoned yet, but corrosive or insidious air, or extreme law).
- Checks are in `tests/ui/hexmap.test.ts`: a chain with its choke points (column 2 sits half a hex lower, so 0204 touches 0104), jump-2 joining more, and the three pack sifters.
- Not yet: the "worlds whose codes hide a contradiction" sifter. It comes from the `codes` op (authored digits that disagree with the physics) and belongs on the world sheet's list mode.
