# GH Local Arm: the home galaxy's arms from measured positions

Owner: "Build galaxy arm fix and atmosphere advice". Alex's yes, 25 Nov: "add the Local Arm from measured
positions (Reid 2019)".

## What landed

- `EnvelopeParams.measured` (`crates/sse-cosmos/src/envelope.rs`): a list of arms, each a log spiral with a kink
  (Reid et al. 2019, ApJ 885, 131, Table 2: β range, β_kink, R_kink, ψ before and after the kink, width at the
  kink), with `r0_kpc` (8.15), `width_slope` (42 pc per kpc, Reid 2014) and `run_on_deg` (30).
- One general mechanism: any galaxy's pack can list measured arms. Only the home galaxy (halo
  `milky-way-like`) keeps them; every other galaxy clears the list and uses the generic m-armed spiral.
- Arm strength is a sum of Gaussians across each arm's width, turned into a boost over the ring mean
  (`(1 + B·S)/(1 + B·⟨S⟩)`), so each ring's total mass is unchanged (test `measured_arms_keep_each_rings_total`).
  The ring mean comes from a table built once (`MeasuredArms::prepare`, 120 radii × 360 azimuths).
- Frame: β is 0 toward the Sun, and the Sun's galactic azimuth at the defined date is the star grid angle
  (`sse_cosmos::stars::grid_angle`), so the Sun sits inside the Local Arm. Radii scale by `r0 / sun_radius`
  for a galaxy of a different size. The pattern turns rigidly at Ω_p about the present.
- Each arm runs on 30° past its measured azimuths with a Gaussian fade. The far side of the disc, beyond the
  parallaxes, is left smooth rather than guessed. A 60° run-on was tried: it washed out the centre-over-anticentre
  contrast in Ks that the glow test checks, so 30° stays. Fudge register row "envelope.rs arms".
- `arm_cos` is the one arm-crossing test (events read it, not the raw contrast).
- Galaxy chart: crests come from `Envelope::arm_crests`, and the chart gains `arm_names` beside `arms`
  ("Norma", "Local", ... for the home galaxy; "arm 1", ... otherwise).
- Glow test: the plane is now 0.96 mag brighter than the pole (0.8 before; the measured sky has 2 to 3, Leinert
  et al. 1998). The margin goes from 0.5 to 0.9, not the 1.0 hoped for, because the far side is left smooth.

- Glow test's dust check: Ks's centre-over-anticentre gain must beat V's (was V + 0.5 mag). The
  Sagittarius-Carina arm now lies about 1.5 kpc inward, inside V's reach, so V gains about 0.96 and Ks 1.03. Ks
  seeing so little further is the inner disc and bulge being faint in Ks; worth a look by the light thread.

- STILL FAILING (25 Nov 23:10): the next check in the same glow test, the midplane lane deeper in V than in Ks.
  With the measured arms V's lane is 0.43 mag and Ks's 0.86: the Local and Sagittarius arms' young stars fill the V
  midplane at low latitude. Not a tuning fix. Look at the young-star scale height and the dust layer under the
  arms (the arm boost may apply to dust and stars alike), with the light thread. The PR is core lane.

## 26 Nov: the dust's cause (thread "Fix Local Arm dust physics")

Alex: "the failing dust check is right and the model is wrong ... do not tune"; then "Dust cause first, then
young stars"; then (in the design thread) yes to sending the mid-disc metals to the conserved budget chain.

What the failing lane check measures. At the test's nside 8 the "b = 0" pixel is centred at l = 5.6°, b = 0
and the "b = -8°" pixel at l = 0, b = -4.8° (it passes over the nucleus at z = -810 pc). Both sight lines were
traced shell by shell (a debug print in `sse_sky::glow`, not committed).

- Young stars' scale height and arms (the first guess) do not fix it: put at the gas layer and in the arms,
  they brighten V's midplane, so V's lane falls (offline from age-split traces: V 0.42 to 0.07 at 60 pc). Ks's
  lane stays deeper whatever their height.
- The real gap is midplane dust. In-plane A_V toward the centre was 64 mag to 1 kpc from it (Ks loses 7.5 mag;
  measured about 2.5, from memory). The gas alone gives about 21 mag (close to measured); the dust rule's
  10^[Fe/H] tripled it, because the inner disc was a closed box at 5% gas with [Fe/H] +0.5 to +0.8.
- Cause one, fixed here: the home galaxy is barred but its bar only ever made a starburst. A bar sweeps its
  gas into the nucleus (`rings.rs`, `RingParams::bar_*`, fudge register "rings.rs bar sweep"): the gas between
  0.12 and 1.15 disc scale lengths falls in once the disc is bar-unstable (`Galaxy::bar_from_gyr`, the first
  unstable history point read forward, so edits stay causal). A_V toward the centre 64 to 46.
- Cause two, handed on: the mid disc's gas runs +0.3 to +0.46 at 3.6 to 6.6 kpc (about 30 of the 46 mag).
  The design thread "Design the conserved budget chain" wrote the check set the chemistry must match
  (`handover/design/SSE4-conserved-chain-checks.md`, PR 604; A1 gas at the Sun 0.00 within 0.05, A10 extinction
  to the centre 30 within 5); a builder retunes the rings against it (CC3). This PR merges after that, when
  the glow check passes unchanged.
- The bulge's stars were missing from the star field. Now: a bulge ledger in the galaxy (`born_hist`,
  `bulge_joins`, `bulge_bursts`, `disc_moves`), the rings give up the disc stars mergers moved (each star counted
  once), a Hernquist bulge sized by DM4's own rule (`GalaxyEnvelope::bulge_pc3`), its own count tree layer and
  keys (`Layer::Bulge`, `f<frame>b<slot>`, `StarSystem::in_bulge`), its own glow light mix. Drawn stars plus glow
  still add up (the glow's double-count test passes). Glow now: V lane 0.80, Ks 2.13 (still fails, as expected).
- The bar's swept zone made a thin complex-life band at 6 Gyr (green test, Lineweaver 2004). Physics, not the
  test: `Green::gas_kept` is false where a bar has swept the ring's gas (`Ring::swept_from_gyr`), so no new
  systems count as green there. The test passes again unchanged.
- New tests: `a_bar_sweeps_its_gas_into_the_nucleus`, `the_bulge_and_the_disc_count_each_star_once`
  (g4_tests). Both seen red with the fix removed (sweep off; rings not giving up moved stars).
- Next, its own package: stars' scale height by age (`packages/GH-age-heights.md`).

## Goldens

Changed on purpose: `tests/golden/envelope.json`, `galaxy-events.json`, and the core contract lines that read
the home galaxy's arms (galaxy chart crests and `arm_names`, sky and glow). Reason: the home galaxy's arms are
now Reid 2019's measured arms rather than a generic two-armed spiral.

26 Nov, changed on purpose again, for every barred galaxy and every galaxy with a bulge: `galaxy.json`
(rings' gas and stars), `envelope.json` (gas, bulge density, `gas_kept`), `stars`/`g5` goldens (bulge stars
drawn, disc stars fewer by what mergers moved), `galaxy-events.json`, and the core contract lines that read
them. Reasons: a bar sweeps its gas into the nucleus; the bulge's stars are drawn and glow; each star is
counted once. To re-bless after the chemistry retune lands.

## Next

- The galaxy charts (GH8) and renderer (GH9) threads can label crests by `arm_names`.
- The far side of the ring is extrapolation; a Gaia or HI map of the far side would replace the run-on.
