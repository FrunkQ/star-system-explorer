# GZ0 kernel checks (grand zoom, stage A), 25 Nov 2026

These are checks only, with no feature. The design is `design/SSE4-grand-zoom-design.md`, sections 6.2, 6.5 and 10.

## 1. Which place a campaign system's galactic history uses today (design 6.5, "to confirm")
I read every caller on main at 25 Nov 13:30 UTC.
- **Per-site events never reach a campaign system.** The per-cell supernovae, kilonovae, gamma-ray
  bursts, close passes and arm crossings (`sse-cosmos::events::Events::exposure`, `encounters`,
  `arm_crossings`) are called only by `events::run_manifest`. That is the G6 manifest runner, used
  by the CLI and `g6_tests`. No kernel op calls it. Its site is `site_r_sun` times the median
  galaxy's Sun radius.
- **What the kernel reads instead:**
  - The `galaxy_chart` op shows galaxy-wide events only (mergers, starbursts, nucleus episodes),
    on the median "milky-way-like" halo (`cosmos.rs`, `named_halo(&hs, None, None)`).
  - `reconstruct_origin` defaults its present radius to `env.sun_radius_kpc()` (F37's stand-in).
  - `sky_from_here`, `sky_glow` and `star_cell` put the observer at the Sun's radius on the grid's
    x axis when none is given (`sky.rs` 143, `glow.rs` 54, `cosmos.rs` 844).
- **Life:** `life_view` reads fixtures only, and an adopted body is refused ("adopted systems do not
  carry habitat graphs yet"). No life history reads a galactic event.
- **Not wired yet:** formation (`run_formation`) takes the star's birth and metals from its ring
  when grown `from_star`, but no event history.

So the design note's "a system not grown from a drawn star gets the Sun's radius of a median galaxy"
is true for the sky and the origin. For events it is weaker: **no campaign system gets a site event
history at all.** GZ13 (a star slot for every system) should come with a kernel read of a system's
exposure at its slot (`Events::exposure` on the system's cell and path). Until then, life and the
Run desk have nothing to read.

## 2. How far apart one galaxy's two places are (design 6.2 point 4)
Test `sse-cosmos` `gz0_tests::a_galaxy_sits_a_few_mpc_apart_in_the_cube_and_the_catalogue`, on
1,439 haloes above 1e11 M☉ in the 5×5×5 cells (8 Mpc each) round the origin, today:
- median **7.2 Mpc**, 90 % 11.3 Mpc, max 15.1 Mpc.

The jump is about one cell. A galaxy drawn by `halo_cell` and `galaxies_seen` (the starting place)
sits one cell's width from where the web cube draws it (the displaced place). GZ15, one place per
halo, is needed before any journey crosses the web and galaxy seam.

## 3. Not measured here
Cold costs on desktop, the phone stand-in and a real phone (section 10) need a browser run. That is
the travel thread's GZ0 half. The orientation chain against the sky is not checked yet.
