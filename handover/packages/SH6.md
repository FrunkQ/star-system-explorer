# SH6: In memory, and the day of a loss (track SH)

Thread "Build roll call and In memory", 25 Nov 2026. Design: `design/SSE4-spaceflight-history-design.md` section 8
(rules of restraint 8.1 are standing; Alex: "stay restrained and respectful - one place we don't joke"). Data:
`store-packs/sol-today/in-memory.json` and the `death` facts on people in `crews.json`, RS9 part 2 (PR 506, thread
"Build real spaceflight history pack"); the pack points to it as `provides.other.in_memory`.

## What landed

- `packages/ui-contract/src/inmemory.ts`: `inMemory(doc, crews)` builds the page: three parts in the design's order
  ("On a flight": `during` flight; "Preparing to fly": training and ground_test; "Working to send others":
  ground_work), each in date order, and animals that died on flights in a short separate part at the foot (ruling 6).
  Names are the pack's (pack people by id, ground losses by their own sourced `names`), with the "*" marker removed;
  `unconfirmed` gives the pack's own words where there are more dead than names. `inMemoryLine` computes the first line
  ("N people have died...", or "Between L and H" when a toll is disputed). `recordDate` writes the record's own date
  with the calendar's months (not the TT instant, so a night landing keeps its record date). `correctionUrl` prepares a
  GitHub issue naming the entry and asking for two sources; opening it sends nothing (rule 7). `lossesOn` finds a day's
  losses.
- `packages/shell/src/rollcall/InMemory.svelte`: the page, in the Roll call panel ("In memory ›" at the panel's foot).
  One quiet column, serif names, ink on the ground colour, no pictures, no accent colour, no motion. A date goes to that
  day in Roll call. Sources for every entry in a fold at the foot, then "Something wrong or missing?". The page does not
  change with the date in view.
- The day of a loss (8.4): under the day's roster, the pack's own sentence and a link "In memory".
- `feed.ts` reads the pack's `in_memory` file when `pack.json` names it.

## Checks

- `packages/ui-contract/test/inmemory.test.ts`: a made-up record (invented names) checks the parts, animals apart,
  the range line, dates, the correction link. The real record (skipped until it is on main) checks every loss has two
  sources and a sentence, every ground name two sources, every person id is in the crews. No figure from a book.
- `tests/ui/rollcall.e2e.mjs` opens In memory when the pack gives it: the first line, the parts, at least two sources
  per entry, the correction link opening in a new tab; a date goes to that day, which names the loss; back again.
- Screenshots: project files `sse4/handover-screens/SH-screens/SH6-*.png`.

## Notes for the data (sent to the pack thread)

- A few names in McDowell's form read oddly when shown as given ("Bondarenko,Valentin", "Bassett, Charles Arthur,II").
  The page shows the pack's text unchanged apart from "*"; tidying them is a data change in the pack.

## Next

- Person links on the names (SH4, a person's flights) and "Where their names are ›" (SH7, memorials.json, RS9 part 3).
- Earth's memorials as text (ruling 10) show when the pack gives `earth_memorials`.
