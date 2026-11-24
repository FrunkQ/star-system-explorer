// The speed probe's GPU side (VP4, SSE4-visual-polish-design.md 8.2): read what the device's graphics can do, then
// draw three tiny scenes off screen at one megapixel and time them. Fill (sixteen texture reads a pixel), March (a
// 24-step ray march through a noise density) and Geometry (300,000 lit triangles), each against an empty frame so
// that the cost of closing a frame is taken off. Timed by the GPU's own timestamp queries where the device has them,
// otherwise by frames closed with a one-pixel read-back. About half a second, once per adapter and browser (cached by
// probe.ts); "Measure again" in More › Performance runs it again. Nothing leaves the device.
import { BufferGeometry, DataTexture, Mesh, OrthographicCamera, PerspectiveCamera, PlaneGeometry, RGBAFormat, RenderTarget, Scene, SphereGeometry, Vector3 } from "three";
import { MeshBasicNodeMaterial, WebGPURenderer } from "three/webgpu";
import { Fn, Loop, dot, float, max, normalView, sin, texture, uv, vec2, vec3, vec4 } from "three/tsl";
import { factorsFrom, PROBE_SIDE, probeKey, type Capabilities, type ProbeResult, type ProbeScene } from "./probe.ts";

type Nav = Navigator & { gpu?: { requestAdapter(): Promise<GpuAdapterLike | null> }; deviceMemory?: number; connection?: { saveData?: boolean } };
interface GpuAdapterLike {
  features: { has(f: string): boolean };
  limits: { maxTextureDimension2D: number };
  info?: { vendor?: string; architecture?: string; device?: string; description?: string };
}

/** What this device's graphics can do, without drawing. */
export async function readCapabilities(): Promise<Capabilities> {
  const nav = (typeof navigator !== "undefined" ? navigator : {}) as Nav;
  const ratio = typeof devicePixelRatio === "number" ? devicePixelRatio : 1;
  const screenPx = typeof screen !== "undefined" ? Math.round(screen.width * screen.height * ratio * ratio) : 0;
  const reduced = typeof matchMedia === "function" && matchMedia("(prefers-reduced-motion: reduce)").matches;
  const base = { memory_gb: typeof nav.deviceMemory === "number" ? nav.deviceMemory : null, screen_px: screenPx, pixel_ratio: ratio, reduced_motion: reduced, save_data: !!nav.connection?.saveData };
  try {
    const a = await nav.gpu?.requestAdapter();
    if (a) {
      const i = a.info ?? {};
      const adapter = [i.vendor, i.architecture, i.device, i.description].filter(Boolean).join(" ");
      return { backend: "webgpu", compute: true, half_float: true, textures_3d: true, max_texture: a.limits.maxTextureDimension2D, timestamps: a.features.has("timestamp-query"), adapter, ...base };
    }
  } catch {
    // No adapter: try WebGL2.
  }
  try {
    const gl = document.createElement("canvas").getContext("webgl2");
    if (gl) {
      const dbg = gl.getExtension("WEBGL_debug_renderer_info");
      const adapter = String(dbg ? gl.getParameter(dbg.UNMASKED_RENDERER_WEBGL) : gl.getParameter(gl.RENDERER));
      const half = !!(gl.getExtension("EXT_color_buffer_half_float") || gl.getExtension("EXT_color_buffer_float"));
      const ts = !!gl.getExtension("EXT_disjoint_timer_query_webgl2");
      const max_texture = Number(gl.getParameter(gl.MAX_TEXTURE_SIZE));
      gl.getExtension("WEBGL_lose_context")?.loseContext();
      return { backend: "webgl2", compute: false, half_float: half, textures_3d: true, max_texture, timestamps: ts, adapter, ...base };
    }
  } catch {
    // Neither.
  }
  return { backend: null, compute: false, half_float: false, textures_3d: false, max_texture: 0, timestamps: false, adapter: "", ...base };
}

const median = (xs: number[]) => {
  const s = [...xs].sort((a, b) => a - b);
  return s.length ? s[Math.floor((s.length - 1) / 2)]! : NaN;
};

/** A seeded pattern for the fill scene's texture (the same pixels on every device). */
function noiseTexture(side: number): DataTexture {
  const d = new Uint8Array(side * side * 4);
  let x = 0x9e3779b9;
  for (let i = 0; i < d.length; i++) {
    x ^= x << 13;
    x ^= x >>> 17;
    x ^= x << 5;
    d[i] = x & 255;
  }
  const t = new DataTexture(d, side, side, RGBAFormat);
  t.needsUpdate = true;
  return t;
}

export interface ProbeOptions {
  /** Timed frames per scene (after two warm-up frames). */
  runs?: number;
  /** Give up timing a scene after this long, ms (a slow device still finishes in about a second). */
  cap_ms?: number;
}

