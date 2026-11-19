// The pixel-stability test the rendering rules ask of the first 3D view:
// zoom from patch scale to a lander on a moon's surface and check that a
// fixed body stays on the same pixel, to under a pixel, at every step, when
// positions are composed in f64 and only camera-relative numbers are
// narrowed to float32. The flattened alternative is shown to fail.
import { test } from "node:test";
import assert from "node:assert/strict";
import { frameFor, pathTo, project, projectExact, projectFlattened, rebase, relative, tree, type Camera, type Vec3 } from "../src/frames.ts";

const AU = 1.495978707e11;
const t = tree([
  { id: "patch", parent: null, local: [0, 0, 0] },
  { id: "star", parent: "patch", local: [3.1e20, -1.7e20, 4.2e19] },
  { id: "planet", parent: "star", local: [29.7 * AU, 11.3 * AU, 0.4 * AU] },
  { id: "moon", parent: "planet", local: [3.84e8, 1.1e7, 2.0e6] },
  { id: "lander", parent: "moon", local: [1.7374e6, 1.2e3, 0] },
]);

test("relative positions never pass through the patch's numbers", () => {
  assert.deepEqual(pathTo(t, "lander").map((n) => n.id), ["patch", "star", "planet", "moon", "lander"]);
  assert.deepEqual(relative(t, "moon", "lander"), [1.7374e6, 1.2e3, 0]);
  const back = relative(t, "lander", "moon");
  assert.deepEqual(back, [-1.7374e6, -1.2e3, 0]);
  // Moon seen from the planet: exactly its local offset.
  assert.deepEqual(relative(t, "planet", "moon"), [3.84e8, 1.1e7, 2.0e6]);
});

test("zooming from patch scale to the surface keeps the lander on its pixel", () => {
  const W = 1920;
  const H = 1080;
  // Look past the lander along a fixed direction so it sits off-centre at a fixed angle: ideal pixel is constant.
  const dir: Vec3 = [0.6, -0.48, 0.64];
  const side: Vec3 = [0.8, 0.6, 0];
  let worst = 0;
  let worstFlat = 0;
  let first: { x: number; y: number } | null = null;
  const frames = new Set<string>();
  for (let e = 21.5; e >= 1; e -= 0.25) {
    const d = 10 ** e;
    // The camera is placed in the lander's frame (f64), then handed to the frame that can carry it.
    const inLander: Camera = { frame: "lander", eye: [dir[0] * d, dir[1] * d, dir[2] * d], look: [side[0] * d * 0.15, side[1] * d * 0.15, 0], fovY: (50 * Math.PI) / 180, width: W, height: H };
    const cam = rebase(t, inLander, frameFor(t, inLander, "lander"));
    frames.add(cam.frame);
    const exact = projectExact(t, cam, "lander")!;
    const gpu = project(t, cam, "lander")!;
    assert.ok(exact && gpu, `on screen at ${d} m`);
    first ??= exact;
    const drift = Math.hypot(exact.x - first.x, exact.y - first.y);
    assert.ok(drift < 1, `exact projection moved ${drift} px at ${d} m`);
    const err = Math.hypot(gpu.x - exact.x, gpu.y - exact.y);
    worst = Math.max(worst, err);
    const flat = projectFlattened(t, cam, "lander");
    worstFlat = Math.max(worstFlat, flat ? Math.hypot(flat.x - exact.x, flat.y - exact.y) : Infinity);
  }
  console.log(`pixel stability: camera-relative worst ${worst.toFixed(4)} px, flattened worst ${worstFlat.toExponential(2)} px`);
  assert.ok(worst < 1, `camera-relative float32 is within a pixel everywhere (worst ${worst.toFixed(4)} px)`);
  assert.ok(worstFlat > 100, `the flattened path jiggles (worst ${worstFlat} px), so the test has teeth`);
  assert.deepEqual([...frames], ["patch", "star", "planet", "moon", "lander"], "the camera handed over through every frame on the way down");
});
