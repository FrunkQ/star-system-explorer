# L1d: boxes I and J: signatures, data views, sifters, the biosphere view and life timeline

Status: done (19 Nov 2026). Kernel track, PR into `main`. Design: life design sections 9, 10 and 13, section 14 row L1d.

## What was built
**`sse-life::signatures` (box I).** True state, surviving evidence and detection are kept apart.
- Input `Observables`: the star's temperature, pressure, abiotic O2 and CH4, an ice shell, clouds, land, tilt, recycling and erosion times, the world's age. These are fixture data until the planet layer carries atmospheres.
- **Gases:** life's O2 = 0.21 x oxygenic production / 1e17 g C/yr (capped). Life's CH4 = 4.5e-15 ppm per g C/yr x (methanogen production + 0.5% of all production decaying as methane), or x 100 lifetime and 50% decay in anoxic air. Only open-air habitats count; an ice shell seals them off.
- **Red edge:** where land plant producers show through the clouds over at least 5% of the disc. The edge sits at 700 nm x sqrt(5772 / Teff).
- **Confidence:** a posterior from a 0.5 prior and stated likelihood ratios:

  | Signal | Ratio |
  |---|---|
  | O2 with CH4 | 20 |
  | O2 alone | 3 |
  | O2 alone round a star under 3900 K (water-loss trap) | 1.5 |
  | CH4 alone | 2 |
  | red edge | 10 |
  | nothing seen where a scan could see | 0.5 |
  | nothing seen through ice, or with no air | 1 |

  A candidate is any signal that lifts the confidence above the prior.
- **Tiers:**
  - obvious: colour (the producer pigment, where the edge shows), seasonal change with tilt over 10 degrees;
  - scanned: gases, edge, signals with reasons, candidate, confidence, what the scan is blind to;
  - surveyed: summary, up to six guild cards by production, the hazard to visitors, biomass;
  - past: isotopes, stromatolites, banded iron and fossils, with the surviving fraction exp(-t / recycling) x exp(-t / erosion) as abundant, scattered, rare or erased.
- **Facts and data views:** `facts()` writes `life.obvious.*`, `life.signature.*`, `life.survey.*`, `life.past.*`, `life.evidence` and the truth under `gm.life.*`. `disclosure()` gives the tier for each pattern; a campaign's observe pack adds them to `disclosure.fields`. The same map is in the base-life pack under `life_signatures.disclosure`.

**`sse-life::sift` (box J).**
- **`epochs`:** boundaries at origin, each capability gained or lost (named by the newest capability: "The age of green mats"), sterilisation, and catastrophes that cut diversity by 30% or more ("The aftermath of ...").
  - Each epoch is scored on persistence, reach (diversity against its peak), capacity change (log production) and reversibility, weighted 0.4, 0.2, 0.3, 0.1. It is major at 0.25 or more.
  - Each epoch carries a `why` in three lenses (physics, ecology, history).
- **`hooks`:**
  - `one_short_of_animals`;
  - `doomed` (habitability ends within 500 Myr; the time comes from a look-ahead's `oceans_lost`);
  - `last_refuge` (one living habitat, a deep one);
  - `false_positive`;
  - `hidden_life`;
  - `dead_world_record`.
  - Each has the same three-lens `why`.
- **`view`:** a `LifeView` is what the UI's biosphere view and life timeline draw. It holds the timeline points, events with reasons and dials, named epochs, hooks, signatures, relation counts per lens, guild cards, traits and dials.

**Pack:** `provides.other.life_signatures` (every number with seven sources) and `life_sift` in `packs/base-life`.

**Fixtures** (`scenario.rs`):
- `europa_biosphere()`;
- `observables(name)`;
- `signature_cases()`: Earth today, Archean Earth, Europa, a red-dwarf desert and a dead Earth;
- `signatures_golden()`.

## Checks (`cargo test -p sse-life signatures`, 7 tests)
- **Earth today:**
  - O2 0.21, CH4 1.81 ppm (observed 1.9; the 4.5e-15 factor is calibrated to this one point);
  - signals O2 with CH4, and a red edge at 700 nm;
  - confidence 0.995;
  - green and seasonal;
  - surveyed summary and guild cards;
  - fossils abundant, banded iron scattered, the first isotopes rare.
- **Archean Earth:** no O2, CH4 675 ppm (Kasting: 100 to 1000), a CH4-only candidate at 0.67, no colour from afar.
- **Scripted false positive** (lifeless red-dwarf desert with 10% abiotic O2):
  - a candidate at 0.6 confidence, whose reason names water loss;
  - a survey finds no biosphere;
  - hook `false_positive`.
- **Scripted false negative** (Europa, alive under 20 km of ice):
  - no candidate, and the confidence stays at the 0.5 prior (a blind scan says nothing either way);
  - `blind` names the ice shell;
  - a survey finds the ecosystem;
  - hook `hidden_life`.
  - An Earth-like world where nothing shows lowers the confidence below the prior.
- **Data views** (tarn-observe views plus `disclosure()`, through `sse_observe::build_scene_through`):
  - long-range shows the colour, not the signatures;
  - scanned shows candidate and confidence, not the survey;
  - survey shows the summary;
  - no player lens shows `gm.life.*`, and the GM sees it;
  - the false positive's candidate shows in the scanned view.
- **Sifters** (Earth, seed a):
  - ten contiguous epochs from "The barren age" to "The age of shells and bones", ending now;
  - "The age of green mats" from 1780 to 4310 Myr is major;
  - a lost capability reopens "The age of rock-eaters";
  - every epoch has its reasons.
  - Hooks: a world without large bodies at 2% O2 is `one_short_of_animals`; oceans lost at 4800 Myr is `doomed`; the dead Earth is `dead_world_record`.
- **Pack:** the values equal the defaults, and the disclosure map equals `disclosure()`.
- **Golden:** `tests/golden/life-signatures.json` (new), harness engine `life-signatures`.

## Known limits
- **Observables are inputs.** Atmospheres, clouds and abiotic gases come from fixtures until the planet layer (P4 world stacks, L2 feedback) supplies them. L2 couples O2 and CH4 to the balance; until then, life's O2 is a production ratio.
- **Gas amounts** are sketches calibrated to Earth today, not photochemistry. There are no spectra and no telescope model (distance, aperture); detection is a floor per tier.
- **Technosignatures** (night-side light) are not modelled: civilisations are a sketch (box K).
- **Epoch naming** follows the newest capability. The persistence score uses the world's age so far, so very short epochs right after a cluster of innovations are minor.
- **No kernel op yet.** The UI can call it through the engines, or the next package adds `life_view { body }` once adopted systems carry habitat graphs.

## Integration points
- The UI's biosphere view and life timeline read `LifeView`, and its scanned panels read the `life.*` facts through data views.
- Campaign packs add `disclosure()`'s patterns to their observe disclosure (or take them from `base-life`).
- The T track supplies `habitable_until_myr` from a look-ahead's `oceans_lost`.
- Shared files touched:
  - `crates/sse-wasm` (engine `life-signatures`) and the harness lists;
  - `packs/base-life`;
  - `crates/sse-life/Cargo.toml` (dev-dependency `sse-observe`).
