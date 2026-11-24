// The drawing rows of the bench (VP4, SSE4-visual-polish-design.md 8.2): the speed probe's three scenes timed on
// this device, and the standard True recipe's predicted frame at each GPU tier at 1080p. Built by
// scripts/build-harness.sh into tests/harness/pkg/drawing/ (three.js bundled), loaded by the bench on /harness/
// only when the bench runs, and written into the report as `drawing`; scripts/min-spec.mjs reads it.
import trueColour from "./recipes/true-colour.json";
import { LIBRARY } from "./passes.ts";
import { drawingBench, type DrawingBench } from "./probe.ts";
import { measureProbe, readCapabilities } from "./probe-gpu.ts";
import type { RecipeDoc } from "./recipe.ts";

/** Measure this device (always afresh: the bench never reads the app's cache) and place it at 1080p. */
export async function benchDrawing(): Promise<DrawingBench> {
  const caps = await readCapabilities();
  const probe = await measureProbe(caps, { runs: 15, cap_ms: 1500 });
  return drawingBench(caps, probe, trueColour as RecipeDoc, LIBRARY);
}
