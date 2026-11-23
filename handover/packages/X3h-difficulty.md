# X3h: a game system's difficulty numbers beside SSE4's words

Alex, 23 Nov: where SSE4 says how hard something is, a game pack that defines its own task ladder shows its
system's name and target beside SSE4's words, GM-only. SSE4 rolls nothing and decides nothing (see the memory
"feedback-gm-judgement-not-rolls", updated). Alex's answers: odds through the system's dice first, the word table as
a fallback; show the skill when the game pack knows the game's skills; GM screens only.

## Pack data (any game pack): `provides.other.difficulty`

`{system, dice: {count, sides}, base_target?, typical_dm, steps: [{name, target | dm}], words?, skills?, source?}`.
A step gives a target to meet, or a modifier on `base_target` (Cepheus style). `typical_dm` is a typical trained
operator's total modifier, used only to turn odds into a step. `words` maps SSE4's difficulty words (routine, easy,
moderate, hard, very hard) to step names; `skills` maps SSE4's roles (pilot, navigator, engineer) to the game's
skills. A d20 system declares `{count: 1, sides: 20}` with targets (5+, 15+, 30+).

## ui-contract `difficulty.ts`

- `difficultyLadderOf(packs)`: the topmost pack's ladder, or null (then nothing is shown).
- `chanceToMeet(dice, target, dm)`: the exact chance from the dice distribution.
- `gameDifficulty(ladder, {p_wrong?, word?, role?})`: with a hazard's chance of going wrong (ship-at-world
  `Danger.chance.p`), the step whose success chance for a typical operator is nearest 1 - p; else the word's step.
  It returns `{system, step, target, dm?, skill?, basis, p_success, words}`, e.g. "Piloting, Average 8+".

## Traveller

From the Cepheus Engine System Reference Document (Open Game Licence, read 23 Nov at orffenspace.com): 2D6 plus
modifiers against 8+, difficulty as a modifier (Simple +6, Easy +4, Routine +2, Average 0, Difficult -2, Very
Difficult -4, Formidable -6), and the skills Piloting, Navigation and Engineering. Typical operator: skill 1, no
characteristic modifier (a game default for this pack). SSE4's words map by target: "routine" is the game's Easy.

## Checks

`tests/ui/difficulty.test.ts`: exact 2D6 and d20 odds; the Traveller ladder's targets 2+ to 14+; a 50% hazard for a
pilot reads "Piloting, Average 8+"; certain failure is Formidable and a safe act Simple; words map, an unknown role
shows no skill; no ladder, nothing shown. Golden: pack hash only.

## Next

The "At world" card and the route console show it beside SSE4's words (travel thread, asked).
