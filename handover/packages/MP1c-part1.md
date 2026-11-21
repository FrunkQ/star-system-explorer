# MP1c, part 1: constants as data, one handle, and the reach report

Matter pack design section 7A and section 10, package MP1c. Part 1 was built 21 Nov 2026 by the kernel thread, to the design's recommendations. Code:
- `crates/sse-core/src/constants.rs`: the handle;
- `crates/sse-reference` (`expr.rs`, plus `constant`, `handle` and the checks in `lib.rs`);
- `crates/sse-atoms/src/reach.rs`: the declarations;
- the kernel's `read_reach`, and the scope set in `Kernel::handle`.

Tests are in `sse-reference/src/tests.rs` and `sse-kernel/src/matter_tests.rs`. The fixture is `tests/fixtures/packs/reference-strong-gravity.json`.

## What it delivers
- **Derived constants worked out, never typed.** A constants row may carry `derived_from`, a formula over constant ids, numbers and `pi`: products and juxtaposition, quotients, powers, sums and brackets, as in `2 pi^5 k_B^4 / (15 h^3 c^2)`. `Reference::constant(id)` works it out from the rows in force, so an edit to h moves sigma_SB.
- **Load checks on derived constants.**
  - A formula that names an unknown constant is refused.
  - So is a formula that goes round in a circle.
  - A typed copy is checked against its formula to 1e-9. The check runs while the row and every input come from one pack. After an edit, the worked-out value wins.
- **`checked_by`.** The same check, but the formula is never used to work the value out. This is for a mass typed beside its GM (`M_sun` checked by `GM_sun / G`), so that doubling G keeps the Sun's mass. It was proposed to the citations thread for PR 93's three mass rows.
- **One handle.** `sse_core::constants`, with fields G, c, sigma_SB and k_B. It is per thread and scoped with `with`, and always restored, even on a panic.
  - Unedited, it holds the compiled values bit for bit.
  - The kernel sets the stack's values for every call (`Reference::handle`). While nothing a derived constant rests on is edited, the handle takes the sheet's typed copy, so an unedited sheet changes nothing.
- **Blocks that follow.** These blocks now read the handle in place of the compiled constants:
  - `sse.orbit.period` and `semi_major_axis_from_period`;
  - `sse.planet.surface_gravity` and `escape_velocity`;
  - both tidal blocks;
  - `sse.climate.equilibrium_temperature`;
  - `sse.atmosphere.scale_height` (and `sse_atoms::air::scale_height`) and `jeans_parameter`;
  - `sse.star.effective_temperature`.

  `climate.rs`, `orbit.rs` and `tides.rs` no longer import the compiled constants at all.
- **Reach declarations.** `sse_atoms::reach::DECLARED` lists each block that follows, and which constants it reads. It holds the converted blocks above plus the pure formulas that read no constant (Hill radius, Roche limit, apsides and the rest). Every other block "cannot follow", the honest default, so a block moves up only with a test.
- **Units beside Constants** (design 7A.6, added on Alex's word on 21 Nov). The table is `provides.reference.units`, with rows `{id, symbol, name, measures, group, factor | factor_from, offset?}`.
  - `measures` names a dimension, or `dim` gives the seven SI exponents.
  - `factor_from` is a formula over constants (`c yr_julian` for the light year), so a unit cannot drift from the constant behind it. A typed copy beside a formula is checked against it.
  - A row with a built-in unit's id must agree with it to 1e-9, or the stack is refused as a redefinition ("add a unit with a new id instead"). A pack unit of the same id must agree too.
  - New units join the session's registry, so a league typed in `derive` is a league.
  - While nothing behind a unit is edited, the built-in factor stays bit for bit. Halve c in a what-if and the metre stays while the light year follows.
  - `read_matter` shows a unit's worked-out factor.
  - Units only ever change how a number is shown and typed. The kernel works in SI.
- **`read_reach {constant, value?}`** returns `reach {constant, real, in_force, proposed?, follows, estimated, cannot_follow}`, with every registered block in exactly one group. A follows entry says whether it uses that constant.

## Not yet (part 2), and why
- **Estimated.** The main-sequence luminosity, radius and lifetime rescaled by stellar homology (Adams 2008, doi:10.1088/1475-7516/2008/08/010), stamped ESTIMATED with the law named. The group exists and is empty, and the stars stay "cannot follow" until the scaling is built and tested.
- **The lint**, that no crate outside `sse-reference` types a physical constant. It is MP0's `scripts/audit-matter.py --retired`, run in CI once the owning threads convert their crates. A CI change needs Alex's yes.
- **More constants in the handle**, and the other crates (formation, surface, weather, light). Each owning thread converts its own, as the design's package table says.
- **Earth's two radii named apart.** The sheet already has `R_earth_equatorial`, `R_earth_polar` and `R_earth_mean`. The code's `R_EARTH` still means equatorial in `sse-core` and mean in `sse-surface` and `sse-atoms`. Renaming them is part 2, crate by crate.
- **The Sol fixture checks for air scale height halving and the Sun's luminosity marked ESTIMATED.** Scale height follows G only through surface gravity, which already doubles. A test through the kernel's full resolve comes with part 2.

## Verified by
- `derived_constants_are_worked_out_and_checked`, `halving_h_moves_stefan_boltzmann_by_eight` (sigma goes as h^-3, and the handle follows), `checked_by_checks_but_never_works_out` and `formulas_parse_as_written`.
- `the_base_sheet_loads_clean`: PR 93's sheet loads. Once sigma_SB is typed as CODATA's 5.670374419e-8, its handle equals the compiled constants bit for bit. That is the precondition for every golden staying identical. The citations thread was asked for that change.
- `doubling_g_shortens_the_year_by_root_two_and_doubles_gravity`: on the base physics stack with a fictional twice-G pack, Earth's orbital period is 1/√2 of the real one to 1e-12, and its surface gravity doubles. The real kernel is untouched afterwards.
- `the_reach_report_lists_every_block_once`: every atom appears exactly once, every declared block is a real atom, and an unknown constant is refused.
- `units_follow_their_constants_and_cannot_be_redefined`: the light year agrees with the built-in one and is not re-added, a league is added, a 999 m kilometre and an unknown dimension are refused, and halving c moves the light year.
- `a_unit_from_the_sheet_can_be_typed` (kernel): a semi-major axis of a million leagues gives the same period as 4.828032e9 m.
- `with_sets_and_restores` (sse-core).

## Goldens
`tests/golden/contract.jsonl` was re-blessed on purpose for one new contract case (`read_reach` for G) and the op lists. Every other golden is identical: the handle holds the compiled values, which is MP1c's acceptance test.

Golden: the contract's `read_matter` listing response gains `"units":[]`
(the units table is new here) and `"origin_legend":{}` (from MP1's
legend follow-up). Both on purpose.
