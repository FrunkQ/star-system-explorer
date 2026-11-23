# V9t: the galaxy in 3D and True

Alex, 23 Nov 2026: visible renderer work first, starting with "True galaxy in 3D", then "one set of controls and similar levels of visualisation at EVERY scale". The galaxy histories design (section 9.4) asks for the True drawing of the same mark formulas as GH9.

## What it does

- The galaxy chart joins the one set of view controls: Chart, 3D and True; Seen from; HUD in 3D and True; Band. A galaxy is always drawn to its own scale, so it has no Scale switch.
  - **Chart** is the face-on diagram exactly as before. Its scars, edge-on strip and "Built from other galaxies" section are unchanged.
  - **3D** turns the same disc as a diagram on the chart's ground. It draws the arm crests, bent by the warp, ripples and tilt the collisions left. HUD On adds the ring edges and the Sun's distance.
  - **True** draws the galaxy in light against black space, made from the kernel's numbers at the chart's time.
- **How True is drawn.**
  - Old stars shine by their mass (mass-to-light, Bell and de Jong 2001).
  - Young stars shine where the gas forms them (Kennicutt 1998, light per unit of star formation from Starburst99).
  - Dust in the gas dims the visible light (gas to dust from Bohlin et al. 1978, scaled by the gas's metallicity), with dust lanes on the inner side of each crest.
  - The bar runs to where the arms start. The bulge is a glow of its own.
  - The disc is drawn as seven layers through its thickness (sech², 0.3 kpc), bent by the marks' heights. Edge-on, it reads as a thin, bent disc with the bulge.
- **Bands in True.**
  - Visible: true colour.
  - Infrared (2.2 µm): the old stars seen through the dust.
  - Ultraviolet: only young stars, deep in the dust.
  - X-ray: high-mass binaries by star formation (Mineo et al. 2012) plus low-mass binaries by stellar mass (Gilfanov 2004).
  - Radio (21 cm): the gas itself, reaching past the stars.
  - Every band except Visible is mapped grey on a log scale 4 decades deep, and each says so.
  - Mag is not offered here yet; the switch says why.
- **Without a GPU,** the face-on picture is drawn tilted on a canvas, without the bend, and the note says so.

## Code

- `renderer/src/galaxy-true.ts`:
  - `galaxyFace(chart, t, band, px)`, plus `surfaceAt`, `armField` and `cloudNoise`;
  - `glowPicture` for the bulge and `sheetWeights` for the layers;
  - `GALAXY_LIGHT` (every law and stand-in, with its source) and `GALAXY_LIGHT_REFS`.
- `gpu.ts`:
  - `GpuSheet` is a picture lying in the plane, added as light and bent by a height grid. `GpuGlow` is a camera-facing glow added as light.
  - New `space` flag (black space without lights).
- New `views/galaxy-3d/Galaxy3D.svelte` covers the camera (drag, keys, wheel, Seen from), True and 3D looks, HUD, canvas fallback and the notes beneath the picture.
- `galaxy-chart/controls.ts` adds `GALAXY_CONTROLS`, and `index.svelte.ts` uses it, adding the GPU tiers. `GalaxyChart.svelte` shows Galaxy3D in 3D and True. Both changes were agreed with the galaxy and travel thread.

## Checks

- `renderer/test/galaxy-true.test.ts` (5 tests):
  - surface densities, and the arm field;
  - visible is brightest at the centre and the same bytes every time;
  - radio reaches past the stars, and no gas means no ultraviolet;
  - the layers sum to one.
- New `tests/ui/galaxy3d.e2e.mjs`:
  - 3D, then True in all five bands, Edge-on, then HUD Off;
  - back to Chart, nothing written, no page errors.
- `galaxy.e2e` passes unchanged. `ui:check` is clean. The MP22 ledger stays at 12.

## Screens

In `V9t-screens/`:
- `v9t-galaxy-true-visible.png`: tilted;
- `v9t-galaxy-true-edge.png`: edge-on;
- `v9t-galaxy-true-xray.png`;
- `v9t-galaxy-true-above-hud-off.png`;
- `v9t-galaxy-3d.png`: the diagram turned.

## Next

- A dust lane seen edge-on: it needs a layer that dims rather than adds.
- The thick disc and stellar halo as their own glows, from GH3's tally.
- Galaxies nearby and the cosmic web in the same looks.
- Mag at galaxy scale once there is a galactic field to draw.
