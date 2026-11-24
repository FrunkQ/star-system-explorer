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
- (Done 24 Nov: PR 419 added them; WAITING is empty and "23d 26m" reads.) Seventeen units the table names were not on the sheet (arcmin, min, tonne, Moon radius and mass, Earth oceans, Earth's sunlight, percent, ppm, tonnes of TNT, and the imperial set). The data sheet thread is adding them (asked 24 Nov 15:28; agreed 15:29). Until then the engine skips those rungs and `waitingUnits()` lists them; `figures.test.ts` holds the list, so a typo in the table fails and each row is struck off as it lands. The Moon's tilt typed as "23d 26m" is a skipped test until arcminutes arrive.

## Checks

- `packages/ui-contract/test/figures.test.ts`: the table is well formed (families, classes, unique readings, Imperial never first); the design's own examples at each depth; honest precision; sticky boundaries and a held rung; click order and remembered choices; typing with words, prefixes, exponents, grams, compounds and °C; another kind told apart; and every reading of sixteen kinds at three depths reads back to its value within what was shown.
- `scripts/build-ui-reference.py --check` (run by `sheet.test.ts`).

## Part 2a: the body card (done 24 Nov)

- `sayFact(fact, cls, opts)` says a scene fact through the engine: its unit symbol ("AU", "M⊕", "K", "g", "μT") is found among the sheet's units (a unit's own symbol first, so g is the gee and d the day), its field says what it is about (`fact_about` in figures-said.json: a_au is an orbit, radius_* a size, surface_k a surface), and its source sets the figures (a catalogued or typed value keeps the figures its own text gave, up to six; a computed one shows three). A fact in a unit the sheet lacks ("dex") or with none keeps the scene's text.
- `readFact(fact, text, shownUnit)` reads typing back in the fact's own unit: any unit of its kind, words and prefixes ("412 thousand km" for an orbit held in AU); a bare number is in the unit on screen. The card's `parseFact` uses it (decimal commas still read).
- A world's surface temperature reads in °C first (`*.surface`, like a site). Science depth too: Kelvin is a press away.
- Pressing a unit steps to the next reading for every body of that scale class (CardField `unit` and `onunit`; `ws.setUnitChoice`). The choices are in `ws.unit_choices` and the campaign save (`workspace.unit_choices`). The handles row and the strip say figures in the same reading.
- `subjectClass(subject)` (handles.ts) gives a scene subject's scale class.
- Tests that read figures as text moved with it: shell.e2e (Tarn's surface in °C, Hasp's orbit "412 thousand km"), walk.e2e (Mars warms past about 230 K, in K or °C); editing.e2e presses the card's orbit unit round its readings and checks the handle follows.

## Next (part 2b)

The other screens' own number formatting onto `say` (about 250 places use toFixed, toPrecision or toLocaleString), with a ledger check that counts them per file and fails if one grows, going down to zero as the typed-copy ledger did; "How figures are said" on Reference beside Units; player seats inheriting the GM's choices. Planet radii as a rung for a moon's orbit needs the parent's radius passed in; local days and years need the world's calendar.

## Integration points

- Data sheet thread: the seventeen Units rows above.
- Chart room thread: part 2 touches the body card and other screens; coordinate with the App.svelte split.
