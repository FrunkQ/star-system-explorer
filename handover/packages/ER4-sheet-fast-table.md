# ER4: the sheet's fast table (not started)

Design: `handover/design/SSE4-edits-reach-design.md`, ER4 row. Follows the stray-numbers PR (SN2 to SN5, #664).
This file holds what earlier packages found for ER4, so whoever builds it starts with it.

## Start with these

1. **Fixed nominal units, so G and c can move (from ER1b, `handover/packages/ER1b-reach-gate.md`).** The sheet
   makes the solar, Earth and Jupiter mass units from GM / G, and the light year from c. A unit never moves (Alex,
   26 Nov), so today a nudged or edited G or c is refused at load, and ER1b's reach gate cannot nudge them. Give the
   mass units and the light year fixed nominal values, as the IAU nominal values are meant for (IAU 2015 B3 for the
   solar and planetary masses' GM; the Julian light year from the defined c), and keep the GMs as the measured
   quantities. Then G and c can be nudged in the reach gate, and edited by a user, without moving a unit.
2. **The door outside a call (from SN, #664).** `sse_physics::constant(id)` and `now()` give nothing outside a kernel
   call. The first two converted reads (`sse-life` world_life.rs `rock_conductivity()`, `sse-cosmos` haloes.rs
   `gm_sun()`) fall back to the shipped sheet (`Reference::base()`, cached once) so tests and tools still run. The
   fast table should give one shared form of this (a read in force, else the shipped sheet) so each crate stops
   writing its own.
