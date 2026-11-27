# TM7: the stepper on the Run desk (track TM)

Status: done 27 Nov 2026, thread "Build the three desks". Design: `handover/design/SSE4-three-modes-design.md` section 3.2 and question 12 (accepted 25 Nov 2026).

## What it does

- On **Run** the place line is the stepper: **every level is drawn**, largest first. The one in view is the title, as before. A level below it that the trail remembers is pencilled with its name (a press goes back to it); a level not visited reads as the level's word, in italic, and a press goes **In** towards it.
- **Out** and **In** sit at the line's end, with **Page Up** and **Page Down** (not while typing, in a list or in a dialog). Out goes up one level. In goes down onto the target the trail remembers, else onto the body selected (at a system), else onto the level's way in (the galaxies nearby from the web, the galaxy from there, the campaign's system from the galaxy). With nothing to go into, In is disabled and says "Select something to go into".
- On **Explore** and **Build** the place line is as it was.
- New shell part `stepper` (desks: Run) in `SHELL_PARTS`; the shell asks `partShows("stepper", desk)`. `SHELL_PARTS`' note now says a form that changes on one desk is a part of its own.
- New `packages/ui-contract/src/stepper.ts`: `stepperLine`, `stepOut`, `stepIn`, over the place line's own `placeSteps`. `PlaceLine.svelte` takes `stepper`.

## Waiting on other tracks

- **Local stars and Site** are not levels yet (`PLACE_SCALES` has five). They join with GZ1; the stepper draws whatever `PLACE_SCALES` holds, so they appear with no change here. So today the stepper shows five steps, not the design's seven.
- **Travel** is the plain swap the place line already does. The grand zoom's short form (GZ4, about 0.6 s a level) and the held press's long form with stops replace it when they land.
- **A press on an unvisited level opens that level's picker** (design 3.2) is simplified to "In towards it"; the picker beside the current name still lists what is below.
- **Zooming past the edge** (question 13) is the grand zoom's.

## Checks

- `packages/ui-contract/test/stepper.test.ts`: every level drawn and unvisited levels named by their word; a remembered world pencilled and In going back to it; In onto the selection, the level's way in, nothing at a world; Out; the part is Run's only. `desk.test.ts` lists the new part.
- New `tests/ui/stepper.e2e.mjs` (added to `ci.yml` and `npm run ui:e2e`): Explore has no stepper; Run draws web, nearby, galaxy, system, world with the world unvisited and In disabled; selecting Tarn enables In; Page Down goes to the world, Page Up back, and the world stays pencilled as Tarn; Out goes to the galaxy and In back to the system.
- Seen red with the shell's `stepper` prop forced off (the Run desk never drew the stepper), green with it.
- Chromium: shell, walk, lenses, known, table, landing-table, bookmarks, playbook, steer, switchboard, delivery, hold-grounds, kits, object-menu, time, desk-memory, place, place-notes, picker, galaxy, cluster, foam, tactile pass.
- `npm run ui:check` clean.
