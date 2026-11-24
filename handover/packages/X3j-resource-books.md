# X3j: resources kept on the books, moved only by transfer (TG11 kept half)

Record-logic design 2.4 (accepted 21 Nov). MP8b and MP8c (PRs 362 and 386) count a world's natural kilograms of each
resource kind. This package adds the books on top.

## What changed (kernel)

- **Two kept kinds per resource kind**, made by the kernel for every kind in `provides.other.resources`:
  - `resource.<kind>.extracted`: what has been taken from a place's natural amount, in kg;
  - `resource.<kind>.stock`: what has been put or made there, in kg.

  They are numbers in kg, 0 or more, at tier "surveyed". Their writers are `gm` and `transfer`. A pack that keeps the
  same id itself keeps its own. Any address can hold them: a body, a ship, a station, a settlement's store.
- **Held = natural - extracted + stock.** `read_inventory` adds `natural_kg`, `extracted_kg`, `stock_kg` and
  `held_kg` to each kind, read from the record at `at_s` (left out: the latest).
- **`transfer {kind, from, to, kg, writer?, reason?, at_s}`** (new op, Light) moves goods in one decision:
  - The source gives its stock first, then its natural amount, which raises its `extracted`. The destination's stock
    rises by the same amount.
  - It is refused (`not_enough`) when the source holds less. A made good nobody stocked cannot appear from nothing.
  - A world's natural amount is read off its composition, as `read_inventory` does. Anything else has a natural
    amount of 0.
  - The answer gives both places' natural, extracted, stock and held amounts after the move, with the change ids.
- The GM can still set stock or extracted by hand with `set_kept`, for example a port stocked at the start of a
  campaign.

## Checks

`sol_tests::transfers_conserve_and_mining_draws_down_a_rock`:
- Mining 5 kt of iron raises the rock's `extracted` and the ship's `stock`, and the rock holds 5 kt less.
- Earlier dates still read the old amounts.
- A ship cannot unload more than it holds.
- Ore put back on the rock becomes stock there, and the books still balance.
- Food nobody stocked cannot be moved. Once the GM stocks it, it moves.
- Bad asks are refused in words: the same place twice, a negative amount, an unknown kind.

Golden changed on purpose (`contract.jsonl`): the hello op list and the bad-request message gain `transfer`, and one
new contract case shows the refusal for a kind the stack does not declare. Every other line is unchanged.

## Next (X3k)

- The mining step: a ship's program moves ore at a computed rate, by `transfer`.
- `resource.*` labels derived from held amounts, so a mined-out rock loses its label by itself (TG8 part 2).
- Pack-declared making and consuming effects, such as a refinery's conversion. Each one is named, so every change in
  a total has a visible cause.
