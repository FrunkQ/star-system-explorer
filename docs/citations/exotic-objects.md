# Citations: exotic objects (track XO)

References relied on by the gravity blocks (`packs/base-physics/formulas/gravity.json`, landing as XO1 part 2; ready in `handover/packages/XO1-part2/`) and the bend table
(`crates/sse-gravity`). The design note's section 14 was written from memory; each row here was checked
when its block was built (XO1, 24 Nov 2026). "Metadata" means the title, journal, volume, page and DOI
or ADS bibcode were seen in a search result for the work (ADS abstract page or publisher DOI); "numbers"
means the relation was checked by the tests against values computed independently.

| Where | Reference | Relied on | Status | Checked |
|---|---|---|---|---|
| `sse.gravity.last_stable_orbit`, `sse.gravity.photon_orbit` | Bardeen, Press and Teukolsky 1972, ApJ 178, 347, doi 10.1086/151796 | eq. 2.21 (last stable orbit by spin), eq. 2.18 (equatorial photon orbit) | verified (metadata via ADS and doi.org; numbers: 6, 1.237 at a = 0.998, 4.233 at a = 0.5, 9 retrograde; photon orbit 3, 1 and 4) | the equation numbers are as widely quoted, the page body was not read |
| `sse.gravity.shadow_angle` | Synge 1966, The escape of photons from gravitationally intense stars, MNRAS 131, 463, doi 10.1093/mnras/131.3.463 | sin θ = √27 (M/r)(1 − 2M/r)^½ for a static viewer | verified (metadata via ADS 1966MNRAS.131..463S); relation checked against the far-viewer limit √27 M/D in the tests | the DOI string is the journal's usual pattern, not resolved here |
| `sse.gravity.horizon` | Kerr 1963, Phys. Rev. Lett. 11, 237, doi 10.1103/PhysRevLett.11.237 | r₊ = M + √(M² − a²) | textbook | |
| `sse.gravity.weak_bend` | Einstein 1915, Sitzungsber. Preuss. Akad. Wiss., 831 (ADS 1915SPAW.......831E); Dyson, Eddington and Davidson 1920, Phil. Trans. A 220, 291, doi 10.1098/rsta.1920.0009 | α = 4GM/c²b; 1.75″ at the Sun's limb | textbook (the test gives 1.751″) | |
| `sse.gravity.einstein_angle` | Einstein 1936, Science 84, 506, doi 10.1126/science.84.2188.506 | point-lens ring radius | textbook | |
| `sse.gravity.clock_rate`, `sse.gravity.tide`, `sse.gravity.gravitational_radius` | Misner, Thorne and Wheeler 1973, Gravitation (ADS 1973grav.book.....M); Schwarzschild 1916 (ADS 1916SPAW.......189S) | static and circular-orbit clock rates; radial tide 2GM/r³ | textbook (0.8165 at r = 6M) | |
| `crates/sse-gravity` bend table | Luminet 1979, A&A 75, 228 (ADS 1979A&A....75..228L); Darwin 1959, Proc. R. Soc. A 249, 180 | the orbit equation d²u/dφ² + u = 3Mu² and the method of reading images off it | textbook (equation); numbers checked against an independent quadrature of the sweep integral to 10⁻⁶ | |
| `crates/sse-gravity/src/tests.rs` | Keeton and Petters 2005, Formalism for testing theories of gravity using lensing by compact objects: static, spherically symmetric case, Phys. Rev. D 72, 104006 (arXiv gr-qc/0511019) | deflection series 4/b + 15π/4b² + 128/3b³ + 3465π/64b⁴ | verified (metadata via ADS 2005PhRvD..72j4006K and arXiv); the series agrees with the table to 2×10⁻⁵ at b = 64 | |
| shadow fixtures (tests) | Event Horizon Telescope Collaboration 2019, ApJL 875, L1; 2022, ApJL 930, L12 | M87* ring 42 ± 3 µas; Sgr A* ring 51.8 ± 2.3 µas; masses 6.5 × 10⁹ and 4.15 × 10⁶ Suns, distances 16.8 Mpc and 8.2 kpc | unverified (from memory; the design note's figures); the computed shadows (39.7 and 51.9 µas) sit where the papers' shadow-to-ring relation puts them | to check in XO2 |
