import type { ViewControlsSpec } from "../../../ui-contract/src/index.ts";

/** The size comparison's view controls (rule 12). Chart is the flat lineup of discs; 3D, True and Wireframe stand the
 * same lineup up as globes, each turning on its own axis (Alex, 24 Nov). Seen from tips every globe together, the row
 * staying put. Scale: Real is one true scale for all (where it opens, Alex 23 Nov); Readable puts radius on a log
 * scale so tiny and huge both show, marked not to scale. */
export const COMPARE_CONTROLS: ViewControlsSpec = {
  styles: { chart: true, "3d": true, true: true, wireframe: true },
  seen: { above: true, tilted: true, edge: true, free: true },
  scale: { readable: true, real: true },
  opens: { scale: "real" },
};
