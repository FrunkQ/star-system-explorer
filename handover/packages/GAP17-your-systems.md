# V3 gaps 17 and 39, part 1: Your systems, and what a system holds at a glance (24 Nov 2026)

Owner: "Build galaxy charts and histories" (status `handover/status/galaxy.md`). U6 part 2b, part 1.
Design: `handover/design/SSE4-v3-gaps.md` items 17 (a campaign map of your own systems) and 39 (hover a system to
see what is in it); Alex accepted both for stage 2 (23 and 24 Nov).

## What was built
- **`ui-contract/src/system-summary.ts`** (`systemSummary`, `summaryWords`): what is in a system from what the caller
  already holds (scene subjects, or the picker's chips): stars, planets, moons, minor bodies, belts, rings, ships and
  stations, life (only from a life, biosphere or inhabited tag or a Life chip; never guessed), and GM-only notes
  (a binary or more, a remnant, brown dwarfs, ringed worlds, bodies marked impossible or fictional). No kernel call.
- **`ui-contract/src/your-systems.ts`**: `readCampaignSystems(session)` is the loader the picker and the map share
  (the `systems` list, then one `resolved_system` per system with no tiles); `mapPoint`, `mapUnit`,
  `layoutSystems` (one scale on both axes, y up, unplaced systems left out), `mapScaleBar`.
- **View `sse.view.your-systems`** (`views/galaxy-chart/src/YourSystems.svelte`) on screen `systems`
  (`shell/src/documents/scale-systems.gm.json`): a second instrument at the galaxy scale on the rail, "Your
  systems". Each system sits where its own data puts it (a V3 starmap's `position`, kept by the importer as
  `system.position_map_units`), with a stalk for height; the rest are listed beside the map as not placed yet,
  never put somewhere made up. Resting the mouse on one shows the peek (title, counts, remarkable notes, where);
  pressing one opens it (a plain place link, `#chart/system/<address>`). Wheel zooms, drag slides, double-click
  resets (interface rule 11). The campaign's own system (the fixture) is counted from the scene once the chart has
  shown it this session.
- Tests: `ui-contract/test/system-summary.test.ts`, `ui-contract/test/your-systems.test.ts`,
  `tests/ui/your-systems.e2e.mjs` (grow one, add the sample save, the sample placed, the grown one listed, the peek,
  press to open, night chart, phone without sideways scroll). Screens in project files
  `sse4/handover-screens/GX-screens/` (not in the repo).

- **Names here** (24 Nov, with the names thread's NM6 part 5): beside the map every system has a row (On the map,
  Not on the map yet) with a "Names here ›" door that opens `NamesHere` from `views/reference` for that system's
  address (read its names; on Build set the scheme per layer). The e2e opens it on the sample.

## Honest limits
- Systems are not yet placed in the galaxy's own star cells (G5 pins and G7, the campaign's galaxy): a grown system
  has no place. A V3 map's unit is read from `v3.distanceUnit` when the read-back carries it; the universe-level fact
  is not in a system's read-back today, so the scale bar says "map units".
- The campaign's own system is counted only once it has been on the chart this session (it lives in the fixture).
- Not a place-line scale of its own yet (the navigation thread owns place.ts); it is an instrument at the galaxy
  scale, which the design allows to become the patch scale later.

## Next
Picker rows read `summaryWords` (navigation thread, NV2 part 2). Part 2: routes between systems as authored links
(HW1), "Add system here" and "Add near here", the home star cell behind the map from `star_cell`.
