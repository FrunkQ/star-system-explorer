# BC1 follow-on: V3 barycentres onto the hierarchy on import

Thread "Build galaxy arm fix and atmosphere advice", 25 Nov 2026. This is the "Next" item in `BC1-barycentre-always.md`: members that orbit a massless V3 `bary:N` used to be left unplaced.

## What V3 writes
How V3 lays out a pair, read in the public SSE3 source (FrunkQ/star-system-generator, `setupStars.ts`, `generateFromConfig.ts`, `physics/barycenterReconcile.ts`):
- A barycentre node lists `memberIds`.
- Each member's orbit has `hostId` set to the barycentre and `a_AU` set to its share of the separation (heavy `sep·m_light/M`, light `sep·m_heavy/M`). The two members sit half a turn apart.
- An auto-promoted pair (the 8% rule, for example a double planet) carries the old host-track orbit on the barycentre itself.
- A circumbinary planet's host is the barycentre.
- In a hierarchical triple, the inner pair's barycentre is a member of the outer one.

## What adoption does now (`rehome_barycentres`, `crates/sse-kernel/src/adopt.rs`)
- The importer writes `v3.memberIds` as addresses.
- Each barycentre's heaviest member stands for the pair. For a nested pair, it is the body at the bottom of the heaviest members.
- That body takes the pair's own orbit round its host. When the pair is the root, the body orbits nothing.
- Each other member orbits that body at the pair's separation (`a_light + a_heavy`, or scaled by the masses when the heavy side's distance is missing). It keeps its own eccentricity, angles and phase. A light member that is itself a pair passes this orbit to its own heavy body.
- Anything else that named the barycentre as host or parent names that body. The Jacobi hierarchy then counts both stars for a world outside the companion.
- The `bary:N` rows are left out, because SSE4 keeps no barycentre objects. Their hand-set `v3.*` facts, such as the pair's name and `coOrbital`, move to the body that stands for the pair as `v3.pair.<bary:N>.<name>`.
- The report records one line per barycentre in `barycentres_mapped`. This field is left out when it is empty, so saves without barycentres hash as before.

## Checks (`cargo test -p sse-kernel adopt_bary`)
- **Binary as V3 writes it**, with a circumbinary world, a world round the companion, and a double planet round star A that has a moon:
  - Every body is placed.
  - The companion's period is Kepler's for the 20 au separation, with everything inside it counted.
  - The circumbinary world's period counts both stars.
  - The double planet's light member orbits the heavy one at the pair's separation.
  - No `bary:` address is left.
- **Alpha Centauri-shaped triple:** the third star orbits A at the outer separation, and B orbits A at the inner one.
- **Synthetic Sol fixture:** it has no barycentres and comes out unchanged.

## Limits
- When V3's two members disagree on shape or angles, which V3 itself never writes, the companion's own values win.
- A pair riding a Lagrange point (`coOrbital` on a barycentre) keeps the marker as `v3.pair.<bary:N>.coOrbital` on its heavy body. The point itself is not derived, as for any V3 co-orbital.
- No golden changes: no fixture or contract case has a barycentre.
