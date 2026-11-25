# SH5 Records and On this day (track SH)

Design: `handover/design/SSE4-spaceflight-history-design.md` sections 4.3 ("On this day") and 4.6 (records), ruling 4
(records are questions, not typed facts).

## Done (25 Nov)

- `packages/ui-contract/src/rollrecords.ts`, over the same `RollIndex` and count steps as the roster:
  - `wholeStays`: a person's counted flight legs that follow on with no gap joined into one stay (up in a Soyuz, the
    station, down in a Soyuz).
  - `rollRecords`: most people in space at once (the highest count step), the last day with nobody in space (a whole
    day before the last zero ended), the longest single stay, the most time in space in all. Each carries how it was
    worked out; stays still going are left out of the two time records, and the record says so.
  - `onThisDay`: the calendar day in view in every year of the pack, with the most people up that day and how many
    went up and came down (moving between craft is neither). Empty years are left out; 29 February only in leap years.
  - `rollcall.ts` gains `againstEdgeCounts` so the records count against the same edge of space as the roster.
- The Roll call panel has two folds at its foot, "Records" and "On this day, <day month>". A record's value goes to
  its day; a year goes to that year's same day. Durations are said in whole days by `sayNumber`. Screenshot: project files `sse4/handover-screens/SH-screens/SH5-desk-records.png`.
- Tests: `packages/ui-contract/test/rollrecords.test.ts` (a made-up campaign for each rule; the real pack only against
  its own count and stays). `tests/ui/rollcall.e2e.mjs` opens both folds and follows a record and a year.
- On the real pack these read, without any figure typed: most at once 20 (May 2023); last empty day 30 October 2000;
  longest stay 437 days (Polyakov); most in all 1,110 days (Kononenko). Not pinned in any test.

## Not done, and why

- **Not kernel sifters yet.** The design says each record is a saved sifter (the `sift` behind Find and Stories). The
  Sol today pack is read as a folder (`?spaceflight=`), not loaded into the kernel, so the records are pure functions
  for now. They become sifter documents when a pack's people and logs are campaign entities (with SH9's record
  sifters as pack data).
- "Farthest anyone has been from Earth", "people who have left Earth orbit" and "first of each kind" need every
  crewed log loaded and the kernel's positions; they wait for the same step.
- Records do not yet follow a changed edge of space on the Build desk (SH9 makes the edge a Build value).
