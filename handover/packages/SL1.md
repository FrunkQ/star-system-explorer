# SL1: land anywhere, fly down, walk about

Alex, 23 Nov 2026: renderer work in order: True galaxy in 3D, then fly to the surface. The surface landing rulings (surfaces design 4.1, "yes to all"): pick a landmark from a map, or anywhere; the ship autoroutes down and the view plays the descent as a flown camera that can be skipped; any spot is allowed, with its dangers shown; once down, walk or fly a low drone over a patch a few km across, planet-bound until you take off.

## What it does

A new GM screen, **Land**, on the world rail under Travel (after Route to here). One view, three moments.

- **Choose.** The world's map, drawn from its six level-0 tiles, with its landmarks pinned. The landmarks are the kernel's named landscape: oceans, seas, continents, islands, ranges, peaks, ice caps and craters, the remarkable first. Pick one from the list or the map, or press anywhere.
  - The kernel judges the site and flies a stock lander down to it (`plan_descent`). The lander is suggested from the world's air (airless, thin, thick and cold, or Earth-like), and any of the four can be chosen.
  - The card shows the verdict, the method, the time and the peak g, and every danger with why: slope, roughness, liquid, heating, deceleration, touchdown. "How the site was read" opens the kernel's own reasons.
  - Nothing is refused. A wet site lands "onto bad ground" with the water as a danger; a capsule on the airless Moon is shown failing; the button says "Land anyway".
- **Flying.** Land reads the ground round the site and plays the kernel's descent as a flown camera. The kernel's clock, height, speed and phase run beside it, with its events (entry, peak heating, parachute, touchdown). It is drawn from 100 km out, where the ground read is in sight, and plays at a steady apparent motion of the ground, so a long fall under a parachute gets its share. Skip to the ground at any time.
- **Down.** The ground at eye height (1.7 m, a view parameter), under the world's own sky.
  - Walk with the arrow keys, W A S D or the on-screen pad; drag to look about; Shift to hurry. Tick Drone to fly 40 m up at 14 m/s.
  - You can go 1.9 km from where you landed. Further means Take off and choosing again.
  - The Time of day slider moves the sun (at the equinox for now).
- **View controls.** Chart, 3D, True and Wireframe all draw the ground; Chart draws it as 3D's diagram, since the chart is the map you choose on.
  - **True:** the ground in its own light. The sky is single Rayleigh scattering from the kernel's air optics (`read_air_optics`), sunlight is dimmed by the air, and the far ground fades into the air (aerial perspective). With no air the sky is black.
  - **3D:** the chart's colours, shaded, with contour lines on the kernel's heights, finer as you come down.
  - **Wireframe:** the kernel's mesh itself.
  - **HUD:** compass, height above the site, distance from it, the true horizon distance, and landmarks within 300 km with their bearings.
  - Seen from is always Free (the view is your own). The ground is always at its true size.
- **The ground is the kernel's.**
  - Within 6 km it is read at the fact floor (level 9 on Earth, 76 m). To 120 km it is read four levels coarser.
  - It falls away with the world's curve, so the horizon is the true one (4.7 km for an eye at 1.7 m on Earth).
  - Within 2 km, ground finer than the floor is decoration sized to the site's roughness. Contours follow the kernel's heights, not the decoration. The note under the view says so.
- **Without a GPU,** the ground is drawn as lines of its profile (the line-terrain style of the reference clip).

## Code

- `ui-contract/src/landing.ts`:
  - types `Landmark`, `LandingSite`, `DescentPlan`, `DescentSample`, `DescentEvent` and `DescentHazard`;
  - `STOCK_LANDERS` and `LANDER_WORDS`, `suggestLander`, `DESCENT_VERDICT_WORDS` and `landmarkOrder`.
- `ui-contract/src/session.ts` (GM only):
  - `surfaceTilesAt(system, body, keys)`: tiles at any level through `resolved_system`, with the refusals;
  - `surfaceLandmarks(system, body)`: the register from a quick `survey_surface`;
  - `planDescent({system, body, lat_deg, lon_deg, lander})`: `plan_descent` on the quick survey, the same world the globe shows.
