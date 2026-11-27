import type { ViewControlsSpec } from "../../../ui-contract/src/index.ts";

/** The black hole view's controls (rule 12; exotic objects design 6): True only, every band and Composite, the HUD's
 * rings, the long exposure for the stars behind. Seen from tips the disc (Above, Tilted 60°, Edge-on 10° above the
 * disc's plane); Free is the hands' own tilt. The bent grid in Wireframe comes with the close-up (XO6). */
export const BLACK_HOLE_CONTROLS: ViewControlsSpec = {
  styles: { chart: "A black hole is drawn from the bending of light: True draws it", "3d": "A black hole is drawn from the bending of light: True draws it", true: true, wireframe: "The bent grid comes with the close-up" },
  seen: { above: true, tilted: true, edge: true, free: true },
  bands: { visible: true, infrared: true, ultraviolet: true, xray: true, radio: true, mag: "A black hole has no field of its own; the disc's field comes later" },
  composite: { true: true },
  hud: { true: true },
  long: true,
};
