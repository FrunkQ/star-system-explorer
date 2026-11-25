# LR6: the lander on the ground

Track LR (status/landing.md). For the trailer's Land beauty shots (storyboard H) and for the table: the ship that came
down stands where it touched down, in frame from where the walk starts.

- Renderer half (agreed with the renderer thread, 25 Nov): `packages/renderer/src/surface-lander.ts` builds the stock
  lander (squat body on four splayed legs with pads, cabin, hatch, four floodlamps and a beacon) and places it about
  the eye; `LanderPose` as the renderer asked (`at_m`, `ground_m`, `heading_deg`, `height_m`, `model?`, `lights`)
  plus `lamp`, the lamps' radiance in the surface's units. gpu-surface.ts gains only `setLander` and a `place` before
  render; the hull is lit by the ground's own sun and sky terms (`paint.lit`), the lamps glow through the same
  exposure. No tone mapping, theme colours or rgba of its own. In Chart, 3D and Wireframe it is drawn in ink.
- Land (Surface.svelte): the lander's height from its mass (`landerHeightOf`, ui-contract); down, the walk starts three
  heights west of it, looking at it; a Lamps switch on the walk pad (off by default) lights them once the sun is near
  the horizon; with them lit the eye adapts to the lit hull (stars fade). The lamps ride in the pose, so the table's
  screens follow. `data-lander-drawn` (dark, lit) marks it for tests.
- `run-through.e2e`: the lander drawn on both screens, dark at midday, not lit at night until the switch, then lit on
  both; True-look screenshots `rt-6b`, `rt-6c` in project files `LR-screens/LR6/`.
- Fudge register row "The lander on the ground".

Not yet: the lamps' light on the ground (the surface shader takes no point lights; renderer), the construct's own 3D
model passed in as `model` (ship-mesh.ts builds one; Land has the party's construct address), and the lander in the
canvas tier. No golden change.
