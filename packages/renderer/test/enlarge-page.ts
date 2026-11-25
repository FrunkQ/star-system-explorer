// Browser half of tests/ui/enlarge.e2e.mjs (VP5): enlarge a small picture with a hard edge four times, by plain
// bilinear and by enlargeNode, then sharpen with sharpenNode, and measure how wide the edge comes out.
import { BufferGeometry, DataTexture, Float32BufferAttribute, LinearFilter, Mesh, MeshBasicMaterial, OrthographicCamera, PlaneGeometry, RGBAFormat, RenderTarget, Scene, UnsignedByteType } from "three";
import { MeshBasicNodeMaterial, WebGPURenderer } from "three/webgpu";
import { texture, uv, vec2, vec4 } from "three/tsl";
import { enlargeNode, sharpenNode } from "../src/enlarge.ts";
import { startFinish } from "../src/finish.ts";
import { restJitter } from "../src/still.ts";

export async function runEnlarge(): Promise<{ backend: string; bilinear: number[]; bicubic: number[]; sharpened: number[] }> {
  const S = 64;
  const B = 256;
  const d = new Uint8Array(S * S * 4);
  for (let y = 0; y < S; y++)
    for (let x = 0; x < S; x++) {
      const v = x >= S / 2 ? 255 : 0;
      d.set([v, v, v, 255], (y * S + x) * 4);
    }
  const src = new DataTexture(d, S, S, RGBAFormat, UnsignedByteType);
  src.magFilter = src.minFilter = LinearFilter;
  src.needsUpdate = true;
  const canvas = document.createElement("canvas");
  const r = new WebGPURenderer({ canvas });
  await r.init();
  const cam = new OrthographicCamera(-1, 1, 1, -1, 0, 1);
  const quad = new PlaneGeometry(2, 2);
  const rt = new RenderTarget(B, B);
  rt.texture.magFilter = rt.texture.minFilter = LinearFilter;
  const out = new RenderTarget(B, B);
  const row = async (target: RenderTarget) => {
    const px = (await r.readRenderTargetPixelsAsync(target, 0, B / 2, B, 1)) as Uint8Array;
    return Array.from({ length: B }, (_, i) => px[i * 4]! / 255);
  };
  const draw = async (colour: unknown, target: RenderTarget) => {
    const m = new MeshBasicNodeMaterial();
    m.colorNode = colour as never;
    const s = new Scene();
    s.add(new Mesh(quad, m));
    r.setRenderTarget(target);
    r.render(s, cam);
    return row(target);
  };
  const bilinear = await draw(texture(src, uv()), out);
  const bicubic = await draw(vec4(enlargeNode((p) => texture(src, p), vec2(S, S), uv()).rgb, 1), rt);
  const sharpened = await draw(vec4(sharpenNode((p) => texture(rt.texture, p), vec2(1 / B, 1 / B), uv(), 0.2).rgb, 1), out);
  const backend = (r.backend as { isWebGPUBackend?: boolean }).isWebGPUBackend ? "webgpu" : "webgl2";
  r.dispose();
  return { backend, bilinear, bicubic, sharpened };
}

/**
 * Sharpen at rest through the shared finish (VP5 part 2): a white wedge whose edge runs at a slant over black, drawn
 * once, then averaged over 16 frames shifted by the rest clock's jitter. One frame leaves the edge in hard steps; the
 * mean puts part-covered pixels on it. Also drawn at half size through the enlarger. Rows across the middle, 0 to 1.
 */
export async function runRest(): Promise<{ backend: string; single: number[]; averaged: number[]; half: number[] }> {
  const N = 128;
  const canvas = document.createElement("canvas");
  canvas.width = canvas.height = N;
  const r = new WebGPURenderer({ canvas, antialias: false });
  await r.init();
  r.setSize(N, N, false);
  const cam = new OrthographicCamera(-1, 1, 1, -1, 0, 2);
  cam.position.set(0, 0, 1);
  const g = new BufferGeometry();
  // A wedge from the left edge whose right side leans an eighth of a pixel a row.
  g.setAttribute("position", new Float32BufferAttribute([-1, -1, 0, 0.1, -1, 0, -0.1, 1, 0, -1, -1, 0, -0.1, 1, 0, -1, 1, 0], 3));
  const s = new Scene();
  s.add(new Mesh(g, new MeshBasicMaterial({ color: 0xffffff })));
  const out = new RenderTarget(N, N);
  const finish = startFinish(r);
  const row = async () => {
    // A row where the edge falls part way across a pixel (at the middle row it lies on a pixel boundary).
    const px = (await r.readRenderTargetPixelsAsync(out, 0, N / 2 + 6, N, 1)) as Uint8Array;
    const xs = Array.from({ length: N }, (_, i) => px[i * 4]!);
    const top = Math.max(...xs, 1);
    return xs.map((v) => v / top);
  };
  const frame = (o: { accumulate?: number; jitter?: [number, number]; render_scale?: number }) => {
    if (o.jitter) cam.setViewOffset(N, N, -o.jitter[0], o.jitter[1], N, N);
    else cam.clearViewOffset();
    cam.updateProjectionMatrix();
    r.setRenderTarget(out);
    finish.render(s, cam, { exposure: 1, pixel_deg: 0.05, glare: false, render_scale: o.render_scale ?? 1, ...(o.accumulate !== undefined ? { accumulate: o.accumulate } : {}) });
    r.setRenderTarget(null);
  };
  frame({});
  const single = await row();
  for (let k = 0; k < 16; k++) frame({ accumulate: k, jitter: restJitter(k) });
  const averaged = await row();
  frame({ render_scale: 0.5 });
  const half = await row();
  const backend = (r.backend as { isWebGPUBackend?: boolean }).isWebGPUBackend ? "webgpu" : "webgl2";
  finish.dispose();
  r.dispose();
  return { backend, single, averaged, half };
}
