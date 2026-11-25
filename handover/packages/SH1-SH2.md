# SH1 and SH2: the Roll call look-up and its screen (track SH)

Thread "Build roll call and In memory", 25 Nov 2026. Design: `design/SSE4-spaceflight-history-design.md` (accepted
25 Nov 01:20 UTC, "yes to all - stay restrained and respectful - one place we don't joke"), sections 3, 4 and 5.
Data: `store-packs/sol-today/crews.json` from RS9 part 1 (PR 506, thread "Build real spaceflight history pack").

## What landed

- **SH1, `packages/ui-contract/src/rollcall.ts`.** `rollCall(index, from_s, to_s, at_s)`: everyone in flight during a
  window (a day), grouped by place (nearest first: home, home's moons, on the way, other bodies, the star), then craft,
  then person, each row keeping its source's mark and why. One `RollSource` shape with two adapters:
  `sourceFromCrews(crews, logs)` (the pack) and `sourceFromManifest(entries, logs)` (any campaign: its people's dated
  `aboard.on` moves, in flight when the construct's log has it off the ground). Also `countSteps` / `countAt` (people in
  space through time, against the edge), `stepEvent` (‹ Event ›), `rollSentence`, and the empty-day words (last person
  down, next up, or "nobody had been to space yet").
  - The edge of space is a number (ruling 11, 100 km): a hop below it is listed with "by the 80 km line, below the
    100 km edge" and not counted; a hop whose height the source lacks is counted and says so; a flight with
    `reached_space: false` (RS9 part 2: STS-51L) is listed on its day and never counted.
  - A docked craft is placed where its host is ("docked to Mir"), through the host's `gcat` id.
  - Kinds of being are never assumed human: the sentence says "1 was in space" for an animal alone.
  - McDowell's "*" (the name a person went by) is stripped for display (`shownName`); nothing else in a name changes.
    RS9 part 2 strips it in the data too.
- **SH2, `packages/shell/src/rollcall/`.** `RollCall.svelte` (the panel in the card area: date as the serif title,
  press to type one; the sentence; ‹ Event, ‹ Day, Day ›, Event ›; the roster; "Also in flight" when every log is at
  hand; the credit), `feed.ts` (where the people and logs come from), `screens.rollcall.svelte.ts` (view
  `sse.view.roll-call`, screen `rollcall`), document `documents/rollcall.gm.json` (chart, roll call, ruler; rail entry
  system / travel / "Roll call", `needs: ["crews"]`).
  - **The front door, part 1 (SH8):** `?spaceflight=<folder URL>` names a "Sol today" folder (crews.json, pack.json,
    logs/). The pack stays on Explorers (Alex, 23 Nov); the feed fetches crews.json and pack.json once, then only the
    logs of the day's crewed craft and what they dock to. A pack opened on a day outside its span starts on its latest
    day. Without the parameter, Roll call reads the open campaign.
  - Shared shell edits (one line each, following the Run desk's pattern): `App.svelte` spreads `ROLLCALL_SCREENS` and
    lets the front door's folder satisfy the rail's `crews` need (`frontDoorGives`); `registry.ts` spreads
    `ROLLCALL_VIEWS`.

## Checks

- `packages/ui-contract/test/rollcall.test.ts`: 11 tests. A made-up campaign (Heron, Wren, Tern, Petrel, Gannet;
  invented names only) checks each rule. The real pack test runs once crews.json is on main: every crewed leg's craft
  has a log, and the look-up's count equals the pack's own `in_space` at every step, and a roll call at every 7th step
  counts the same people. No figure from a book is pinned (design 4.6).
- `tests/ui/rollcall.e2e.mjs`: serves `store-packs/sol-today` at `/sol-today/` as Explorers would, opens Roll call from
  the rail, types 20 July 1969, steps by day and event, reads an empty day before 1961, refuses a date it cannot read,
  and fits 390 px. It exits quietly while crews.json is not in the pack. Run locally in chromium (passes). **Not yet a
  CI line:** adding it to `.github/workflows/ci.yml` was stopped by a session permission check, which needs Alex's own
  yes. The line is `- run: node tests/ui/rollcall.e2e.mjs chromium firefox --screens ui-screens` after `time.e2e.mjs`.
- `tests/ui/constructs.test.ts` rail bound now includes the rollcall document.
- Screenshots: project files `sse4/handover-screens/SH-screens/SH2-*.png` (not in the repo).
- No golden change. Nothing here feeds physics or the state hash.

## Fudges

Two entries in `docs/fudges.md`, section "Roll call": days are read on TT (up to about a minute off UTC), and a craft
with no log piece at the clock's moment is placed at another moment of its day.

## Open and next

1. **Rail room (question for Alex):** the system rail is full at ten with Ships and stations (RAIL_MAX). Roll call's
   entry therefore `needs: ["crews"]` and shows only when a pack with crews is stacked or the front door names one; a
   campaign with its own flight logs reaches Roll call by its address (`#chart/system/<system>~rollcall`) but not yet
   from the rail. Recommended: keep it so until the rail owner (thread "Build object selector and menus") folds two
   travel entries together.
2. Campaign mode reads flight logs through `session.flightLog`, which arrives with PR 452 (travel thread); until then a
   campaign's people are found but nobody is "in flight".
3. Pack mode cannot count "Also in flight" (it fetches only the crewed craft's logs). Asked of the pack thread:
   `logs/index.json` (RS9 part 2, done there): the feed reads it, so hosts are found by gcat id; counting every craft
   in flight still needs either every log or in-flight spans in the index.
4. The front door opens the campaign's own system on the chart (the fixture Kestrel today), not Sol: the chart side of
   SH8 is to open Sol with the pack.
5. Next packages: SH3 (the count strip on the ruler, with the chart room), SH4 (a person's flights), SH5 (records as
   sifters, On this day), SH6 (In memory, after RS9 part 2's `in-memory.json`), SH7 (names carried, after
   `memorials.json`, RS9 part 3).
