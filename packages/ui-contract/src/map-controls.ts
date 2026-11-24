// The view controls the galaxy and travel maps declare (interface style rule 12; Scale and HUD, Alex 23 Nov 2026).
// Each is a diagram seen from above. Real means the sizes as they truly were: the cosmic web at its physical size
// then (comoving times the scale factor), a system's orbits on a linear scale of AU instead of the log chart.

import { WHY_TRUE_HELD, WHY_WIREFRAME_LATER, type ViewControlsSpec } from "./view-controls.ts";

const ABOVE_ONLY = "A slice, seen from above for now";

/** A map drawn from above that has a true scale beside its readable one (not `flat`: a flat chart is always Readable). */
function sliceWithScale(why3d: string, whyHud: string): ViewControlsSpec {
  return {
    styles: { chart: true, "3d": why3d, true: WHY_TRUE_HELD, wireframe: WHY_WIREFRAME_LATER },
    seen: { above: true, tilted: ABOVE_ONLY, edge: ABOVE_ONLY, free: ABOVE_ONLY },
    scale: { readable: true, real: true },
    hud: { chart: whyHud },
  };
}

/**
 * The cosmic web: Chart is the flat slice; 3D turns a cube of it (parcels of matter where the truncated Zel'dovich
 * step has moved them, voids, sheets, filaments and knots by colour); True draws the same parcels as light, crowded
 * matter brightest. Real shrinks the slice or cube by the scale factor at the slider's time. HUD: the cube's edges.
 */
export const FOAM_CONTROLS: ViewControlsSpec = {
  styles: { chart: true, "3d": true, true: true, wireframe: WHY_WIREFRAME_LATER },
  flat: true,
  scale: { readable: true, real: true },
  hud: { chart: "The slice has no lines or names to hide", "3d": true, true: true },
};

/** Voyages: in a system Real lays the orbits out in AU, not by the log of their size; between stars it is in parsecs either way. */
export const VOYAGE_CONTROLS = sliceWithScale("Voyages are charted flat for now", "The voyage chart always shows its names");
