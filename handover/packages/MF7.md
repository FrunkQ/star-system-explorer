# MF7: cut-off, dose and belts (part 1: cosmic-ray cut-off and dose)

Track MF (magnetic fields), design `handover/design/SSE4-magnetic-fields-design.md` section 6.3. Light track.
Alex, 22 Nov: "the shielding is an effect - not a switch". So nothing here asks "is it shielded?". Every
world gets a cut-off from its own field and a dose from its own air column, and the "magnetically
unshielded" label can be read off the numbers.

## What was built

- `crates/sse-atoms/src/shield.rs`, four cited atoms (count 73 to 77):
  - `sse.shield.cutoff`: Størmer's vertical cut-off rigidity, R_c = (μ₀ M c / 16π r²) cos⁴λ (Smart and Shea 2005).
  - `sse.shield.space_share`: share of the free-space dose above the cut-off, (1 + R_c/R₀)^−γ (PDG 2024 index).
  - `sse.shield.ground_share`: the Dorman function 1 − exp(−α R_c^−k) for the ground under thick air
    (Clem and Dorman 2000).
  - `sse.shield.air_dose`: two-part absorption by the air column (a nucleonic part at 140 g/cm², a muon
    part at 600 g/cm²), moving from the space share to the ground share as the nucleonic part is absorbed;
    the body blocks the sky below the horizon.
- `sse-field`: `read` now returns `shield` (when the body has no air, or has air and a surface gravity):
  - the free-space dose (body's `free_space_dose_msv_day`, else the settings' 1.84 mSv a day measured by MSL);
  - the air column (pressure over gravity);
  - the cut-off and ground dose at magnetic latitudes 0 to 90 in 15° steps;
  - the area-averaged ground dose;
  - for each of the body's `orbits_m`, the dose at the equator, over the pole and averaged.
  The own field cuts off only inside its own stand-off; a moon inside its host's bubble takes the host's
  equatorial cut-off at its orbit as well. A body with air but no gravity gets a why line and no dose.
- New body inputs (all optional): `surface_gravity_m_s2`, `free_space_dose_msv_day`, `orbits_m`. New
  settings: the free-space dose and each atom's parameters.
- The contract's Earth case for `read_field` now carries air, gravity and one low orbit, so the golden
  `read_field` line gained the `shield` block. The stack hash changed because base-physics gained the atoms.
  Both on purpose.

## Checks (crates/sse-field/src/tests.rs, crates/sse-atoms/src/tests.rs)

| Check | Computed | Measured |
|---|---|---|
| Earth equatorial cut-off | 14.8 GV | 14.9 GV (Smart and Shea 2005) |
| Earth sea level, area average | 0.35 mSv a year | 0.39 (UNSCEAR 2008) |
| Mars surface | 0.82 mSv a day | 0.64 (Hassler et al. 2014, Curiosity RAD) |
| Moon surface | 0.92 mSv a day | 1.37 (Zhang et al. 2020, Chang'e 4; includes albedo neutrons) |
| Low Earth orbit, 400 km, averaged | 0.26 mSv a day | about 0.2 from cosmic rays on the ISS (belts add more) |
| Europa, in Jupiter's bubble | host cut-off 25 GV, 4.4 mSv a year from cosmic rays | belts dominate there (part 2) |

Earth's muon share is fitted to the sea-level value, so Earth's sea level is not an independent check;
Mars and the Moon are.

## Stand-ins (docs/fudges.md row "sse.shield.*")

One free-space dose for every star and galaxy place; a centred untilted dipole; the Dorman function's
sea-level parameters at every depth, recalled from memory and to verify; no Pfotzer peak; the muon share
fitted to Earth.

## Next (part 2 and readers)

- Belts: `sse.belts.zone` (closed field lines clear of air, rings and the bubble), `sse.belts.intensity`
  (the Kennel and Petschek limit), sources (cosmic-ray neutron splash, wind injection, plasma tori from
  eruptive moons), gaps swept by moons and rings; dose behind three shield thicknesses. This also brings
  the plasma that inflates Jupiter's bubble to 60 to 90 radii.
- The star's particle storms from O10's events through the same cut-off.
- Readers: life's `radiation_dose_msv_yr` (kernel resolve thread) and NAV's belt hazard (travel screens
  thread) read `shield`, once MF2 publishes each world's field.
