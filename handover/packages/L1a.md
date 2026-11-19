# L1a: box B, energy and nutrient budgets per habitat

Status: done (19 Nov 2026). Kernel track, PR into `main`. Design: life design section 3 (box B), section 14 row L1a.

## What was built
`sse-life::budget`, over an L0 habitat graph and a set of metabolisms (which exist is box D's question, L1b):
- **Closures** (cap zero, with the reason in words):
  - no chemistry for the solvent in the pack stack (Titan's seas);
  - the temperature range misses the 253 to 395 K window;
  - water below its triple-point pressure (Mars's surface);
  - water activity below 0.585 (Venus's clouds);
  - a dose above 1000 Sv per year;
  - no installed metabolism can use the habitat's energy channels.
- **Energy cap**, per metabolism, in g C fixed per year:
  - Light: flux x the share of a blackbody at the light's Wien peak inside the metabolism's band (Simpson's rule, 400 intervals; `band_fraction`) x efficiency x area. Anoxygenic photosynthesis is also capped by its electron donor (Fe2+ at 0.25 C per Fe, H2 at 0.5 C per H2).
  - Supply: donor supply x carbon per mole, capped by the oxidant if it needs one (aerobic chemolithotrophs: reductant 0.1, O2 0.2; methanogens 0.01 per H2).
  - Light metabolisms compete for the same light, so the best one counts. Supply metabolisms add.
- **Nutrient cap:** new phosphorus per year (`material.p_new_mol_yr`) x reuse before burial (by habitat kind) x C:P (by medium). Production is the smaller of the two caps (Liebig).
- **Power, biomass, feeding levels:**
  - power = production x 39.9 kJ per g C;
  - biomass = production x residence time (by habitat kind);
  - feeding levels = 1 + floor(log10(density / 0.1 g C per m^2 per yr)), with 10% transfer;
  - without a high-power (oxygen-making) metabolism anywhere on the world, levels are capped at 2;
  - biomass classes are none, trace, sparse, modest, rich, teeming (above 100 Gt C).
- **Unknown, never zero:** a habitat whose inputs nobody supplied has production `None` and lists them in `missing`, and the world lists it in `unknown`. The world totals sum known habitats only.
- **`envelope(budget, epoch)`:** the L0 `LifeEnvelope` with `energy_budget_w`, `biomass_class`, feeding levels and high power. It is labelled plausible reconstruction, and P(present) stays `None` (box C, L1b).
- **Pack:** `provides.other.life_budget` in `packs/base-life`. Every setting is there with its source: the window, the limits, the stoichiometry, reuse, C:P, residence times, transfer efficiency and five metabolisms (oxygenic, anoxygenic on iron, anoxygenic on hydrogen, aerobic chemolithotrophs, methanogens).

## Checks (`cargo test -p sse-life budget`, 8 tests)

| World | Result | Reference | Notes |
|---|---|---|---|
| Earth today | 80 Gt C per year (ocean 38, land 42) | Field et al. 1998: 105 | Ocean nutrient-limited, land energy-limited. Biomass 419 Gt C against Bar-On et al. 2018's 550; teeming; 4 feeding levels; high power. |
| Early Earth | 3 Gt C per year, 27 times lower | Canfield et al. 2006 (iron-based, 10 to 25 times lower) to Kharecha et al. 2005 (H2-based, thousands of times lower) | Fixture is today's with a ferrous ocean (Fe2+ 1e15 mol per year) and only early metabolisms. Energy (donor) limited, at most 2 levels, modest. |
| Europa | 2.4e9 g C per year | Chyba and Hand 2001: 1e8 to 1e14 | Oxidants 1e10 and reductants 1e9 mol per year (Hand et al. 2007); sparse. |

- Closed, each with its reason: Mars's surface and soil (triple point); Venus's ground (window) and clouds (water activity 0.004, Hallsworth et al. 2021); Titan's hydrocarbon lakes (no chemistry). Titan's ocean is unknown, with its missing inputs listed.
- Band fraction: the Sun puts 34 to 40% of its energy into 400 to 700 nm (we get 36.7%), and a 3000 K star under 15%.
- 10,000 keyed random habitats:
  - production never above either cap, nor any metabolism above the energy cap;
  - nothing is NaN or negative, and a closed habitat is exactly zero;
  - an unknown habitat always says what is missing;
  - every result is reproducible.
- Golden `tests/golden/life-budget.json` (new), harness engine `life-budget`.

## Discrepancy logged
The design says early Earth was "hundreds to thousands" of times less productive. Canfield et al. 2006 (table 12) give Fe-based anoxygenic production of 1.7 to 5.0e14 mol C per year against today's marine 4e15, only 10 to 25 times lower; H2-based alone gives about 1000 times. The check uses the band 10 to 3000 and cites both.

## Golden and fixture changes (on purpose)
- New inputs in the world fixtures (box B needs them; L0 read none of them):
  - Earth: sunlight by climate band (Wild et al. 2013); new phosphorus for the ocean (3e10 mol per year) and for land (1.5e11, shared by land area).
  - Europa: oxidant and reductant supply.
  - Venus: cloud water activity.
- So `tests/golden/life.json` (graph hashes) and `tests/fixtures/life/interfaces.json` (Europa's sample profile now has materials) were re-blessed. No shape changed.

## Known limits
- **Double counting:** a shell's material supply is read by every habitat derived from that shell. On Europa, the sea floor and the open ocean both use the same oxidant supply, so the world total counts it twice. Sharing a supply between habitats (or labelling each supply with its habitat) belongs with the flux coupling in L2.
- **Heterotrophs:** soils, the ocean interior, the deep crust and caves on Earth are unknown, because they live on imported carbon. Imports along graph links are L2's.
- **Feeding levels:** for a global volume without an area, levels use volume^(2/3) as the area. Rough.
- **Cloud decks:** no light flux is given in the fixtures yet.

## Integration points
- L1b (boxes C and D) reads `WorldBudget`: P(present) needs an open, productive habitat. Which metabolisms are installed becomes box D's output instead of an input list.
- The planet chart's life panel (UI track) can show `limited_by`, `closed` reasons and `missing` per habitat.
- Shared files touched:
  - `crates/sse-wasm` (engine `life-budget`) and the harness lists;
  - `packs/base-life`;
  - `tests/fixtures/worlds/{earth,europa,venus}.json`.
