# VP5: sharpen at rest, and enlarging on Light

Design: `handover/design/SSE4-visual-polish-design.md` 8.4 and 5.1 (accepted 24 Nov 2026, question 4: sharpen at rest automatically in every True view). Thread: "Build quality tiers and speed probe".

## Part 1
- `still.ts` `RestClock`: still for 500 ms (`REST_AFTER_MS`), then frames 0, 1, 2, ... with Halton (2, 3) sub-pixel jitter (frame 0 unshifted), up to `REST_FRAMES` by tier (Light and High 16, Ultra and still 32, a canvas one supersampled frame, words none), stopping at 3 s (`REST_BUDGET_MS`) with the slowest frame as the guess. Any drawn frame (`moved`) drops back at once. Clock-free; tests `test/still.test.ts`.
- `still.ts` `RestLayer`: a 2D canvas over the live one keeping the running average (frame k weighs 1/(k+1)), shown as it sharpens, or under reduce motion only when sharp, fading in over 400 ms; hidden at once on input. `data-rest` says the state.
- The Body globe (`runtime.ts`): when not spinning, after any frame the rest wait starts; at rest it draws the recipe's still passes (finer mesh, more air samples) at full size with the jitter, on the GPU, or one canvas frame at twice the size. `GlobeStatus.rest` and `rest_frames`; readout source `render.rest`. Check in `tests/ui/render.e2e.mjs` (the globe reaches `data-rest="sharp"` and keeps its blue ocean).
- `enlarge.ts`: `enlargeNode` (Catmull-Rom in nine bilinear taps, kept non-negative) and `sharpenNode` (an RCAS-style contrast-adaptive sharpener after FSR1's published method, written fresh; lobe limit a quarter less a sixteenth, green-channel lobe, clamped to the five pixels' range so nothing rings), with the same maths in plain numbers (`rcasLobe`, `rcasValue`, `catmullRom`, tests `test/enlarge.test.ts`). `tests/ui/enlarge.e2e.mjs` builds a test page into a scratch folder and checks in Chromium, on WebGL2 and WebGPU, that a hard edge enlarged four times is 2 px wide against bilinear's 4, and that sharpening keeps it within range.

## Part 2 (next)
Accumulation inside the shared finish for system True, Land, ships and stations (agreed with the renderer thread: `jitter?` and `accumulate?` on FinishFrame, the finish keeps the half-float running mean before glare and exposure; the jitter on the camera in gpu.ts), and the enlarging nodes in the finish's output when the render scale is below 1.
