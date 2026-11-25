import { WHY_WIREFRAME_LATER, type ViewControlsSpec } from "../../../ui-contract/src/index.ts";

/**
 * The system chart's view controls (rule 12): Chart is the flat log chart seen from above; 3D turns; True (V10a) is
 * the 3D frame with each world in its own surface, lit by its star. Scale: Readable or Real (3D and True; the flat chart
 * is always Readable, and choosing Real turns it into 3D). HUD in every style: scale rings, orbit lines, names and
 * outlines over the bodies, or the bodies alone, like a photograph. Band: each mark painted by its light in that band
 * (the light recipes); Mag draws each world's magnetosphere round it in 3D and True (V10e). Composite (XO17) lays
 * several bands into one picture in 3D and True.
 */
export const SYSTEM_CHART_CONTROLS: ViewControlsSpec = { styles: { chart: true, "3d": true, true: true, wireframe: WHY_WIREFRAME_LATER }, flat: true, scale: { readable: true, real: true }, hud: { chart: true, "3d": true, true: true }, bands: { visible: true, infrared: true, ultraviolet: true, xray: true, radio: true, mag: true }, composite: { "3d": true, true: true, chart: "The flat chart paints its marks in one band: turn it to 3D or True for Composite" }, long: true };