- `renderer/src/surface-patch.ts`:
  - `siteFrame`, `localToDir`, `tileSpot`, `patchKeys`, `levelFor` (the kernel's `level_for`) and `floorFromRefusal`;
  - `readPatch` and `heightAt` (tiles into a grid in metres about the site), `patchPoint`, `groundColours` and `groundMesh` (rings with holes and layers, curved);
  - `curveDrop`, `horizonDistance` and `decoration`;
  - `descentTrack` (telemetry laid in on the curve) and `playbackKeys`.
- `renderer/src/surface-sky.ts`: `airMass`, `rayleighPhase`, `skyRadiance` (closed-form single scattering), `sunTransmittance`, `skyIrradiance` and `sunDirection`.
- `renderer/src/gpu-surface.ts`:
  - three.js in TSL: a sky dome, the ground's three looks, and aerial perspective;
  - a logarithmic depth buffer, so a pebble and the far ring share a frame;
  - meshes moved by −eye, so only patch-sized numbers reach the GPU.
  - `startSurface` loads it lazily.
- `views/surface/`: `Surface.svelte`, `controls.ts` (`SURFACE_CONTROLS`) and the module `sse.view.surface`.
- Shell:
  - `documents/surface.gm.json` (rail: world, travel, order 2);
  - `screens.renderer.ts` (`RENDER_VIEWS`, `RENDER_SCREENS`);
  - one spread each in `registry.ts` and `App.svelte`.
- `docs/view-controls.md` and its generator: the Land row; "Down a lava tube" is the next gap.

## Checks

- `renderer/test/surface-patch.test.ts` (8 tests):
  - the frame is orthonormal; the kernel's levels and refusals are read back;
  - the curve and the true horizon (Earth 4.65 km, Moon 2.43 km), and the mesh's rings and holes;
  - albedo kept in the colour;
  - the track ends on the site, and playback is steady;
  - the sky is blue overhead, the low sun is reddened, and there is no light with no air; the sun's place.
- `ui-contract/test/landing.test.ts` (2 tests), on the sample Sol:
  - landmarks, remarkable first;
  - a sea site lands with the water as a danger; a capsule on the Moon is shown failing;
  - Earth's ground at level 9 (76 m) agrees with the kernel's judged site height; finer tiles are refused, naming the floor;
  - players get none of it.
- New `tests/ui/surface.e2e.mjs`:
  - the map and pins; a peak's verdict with slope, roughness and touchdown;
  - pressing anywhere on the largest continent; Land; the flown descent with its log;
  - Skip, and the 76 m floor note; walking moves the eye;
  - True with HUD, morning, drone; Wireframe; Chart; Take off;
  - no page errors.
- `ui:test` passes except the known local-only Minerva sifter failure. `ui:check` is clean. The MP22 ledger stays at 12.

## Screens

In `SL1-screens/`, the sample Sol's Earth (its synthetic continents stand 5 to 10 km above its seas):
- `sl1-map.png` and `sl1-site.png`: the map, landmarks and a verdict;
- `sl1-flying.png` (25 km up) and `sl1-flying-low.png` (under the parachute);
- `sl1-ground-3d.png`, `sl1-ground-true.png`, `sl1-ground-true-hud.png`, `sl1-ground-true-morning.png`, `sl1-ground-true-drone.png` and `sl1-ground-wireframe.png`.

## Fudges

Five new rows in `docs/fudges.md`:
- ground below the fact floor;
- sky at the ground;
- the sun's place over a site;
- the flown descent's ground track;
- where liquid sits.

## Next

- The voyage's own ship beside the stock landers: the galaxy and travel thread's `shipAtWorld` gives land and take-off rows for it. They will add "Land here" to the Voyages "At world" card once this screen opens on a world and site.
- 10 m tiles inside a declared landing site, which needs the kernel to take sites. Landing facilities and constructs on the ground. Caves from `survey_caves`.
- The sun from the world's spin and date, and the night sky from `sky_from_here`. Multiple scattering, clouds and weather along the descent.
- The world's globe beyond the far ground, seen from high up. The line-terrain style as a chosen look, not only the canvas fallback. Phone checks at `gpu-basic` with "Take photograph".
