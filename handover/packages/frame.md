# System frame in the galaxy (for the renderer's Y5 sky)

Asked for by the renderer thread on 22 Nov 2026: the Window gives bodies in the system's own frame (orbit-plane longitude and latitude), while `sky_from_here` and `sky_glow` give galactic l and b. To draw both in one sky the renderer needs each system's orientation in the galaxy. Built 22 Nov 2026 by the kernel thread. Code `crates/sse-kernel/src/frame.rs`.

## What it gives
`frame: {pole_l_deg, pole_b_deg, zero_l_deg, zero_b_deg, node_l_deg, to_galactic, evidence, source, model}`:
- the system's north pole (+z) and longitude zero (+x) in galactic l, b;
- `node_l_deg`, where the system's plane rises through the galactic plane;
- `to_galactic`, the 3 × 3 rows taking a unit vector in the system's frame to galactic Cartesian (x toward l 0, b 0; z the north galactic pole);
- `evidence`: "measured" for Sol, "drawn" for any other system.

It comes on:
- `resolved_system`, always (drawn, keyed by the universe seed and the system's address);
- `window_view`: the Sol fixture gets Sol's measured frame; an inline `nav` gets a drawn frame when the request names its `system` and a universe is open, and null otherwise.

## Sol
`packs/base-sol` now carries the Sun's `system.frame.pole_l_deg`, `pole_b_deg`, `zero_l_deg` and `zero_b_deg`: the J2000 ecliptic north pole (l 96.384, b 29.811) and equinox (l 96.337, b −60.189). `scripts/build-base-sol.py` works them out from the Hipparcos ICRS-to-galactic matrix (ESA SP-1200) and the IAU 2006 obliquity (Hilton et al. 2006, doi:10.1007/s10569-006-0001-2). The Sol fixture's elements are J2000 ecliptic, so this is the frame its Window reports in.

## Other systems (fudge F38)
The pole is isotropic (sin b uniform, l uniform) and the longitude zero uniform about it, from the keyed stream `system.frame.v1`. A later stage can derive the pole from the birth cloud's spin, with sibling systems correlated.

## Verified by
- `frame::tests::sol_s_frame_puts_the_ecliptic_where_it_is_measured`: the pole is at l 96.38, b 29.81; ecliptic longitude 90 lands at l 186.37, b −0.02 (where the ecliptic crosses the Milky Way); the matrix is a rotation.
- `frame::tests::drawn_frames_are_keyed_and_isotropic`: 2,000 systems; the same address gives the same frame; mean sin b is within 4 sigma of zero; the zero lies in the system's plane.
- `the_window_says_how_its_frame_sits_in_the_galaxy` and `templates_through_the_api` (kernel API).

## Known limits
- A system resolved from the campaign is always "drawn", even if a GM authored it as Sol. Sol's measured frame comes only through the Sol fixture for now.
