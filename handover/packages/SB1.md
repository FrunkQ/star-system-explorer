# SB1: one belt member resolved into a definite body

Track SB, thread "Build asteroids and rings up close", 24 Nov 2026. Brief: Alex 24 Nov 18:02 and 18:05 UTC (see `status/small-bodies.md`).

## What was built
- Crate `crates/sse-small` (new; depends on sse-core and sse-formation, changes neither).
  - `resolve(seed, cloud, rank, host, star_au, upheaval, settings) -> Member`: starts from `Cloud::resolve` (the existing pinpointing of the k-th member) and adds, with draws on stream `small.member.v1` at the member's own address: makeup (scatter around the cloud mean, or one layer of a melted parent), blackbody temperature, rubble-pile strength (Farinella 1998 disruption time scaled to the cloud x gravity share), bulk density and porosity, YORP strength and spin (capped at the spin barrier for rubble), pair (YORP fission, gentle merger in a cold disc, impact moon) at the cited shares, figure (Maclaurin or Jacobi when relaxed, irregular or top-shaped otherwise, two lobes for contact binaries), weathering, type label read off all that, albedo and a history in plain sentences. `facts` carries `mass_kg`, `radius_m`, `makeup.*` for the resource books.
  - `notables(upheaval, leftovers)`: each world the violent phase shattered has its core as rank 1 of its debris cloud; `resolve` gives it a metal makeup (90% of the parent's metal) and the history line "The iron core of ..., laid bare when ... shattered it". The Psyche case.
  - `figure.rs`: Maclaurin q(e), c/a from q, the Jacobi line, barrier and relaxed limits.
- Pack `packs/base-small-bodies` (not loaded by the contract, so no golden change): every number, plus the Add list recipes.
- `docs/fudges.md` section "Small bodies" (14 rows), `docs/citations.md` section "Small bodies" (14 rows; three verified 24 Nov: Benner 2008 11% contact binaries, Margot 2002 16% binaries, Farinella 1998 lifetimes partly).

## Evidence
`cargo test -p sse-small`: 14 pass. Notably: 4,000 members a few hundred metres across at 1.2 AU give contact and separated shares within 0.04 and 0.05 of 11% and 16%; a cold Kuiper-belt cloud gives flattened gentle-merger contact binaries (lobe c/a < 0.6), mostly D types, little YORP; the barrier for 2,250 kg/m3 is 2.2 h; Maclaurin values match Chandrasekhar's bifurcation and maximum; km-sized main-belt members are mostly rubble; a 600 km+ member is relaxed; a formation run with a shattered world gives an M-type stripped core. Lobes keep the member's volume.

Golden: none changed. `scripts/check.sh` green.

## Integration points
- Kernel op (SB2) to call `resolve` with the system's star (luminosity, mass), age and the run's upheaval, and to list resolved members at their own address for the resource books.
- Formation gives one makeup per cloud, so S and C mix evenly across a belt (fudge SB-belt-gradient, SB5).
