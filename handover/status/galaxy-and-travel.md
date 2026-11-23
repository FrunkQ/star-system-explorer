# Status: Galaxy charts and travel screens

Owner thread: "Build galaxy charts and travel screens". Covers: U6, U8, V8, V9, voyages, GH8.

Only the owner thread edits this file. One row per package: update it in the same pull request as the package. Logs are in `handover/packages/`. Index and standing rules: `handover/START-HERE.md`. Stage order: `handover/design/SSE4-coming-together.md`.

| Package | State | Log |
|---|---|---|
| V8 Route, the astrogation console (part 1: GM Route screen on plan_route; courses compared, the chosen one charted in the star's frame) | parts 1, 2a (Window), 3, 3b and 3c (voyages between stars and inside a system: `plan_voyage`, Voyages screen; fuel types and refuelling as a ship capability) done (second UI track); route offers braking in the air beside the arrival burn; next: aerocapture as its own kernel candidate, the join at a system edge, course requests, player console, drift | packages/V8.md |
| U6 galaxy and patch charts (part 1: Galaxy under the Explore tab, from the new `galaxy_chart` op; also `web_slice` for V9) | part 1, 2a (inked history preview) and 2c (what other galaxies brought, GH3) done (second UI track); part 2b: patch survey, find a home, save inked history | packages/U6.md |
| V9 map views (part 1: the Cosmic web foam under Explore, from `web_slice`, with time scrub and Play) | part 1 and part 2 (cluster map "Galaxies nearby", op `halo_cell`; `galaxy_chart` charts catalogued haloes) done (second UI track) | packages/V9.md |
| U8 story finder (part 1: Stories under Explore, on the new `sift` op; starter sifters as data, hits with evidence, the GM's own sifter, a brief for a storyteller) | part 1 done (second UI track); Minerva and rarity next | packages/U8.md |
| V3 gap 19: stars that show what they do (star marks sized by power, flares from the kernel's activity; shells and jets later) | part 1 done 23 Nov (second UI track); part 2: shed shells from each star's life line through read_nebulae | packages/GAP19-star-marks.md |
| V8 ship at a world: aerocapture, entry and landing, runway landing, powered landing and take-off on the physics (op `ship_at_world`; dangers, margins, what goes wrong and rough chances; nothing refused or rolled) | kernel done 23 Nov (PR 228); part 2, the card under an in-system voyage with the ship drawn, done 23 Nov; next: the leg's own arrival speed, and the aerobraked arrival offered beside the burn in the route finder | packages/V8-ship-at-world.md |
| Opening Voyages under budget (plan_voyage 633 to 86 ms in wasm, PR 203) | done 22 Nov | packages/V8.md |
