# CM14a: the ship builder, first slice (compound ships)

Track CM. Design: `handover/design/SSE4-compound-ships-design.md` rulings 14A.7 (two modes, catalogue first, old designer gone) and 14A.8 (eras, not decades). Asked for by Alex 27 Nov 2026 11:47 to 11:52 UTC; slice agreed by the coordinator (convert-on-open only, no assembler). Built 27 Nov 2026 by the thread "Build the snap-together ship builder". TypeScript and pack data; no kernel change, no golden moves.

## What it does

- **One ship designer, two modes.** "Build it" shows what the ship can do as each part goes on: mass full and empty, delta-v, push, tank room by fuel, berths, cargo, radiator and solar area, and a named list of what the parts cannot count. "Build to a plan" takes the catalogue ship's figures as the plan and shows budget bars (tanks, habitat, cargo, radiator, push) counting down; an overshoot shows past zero, never clipped.
- **Catalogue first.** The Workshop lists "Ship builder" (Explore and Build desks). It opens on the first catalogue ship; any catalogue ship, or a blank build, can be picked. On a ship's Picture page in edit mode the builder replaces the old Style and Remix controls; "Keep" saves the build with the ship (`sse.build/1` in the design field), and a kept build reopens in the builder.
- **Convert on open.** `fromLayout` (`packages/renderer/src/ship-build.ts`) turns a catalogue ship as the old generator drew it into palette parts: generic tanks, habitat and cargo, and the tech part for its fuel as the drive. Until each part's own figures are read, the drive's thrust and exhaust and the ship's structure mass are carried over from the catalogue record, and the builder says so.
- **Clipping.** Parts clip to free points: the open ends of the stack and hull stations at a clock angle (0 up, 90 starboard). A picked part stays picked; Longer, Shorter, Wider, Narrower, Turn and Remove act on it. Stretching a tank grows its room and load by the part's own law.
- **Eras.** The palette groups parts by era, not decade: seven eras of varying length, three of them future (pack data in `packs/base-parts/pack.json`, from `scripts/build-base-parts.py`). The year control offers parts by exact first-flight year; tech parts by the era they first flew in, or for unflown advances the first future era at their level. Era list proposed to Alex, not yet confirmed.
- **Plans and model.** The plans draw each part as its own path; the 3D model sits over a fixed blueprint background that does not move with the model (Alex's ruling on the mock-up).
- **What can it do?** asks the kernel (`ship_at_world`) for verdicts at Earth, Mars and Venus (the sol fixture has planets only).
- **Old designer gone from the GM path.** Style and Remix are removed from the Picture page. The generator code stays in the renderer only to convert catalogue ships on open, until every catalogue ship is saved as parts.

## What it does not do yet

- No compound ships: one construct at a time. Mounts, staging and travel need CM3 (manifest), CM5 (kernel staging), CM6 (travel) and CM8 (compound picture); full CM14 follows CM8.
- No mouse-wheel stretch or drag to clip yet: buttons and a clip-target list stand in.
- Totals are summed in the renderer from part figures as a stand-in; the kernel does not yet read a build. Real engines have no mass (GCAT gives none), so a build from real hardware names its missing mass.
- No sloping or any-angle superstructure and skins yet; no part editor (CM17), no store parts packs (CM18).
- Catalogue ships are converted each time they open; saving them all as parts (and deleting the generator) is a later step.
- Eras await Alex's confirmation.

## Checks

- `packages/renderer/test/ship-build.test.ts` (9 tests): clipping on the axis and hull; plans and 3D draw a build; totals summed with what is missing named; stretch law; budgets count down and overshoot; free points; catalogue ship converts and redraws within 35% of its length; eras and exact-year offering; carried-over drive and structure said so. Seen red with clock sin and cos swapped, tank volume not added, structure carry-over off, and the era filter off; green restored.
- `tests/ui/ship-builder.e2e.mjs` (desk and phone; in CI): catalogue ship opens as parts, a longer tank raises delta-v, a radiator clips on, plan budgets show, 3D model, kernel verdicts, blank build, no sideways scroll on a phone, nothing written.
- `tests/ui/manifest.e2e.mjs`: the Remix steps replaced by builder Longer then Keep. `tests/ui/desk-rail.test.ts`: "Ship builder" off the Run desk.
- `npm run ui:test`, svelte-check, `scripts/build-base-parts.py --check`, `scripts/audit-matter.py --closed` and the side-door ledger pass.

## Pictures

Project files `sse4/handover-screens/CM14a-screens/`: catalogue, plan, model and can-do, each on desk and phone.
