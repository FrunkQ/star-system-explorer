import { WHY_WIREFRAME_LATER, type ViewControlsSpec } from "../../../ui-contract/src/index.ts";

/**
 * The cluster map in the shared view controls: Chart is the cube-by-cube catalogue; 3D is Galaxies nearby centred on
 * home (GZ5 part 1); True draws each galaxy in its own light, and Free turns the sphere by dragging (GZ5 part 2).
 */
export const CLUSTER_CONTROLS: ViewControlsSpec = {
  styles: { chart: true, "3d": true, true: true, wireframe: WHY_WIREFRAME_LATER },
  flat: true,
  seen: { above: true, tilted: true, edge: true, free: true },
  scale: { readable: true, real: true },
  hud: { "3d": true, true: true },
};
