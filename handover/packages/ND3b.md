# ND3b part 1: physics blocks move from Rust into formula files

Design: `handover/design/SSE4-equation-block-design.md`, sections 8A and 8B (accepted by Alex, 22 Nov).
Sweep order (Alex, 23 Nov, coming-together Q7): typed copies first, then Equations crate by crate.
Owner: Build kernel resolve and rules.

## What landed

- **New crate `sse-formula`**: the Equation's parser and evaluator, moved out of `sse-logic` so the
  physics blocks in `sse-atoms` can use it (sse-logic depends on sse-place, which depends on sse-atoms,
  so the formula engine had to sit below both). `sse-logic::equation` re-exports it unchanged and keeps
  the Equation block and `expand`. One shared function, `sse_formula::number`, still holds the pinned maths
  of every number block, so blocks, Equations and formula atoms give the same bits and refusals.
- **`FormulaAtom`** (`crates/sse-formula/src/atom.rs`): a physics block made from one formula entry. It keeps
  the block's id, version, cost, ports, references and Why text; letters wire to ports or to the constants
  in force (`constants/G`, `constants/c`, `constants/sigma_SB`, `constants/k_B`), so a changed G reaches
  it as before. `positive` lists the refusals, checked in the order the Rust code checked them, with the
  same words. The formula is parsed once, when the pack loads. A worked example never types a constant: it reads the sheet's value in force (the matter audit would flag a typed G), and its inputs are round numbers for a world, a star and an orbit like ours.
- **Formula files** `packs/base-physics/formulas/{planet,orbit,star}.json` (format `sse.formulas/1`), read by
  `sse-atoms::formulas` (compiled in with `include_str!`, parsed once). Six blocks moved:
  bulk density, surface gravity, escape velocity, total mass, orbital period, star surface temperature.
  Each formula is written in the Rust operation order (`R * R * R`, `2 * pi`, `^ (1/4)`), so the bits match.
- **Shrinking list** `handover/packages/ND3b-rust-atoms.txt`: every block still written in Rust, with why.
  A test checks it against the registry both ways.

## What stays in Rust for now, and why

- A cube root (`hill_radius`, `semi_major_axis_from_period`): `x^(1/3)` is not the same bits as `cbrt`;
  the formula language needs a `cbrt` function first.
- Parameters (`roche_limit`, `ms_lifetime`, `ms_luminosity_at_age`): wait for parameters as letters.
- Own refusal words (`apsides`), iteration (`kepler_position`), piecewise fits over mass ranges
  (`ms_luminosity`, `ms_radius`, `radius_from_mass`), fitted numbers (`rocky_radius`,
  `core_mass_fraction`, which become Model number letters first).
- The rest are marked "not yet looked at".

## Tests

- `formula_blocks_give_the_old_bits_and_refusals`: each moved block against its old Rust maths (kept in the
  test) over 284 values from 1e-30 to 8e40, bit for bit, with the same refusal words. Where the old maths
  overflowed to infinity the formula refuses "too large to hold" (only far outside any physical value).
- `formula_blocks_follow_a_changed_constant`: doubling G doubles surface gravity.
- `the_rust_blocks_list_matches_the_registry`, `formula_blocks_have_worded_names_and_keep_their_faces`.
- Golden hashes unchanged: every golden test passes untouched, so no world changed.

## Next (part 2 onward)

`cbrt` in the formula language, parameters as letters, then the next families by the ledger.
