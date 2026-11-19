# L1b: boxes C, D and E: origin, innovations, catastrophes, dated history

Status: done (19 Nov 2026). Kernel track, PR into `main`. Design: life design sections 4 to 6 and section 14 row L1b.

## What was built
`sse-life::history`. A world is a `WorldTimeline`:
- epochs, each a start age, an L0 habitat graph (or none) and world facts such as `o2_fraction`;
- physical catastrophes, each with an id, age, kind, severity, refuge habitat kinds and a reason.

`history(world, budget, params, seed)` steps through it in 5 Myr steps.

**C, origin**
- Each open habitat (box B's closures decide which) has a hazard: origin dial x base rate (0.0018 per Myr for one Earth surface) x the sum of its settings' factors x its area / Earth's surface.
- The settings are pack data, each switchable: alkaline vents (sea floor, under-ice ocean), wet-dry pools (land, lakes), delivered organics (sunlit water) and mineral catalysts (soil, crust, caves).
- Origin happens when the accumulated hazard passes -ln u, where u is a keyed draw per habitat and attempt. So a bigger dial can never make origin later.
- Life then holds every open habitat, in the origin media (water, subsurface, clouds).

**D, innovations**
- Opportunity per Myr is the product of:
  - (production / 8e16 g C per yr)^0.25;
  - a generation-rate factor exp(-0.65 eV / kT) against 288 K (mean temperature of open habitats);
  - habitat variety (open habitat kinds / 7).
- Production is box B's, for the metabolisms the world holds. Budgets are cached per epoch and capability set.
- Ten capabilities form a graph. Each has prerequisites (capabilities and world facts), a rate per Earth-Myr of opportunity, and optionally a dial (hard steps: `oxygenic`, `complex_cells`). Each unlocks metabolisms or media:

  | Capability | Needs |
  |---|---|
  | chemotrophy | given at origin |
  | dormancy | chemotrophy |
  | anoxygenic photosynthesis | light |
  | oxygenic photosynthesis | sunlit water |
  | complex cells | oxygen at 0.1% |
  | sex | complex cells |
  | multicellularity | complex cells |
  | large bodies | multicellularity, a high-power metabolism, oxygen at 10% |
  | hard parts | large bodies |
  | land colonisation | multicellularity, land, oxygen at 1% (ozone); unlocks the surface medium |

- A capability is acquired when rate x dial x accumulated opportunity passes its own keyed threshold. A lost capability gets a new draw.
- The world's oxygen fact counts only once a high-power (oxygen-making) metabolism exists, so it can never precede oxygenic photosynthesis.

**E, catastrophes**
- Each catastrophe costs the share of diversity = severity x exposed living habitats / all of them, and is recorded with the input's id as its cause.
- Capabilities younger than 30 Myr may be lost (a keyed draw against that share).
- A total event with no refuge sterilises the world. Origin can happen again (a new attempt).
- Diversity relaxes towards log10(1 + production / 1e6) with a 10 Myr time constant. A recovery event marks 90%, caused by the catastrophe.

**Output** (`LifeHistory`):
- dated events (origin, capability, capability_lost, catastrophe, recovery, sterilised), each with causes, a reason and the dials it depends on;
- points every 50 Myr: alive, capabilities, production, diversity, opportunity;
- each capability's first age;
- the dials;
- the evidence label (plausible reconstruction).

`record(chronicle, history, birth_year)` writes derived `life.<kind>` events whose chronicle causes link to the earlier life events.

**Pack:** `provides.other.life_history` in `packs/base-life`, with every rate, factor and dial and its sources.

**Fixtures** (`scenario.rs`):
- `earth_timeline()`:
  - Hadean without habitats to 150 Myr;
  - a ferrous ocean with hydrothermal H2 (3e11 mol per year) throughout;
  - oxygen inputs of 1e-6, 1% from 2.1 Gyr and 21% from 3.9 Gyr;
  - five catastrophes: Hadean impact, Huronian and Sturtian glaciations, end-Permian, end-Cretaceous.
- `europa_timeline()`: today's ocean held for 4.5 Gyr.

## Checks (`cargo test -p sse-life history`, 7 tests)
- **Earth outline in most seeds** (200 seeds, default dials):
  - life before 1 Gyr in 176;
  - microbial (no multicellularity) for the first 2.5 Gyr in 167;
  - large bodies, appearing after 3.5 Gyr, in 164;
  - all three in 118 (59%).
- **Dials move outcomes monotonically** (60 seeds):
  - a 10x bigger origin dial never makes origin later in any seed, and living worlds never fall;
  - oxygenic dial 0.1, 1, 10: oxygenic photosynthesis in 12, 57, 60 worlds;
  - complex-cells dial 0.1, 1, 10: large bodies in 8, 48, 55.
- **Every event has a cause** (20 seeds): each cause is an earlier event or an input (epoch, setting, catastrophe). Every event has a reason, the oxygenic event is stamped with its dial, and Earth is never sterilised (deep refuges).
- **Catastrophes:** recoveries follow their catastrophe within 100 Myr. An added total event with no refuge sterilises every world alive by then, and some seeds get a second origin.
- **A cold, thin biosphere is slow:** Europa accumulates opportunity about 1200 times more slowly than Earth today. The design says thousands; this is from 3e-8 of Earth's production, 0.24x the generation rate and 2 of 7 habitat kinds. Europa stays at chemotrophy and dormancy.
- **Determinism and chronicle:** same seed, same history; another seed differs. Every life event is recorded, and the chronicle causes match the internal ones.
- **Golden:** `tests/golden/life-history.json` (new; Earth and Europa, three seeds each), harness engine `life-history`.

## Known limits
- **Physical states are inputs.** Oxygen levels, the ferrous ocean and the catastrophes come from the fixture. L2 couples them to life: oxygen from the balance, not by date. The large-bodies-late result therefore leans on the oxygen input at 3.9 Gyr. The ferrous ocean stays throughout; the better light metabolism wins, so it matters only before oxygenic photosynthesis.
- **Spread and transfer:** life spreads to every open habitat at once, and there is no transfer between worlds or shared origin yet (L2, with the system's impact history). Multiple origins and shadow biospheres are not modelled.
- **Capability effects:** dormancy has no effect yet, nitrogen fixing is not in the default set, and catastrophes do not yet come from the physical chronicle automatically (the input list stands in for it).
- **Opportunity scaling:** the 0.25 exponent on population is a pack setting chosen so small biospheres can innovate slowly; the pack cites clonal interference.
- **Rate tuning:** the complex-cells rate (0.0012) was tuned so the Earth outline holds in most seeds. That is a calibration to one data point, as the design asks, not a law.

## Integration points
- The L1c resolve (guilds, traits) reads `LifeHistory` capabilities and box B production.
- The UI's life timeline draws `points` and the events with their reasons and dials.
- The T track: catastrophes should come from the physical chronicle (`time.*`, galaxy hazards) once a system's history is assembled.
- Shared files touched:
  - `crates/sse-wasm` (engine `life-history`) and the harness lists;
  - `packs/base-life`;
  - `budget::closed_reason` is now `pub(crate)`.
