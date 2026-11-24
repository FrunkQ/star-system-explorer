# RK5: slow spirals (track RK, travel thread)

Atomic Rockets plan, section 9 and the Mission, Orbits row: "Slow spirals for low-thrust ships (weeks of thrust, not a burn)"; one new leg-maker.

## Part 1: the spiral leg-maker (24 Nov 2026)

- **The gap (from RK6, Paper spacecraft).** At very low thrust, plan_voyage offered impulsive Lambert transfers as if the burns were short. HOPE's nuclear electric Callisto ship (6e-5 g, 8,000 s) got 375 days to Jupiter where the study flies 832.
- **Short kicks refused when they are not short.** A Hohmann or Lambert course whose burns take more than `impulsive_burn_max` of its flight (0.1, a NavSettings value with a serde default) is refused, saying "its burns take N days of an M-day flight, so they are not short kicks; flown as a slow spiral instead".
- **The spiral (`courses.rs` `spiral`, kind and id `spiral`).** It is offered when the textbook transfer's own burns would be long.
  - Three parts: out of the parking orbit, across between the circular orbits, and down into the far parking orbit. Each costs about the difference of the circular speeds (Edelbaum 1961, coplanar case; escape by tangential thrust costs about the orbit's own speed).
  - Constant thrust, so t = (vₑ/a₀)(1 − e^(−Δv/vₑ)).
  - It leaves when the far world will be alongside where the spiral ends: the angle swept is integrated in Δv steps, and the window is searched as for the Hohmann phase.
  - The track is drawn as a circular-speed spiral. It arrives at rest (`arrive_v_inf_m_s` 0) and burns for the whole flight.
- **Results.** HOPE's Callisto ship takes 868 days (141 out, 269 across, 458 down to Jupiter) against the study's 832. The fission fragment ship (1.5e-5 g, 527,000 s) takes 5,071 days against "over 2,100", because here it goes down to a low orbit round Jupiter (40.6 km/s). The fudge register has the row.
- Checks: `crates/sse-nav/src/spiral_tests.rs`.
  - A gentle drive spirals and its kicks are refused.
  - A brisk drive keeps its short burns.
  - Spirals are repeatable.
- No stock ship is gentle enough to change, so no goldens move.

## Next

- Part 2: a beamed leg that depends on its sender (thrust from a construct elsewhere; the "beams spread" Equation). Sails get thrust that falls with distance squared, which also needs the spiral's Δv steps to read the local flux.
- A parking orbit chosen per mission (for example Callisto's orbit rather than a low one), and plane changes (Edelbaum's full formula).
