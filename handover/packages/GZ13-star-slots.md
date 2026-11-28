# GZ13: a star slot for every system (26 Nov 2026)

Design: `design/SSE4-grand-zoom-design.md` 6.3 point 2, 6.5; GZ0's finding (no campaign system had a site event history). Status: `status/grand-zoom.md`. Built on GZ12.

## What it does
- **At adoption.** Every system that `adopt` takes in (a V3 save or a formation run), except one grown from a drawn star, is given a **star slot**: a drawn star of the home galaxy it stands for. The slot is written into the adoption's own event, so an adoption is still one event (two for a V3 starmap, as before).
  - **By map position** (`by: "map"`): a V3 starmap position is turned into parsecs (map pixels over `scale.pixelsPerUnit`, default 25 as in V3, in `distanceUnit` ly or pc) and laid out from the Sun's place on the galaxy grid's axes. The best match in the nearest shell of 10 pc cells holding a free star is taken. A diagrammatic map has no distances, so it is placed by fit. The importer now keeps the starmap's `scale` and `mapMode` (they were dropped as presentation).
  - **By fit** (`by: "fit"`): the nearest drawn star to the Sun's place whose mass is within 10 %, age within 10 % (at least 0.5 Gyr) and companions the same as the system's own star, searching up to 343 cells. With none, the closest match is taken and marked `fits: false`. It is never refused.
  - The tolerances are pack data, `cosmology.slot` (`sse-cosmos/src/slot.rs`).
- **The pin.** The drawn star is pinned with the system's authored star laid over it: mass, age, [Fe/H] and companions where authored. Its place, orbit and other abundances stay the place's. It claims its share of the cell by the pin rule, so nothing is counted twice, and `star_cell`, the sky and the event layer read the same star. The drawn star is kept beside it (`star.drawn`), and so is what was laid over (`star.authored_over`).
- **`star_slot {system}`**: the slot, where it is now (cell, xyz, radius, azimuth), what is authored, the drawn star, and **advice**. "Unusual for its birthplace" is the reconstructed-origin check at the slot's place with the star's [Fe/H] kept. It is advice, never a refusal.
- **`set_star_slot {system, star?}`**, "Move it": another drawn star (refused if it is already pinned), or with no star, look again. The old slot's star goes back to the fill. A system grown from its star cannot be moved, because its address is the star's.
- **`site_history {system, from_myr?, to_myr?}`**: G6's event layer on the slot star's own orbit, by default over its whole life. It covers supernovae, kilonovae and gamma-ray bursts from every cell in reach, the galaxy's nucleus episodes, mergers and starbursts, close passes and arm crossings. For the synthetic Sun over 4.57 Gyr: 510 core collapses within reach, 33 Ia, 5 bursts, 805 close passes, 22 arm entries, 2 nucleus episodes.
- **`set_system_age`** reconstructs the origin from the slot's radius and azimuth now. The `reconstruct_origin` place stand-in is retired for adopted systems.
- Session methods: `starSlot`, `setStarSlot` and `siteHistory` (`ui-contract/session.ts`, types in `galaxy.ts`), for Your systems' "Find it a place" and moving, and for the life track.

## Fudge
F37c: a fit by three tolerances, not a likelihood over the cell's population, and a V3 map laid out on the galaxy grid's axes (V3 maps have no galactic frame).

## Not done here (follow-ups)
- **GZ13b**: Traveller sector imports (`import_sector` with `grow`) do not get slots yet. A hex is a parsec, so they would be placed by map position the same way.
- "Grow a system here" on a drawn star is `from_star` (done 25 Nov). The button is on the galaxy charts thread's branch.
- The screen half (Your systems' place list, Move it) is the galaxy charts thread's.

## Tests and goldens
- New: `sse-kernel` `slot_tests` (6): adoption gives a home star counted once; a map keeps its layout (300 ly is about 92 pc); fit within tolerances and never the same star twice; the GM's move and a refused taken star; a site history that repeats; and the standing rule that no adopted system is without a place.
- Golden area `grand-zoom` gains the GZ13 cases. Core golden `contract.jsonl` and area `small-bodies` move on purpose: every adoption now carries its slot (`places` in the answer, the slot facts in the event).
- Reach gate (ER1b): `adopt` now follows the physics (its slot is read from the drawn star field), so its `still adopt` line is off `handover/packages/ER-reach-ledger.txt`. `star_slot`, `set_star_slot` and `site_history` have area cases and follow.

## Merging main after TP4 (27 Nov)
TP4's check asks every op that takes a date to be named by a process in `packs/base-time`. `site_history` takes a window (`from_myr`, `to_myr`), so it gets its own declaration, `galaxy.events`: G6's event layer on the star's path, paced by its own marks. The live ladder test now finds both rungs paced by marks (the campaign's record and the galaxy's events) among the last, rather than asking that the campaign's record be the only one. Goldens that print the base-time stack hash move with it.

Found by the browser walk (`your-systems.e2e.mjs`): the campaign's systems list (`systems`) counted each pinned slot star as a system of its own, because the star sits at a `sys:` address of the home galaxy. `adopt::systems_of` now leaves out a `sys` whose `star.slot_of` names another system; the slot test asserts it (red without the fix). The core golden's `systems_listed` line is back to main's.
