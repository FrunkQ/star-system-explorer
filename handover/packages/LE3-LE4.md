# LE3 and LE4: where the energy goes, and the speed read-out

Track LE, design `design/SSE4-life-from-energy-design.md` sections 5, 6.1, 7 and 9. Built 27 Nov 2026 by "Build life's energy baseline", after LE1 and LE2 (PR 680).

## What changed

**LE3, where the energy goes.** Each guild on `life_view` (`view.guilds`) now carries:

- `diet`: `gradient:<metabolism>` for a producer; otherwise the guilds it takes carbon from (from the resource relations);
- `power_w`: power through it (production or intake);
- `uses_w`: power it finally spends: intake less what the next level eats from it; a decomposer takes everything not grazed. Summed over every guild this is exactly the producers' captured power;
- `share_of_captured`: `uses_w` over the captured power (sums to 1);
- `sizes` (eating guilds only): rough numbers in broad classes (cells, insect-sized, animal-sized, tree-sized), by spreading `uses_w` evenly over log body mass from the guild's smallest member to its largest (energy equivalence) and counting at Kleiber's rate with Hemmingsen's three b0 values (single cells, cold-blooded, warm-blooded from 10 g on land). Producers carry no `sizes`: a gap, never a zero.

Settings: `life_budget.sizes` in `packs/base-life`.

**LE4, the speed read-out.** New `sse-life/src/speed.rs`; `view.speed` on a world of the chart:

- per habitat: pioneer growth rate `r_per_s` (the smaller of a cell's best, 1 a day at 288 K moved by 0.65 eV, and the surplus per gram of the thinnest population over the free energy built into a gram) and what set it (`biology`, `energy`, `none`), mixing `D` by medium, Fisher's front `2 sqrt(r D)`, the span (width, or thickness for rock) and `fill_yr`;
- `pace_x_earth`: opportunity per Myr now in Earth-Myr, from `history::opportunity_rate`, now the one copy of the rule (the history steps it, the read-out quotes it); and `words`: "life here moves and changes at about 1.1 times Earth's pace".

Settings: `life_history.spread`.

## Results

- Earth (fixture): pace 1.1; the sunlit ocean fills in about 3 years, land in 6 to 14 years (biology-limited). Catalogued Earth's pace is checked between a third and three.
- Europa: alive, barely, so no front moves (`r` 0) and pace is far below Earth's.
- Rock (a vent field's crust, 10 km across and 1 km deep, well fed): pioneers biology-limited, the front under 1e-5 m/s, so it fills more slowly than an ocean.
- Earth's warm-blooded animal-sized land grazers: 1.8e12 of them, about 0.12 Gt C.

## A design check changed, and why

The design asked for "Damuth's mammal densities within a factor of ten". Damuth's line (about 10^4.23 M^-0.75 per km², M in g) is one species' density; LE3 counts every warm-blooded grazer of a class, so the two differ by the number of species (ours read 260 times Damuth's single-species figure at 2.6 kg). The test compares mass instead: the class's standing mass against Earth's land mammals today, about 0.17 Gt C (Bar-On et al. 2018, livestock and people included), within a factor of ten. It reads 0.12 Gt C. Reported to the coordinator.

## Checks

New tests: `biosphere_tests::shares_sum_to_the_captured_power_on_every_world`, `earth_land_grazers_weigh_about_what_earths_land_mammals_do`; `speed_tests` (Earth reads one; fronts have the right orders of magnitude, absolute speeds; growth falls to zero smoothly and monotonically at the keep-alive line; a barely-alive ocean has no front); `sse-kernel` `life_tests::life_view_shares_the_energy_and_reads_earths_pace`.

Seen red: with the decomposer taking all production (not less the grazers'), the share test failed; with the energy cap on `r` removed, the smooth-to-zero test failed; counting only productive habitat kinds for the pace (a first draft) read 0.18 and failed "Earth reads one".

## Goldens

`life-biosphere.json`, `life-feedback.json`, `life-signatures.json`: hash lines only. `life-history.json` unchanged (the opportunity refactor is exact). Core contract and area goldens: no diff.

## Registers

Fudges F71 to F74 and a flow-candidate row; citations rows in `docs/citations/climate-life.md`; master list updated. All new figures unverified (paper search out until 1 Dec 2026).

## Integration

LE-U2 (Life screen card "Where the energy goes") reads `view.guilds[].diet`, `uses_w`, `share_of_captured`, `sizes[]` (`class`, `from_kg`, `to_kg`, `rate`, `power_w`, `count`) and `view.speed.words`, `pace_x_earth`, `habitats[]`. The UI types in `packages/ui-contract/src/life.ts` do not have these fields yet; that is the UI thread's.
