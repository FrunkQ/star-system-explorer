# MP8a: what a world's deposits hold (proven core thread, 24 Nov)

Accepted by Alex 23 Nov ("build them as stated, use blocks"). The design is in the matter pack design, section 5, step 5 (concentration), and build table row MP8. MP8 comes in two parts: this one (deposits hold real kilograms) and MP8b (TG11's inventory over `read_composition`, with resource kinds naming matter ids).

## Block

`packs/base-physics/formulas/composition.json` gains `sse.composition.deposit_amount`:

- grade = clamp(c · k, 0, 1), where c is the element's share of the world's crust (MP6's Survey crust) and k is the process's enrichment;
- mass = A · h · ρ · f · grade, where A is the footprint, h is the depth counted (3.3 km), ρ is the crust density and f is the ore share.

The same sum is `sse_atoms::matter::deposit_amount`, held to the block by `composition_blocks_match_the_chain`.

## Data

`packs/base-surface` gains `cover.ores`:
- `depth_m` is 3300, Kesler and Wilkinson's depth.
- `ore_share` is 4.5e-5. It is calibrated so that Earth's arc copper holds their 1.7e11 t of porphyry copper. The same value is used for every kind (fudge register).
- Each kind lists its enrichments, a typical ore grade over Rudnick and Gao's crust:
  - copper-arc: Cu 160 (Singer et al. 2008);
  - gold-orogenic: Au 3800 (Goldfarb et al. 2005);
  - nickel-flood: Ni 250 and Cu 370 (Naldrett 2004);
  - nickel-impact: Ni 200 and Cu 400 (Lightfoot 2017).
- Carbon-buried and ice-shadowed deposits hold nothing counted yet; the stand-ins say so.

Element ids are the sheet's, in lower case (`cu`, `au`, `ni`).

## Chain and op

`sse_surface::composition::holdings` (`deposit-holdings-v1`) takes the Survey composition, S7's layers and `cover.ores`.

**Footprints.** A kind's footprint is its plate cells (the sphere's area shared over the 6n² cells) plus its point deposits' discs.

**Amounts.**
- Each kind gets its amounts.
- Each point deposit (an impact melt sheet) gets its own share.
- Where the deposits would hold more of an element than the crust has, every deposit's amount is scaled to the crust.

**Crust split.** For each element named, `crust` gives crust, in deposits and disseminated. Deposits and disseminated always add to the crust `read_composition` gives, so totals hold at every level.

`survey_surface` now returns `holdings` beside `deposits`. Its `places` follow the same listing limit. The kernel builds the composition with the same `bulk_of` as `read_composition` (facts, plus formation's ledger and Al-26 for run bodies), so the two agree.

## Checks

`sol_tests::earths_deposits_hold_its_porphyry_copper` uses base-sol's Earth, surveyed in full:
- arc copper is within 30% of 1.7e14 kg;
- orogenic gold is above zero;
- every element's deposits plus disseminated equal its crust;
- that crust equals `read_composition` at Survey.

`composition_blocks_match_the_chain` also covers `deposit_amount`. The atom count goes from 90 to 91.

On Earth with the calibrated share:
- orogenic gold reads about 6e7 t, which is in line with known orogenic gold scaled the way Kesler and Wilkinson scale copper;
- flood and impact nickel read about 1.3e13 kg together.

## Goldens changed on purpose

- `packs/base-physics/pack.json` and `tests/fixtures/packs/base-physics.json` gain the block.
- `tests/golden/contract.jsonl`:
  - stack hashes and manifests follow the two pack changes;
  - the two `survey_surface` cases gain `holdings`.

## Next

MP8b, TG11's inventory: resource kinds declared by packs name matter ids; inventory per body comes from `read_composition` and `holdings`. Kept kinds and the mining program stay with the Traveller pack thread.
