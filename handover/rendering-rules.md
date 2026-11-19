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
  another body (a moon) is left to that body's frame. There is no planet-frame chart yet (no fixture has moons).
- **Players' chart** (`player-stack/PlayerChart`) draws only the scope's direct children (`topLevel`), so it is one
  frame too; its optional drift is presentation (an angle, kept modulo 360), not accumulated position.
- No 3D or GPU view exists yet. The first one (renderer packages) must follow rules 3 and 4 and prove it with a test
  that zooms from patch to surface and checks a fixed body's screen position is stable to a pixel.