/** Measure the device. Null when it has no GPU path, or when drawing fails. */
export async function measureProbe(caps: Capabilities, o: ProbeOptions = {}): Promise<ProbeResult | null> {
  if (!caps.backend || typeof document === "undefined") return null;
  const runs = o.runs ?? 7;
  const cap = o.cap_ms ?? 300;
  const canvas = document.createElement("canvas");
  canvas.width = canvas.height = 1;
  let renderer: WebGPURenderer;
  try {
    renderer = new WebGPURenderer({ canvas, antialias: false, trackTimestamp: caps.timestamps });
    await renderer.init();
  } catch {
    return null;
  }
  const backend = (renderer.backend as { isWebGPUBackend?: boolean }).isWebGPUBackend ? "webgpu" : "webgl2";
  const rt = new RenderTarget(PROBE_SIDE, PROBE_SIDE, { depthBuffer: true });
  const flat = new OrthographicCamera(-1, 1, 1, -1, 0, 1);
  const quad = new PlaneGeometry(2, 2);
  const disposers: { dispose(): void }[] = [rt, quad];

  // Fill: sixteen reads of a 512² pattern at spread offsets.
  const tex = noiseTexture(512);
  disposers.push(tex);
  const fillMat = new MeshBasicNodeMaterial();
  let sum = vec4(0, 0, 0, 0) as ReturnType<ReturnType<typeof vec4>["add"]>;
  for (let i = 0; i < 16; i++) sum = sum.add(texture(tex, uv().mul(1 + i * 0.37).add(vec2(i * 0.013, i * 0.029))));
  fillMat.colorNode = sum.mul(1 / 16);
  // March: 24 steps along a ray through a smooth density, accumulated as light.
  const marchMat = new MeshBasicNodeMaterial();
  marchMat.colorNode = Fn(() => {
    const acc = float(0).toVar();
    const p = vec3(uv().mul(2).sub(1), 0).toVar();
    Loop(24, () => {
      p.addAssign(vec3(0.011, 0.007, 0.05));
      const d = sin(p.x.mul(7.1)).mul(sin(p.y.mul(5.3))).mul(sin(p.z.mul(3.7).add(p.x)));
      acc.addAssign(max(d, 0).mul(0.04));
    });
    return vec4(acc, acc.mul(0.8), acc.mul(0.6), 1);
  })();
  // Geometry: 300,000 triangles, lit by one direction.
  const sphere = new SphereGeometry(1, 388, 387);
  const litMat = new MeshBasicNodeMaterial();
  litMat.colorNode = vec4(vec3(max(dot(normalView, vec3(0.5, 0.6, 0.62)), 0.05)), 1);
  disposers.push(fillMat, marchMat, sphere, litMat);

  const one = (m: MeshBasicNodeMaterial, g: BufferGeometry = quad) => {
    const s = new Scene();
    s.add(new Mesh(g, m));
    return s;
  };
  const persp = new PerspectiveCamera(40, 1, 0.1, 10);
  persp.position.set(0, 0, 3.2);
  persp.lookAt(new Vector3());
  const scenes: { id: ProbeScene | "empty"; scene: Scene; camera: OrthographicCamera | PerspectiveCamera }[] = [
    { id: "empty", scene: new Scene(), camera: flat },
    { id: "fill", scene: one(fillMat), camera: flat },
    { id: "march", scene: one(marchMat), camera: flat },
    { id: "geometry", scene: one(litMat, sphere), camera: persp },
  ];

  let method: ProbeResult["method"] = caps.timestamps ? "timestamp" : "readback";
  // Closing a frame: WebGL2 reads one pixel synchronously (three's own read-back waits on animation frames, which
  // would round every time to the display's frame); WebGPU maps a one-pixel read-back.
  const gl = backend === "webgl2" ? ((renderer.backend as { gl?: WebGL2RenderingContext }).gl ?? null) : null;
  const px = new Uint8Array(4);
  const close = async () => {
    if (gl) gl.readPixels(0, 0, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, px);
    else await renderer.readRenderTargetPixelsAsync(rt, 0, 0, 1, 1);
  };
  /** Draw a scene `k` times and close the frame: the total, ms (GPU time by timestamp, else wall time). */
  const timeBatch = async (s: (typeof scenes)[number], k: number): Promise<number> => {
    renderer.setRenderTarget(rt);
    const t0 = performance.now();
    for (let i = 0; i < k; i++) renderer.render(s.scene, s.camera);
    if (method === "timestamp") {
      const d = await renderer.resolveTimestampsAsync("render").catch(() => undefined);
      if (typeof d === "number" && d > 0) return d;
      method = "readback";
    }
    await close();
    return performance.now() - t0;
  };

  const per: Partial<Record<ProbeScene | "empty", number>> = {};
  let closeMs = 0;
  let steady = true;
  try {
    for (const s of scenes) {
      await renderer.compileAsync(s.scene, s.camera).catch(() => undefined);
      await timeBatch(s, 1);
      const first = await timeBatch(s, 1);
      // A fast device draws a scene in well under a millisecond: batch draws so each sample is a few ms long.
      const k = s.id === "empty" ? 1 : Math.max(1, Math.min(32, Math.ceil(4 / Math.max(0.05, first - closeMs))));
      const xs: number[] = [];
      const start = performance.now();
      for (let r = 0; r < runs && (r < 3 || performance.now() - start < cap); r++) xs.push(await timeBatch(s, k));
      const m = median(xs);
      if (xs.length > 1 && Math.max(...xs) > 2 * m && Math.max(...xs) - m > 8) steady = false;
      if (s.id === "empty") closeMs = method === "readback" ? m : 0;
      else per[s.id] = Math.max(0, m - (method === "readback" ? closeMs : 0)) / k;
    }
  } catch {
    return null;
  } finally {
    for (const d of disposers) d.dispose();
    renderer.dispose();
  }
  const scenes_ms = {
    fill: Math.max(0.01, per.fill ?? NaN),
    march: Math.max(0.01, per.march ?? NaN),
    geometry: Math.max(0.01, per.geometry ?? NaN),
  };
  const round = (x: number) => Math.round(x * 1000) / 1000;
  const sm = { fill: round(scenes_ms.fill), march: round(scenes_ms.march), geometry: round(scenes_ms.geometry) };
  return { schema: "sse.render-probe/1", key: probeKey({ backend, adapter: caps.adapter }, typeof navigator !== "undefined" ? navigator.userAgent : ""), backend, method, scenes_ms: sm, factors: factorsFrom(sm), steady };
}
