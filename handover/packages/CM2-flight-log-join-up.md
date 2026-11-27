# CM2: mounts joined up with the flight log (compound ships)

Track CM. Design: `handover/design/SSE4-compound-ships-design.md` sections 6.1 to 6.3 and 13, accepted 27 Nov 2026 with Alex's rulings in section 14A. Built 27 Nov 2026 by the thread "Build mounts for compound ships", after CM1 (`packages/CM1-mounts.md`).

## What it does

- **`mounted` pieces** (`sse-nav/src/flightlog.rs`, `State::Mounted {to, place, port}`): a carried construct's log says who carries it and where. `docked` pieces keep working and read the same way (a latch on the hull).
- **Where a carried craft is** is where its compound's root is: `flight_log` with `where_at` follows `mounted` and `docked` pieces up to the root's log (at most 32 carriers deep) and gives the root's position and velocity, keeping the carried log's own kind and words ("carried at the front of ...").
- **Capture** (`mount_construct`) writes a `join` event on the carrier's log and a `mounted` piece on the carried construct's, whenever either has a log. With a system (`fixture`, `nav` or `system`, as `flight_log` takes them) it first reads both logs at that second: further apart than `capture_reach_m` or faster than `capture_speed_m_s` (base-nav `nav.mounts`) is refused (`capture_apart`) with the distance, the speed and the cheapest one-arc rendezvous (a Lambert transfer around the target's body over flight times that are fractions of its period: when to leave, when to meet, the Δv). `authored: true` makes it anyway, with a note of how far apart they were (steer, never stop).
- **Release** (`release_construct`) writes a `split` event on the carrier's log and seeds the released construct's first free piece from the compound root's state at that second:
  - an orbit, with a system: the state worked out, the separation push added (the mount's own, else `nav.mounts.push_m_s`), and turned back into elements (`elements_from_state`, the exact inverse of `OrbitElements::state`, round-trip tested for ellipse, circle, equatorial, retrograde and hyperbola);
  - an orbit, with no system: copied, and the push noted as not applied;
  - a course, with a system: the course's state at that second, plus the push, as an orbit of the star. This removes the old "give it its own first piece" refusal for a split on a course;
  - landed or between the stars: copied (a rover driving out of a garage, a lander lifting off later has its own launch);
  - a track or an end: not seeded, with a note.
- Nothing is written to the flight logs when neither construct has one, so a GM using mounts without logs gets no noise.

Push direction: along the flight for the front, against it behind, outward from the body for the hull, a bay or a tether. The stack's attitude is not modelled; fudge row "Release push direction from the place".

## Settings

`packs/base-nav` `nav.mounts`: `push_m_s` 0.5, `capture_reach_m` 10, `capture_speed_m_s` 0.3 (design 4.1 and 6.3, estimated and unverified, fudge row "Mount defaults"), and `rendezvous_tof_fractions` (a search grid). Read through the door `sse_physics::settings("nav")`, never the shipped pack: with no nav settings on the stack the push is 0 and the capture is not checked, each with a note.

## Gates, seen red

- `mount_log_tests.rs` (5 tests). With the push removed from the seed, `a_release_seeds_the_free_log...` fails (0 m/s apart, not 0.5). With the reach check always passing, the same test fails (the drifted capture is made). Both restored.
- `elements_from_a_state_give_the_state_back` fails with the true anomaly's sign flipped.

## Goldens

- Area `compound` only: replies gain `logged`, and new cases log Columbia's orbit, mount the retroreflector on the hull, read where it is, release it with a 0.2 m/s push in the Sol fixture and read its free orbit. No core line moves. The nav crate golden does not move.
- Reach ledger: `flight_log` gains an area case and does not follow the stack (it reads the NAV fixture): `untested` becomes `still`. `log_flight` becomes `quiet` (writes the record). `mount_construct` and `release_construct` change from `quiet` to `still`: with a system they read its orbits from the fixture or the universe system, not from the stack. Same count of lines.

## Left for later

- Propellant and other lines across a mount (CM3); a release does not yet check what the carried construct's tanks hold.
- The sequence that flies releases and captures in order, with the attitude (CM6).
- A rendezvous offer around a body other than the target's own, and one when the target is landed or on a course.
