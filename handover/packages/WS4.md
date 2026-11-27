# WS4: carrying a giant's cloud backwards along its winds

Design: `handover/design/SSE4-stirred-clouds-design.md` section 3.6, checks in section 9. Accepted by Alex on 27 Nov 2026. Needs WS2 and WS3. Status row: `handover/status/true-air.md`.

## What it does (`crates/sse-weather/src/stirred.rs`)

- **`Carrier`** holds everything that moves cloud over a span of dates:
  - the bands' and polar jet's winds (`giants::band_wind_at`, now shared with `giant_cell`);
  - every spinning storm alive in the span (the storm list read every 30 days), each drifting with the zonal wind at its centre, polar ones fixed;
  - every eddy (WS2), filed by band, size tier and half-day bin so a sum only looks as far as each tier reaches.
- **`wind`**: the storms' own winds, plus the background (bands plus eddies).
  - Eddies are round, from Ψ = −q exp((1 − x²)/2) with the slopes worked by hand on the sphere.
  - **A storm turns as one body.** Within its radius the background's stream function is replaced by the storm's drift (Ψ at its centre, plus −ω a² sin φ). It rises back to the full background by 1.5 radii, through a quintic smoothstep.
  - Because this is written on the stream function, the flow stays free of divergence.
- **`back` and `back_path`**: fourth-order Runge-Kutta on the unit sphere, backwards.
  - Step: a quarter of the fastest shear time, an eighth of the fastest eddy's turn, a sixteenth of the fastest storm's turn, at most a day. Polygons get a finer step near them.
  - `back_path` carries the map's Jacobian step by step, from each step's own neighbours a metre off, with the stretch kept as a power of two and the determinant as a running product.
- **`read_place`**: where today's face is read.
  - Within three radii of a storm, cloud moves with the storm's drift over its age.
  - Elsewhere it moves with its row's drift, except where a pole's polygon holds it (the polygon stands still).
- **`stirred_patch`** (weather want `stirred`, `lat`, `lon`, `half_km`, `n` up to 16, default 8). For each cell it gives two generations on a lattice τ apart (WS3's renewal time), offset band by band (stream `weather.giant_generations.v1`). Each generation carries its age, weight (the two sum to one, mean age τ), read place, `stretch_log2` and `area`.
- `stirred` is left out of the refusal's list of wants, as `flow` and `glint` are, so the core golden does not move.

## Where it departs from the design, and why

1. **The read rule is a switch, not a blend.** The design blended the storm's drift and the row's by the storm's Gaussian. Over a renewal the two can differ by a band's whole length: 16 m/s for 150 days is about 200,000 km. A blend then reads neither place, and a ring cyclone's core was read 2 to 20 radii away. The bands under a storm don't change along the row, so the storm's drift is right for all of it.
2. **Storms turn as one body, and shield their cores from the eddies.** These are two faults the design did not foresee.
   - The kernel's storm winds are already Moore and Saffman's ellipse, the shape the shear holds a vortex in. Adding the band's shear on top counts it twice: Jupiter's oval came out at aspect 1.73 against 1.44.
   - The eddies blew through storms that do not move with them, so polar cyclones lost their cores within 20 days.
   - With the rule, both test ovals keep their aspect exactly and the poles' cyclones keep their cores. Fudge row "A giant's cloud carried back".
3. **Eddies are round.** Their steady ellipse is what the shear does to a round vortex, so giving them that shape as well would count the shear twice.
4. **Stretch as a power of two, and area as a running product.** The stirring is chaotic: Jupiter's cloud at the mean age is stretched 2^30 to 2^54. Finite differences between whole paths separate across the planet, and the product's own determinant cancels away. Per-step Jacobians keep area within 1e-4 over two renewals.

## Findings for the owner (numbers unverified)

- **The renewal time decides whether the picture shows anything.**
  - With WS3's τ of 150 days on Jupiter, cloud of the mean age is stretched by 2^30 to 2^54. That is far finer than any pixel, so it reads as the local mean.
  - Only the youngest few per cent of the cloud keeps visible structure. Cloud 23 days old is already stretched 2^6 to 2^14.
  - The design's worked example assumed about 15 days. WS3's fall time uses the pack's 1 µm ammonia grains; 3 µm grains would give about 17 days.
  - A deck is also renewed by condensation in rising air, which WS3 does not have. This needs a design answer before WS5's picture is judged.
- **PR 646's polar storms stand still while the bands blow through them** (16 m/s at Jupiter's ring cyclones). Without eddies their cores stay held, but swing between 0.25 and 0.85 radii. A storm should drift with the wind at its centre, or the wind there should be calm.
- **Saturn has two ovals of about 8,000 km placed under 1,000 km apart** (60.3° and 61.2°) on day 1,000. They pass through each other, and neither keeps its shape.

## Checks (`stirred_tests.rs`, `sol_tests.rs`)

- `ws4_the_same_picture_whichever_way_the_date_is_reached`: the same carried places, to the bit, whichever way the date is reached.
- `ws4_carrying_keeps_area`: absolute, the determinant within 1 per cent over two renewals on Jupiter (worst 8e-5). It failed by 1e15 before the per-step Jacobian.
- `ws4_with_no_eddies_a_band_edge_stays_straight`: read within 1 km of where it started, away from storms.
- `ws4_the_weights_sum_to_one_and_the_ages_to_tau`.
- `ws4_a_storms_outline_keeps_its_shape_for_a_renewal`: Jupiter's oval:4:0 (day 20,000) and Saturn's oval:-6:0 (day 1,000) keep the kernel's aspect within 5 per cent, and each point keeps its radius within 10 per cent. It went red (1.77) with the one-body rule removed.
- Kernel: `a_giants_stirred_patch_says_where_its_cloud_came_from`.

## Cost

- About 1 to 5 s a cell on Jupiter at the dev profile's opt-level 2. `sse-weather` now builds at opt-level 2 in dev, like the other heavy crates.
- The patch is capped at 16 across. It is the reference for WS5's GPU pass, which the design sizes at about 40 steps.
- WS5 will need far fewer steps than this, or a shorter τ.
