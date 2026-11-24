# ED1a: One units engine (part 1)

Status: part 1 done 24 Nov 2026 (thread "Build planet and star editing"). Design: `handover/design/SSE4-editing-design.md` section 3C, accepted by Alex 24 Nov 2026 (questions 34 to 37).

## What it does

- **`say(si, kind, about, opts)`** (`packages/ui-contract/src/figures.ts`) turns a value in SI into the words a screen shows: `say(384.4e6, "length", { class: "moon", what: "orbit" })` is "384 thousand km"; a planet's orbit reads in AU, a world's size in km, a giant's mass in Jupiter masses, a star's in Sun masses, air in bar and millibar (Pluto's in microbar), the inside of a world in GPa, a site's temperature in °C and everything else's in K, time spans climb hours, days, years, Myr and Gyr. It returns the number and the unit apart (`number`, `unit`) so a screen can make the unit pressable, the reading and rung used, and whether it is approximate.
- **Readings and rungs.** Each kind and "about" has readings a click steps through (`readingsFor`, `nextReading`): the Scientific family first, reference bodies next (a giant's mass: Jupiter masses, then Earth masses), then SI, then Imperial last and never a default (a test fails any key that puts Imperial first). Inside a reading the engine picks the rung by size (km with "thousand" and "million", then AU from 0.01 AU, then light years from 0.2 ly), with 10% stickiness at each boundary (`prev`) and a held rung for the length of a drag (`hold`).
- **Depth.** Curious writes words ("about 0.82 Earth masses", "billion years") and "about" on computed figures; Science uses symbols and the SI prefixes listed for each unit (mbar, µbar, MW, GPa, kpc), never megametres; Engine writes exponents where Science writes words ("3.84 × 10⁵ km"). An exponent is always on the plain unit ("3.83 × 10²⁶ W", never "× 10¹¹ PW").
- **Honest precision.** Computed figures show three significant figures; catalogued and typed ones the figures their source gave (`source`, `sig`, `significantFigures`), trailing zeros kept: "149.6 million km" typed says "1.000 AU", not "1 AU" and not "1.00003 AU".
- **`readFigure(text, kind, { unit })`** reads anything `say` writes, plus any unit of the kind by id, symbol, name or alias, SI prefixes by symbol or name (any prefix from quecto to quetta), grams, "thousand", "million", "billion", exponents written "e19", "× 10^19" or "× 10¹⁹", "about", a leading minus, and compounds that add up ("1 day 2 h", "23d 26m"). A bare number takes the unit on screen. A figure of another kind is refused with `as` set (`"300 d"` typed for an orbit comes back as a time), so the Orbit handle can turn a period into a distance with the physics.
- **Remembered per class.** `withChoice` and `chosenReading` keep the reading per kind and per scale class and figure (`"length|planet.orbit"`), so AU for planets' orbits and thousand km for moons' orbits both stick, and a planet's size is its own choice. `null` is "Use the scheme's". Saving it with the campaign is part 2.

## Where the numbers come from

- `packages/ui-contract/src/figures-said.json` is the "How figures are said" table: kinds, readings, rungs, where each rung takes over, the words, prefixes, which units take prefixes, the twelve scale classes, and how each unit is written in words. It holds **no factor**: every unit is a row of the master data sheet's Units table. It moves into a pack with unit schemes (MP15) so that a setting can edit it on Reference.
- `scripts/build-ui-reference.py` now also writes `unit_rows` (symbol, name, what it measures, factor, offset) into the interface's generated copy of the sheet, because °C needs its offset and the parser needs names and symbols.
- Seventeen units the table names are not on the sheet yet (arcmin, min, tonne, Moon radius and mass, Earth oceans, Earth's sunlight, percent, ppm, tonnes of TNT, and the imperial set). The data sheet thread is adding them (asked 24 Nov 15:28; agreed 15:29). Until then the engine skips those rungs and `waitingUnits()` lists them; `figures.test.ts` holds the list, so a typo in the table fails and each row is struck off as it lands. The Moon's tilt typed as "23d 26m" is a skipped test until arcminutes arrive.

## Checks

- `packages/ui-contract/test/figures.test.ts`: the table is well formed (families, classes, unique readings, Imperial never first); the design's own examples at each depth; honest precision; sticky boundaries and a held rung; click order and remembered choices; typing with words, prefixes, exponents, grams, compounds and °C; another kind told apart; and every reading of sixteen kinds at three depths reads back to its value within what was shown.
- `scripts/build-ui-reference.py --check` (run by `sheet.test.ts`).

## Next (part 2)

Move every existing figure onto `say` (about 250 places format their own numbers today), a pressable unit on screen that steps the reading, the choices saved with the campaign and inherited by player seats, "How figures are said" on the Reference screen beside Units, and the editing-grammar check that fails a screen formatting its own numbers. Planet radii as a rung for a moon's orbit needs the parent's radius passed in; local days and years need the world's calendar.

## Integration points

- Data sheet thread: the seventeen Units rows above.
- Chart room thread: part 2 touches the body card and other screens; coordinate with the App.svelte split.
