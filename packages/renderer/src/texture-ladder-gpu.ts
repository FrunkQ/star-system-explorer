// The texture ladder on the GPU (VP3): the composed face pictures (texture-ladder.ts) baked into two cube maps, the
// world's colour and its relief normal, by a cube camera looking out from the centre at an inside-out sphere. Each
// texel reads the kernel face its direction falls on (bilinear inside that face), and its normal is tilted by the
// true slope between points one texel either side, which may lie on a neighbouring face: no seams at cube edges.
// One shader source for WebGPU and WebGL2. Baked once per world and tile read, kept in a small LRU.
import { BackSide, ClampToEdgeWrapping, CubeCamera, DataTexture, DataUtils, FloatType, HalfFloatType, LinearFilter, LinearMipmapLinearFilter, Mesh, NearestFilter, NoColorSpace, RedFormat, RGBAFormat, Scene, SphereGeometry, UnsignedByteType, Vector3 } from "three";
import { CubeRenderTarget, MeshBasicNodeMaterial, MeshStandardNodeMaterial, type WebGPURenderer } from "three/webgpu";
import { abs, atan, clamp, cross, cubeTexture, dot, float, floor, Fn, int, ivec2, length, max, min, mix, normalize, positionLocal, positionWorld, select, smoothstep, sRGBTransferEOTF, texture, textureLoad, transformNormalToView, uniform, vec2, vec3, vec4 } from "three/tsl";
import { cubeBytes, texelAngle, type Composite, type PatchPicture } from "./texture-ladder.ts";

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
  /** Lay a deep patch over it (VP3 part 4), or take it off with null. The same picture again does nothing. */
  setPatch(p: PatchPicture | null): void;
}

/** A blank patch texture while none is laid (alpha 0: the cube shows; never all zero, so a normal read from it is not NaN). */
const blank = () => {
  const t = new DataTexture(new Uint8Array([128, 128, 255, 0]), 1, 1, RGBAFormat, UnsignedByteType);
  t.needsUpdate = true;
  return t;
};

/**
 * The deep patch's read at a direction, as the material and the GPU check use it: vec4(sRGB colour, weight) and the
 * normal. The weight is the texel's alpha, eased to nothing over the patch's outer seventh so it meets the globe
 * with no edge, and nothing behind the patch's centre.
 */
export function patchNodes(d: N) {
  const colTex = texture(blank());
  const norTex = texture(blank());
  const uC = uniform(new Vector3(1, 0, 0));
  const uE = uniform(new Vector3(0, 1, 0));
  const uN = uniform(new Vector3(0, 0, 1));
  const uHalf = uniform(1);
  const uOn = uniform(0);
  const k = dot(d, uC);
  const a = dot(d, uE).div(k);
  const b = dot(d, uN).div(k);
  const at = vec2(a.div(uHalf).mul(0.5).add(0.5), b.div(uHalf).mul(0.5).add(0.5));
  const c = colTex.sample(at) as N;
  const edge = float(1).sub(smoothstep(0.86, 1, max(abs(a), abs(b)).div(uHalf)));
  const w = uOn.mul(select(k.greaterThan(0), float(1), float(0))).mul(edge).mul(c.w);
  const normal = normalize((norTex.sample(at) as N).xyz);
  let shown: PatchPicture | null = null;
  const textures: DataTexture[] = [];
  return {
    colour: c.xyz as N,
    weight: w as N,
    normal: normal as N,
    set(p: PatchPicture | null) {
      if (p === shown) return;
      shown = p;
      for (const t of textures.splice(0)) t.dispose();
      if (!p) {
        uOn.value = 0;
        return;
      }
      const col = new DataTexture(p.rgba, p.n, p.n, RGBAFormat, UnsignedByteType);
      const rgba = new Float32Array(p.n * p.n * 4);
      for (let i = 0; i < p.n * p.n; i++) rgba.set([p.normal[i * 3]!, p.normal[i * 3 + 1]!, p.normal[i * 3 + 2]!, 1], i * 4);
      const nor = new DataTexture(toHalf(rgba), p.n, p.n, RGBAFormat, HalfFloatType);
      for (const t of [col, nor]) {
        t.colorSpace = NoColorSpace;
        t.magFilter = LinearFilter;
        t.minFilter = LinearFilter;
        t.wrapS = ClampToEdgeWrapping;
        t.wrapT = ClampToEdgeWrapping;
        t.generateMipmaps = false;
        t.needsUpdate = true;
        textures.push(t);
      }
      colTex.value = col;
      norTex.value = nor;
      uC.value.set(...p.centre);
      uE.value.set(...p.east);
      uN.value.set(...p.north);
      uHalf.value = p.half;
      uOn.value = 1;
    },
    dispose() {
      for (const t of textures.splice(0)) t.dispose();
    },
  };
}

