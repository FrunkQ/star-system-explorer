# MP6: a body's elements on a click (proven core thread, 23 Nov)

Accepted by Alex 23 Nov ("build them as stated, use blocks"). The design is in the matter pack design, section 5, steps 3 and 4, and the build table row MP6. It builds on MP24 (each element's condensation host, CI share, and Earth's core, mantle and crust, PR 301) and PR 311 (CI H and O, rock oxides).

## Blocks

`packs/base-physics/formulas/composition.json` holds five Equation blocks, each with sources, words and worked examples:

| Block | What happens |
|---|---|
| `sse.composition.share_in_material` | An element's share of the material it rode in on: CI × the star's ratio, over the material's summed weight (bound oxide oxygen included) |
| `sse.composition.separation_strength` | How far a world melted and sorted itself: (3/5 GM/R + Al-26 left) / energy to melt, capped at 1 |
| `sse.composition.core_share` | How much of an element sank with the iron: s · f·D / (f·D + 1 − f) |
| `sse.composition.crust_share` | How much of the rock's element rose into the crust: s · fc·E / (fc·E + 1 − fc) |
| `sse.composition.crust_fraction` | The crust's share of a world's rock (a shell of set thickness) |

The chain runs the same sums from `sse_atoms::matter`, once per element. The test `composition_blocks_match_the_chain` holds the two together.

## Chain and op

`sse_surface::composition` (`composition-v1`) works as follows.

**Materials.** Formation's ledger supplies them (`Protoplanet.by_group`, scaled to the body's solids). A world built from facts gets its materials from its core share and its water instead.

**Elements.** Each element is a CI rider of its material. Hydrogen and oxygen come with their carriers instead: water's h2o formula, and the oxides of the rock-forming elements by stoichiometry.

**Reservoirs (Survey only).**
- Solids separate into core, then crust, with the mantle holding the rest.
- What came in the water goes to the sea (h2o's elements) and the air (the rest), in the share the world outgassed.
- Totals are identical at both levels.

**Al-26.** Formation-run bodies take Al-26 from `sse.heat.al26_planetesimal_heating` at their formation time.

**Star ratios.** They read as 1 until MP7 lands. The kernel thread agreed the shape: `sse_formation::inherit::star_element_ratios(inh, seed) -> BTreeMap<id, ratio>`, a linear ratio by mass per hydrogen.

The kernel op is `read_composition {system, body?, level: "orbit" | "survey", at_s?}`. It returns `composition_read {bodies}` and reads worlds through `worlds_at`, so no surface stage runs.

## Checks

These are in `crates/sse-kernel/src/sol_tests.rs`, from `packs/base-sol` alone:
- Earth against McDonough and Sun 1995 bulk Earth: O, Si and Mg within 15%. Fe is within 30%; it reads 23% against 32%, because metal keeps CI's sulfur until MP14 (fudge register).
- The elements sum to the body.
- Earth separated fully.
- At Survey, Earth's core holds over 90% of its gold, and its crust holds over ten times its share of the rock's potassium. Its sea holds hydrogen and its air nitrogen.
- Every element's Orbit total equals its Survey reservoirs summed.
- Ceres barely separates (s < 0.1).
- An unknown level is refused.

## Goldens (on purpose)

`tests/golden/contract.jsonl` changes because of:
- the new op in `hello` and in the unknown-op message;
- the base-physics stack hash, and the manifests that carry it, because the pack gains five blocks;
- the G reach list (`separation_strength` reads G);
- four appended cases (a new universe, a formation adoption, a composition read, a refused level).

`packs/base-physics/pack.json` and its fixture are re-blessed for the five blocks.

## Fudge register

Five rows were added under Interiors and heat:
- coarse materials with no per-element volatility;
- single partition and enrichment values;
- a 20 km crust;
- materials for worlds built from facts;
- water riders going to the air.

## Next

**MP8.** Deposits: ore-forming processes enrich the crust locally. The deposits are part of the crust reservoir, so totals hold at Site too. This also builds TG11's inventory over `read_composition`.
