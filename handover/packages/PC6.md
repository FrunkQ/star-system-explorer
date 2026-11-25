# PC6 part 1: postcard sets, the base set's first cards, and the script that makes them

Design: `design/SSE4-postcards-design.md` sections 7 and 12A point 6 (a universe has a default set).

## What
- **`sse.postcards/1`** (ui-contract `postcard-sets.ts`): a pack's `provides.other.postcard_sets` is a list of sets
  `{ schema, id, title, cards: Postcard[] }`. `postcardSetsOn(packs)` reads them in stack order. A later pack's set
  with the same id replaces the earlier one. Bad cards are dropped with one line each.
- **The base set** is `sse.base` ("From the base set") in `packs/base-presentation/postcards.json`, a set file beside the base
  presentation pack (served as `packs/base-postcards.json`). It is served
  beside the shell and is not on the stack, so there are no state hash or golden changes. It has three cards so far,
  all of the sample save's Earth and Moon in True. Each card is a clip with its origin, so it draws live where it is
  opened, and each carries a 640 px stamp, so a held screen can show it without drawing.
- **The wall** shows each set after the campaign's postcards and visits (`shell/src/postcards/sets.svelte.ts`). A set
  card opens read-only and draws through the visiting kernel (PC4 part 2).
- **Held screens**: App passes `sets: () => postcardSets.list` to `share.setGrounds` (PC5's seam), so the base set
  is the default ground set.
- **`scripts/build-postcard-set.mjs`** makes the set in the built shell, the way a GM would make one. For each
  place it opens a fresh browser (a page holds only so many drawing contexts) and imports the sample save. It then
  picks the world on the chart (a moon inside its planet's chart), switches to True, turns the globe by keys, Take
  photograph, Keep as postcard, then on the wall Send and Save file. It names and captions the card, credits SSE4,
  drops the message, and writes the set. `--check` reads the set and says whether every card has its stamp and
  origin. Run it after `scripts/build-shell.sh`. It takes about 2 minutes a card.

## Checks
- `ui:test` covers the reader, the replace rule, and the shipped set (every card has a stamp and origin, and no
  message).
- The `postcards` e2e opens the wall in a fresh browser, sees the base set with its stamps, and checks that a card
  draws live.
- Screenshots are in project files at `sse4/handover-screens/PC-screens/build/desk-7-base-set.png` and
  `base-set-stamps.png`.

## Next (part 2)
- Grow the set to about 24, spread across scales and bands, with the trailer thread (STORYBOARDS.md section H,
  SHOTLIST.md). Add places to `PLACES` in the script and run it again.
- System 3D cards (PR 564's stills) need a step in the script: the 3D view style, then a bookmark from the
  Bookmarks list (that is a postcard already), then the wall.
- Galaxy, web and surface stills wait on PC2's next parts.
- Kestrel's Tarn has no surface: the kernel builds surfaces only for systems in the campaign that were grown or
  imported. It joins when that changes.
- A universe pack's own set: the reader already takes it (last set wins). Making the wall open on that universe's set
  first is still to do.
