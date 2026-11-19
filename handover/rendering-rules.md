# Rendering rules for every view (standing)

Alex, 18 Nov 2026: "full local orbits and layered coordinate systems to avoid jiggle as renders run out of resolution."
No view may show precision jiggle at any scale, from a lander's ground track to galactic filaments.

1. **Positions live in the kernel, in f64, local to their parent.** A moon's orbit is about its planet, a planet's
   about its star (or barycentre), a star's within its patch, a patch within its region. The kernel never hands a view
   a "world" coordinate that adds a tiny offset to a huge one.
2. **Views compose frames, they do not flatten them.** A view draws in one frame at a time (the star's frame, the
   planet's frame, ...) and nests the next frame's contents relative to that frame's origin. Children of another
   frame are drawn in that frame, never in this one's numbers.
3. **The GPU only sees small numbers.** Everything sent to WebGL/WebGPU (or an SVG, which browsers draw in 32-bit
   floats) is relative to the camera or the current frame origin, computed in f64 on the CPU first. Combined with
   reversed depth for 3D (SSE4-surfaces-and-rendering.md), this is the camera-relative rule.
4. **Zoom changes the frame, not the scale factor alone.** When a view zooms past what its frame's numbers can carry,
   it hands over to the child frame (re-centres on the body) rather than multiplying further.
5. **Animation reads the kernel's time.** A drawn position at time t comes from the kernel's orbit for t (or a
   presentation drift the view labels as such), never from accumulating small per-frame steps.

## Where the current views stand (checked 18 Nov, UI track)
- **GM system chart** (`views/system-chart`) is the star's frame: 2D SVG, log-radial screen scale computed in f64
  per frame, so SVG gets pixel-sized numbers. It now draws only the star's own children; a body whose `parent` is
  another body (a moon) is left to that body's frame. A planet's frame opens from the chart (packages/planet-chart.md).
- **Players' chart** (`player-stack/PlayerChart`) draws only the scope's direct children (`topLevel`), so it is one
  frame too; its optional drift is presentation (an angle, kept modulo 360), not accumulated position.
- **System 3D** (`views/system-3d`, V6) draws one frame at a time through `packages/renderer/src/frames.ts`: positions
  local to their parent in f64, camera subtracted in f64, float32 only for the GPU, and the frame is handed over on
  zoom. `packages/renderer/test/frames.test.ts` is the pixel-stability proof. It zooms from 3×10^21 m to 10 m onto a
  lander on a moon, through five frames, and holds it within 0.0001 px. Any new GPU view uses `frames.ts`, not its
  own maths.
