# V3 gap 16 part 1: the ship's log

Track: Traveller pack. Gap list: `handover/design/SSE4-v3-gaps.md` item 16 ("the log is a view of the campaign record, so it cannot drift from what happened").

## What it adds

- **Log** on a construct's card in Ships and stations (GM only). Newest first: when it was made, each edit of its facts, every change of a kept value (crew aboard, readiness, discipline, anything a pack keeps) with who wrote it and the reason given, each note at it, and when it was taken out of play. Dates in the stacked pack's calendar (the Traveller pack's Imperial dates) or the campaign's standard one.
- `ui-contract`: `shipLog(history, record, yr, label)` builds the entries from the kernel's `history` and `record` replies alone; `writerWords` says who ("the GM", "rule traveller.status.derelict"). A template's opening values fold into "Made". Session method `record(address, at_s?)`, GM only (seats get disclosures).
- Depth: Curious shows date and what; Science adds who; Engine adds the kept kind or event kind.

Nothing new in the kernel: the history and the record already held everything.

## Checks

`tests/ui/constructs.test.ts`: the log of a trader made, renamed, a crew member lost (with the reason), a note, a derelict trigger and scrapped, from the kernel's own replies, in order, with the writer named and the template's opening values folded into "Made"; with no record, the history alone.

## Next

- Journeys, cargo and refuels join the log as the voyage thread writes them to the record (as notes or kept values); nothing here needs to change for that.
- g-force and dose along a trip go on the voyage card (galaxy charts and travel thread); drift (a stopped ship coasting) is V8.
- "Facts edited" does not yet say which facts: `history` summaries carry no field list. A `fields` list on `EventSummary` would do it (a contract change, so with the kernel thread).
