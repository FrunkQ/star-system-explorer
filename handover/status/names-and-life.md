# Status: Names gallery and life board

Owner: the thread "Build names gallery and life board" (UI, from 24 Nov 2026). Covers: the rest of NM6 (the Classes
gallery, layer choice in lenses, "Names here" for a whole system) and U7 part 2 (the LifeFlow board). Code lives in
`packages/views/reference/`, `packages/views/biosphere/`, `packages/ui-contract/src/names.ts` and
`packages/ui-contract/src/classes.ts`. NM6 parts 1 to 3 and U7 part 1 are rows in `chart-room.md`.

Only the owner thread edits this file. One row per package: update it in the same pull request as the package. Logs
are in `handover/packages/`. Index and standing rules: `handover/START-HERE.md`.

| Package | State | Log |
|---|---|---|
| NM6 part 4, the class gallery (V3 gaps item 35): Reference "Names and classes" gains a Classes page. Every class of every class atlas on the stack, scheme by scheme (each scheme's question), as tiles with the chart's mark for it, base classes under their parents and modifiers apart; a class opens to its bands in words with units and fringe, what it needs beyond numbers, the named worlds it is checked against and its near misses. With no atlas on the stack the shipped ones are shown (SSE4 class atlas first, SSE 3 classic second) and the Build desk offers to add one. Checks: classes.test, reference.e2e (desk and phone) | done; the still and turning pictures wait on the kernel's `class_sample` (kernel queue item 6); atlas classes draw as plain marks until a symbology binds their ids | packages/NM6-classes.md |
| U7 part 2, the LifeFlow board: the Life screen gains Story and Board; Board draws the life pipeline (habitats, budget, origin, opportunity, catastrophes, innovations, ecosystem, traits, fluxes, signatures, hooks) as a wired flow from the kernel's `life_view`, each stage opening to its events with reasons and the dials it leans on; a list on a phone. Checks: life-flow.test, biosphere.e2e | done, read only; the knobs go live when `life_view` takes dials (kernel queue item 6) | packages/U7-part2.md |
| NM6 part 5, "Names here" for a whole system: `NamesHere.svelte` exported from views/reference (session, workspace, address, label, depth): each layer's name and source for the place, and on Build a scheme or "As set above" per layer for the place and everything beneath, for one kind if wanted, kept names too if wanted. Mounted on Reference, Names for the system the GM is in. Check: reference.e2e | done; mount on the sector and galaxy charts' system card is the galaxy charts thread's one line (asked) | packages/NM6-names-here.md |
| NM6 part 6, layer choice in lenses: a lens may carry `names` (designation, local or any layer); the lens editor offers it; the shell relabels the scene every view draws through read_names (`namesInLayer`, `relabel`). Checks: names.test, reference.e2e | done; the place line and picker keep common names; player seats read only the layers their view discloses | packages/NM6-lens-names.md |
