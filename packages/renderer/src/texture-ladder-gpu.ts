// The texture ladder on the GPU (VP3): the composed face pictures (texture-ladder.ts) baked into two cube maps, the
// world's colour and its relief normal, by a cube camera looking out from the centre at an inside-out sphere. Each
// texel reads the kernel face its direction falls on (bilinear inside that face), and its normal is tilted by the
// true slope between points one texel either side, which may lie on a neighbouring face: no seams at cube edges.
// One shader source for WebGPU and WebGL2. Baked once per world and tile read, kept in a small LRU.
import { BackSide, ClampToEdgeWrapping, CubeCamera, DataTexture, FloatType, HalfFloatType, LinearFilter, LinearMipmapLinearFilter, Mesh, NearestFilter, NoColorSpace, RedFormat, RGBAFormat, Scene, SphereGeometry, UnsignedByteType } from "three";
import { CubeRenderTarget, MeshBasicNodeMaterial, MeshStandardNodeMaterial, type WebGPURenderer } from "three/webgpu";
import { abs, atan, clamp, cross, cubeTexture, float, floor, Fn, int, ivec2, length, min, mix, normalize, positionLocal, positionWorld, select, sRGBTransferEOTF, textureLoad, transformNormalToView, uniform, vec3, vec4 } from "three/tsl";
import { cubeBytes, texelAngle, type Composite } from "./texture-ladder.ts";

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type N = any;

export interface Baked {
  key: string;
  signature: string;
  size: number;
  colour: CubeRenderTarget;
  normal: CubeRenderTarget;
  bytes: number;
  /** The material that wears it (lit by the scene's lights, relief from the normal cube). */
  material: MeshStandardNodeMaterial;
}

const Q = Math.PI / 4;

/** Lay the six face pictures into one 3 × 2 atlas of `m` texels a face (smaller faces repeated up, nearest). */
function atlas(c: Composite): { m: number; rgb: Uint8Array; h: Float32Array; present: number[] } {
  const m = Math.max(1, ...c.faces.map((f) => f?.n ?? 1));
  const W = 3 * m;
  const rgb = new Uint8Array(W * 2 * m * 4);
  const h = new Float32Array(W * 2 * m);
  const present: number[] = [];
  c.faces.forEach((f, face) => {
    present.push(f ? 1 : 0);
    const ox = (face % 3) * m;
    const oy = Math.floor(face / 3) * m;
    for (let y = 0; y < m; y++) {
      for (let x = 0; x < m; x++) {
        const o = (oy + y) * W + ox + x;
        if (!f) {
          rgb.set([128, 128, 128, 255], o * 4);
          continue;
        }
        const s = Math.floor((y * f.n) / m) * f.n + Math.floor((x * f.n) / m);
        rgb[o * 4] = f.rgb[s * 3]!;
        rgb[o * 4 + 1] = f.rgb[s * 3 + 1]!;
        rgb[o * 4 + 2] = f.rgb[s * 3 + 2]!;
        rgb[o * 4 + 3] = 255;
        h[o] = f.height_m[s]!;
      }
    }
  });
  return { m, rgb, h, present };
}

/**
 * The bake shader's surface read at a direction: vec4(sRGB colour 0..1, height m). Face by the largest axis, cube
 * coordinates by the equiangular map (as the kernel's atlas), then bilinear within that face's cell of the atlas.
 */
function surfaceRead(rgbTex: N, hTex: N, uM: N) {
  return Fn(([d]: [N]) => {
    const ax = abs(d.x);
    const ay = abs(d.y);
    const az = abs(d.z);
    const isX = ax.greaterThanEqual(ay).and(ax.greaterThanEqual(az));
    const isY = isX.not().and(ay.greaterThanEqual(az));
    // Face index and its normal, right and up (atlas.rs FACES).
    const face: N = select(isX, select(d.x.greaterThanEqual(0), float(0), float(1)), select(isY, select(d.y.greaterThanEqual(0), float(2), float(3)), select(d.z.greaterThanEqual(0), float(4), float(5))));
    const k = select(isX, ax, select(isY, ay, az));
    // right · d and up · d for each face, written out.
    const rx = select(isX, select(d.x.greaterThanEqual(0), d.y, d.y.negate()), select(isY, select(d.y.greaterThanEqual(0), d.x.negate(), d.x), d.y));
    const ux = select(isX.or(isY), d.z, select(d.z.greaterThanEqual(0), d.x.negate(), d.x));
    const u = atan(rx.div(k)).div(Q);
    const v = atan(ux.div(k)).div(Q);
    const cx: N = face.mod(3);
    const cy: N = floor(face.div(3));
    const x = clamp(u.add(1).mul(0.5).mul(uM).sub(0.5), 0, uM.sub(1));
    const y = clamp(v.add(1).mul(0.5).mul(uM).sub(0.5), 0, uM.sub(1));
    const i0 = floor(x);
    const j0 = floor(y);
    const i1 = min(i0.add(1), uM.sub(1));
    const j1 = min(j0.add(1), uM.sub(1));
    const fx = x.sub(i0);
    const fy = y.sub(j0);
    const at = (i: N, j: N) => ivec2(int(cx.mul(uM).add(i)), int(cy.mul(uM).add(j)));
    const tap = (i: N, j: N) => vec4(textureLoad(rgbTex, at(i, j)).xyz, textureLoad(hTex, at(i, j)).x);
    const a = mix(tap(i0, j0), tap(i1, j0), fx);
    const b = mix(tap(i0, j1), tap(i1, j1), fx);
    return mix(a, b, fy);
  });
}

