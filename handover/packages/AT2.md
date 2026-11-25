# AT2: the Atlas on screen

Track AT (atlas design sections 4.1 and 4.2). Built 24 Nov 2026.

## What it is
- `packages/views/atlas/src/Atlas.svelte`: the GM's Atlas view (`sse.view.atlas`). Contents down the left (built from
  the audience's own scene by `atlasContents`), the page, step by the previous and next buttons (named after the
  pages they go to), the left and right arrow keys while focus is anywhere in the Atlas, or a sideways swipe on touch
  (a vertical drag still scrolls). Links on a page and the marks on its plate go to that page and select the object.
  Focus moves to the new page's title. Seen by: GM or Everyone; the page is built from that audience's scene
  (`session.scene(system, { audience, at_yr })`), read again when the record moves or the ruler moves.
- `packages/views/atlas/src/AtlasPage.svelte`: one page drawn from its block list; formats nothing. Gaps in quiet
  italic words, or a fixed-length bar for `gaps: "bar"`. The plate is a small SVG schematic (star and the subject's
  children in book order, each a link). Footer: edition, book, "as known to". The look transitions seam (AT15a):
  `[data-atlas-page]`, each block a direct child with `data-deliver-item`, prose and figures `data-deliver-text`.
- `packages/shell/src/atlas/`: `atlas.gm.json` (screen document with `rail` entries "Atlas" in Understand at the
  system and world scales) and `screens.atlas.ts` (`ATLAS_VIEWS`, `ATLAS_SCREENS`, spread into `registry.ts` and
  App's `SCREENS`, one line each). The view is given the plain vocabulary.
- `ui-contract/src/index.ts` exports the Atlas (`export * from "./atlas/index.ts"`): the boundary test lets views
  import only the client API. `label` and `contents` were renamed `editionWord` and `atlasContents` for that.
- Narrow boxes (a container query at 760 px, so a narrow panel on a wide screen folds too): the contents fold into a
  "Contents" button, the step buttons move under the page, and the page goes to one column.

## Not yet (and who)
- "Open in Atlas" in the shared action list: the navigation thread ("Build object selector and menus") was asked.
- Seen by one role: the table port lists seats, not audiences; the plan's audiences need handing to views (AT9 or a
  small App prop).
- The Edition switch is AT3; print is AT4; page turn for paper editions is AT4 with AT15a.
- The sector scale: the rail has no sector scale; the sector template is ready for when a scene's scope is a sector.

## Checks
- `node tests/ui/atlas.e2e.mjs --screens <dir>` (added to CI and `npm run ui:e2e`): the rail opens the system page;
  the GM's book has the system, worlds and moons; a link opens Tarn; arrow keys step and focus follows; Everyone's
  book has no page the GM's lacks; Night chart; the state hash never moves; on a phone the contents fold, choosing a
  page folds them again, and nothing runs off the side.
- `npm run ui:check` clean; `npm run ui:test` 600 pass.
- Screenshots: project files `sse4/handover-screens/AT2-screens/` (desk GM, desk Everyone, Night chart, phone).
