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

## Part 2: electron belts (`read_field` gains `belts`)

Source paper: Mauk and Fox 2010 (doi:10.1029/2010JA015660), supplied by Alex on 22 Nov.

- Atoms (`crates/sse-atoms/src/belts.rs`):
  - `sse.belts.kp_limit`: the relativistic, differential Kennel-Petschek limit. The whistler growth rate is
    Summers et al. 2009's form (Mauk and Fox appendix A, eqs. A1 to A9), integrated numerically along each
    cyclotron resonance curve for the spectrum C E^-index / (1 + (E/E0)^break) sin^(2S)α (their eq. 7,
    kT = 0). It solves D R_p γ = 3 V_g (their eq. 6) for C. The cap is the lowest C_K over strictly
    parallel resonant energies 0.1 to 1 MeV, in a dense plasma (ω_pe = 300 Ω), where the limit no longer
    depends on the cold density and scales exactly as B / (D R_p). Electron mass, charge and ε₀ are
    inputs, read from the data sheet by the kernel (`constant.m_e`, `constant.e`, `constant.eps0`).
  - `sse.belts.electron_range`: Katz and Penfold 1952 range, inverted to the energy that crosses a shield.
- `sse-field::belts` (`Belts` on `FieldRead`):
  - Zone from the top of the air (100 scale heights), the rings' outer edge (`Body.rings_outer_m`) or the
    surface, out to the last closed drift shell at 0.7 of the stand-off.
  - Wave path D = L, or min(L, 3) radii when moons load the field with plasma (`plasma_source_kg_s` > 0:
    Mauk and Fox use D = 3 at Jupiter after Summers et al.). Such a "spun" belt keeps a hard spectrum
    (break 2,500 keV, index 1.75, as Garrett's Jupiter fit); a wind-fed one breaks at 1,500 keV, index 6
    (Earth and Uranus fits).
  - Filled to 0.6 of the cap at and beyond the source shell, and kept at the source shell's value inward.
    The source shell is half the stand-off (wind-fed) or the innermost moon in the zone (spun).
  - Moons sweep gaps of their own width (`MoonOrbit.radius_m`). A moon's own dose uses the unswept belt.
  - Per shell (16, log spaced): field, the limit C_K and the belt's C, intensity at 1 MeV, flux above
    1 MeV, and dose (Sv/h) behind 0.3, 1 and 3 g/cm². The same is given at each `orbits_m` and each moon.
  - Without the sheet's constants (`Settings.electron` is None) no belts are computed, and a why line says so.

### Checks

- `sse-atoms` `belts_reach_the_kennel_petschek_limit`: Earth L 5 storm spectrum (Mauk and Fox Table 1) at
  0.4 to 0.8 of the limit (theirs 0.60; ours 0.55); the cap scales as B/D; Earth's storm cap within 10 % of
  3.8e6; Neptune near the limit at 0.1 MeV and under a tenth of it at 1 MeV (theirs: 30 times below).
  Katz-Penfold range at 1 MeV 0.412 g/cm².
- `sse-field` `belts_where_field_lines_close`: Earth's belts from 1.13 to 6.9 radii, set by the air;
  intensity at 1 MeV near L 5 is 3.0e3 per cm² s sr keV (Mauk and Fox Figure 5: 2.4e3 for their storm);
  doses fall with each shield; none without the sheet's constants, none for Mars. Saturn's start at the
  rings (2.27 radii).
- `jupiter_belts_and_moons`: Io sweeps a gap; Europa's orbit takes 22 times Callisto's dose behind 1 g/cm²
  (6.5 against 0.30 Sv/h).
- Contract: Saturn with rings and Enceladus added to the `read_field` case; golden changed on purpose (new
  `belts` blocks, and the pack's two new atoms move the stack hash).

### Gaps (fudge F40)

- The design's check "dose at Europa's orbit hundreds of times Callisto's" is not met: 22 times. The cap
  falls as B (L^-3) outward when D is fixed at 3; Garrett et al.'s Galileo fit falls about 26 times from 8
  to 16 radii, steeper than the cap, so real belts fill less of it outward.
- "Earth has two belts and a slot" is not met yet: no proton belt (cosmic-ray neutron decay) and no slot
  from hiss inside the plasmasphere. The inner zone keeps the L 5 electron intensity, far above the real
  inner zone at 1 MeV.
- Neptune (quiet, no injections) and Saturn (Enceladus gas and dust) get a belt at the limit; real ones sit
  20 to 30 times lower at 1 MeV.
- Doses are for a belt filled after strong injections (often ten times quiet levels): Earth's geostationary
  orbit gets 0.29 Sv/h behind 1 g/cm². No bremsstrahlung; constant stopping power 1.85 MeV cm²/g.

## Next (part 2b and readers)

- Proton belt from cosmic-ray albedo neutron decay, its outer edge where protons stop being adiabatic
  (field-line curvature scattering; the threshold needs a citable value), and the slot from plasmaspheric
  hiss. Losses to gas and dust (Saturn), and quiet magnetospheres that never fill (Neptune).

- Sources that set how full each belt gets (wind injection, plasma tori from eruptive moons), and the plasma
  that inflates Jupiter's bubble to 60 to 90 radii.
- The star's particle storms from O10's events through the same cut-off.
- Readers: life's `radiation_dose_msv_yr` (kernel resolve thread) and NAV's belt hazard (travel screens
  thread) read `shield` and `belts`, once MF2 publishes each world's field.

## MF14a fixture: the Solar System's fields (for the renderer)

`read_field` takes `fixture: "solar-system"` in place of `bodies`. It builds
each magnetised planet from three files, typing nothing:

- `packs/base-sol/pack.json`: radius and orbit.
- `tests/fixtures/magnetic/measured-fields.json` (MF0): moment, tilt,
  offset, reference radius, grade, model and source.
- `crates/sse-field/data/sol-field.json`: the solar wind at 1 au (ECSS-E-ST-10-04C
  Table 8-2), each dipole's sense (Alken 2021, Anderson 2011, Connerney 1993),
  Earth's air and gravity, Io's plasma (Bagenal and Dols 2020), Saturn's ring
  edge and the moons that open belt gaps (NSSDCA).

The proton mass that turns the wind's count into a density, and the electron's
constants for the belts, come from the loaded data sheet. Without
`base-reference` loaded the op refuses with `bad_field` (the contract case
shows that path; `read_field_solar_system_fixture` in the kernel tests reads
the real one).

Each body is the usual FieldRead plus `name` and `field`:
`reference_radius_m`, `g10_t`, `g11_t`, `h11_t` (Schmidt semi-normalised, T),
`degree` (1), `tilt_deg`, `tilt_longitude_deg` (0 for now), `offset_radii`,
`polarity` (+1 moment along spin north, −1 south), `grade`, `model`, `source`,
`polarity_source`, `stand_ins`. g10 = polarity × B0 cos(tilt), g11 = polarity ×
B0 sin(tilt), with B0 = μ0 M / (4π a³). Dipole-only, pole longitude and offset
direction are stand-ins (fudge F41). Code: `crates/sse-field/src/sol.rs`.
