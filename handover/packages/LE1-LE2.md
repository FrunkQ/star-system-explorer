# LE1 and LE2: the keep-alive test and the energy ledger

Track LE (life from the energy it can get), design `design/SSE4-life-from-energy-design.md` sections 3, 4 and 9, accepted by Alex 26 Nov 2026. Built 27 Nov 2026 by the thread "Build life's energy baseline".

## What changed

**LE1, keep-alive in the shared step.** `budget::finish`, which the listed model (`listed/1`, the default) and the gradient catalogue (`gradients/1`) both run, now tests captured power against maintenance at the habitat's temperature:

- below the **starved** maintenance of the smallest population (1e-9 C-mol, about a million cells): closed, with its numbers ("too little energy to keep anything alive: 1.7e-18 W is captured here; keeping even the smallest population alive at 275 K costs 9.1e-16 W");
- below the **laboratory** maintenance of the thinnest density of cells ever counted (1e-5 g C per m³), through the whole habitat (its volume, or its area times a 1 m living layer): open and `barely` ("alive, barely"), with no feeding levels;
- standing biomass is the smaller of production x residence and power / starved maintenance per gram, and `biomass_limited_by` says which.

The numbers are one shared cited block, `life_budget.maintenance` in `packs/base-life` (laboratory 1.25 W per C-mol at 298 K, Tijhuis et al. 1993; starved 1.2e-5 W per C-mol, LaRowe and Amend 2015; activation 69.4 kJ per mol). The four maintenance fields left `life_gradients`; `gradients/1` reads the starved value for its per-gradient power test and `max_biomass_gc`. A stack without the block runs no test and lists `life_budget.maintenance` in `missing`.

Why these two populations: the laboratory value applied to standing biomass would shrink Earth's forests about 300 times (wood is cheap to keep), so it is the test for "can a population grow", applied to the thinnest real population; the starved value is the floor of what living carbon costs, so it closes a habitat and caps biomass.

**LE2, the energy ledger.** New `sse-life/src/ledger.rs`: per habitat and for the world, `arriving_w` (light at the ground x area x Petela's exergy factor, plus chemical supply), `offered_w` (energy cap of every metabolism the pack lists), `captured_w` (production power of what life holds), `keep_alive_w`, `surplus_w`, `brought_in_w` (always 0 in 4.0, tech's seam). `life_view` for a world of the chart carries it as `view.energy`. `absorbed_sunlight_w` is null with a note until the climate's sunlight budget (PR 678) is on main. Fixture life views carry no ledger (they have no habitats).

## Results

- Earth (fixture and catalogued): unchanged totals (production 80 Gt C a year, biomass 419 Gt C), captured about 100 TW, surplus positive, residence binds everywhere.
- Europa (fixture and catalogued): open and "alive, barely": about 1.5e6 W captured per habitat against about 2.6e8 W to keep the thinnest population growing through its ocean.
- No dated history moved (life-history golden unchanged).

## Checks

New tests: `budget_tests` (a starved habitat closed with its numbers in both models; Europa alive, barely; Earth grows and keeps its totals; maintenance caps biomass in a warm starved crust, an absolute check; a stack without the block says so), `ledger_tests` (Petela values; Earth's lines sum and capture 130 TW within a factor of three; captured <= offered <= arriving on 10,000 random worlds; a lifeless world captures nothing), `sse-kernel` `life_tests::life_view_carries_the_energy_ledger`.

Seen red first: with the closure, the "barely" test and the biomass ceiling switched off, `a_starved_habitat...`, `europa_is_alive_barely...` and `maintenance_caps_biomass...` failed; with the chemical part of `arriving` zeroed, the 10,000-world order check failed.

## Goldens

`tests/golden/life-budget.json`, `life-biosphere.json`, `life-feedback.json`: hash lines only (new fields, and the budget settings in the coupled manifest). Core `tests/golden/contract.jsonl` and every area golden: unchanged (re-blessed, no diff). So this package is not in the core lane.

## Registers

Fudges F66 to F70 (two maintenance values with one activation energy, the smallest population, the thinnest density and the 1 m layer, the biomass ceiling, the chemical part of arriving) and a "candidates to move into flows" row. Citations rows in `docs/citations/climate-life.md` section B; new papers added to the master list with `scripts/citations.py --add` (the tool also picked up 33 other threads' missing papers, added as unchecked). Every figure is unverified: the paper search is out until 1 Dec 2026.

ER ledger: no new side door (`R_GAS` read from `sse_atoms::chem` as `gradients.rs` already does; the ledger reuses box B's constants). The `sheet` door (ER4) is not built, so `budget.rs`'s six sheet reads stay.

## Integration

- LE-U1 (Energy flow life branch) reads `view.energy`: habitats with `arriving_w`, `offered_w`, `captured_w`, `keep_alive_w`, `surplus_w`, `brought_in_w`, `light_w`, `chemical_w`, `exergy_factor`, `barely`, `closed`; `world` (same lines), `unknown`, `absorbed_sunlight_w`, `notes`.
- Once PR 678 is on main, `absorbed_sunlight_w` can read `survey.climate.sunlight.absorbed` x the sphere's area.
