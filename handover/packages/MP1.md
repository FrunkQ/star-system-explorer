# MP1: the master data sheet's schema and reader

Matter pack design (`/mnt/project-files/sse4/SSE4-matter-pack-design.md`, repo PR 89) section 4 and section 10, package MP1. Built 21 Nov 2026 by the kernel thread, to the design's recommendations (Alex has not yet ruled on questions 2 to 13). Code: crate `crates/sse-reference`, the kernel op in `crates/sse-kernel/src/lib.rs` (`read_matter`), tests in `crates/sse-reference/src/tests.rs` and `crates/sse-kernel/src/matter_tests.rs`. Fixtures: `tests/fixtures/packs/reference-five-errors.json` and `reference-toy.json`.

## What it delivers
- **`provides.reference`** on any pack (`sse_pack::Provides.reference`, skipped when empty, so no existing pack or stack hash changes). Tables: `constants`, `elements`, `isotopes`, `substances`, `mixtures`, `reactions`, each a list of rows with an `id`, plus `sources` (`[{id, text, doi?, url?}]`). Any other table name is refused.
- **Values.** `{"v", "unit", "src"}`, with `valid` and `status` (`verified` or `unverified`, from the citations thread) allowed. A fit is `{"fit", "coeffs", "valid_k", "src"}`. A value may sit at the top of a row or inside a property group (`phases`, `thermal`, `mechanical`, `optical`, `chemical`, `cosmochemical`).
- **Stacking.** Packs merge bottom to top. A row whose id is already present is merged value by value, and each value it sets must carry `replaces` (the pack it overrides), or the stack is refused. New rows need nothing. `read_matter` lists the packs that set each row.
- **Load checks** (design section 4.6), all reported at once as readable sentences:
  - every value names a source, and every source id resolves in `reference.sources`;
  - a substance's formula names known elements, and a typed molar mass (kg/mol or g/mol) agrees with the formula's within 0.1 percent;
  - a vapour curve passes through its own triple point (ln P within 0.05), and a sublimation latent heat agrees with the curve's slope within 15 percent. Fits read: `fray-schmitt-2009` (ln P[bar] = A0 + Σ Ai/T^i) and `clausius-clapeyron` (`[p_ref Pa, t_ref K, L J/mol]`);
  - a mixture's recipe names known substances and sums to 1;
  - every reaction (`terms: [[substance, coefficient], ...]`) names known substances and conserves elements and charge.
  - a string `about` on the reference block is a note and is skipped; an element with no standard atomic weight (no stable isotope: Tc, Pm, Po onward) is allowed, and a formula using one has no worked-out molar mass; an empty recipe is a bulk-only mixture (rock, magma and the formation materials, for now). The recipe tolerance is 1e-4, room for trace gases a recipe leaves out.
  A failing sheet fails `load_stack` (`stack_rejected`) like any other pack problem.
- **`read_matter {table?, id?}`** → `matter {value}`. With table and id: the row, its worked-out molar mass, its sources, the packs that set it, and `used_by` and `skipped_by`. With id alone: the first table holding it. With neither: each table's ids. An unknown row gives `no_row`, an unknown table `no_table`. Any seat may read it.
- **Stack hash.** The sheet is part of the pack manifests, so it is in the stack hash already. The same stack hashes the same on two kernels (tested).

## Not yet, and why
- **`used_by` and `skipped_by` are empty, and blocks do not yet declare what they read.** No block reads the sheet until MP1c (constants) and MP3 to MP6 (volatiles, light, life, formation). The declaration lands with the first reader, so it is designed against a real use, not guessed.
- **`packs/base-reference`** is MP2's data, written by the citations thread in the row shape agreed on 21 Nov. MP1 ships only fixtures.
- **Substance ids named elsewhere in the stack** (gas mixes, spectral lines, cloud decks, resource kinds) are not cross-checked yet; each reader adds its own names to the check when it moves onto the sheet (MP3 to MP8). Within the sheet, every id is checked.
- **Marked rows.** "A failed check on an authored or fictional row loads with the value marked impossible" is MP9. Until then every failure refuses the stack.

## Verified by
- `five_mistakes_give_five_messages` and the kernel's `a_sheet_with_five_mistakes_is_refused_with_five_messages`: the fixture has a wrong molar mass for water, a CO2 vapour curve that misses its triple point by a factor of ten, a recipe summing to 0.9, an unbalanced hydrogen burn and a reaction naming `unobtainium`. Exactly five messages come back, one each.
- `an_override_must_say_what_it_replaces`: an override without `replaces` is refused; with it, the value changes and the rest of the row stays; a new row needs nothing.
- `charge_is_conserved_and_unknown_names_are_caught`: two protons becoming H2 balances elements but not charge.
- `fray_schmitt_form_is_read`: CO2's published coefficients give one atmosphere at 194.7 K, within 5 percent.
- `the_base_sheet_loads_clean`: `packs/base-reference/pack.json` (MP2, PR 93) loads with no problem. Before that pack is on main the test reads it from `SSE_BASE_REFERENCE` or skips; it was run on PR 93's file on 21 Nov.
- `read_matter_gives_a_row_with_its_sources_and_packs`: water's row, its molar mass worked out (0.018015 kg/mol), a lookup by id alone, a listing, an unknown table refused, and the same stack hash on two kernels.

## Goldens
`tests/golden/contract.jsonl` was re-blessed on purpose for two new contract cases (`read_matter` with no sheet loaded, and an unknown row) and the new op in `hello`'s op list and in the unknown-op message. No other golden changed.
