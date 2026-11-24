// Browser half of tests/ui/enlarge.e2e.mjs (VP5): enlarge a small picture with a hard edge four times, by plain
// bilinear and by enlargeNode, then sharpen with sharpenNode, and measure how wide the edge comes out.
import { DataTexture, LinearFilter, Mesh, OrthographicCamera, PlaneGeometry, RGBAFormat, RenderTarget, Scene, UnsignedByteType } from "three";
import { MeshBasicNodeMaterial, WebGPURenderer } from "three/webgpu";
import { texture, uv, vec2, vec4 } from "three/tsl";
import { enlargeNode, sharpenNode } from "../src/enlarge.ts";

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
