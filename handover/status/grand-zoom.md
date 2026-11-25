# Status: the grand zoom and travelling between places (track GZ)

Owner thread: "Design the grand zoom" (Fable, design only, 25 Nov 2026) for the note. No builder thread yet: the note's section 13 proposes "Build travel between places" (Opus) for GZ0, GZ1, GZ3, GZ4, GZ8, GZ9 and GZ11. Until one exists, each package below belongs to the thread named in its row, and that thread records it in its own status file too.
Design: `design/SSE4-grand-zoom-design.md`, **PROPOSED 25 Nov 2026, not yet accepted by Alex**. It has 18 numbered questions (section 14; 13 to 18 added after Alex's follow-up of 25 Nov 12:22 UTC on tying the levels together and a velocity HUD). No builder starts on a feature before his yes; GZ0 (checks and timings only) needs no answer. Mockup in project files at `sse4/handover-screens/GZ-screens/` (`grand-zoom-mockup.html` and screens; painted stand-ins, not SSE4 output).

What it is, in one line: a change of place can be travelled instead of cut. The use of a bookmark, postcard, Playbook Show step or place link gains a "way in" (`sse.approach/1`: where the journey starts, stops or one smooth zoom, HUD, words, pace), each level keeps its own view, and at each change of level two views cross-fade while both zoom onto the same target (the one new fudge, GZ-F1). The grand zoom is the longest way in, from the cosmic web to a site; the short form makes every step on the place line a zoom. The note also says what "Galaxies nearby" needs to become a proper Local Group view (its section 7) and adds a "Local stars" rung (section 8).

Feasibility, as told to Alex 25 Nov: feasible. The levels are one computed chain (web, halo, galaxy, ring, star cell, star, formation by `from_star`, world, site) with three gaps: our galaxy is the median halo, not one of the web's catalogue (F37); systems not grown from a drawn star have no cell; the screens do not pass the place along. The note's section 6.3 closes them, each computed once (GZ12, GZ13), adds galaxy tilt from the web's tides (GZ14) and drift from its displacement (GZ15), and section 8A adds a Motion layer to the HUD (velocity and pull arrows with speed, Alex's ask; GZ16). Also missing for the zoom itself: a camera carried across a level change, one read for a place's chain, a 3D Local Group, two rungs on the place line.

Nothing is built. GZ is a code only, never shown in the product.

| Package | State | Owner (proposed) |
|---|---|---|
| GZ0 checks and timings (read costs on desktop and phone, one position per galaxy at every level, the orientation chain) | proposed; needs no answer | "Build travel between places" |
| GZ1 the contract: `approach.ts`, two new rungs, the pure plan and camera | proposed | "Build travel between places" |
| GZ2 kernel read `place_chain` with honesty marks (own golden area) | proposed | kernel |
| GZ3 the journey port on System 3D, the globe and the Land ground; the player in the shell; "Zoom in from System" to a site | proposed | "Build travel between places", with chart room and renderer |
| GZ4 the short form on the place line, the picker and "Chart this galaxy" | proposed | "Build travel between places", with the selector thread |
| GZ5 Galaxies nearby part 1: centred on a galaxy, two reaches, 3D and True, type pictures from `galaxies_seen`, HUD, the galaxy's card | proposed | galaxy charts and histories, with the renderer |
| GZ6 Galaxies nearby part 2: home's satellites on their orbits with Play, streams, real charts for the biggest neighbours, the web's threads, the group's edge | proposed | galaxy charts and histories, with the kernel |
| GZ7 the Local stars rung: the home cell in 3D and True, the dive through the disc | proposed | galaxy charts and histories, with the renderer |
| GZ8 the upper ports (web cube centred on home, galaxies nearby, Galaxy 3D, local stars): the whole grand zoom | proposed | "Build travel between places", cosmic web view |
| GZ9 stops in full: captions, the width line, the target ring, free look and return | proposed | "Build travel between places" |
| GZ10 where it is offered: bookmark rows, postcards, the Playbook Show step, the players' side, the clean output link | proposed | Run desk, postcards |
| GZ11 small devices and film: lighter and Chart-style journeys, bench rows, capture by journey time, the shipped preset | proposed | "Build travel between places", quality tiers, trailer |
| GZ12 home in the web: the kernel chooses the home halo from the catalogue once per universe and stores it; F37 and two stand-ins retired (core golden) | proposed | kernel (core lane), galaxy charts and histories |
| GZ13 a star slot for every system at adoption and import; "Grow a system here" wired to `from_star` | proposed | kernel or its hand-off thread, galaxy charts and histories |
| GZ14 galaxy tilt from the web's tides (citations checked first) | proposed | kernel, citations |
| GZ15 galaxy drift from the web's displacement; redshifts with peculiar motion | proposed | kernel |
| GZ16 the Motion layer: one arrow mark with a key, the Motion switch, system level first | proposed | "Build travel between places" and each view's owner |

## Next
- Alex's answers to the 18 questions (note section 14).
- Then open "Build travel between places" (Opus) and start GZ0. Two things were not verified by the design thread and are GZ0's first jobs: whether the web cube and the halo cell put a galaxy in the same place, and what the reads marked "to measure" in the note's section 10 cost.
