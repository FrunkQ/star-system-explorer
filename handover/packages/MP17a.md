# MP17a: the Look up block

Matter pack design (`design/SSE4-matter-pack-design.md`) section 3A, package MP17a, on Alex's word of 22 Nov 01:03 ("a special lookup block for all these things, rather than in code or in a massive block, as getting the physical properties for a material is a basic thing probably used everywhere"). Built 22 Nov 2026 by the kernel thread.

## What it does
- **`sse.reference.lookup`** (`crates/sse-logic/src/lookup.rs`), an engine-class block whose editor is the Reference screen. Config `{what?, property, conditions?}`:
  - `what` names a row: `h2o`, `constants/G`, `elements/fe`. A bare id must be in one table only, so `h` (Planck's constant and hydrogen) is refused with a hint to write `constants/h` or `elements/h`. Left out, the block grows a `what` input (a row reference) so a flow can loop over substances.
  - `property` names the value: by its path (`phases.triple_point_k`), or by its last name when the row has it once (`latent_vaporisation`). A constant needs none.
  - `conditions: true` adds `p_pa` and `t_k` inputs.
  - Outputs: `value` (its dimension is the sheet's unit for that property, so a density wires into a density port), `phase`, `known` and `in_range`.
  - The node's note is its "Why?": the row, the property's path, the unit, the source row with its DOI or URL, the evidence grade, the packs that set the row (`edited` when more than one did) and the reason when the value is not known.
  - It keeps no number of its own and refuses to load with no master data sheet in the stack, or naming a row the sheet lacks.
- **How a property is found** (`crates/sse-reference/src/lookup.rs`):
  1. at given conditions on a substance, the phase there (MP17's `phase_at`) and that phase's own `props`, so "density of water at 1 GPa and 270 K" answers 1310 kg/m³ for ice VI without the flow knowing ice VI exists. `in_range` is then whether the conditions sit inside the phase's stated evidence range;
  2. for a familiar phase, `<property>_<phase>` on the row (`phases.density_liquid`);
  3. the property by path or last name. Two places with the same last name is an error naming both paths;
  4. worked-out values: a substance's `molar_mass` from its formula.

  Fits are read where the reader knows the form (the vapour-pressure fits, through the same sublimation reading `phase_at` uses, and `k = a / T`). They need a temperature: without one the answer is `known` false with the reason. `in_range` is the fit's `valid_k`. A row without the property answers `known` false and a value of 0; its unit comes from another row that has the property, so the flow still loads.
- **The sheet in force** (`crates/sse-flow/src/sheet.rs`): a `Sheet` trait the reference implements, and a per-thread slot the kernel fills for each call beside the constants handle, and while a new stack's flows are checked and their golden cases run. The kernel's sheet is now an `Arc`, shared, never copied.
- **Load order.** `load_stack` now reads the sheet (and the rest of the observe set, and the sheet's units) before it compiles flows, so a flow can use a sheet unit and a Look up block can check its row. A stack with problems in both is now refused for the sheet's first.
- **"Used by", read off the wiring.** `read_matter` on a row lists every flow with a Look up block naming that row: `{flow, node, property, pack}`. A block whose row is wired in reads whatever arrives, so it is not listed. `skipped_by` stays empty (see below).
- **mu0** is already on the sheet's constants (the master data sheet thread added it).

## Verified by
- `the_look_up_block_reads_the_sheet` (sse-kernel, fixture `tests/fixtures/packs/lookup-flows.json` over the real base sheet):
  - water's latent heat through a flow gives 2.468e6 J/kg, the sheet's value and the typed copy's;
  - the density at 1 GPa and 270 K is ice VI's, and at 1 atm and 300 K liquid water's;
  - a row without the property gives 0;
  - with `tests/fixtures/packs/lookup-edit.json` doubling the latent heat, that flow gives 4.936e6 and the other two flows give identical answers;
  - `used_by` on water lists exactly the three flows, and on CO2 none.
- `a_look_up_block_with_no_row_or_no_sheet_is_refused`.
- `lookup_tests.rs` (sse-reference): the unit reader; row naming and the shared id `h`; G with its source and c graded convention (exact by definition); latent heat by name, ice VI's density with grade and source, molar mass worked out; a missing property, a fit without a temperature, ice's sublimation pressure at 200 K, and bad conditions refused.

## Goldens
`tests/golden/contract.jsonl` changed on purpose in one line: the `read_reach` case for G now lists `sse.reference.lookup` among the blocks that follow (declared in `sse_atoms::reach::DECLARED`, since it gives whatever the stacked sheet holds). The contract stack has no data sheet, so nothing else moves.

## Not done
- **Marks along the wire.** The note says `edited` when a pack changed the row, but the flow's evidence does not yet carry it value by value (design 3A, "carries the mark along its wire"). This needs K5's propagation made finer, per value.
- **`skipped_by`**: which flows skipped a row because it lacked the property. It is known only when a flow runs over that row, so it wants a record of runs, not the wiring.
- **The node editor's face** (current answer and phase chip): the chart room thread, with the contract `explain` already carrying the note.
- **Converting typed copies.** No file touched here holds a labelled copy (the MP22 ledger is unchanged). Heavy blocks move to this door as their threads touch them.
- **The audit over packs** ("fails on a fixture flow with a typed density") belongs to MP22's `scripts/audit-matter.py`, which the master data sheet thread owns.
