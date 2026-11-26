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

- **Part 2a, routes, grid and heights** (24 Nov): `Session.linksIn` reads the kernel's `links_in`; `systemRoutes`
  joins the links between places in two campaign systems into one route per pair (`systemOf` matches the longest
  system prefix), one way only when every link runs the same way (dashed, with an arrow). The view has its own controls
  (`YOUR_SYSTEMS_CONTROLS`, chart from above only for now); HUD shows a grid at the scale bar's step (`gridLines`) and
  names. `layoutSystems` fits both a system and the foot of its stalk; the stalk runs from the map up (or down) to the
  system, and the e2e checks it joins.

- **Part 2b, Stars around home** (24 Nov): `HomeCell.svelte` beside the list, a closed "Stars around home" that reads
  `star_cell` when first opened and draws the home 10 pc cell's drawn stars from above (dot by the cube root of mass;
  ringed if promoted; coloured if a campaign system). Resting on one gives its short name, mass, companions and
  [Fe/H]. "Grow a system here" waits on the kernel: `from_star` on run_formation and adopt (formation from
  `Stars::birth` of that star; agreed with "Build kernel resolve and rules" 24 Nov, golden lane). No TypeScript copy
  of the birth physics.

## Honest limits
- Systems are not yet placed in the galaxy's own star cells (G5 pins and G7, the campaign's galaxy): a grown system
  has no place. A V3 map's unit is read from `v3.distanceUnit` when the read-back carries it; the universe-level fact
  is not in a system's read-back today, so the scale bar says "map units".
- The campaign's own system is counted only once it has been on the chart this session (it lives in the fixture).
- Not a place-line scale of its own yet (the navigation thread owns place.ts); it is an instrument at the galaxy
  scale, which the design allows to become the patch scale later.

## Next
Picker rows read `summaryWords` (navigation thread, NV2 part 2). Then "Add near here" (a star picked by what the GM
wants nearby) and the home star cell drawn behind the map itself from `star_cell`.

## Part 3: pencil and map layers (25 Nov 2026)
- Pencil and ink (SV3): on the Build and Run desks each system is drawn by what the table knows of it, read with
  `tableLandmarks`: ink, heard of (outline) or pencil (faint, dotted ring, muted italic name, "not known to the
  table" in the list and the hover). Explore and players draw ink. The e2e checks every placed system has a pen.
- Map layers (OV3, OV4): a "Map layer" picker when a stacked pack declares an area layer; `Session.holdingsIn` reads
  `holdings_in` under the universe; `placeOnMap` puts each held place at its campaign system; the reach comes from the
  layer (`between_stars.reach`, else 4 pc) in the map's units (`reachInMapUnits`), or as map units when the V3 map
  gives none (the hint says so). Slots by `assignSlots`; drawn by `Overlays` under the routes; `AreaKey` lifts a power.
  `your-systems-areas.test.ts` runs it through the kernel with a test layer pack and two held systems.

## Part 2b, Grow a system here (25 Nov 2026)
- On the GM's seat, pressing a drawn star in "Stars around home" keeps it said and offers "Grow a system at <star>".
  That is the kernel's `adopt` with `formation: { seed: <star address>, from_star: <star address> }` (PR 546): the
  system takes the star's address and its own birth (mass, age, metals), so the chart and the system you fly into
  agree. The list and the map re-read (workspace revision), the star fills as a campaign system, and an "Open it ›"
  link goes to the new system. A star already grown says so. `FormationRequest` gains the optional `from_star`.
- Players never see the button. The e2e grows the last drawn star and checks it joins the list and fills in the cell.
