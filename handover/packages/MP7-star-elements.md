# MP7: a star's elements (matter pack design 8.1)

Approved by Alex on 23 Nov. The aim: a star's inheritance returns any element's abundance. The Sun returns solar values for every element. Stars with different merger histories get different gold.

## What changed
- **One Equation block**, in `packs/base-cosmos` at `provides.other.element_sources.equation`, named "How much each kind of source made for this star". It works out each source's scale against the Sun's from the star's inheritance:
  - massive stars: 10^([Fe/H]+[α/Fe]);
  - exploding white dwarfs: whatever iron then needs, (10^[Fe/H] − f_Fe,massive · s_massive)/(1 − f_Fe,massive), floored at 0;
  - low-mass stars: whatever C/O then needs, with the same form over carbon's shares;
  - cosmic rays: 10^[Fe/H];
  - neutron-star mergers: the star's own `r_process_rel`.
  - The big bang is 1 and human-made is 0.
  - The Sun's C/O is read from `patch_model.c_o.solar`. The iron and carbon shares are read from the sheet, never typed.
  - Source: Kobayashi, Karakas and Lugaro 2020. The worked examples (the Sun, and a halo star at [Fe/H] −1, [α/Fe] +0.3) run on load.
- **`crates/sse-kernel/src/star_elements.rs`**:
  - `origin_shares(sheet)`;
  - `source_scales(inh, shares)`;
  - `star_element_ratios(inh, sheet) -> BTreeMap<element, ratio>`. Each ratio is linear by mass per hydrogen against the Sun: the element's `origin.shares` weighted by the source scales.
  - This is the shape agreed with proven core, except that it takes the sheet for the shares. The seed argument waits for the sheet's `scatter_dex` rows; per-star r-process patchiness already comes through `r_process_rel` (GH's per-star [r/Fe]).
- **`read_composition`** (`composition.rs`) passes the star's ratios to `compose` whenever the system has a formation run. Otherwise it keeps the Sun's, and the stand-in now reads "no formation run for this system, so its star's elements read as the Sun's".

## Checks (`sse-kernel/src/star_elements_tests.rs`)
- The Sun's inheritance gives 1 for every element the Sun's cloud had, to within 1e-9.
- A halo star at [Fe/H] −1 and [α/Fe] +0.3 gives oxygen 10^−0.7, iron exactly 0.1 and hydrogen 1. Past [α/Fe] = log 2, the white dwarf scale floors at 0.
- With `r_process_rel` 0.5 against 3.0, gold moves more than four times, while iron and oxygen don't move.

## Golden (on purpose)
`tests/golden/contract.jsonl` line 353, the `read_composition` case, changes only the stand-in sentence, because that case has no formation run. No G golden changes.

## Next
- Per-star scatter from the sheet's `scatter_dex` rows when the data sheet thread adds them (seeded per star).
- MP9's unobtainium (made in mergers at a thousandth of gold) follows these same ratios without new code.
- "For this star" on the Reference screen (MP10) can quote the source scales.
