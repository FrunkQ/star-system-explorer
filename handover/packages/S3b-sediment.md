# S3b: the sediment budget (where wear's load really lies)

Why: on 21 Nov Alex saw Earth's globe say "96% granite, 4% basalt" for its bare land. Real land is about two
thirds sedimentary rock (Blatt and Jones 1975, GSA Bulletin 86, 1085; Hartmann and Moosdorf 2012, G3 13,
Q12004, doi:10.1029/2012GC004370). The live globe then had no wear at all (S16a part 2 has since put it
there). Checking the wear stage itself turned up three bugs in its sediment budget.

## The bugs (confirmed with a probe on Earth, 64 cells a face)
- `sediment_m` was every metre ever laid in a cell, summed over epochs, even if a later epoch cut it away
  again. It was 4 to 10 km on every land cell, so the whole land would have read as sediment.
- A river's whole load was dumped on the first sea cell it reached. These "deltas" rose kilometres above
  the sea, and most of what the wear stage reported as land was these towers.
- With the towers gone, nothing held the continents up: 100 Myr of stream power on the plate history's
  final heights, with no uplift, wore all of Earth's land below the sea. The plate history's heights are
  already the end of uplift against wear (S2 wears its belts down), so wearing them again for 100 Myr
  counted the wear twice.

## What changed (`wear.rs`)
- **Sediment that still lies there.** `sediment_m` is now what each cell has left lying: each epoch adds
  what was laid and takes away what was cut, oldest cover first. Creep strips the cover where slopes
  lower and lays it where they fill.
- **Shelves, not towers** (`shelf_out`). What reaches the sea fills the sea floor up to the shelf break,
  `wear.shelf_depth_m` 130 m below sea level (Kennett 1982), nearest cells first through the sea. Then it
  spreads on. A load that fills its whole sea stays at the mouth. Eroded and deposited volumes still
  balance exactly.
- **Steady state on plate worlds.** On a world whose plates move, the land is raised as fast as it is worn
  (Whipple and Tucker 1999). Rivers carry their load and lay it down, but continents keep the plate
  history's height and shape and rise only by what is laid on them. `wear()` takes `steady`, which is
  true when the plates moved.
- **Isostasy elsewhere.** On worlds without moving plates, the crust rebounds for what is worn off and
  sags under what is laid. This is Airy isostasy filtered by flexure at the grid's feature size,
  C = 1/(1 + D k⁴/(ρ_m g)) (Turcotte and Schubert 2014). The elastic thickness comes from today's heat
  flow and the 450 °C isotherm (McNutt 1984; Watts 2001). Earth comes out about 0.4 at two cells, and
  Mars about 0.003, which is barely compensated.

## Result on Earth (base pack fixtures)
| | Before | Now |
|---|---|---|
| Land cells kept from the plate run | towers, not the plate land | 4,335 of 4,328 (sediment adds a few) |
| Land cells with over 10 m of sediment | all | 29% |
| Bare land on the Survey globe (level-0 tiles) | 96% granite, 4% basalt on the live globe; all sediment in the wear stage | 70% crystalline, 28% sediment, 1% basalt |
| Sea cells with sediment | a few towers | 22%, as shelves |

## Checks
- `wear_tests::sediment_lies_where_it_was_left_and_shelves_stay_under_water`:
  - plate land is kept to within 2%;
  - no sea cell is built into land by sediment;
  - sediment covers between 15% and 70% of land (measured 29%).
- `sediment_balance_closes` and Mars's `an_old_dried_world_keeps_softened_valleys` still pass.
- Goldens `surface-wear` and `surface-cover` are re-blessed on purpose, for this reason.

## Known limits, next
- Earth's real cover is about two thirds sediment. The missing part is marine platform cover, laid when
  high seas flooded the continents (Ordovician and Cretaceous highstands). It needs a sea-level history
  (`WaterHistory` is steady) and a plate history that keeps past heights. This belongs with S9 strata.
- Volcanic rock on land is about 8% on Earth, but here it is 1%, only from plains. Arc volcanism (cells
  already carry `arc_myr`) and large igneous provinces should lay volcanic cover. A material code for
  andesite and felsic volcanics would help, since "basalt" is the only volcanic one.
- The plate history's continents are too high and too few: median land is 2.9 km (Earth's is about
  0.4 km) and land is 18% of the surface (Earth's is 29%). This is S2b's (continental freeboard and
  hypsometry, from crust thickness and isostasy).
- Within an epoch, a cell may refill its own cut first (room is measured on the cut surface). This
  stands in for the balance inside a 100 Myr epoch. Without it, one implicit step wears every drainage
  down to its outlet, because K A^m dt / L is about 200 at these cell sizes. It is in the fudge register;
  the real fix is sub-epoch steps with K calibrated to the grid.
