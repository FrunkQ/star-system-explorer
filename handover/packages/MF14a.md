# MF14a part 1: the Mag layer's shared line tracer

Magnetic fields design (`design/SSE4-magnetic-fields-design.md`), sections 5, 8, 9 and 9B, package MF14a, renderer
side. Alex accepted the Field layer mockup on 22 Nov. On the same day he ruled that the interface label is **"Mag"**:
"Mag layer" for this layer and "Mag" in the band switch for MF14c. Kernel names (`read_field`, `field.source`) are
unchanged. Section 8 says lines are drawn in the views by "one shared, tested tracer": a line is a picture of a
fact, not a new fact. This package is that tracer, plus the data the Chart and 3D styles draw from it.

## What was built

`packages/renderer/src/field.ts`, exported from the renderer's index:

- **The field from coefficients.** `fieldAt(model, point)` gives B in tesla from Gauss coefficients g and h. They are
  Schmidt semi-normalised, as IGRF and the giant-planet models publish them (Winch et al. 2005). Any degree works.
  - The functions come from `schmidt`'s recursions; scratch tables are reused, and there is no trigonometry in the
    inner loop.
  - An offset dipole is a shifted expansion centre.
  - `dipole(radius, B0, tilt, toward, offset)` builds degree-1 coefficients from the facts the measured-fields
    fixture holds.
- **The wind's bubble.** Shue et al. 1998's surface comes from read_field's stand-off and flaring. Inside it, the
  traced field is the sum of three parts (`inBubble`):
  - **the body's own field;**
  - **the wind's squeeze**: an image of the dipole mirrored at the stand-off (Chapman and Ferraro 1931; Mead 1964).
    It doubles the field at the nose, the same factor of 2 that read_field's pressure balance uses, and lets no field
    through the nose;
  - **the wind's own field**, added uniformly (Dungey 1961, as Cowley 1973's vacuum superposition). This is what
    opens the polar caps. Without it, the squeezed field closes.