function toHalf(a: Float32Array): Uint16Array {
  const out = new Uint16Array(a.length);
  for (let i = 0; i < a.length; i++) out[i] = DataUtils.toHalfFloat(a[i]!);
  return out;
}

const Q = Math.PI / 4;

const o0 = (f: { n: number }, x: number, y: number) => y * f.n + x;

/** Lay the six face pictures into one 3 × 2 atlas of `m` texels a face (smaller faces spread up bilinearly). */
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
        if (f.n === m) {
          rgb[o * 4] = f.rgb[o0(f, x, y) * 3]!;
          rgb[o * 4 + 1] = f.rgb[o0(f, x, y) * 3 + 1]!;
          rgb[o * 4 + 2] = f.rgb[o0(f, x, y) * 3 + 2]!;
          rgb[o * 4 + 3] = 255;
          h[o] = f.height_m[o0(f, x, y)]!;
          continue;
        }
        // A coarser face is spread up bilinearly, not repeated in blocks (no stair-stepped edges).
        const fx = Math.min(f.n - 1, Math.max(0, ((x + 0.5) * f.n) / m - 0.5));
        const fy = Math.min(f.n - 1, Math.max(0, ((y + 0.5) * f.n) / m - 0.5));
        const i = Math.max(0, Math.min(f.n - 2, Math.floor(fx)));
        const j = Math.max(0, Math.min(f.n - 2, Math.floor(fy)));
        const ax = fx - i;
        const ay = fy - j;
        const w = [(1 - ax) * (1 - ay), ax * (1 - ay), (1 - ax) * ay, ax * ay];
        const i1 = Math.min(f.n - 1, i + 1);
        const j1 = Math.min(f.n - 1, j + 1);
        const k = [j * f.n + i, j * f.n + i1, j1 * f.n + i, j1 * f.n + i1];
        for (let ch = 0; ch < 3; ch++) rgb[o * 4 + ch] = Math.round(w[0]! * f.rgb[k[0]! * 3 + ch]! + w[1]! * f.rgb[k[1]! * 3 + ch]! + w[2]! * f.rgb[k[2]! * 3 + ch]! + w[3]! * f.rgb[k[3]! * 3 + ch]!);
        rgb[o * 4 + 3] = 255;
        h[o] = w[0]! * f.height_m[k[0]!]! + w[1]! * f.height_m[k[1]!]! + w[2]! * f.height_m[k[2]!]! + w[3]! * f.height_m[k[3]!]!;
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
  const patches = new Map<string, ReturnType<typeof patchNodes>>();
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
    // A deep patch (VP3 part 4) is laid over the cube where it was read, eased in at its edge.
    const material = new MeshStandardNodeMaterial({ roughness: 1, metalness: 0 });
    const d = normalize(positionLocal);
    const patch = patchNodes(d);
    // Only where the patch has weight: elsewhere its read can be NaN (off its side of the globe), and NaN times zero is
    // still NaN, which would black out the whole world.
    const on = patch.weight.greaterThan(0);
    const cubeCol = cubeTexture(colour.texture, d).xyz;
    const cubeNor = normalize(cubeTexture(normal.texture, d).xyz.mul(2).sub(1));
    material.colorNode = sRGBTransferEOTF(select(on, mix(cubeCol, patch.colour, patch.weight), cubeCol)) as N;
    material.normalNode = transformNormalToView(select(on, normalize(mix(cubeNor, patch.normal, patch.weight)), cubeNor));
    const b: Baked = { key, signature: c.signature, size, colour, normal, bytes: cubeBytes(size) * 1.5, material, setPatch: (p) => patch.set(p) };
    patches.set(key, patch);
    return b;
  }

  function drop(k: string) {
    const b = cache.get(k);
    if (!b) return;
    b.colour.dispose();
    b.normal.dispose();
    b.material.dispose();
    patches.get(k)?.dispose();
    patches.delete(k);
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

