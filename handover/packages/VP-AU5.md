# VP-AU5 part 1: aurora shells on the Body globe

Track VP-AU (drawing), visual polish design section 16.4. Thread "Build aurorae and night lights", 24 Nov 2026.

## What was built

- `packages/renderer/src/aurora.ts` (pure, no GPU):
  - `planAurora(read, world)`: from a `read_field` body (its `aurora`, its field block for the magnetic axis) and the
    world's radius, scale height (`read_air_optics`) and gas mix, one layer per line per oval: magnetic colatitude,
    full width, peak height and half-peak thickness, colour (chromaticity), and radiance seen face-on
    (power ÷ 4π ÷ the two bands' area). The kernel's `emissions` (MF9 part 2's agreed shape) are drawn as given.
    Without them, the oxygen stand-in (557.7 nm and 630.0 nm at Earth's heights times the scale-height ratio)
    only where O₂ or O is at least 1% of the air; anywhere else the oval waits and the note says why.
    No body names or classes anywhere.
  - `exposeAurora(plan, radius, mode, sun, flux)`: "long" (the default, Alex 19:12) lifts every layer by one ratio
    so the brightest shows at 0.6 of the sunlit ground, keeping ratios between lines; "eye" keeps the daylight
    exposure and greys any line below colour vision (mesopic 0.001 to 0.01 cd/m²). The HUD line says which.
  - Units: rayleighs to radiance (Hunten et al. 1956), line colours from the CIE 1931 fit (Wyman et al. 2013).
- `packages/renderer/src/gpu-aurora.ts`: up to 8 additive shells, each a Gaussian ring round both magnetic poles at
  its colatitude, the path through the layer 1/cos of the view angle capped at the tangent chord (so colours stack
  and brighten at the limb), nothing on the sunlit side. TSL, so WebGPU and WebGL2 both.
- Pass `aurora` (passes.ts; params `exposure` long|eye, `lift_target`), in the true-colour recipe on still, gpu-high
  and gpu-basic. Hooks: `gpu-globe.ts` `draw(..., aurora)`, camera framed to the aurora's top; `runtime.ts`
  `aurora` option and `aurora_note`; `photograph.ts`; `Body.svelte` reads `read_field` (Solar System fixture by
  name, `{system}` for any adopted system, with its `field.tilt` fact) when True is shown, shows the note in the
  badge detail and a visible stand-in line under the globe.

## Proving worlds (rule 5), `tests/fixtures/aurora/proving-worlds.json`

Read from `sse_field::read` on handed bodies at main ab99ecb:

| World | Inputs | Result |
|---|---|---|
| earth-handed (Sol) | R 6371 km, moment 7.7e22 A m², 1014 hPa, H 8.5 km, tilt 9.4°, solar wind at 1 au; N₂ 78%, O₂ 21% | oval at 70.2°; green 110 km, red 250 km |
| tall-tilted (not Sol) | R 3600 km, moment 2e22, 600 hPa, H 21 km, g 4.1, tilt 40°, wind at 0.3 au; N₂ 70%, O₂ 28% | oval at 67.5°, swinging 27.5° to 72.5° spin latitude; green 272 km, red 618 km (2.5×) |
| tipped-giant (world 9) | R 40,000 km, moment 5e26, H₂ 86% He 14%, tilt 70°, a moon loading 300 kg/s | two ovals and a moon spot placed; nothing drawn: no oxygen, colours wait on part 2 |
| airless-lava (world 8) | R 5200 km, no field, no air | no aurora; "no air to glow" |

Unit tests (`packages/renderer/test/aurora.test.ts`): green below red; heights and thickness scale with the scale
height; tilt carried; off on the airless world; no oxygen borrowed on the hydrogen giant; the kernel's emissions
replace the stand-in, radiance = power over area into 4π; long exposure keeps line ratios; eye greys a 10 kR oval;
a kilorayleigh of 557.7 nm is 2.84e-7 W m⁻² sr⁻¹.

Browser check `tests/ui/aurora.e2e.mjs` (desk and phone, Chromium; added to CI): Earth in True names its oval and
both lines at their heights, shows the stand-in line and the long-exposure HUD line; Mars draws none and says why;
nothing written. Screens in the project files, `sse4/handover-screens/VP-AU5-screens/`: `earth-above-*`,
`earth-edge-*` (the southern oval at the limb, green below red), `mars-off-*`.

A browser screenshot of a world outside the Solar System waits on MF2: generated worlds have no magnetic moment
fact yet, so `read_field {system}` reads them as fieldless, and a diffuse glow waits on part 2's lines. Until then
the non-Sol proof is the kernel-read fixture (tall-tilted) in the unit tests. Earth's measured heights checked
against Alex's Consensus pass (section 17.2): green 100 to 115 km, red 150 to 350 km; the stand-in's 110 and 250 sit
inside both.

## Stand-ins (docs/fudges.md, rows "Aurora stand-in" and "Aurora shells")

Oxygen lines only, Earth's heights scaled by scale height, fixed brightness; 5° width; pole longitude not
published; moon spots not drawn; one thin shell per line; daylight cut by a fixed terminator band. Citations:
docs/citations/aurora-drawing.md (all unverified).

## Integration points

- Light thread: MF9 part 2 fields as agreed (status/aurora.md); the drawing switches per oval as they arrive.
- VP1 (renderer thread): the shared Exposure control (Long exposure default, Eye) and the finish's lift; this pass
  then takes the view's exposure instead of its own `exposure` parameter.
- Chart room: `Body.svelte` gained the aurora read and two notes.
