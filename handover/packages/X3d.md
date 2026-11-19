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
