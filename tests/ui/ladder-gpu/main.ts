// The detail ladder's GPU node against its processor twin (VP6): each noise kind drawn into a float target and read
// back, then compared pixel by pixel with ladderAt. Built and run by tests/ui/ladder-gpu.e2e.mjs.
import { FloatType, Mesh, NearestFilter, OrthographicCamera, PlaneGeometry, RGBAFormat, RenderTarget, Scene } from "three";
import { MeshBasicNodeMaterial, WebGPURenderer } from "three/webgpu";
import { float, uv, vec3, vec4 } from "three/tsl";
import { ladderNode } from "../../../packages/renderer/src/ladder-node.ts";
import { ladderAt, ladderLayer, resolveLayer, rungs, type NoiseKind, type ResolvedLayer } from "../../../packages/renderer/src/ladder.ts";

import { composeFaces, faceDirection, reliefNormal, sampleFaces, texelAngle } from "../../../packages/renderer/src/texture-ladder.ts";
import { createTextureBaker } from "../../../packages/renderer/src/texture-ladder-gpu.ts";
import { cubeTexture, normalize, sRGBTransferEOTF } from "three/tsl";
import type { SurfaceTile } from "../../../packages/ui-contract/src/index.ts";

type Out = { backend?: string; rungs?: number; results?: Record<string, { maxErr: number; maxVal: number }>; bake?: { colourErr: number; normalErr: number; tiltMax: number }; error?: string };

/** A made-up world of six level-0 tiles: smooth hills (heights from a sum of waves on the sphere) and a sea below 0. */
function fakeTiles(): SurfaceTile[] {
  const s = 32;
  const out: SurfaceTile[] = [];
  for (let face = 0; face < 6; face++) {
    const elevation_dm: number[] = [];
    const cover: number[] = [];
    for (let j = 0; j < s; j++) {
      for (let i = 0; i < s; i++) {
        const d = faceDirection(face, -1 + (2 * (i + 0.5)) / s, -1 + (2 * (j + 0.5)) / s);
        const h = 4000 * Math.sin(3 * d[0]) * Math.cos(2 * d[1]) + 2500 * Math.sin(4 * d[2] + 1);
        elevation_dm.push(Math.round(h * 10));
        cover.push(h < -3000 ? 1 : 0);
      }
    }
    out.push({ address: "test/world", key: { face, level: 0, x: 0, y: 0 }, samples: s, spacing_m: 3e5, epoch_myr: 0, elevation_dm, material: new Array(s * s).fill(0).map((_, k) => k % 3), cover, albedo: new Array(s * s).fill(300), slope_ddeg: [], surface_age_myr: [], pending: [], hash: `f${face}` });
  }
  return out;
}
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
    // VP3: bake the made-up world's faces, read the cubes back along a grid of directions, compare with the processor.
    {
      const comp = composeFaces(fakeTiles());
      const size = 128;
      const R = 6.4e6;
      const baker = createTextureBaker(r);
      const b = baker.get("test", comp, size, R);
      const H = 32;
      const dirs: [number, number, number][] = [];
      for (let j = 0; j < H; j++) for (let i = 0; i < 2 * H; i++) {
        const th = Math.PI * (j + 0.5) / H;
        const ph = 2 * Math.PI * (i + 0.5) / (2 * H);
        dirs.push([Math.sin(th) * Math.cos(ph), Math.sin(th) * Math.sin(ph), Math.cos(th)]);
      }
      const Wd = 2 * H;
      const dirAt = vec3(uv().x.mul(2 * Math.PI).cos().mul(uv().y.mul(Math.PI).sin()), uv().x.mul(2 * Math.PI).sin().mul(uv().y.mul(Math.PI).sin()), uv().y.mul(Math.PI).cos());
      const readBack = async (node: unknown) => {
        const m = new MeshBasicNodeMaterial();
        m.colorNode = node as never;
        const scene = new Scene();
        scene.add(new Mesh(new PlaneGeometry(2, 2), m));
        const cam = new OrthographicCamera(-1, 1, 1, -1, 0, 1);
        cam.position.z = 0.5;
        const rt = new RenderTarget(Wd, H, { type: FloatType, format: RGBAFormat, minFilter: NearestFilter, magFilter: NearestFilter });
        r.setRenderTarget(rt);
        await r.renderAsync(scene, cam);
        return (await r.readRenderTargetPixelsAsync(rt, 0, 0, Wd, H)) as Float32Array;
      };
      const col = await readBack(vec4(sRGBTransferEOTF(cubeTexture(b.colour.texture, normalize(dirAt)).level(0).xyz), 1));
      const nor = await readBack(vec4(normalize(cubeTexture(b.normal.texture, normalize(dirAt)).level(0).xyz.mul(2).sub(1)).mul(0.5).add(0.5), 1));
      const flip = out.backend === "webgpu";
      let colourErr = 0;
      let normalErr = 0;
      let tiltMax = 0;
      const lin = (c: number) => { const v = c / 255; return v <= 0.04045 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4); };
      for (let j = 0; j < H; j++) for (let i = 0; i < Wd; i++) {
        const u = (i + 0.5) / Wd;
        const v = (j + 0.5) / H;
        const d: [number, number, number] = [Math.cos(2 * Math.PI * u) * Math.sin(Math.PI * v), Math.sin(2 * Math.PI * u) * Math.sin(Math.PI * v), Math.cos(Math.PI * v)];
        const o = ((flip ? H - 1 - j : j) * Wd + i) * 4;
        const cpu = sampleFaces(comp, d)!;
        for (let k = 0; k < 3; k++) colourErr = Math.max(colourErr, Math.abs(col[o + k]! - lin(cpu.rgb[k]!)));
        const n = reliefNormal(comp, d, R, texelAngle(size));
        const g = [nor[o]! * 2 - 1, nor[o + 1]! * 2 - 1, nor[o + 2]! * 2 - 1];
        normalErr = Math.max(normalErr, Math.hypot(g[0]! - n[0], g[1]! - n[1], g[2]! - n[2]));
        tiltMax = Math.max(tiltMax, Math.acos(Math.min(1, n[0] * d[0] + n[1] * d[1] + n[2] * d[2])));
      }
      out.bake = { colourErr, normalErr, tiltMax };
      baker.dispose();
    }
  } catch (e) {
    out.error = String((e as Error).stack ?? e);
  }
  w.__done = out;
})();
