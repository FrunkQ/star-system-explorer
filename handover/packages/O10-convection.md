# O10 surface convection (light track): granules and supergranules on any churning surface

Asked for by Alex (21 Nov) after the exogeology research thread: stellar surface cells for magnetics, flaring and
visuals, as one process driven by the body's own temperature, gravity and size ("processes, not classes").

## What was built
- Five atoms `sse.convection.*` in `sse-atoms::convection`: granule size (a multiple of the pressure scale height,
  which is the existing `sse.atmosphere.scale_height` block read at the photosphere), convective speed (flux
  σT⁴ carried as ρv³ with ρ ∝ gμ/T, so v ∝ T^(5/3) g^(−1/3), normalised to the Sun, capped at the sound speed),
  cell lifetime (k × size / speed), supergranule size (ratio to granules) and supergranule lifetime (size over a
  share of the granule speed). Atom count 64 → 69.
- `sse_activity::surface_convection(star, settings)` and the field `convection` on every `Activity`:
  surface gravity, scale height, granule size, speed, sound speed, granule life, supergranule size and life,
  granules across the diameter, network cells over the whole surface. `None` at or above the granulation boundary
  (`convective_below_k`, 6900 K, the same boundary activity already used). Remnants are not excluded: a cool white
  dwarf gets cells.
- A why line ("Its 5772 K surface boils in granules about 1346 km across ...") and the references on the activity
  read-out. Kernel op `read_activity` returns it with no request change.

## Evidence (`cargo test -p sse-activity surface_cells_from_temperature_and_gravity`)
- Sun: granules 1.35 Mm living 5.6 minutes (Nordlund, Stein and Asplund 2009: 1 to 2 Mm, minutes); supergranules
  34 Mm living 23 hours (Rieutord and Rincon 2010: about 30 Mm, one to two days); 5,400 network cells.
- Betelgeuse (18 M☉, 764 R☉, 3600 K): about 8 large cells over the whole star, living about 17 years, over 39
  granules across the diameter living about 25 days, churning at the sound speed; this matches the few giant cells
  of Freytag et al. 2002 and Chiavassa et al. 2010.
- EV Lac: cells 295 km, slower churning. Vega-like 9600 K: none (Gray and Nagel 1989).
- Cool white dwarf (0.6 M☉, 0.0125 R☉, 6000 K): cells 0.36 km. Tremblay et al. 2013 find about 1 km and much
  faster flows, so the speed rule fails at white dwarf gravity (logged in docs/fudges.md).

## For the renderer
Animate cells with `granule_size_m` and `granule_lifetime_s`, a coarser layer with `supergranule_size_m` and
`supergranule_lifetime_s` (its edges are the bright magnetic network), and `granules_across` to choose the
detail. Spots (`spot_coverage`) sit in that network. No simulation needed.

## Not yet
The activity cycle still comes from rotation by the Mittag 2023 branches (the Sun gives 6.1 years); deriving it
from the Rossby number with the dynamo is a later step. Spot sizes are not yet tied to supergranule size.
