---
name: sse4-galaxy-histories
description: Dark matter and galaxy histories design (track GH, ACCEPTED by Alex 22 Nov 2026): shape, rulings, packages DM1-DM6 and GH0-GH12
metadata:
  type: project
  modified: 2026-11-22T13:12:16.577Z
---
Thread "Design dark matter and galaxy histories" (Fable, 22 Nov 2026). Design: /mnt/project-files/sse4/SSE4-galaxy-histories-design.md, repo handover/design/ (PR 162, docs only). Track GH in the build plan. ACCEPTED 22 Nov 13:15 ("I agree with all the other recs"); section 14 holds the rulings (PRs 162, 167, 168).

Alex's asks (22 Nov 12:39 to 12:47): dark matter on the data sheet "listed as theoretical" with "tweakable parameters like any other universal constant"; mass and energy balance of filaments and galaxies; galaxies with very much or very little dark matter (stripped in collisions); dark energy in voids (he offered a frig; design says none is needed, it is already in expansion and growth); the Milky Way "battle-worn"; galaxies resolved "a bit like planets"; patches of ancient stars and pristine gas (Pop III); globular clusters, dwarf and "micro" galaxies orbiting the host; "the galaxy effectively has its own accretion model". Reference video: Anton Petrov, "Gaia Data Rewrites Everything We Knew About Galaxy History" (Gaia-Sausage-Enceladus, Loki, early disc, LMC heating the hot halo).

Shape: a fourth sheet mark "theoretical"; Cosmos group on Constants (measured cosmic parameters move from base-cosmology); matter ledger per cell and galaxy; halo structure and rotation curve block (retires typed 220 km/s and similar); ONE assembly tree per galaxy (G2 major mergers as pins, Y4 part 2's sub-halo draw as fill, recursive); each progenitor's galaxy by G3; four processes after infall (sinking, tidal stripping, gas stripping, shock heating) on a stored test-body orbit (own leapfrog, not n-body); remains as density formulas in the patch envelope; clusters as one continuum; Milky Way history as graded pins (GH7, the parked "Milky Way mergers" question). Packages DM1 to DM5, GH0 to GH12. Most citations in it are from memory: GH0 must check them.

Ruling, Alex 22 Nov 13:10: no MOND or altered-gravity mode at all, and the Cosmos page must SAY SSE4 takes the particle route and why (he supplied arXiv:2610.04048, arXiv:2003.12832, doi:10.1103/3dzh-d8f5; note the first two test G varying in time, not MOND). The "bone": constants may carry a time law so G can vary through time as a what-if (package DM6, PR 167).

**Why:** dark matter was only implicit (Ωm minus Ωb), satellites had no orbits or stripping (fudge F39), and mergers and satellites were two separate draws.
Ruling, Alex 22 Nov 13:14: wants "cool realism renderer work" for damaged galaxies: twist, ripples along the disc, glowing dwarfs, each collision leaves its mark, fast ones punch a hole, slow ones stay orbiting. Design 9.4: each merger response writes a dated Mark the renderer draws (GH3 emits, GH9 draws).

**How to apply:** builders can start: GH0 (citation check, timings) and DM1 first. Galaxy chart mockup done and accepted (Alex 22 Nov 15:42, "PERFECT"): artifact "SSE4 Galaxy chart mockup" https://claude.ai/artifact/Qo5QqCCfJV6Y7EZzL1bDmn, the reference for GH8. Alex also ruled: every diagram gets zoom, slide and tilt, with a Free view beside the fixed "Seen from" views (interface-style.md rule 11). GH0 also walks a galaxy type coverage table (every identified type must fall out of processes and Marks). GH0 timing: orbits 8x dearer than estimated; GH2's three cuts accepted with conditions in design section 10. Honest caveat told to Alex: in True style a galaxy's scars are faint; Chart style inks them. Related: [[sse4-matter-pack]], [[sse4-night-sky]], [[sse4-processes-not-classes]]
