# X3k: mining and resource labels (TG11 mining step, TG8 part 2)

Builds on X3j (resources on the books, `transfer`). Record-logic design 2.4 and the tags design (TG1).

## What changed

- **Labels read off held amounts.** A resource kind in `provides.other.resources.kinds` may carry
  `label: {tag, min_kg?, min_per_kg?}`. `read_inventory` adds `"label": {tag, earned}` to that kind and a sorted
  `"labels"` list to the body. `earned` uses held kg (natural - extracted + stock), so a mined-out rock loses its label
  by itself, and an earlier date still shows it. `min_per_kg` is held kg over the body's mass.
- **Tags may name `resources` as their reader** (`tags.rs` READERS), so a pack's `resource.*` tags pass the check.
- **New pack `sse.base-resources`** (`packs/base-resources/pack.json`, from `scripts/build-hooks-packs.py`; on the Packs
  screen as "Add resource labels"). Four kinds, each with its label; the thresholds are label choices, said so in the
  script, not physics:

  | Kind | Matter | Reach | Label | Earned at |
  |---|---|---|---|---|
  | metal | fe | whole | `resource.metal-rich` | 40% iron by mass |
  | water | h2o | surface | `resource.water` | 10^18 kg |
  | hydrogen | h | envelope | `resource.hydrogen` | 10^20 kg |
  | helium-3 | he3 | envelope | `resource.helium-3` | 10^12 kg |

- **New op `mine {ship, body, kind, hours, writer?, at_s}`** (Light): reads the ship's fact `mining.rate_kg_h` (kg an
  hour) from the record, takes rate x hours or what the body holds, whichever is less, and moves it by `transfer`
  with a reason naming the rate and hours. Refused in words: `no_mining_rate` when the ship has no rate fact,
  `not_enough` when the body is mined out. A ship's program calls it like any other op; the rate is a fact a template
  or the GM sets (or later a computed block for the drill).

## Checks

- `sol_tests::resource_labels_read_off_sol`: an authored iron rock is metal-rich; Earth is not, and has water; Venus
  has no water; the four giants have hydrogen and helium-3; the four rocky planets have no helium-3.
- `sol_tests::mining_out_a_rock_takes_its_label`: no rate is refused; one hour at a quarter of the rock's ice an hour
  takes a quarter; ten more hours take the rest and the label goes; the earlier date still has it; mining again is
  refused.

Golden changed on purpose (`contract.jsonl`): one appended case, `mine` refused for a ship with no rate fact. The op
list is masked since PR 454, so nothing else moves. (Since the golden split, PR 536, this case lives in `tests/contract/areas/traveller.json`, golden `tests/golden/areas/traveller.jsonl`.)

## Known gap

Mercury reads 23% iron, like every catalogue Sol body: their composition is one stand-in mix, so no catalogue planet
is metal-rich yet. The inventory thread owns the compositions; the test uses an authored iron rock instead.

## Next

- Pack-declared making and consuming effects (a refinery), each named so every change in a total has a cause.
- A computed mining rate from the drill's power and the rock's strength, in place of the authored fact.