- **Tracing.** `traceLine` steps with fourth-order Runge–Kutta, each step 4% of the distance from the centre.
  - A line ends **closed** when it lands back on the body (the step's crossing of the surface is solved exactly),
    **open** when it leaves through the bubble, and **unfinished** when it runs out of reach.
  - Each point carries |B| and whether the field there points away from the body or toward it.
  - `traceField` seeds at equal steps of the dipole's flux, so line spacing means strength. It keeps the northern
    lines whole and only the open southern ones.
- **Chart cut** (`chartCut`, `cutPaths`). This is the Body screen's main view: a plane through the magnetic axis and
  the wind's direction. It gives the lines' paths, weighted by the field at their footpoint (open lines are drawn
  fainter), the bubble's edge, both axes and the tilt between them. The output is SVG path data for the views to ink.
- **3D layer** (`project3D`, `drawField3D`, `lineBuffers`).
  - The camera is orthographic. Lines are depth-tested against the body, so a line never draws across the globe it
    wraps round.
  - Each line has two inks, one where the field points away from the body and one where it points toward it.
    Brightness and weight follow the field's strength on a log scale, and open lines fade toward their far end.
  - `lineBuffers` gives the same data as typed arrays for a GPU line pass.
  - Inks are passed in by the view's theme: the renderer holds no colours.
- **Words** (`narrateField`). This is the text tier: "Field 31 µT at the equator, tilted 9° from the spin axis; its
  centre 0.076 radii off the body's; holds the wind off at 9.3 radii. 92 of the lines drawn loop back to the body and
  5 open into the wind."
- **Tiers** (`FIELD_LINES`): 16 × 6 seeds on gpu-high, 12 × 6 on gpu-basic, 10 × 4 on canvas, none on text.

## Checks (`packages/renderer/test/field.test.ts`, 8 tests)

- The Schmidt functions match their closed forms to degree 3, and dP/dθ matches a finite difference.
- An aligned dipole has B0 at the equator, 2·B0 at the pole (pointing in, Earth's sign) and falls as 1/r³. A
  degree-2 field is curl-free (a loop integral vanishes).
- A traced dipole line follows r = L cos² λ to within 1e-4 with fine steps, and within 1e-3 at the default step. It
  lands at the mirror latitude on the same meridian.
- With a 5 nT southward wind field against 31 µT: a footpoint at 60° is closed, and one at 80° opens (the vacuum
  model's cap edge is near 76°). Without the wind's field, fewer lines open.
- The squeeze lets no field through the nose plane, doubles the field at the nose, and lets no field through the
  plane at the flank either.
- In 3D, no point drawn over the disc lies behind the body. The cut's paths fit the frame.
- Cost (see below).
- `test/field-fixture.ts` `earthField()`: a dipole Earth and its bubble for the views (chart room thread) to build against before part 1b.

## Cost (design 9B asked for it to be measured)

Measured in Chromium (Playwright). The phone figures use the Moto G4 profile with a 4× CPU slowdown.

| | Trace (once per change of field or wind) | Turning the 3D layer, per frame |
|---|---|---|
| Desktop, Earth, 98 lines | 7.8 ms | 1.3 ms |
| Desktop, Neptune, 97 lines | 7.4 ms | 1.1 ms |
| Moto G4 emulation, Earth | 33 ms | 4.8 ms |
| Moto G4 emulation, Neptune | 37 ms | 5.5 ms |

A phone traces fewer lines (`FIELD_LINES`). The trace runs on a change, never per frame.

## Part 1b: the kernel's measured Solar System fields

- `session.readField({ fixture: "solar-system" })` (ui-contract) reads `read_field`'s fixture (light thread, PR 172).
  It needs `base-reference` loaded. Each body comes back with `name`, `field` (reference radius, g10, g11, h11,
  tilt, offset, grade and sources) and `wind_field` (Parker's spiral in the ecliptic, away sector).
- `fieldFromRead(body)` turns one into `{ model, bubble, notes }`.
  - In the body frame, z is the spin axis and x points to the star.
  - The wind's field is radial along −x. Its azimuthal part follows the star's spin, which is the orbit's prograde
    sense, −y here.
  - The offset's direction is not published yet, so it is drawn along the magnetic axis and said so in `notes`.
  - A body whose air or surface holds the wind off gets no bubble, and the lines are drawn to their own reach.
- Check: Earth's B0 is 30.6 µT, within 1.5 µT. Its stand-off lies between 7 and 12 radii, and Parker's spiral is
  near 45°. Earth traces closed and open lines. Mercury, Jupiter, Saturn, Uranus and Neptune all trace on the
  canvas tier.

## Pictures (`MF14-screens/`)

These are `mf14a-earth.png`, `mf14a-jupiter.png`, `mf14a-uranus.png` and `mf14a-neptune.png`. Each shows the Chart
cut, the 3D layer and the words, all from `read_field`'s fixture through `fieldFromRead`. Jupiter's open cap is
smaller than the first ring of seeds, so its words honestly say "0 open". The pole's longitude is 0 in the fixture.

## Fudge register

Two rows, `field.ts` compressed and inBubble, and `field.ts` seeds and FIELD_LINES: no tail or ring currents, a flat
mirror, and seeding by dipole flux.

## Next

- Add the Sun and a pulsar once stellar coefficients are published, and stippled belts from read_field's `belts`.
- **Part 2.** The 3D style's tubes, belts and the bubble shell in the GPU pass, and True's band emissions: aurora
  glow on the night side, a giant's radio belts and X-ray loops. Then particles drifting along lines at a stated
  speed-up.
- **MF14b** (chart room thread). Mounts the Chart cut and the 3D layer on the Body screen and the system chart under
  the "Mag" layer toggle. Inks for the two polarities come from the theme (proposed tokens `field-out` and
  `field-in`).
- **MF14c** (accepted 22 Nov). The "Mag" entry in the band switch: Lines, Halo and Surface readings. The trace is the
  same as here; extent is set by the instrument floor. The entry is driven by the seat's lens and instrument list,
  not a field flag. The switch entry is the chart room thread's.
