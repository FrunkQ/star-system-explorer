# ND3b: physics blocks move from Rust into formula files

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

## Part 2 (23 Nov): the families

71 blocks now live in 20 formula files under `packs/base-physics/formulas/`; 14 stay in Rust, each with
its reason in `ND3b-rust-atoms.txt` (series, bisection, piecewise fits over mass tables, an answer of
infinity, a rounded gas constant that MP22 swaps first).

- **The formula language** gains `cbrt` (`math.cbrt`, also a maths block in sse-logic, unit rule root 3),
  so `x^(1/3)` written as `cbrt(x)` keeps the Rust bits.
- **Letters for settings, sheet rows and model numbers.** `params` become the block's settings (default a
  number or a sheet row, `"sheet/T0C"`), `constants` may read any sheet row unedited (`sheet/<id>`), and
  `models` carry fitted or measured numbers with their grade and source, like the Model number block.
- **Own refusal words.** `positive` and `checks` (`min`, `max`, `over`, `below`, `above` another port,
  `finite`) refuse with the old block's words, so a moved block refuses exactly where it did.
- **Valid ranges** may give an end as a multiple of a sheet row (`"8 sheet/M_earth"`), so no figure of
  nature is typed in a pack.
- **`sse_atoms::decl_of` and `decls_of`** give a block's face by id; callers that took the faces from the
  old Rust modules (activity indices, convection references, the light air list, the field block list)
  read them from the registry.
- **The old Rust** lives under `crates/sse-atoms/src/test/` as the `was` module, test only.
  `moved_blocks_keep_their_faces_and_bits` checks every moved block against it: the same face, and the same
  bits and refusal words over a grid of values with the settings at their defaults and range ends.
- **Tools** (not in the repo, rebuildable from this log): a script read each Rust `Def`, wrote the formula
  entry and worked examples, and moved the `Def` under `src/test/`. Worked examples use round numbers.

- **The gas constant.** `sse_atoms::chem::R_GAS` was a rounded copy (8.314462618); it now is the sheet
  row `R_gas` (N_A k_B, exact), and `sse.chem.reaction_gibbs` moved with it. Its ledger line is gone (423
  left). Values move by about 1e-11 relative; no golden moved.

### Golden

The contract golden changes by one line, on purpose: the reach report lists the new `math.cbrt` block.
No world changed: every other golden passes untouched.

## Next

The 15 blocks in the list, as the language grows: series and bisection (a fixed-step "repeat" form),
fit tables as data (mass-range pieces, CCM89) and an unbounded answer.

## Timing (8B, within 5 %)

PF2 bench, Node runner, cloud container, main a7ad88b against this branch, one machine. Full run, 5 runs
each: every chunk within 5 % except `resolve` at 4.0 to 4.2 ms (+5.0 %, one step of the 0.1 ms
display). Resolve alone, 9 runs, the two builds alternated three times: main 4.7, 4.6, 4.9 ms; this
branch 4.8, 4.6, 4.8 ms. Same within noise.
