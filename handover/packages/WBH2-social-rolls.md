# WBH part 2: the social codes as a rolled profile

Part 1 (`packages/WBH1-extended-profile.md`) reads the physical profile off the physics. This part rolls what the
physics cannot decide, as Alex ruled (23 Nov 22:11): a seeded social layer for the GM only, with the Cepheus Engine
SRD's numbers (Open Game Licence) and no World Builder's Handbook tables. Physical digits are never rolled.

## What changed

- **`sse-codes::roll`: a general rolled-profile engine.** A pack's `provides.other.rolls` lists programs. Each names
  a code sheet and its steps, and each step may give:
  - `dice` (d6s), `plus`, `add` (segments whose values are added), `dm` (a modifier by another segment's symbol);
  - `zero_when` (0 without a roll), `only_when` (skipped otherwise), `floors`, `min` and `max`;
  - a `table` (totals to symbols) or a `target` (a yes-or-no roll, such as a base);
  - `writes` (the fact; left out, the segment's authored field).

  Steps read the sheet's current symbols and each earlier step's result. The dice are drawn from the universe's seed
  on the program's own stream (`roll.<program>.v1`), per world and step, so they are the same on every device. A code
  already set by hand is kept unless `reroll`. Loading checks that every segment a step reads exists.
- **Kernel op `roll_profile {program, body, reroll?, apply?, at_s}`** (Light, GM machine). The preview gives each
  step's dice, total, symbol, how (`rolled`, `zero`, `kept`, `skipped`) and a why line ("2D6 rolled 4 + 3, -7,
  + population 8 = 8"). With `apply`, one authored event writes the codes, plus a `roll.<program>.<step>` fact
  beside each saying how it was rolled. The sheet then reads them as hand-set codes, which the GM overrules like any
  other.
- **The Traveller program `traveller.social`** ("Roll the social codes"), from the Cepheus Engine SRD chapter 12:
  - government 2D6-7 + population, 0 to 15, and 0 when uninhabited;
  - law 2D6-7 + government, 0 when the government is 0;
  - starport 2D6-7 + population through the SRD's table (X, E, D, C, B, A);
  - tech level 1D6 with the SRD's modifier table by starport, size, atmosphere, hydrographics, population and
    government, its four floors, and 0 when uninhabited;
  - naval base 8+ at A or B; scout base 7+ (not at E or X; -1 at C, -2 at B, -3 at A); a corsair base (the SRD's
    pirate base) 12+ where there is no A port and no naval base.

  Population is not rolled: it is the world's settlements (or a typed code).
- **World codes** shows "Roll the social codes…" for the picked world on the GM's screen: the preview lists each
  step with its why, then "Write these codes".

## Checks

`universe_pack_tests::traveller_social_codes_roll_from_the_seed`:
- the same seed gives the same dice twice;
- government is its dice - 7 + population 8, and law adds that government;
- the starport is a class letter;
- applied codes read back on the sheet as authored;
- a second roll keeps them, and `reroll` rolls again;
- an uninhabited world gets government, law and tech level 0 without a roll;
- an unknown program is refused.

Golden changed on purpose (`contract.jsonl`): one appended case, `roll_profile` refused on a stack without the
program. `traveller-codes.json` changes only by the pack hash.

## Next

- Roll a whole sector's or system's unset worlds in one ask.
- More of the WBH social layer as further steps (factions, culture, economy) once there are OGL or our own numbers.
