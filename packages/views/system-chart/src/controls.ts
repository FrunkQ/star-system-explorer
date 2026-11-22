import { WHY_TRUE_HELD, WHY_WIREFRAME_LATER, type ViewControlsSpec } from "../../../ui-contract/src/index.ts";

/** The system chart's view controls (rule 12): Chart is the flat log chart seen from above; 3D turns. */
export const SYSTEM_CHART_CONTROLS: ViewControlsSpec = { styles: { chart: true, "3d": true, true: WHY_TRUE_HELD, wireframe: WHY_WIREFRAME_LATER }, flat: true };