export interface TextureBaker {
  /** The bake for this world at this size, baking it if new or if its tiles changed or its size moved. */
  get(key: string, c: Composite, size: number, radius_m: number): Baked;
  /** Drop bakes beyond the byte budget, least recently used first (the one just asked for is kept). */
  trim(budget: number): void;
  readonly bytes: number;
  dispose(): void;
}

export function createTextureBaker(renderer: WebGPURenderer): TextureBaker {
  const cache = new Map<string, Baked>();
  const sphere = new SphereGeometry(10, 96, 48);
  let bytes = 0;

  function bake(key: string, c: Composite, size: number, radius_m: number): Baked {
    const a = atlas(c);
    const rgbTex = new DataTexture(a.rgb, 3 * a.m, 2 * a.m, RGBAFormat, UnsignedByteType);
    rgbTex.colorSpace = NoColorSpace;
    const hTex = new DataTexture(a.h, 3 * a.m, 2 * a.m, RedFormat, FloatType);
    for (const t of [rgbTex, hTex]) {
      t.magFilter = NearestFilter;
      t.minFilter = NearestFilter;
      t.generateMipmaps = false;
      t.needsUpdate = true;
    }
    const uM = uniform(a.m);
    const read = surfaceRead(rgbTex, hTex, uM);
    const dir = normalize(positionWorld);
    const colourMat = new MeshBasicNodeMaterial({ side: BackSide });
    colourMat.colorNode = vec4(read(dir).xyz, 1);
    const step = texelAngle(size);
    const uStep = uniform(step);
    const uSpan = uniform(2 * step * radius_m);
    const normalMat = new MeshBasicNodeMaterial({ side: BackSide });
    normalMat.colorNode = Fn(() => {
      const r = dir;
      // East round the pole (z); near the pole round x instead.
      const e0 = vec3(r.y.negate(), r.x, 0);
      const e1 = vec3(0, r.z.negate(), r.y);
      const e = normalize(select(length(e0).greaterThan(1e-4), e0, e1));
      const n = cross(r, e);
      const h = (p: N) => read(normalize(p)).w;
      const ge = h(r.add(e.mul(uStep))).sub(h(r.sub(e.mul(uStep)))).div(uSpan);
      const gn = h(r.add(n.mul(uStep))).sub(h(r.sub(n.mul(uStep)))).div(uSpan);
      // Stored as n/2 + 1/2: a material's colour output does not keep negative values.
      return vec4(normalize(r.sub(e.mul(ge)).sub(n.mul(gn))).mul(0.5).add(0.5), 1);
    })();
    const make = (type: typeof UnsignedByteType | typeof HalfFloatType) => {
      const rt = new CubeRenderTarget(size, { type, generateMipmaps: true, minFilter: LinearMipmapLinearFilter, magFilter: LinearFilter });
      rt.texture.colorSpace = NoColorSpace;
      rt.texture.wrapS = ClampToEdgeWrapping;
      rt.texture.wrapT = ClampToEdgeWrapping;
      return rt;
    };
    const colour = make(UnsignedByteType);
    const normal = make(HalfFloatType);
    const scene = new Scene();
    const mesh = new Mesh(sphere, colourMat);
    scene.add(mesh);
    const prevTone = renderer.toneMapping;
    renderer.toneMapping = 0;
    for (const [rt, mat] of [[colour, colourMat], [normal, normalMat]] as const) {
      mesh.material = mat;
      const cam = new CubeCamera(0.1, 100, rt);
      cam.position.set(0, 0, 0);
      cam.update(renderer, scene);
    }
    renderer.toneMapping = prevTone;
    rgbTex.dispose();
    hTex.dispose();
    colourMat.dispose();
    normalMat.dispose();

    // The material that wears it: colour decoded from sRGB, the relief normal carried from the body's frame to view.
    const material = new MeshStandardNodeMaterial({ roughness: 1, metalness: 0 });
    const d = normalize(positionLocal);
    material.colorNode = sRGBTransferEOTF(cubeTexture(colour.texture, d).xyz) as N;
    material.normalNode = transformNormalToView(normalize(cubeTexture(normal.texture, d).xyz.mul(2).sub(1)));
    const b = { key, signature: c.signature, size, colour, normal, bytes: cubeBytes(size) * 1.5, material };
    return b;
  }

  function drop(k: string) {
    const b = cache.get(k);
    if (!b) return;
    b.colour.dispose();
    b.normal.dispose();
    b.material.dispose();
    bytes -= b.bytes;
    cache.delete(k);
  }

  return {
    get bytes() {
      return bytes;
    },
    get(key, c, size, radius_m) {
      const hit = cache.get(key);
      if (hit && hit.signature === c.signature && hit.size === size) {
        cache.delete(key);
        cache.set(key, hit);
        return hit;
      }
      if (hit) drop(key);
      const b = bake(key, c, size, radius_m);
      cache.set(key, b);
      bytes += b.bytes;
      return b;
    },
    trim(budget) {
      const keys = [...cache.keys()];
      for (const k of keys.slice(0, -1)) {
        if (bytes <= budget) break;
        drop(k);
      }
    },
    dispose() {
      for (const k of [...cache.keys()]) drop(k);
      sphere.dispose();
    },
  };
}

