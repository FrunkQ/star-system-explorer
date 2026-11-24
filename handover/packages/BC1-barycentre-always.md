# BC1: barycentre always

Kernel track, 24 Nov 2026.

Binaries caught SSE3 out badly when they were added late. Alex asked (24 Nov, editing design thread) for SSE4 to handle them properly before editing goes further. His points: Pluto and Charon on a barycentre; users adding stars and planets by hand; adding and removing bodies as "well managed chaos". He suggested always assuming a barycentre instead of V3's 8% mass rule, and said "everything can be a barycenter - the maths just rounds down to zero". The barycentre stays hidden from users; the editing design thread owns the handles.

## The model
There is one rule and no threshold. Each body orbits the centre of mass of its host plus everything that orbits that host further in, including those bodies' own moons. This is the Jacobi hierarchy. Its mean motion counts all of that mass and its own group's mass. Every host is offset by its children's pull. Nothing new is stored: the data is still each body's `orbit.host` and elements, and the barycentre is worked out.

What follows from the one rule, with no special cases:
- **Pluto and Charon** go round a point about 2,100 km from Pluto's centre, outside Pluto.
- **The Earth and Moon** share a barycentre inside the Earth.
- **A binary's stars** go round each other. A companion's elements are the mutual orbit.
- **A planet inside a companion's orbit** goes round its own star.
- **A planet outside the companion** goes round the pair, with both stars' mass. This makes circumbinary planets correct even though formation writes `orbit.host` = star 0.
- **A tertiary** goes round the inner pair, as formation meant.

## What changed
- **`orbits_at`** (`crates/sse-kernel/src/orbits.rs`) works in two passes: read each orbit, then place the tree (`Tree`).
  - `r_m` keeps its meaning (from the host's centre), so existing views get the wobble for free.
  - New per time: `sys_m` (from the barycentre of the whole hierarchy) and `centre_m` (the centre the orbit goes round, from the host's centre: the point a view draws the ellipse round).
  - `period_s` counts the inner mass.
  - A body whose hosts form a ring is unplaced and the reason is named.
- **Stability (ST1):**
  - The central mass for the Hill, AMD and AMD-critical checks is the host plus everything further in.
  - Worlds either side of a companion star are no longer paired as neighbours.
  - A star is known by its `body.kind`, its temperature, or a V3 role, so V3 binaries are checked.
- **NAV:** `NavSystem::at` counts the world's own mass (G(M+m)), as `orbits_at` does, so the chart and NAV put a world in the same place.
- **Withdraw:** a withdrawn body takes everything that orbits it, following the `orbit.host` chain as well as the address tree. A V3 moon sits beside its planet, so before this change it was left with a dangling host.
- **Hosts checked on write (K13):** `set_authored` and `set_authored_batch` refuse an `orbit.host` that is not in the system, is the body itself, or would close a ring of hosts (code `bad_host`). Every write path that sets a host goes through the same check (`hosts_ok`).
- **`rehost {address, to, keep?, at_s}` (K13, the editing design's re-parent):** moves a body to a new host in one authored event. `keep: "motion"` turns its present position and velocity (finite difference over 30 s) into elements round the new host's barycentre (Vallado alg. 9) and refuses with the reason when it would not be bound ("would escape"). `keep: "tidy"`, the default, gives a round orbit at its present distance. A host inside the body's own subtree is refused ("orbits it").
- **`withdraw` gains `others`** (`take`, the default, `leave` or `hand`, with `to` and `keep`): what went round the withdrawn body goes with it, stays round its host, or goes to a named body. It is one event however many bodies move.
- **`preview_tree {system, at_s, edits}` (K13):** runs the edits (only `set_authored`, `set_authored_batch`, `rehost`, `withdraw`, `restore`) on a copy of the universe, then puts the universe back. It returns `before` and `after` trees and `changed`. Each tree row has `address`, `name`, `kind`, `host`, `period_s`, `pair`, `verdict`, `says` and `missing`. `pair` is true when the pair's centre lies outside the host's surface (Pluto and Charon, not the Earth and Moon). A refused edit is returned as `edit N: ...` and nothing is written. `Universe` derives `Clone` for this.
- **`orbit_stability`** leaves withdrawn bodies out.
- **Starter systems (`packs/base-starters`, `scripts/build-base-starters.py`, checked in `scripts/check.sh`):** Alpha Centauri (Kervella et al. 2016 and 2017; Proxima's orbit from Kervella, Thévenin and Lovis 2017) and Kepler-16 (Doyle et al. 2011). Each has base-sol's sheet shape, so the start sheet's door can read it. Kepler-16 B and the binary orbit are marked unverified: the paper's Table 1 was not reachable and the NASA Exoplanet Archive does not print them. The door should send each body's catalogue `kind` as `body.kind`.
- **base-sol:** Charon, Phobos, Deimos, the Galileans, Titan and Triton, from the NSSDCA satellite sheets (read 24 Nov 2026). Triton's orbit is retrograde (157.3°). The pack is rebuilt by `scripts/build-base-sol.py`.

## Checks
`cargo test -p sse-kernel bary_tests`:
- Pluto's barycentre offset is 2,126 km, and Charon's period is 6.387 days.
- In a binary, the outer planet's centre is the pair's barycentre and its period counts both stars.
- A ring of hosts is named and does not loop.
- Stability reads the hierarchy.
- A withdrawn world takes its moons.
- Hosts are checked on write; rehost keeps motion or tidies; withdraw takes, leaves or hands on; preview_tree writes nothing.
- `starters_tests`: Alpha Centauri B's period comes back as 79.93 years and Proxima's as about 550 thousand years (the paper gives 547, +66 or -40). Kepler-16's planet comes back at 228.8 days, which needs both stars' mass; star A alone would give 242. It lies outside the nearest lasting orbit round both stars (Holman and Wiegert, 0.646 au), and both systems last.

## Golden (on purpose)
- `tests/golden/contract.jsonl`:
  - `orbits_placed` lines gain `sys_m` and `centre_m`, and periods now count the body's own group mass.
  - NAV route lines move by the G(M+m) change.
  - `orbit_stability` lines change by the hierarchy's central mass.
  - Line 1 (the op list) and line 41 (the unknown-op message) gain `rehost` and `preview_tree`.
  - New cases at the end, in universe `u:k13`: a host that is itself is refused, a tree preview, a rehost keeping motion, a withdraw that leaves what orbited it, and a stability read.
- `crates/sse-nav` golden: positions move by the same G(M+m) change.

## Next
- **V3 barycentre nodes:** members that orbit a massless `bary:N` are unplaced today. On import they should become the primary's companion orbit.
- **Approximations:** rehost's inner set uses the body's present distance. Withdraw with `leave` counts the withdrawn body's mass in the inner barycentre for the instant of the move.
- **Starters:** read Kepler-16's Table 1 at source and mark it verified. Add Proxima's planets.
- **NAV moons:** NAV still knows only the star's worlds.
- **Binary evolution** (merger, mass transfer) as dated changes, after the run-through.
