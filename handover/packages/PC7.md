# PC7: the quote set for held screens

Track PC (postcards design, `SSE4-postcards-design.md`, accepted by Alex 25 Nov 2026), owned by the Atlas thread.
Built 25 Nov 2026.

## What it is
- A margin line may be a quotation: `by` (who said it) and `source` (a book, a mission, a year) in
  `ui-contract/src/atlas/margin.ts`. A quotation is someone's words, shown with their name, never the Atlas's claim:
  the loader does not read figures in its words or its source, allows no slot in it, and lets it run to 320
  characters (it stands alone on a held screen). A source without a speaker is refused.
- `ui-contract/src/atlas/notes/space.quotes.json` (`sse.quotes.space`, exported as `SPACE_QUOTES`): 21 real
  scientists' and astronauts' words from SSE3's `static/space-quotes.txt` (Sagan, Hawking, Tyson, Tsiolkovsky,
  Hubble; Armstrong, Gagarin, Aldrin, Ride, Collins, Shepard, the Apollo 11 plaque). Sources only where sure; a
  quote known only by attribution says "attributed".
- `store-packs/space-quotes-fan/` (`fan.quotes.space`): the other 51, from films, songs, games, fiction and humour,
  each credited. A free unofficial pack for Explorers (question 7: Alex, 25 Nov, "yes to all").
- Every quote has `uses: ["waiting"]`. `pickBanners(scene, null, "holding", [SPACE_QUOTES], opts)` returns a line
  with `by` and `source`. Section 12A: on a held screen a quote stands alone (no joke or waiting line under it); a
  universe pack may bring its own quote set under `provides.other.atlas.note_sets`.

## Not in this package
- Drawing the quote card over a postcard on the held screen: the Run desk side of track PC.

## Checks
- `node --test packages/ui-contract/test/atlas.test.ts`: both sets load; every quote has a speaker and the waiting
  use; all 72 of SSE3's quotes are in one set or the other; a source's year passes; a slot in a quote, and a source
  without a speaker, are refused; the hold pick carries the speaker.
