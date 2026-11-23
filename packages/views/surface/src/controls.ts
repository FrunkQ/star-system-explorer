import type { ViewControlsSpec } from "../../../ui-contract/src/index.ts";


/**
 * The Surface view's controls (rule 12). Choosing where to land is always on the world's map. On the way down and on
 * the ground, Chart and 3D draw the ground as a diagram (the chart's colours, shaded, with contour lines); True is the ground in its own light under its own sky; Wireframe is the kernel's mesh itself. Down at
 * eye height the view is your own, so it is always Free. The ground is always drawn at its true size. HUD: the
 * compass, the readouts and the landmarks' names over the ground.
 */
export const SURFACE_CONTROLS: ViewControlsSpec = {
  styles: { chart: true, "3d": true, true: true, wireframe: true },
  seen: { free: true, above: "On the ground the view is your own: drag to look about", tilted: "On the ground the view is your own: drag to look about", edge: "On the ground the view is your own: drag to look about" },
  scale: { real: true, readable: "The ground is always drawn at its true size" },
  hud: { "3d": true, true: true, wireframe: true },
  bands: { visible: true, infrared: "The ground is drawn in visible light for now", ultraviolet: "The ground is drawn in visible light for now", xray: "The ground is drawn in visible light for now", radio: "The ground is drawn in visible light for now", mag: "The field at the ground is not drawn yet: the Globe's Mag shows the world's field" },
};
