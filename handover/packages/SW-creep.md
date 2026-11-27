# Surface wear: creep tied to the process, and a solve that stays cheap

Two fixes found while chasing the Phobos Body globe hang (RW16a thread, 26 Nov 2026).

## 1. Implicit slope creep (no golden moves)

`wear.rs` relaxed slopes with explicit diffusion steps, whose count grows with κ dt / (cell size)². On an
11 km moon that was thousands of sweeps per epoch: Phobos's globe took 85 s natively, Deimos's 264 s.
Above 64 explicit steps an epoch is now solved backward-Euler in 32 steps by Jacobi-preconditioned
conjugate gradients (`relax_slopes`). Up to 64 steps the arithmetic is unchanged, so no golden moved.
Tests: `implicit_slopes_match_the_explicit_steps` (stated tolerance 2% RMS of the change, measured 0.9%;
one implicit step misses by 21%), `slope_relaxation_on_a_small_body_is_cheap` (red with explicit forced).

## 2. One creep rate per process

Slopes relaxed at one rate, 10 m²/yr, on every world: airless Vesta and Phobos lost their relief as fast
as a wet Earth would. New block `sse.surface.creep_diffusivity` (K = K_dry + K_wet w + K_air P) gives the
ground's own creep; its three rates are on the data sheet (`lunar-soil-mature-mare` hillslope
`creep_diffusivity`, Fassett and Thomson 2014, verified; `sediment` `creep_diffusivity` and
`air_creep_diffusivity`, fudge register) and reach the surface settings as sheet references
(`surface.creep`). Crater fading reads the same rates (they were pack numbers in `impacts` before), and
slope wear reads them times the pack's `wear.grid_factor` 200, a documented fudge for a cell kilometres
across, set so wet ground under 1 bar keeps the 10 m²/yr the wear was calibrated with. Airless ground now
relaxes about 10,000 times slower.

Test `a_dry_airless_world_keeps_its_relief`: Vesta's cell-scale roughness 0.97 kept over 4 Gyr (absolute
limit 0.9; the old single rate left 0.02, red). `creep_rates_come_from_the_sheet`.

The grid factor is a workaround, not a derivation (Alex, 26 Nov: do it right if possible; otherwise a simple
workaround, a fudge and a note to improve). Tried first: the ground's own rate alone, and the wear's sediment
cover and Mars's wet-era valleys vanished (three behaviour tests red), so the coarse grid needs an effective
rate. Looked for one from cell size: a cell relaxing on its hillslopes' own time gives (cell / hillslope
length)² ≈ 10⁶ on Earth, far too fast, because the grid's stream power already carries the channelled share
and the remainder is not modelled. So one factor stays, in the fudge register with what is wrong, why 200,
and what would replace it (a sub-grid hillslope model, or a rate scaling with cell size fitted to relief
spectra), with the same note beside the code.

### Goldens moved on purpose

- `tests/golden/contract.jsonl`: 6 lines (Earth and Moon surface surveys, Titan and Moon descents, Moon
  caves), from the new creep rates.
- `tests/golden/surface-wear.json`, `surface-cover.json`: Earth's and Mars's tiles (Mars's dry epochs now
  creep slower; Earth by the air term and rounding).
- `tests/golden/contract-reach.txt`: the new block listed as not following a changed G (it reads no
  constant).
- Atom count 177 to 178 (P1.md).
