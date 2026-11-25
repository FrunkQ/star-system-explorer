# LR6: the lander on the ground

Track LR (status/landing.md). For the trailer's Land beauty shots (storyboard H) and for the table: the ship that came
down stands where it touched down, in frame from where the walk starts.

- Renderer half (agreed with the renderer thread, 25 Nov): `packages/renderer/src/surface-lander.ts` builds the stock
  lander (squat body on four splayed legs with pads, cabin, hatch, four floodlamps and a beacon) and places it about
  the eye; `LanderPose` as the renderer asked (`at_m`, `ground_m`, `heading_deg`, `height_m`, `model?` (now `ship?`, LR6b), `lights`)
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

Not yet: the lamps' light on the ground (the surface shader takes no point lights; renderer) and the lander in the
canvas tier. No golden change.

## LR6b the party's ship's own model on the ground (25 Nov)

When the lander is the party's ship, it stands where it came down as its own designed model, not the stock shape.

- Seam (ship flow thread, 25 Nov): Land only reads the renderer's `designShip` and `shipMesh`; nothing of theirs
  changes. `LanderPose.model` became `ship?: LanderShip` (`{ mesh: ShipMesh, albedo }`, surface-lander.ts) and
  `lamp_m`, the lamps' radius in metres.
- Land (Surface.svelte), GM: the ship's numbers from the kernel (`shipRecordOf`, as the Ships and stations picture
  reads them), its fuels' storage (`readPropellants`) and the design kept with the construct (`construct.design`:
  style, seed, overrides; first style and seed 1 when none is kept). They go with the landing as
  `LandingShow.ship_model` (ui-contract landing.ts), since a seat reads neither the sheet nor the construct.
- Every screen designs the same ship from those and the base-ships pack, and surface-lander.ts turns the mesh so the
  ship's stern-to-bow axis is up (a turn of the axes, so faces keep their winding), at its own length, painted with
  the style's albedos and lit by the ground's sun and sky. The radiators are left off (stowed with the drive off).
  Four lamps sit on the hull's sides just above the drive and a beacon on the bow; the lamps keep the size the
  ship's mass gives the stock shape, and their spill on the hull falls with that size over the hull's, squared.
- The walk starts three ship lengths west. Beowulf (the Traveller far trader) in the spine style is about 600 m
  of truss: a needle on the horizon, as its torch drive's design makes it.
- `data-lander-ship` (the drawn ship's length in metres) marks it; `run-through.e2e` checks Beowulf is drawn at its
  own length on both screens. Screens in project files `LR-screens/LR6b/`.
- Fudge register row "The lander on the ground" now covers the tail-standing pose, no legs, stowed radiators.

Next (renderer ship-design.ts): landing legs, a stowed-radiator pose and a "lands on" (tail or belly) choice.
