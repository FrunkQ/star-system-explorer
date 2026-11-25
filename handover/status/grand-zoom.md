# Status: the grand zoom and travelling between places (track GZ)

Owner thread: "Design the grand zoom" (Fable, design only, 25 Nov 2026) for the note. No builder thread yet: the note's section 13 proposes "Build travel between places" (Opus) for GZ0, GZ1, GZ3, GZ4, GZ8, GZ9 and GZ11. Until one exists, each package below belongs to the thread named in its row, and that thread records it in its own status file too.
Design: `design/SSE4-grand-zoom-design.md`, **ACCEPTED by Alex 25 Nov 2026 12:45 UTC** ("yes to all... once you have manipulated them based on above"). Read its sections 1, 6 and 14A first; 14A (his rulings) wins where it differs. Mockup in project files at `sse4/handover-screens/GZ-screens/` (`grand-zoom-mockup.html` and screens; painted stand-ins, not SSE4 output).

What it is, in one line: a change of place can be travelled instead of cut. The use of a bookmark, postcard, Playbook Show step or place link gains a "way in" (`sse.approach/1`: where the journey starts, stops or one smooth zoom, HUD, words, pace), each level keeps its own view, and at each change of level two views cross-fade while both zoom onto the same target (the one new fudge, GZ-F1). The grand zoom is the longest way in, from the cosmic web to a site; the short form makes every step on the place line a zoom. The note also says what "Galaxies nearby" needs to become a proper Local Group view (its section 7) and adds a "Local stars" rung (section 8).

Feasibility, as told to Alex 25 Nov: feasible. The levels are one computed chain (web, halo, galaxy, ring, star cell, star, formation by `from_star`, world, site) with three gaps: our galaxy is the median halo, not one of the web's catalogue (F37); systems not grown from a drawn star have no cell; the screens do not pass the place along. The note's section 6.3 closes them, each computed once (GZ12, GZ13), adds galaxy tilt from the web's tides (GZ14) and drift from its displacement (GZ15), and section 8A adds a Motion layer to the HUD (velocity and pull arrows with speed, Alex's ask; GZ16). Also missing for the zoom itself: a camera carried across a level change, one read for a place's chain, a 3D Local Group, two rungs on the place line.

Nothing is built. GZ is a code only, never shown in the product.

**Order (Alex, 25 Nov 12:45 UTC: the Local Group and the tie-up first).** Stage A, the tie-up: GZ0, GZ12, GZ13, GZ2, GZ17, then GZ15, GZ14. Stage B, the Local Group: GZ5, GZ6, GZ7. Stage C, travel: GZ1, GZ3, GZ4, GZ16, GZ8 to GZ11. Standing rule from here: no system without a star slot, no star slot without a cell, no cell without a galaxy, no galaxy without a halo in the web; a test fails if an op falls back to a stand-in place. On the Run desk a level is shown plain and stepping the place line by the short zoom is the way to move (note 5.9).

| Package | State | Owner (proposed) |
|---|---|---|
| GZ0 checks and timings (read costs on desktop and phone, one position per galaxy at every level, the orientation chain) | accepted, not started | "Build travel between places" |
| GZ1 the contract: `approach.ts`, two new rungs, the pure plan and camera | accepted, not started | "Build travel between places" |
| GZ2 kernel read `place_chain` with honesty marks (own golden area) | accepted, not started | kernel |
| GZ3 the journey port on System 3D, the globe and the Land ground; the player in the shell; "Zoom in from System" to a site | accepted, not started | "Build travel between places", with chart room and renderer |
| GZ4 the short form on the place line, the picker and "Chart this galaxy" | accepted, not started | "Build travel between places", with the selector thread |
| GZ5 Galaxies nearby part 1: centred on a galaxy, two reaches, 3D and True, type pictures from `galaxies_seen`, HUD, the galaxy's card | accepted, not started | galaxy charts and histories, with the renderer |
| GZ6 Galaxies nearby part 2: home's satellites on their orbits with Play, streams, real charts for the biggest neighbours, the web's threads, the group's edge | accepted, not started | galaxy charts and histories, with the kernel |
| GZ7 the Local stars rung: the home cell in 3D and True, the dive through the disc | accepted, not started | galaxy charts and histories, with the renderer |
| GZ8 the upper ports (web cube centred on home, galaxies nearby, Galaxy 3D, local stars): the whole grand zoom | accepted, not started | "Build travel between places", cosmic web view |
| GZ9 stops in full: captions, the width line, the target ring, free look and return | accepted, not started | "Build travel between places" |
| GZ10 where it is offered: bookmark rows, postcards, the Playbook Show step, the players' side, the clean output link | accepted, not started | Run desk, postcards |
| GZ11 small devices and film: lighter and Chart-style journeys, bench rows, capture by journey time, the shipped preset | accepted, not started | "Build travel between places", quality tiers, trailer |
| GZ12 home in the web: the kernel chooses the home halo from the catalogue once per universe and stores it; F37 and two stand-ins retired (core golden) | accepted, not started | kernel (core lane), galaxy charts and histories |
| GZ13 a star slot for every system at adoption and import; "Grow a system here" wired to `from_star` | accepted, not started | kernel or its hand-off thread, galaxy charts and histories |
| GZ14 galaxy tilt from the web's tides (citations checked first) | accepted, not started | kernel, citations |
| GZ15 galaxy drift from the web's displacement; redshifts with peculiar motion | accepted, not started | kernel |
| GZ17 the gazetteer: places and names worked out once at the defined time and kept with the campaign; placeholders fill in when looked at | accepted, not started | kernel, Run desk, names |
| GZ16 the Motion layer: one arrow mark with a key, the Motion switch, system level first | accepted, not started | "Build travel between places" and each view's owner |

## Next
- Builders to be opened by the coordinator (asked 25 Nov 12:50 UTC): the kernel thread takes GZ0's kernel checks, GZ12, GZ13, GZ2, GZ15, GZ14 in its lane; "Build galaxy charts and histories" takes GZ5 to GZ7; a new "Build travel between places" (Opus) takes GZ1, GZ3, GZ4, GZ8, GZ9, GZ11, GZ16.
- A separate design task, what every display looks like on the Explore, Build and Run desks (Alex's ask), was asked of the coordinator; the note's 5.9 is this track's input.
- GZ0's first jobs, not verified by the design thread: whether the web cube and the halo cell put a galaxy in the same place; which place each consumer (life history, the sky, the events op) passes for a campaign system today; what the reads marked "to measure" in the note's section 10 cost.
