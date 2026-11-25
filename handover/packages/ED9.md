# ED9: the Makeup box (editing design section 11B)

## Part 1 (25 Nov): the bulk bar, Kinds for makeup, the radius the science expects

- `packages/ui-contract/src/makeup.ts`: `makeupOf(facts)` reads `makeup.metal/rock/carbon/ice/gas` (shares by mass,
  the fields the kernel's K3 reads), every part listed in the bar's order, scaled to sum to one; empty when none is
  set. `retradeMakeup(mix, part, share)` sets one part and scales the rest to keep their proportions (the Air box's
  `retrade`); a world with none set starts from Earth's 0.32 metal and 0.68 rock, the kernel's own default.
  `makeupEdits(facts, after)` gives only the fields that moved. `makeupKinds(packs)` reads Kinds for makeup from the
  class atlas (`packs/base-atlas`): the top level of the physical scheme, each class's first named example that belongs
  to it and carries makeup shares (Like Eros, Mercury, Venus, Ceres, GJ 1214 b, Uranus, Jupiter, Gliese 229 B). No
  share is typed here. `PART_WORDS` holds the Curious and Science words. Test: `packages/ui-contract/test/makeup.test.ts`.
- `Session.expects(system, address, fields)` wraps kernel K2 `expects` (GM only; nothing moves).
- `packages/views/editing/src/MakeupBox.svelte` opens from the Makeup handle (it was disabled as "soon"; the row now
  shows the two largest parts, or "Earth's (not set)"). It has the bulk bar, a chip per part that opens that part's strip
  (share, linear, 0 to 1), "Start from" chips for the kinds, and a pencil line: "With this mass and makeup the science
  expects a radius of X; it is set at Y", with "Set the radius to it" as one more change. On a what-if the line is not
  said, because K2 reads the main line. Changes go through `Handles.commitMix`, one `set_values`, one Undo, and "What
  followed" is filled.
- The kernel's radius follows the makeup only for a world with no radius set (K3). Every catalogued world has one, so
  the change is the pencil line and the button, not a silent move. A world mostly of gas has no radius from its makeup
  yet (no giant radius block in the base pack), and the box says so.
- e2e (`tests/ui/editing.e2e.mjs`, desk): Mars "Like Mercury" (metal 70%, expected radius about 3,210 km against the set
  3,390 km), one Undo; ices to 40% on its strip, one Undo. Screen: project files
  `sse4/handover-screens/ED9-screens/makeup-box-desk.png`.

## Next (part 2)

- The cutaway wedge beside the bar, its layers as results (kernel K9 for layers per sweep sample), each layer with why?,
  Reach it and Pin it.
- Dragging the bar's joins; tipping points on the joins from a sweep of `makeup.<part>` ("core as big as Mercury's",
  "gas past 1%: the radius swells"); radius, density and gravity in pencil while held.
- Science depth: heat (`k40_rel`, `r_process_rel`) and redox (`redox_iw`) strips; research-is-divided choices.
