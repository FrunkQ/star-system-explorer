# VP-R3: ring structure from the moons (part 1)

Track SB (visual polish design 5.9, section 12), thread "Build asteroids and rings up close", 24 Nov 2026.

## What was built
- `packages/renderer/src/ring-structure.ts` (new): `ringMarks(moons, planet_kg, inner_m, outer_m, nu)` lists each moon's first-order inner Lindblad resonances inside the rings, r = a ((m-1)/m)^(2/3), with the ratio of the resonant torque (Tajeddine et al. 2017 eq. 5, f = 8.6 m^2) to the ring's viscous torque (eq. 6); Sigma cancels. A ratio of 1 or more is an edge; below, a dip of depth ratio/(1+ratio). A moon inside the rings clears a gap of half-width 415 sqrt(mu) a. `cutRings` cuts those marks into the ring data (edges only where they fall more than 1% inside a recorded edge, since resonances are placed without J2).
- `true-body.ts`: optional `mass_kg` and `moons` on `TrueBody`; with both, the profile is cut and drawn at 2048 pixels instead of 512.
- Fudges SB-ring-viscosity, SB-resonance-j2, SB-resonance-dip, SB-gap-width; citation row "VP-R3 resonant torque".

## Evidence
`packages/renderer/test/ring-structure.test.ts`, 6 pass, all against Saturn: Mimas 2:1 within 1% of the B ring's outer edge with a torque ratio above 10 (the paper's "more than ten times"); Janus 7:6 within 0.2% of the A ring's outer edge with a ratio near 1; Encke and Keeler gap widths within 10% (fit); cutting keeps recorded edges, splits the A ring at Pan, cuts a made ring at Mimas 2:1; trueLook clears the pixel at Pan.

## Next
- The system view (System3D, `bodies.ts`) to pass the world's mass and its moons' masses and orbits on the TrueBody; Body3D carries no masses yet (owner: chart room / system view).
- Wavy gap edges (wavelength 3 pi x half-width, Tiscareno 2013) need the rings drawn as a 2D texture round the world; density-wave trains need the ring's surface density.
