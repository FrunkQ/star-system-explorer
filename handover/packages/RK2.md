# RK2: base-tech additions from the Atomic Rockets plan (track RK)

This package carries out the Atomic Rockets plan (`handover/design/SSE4-atomic-rockets-plan.md`) sections 3, 7 and 9, under question 3, which Alex accepted on 23 Nov 2026. It is built by `scripts/build-base-tech.py` into `packs/base-tech`. Sources, with quotes and gaps, are in `RK2-data/RK2-sources.md`. The Atomic Rockets site was used only as a map to the primary sources.

## What it adds

- **The heat figure (`working_k`, under limit 7 "materials").** This is the hottest a working part can run.
  - `built` marks: Inconel 718's service limit and tungsten heat shields at 2800 K. Both are read from the sheet's new RK1 part 3 materials.
  - `ceiling` marks with `for`: UO2 fuel melting at 3120 K and tungsten melting at 3683 K, also read from the sheet.
  - A hot-gas rocket's exhaust goes as the root of temperature over gas weight, so this figure caps nuclear thermal engines, nozzles, radiators and heat shields.
- **Limit 11, "Beams spread".** It has one figure, `spread_over_diffraction` (lower is better), with a ceiling of 1 under the law of diffraction (Airy 1835).
  - It is the same Equation for beamed power, laser sails, link rates and how sharply sensors and telescopes see.
  - The validator now expects 11 limits.
- **The size floor.** An advance can carry `least_size` as `{v, unit, what, src}`.
  - The fission reactor's floor is 53.9 kg of fissile metal: the bare-sphere critical mass of 93 to 94% U-235 metal (Gwin and Mee 1959, Y-1248).
  - Every advance in the `least-size` family must state its floor.
- **Flown advances.** Each carries a `flown` field, `{what, src, also?}`, and must earn the `built` mark.
  - `hall-thruster`: SPT, run in space since 1972. The SPT-100 gives 1600 s (Brophy 1992; Sankovic 1993).
  - `arcjet`: Telstar 401, 1993, 500 s (Curran and Byers 1994).
  - `ion-thruster`: NSTAR on Deep Space 1, 3100 s (Sovey 2001; Rayman 1999).
  - `rtg`: GPHS-RTG, 292 W for 55.9 kg (Schmidt 2009). The value comes from the same constants as the GPHS-RTG entry in REAL_CRAFT.
  - `light-sail` now cites IKAROS (2010).
  - New `built` marks: `exhaust_km_s` gains NSTAR, and `plant_w_per_kg` gains the GPHS-RTG.

## Kernel change (FT2's reader, `crates/sse-kernel/src/tech.rs`)

- **Ceilings for one fuel.** A ceiling marked `for` one fuel or material (`Mark.for_`) is no longer applied to the whole figure. Before this, any drive that needed more than hydrogen-oxygen's 4.9 km/s read as fictional, even though that ceiling is only hydrogen-oxygen's.
- **Matching part first.** Among the advances that meet a need, one made for the same `part` comes first, then the lowest level, then the id. The electric thrusters have the part `electric drive` and the RTG has `power plant`, so a "drive" need of 4 km/s still reads the chemical rocket. This is right because an electric drive also needs a power plant (RK4).
- **Test:** `ceilings_for_one_fuel_and_electric_drives`.

## Not done

- **Solar arrays as an advance.** Juno's arrays give power, but no array mass has been read.
- **The size floor for fusion ignition and Orion pulse units.** These go with RK4 and RK6.
- **A `built` mark for beam spread.** This needs a measured M-squared for a real laser or link.
