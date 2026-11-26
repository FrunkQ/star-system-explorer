# RW16a: small bodies are lumpy, consistently, and a designer for them

Real catalogues design section 6A.1 (accepted by Alex 26 Nov 2026 18:08 UTC). Alex: "low sized objects need to be lumpy - consistently and we need the contact binaries and other interesting random asteroids made... a little asteroid and comet designer to do convincing small objects of every size would be useful." Thread "Build the asteroid and comet designer", 26 Nov 2026.

## What was built

### Kernel (crate `sse-small`, kernel `small.rs`): one resolve, more pins
- `Pins` (member.rs) gains the designer's pins, each optional: `pair` (now also "single"), `makeup`, `structure` (rubble-pile, monolith or relaxed), `spin_hours`, `battered` (0 to 1 within the structure's relief range), `size_ratio`. With every pin left out the answer is the field's own, byte for byte: no existing golden line moved.
- A pinned structure is taken whole: pinned lumpy keeps none of the fluid figure (so a 1,500 km rock can be kept lumpy), pinned relaxed keeps all of it. A pinned spin is not capped at the barrier. Nothing is refused ("implausible is not a limit").
- `takes.rs` (new): what a design would take, against what its material holds. Spin: the tensile stress across the middle of a spinning uniform sphere, rho R^2 (w^2 - 4 pi G rho / 3) / 2. Shape: the centre pressure (2 pi / 3) G rho^2 R^2 of a body past the round size. Said in words ("Turning once in 1.20 hours it would fly apart: holding it together needs 1.6 kPa across its middle, and loose rubble clings with only about 25 Pa"). The answers carry `takes` only when there is something to say. Material figures are new strength rows on the data sheet's `formation-rock` and `formation-water` mixtures, read at load (fudge SB-strength-sheet: recalled, one value for every size).
- `designed_field`: a one-member field for a system with no formation record (the Solar System as measured, imports), so `make_small_body` works everywhere; `made_from` says "designed". Fudge SB-designed-field.
- `makeup_for_density`, `mass_for_makeup`, `round_km`, `relaxed_strength_of` (the round-size rule, now one function).
- Kernel op `make_small_body` takes the pins. New op `small_shape` {system, address, diameter_km?, mass_kg?, ice?, spin_hours?, a_au?}: any body's lump from its address and the universe seed, single, with `round` and `round_km`; needs no formation record.
- Contract: six new cases in `tests/contract/areas/small-bodies.json` (a pinned contact binary spinning past its barrier, a lumpy 1,500 km body, a comet designed in a system with no record, Phobos's shape, a round 1,200 km icy body, a refused ask). Golden: `tests/golden/areas/small-bodies.jsonl` +6 lines, nothing else (area lane; no core line moved).

### Drawing: nothing below the round size is a sphere
- `ui-contract/src/small-shape.ts` (new): `shapeAsk` (a scene subject's size, mass and day), `readShapes` / `readShape` (asked once per kernel session), `shapeOf`. It decides no physics: the kernel says round or not.
- System True: `SystemChart.svelte` reads the lumps once per system after the surfaces, on every seat; `Body3D.figure` carries one; `System3D.svelte` passes it to the shared `trueLook` door. A world's own surface texture wraps the lump: `smallBodyMesh` now lays its grid out the way the globe's texture is laid out and gives it UVs (`gpu.ts` sets them when given, one line).
- Body globe: `GlobeRunOptions.shape`, `GlobeGpu.setShape`; `gpu-globe.ts` displaces its cube-sphere to the lump (largest radius at the globe's), with normals from the displaced faces tilted by the tiles' own relief. `renderer/small-body.ts` `smallBodyShape` is the same lump as True's. Fudge SB-globe-lump.
- Drawing paths: **system True** and the **Body globe** (GPU tiers) are covered. **Not covered:** the Body globe's canvas tier (still a disc), **Land** (`gpu-surface.ts`), the size comparison and the class gallery (they share `trueLook` but are not handed a figure yet).

### The designer (Workshop rail, "Small bodies")
- `views/small-designer/` (new view `sse.view.small-designer`), screen `documents/small-designer.gm.json`, wired through `shell/src/screens.small.ts` (one spread line each in `registry.ts` and `App.svelte`).
- Pick a kind (Asteroid, Comet, Contact binary, Pair); set Size (10 m to 1,000 km), Made of (Let it decide, Stony, Carbon-rich, Iron, Icy, or fine shares), Built as (Let it decide, Rubble pile, Solid, Pulled round), Battered, Spin, the second lobe's or moon's size, and distance from the star (a comet's closest approach). "Another like it" gives a new address, so a new lump made the same way. Each change asks `make_small_body` and the body is redrawn turning in True (`trueTurn`, a strip of one turn stepped by the page; still under reduced motion). Below: size, mass, structure, spin and the barrier, type and why, its pair's origin, a comet's activity at its closest, "What it would take", and its story.
- On each desk: **Explore** and **Build** list it under Workshop; it is a design to look at and nothing is kept on either desk. **Run**: not listed. Placing a design in a system is the editing track's Add list (next step below).

## Evidence
- `cargo test -p sse-small`: 23 pass, 5 new in `design_tests.rs` (the same lump every time and another by address; nothing from a boulder to 250 km comes out a sphere, absolute: axis ratio under 0.97 or relief over 1%; every pin taken as given; a 1 h rubble pile and a lumpy 1,500 km rock are kept and say what they take, the rock's centre at 0.4 to 0.7 GPa; a measured density reads as ice, rock or metal).
- `cargo test -p sse-kernel small`: 3 new (pins reach the body and `takes` comes back; a comet designed where no formation ran; `small_shape` gives Phobos a single lump, the same twice, another for another address, and a 1,200 km icy body round).
- `packages/ui-contract/test/small-shape.test.ts`: the Solar System entered from base-sol; every body under 300 km gets a lump, Earth, the Moon, Jupiter and Pluto stay round, and a second kernel gives the same lumps. Red with the fix removed: without `small_shape` the shapes map is empty and the first assertion fails.
- `tests/ui/small-designer.e2e.mjs` (desk and phone): opens the designer from the Workshop, the turning body is drawn, a contact binary at 1 hour as a rubble pile says it would fly apart, a comet near its star is awake, and the state hash does not move.
- Screenshots: project files `sse4/handover-screens/RW16a-screens/` (asteroid, a fast contact binary, a comet; desk and phone).

## Caveats (honest state)
- The Body globe lump is **not verified on screen**: in headless Chromium Phobos's globe stayed on "Building the surface of Phobos..." with no page error, so the e2e does not check it. Not yet checked whether main does the same. Eyeball on a GPU: open Phobos, Body, the globe should be a lump.
- Icy moons near the round size (Mimas, Enceladus, Miranda) come out fairly elongated (Enceladus about 405 x 290 x 205 km; the real one is nearly round). The cause is sse-small's linear blend between the irregular and the fluid figure at relaxed strength 0.3 to 0.45, plus an inferred ice share of 0.34. Not changed here; measured shapes for catalogued bodies would fix Sol, and the blend belongs to the small-bodies track.
- Phobos comes out 14 x 13 x 11 km against the real 13.5 x 11 x 9.

## Next
- Place a design: "Add to this system" from the designer, as a body of kind asteroid or comet with its pins kept as its recipe (with the editing track: the Add list and `Clip`), and the same designer opening any existing member to adjust it as a deed.
- Measured shapes for catalogued bodies (axes, two-lobe models) and light-curve bounds (6A.1 "Catalogued bodies too"): waits on the catalogue format (RW thread "Build the catalogue format and reader").
- Land and the canvas globe; the size comparison and the gallery handed figures.
- Strength falling with size (Holsapple 2007) and the strength values read from the papers (fudge SB-strength-sheet).
