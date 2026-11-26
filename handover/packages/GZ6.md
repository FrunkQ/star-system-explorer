# GZ6: Galaxies nearby part 2 (grand zoom design sections 7.2 and 7.4)

Owner: "Build galaxy charts and histories" (this package built by the thread "Build Galaxies nearby part 2", 26 Nov
2026). Design: `handover/design/SSE4-grand-zoom-design.md` sections 7.2 (the group's edge, the web's threads), 7.3
(real charts for the biggest neighbours), 7.4 (our satellites, Play, streams).

## Part 1 (26 Nov 2026): our satellites on their orbits with Play, streams, the group's edge
- Kernel: `sky_from_here` takes `satellite_paths` (default false, so no golden line moves). With it,
  `galaxy_walk.satellites.paths` holds every member of home's assembly tree that was followed (the same members
  `subhaloes` counts): its fate (survives, torn apart, merged), labels, infall, merge and disruption times, its halo
  and stars at infall, and its path from infall to now as `[t_gyr, x, y, z]` in kpc from the observer in the frame
  of each galaxy's `dir` (the orbit's own 0.02 Gyr points, one in five, plus its last). `centre_kpc` is our galaxy's
  centre from the observer. Test `satellite_paths_are_asked_for_and_end_where_the_sky_puts_each_survivor`: off by
  default the answer is unchanged; one path per member; merged count matches; each survivor's path ends exactly
  where the sky puts it (its `dir` times its distance).
- A third reach, **Our satellites** (0.4 Mpc, past home's 210 kpc halo), beside the group and the neighbourhood: the
  design's two reaches leave a satellite's orbit a few pixels across.
- **Play our satellites** (satellites and group reaches): from the first infall to the campaign's date in 24 s, with a
  scrubber and the time in Gyr after the Big Bang. Every place is read along the kernel's own path (`pathAt`); a dwarf
  falls in, is stripped, is torn into a stream or reaches our centre (drawn in the accent colour until it does). The
  caption counts those on their orbits, torn into streams and merged at that time. At the campaign's date the
  satellites are the sky's own galaxies again.
- **Streams** under HUD: each torn-apart satellite as a faint arc along its own path (`streamArc`, fudge GZ-F12),
  also drawn during Play from the moment it is torn apart. A pressed satellite shows its orbit so far, and its card
  says when it fell in.
- **The group's edge** under HUD (group and neighbourhood reaches): Sandage's zero-velocity sphere of the mass inside
  it (`zeroVelocityRadius`, `groupEdge`: home's halo, then every catalogued halo inside, widened until it settles;
  fudge GZ-F13, Sandage 1986 unverified, on the master list). The caption says how far out and how many galaxies.
- The chip now says "as it is at the campaign's date": the sky op was already read at the campaign's time (the
  kernel's default `t_myr` is the universe's now plus the campaign's elapsed time), so part 1's honest limit "read at
  the present day" was wrong and is corrected here.
- Pure code in `ui-contract/src/galaxies-nearby.ts`, tested in `galaxies-nearby.test.ts` (paths, streams, the edge
  with an absolute hand-worked value: 3e12 suns at 13.8 Gyr is 1.277 Mpc). E2e in `tests/ui/cluster.e2e.mjs`: the
  edge about a megaparsec out, streams under HUD, Play, pause in the past, scrub to halfway, back to the date.

## Part 3 (26 Nov 2026): the biggest neighbours' real faces, the web's threads
- The three biggest neighbours in stars within the reach (`biggestNeighbours`, never home's satellites, which have
  no chart of their own) have their real charts read from the kernel (`galaxy_chart` with their cell and halo), one
  at a time in the background, and kept. In True each swaps its type picture for its real face (`galaxyFace`, the
  same picture Galaxy 3D's True draws, with its bulge glow) as the read finishes, laid at the sky's tilt and turn. A
  chip says "Histories read: 2 of 3", and the card says whether that galaxy's history is read.
- Under HUD, **the web's threads** lie on our disc's plane, from a web slice through home (`web_slice` centred on
  `galaxy_walk.home_mpc`, the reach's width, 64 cells): only what is over the mean density, inside the reach's circle,
  turned and tipped with the view. The web's axes are the galactic frame's (F37), so the slice is our disc's plane.
- E2e: the web's threads appear in the neighbourhood under HUD; in True the neighbourhood reads 3 of 3 histories and
  draws their real faces. Screens: project files `sse4/handover-screens/GZ6-screens/` (`gz6-nearby-real-faces`).

## Follow-up (26 Nov): physics out of the interface
After an audit against "physics lives in the kernel's editable blocks":
- The group's edge is now the formula block `sse.gravity.zero_velocity_radius` (Sandage 1986, `packs/base-physics/formulas/gravity.json`), which reads G through Look up; its reach is declared as following G. The kernel runs it in `galaxies.rs` `group_edge` when `sky_from_here` is asked with `satellite_paths`, and returns `galaxy_walk.group_edge`. The interface's own copy (`zeroVelocityRadius`, `groupEdge`, G typed in) is gone; Nearby only reads it. In the browser the kernel gives the same 0.885 Mpc as the old sum.
- Golden: `tests/golden/contract-reach.txt` gains one line on purpose (the new block follows G). Atom count 176 to 177 (P1 log).
- Colour from B-V stays a display rule (fudge GZ-F14), now in one copy: `ui-contract/src/compare.ts` `tempFromBV`, read by the sky's stars, the chart's galaxy dots and True's type pictures.

## Honest limits
- Play is this view's own playback of the stored orbits, not the universe clock: the rest of the screen stays at the
  campaign's date. The Run desk has no clock above the system (design 5.9); Play shows on every desk until the
  three desks work says which parts each desk shows.
- During Play every satellite is drawn as a dot sized by its stars at infall, not with its type and colour at that
  time (those are the sky's, read only at the date).
- The group's mass is the haloes' r200 masses and the sphere has no dark energy (GZ-F13).
- Big neighbours' own satellites are not walked yet (design 7.4).
- The web's threads are one slice, in our disc's plane only, and the galaxies are not yet moved by the web's
  displacement (grand zoom gap 4, GZ15), so a galaxy can sit beside its thread rather than on it.
- A neighbour's real face is laid at the sky's tilt and turn, not the axis its chart would give (GZ14).

## Next
GZ6 is done but for the big neighbours' own satellites (design 7.4), which wait on a satellite walk per neighbour in the kernel. GZ7 (the Local stars rung) is the next package of Stage B.
