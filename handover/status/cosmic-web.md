# Status: Cosmic web view

Owner thread: "Build cosmic web view" (from 24 Nov 2026; it took the web over from "Build photoreal renderer"). Covers: V9w and what follows it for the web cube, the 3D and True looks of the foam map.

One row per package: update it in the same pull request as the package. Logs are in `handover/packages/`. Index and standing rules: `handover/START-HERE.md`.

Alex's ruling (23 Nov 2026): the cosmic web stays close to real; the void fraction goes toward the measured 77% as a cited editable number; the view must read as the 3D spiderweb of filaments, nodes and voids, not a "wrinkled rug". Lean into the physics: the kernel's numbers drive it, every shortcut is in `docs/fudges.md`.

Boundaries agreed 24 Nov 2026:
- This track owns `packages/renderer/src/web-cube.ts` and its test, `packages/views/foam-map/src/Web3D.svelte`, `crates/sse-cosmos/src/web_light.rs` and its tests, and the cube parts of `web_slice` in `crates/sse-kernel/src/cosmos.rs`.
- "Build photoreal renderer" keeps the rest of `packages/renderer` (ask it before touching `gpu.ts`).
- "Build galaxy charts and histories" owns `FoamMap.svelte` (flat slice, time scrub, ledger) and the cluster map; this track changes FoamMap only in small hunks that feed Web3D, with a heads-up first.
- The kernel thread owns the rest of sse-cosmos and sse-kernel; G2's `Haloes::cell` stays bit-identical (tested).

| Package | What it does | State | Log |
|---|---|---|---|
| V9w | Voids fill 77% of space (cited, editable `void_fraction`), one web for slice, cube and ledger; the foam map turns a cube of it in 3D and True | done (PR 368, by the renderer thread); voids hold about 40% of the matter against 15% measured (fudge) | packages/V9w.md |
| V9w2 | Light from haloes: True shines by the cube's galaxies, G2's haloes from 10¹¹ M☉ (pack `web_light.floor_msun`, the floor grows with the cube's volume) moved by the cube's own truncated Zel'dovich step, each lit by its G3 galaxy's stars and star formation then (old light warm, young blue); HUD adds the dark matter faintly; kernel `web_slice` with `cube` and `haloes` | in review (PR 453) | packages/V9w2.md |
| V9w3 | The web reads as a web: parcels in 3D inked by their density today (one over their room), each dot drawn inside its cell as the step deformed it (threads and walls), big galaxies, groups and clusters marked in 3D, Play runs on the parcels | in review | packages/V9w3.md |

## Next
- Void galaxies bluer and fainter by environment (Hoyle et al. 2005).
- A contract case for `haloes: true` in `tests/golden/contract.jsonl`, through the golden lane (the coordinator keeps the order).
- The voids' matter share toward 15% (NEXUS+ or a 2LPT step), a kernel change in the golden lane.
