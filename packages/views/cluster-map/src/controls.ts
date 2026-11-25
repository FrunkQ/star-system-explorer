import { WHY_TRUE_HELD, WHY_WIREFRAME_LATER, type ViewControlsSpec } from "../../../ui-contract/src/index.ts";

/** The cluster map in the shared view controls: Chart is the cube-by-cube catalogue, 3D is Galaxies nearby centred on home (GZ5 part 1). */
export const CLUSTER_CONTROLS: ViewControlsSpec = {
  styles: { chart: true, "3d": true, true: WHY_TRUE_HELD, wireframe: WHY_WIREFRAME_LATER },
  flat: true,
  seen: { above: true, tilted: true, edge: true, free: "Turning it freely comes with the True galaxies (GZ5 part 2)" },
  scale: { readable: true, real: true },
  hud: { "3d": true },
};
