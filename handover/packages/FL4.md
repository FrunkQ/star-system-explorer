# FL4: said and checked

Status: built 28 Nov 2026, thread "Build the shared flourish system" (Opus). Design: `design/SSE4-visual-flourish-design.md`
section 4.6, rule 7 of section 3, and 8A. No kernel, pack or golden change. Nothing new is said on screen until a view
puts a layer on the schedule (FL3, FL5): the words and the board are ready for them.

## What it adds

- **The note's clause** (`ui-contract/src/settle.ts` `standInClause`): one clause from the kind drawn and what moves
  the layer, empty while the proper picture is up. "Jupiter's clouds are carried along by their winds between
  pictures. The full picture settles when the clock stops." Likewise for a long exposure, a coarse picture ("on the
  way" in a zoom), a held one, a seam and a hint.
- **`motionWords` takes the clauses** (`picture-clock.ts`, a third argument, default none): added at the end of the
  note, also while the clock is stopped and a layer has not settled yet. With none, the note is as before.
- **The readout's lines** (`SettleBoard`, the page's `settleBoard`, `settleLineWords`): a view reports each layer's
  state, last build time and size; the performance readout (`shell/src/PerfReadout.svelte`) shows one line a layer,
  biggest first, and shows nothing while no layer reports. ("layerWords" was taken by the names gallery, so the words
  function is `settleLineWords`.)
- **The list against the register** (`ui-contract/test/stand-ins.test.ts`): every use in `stand-ins.json` has its
  register row; every register row that names a declared use (the words "declared use `id`") is in the list; kinds are
  the design's, no use draws a Hint (that needs Alex's yes per case), each says what moves it or why nothing does.

## Tests

`stand-ins.test.ts` (5, in `ui:test`). Red first (28 Nov, in the container): FL-F2's file removed (the register test
red); FL-F2 naming a use the list lacks (the reverse test red); the clause said at rest (the clause test red).

## The settle kit on AC-ProArt (owed from FL2)

Run 28 Nov 2026 on AC-ProArt with the installed Chrome (`SSE_CHANNEL=chrome`), in a fresh clone, FL2's head
(dd086827). **All 24 cases passed on both backends, on both of its GPUs**; every bit-exact case equal to the bit, the
worst difference 0.0013 (1e-4 of a pixel, the ten-day jet).

- **Which GPU.** Chrome alone picks the AMD Radeon 890M (the laptop's built-in GPU, hardware): WebGL2 "ANGLE (AMD,
  Radeon 890M, Direct3D11)", WebGPU amd / rdna-3, headless or not, even asked for high performance. With Chrome's
  `--force_high_performance_gpu` it is the RTX 4070 (WebGPU nvidia / lovelace). The check now takes
  `SSE_GPU=high` to add that flag, so a real-GPU run means the 4070 when asked.
- **Bench, 512 × 256 with upload and read-back:** 890M WebGL2 carry 33.3, exposure 33.8, fade 33.3 ms; WebGPU 12.2,
  13.8, 11.2 ms. RTX 4070 WebGL2 32.1, 33.0, 32.8 ms; WebGPU 13.1, 13.5, 10.6 ms. WebGL2's 33 ms on both is two
  frames at 60 Hz: paced by the read-back, not a measure of the GPU. The pieces themselves cost far less; a timing
  inside a frame comes with FL3, where the kit runs in the view's own material.
- **Screens:** `sse4/handover-screens/FL2-screens-proart/` (890M) and `FL2-screens-proart-rtx4070/`, each
  `settle-kit-webgl2.png` and `settle-kit-webgpu.png`.
