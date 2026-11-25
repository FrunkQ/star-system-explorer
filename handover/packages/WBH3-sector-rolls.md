# WBH part 3: a whole sector rolled in one ask

Part 2 (`packages/WBH2-social-rolls.md`) rolls one world's social codes. This part rolls every mapped world at once,
and fixes a fault the browser check found in part 2: an imported world's codes were rolled over.

## What changed

- **The fault.** An imported world's government, law and tech level are read through the sheet's rules (the
  import writes the class's typical facts), not as hand-set codes, so `roll_profile` rolled them again and wrote
  new hand-set codes over the list's. Its bases were rolled too, since a list with no base writes no flag.
- **`decided_by` on a rolled program** (sse-codes `roll::Program`, optional). It names a fact whose presence means
  the world came with its whole profile. When the world has it, every step is kept ("kept: the world came with its
  profile (provenance.code C566600-7)"), bases included, unless `reroll`. `traveller.social` names
  `provenance.code`, which every imported world carries. Hand-set codes are still kept as before.
- **Sector desk: "Roll every world…"** (GM only, shown while the stack gives a rolled program). The popup says what
  it does, offers a picker when there is more than one program, and "Roll and write for N worlds" runs
  `roll_profile` with `apply` for each mapped world's body. The tally reads "4 worlds rolled: 0 codes written; 28
  codes already set kept." One world's refusal is counted and never stops the others. The logic is
  `rollEveryWorld` and `rollTallyWords` in `packages/views/sector/src/desk.ts`.

## Checks

- `universe_pack_tests::traveller_social_codes_roll_from_the_seed` gains a listed world: every step is kept and
  nothing is written; with `reroll`, its government rolls.
- `tests/ui/sector.test.ts`: the tally over a fresh world, an imported one and a refusal.
- `tests/ui/sector.e2e.mjs`: after importing four worlds, "Roll every world" writes nothing and keeps 28 codes.
- Golden: `tests/golden/traveller-codes.json` (the pack hash, for `decided_by`); no contract lines move.

## Next

- The sector map after a roll of grown (unimported) worlds: a browser check once a walk grows a sector of systems.
- More social lines once there are OGL or our own numbers.
