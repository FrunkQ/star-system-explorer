import { WHY_WIREFRAME_LATER, type ViewControlsSpec } from "../../../ui-contract/src/index.ts";

/**
 * The system chart's view controls (rule 12): Chart is the flat log chart seen from above; 3D turns; True (V10a) is
 * the 3D frame with each world in its own surface, lit by its star. Scale: Readable or Real (3D and True; the flat chart
 * is always Readable, and choosing Real turns it into 3D). HUD in True: orbit lines, names and outlines, or a photograph.
 */
export const SYSTEM_CHART_CONTROLS: ViewControlsSpec = { styles: { chart: true, "3d": true, true: true, wireframe: WHY_WIREFRAME_LATER }, flat: true, scale: { readable: true, real: true }, hud: { "3d": true, true: true } };
