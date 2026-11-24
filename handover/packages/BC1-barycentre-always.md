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
- **base-sol:** Charon, Phobos, Deimos, the Galileans, Titan and Triton, from the NSSDCA satellite sheets (read 24 Nov 2026). Triton's orbit is retrograde (157.3°). The pack is rebuilt by `scripts/build-base-sol.py`.

## Checks
`cargo test -p sse-kernel bary_tests`:
- Pluto's barycentre offset is 2,126 km, and Charon's period is 6.387 days.
- In a binary, the outer planet's centre is the pair's barycentre and its period counts both stars.
- A ring of hosts is named and does not loop.
- Stability reads the hierarchy.
- A withdrawn world takes its moons.

## Golden (on purpose)
GOLDENS

## Next
- **Re-home on remove** (Alex's question 3, proposed): an op that turns a body's current position and velocity into elements about a new host. It would serve "remove the planet, keep its moons", and also capture.
- **V3 barycentre nodes:** members that orbit a massless `bary:N` are unplaced today. On import they should become the primary's companion orbit.
- **NAV moons:** NAV still knows only the star's worlds.
- **Binary evolution** (merger, mass transfer) as dated changes, after the run-through.
