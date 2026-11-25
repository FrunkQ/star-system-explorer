# Status: Aurorae and night lights (drawing)

Owner thread: "Build aurorae and night lights". Covers the drawing side of track VP-AU (visual polish design `handover/design/SSE4-visual-polish-design.md` section 16, accepted by Alex 24 Nov 2026 19:12 UTC, "yes to all", "Long exposure" the default): aurora shells, curtains, aurora from the ground, other bands, airglow drawn; VP12 (city lights, lightning); night-side lights on constructs (cities on the ground and objects in space, Alex 24 Nov 19:13) and the glow of a drive's plume; item 9 of the SSE3 render audit (city lights). Colour by speed is the shift pass (O7), not this track; every light here passes through it like any other.

Only the owner thread edits this file. Logs are in `handover/packages/`. Index and standing rules: `handover/START-HERE.md`.

**The split (agreed with "Build light and nebulae", 24 Nov 2026 19:13 to 19:15 UTC):** this track draws; the light thread supplies the facts as MF9 part 2 (VP-AU1 to AU4, rows under MF9 in `status/light.md`). The data sheet side takes the "Light it gives off" card section on element and substance cards (question 15); not this track's. Part 2 only adds fields to `read_field`'s `aurora`, on each oval and footprint, and later on patches and airglow:
- `power_w` (total, both hemispheres) and `power_by_hemisphere` [north, south]; `width_deg` (full width, equatorward to poleward edge);
- `emissions`: `{line, matter, wavelength_m, power_w, share, peak_height_m, thickness_m (full height range above half peak), colour {hex, linear_srgb}}`, colour a chromaticity with max channel 1 (as every kernel colour), brightness in `power_w`; and `colour`, all emissions blended;
- footprints give `lead_deg` (the angle the spot runs ahead of its moon); the moon's longitude is taken from the system's positions;
- the diffuse glow gets `peak_height_m` and colour too.
`aurora.ts` reads all of these already (tested against the shape) and draws the kernel's lines the moment they arrive.

**Rule 5 (section 16.1):** a look is switched on by kernel numbers, never a name or class; it scales with them; it stays off where the physics says so; each package is proven on a Solar System body, a world outside it, and a world where it stays off. Aurora proving worlds: `tests/fixtures/aurora/proving-worlds.json`.

Files: `packages/renderer/src/aurora.ts`, `gpu-aurora.ts`, `gpu-aurora-sky.ts`, their test `packages/renderer/test/aurora.test.ts`, the `aurora` pass in `passes.ts` and the true-colour recipe's `aurora` entry. Small additive hooks, agreed as seams: an optional last argument `aurora` to `gpu-globe.ts` `draw`, `aurora` in `runtime.ts` `GlobeRunOptions` and `aurora_note` in its status, `aurora` in `photograph.ts` options, the aurora read in `packages/views/body/src/Body.svelte`, and for the ground view an optional `aurora` in `gpu-surface.ts` `SurfaceFrame` and the aurora read in `packages/views/surface/src/Surface.svelte` (renderer and chart room threads told). `gpu.ts` untouched.

| Package | State | Log |
|---|---|---|
| VP-AU5 part 1, glowing shells on the Body globe: each line of each oval a thin shell at its own height round the magnetic poles, limb-stacked, drowned by daylight; long exposure by default (one stated lift, ratios kept), Eye true brightness and grey below colour vision; oxygen stand-in only where the air holds oxygen, marked on screen | in review (PR 469) | packages/VP-AU5.md |
| VP-AU5 part 2: moon spots and tails from the system's moon positions and `lead_deg`; shells on system True (V10a) and System 3D worlds; the Exposure control moves to the shared view controls when VP1 lands | next | |
| VP-AU6 curtains through the detail ladder (VP6), pulsing under the three-flashes cap | queued (needs VP6) | |
| VP-AU7 aurora from the ground on the Land screen: each line a shell met by the sight line, ring times slant path, dimmed by the air below, arcs low toward the pole from mid latitudes, overhead under the oval; long exposure lift on the brightest point of the sky | in review (PR 469) | packages/VP-AU7.md |
| VP-AU8 ribbons along field lines; volumes | queued (needs VP4) | |
| VP-AU9 other bands (UV, IR, X-ray) through the Band control | queued (needs part 2) | |
| Storm aurora drawn on the date an ejection arrives (VP-AU3 facts) | queued (needs part 2) | |
| Airglow drawn (VP-AU4 facts): the green limb line on oxygen worlds | queued (needs AU4) | |
| VP12 night side: city lights from population facts, lightning from the weather op | queued | |
| Construct lights: lamps from emit Conditions (sodium or white, with power) on the ground and in space; a drive plume's glow from `sse_sense::fog::signature` (RK14, PR 406) | queued | |
