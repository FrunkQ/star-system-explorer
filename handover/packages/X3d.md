# X3d part 1: gas mixes and what to wear outside

Status: done (19 Nov 2026), Traveller pack track. Design: handoff 7A part 5. Still to come in X3d: sensors and the knowledge ladder, stock ships (J7 templates), the jump-era fictional physics pack (programs need J6).

## What was built
- `crates/sse-codes/src/mix.rs`: `GasMix` pack data (`provides.other.gas_mixes`), with a composition as fraction ranges per gas, an optional named taint, the code symbols it can stand for, a weight, the source and a note. `apply` writes `atmosphere.pp.<gas>` (bar) at the world's surface pressure, plus `atmosphere.main`, `atmosphere.mix` and `atmosphere.taint`. `pick` chooses by weight from a unit draw.
- `Codes` loads mixes from every pack. Duplicate ids are refused.
- `plan` (X3b) picks a fitting mix for the coded air with the seeded stream `codes.gas_mix.v1`, so the same seed always gives the same air. `Plan.mix` names it.
- The Traveller pack has 23 mixes ported from V3's starter-sf `atmospheres.json` (Traveller-0 to F and the exotic presets). Three were moved or changed because they would read back as another digit, and each says why in its `note`:
  - V3's 2a oxygen was raised to 10% so the air is not exotic.
  - V3's D (no oxygen) became an exotic A mix, and a new very dense nitrogen-oxygen mix stands for D.
  - V3's "F (thin, low)" is kept as unusual F, because low air is E.
- The corrosive flag now also reads chlorine, fluorine and sulphuric acid partial pressures.
- A new rule group `needs` (`traveller.need.*`) gives what a traveller needs outside, from the atmosphere digit: vacuum suit, respirator, filter mask, air supply, protective suit, sealed suit, high ground or low ground. Its words are in the vocabulary.

## Checks
- `every_gas_mix_reads_back_as_its_own_digit`: every mix, for every digit it stands for, at that digit's typical pressure, reads back to the same UWP.
- An Earth-like code always gets the Earth mix. Exotic air draws several mixes across seeds. Corrosive air needs a protective suit.
- Golden `traveller-codes.json`: only the pack hash moved.

## Limits
- Composition only. The pressure comes from the code's class, not the preset's own range (`docs/fudges.md`).
- Needs come from the digit, not from per-gas physiological limits (`docs/fudges.md`).
- No atmosphere flow consumes mixes yet beyond the X3b plan. A general atmosphere flow could take a mix as its starting composition.

# X3d part 2: sensors and the knowledge ladder (19 Nov 2026)
- All pack data, with no new code. It runs on the general triggers (J4), grants (K10) and data views:
  - `provides.other.sensors`: the Traveller range bands (adjacent to far), 12 sensor kinds ported from V3's `sensors.json` (range, targets, what it reveals, the highest rung it can reach), and the ladder.
  - `provides.other.observe`: the knowledge ladder long-range, scanned, surveyed and detailed survey, with field disclosure tiers. It defaults to long-range.
  - `provides.other.triggers`: one rule per body sensor (`traveller.sense.<kind>`). When a ship (`tag.ship`) whose `sensor.<kind>.range_m` reaches a body holds for an hour, the body moves one rung up the ladder for the `players` audience, up to that sensor's ceiling. Construct sensors are data only for now, because ships as subjects come with stock ships.
- Check: `traveller_sensors_move_a_world_up_the_ladder` (kernel). A ship with a spectral analyser at 10 million km and one hour of `rules_step` lift a world from long-range to scanned, and the players then see its air.
- Limits:
  - The audience is the fixed `players`. The kernel thread has been asked about `{observer.<field>}` audiences, so each crew could learn for itself.
  - Sensor targets (stars, belts, moons) are listed but not yet enforced in the condition.
  - The Traveller pack now provides the stack's observe setup, and the top pack with one wins, so a pack above it replaces the ladder.

## After kernel PR 28 (20 Nov)

- Sensor grants now go to the observing ship's own audience,
  `{observer.crew.audience}` (J4), instead of a fixed "players". A ship needs
  `crew.audience` (for example "players") as well as its
  `sensor.<kind>.range_m`; a ship without it gets the grant refused, nothing
  half-done. The kernel test sets it on the test ship.
- The Imperial calendar now spells out `week.continuous: false`, which the
  kernel's calendar check requires for a day outside the weeks (Holiday).
- `tests/golden/traveller-codes.json`: pack hash only, for these two edits.
