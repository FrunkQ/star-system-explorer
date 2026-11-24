// The detail ladder's GPU node against its processor twin (VP6): each noise kind drawn into a float target and read
// back, then compared pixel by pixel with ladderAt. Built and run by tests/ui/ladder-gpu.e2e.mjs.
import { FloatType, Mesh, NearestFilter, OrthographicCamera, PlaneGeometry, RGBAFormat, RenderTarget, Scene } from "three";
import { MeshBasicNodeMaterial, WebGPURenderer } from "three/webgpu";
import { float, uv, vec3, vec4 } from "three/tsl";
import { ladderNode } from "../../../packages/renderer/src/ladder-node.ts";
import { ladderAt, ladderLayer, resolveLayer, rungs, type NoiseKind, type ResolvedLayer } from "../../../packages/renderer/src/ladder.ts";

type Out = { backend?: string; rungs?: number; results?: Record<string, { maxErr: number; maxVal: number }>; error?: string };
const w = window as unknown as { __done?: Out };

(async () => {
  const out: Out = {};
  try {
    const r = new WebGPURenderer({ canvas: document.getElementById("c") as HTMLCanvasElement, forceWebGL: location.search.includes("gl") });
    await r.init();
    out.backend = (r.backend as { isWebGPUBackend?: boolean }).isWebGPUBackend ? "webgpu" : "webgl2";
    const W = 64;
    const X = 2e7;
    const layer = resolveLayer(ladderLayer("giant.curls")!, { rhines_scale_m: 1e7, deformation_radius_m: 2.5e6, jet_shear_m_s: 60, jet_speed_m_s: 120, eddy_speed_m_s: 30 }) as ResolvedLayer;
    // A far anchor, a time well past the epoch and a stretch, so origins, phases and the squash are all exercised.
    const rs = rungs(layer, { metres_per_px: 3e4, span_m: 1e9 }, "gpu-high", 12345, 777, [3e7, -2e7, 5e6]);
    out.rungs = rs.length;
    const at = vec3(uv().x.mul(X), uv().y.mul(X), float(1e6));
    const opts = { normal: vec3(0, 0, 1), along: vec3(0.6, 0.8, 0) };
    const cpuOpts = { normal: [0, 0, 1] as [number, number, number], along: [0.6, 0.8, 0] as [number, number, number], axis: [0, 1, 0] as [number, number, number], stretch: 3 };
    out.results = {};
    for (const kind of ["simplex", "ridged", "cell", "curl", "flow"] as NoiseKind[]) {
      const lad = ladderNode(kind, at, opts);
      lad.set(rs, { axis: [0, 1, 0], stretch: 3 });
      const m = new MeshBasicNodeMaterial();
      // Offset so negative values survive the colour output; curls are metres, scaled down.
      m.colorNode = kind === "curl" ? vec4(lad.node.mul(1e-8).add(0.5), 1) : vec4(lad.node.mul(0.01).add(0.5), 0, 0, 1);
      const scene = new Scene();
      scene.add(new Mesh(new PlaneGeometry(2, 2), m));
      const cam = new OrthographicCamera(-1, 1, 1, -1, 0, 1);
      cam.position.z = 0.5;
      const rt = new RenderTarget(W, W, { type: FloatType, format: RGBAFormat, minFilter: NearestFilter, magFilter: NearestFilter });
      r.setRenderTarget(rt);
      await r.renderAsync(scene, cam);
      const px = (await r.readRenderTargetPixelsAsync(rt, 0, 0, W, W)) as Float32Array;
      // WebGPU reads rows top down, WebGL2 bottom up.
      const flip = out.backend === "webgpu";
      let maxErr = 0;
      let maxVal = 0;
      for (let j = 0; j < W; j++) {
        for (let i = 0; i < W; i++) {
          const cpu = ladderAt(kind, rs, [((i + 0.5) / W) * X, ((j + 0.5) / W) * X, 1e6], cpuOpts);
          const o = ((flip ? W - 1 - j : j) * W + i) * 4;
          const want = kind === "curl" ? (cpu as number[]).map((x) => x / 1e6) : [cpu as number];
          for (let k = 0; k < want.length; k++) {
            const got = (px[o + k]! - 0.5) * 100;
            maxErr = Math.max(maxErr, Math.abs(got - want[k]!));
            maxVal = Math.max(maxVal, Math.abs(want[k]!));
          }
        }
      }
      out.results[kind] = { maxErr, maxVal };
    }
  } catch (e) {
    out.error = String((e as Error).stack ?? e);
  }
  w.__done = out;
})();
