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

## Goldens

Changed on purpose: `tests/golden/envelope.json`, `galaxy-events.json`, and the core contract lines that read
the home galaxy's arms (galaxy chart crests and `arm_names`, sky and glow). Reason: the home galaxy's arms are
now Reid 2019's measured arms rather than a generic two-armed spiral.

## Next

- The galaxy charts (GH8) and renderer (GH9) threads can label crests by `arm_names`.
- The far side of the ring is extrapolation; a Gaia or HI map of the far side would replace the run-on.
