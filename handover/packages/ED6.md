# ED6: Kinds (part 1, Kinds in Add)

Status: part 1 done 25 Nov 2026 (thread "Build planet and star editing"). Design: `handover/design/SSE4-editing-design.md` section 9 and question 8 (Alex 24 Nov 15:13: yes).

## Part 1: Kinds in Add round a star

- **Add round** a star (Build, More, Add, or `a`) shows **Kinds** between "What forms here" and "Something else": one world of each top-level class of the atlas's physical scheme (water rich, iron rich, rocky, sub-Neptune, ice giant, gas giant, brown dwarf), smallest first, each with its first named example's mass and size (rocky like Venus, gas giant like Jupiter), at the orbit typed above. `worldKinds(packs)` in `packages/ui-contract/src/add-here.ts`.
- The recipe is the class's own example in the pack that owns the class (`provides.other.classes[].examples[].values`), so a pack that adds a class adds a kind; classes whose example has no size (small bodies) are left out. The kinds come from the atlases on the stack, or the base atlas the shell ships (`packs/base-atlas.json`, as Names and classes offers it) when none is stacked.
- A kind goes in as authored values (question 4: hand-set inputs inside the physics are authored, not impossible), as one change with one Undo. The class is read off the results as always, so the label can change once it is in place.

## Still to do in ED6

- Kinds on the star chooser (Mass and Age strips with Red dwarf, Sun-like, Blue giant, Red giant, White dwarf, Neutron star, Black hole chips); the base atlas has no star classes yet, so these need star recipes in a pack first.
- Changing a star that has worlds: Keep the worlds, or Grow the system again on a what-if (needs BC1 to show the branch resolved).
- A companion star (section 10).
- "A kind that cannot hold where you put it" advice (the sea boils off): wants the sweep (K1).

Checks: `packages/ui-contract/test/add-here.test.ts` (the base atlas's kinds, top level, smallest first, Jupiter for a gas giant, authored; no atlas gives none); `tests/ui/editing.e2e.mjs` (round the Sun, Add lists rocky, gas giant and ice giant, the gas giant like Jupiter). Screen in project files `sse4/handover-screens/ED7-screens/add-kinds-desk.png`.
