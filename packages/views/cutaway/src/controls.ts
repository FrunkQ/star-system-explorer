import { flatChart, type ViewControlsSpec } from "../../../ui-contract/src/index.ts";

/** The cutaway's view controls (rule 12): a flat slice. Scale: Readable spreads depth on a log scale so thin layers
 * show; Real draws every depth to scale. */
export const CUTAWAY_CONTROLS: ViewControlsSpec = { ...flatChart("The cutaway is a slice: it is drawn flat"), scale: { readable: true, real: true } };
