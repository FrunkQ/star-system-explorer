# ED6: Kinds (part 1, Kinds in Add; part 2a, the star chooser)

Status: part 1 done 25 Nov 2026 (thread "Build planet and star editing"); part 2a done 26 Nov 2026 (thread "Make Explore try things at once", new account). Design: `handover/design/SSE4-editing-design.md` section 9 and question 8 (Alex 24 Nov 15:13: yes).

## Part 1: Kinds in Add round a star

- **Add round** a star (Build, More, Add, or `a`) shows **Kinds** between "What forms here" and "Something else": one world of each top-level class of the atlas's physical scheme (water rich, iron rich, rocky, sub-Neptune, ice giant, gas giant, brown dwarf), smallest first, each with its first named example's mass and size (rocky like Venus, gas giant like Jupiter), at the orbit typed above. `worldKinds(packs)` in `packages/ui-contract/src/add-here.ts`.
- The recipe is the class's own example in the pack that owns the class (`provides.other.classes[].examples[].values`), so a pack that adds a class adds a kind; classes whose example has no size (small bodies) are left out. The kinds come from the atlases on the stack, or the base atlas the shell ships (`packs/base-atlas.json`, as Names and classes offers it) when none is stacked.
- A kind goes in as authored values (question 4: hand-set inputs inside the physics are authored, not impossible), as one change with one Undo. The class is read off the results as always, so the label can change once it is in place.

## Part 2a: the star chooser (26 Nov 2026)

On kernel K18 `describe_star` (PR 592). A star's Mass handle now opens the chooser (`views/editing/src/StarChooser.svelte`, logic in `ui-contract/src/star-chooser.ts`, Kinds in `star-kinds.json`) in place of a plain strip:

- **The mass strip** runs over the life line's whole range (0.0124 to 100 solar masses, the walls `describe_star` returns), log scale, painted in the colour of each mass's light **at this system's present age** (`thermalColour` at the life line's temperature; clear for a black hole). So at 4.57 billion years the heavy end is white dwarfs, neutron stars and black holes, not blue stars: what a star of that birth mass would be today. Spectral letters sit under their runs; the walls (hydrogen fusion from 0.08, a neutron star from 8, a black hole from 25) are drawn and said in the units engine's words. The mark is the star's mass at birth.
- **The life bar** of the held mass (or the star's own): its phases along its life, the phase the system's present age falls in in the accent, the present age marked. Read only: the age moves with Look ahead on the time ruler (the design's "set_system_age" drag is not built; no such op exists).
- **Kinds chips** (Brown dwarf, Red dwarf, Sun-like, Blue giant, Red giant, White dwarf, Neutron star, Black hole): each kind is a phase (and spectral letters where it matters), and the chooser finds the mass that is that kind **now** on the life line (`kindMass`: the strip's samples, else a bisection for a short phase such as the giant branch, taking the middle of the window). Nothing typed: a red giant at 4.57 billion years is born at about 1.3 solar masses because that is what the fits say. A kind no star can be at this age (a blue giant in an old system) is offered greyed with the reason and the way to change the age.
- **Choosing** (drag along the strip, the arrow keys then Enter, or a kind) writes the star's mass now, radius, luminosity and temperature from the life line, plus `star.mass_msun` (the birth mass, the field a drawn star carries), as one change with one Undo (`starEdits`, through the handles row's `commitMix`). On Build it is kept; on Explore it is a try (TM4): the what-if opens by itself.
- `describeStar` on the session (`describe_star`, read only).

Honest notes:
- The catalogued planets of the Solar System carry their measured temperatures, so changing the Sun moves a hand-added world's temperature (the unit test: a world at 1 AU warms under 1.1 solar masses) but not Earth's or Mars's on the card ("nothing else this card watches moved"). Whether a changed star should override a measured temperature is a question for the physics, not this package.
- A what-if's own resolve of the Solar System takes several seconds in the browser, so on Explore the card's "what was set" line arrives a few seconds after the strip moves.
- The new browser test `tests/ui/star-chooser.e2e.mjs` is not yet in `ci.yml` (the coordinator said yes on 26 Nov, but this session's permission guard refused the edit twice; another session, or Alex, adds the line `- run: node tests/ui/star-chooser.e2e.mjs --screens ui-screens/star-chooser` after the manifest line).

Checks: `packages/ui-contract/test/star-chooser.test.ts` (letters; the strip ends; Sun-like near 1, a red giant found and checked to be a giant now, each other kind checked against its phase, no blue giant at 4.57 Gyr but one at 5 Myr; choosing for the Sun is one change, reopens at the birth mass, warms a world at 1 AU). `tests/ui/star-chooser.e2e.mjs` desk and phone (painted strip, walls, life bar, kinds greyed and live, Red giant kept with one Undo on Build, White dwarf as a try on Explore with the main line unmoved, nothing off the side on a phone). Screens: `sse4/handover-screens/ED6-screens/`.

## Still to do in ED6

- The chooser from Add and from "New system"; the full HR diagram at Science depth; dragging the age (needs a kernel op). Move the star Kinds into the class atlas once it has star classes.
- Changing a star that has worlds: Keep the worlds, or Grow the system again on a what-if (needs BC1 to show the branch resolved).
- A companion star (section 10).
- "A kind that cannot hold where you put it" advice (the sea boils off): wants the sweep (K1).

Checks: `packages/ui-contract/test/add-here.test.ts` (the base atlas's kinds, top level, smallest first, Jupiter for a gas giant, authored; no atlas gives none); `tests/ui/editing.e2e.mjs` (round the Sun, Add lists rocky, gas giant and ice giant, the gas giant like Jupiter). Screen in project files `sse4/handover-screens/ED7-screens/add-kinds-desk.png`.
