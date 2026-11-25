# O3b: giants' dusty shells and young stars' jets (gap R26)

Light track, 24 Nov 2026. Asked by the thread "Build star glow and flares" (gap R26 of the third V3
audit). The light track owns the physics and the star glow thread owns the drawing. Pulsar and
black-hole jets stay with the exotic stars thread.

## What landed

- Three Equations in `packs/base-physics/formulas/outflow.json` (family `sse.outflow`):
  - `sse.outflow.giant_mass_loss`: Schröder and Cuntz 2005, doi 10.1086/491579.
  - `sse.outflow.dust_shell`: condensation radius, the dust's optical depth and the light it re-emits;
    Höfner and Olofsson 2018, doi 10.1007/s00159-017-0106-5.
  - `sse.outflow.young_jet`: a tenth of the accretion (Hartigan, Edwards and Ghandour 1995, doi
    10.1086/176343), the escape speed at launch (Bally 2016, doi 10.1146/annurev-astro-081915-023341),
    the jet power and its shock light.
- `sse_nebula::outflow`:
  - `dust_shell(seed, GiantInput, at)`: kind `dust-shell`; phase `blowing`, then `detached` once
    `ends_s` passes; gone when its inner edge passes a parsec. The light is a Thermal part for the dust.
  - `young_jet(seed, YoungStarInput, at)`: kind `jet`, while accretion lasts. `lobes.axis` is the spin
    axis, plus a field `jet {length_m, speed_m_s, opening_rad, mass_rate_kg_s, knots [{distance_m, age_s}]}`
    and the shock light as an Hα line.
- `read_nebulae` takes `giants` and `young` beside its other lists.
- Tests: `outflow_tests.rs` (2) and kernel `read_nebulae_gives_dust_shells_and_jets`. Fudge F57.
- Golden: the base-physics pack hash and one new contract case.

## Next

- The caller (kernel resolve) passing giants from the star's life line (its AGB or red giant stage)
  and young stars from formation stage 3 (the accretion rate), so systems publish them without asking.
- Pulsed shells (rings), binary spirals, Herbig-Haro bow shocks, and shock line ratios once O4 has
  metal lines.
