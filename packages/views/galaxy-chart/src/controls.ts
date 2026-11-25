import { WHY_TRUE_HELD, WHY_WIREFRAME_LATER, type ViewControlsSpec } from "../../../ui-contract/src/index.ts";

/**
 * The galaxy chart in the one set of view controls (rule 12; V9t). Chart is the face-on diagram with its rings,
 * arms, green band and scars. 3D turns the same disc, bent by its collisions' marks. True draws the galaxy in light,
 * in any band, from the kernel's own numbers (renderer galaxy-true.ts). HUD over 3D and True: the Sun's distance and
 * the disc's edge. A galaxy is always drawn to its own scale, so there is no Scale switch here.
 */
export const GALAXY_CONTROLS: ViewControlsSpec = {
  styles: { chart: true, "3d": true, true: true, wireframe: WHY_WIREFRAME_LATER },
  flat: true,
  hud: { "3d": true, true: true },
  bands: { visible: true, infrared: true, ultraviolet: true, xray: true, radio: true, mag: "The galaxy's magnetic field is not drawn yet" },
  // Composite (XO17): several bands laid into one picture of the turned disc.
  composite: { "3d": true, true: true, chart: "The face-on chart is a diagram: turn it to 3D or True for Composite" },
};

/**
 * Your systems (V3 gaps 17, 18 and 39): a flat map seen from above. HUD is the overlay: the grid (every scale-bar
 * length) and the systems' names; routes and the systems themselves always show.
 */
export const YOUR_SYSTEMS_CONTROLS: ViewControlsSpec = {
  styles: { chart: true, "3d": "The campaign's map is drawn from above for now", true: WHY_TRUE_HELD, wireframe: WHY_WIREFRAME_LATER },
  flat: true,
  hud: { chart: true },
};
