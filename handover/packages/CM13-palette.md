# CM13 part 1: the parts palette (compound ships)

Track CM. Design: `handover/design/SSE4-compound-ships-design.md` sections 8A.4, 8A.5, 8B.2, 8B.3 and 13, rulings 14A.3 to 14A.6 (accepted by Alex 27 Nov 2026 00:43 UTC; 14A wins). Built 27 Nov 2026 by the thread "Build the snap-together ship builder". Pack data and a TypeScript format; no kernel change, no golden moves.

## What it is

- **The one part format**, `packages/ui-contract/src/parts.ts`: figures (value, unit, source, mark: cited, authored, predicted, derived or estimated), connection points (place, size on the standard, lines, where and which way it faces, any direction), a shape (lathe, panel, truss, dish, box), a scaling law (volume, area or fixed), the year it first flew, maker, fuels, and shape rules a drive brings (keep away, shadow cone, pusher plate, radiator area, clear view; ruling 14A.5). Whatever a store pack or the part editor brings (CM17, CM18), Atomic Rockets drives included, uses this format, so it clips in with no change.
- `clips`: two points join when their kinds of joint meet (front with behind, ports and hull stations with each other, a bay's mouth with either end) and their sizes match; it returns the lines both pass. Different sizes say an adapter joins them, as real stacks do (a 10 m S-II under a 6.6 m S-IVB). Points are matched by place and size, never by pack (8B.3).
- `stretchPart`: stretch and widen (ruling 14A.4). Figures the part lists with its volume or its area follow that measure and are marked `derived`, with the law in words; points move with the ends and keep their sizes; a fixed part (real hardware) is left as flown and says it comes in the sizes its pack offers.
- `partWarnings`: a part with no figures is a decoration and says so; a stated volume its shape cannot hold; a port or hull point off the size standard.
- `flownBy`: the palette as it stood in a year (the reading the era control, CM15, uses).
- **The palette**, `packs/base-parts/pack.json`, built by `scripts/build-base-parts.py` (`--check` in `scripts/check.sh`): 61 parts.
  - 13 real stages and 17 real engines from the Sol today pack's `parts.json` (GCAT, CC BY 4.0, credited), each with the year it first flew from the pack's own launch list.
  - A radioisotope generator, a solar array wing, three high-gain dishes and the Shuttle's payload bay from the Sol today pack's `craft.json`, with each craft's launch year.
  - Generic primitives with no year: propellant tanks, habitat cans and cargo pods at 2, 4 and 10 m, a four-port docking node, a truss and a boom, solar and radiator panels, a bay and a dish. Their figures are geometry only (volume, area); what the kernel reads from them (berths, watts, heat shed) is worked out, never typed, and each says so under `to_read`.
  - Nine drives and power plants from the future tech pack's advances (hydrogen-oxygen rocket, nuclear thermal rocket, light sail, fusion torch, Hall thruster, arcjet, gridded ion thruster, fission reactor, radioisotope generator). They copy no figure: each names its advance and home level, and the kernel reads the advance's figures at the setting's level (ruling 14A.5: each era ships its own drives). Their size is a stand-in until a sizing block draws it from their output.
  - The port size standard: 0.8 m crew passage and 1.27 m berthing passage (both from memory, unverified), then 2, 4 and 10 m (authored).
- **Names are physics names** (14A.5): the builder refuses a name with the Atomic Rockets brand or a named screen ship before it writes the palette, and the test checks the palette.

## What it does not do yet (part 2 and later)

- The tank split blocks (wall mass from the thin-wall pressure vessel, insulation and boil-off from the surface, N^(1/3)), a tank's wall mass and the dish's link budget are kernel Equation blocks; adding them to `packs/base-physics/formulas` needs a check on whether the core golden moves, so they are part 2.
- Sensors with their band and aperture (the instrument shape, Atomic Rockets plan section 12) and their clear field of view (14A.6); the trend fits between ladder levels, so drives shrink era by era (CM15; eras, not decades, ruling 14A.8), and sizes for the advance-linked drives from their output.
- No screen reads the palette yet (the builder, CM14), so the pack is not copied into the shell's public packs.
- Real engines have no mass (GCAT gives none; listed under `to_read`) and their bells are drawn to fit their stage (fudge register).

## Checks

- `packages/ui-contract/test/parts.test.ts` (7 tests, in `npm run ui:test`): the format and sources of every part, no warnings; real years and stage figures adding up; names; clipping; stretch, widen and a fixed part's note; the era reading. Seen red with stretch not scaling, the size check off, and the era filter off; green restored.
- `python3 scripts/build-base-parts.py --check`; `scripts/audit-matter.py --closed` passes (an RTG's decay half-life copy was taken out: it stays in the craft record).
