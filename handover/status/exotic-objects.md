# Status: Exotic objects on screen

Owner thread: "Design exotic stars and black holes" wrote the design; no builder thread owns the track yet. Until one does, the builder of each package updates its row. Covers: XO (short for exotic objects; a package code only, never shown in the product).

One row per package: update it in the same pull request as the package. Logs are in `handover/packages/`. Index and standing rules: `handover/START-HERE.md`. Stage order: `handover/design/SSE4-coming-together.md`.

Design: `handover/design/SSE4-exotic-objects-design.md`, **accepted by Alex 24 Nov 2026** (rulings in its section 13; small open points 12 to 15 and, for the addendum in section 15, questions 16 to 21, each with a recommendation builders follow). Mockup and screenshots are in project files at `sse4/handover-screens/XO-screens/` (`black-hole-mockup.html` is a working picture: light traced from the real bending equation, the band switch with Composite, and the Hazard map for the hole, the Sun and Jupiter).

Nothing is built. The design's section 1 says what the kernel and renderer already hold (checked at main 67a5957): the thin disc and the spectrum shifts in `sse-light`, remnants with mass only, no light bending anywhere.

| Package | State | Proposed builder | Needs |
|---|---|---|---|
| XO1 gravity blocks and the bend table (design 3.2, section 8) | **next**: start here | Build light and nebulae | none |
| XO2 the feed descriptor on three fixtures, op `read_light_paths` (2.2, 3.5) | waiting on XO1 | Build light and nebulae | XO1 |
| XO3 the disc's light point by point; tilt and shifts into `read_light` (3.3) | waiting on XO2 | Build light and nebulae | XO2, O7 (built) |
| XO4 the `light-path` stage and the black hole in True, every band (2.1, 3.1 to 3.4, 7); was the renderer's R5b | waiting on XO1 to XO3 | Build photoreal renderer | XO1 to XO3 |
| XO5 phone path, table lookups in WebGL2 (7) | waiting on XO4 | Build photoreal renderer | XO4 |
| XO6 close-up, Seen from, Scale, Wireframe bent grid, time stretch chip (2.4, 3.7, 6) | waiting on XO4 | chart room, with the renderer | XO4 |
| XO7 spin: kernel reference integrator (tests only) and the shader pass (3.6) | waiting on XO4 | Build photoreal renderer, with light | XO4 |
| XO8 real feeds from binaries, outbursts, the donor star (2.2) | waiting on B4 | proven core, with light | XO2, B4 |
| XO9 the galaxy's centre as a body, from `sse-cosmos/src/feed.rs` (3.9) | waiting on XO4 | kernel thread, with the renderer | XO4, the G nucleus package |
| XO10 neutron stars and pulsars on screen, seen-from geometry, glitches (4.1 to 4.3) | can start on MF0's fixture pulsars once XO1 is in | light, then renderer | XO1, B3, MF11 |
| XO11 pulsar wind nebula and bow shape as nebula kinds (4.1) | waiting on XO10 | Build light and nebulae | XO10, O3 |
| XO12 magnetars: bursts and giant flares as events (4.4) | waiting on XO10 | light, with the renderer's Mag layer | XO10, MF11, O10 |
| XO13 shape from spin and pull, gravity darkening, for every body (2.3, 5.3) | waiting on B1 | light for blocks, renderer for mesh | B1 |
| XO14 the wind you can see; Wolf-Rayet stars (5.1) | waiting on B2 | light, then renderer | B2, O4 |
| XO15 colliding winds and dust spirals (5.2) | waiting on XO14 | light, then renderer | XO14, B4 |
| XO16 later: relativistic disc, thick and warped discs, cluster arcs, the plunge shot, pack benders | later | as they come | XO7 |
| XO17 Composite in the band switch, every view (6A) | **next**: needs nothing from XO | chart room for the control, renderer for the layering | V10b (built) |
| XO18 the Hazard map for any hazardous body, op `hazard_map` (3.8): Map and Profile, radiation belts from `read_field` with edges, gaps and latitude, Park and Dive marks | **next**: start on Jupiter, Saturn, Earth and the Sun, which need nothing from XO; compact bodies after XO1 | light for the op, renderer for the Map, chart room for the rail entry and card | MF7 belts and shielding, `read_light`, `read_activity`, the MF14a tracer (all built), the ship record |
| XO19 pulsating stars (design 15.2 to 15.6): strip, period from density, light curve, `star.light_now` with modulators, op `read_light_curve`, readers | **can start now** on fixtures and main-sequence stars, beside XO1; golden lane for stars inside a strip | light for the kernel part; renderer, climate and chart room for the readers | none; B2 for Cepheids and RR Lyrae in real galaxies |
| XO20 clocks (15.7): `clock_rate` widened, offset over the flight log, the clock chip, Hazard map marks, stream rate | waiting on XO1 | travel thread for the kernel side, chart room for the chip | XO1, voyages V8 (built) |

Before any block relies on a source in the design's section 14: those references were written from memory and must be checked (the citations work in `docs/citations.md`). Planned shortcuts are listed in the design's section 9; each goes in `docs/fudges.md` in the pull request that adds it. XO1 to XO6, XO17 and XO18 change no golden hash; XO8 onward add facts to bodies and go through the golden lane one at a time.
