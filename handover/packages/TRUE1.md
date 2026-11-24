# TRUE1: plumes above the limb, part 1 (the kernel's plume facts)

Track TRUE-surface (thread "Build plumes, frosts and small bodies"), third V3 audit item R10 (`handover/design/SSE4-v3-gaps.md`, third audit).

## What
`crates/sse-surface/src/plumes.rs`: a world's vapour plumes as a chain of process strengths read from its own heat and ice, not a class:
1. ocean under the ice: water layer thickness against the conducting shell, smooth in (H/d)^4;
2. cracks reaching it at the pole: `shell_activity` at the polar heat flow (tidal share focused 4x, radiogenic not);
3. speeds: vapour at the adiabatic limit from water at its triple point (1004 m/s), grains at a share (Hedman 2009);
4. height: h = R v² / (v_esc² - v²), or escape (feeds a torus);
5. rate: a share of the heat over the latent heat (calibrated on Enceladus).
Which pole is a seeded draw (stream `surface.plumes.pole.v1`).

Wired into `resolved_system`: each world's `surfaces[]` record carries `plumes` when it has any (left out otherwise), so System True and the cards can read it without a new op. Agreed with "Build SSE4 proven core" (24 Nov): plumes.rs is this track's; facts on the body record are a later step if the cards want them.

## Results (surface fixtures)
- Enceladus: strength 0.77, ocean 0.95, polar shell about 10 km, grains rise about 86 km, gas escapes (feeds the E ring), about 256 kg/s (measured 200 to 300).
- Europa: strength 1.0, grains about 6 km (too low to see at system scale), about 35 t/s (likely high; fudge row).
- Triton: the fixture's tidal heat (0.1 W/m²) gives ocean plumes of about 9 km. Its real geysers are solar-driven nitrogen, a different engine, not here yet.
- None for Callisto, Ganymede, Titan, Io, Miranda, Earth, Mars, Moon, Jupiter.

## Golden
`tests/golden/contract.jsonl` line 155 (the Kestrel `resolved_system` case) changes: four generated icy worlds there now carry `plumes` (an ice-covered ocean world and an ice shell erupt strongly; two cold lids weakly, strength 0.04). Reason: new plume facts on the surface records, on purpose.

## Checks
`scripts/check.sh`. Tests: `plumes_tests.rs` (Enceladus against measurements, cold or dry worlds none, monotonic in heat with no cut-off, the pole a repeatable draw).

## Next
TRUE2: draw them above the limb in System True (renderer `plumes.ts`, a GpuItem field after renderer PR 416, filled in `trueLook`). Then hot glow in Visible (`hot-glow.ts`, written), frosts and tholins (data sheet MP33 and MP34), small bodies and comets, lava tube openings on the ground.
