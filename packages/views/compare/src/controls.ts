import { flatChart, type ViewControlsSpec } from "../../../ui-contract/src/index.ts";

/** The size comparison's view controls (rule 12): a flat lineup. Scale: Real is one true scale for all (where it opens,
 * Alex 23 Nov); Readable puts radius on a log scale so tiny and huge both show, marked not to scale. */
export const COMPARE_CONTROLS: ViewControlsSpec = { ...flatChart("The lineup is drawn flat, side by side"), scale: { readable: true, real: true }, opens: { scale: "real" } };
