# X3e: Traveller screens in the chart room (sector map and world codes)

Track: Traveller pack. Stage 5 ("Run a game": import a subsector, open a world as a full system). Date 23 Nov 2026.

## What a GM can do

1. Workshop, Packs, **Add the Traveller pack** (a sample button; the pack is copied to `packs/samples/traveller.json` by `scripts/build-shell.sh`). Accept the preview.
2. Two rail entries appear, and only while a stacked pack gives a code sheet:
   - **Galaxy scale, Look: Sector map** (`sector.gm.json`, view `sse.view.sector`).
   - **World scale, This world: Codes** (`codes.gm.json`, view `sse.view.world-sheet`, with the body card).
3. Sector map: **Import a sector…** opens a panel from the button: paste, choose a file, or fetch by name (Traveller Map through the site Worker, attribution shown). **Preview** reads it through the code sheet and lists every world with refused lines, impossible values (kept and marked), guesses and codes that differ from the physics. **Write N worlds as history** applies it, optionally growing each whole system first. The map fills, opening on the one subsector in use.
4. Pick a hex: the hexes within **Reach** (in the pack's hex unit) light up, and the world's code sheet opens beside the map, every digit with its why, with **Open the system** and **Open the world** links (place hashes, so Back works).
5. **Find** runs the pack's map sifters (refuelling stops, refined fuel, red zone candidates).
6. **Export…** downloads the mapped systems as T5 tab, SEC or one line per world.

Click count: from the galaxy scale, the map is one click; a world's sheet one more; its system one more. Import is two (open, preview) plus the write.

## What changed

- Kernel: new op `sector_map {scope, sheet?, at_s}` gives `sector_mapped {sheet, worlds: [{system, body, hex, name, sector, reading}], skipped}`, by hex. `export_at` and it share one walk (`mapped_at`), so the map and the exported file always name the same main world. Contract requests: two `sector_map` lines appended (error cases on the toy stack); golden `contract.jsonl` re-blessed for those two lines plus the op list in `hello` and the unknown-variant message (new op only). Perf chunk: `sector_map` joins the galaxy chunk.
- `ui-contract`: `sector.ts` (typed answers, `SECTOR_FORMATS`, `mapFields`: a reading to hex map fields, groups as lists plus `<group>_first`; `stackProvides`). Session methods `codes`, `sectorMap`, `importSector`, `exportSector` (optional on `ViewSession`, GM only, at the campaign's now).
- Rail entries may say `needs: ["code_sheets"]` (any `provides.other` key); `railOf(screens, scale, has)` drops an entry whose needs the stack lacks; `checkScreen` checks the field. The shell passes `stackProvides` over the stack in force. This is general: any pack-brought screen can use it.
- Shell: `screens.codes.ts` (CODES_VIEWS, CODES_SCREENS) spread into `registry.ts` and `App.svelte` like `screens.more.ts`. Pack stack view: `sample_labels` param.
- Traveller pack: zone glyphs use `has` (zones are a list); the red-candidate sifter reads `zone_first`. The TG1 tags block now lives in `scripts/traveller-tags.json` and the build script writes it (it had been added to `pack.json` by hand and would have been lost on the next build). Pack hash changes, so `tests/golden/traveller-codes.json` was re-blessed: only its `pack` hash moved (every code line and the grown system are unchanged).

## Checks

- `tests/ui/sector.test.ts`: fields from a reading draw the pack's glyphs; sifters; opening block and export name; rail needs on and off; bad needs refused. `tests/ui/hexmap.test.ts` updated for list zones.
- Kernel: the grown-sector test reads `sector_map` after export (12 worlds, by hex, each with a UWP line).
- Not run here: browser e2e (no wasm-bindgen in this container). The walk above is not yet in an e2e; add one when the stage 5 walk is written.

## Rough edges

- The hex map is a flat chart without the shared view controls (rule 12: zoom and slide only). Comes with the next touch of the hex map.
- After an import the other screens learn of it through `ws.revision`; the system list in the place line picks up the new systems on its next read.
- The start sheet's "A Traveller sector" line still opens the campaign dialog; it should open the Sector map once the pack is stacked (chart room thread's start lines).
