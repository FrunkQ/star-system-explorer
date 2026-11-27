# RW7a: Saturn's small moons and J2 in base-sol

Thread "Add every Solar System body", 26 Nov 2026. Track RW, package RW7 (`design/SSE4-real-catalogues-design.md` 6.1 and 11), first part: asked for early by the ring structure work (PR 654), whose Encke and Keeler gaps and shepherded edges need the moons that carve them, and whose resonances need Saturn's J2.

## Built
- `scripts/build-base-sol.py` gains Pan, Daphnis, Atlas, Prometheus, Pandora, Epimetheus and Janus (mass, mean of the three printed axis radii, semi-major axis, eccentricity, inclination, geometric albedo; Daphnis prints none), and Saturn's `body.j2` (16,298 x 10^-6, dimensionless; the sheet names no reference radius, and the note says so).
- They are listed after Titan, so every moon slot already in use keeps its address (Titan stays Saturn's slot 6; the access-label test pins it).
- `packs/base-sol/pack.json` rebuilt: 43 bodies. `crates/sse-pack` count test 36 to 43.

## Where the numbers came from
NASA NSSDCA Saturnian Satellite Fact Sheet (last updated 22 July 2025) and Saturn Fact Sheet (last updated 18 March 2025), read on 26 Nov 2026 through the cloud's page reader, table rows quoted cell for cell. No value was typed from memory. The radii and orbit values are rounded on the sheet from Thomas 2010 (Icarus 208, 395), as its footnote says.

## Golden
No golden moved: the contract runs on its own fixtures, not on base-sol.

## Not done here
The rest of RW7 (the catalogue build scripts for every moon, minor planet and comet; full orbits at a date; belt masses by group; the Expected stop; notable flags). J2 for the other giants.
