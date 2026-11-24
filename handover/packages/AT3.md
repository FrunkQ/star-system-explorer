# AT3: editions as data, the loader, the Edition switch

Track AT (atlas design sections 11, 12.1 to 12.6, 14A note 6). Built 24 Nov 2026.

## What it is
- Three shipped editions in `packages/ui-contract/src/atlas/editions/`: **Survey atlas** (default; look `paper`,
  plain words, gaps in plain words), **The Guide** (vocabulary `guide`, the Guide's section words, gaps in its voice,
  "nobody has looked yet"; look `deck` for now) and **Company** (numbered formal sections, barred gaps with one line
  saying what a bar means, wireframe plates, a letterhead template for AT6; look `glass` for now).
- `sse.edition/1` gains `vocab`: the vocabulary layer the page's words and sentences come from; `voice.labels` sit on
  top (`section.*`, `column.*`, `line.<field>`, `page.<kind>`, `prose.empty`, `note.bars`).
- `ui-contract/src/atlas/loader.ts`: `checkEdition(doc, stack) -> string[]` (checks 1 to 5: schema, `extends`,
  catalogued fields, no typed figure in any text, slots only in letterhead and print lines and only known ones, named
  looks, vocabularies and note sets exist, sizes) and `testRender(doc, scenes, vocabs, stack) -> string[]` (check 6:
  every figure matches the Survey atlas's, nothing shows that the audience lacks). Pure and DOM-free; plain sentences,
  an empty list passes. Agreed with "Build Orac, Minerva and Tars" for AT11. `typedFigures(text)` finds a number with
  a unit the units engine reads ("5 kg", "71%"), so section numbering ("1. Summary") is allowed.
- The Atlas view: an **Edition** switch (the GM's own, for browsing; a player's screen has none, 14A note 6). The page
  is dressed in the edition's look by the shell (`AtlasHost.lookCss`, from the player themes), so the view never
  names a theme. Switching re-dresses the same page in place.

## Interim choices (to revisit)
- The Guide's look is `deck` and Company's `glass` until AT7 brings the "Guide gadget" look (cheap electronics, cover,
  stepped wipe; 14A note 4) and AT6 brings letterhead, brand accent and ASCII art.
- Editions come from the shipped list only. Campaign and pack editions (`checkEdition` on load) arrive with the
  first pack that carries one, or with Orac's AT11.

## Checks
- `node --test packages/ui-contract/test/atlas.test.ts`: the shipped editions load and pass the test render;
  switching edition changes no figure (compared block by block, on the GM's and two seats' scenes) and gives the same
  gaps; the loader's refusals each say what to change.
- `tests/ui/atlas.e2e.mjs`: the Edition switch keeps the page and every figure, shows the Guide's words, and the
  Company edition bars what Everyone has not learnt.
- Screenshots: project files `sse4/handover-screens/AT3-screens/`.
