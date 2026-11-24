---
name: sse4-visual-polish
description: SSE4 track VP (visual polish, closing the look gap to SpaceEngine): the design note, its state, Alex's steer of 24 Nov 2026, and what is still open
metadata:
  type: project
  modified: 2026-11-24T18:52:40.472Z
---

- Alex, 24 Nov 2026 18:39 to 18:43 UTC: aim as high as SpaceEngine ("shoot for the stars"), borrow the gloss others spent years on, "getting close would be a major achievement because we can always beat them on breadth"; add cheap flourishes (more swirls on a giant close up); heavy GPU techniques are welcome if gated on performance checks; long physics on the GPU is "not sure, but bear in mind".
- Note: `/mnt/project-files/sse4/SSE4-visual-polish-design.md`, repo copy `handover/design/` (PR 462), copy on Alex's PC at `C:\Development\sse4-project-files\`. Design thread "Design visual polish and flourishes" (Fable).
- State: **accepted by Alex 24 Nov 2026 18:57 UTC ("yes to all" ten questions)**; ruling logged in handover/decisions.md. Builders may take the section 12 packages.
- Core ideas: three grades for anything drawn (fact; drawn from the kernel's numbers; camera effect); one "detail ladder" for detail that arrives on approach; tiers B/H/U/S with a new `gpu-ultra` placed by a measured probe (probe.ts was designed in R1 but never built); sharpen at rest; ranked list of 24 items; packages VP1 to VP16 (photoreal), VP-A (giants and clouds), VP-P (plumes), VP-S (stars), VP-R (rings), VP-L (light).
- Findings in the code (main f95a334): system True draws 128 x 64 CPU pictures on black with no stars, no tone map, no bloom; five separate renderers, only the Body globe is under recipes and the governor; the kernel runs in one worker with no SIMD.
- GPU physics view (not a decision): GPU for drawing and previews now; kernel speed-ups first on the processor (worker pool, then wasm SIMD, both bit-identical); one integer-maths erosion experiment later.
- (The "from memory" citations were checked on 24 Nov: see the citation check line below.)

Related: [[sse4-renderer-track]], [[sse4-star-look]], [[sse4-performance-readout]].
- Addendum (section 16, PR 464), Alex 24 Nov 19:06: **rule 5, his own words and standing: no look is "fixed for the solar system"; every visual is a system switched on by kernel conditions (never a body name or class), scales with the numbers, and each package is proven on a Sol body, a non-Sol world and a world where it stays off.** "Auroras are key": one driver table (wind oval, spin oval, moon spots built in MF9 part 1; power, colours, heights, proton, storm, host-plasma kinds to add as MF9 part 2 = VP-AU1 to AU4, light thread), colours from the air's emission lines, heights computed from line lifetime against air density. Element editor does NOT have aurora colours yet: 27 lines in packs/base-light with free-text species; Elements screen read only.
- Alex 24 Nov 19:12: yes to all of 11 to 15, and **"Long exposure" is the default in every True view; "Eye" (honest single exposure, faint aurorae grey) is the choice**. One Exposure control in the shared view controls; the HUD says when faint light is lifted (note 16.7, decisions.md). Drawing thread "Build aurorae and night lights" started 19:12.
- Citation check (Alex's Consensus pass, 24 Nov 19:27) folded in as note section 17; full check with DOIs is `SSE4-visual-polish-citation-check.md` (project files, handover/design/, Alex's PC). Design changes from it: phase table per particle kind from the kernel (Mie from size and sheet refractive index; the Jendersie fit only for water 5 to 50 µm; ask K-e); Rhines scale from eddy speed not jet speed; gravity darkening exponent a block input; GPU float caution confirmed against the specs. Still from memory: the licences (Celestia, Stellarium, FSR 1, Bruneton, psrdnoise, hex-tiling, Shadertoy default) and a few papers, listed in 17.3.
