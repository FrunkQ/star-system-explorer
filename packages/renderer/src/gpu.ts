// The GPU path: three.js WebGPURenderer (one shader source; WebGL2 when the
// device has no WebGPU). It is handed camera-relative positions only, already
// narrowed from f64 (frames.ts), and draws with the camera at the origin, so
// no large number ever reaches the GPU (rendering rule 3).
import { AdditiveBlending, AmbientLight, CustomBlending, OneFactor, OneMinusSrcAlphaFactor, BufferGeometry, DoubleSide, Quaternion, RingGeometry, PlaneGeometry, Color, DataTexture, DirectionalLight, Float32BufferAttribute, Line, LineBasicMaterial, LinearFilter, Matrix4, Mesh, MeshBasicMaterial, OrthographicCamera, PerspectiveCamera, PointLight, Points, PointsMaterial, RGBAFormat, Scene, SphereGeometry, Sprite, SpriteMaterial, SRGBColorSpace, TorusGeometry, Vector3 } from "three";
import { MeshBasicNodeMaterial, MeshStandardNodeMaterial, WebGPURenderer } from "three/webgpu";
import { cubeTexture, normalize, positionLocal, texture as textureNode, uv as uvNode, vec3, vec4 } from "three/tsl";
import type { Vec3 } from "./frames.ts";
import type { TrueTexture } from "./true-look.ts";
import { ringLight } from "./rings.ts";
import { startFinish, type Finish } from "./finish.ts";
import type { AirLuts, AirShape } from "./air-luts.ts";
import { flattenScale } from "./true-shape.ts";
import { makeAirShell, type AirShell } from "./gpu-air-shell.ts";
import { BAKE_BUDGET, faceSize } from "./texture-ladder.ts";
import { createTextureBaker } from "./texture-ladder-gpu.ts";
import { makeShade, type ShadeState } from "./gpu-shade.ts";
import type { Shade } from "./eclipse.ts";
import { makeAurora, type AuroraGpu } from "./gpu-aurora.ts";
import type { AuroraDraw } from "./aurora.ts";

/** Clouds are drawn this far out (over the radius) until the weather op gives their height: a hair, so they do not fight the ground. */
const CLOUD_LIFT = 1.002;

export interface GpuItem {
  id: string;
  /** Camera-relative centre (float32 values). */
  rel: Vec3;
  radius: number;
  colour: string;
  ring?: boolean;
  selected?: boolean;
  /**
   * True look: the world's own surface (true-look.ts), keyed so it is uploaded once. A new `version` uploads new
   * pixels of the same size (a giant's face carried by its winds, true-air.ts `flowFace`).
   */
  surface?: { key: string; texture: TrueTexture; version?: number };
  /** True look: a star, drawn by its own light (not lit). */
  shines?: boolean;
  /**
   * True look: the star's face, a square of RGBA pixels drawn for this moment (live-star.ts), row 0 at the top, the
   * disc filling `disc` of the square. It faces the camera, so the limb darkening drawn into it is the true one.
   * A new `version` uploads new pixels.
   */
  face?: { key: string; version: number; width: number; data: Uint8ClampedArray; disc: number };
  /** True look in a band other than visible: the body's own light as a mapped grey, 0..1 (0 is dark), not lit. */
  emit?: number;
  /**
   * True look: the world's rings (rings.ts `ringProfile`), lit by the star: edges in units of the world's drawn
   * radius, the ring plane's normal, and the profile's row of RGBA pixels from the inner edge out. Keyed: uploaded once.
   */
  rings?: { key: string; inner: number; outer: number; normal: Vec3; width: number; data: Uint8ClampedArray };
  /**
   * The globe's own axes (x, y, and z its north pole) in camera space: it is drawn turned and tilted to them, so a
   * surface or a graticule turns with it (the size comparison's spinning globes). Left out: pole on z, unturned.
   */
  frame?: [Vec3, Vec3, Vec3];
  /**
   * True look: the world's own visible light by its heat (hot-glow.ts), linear sRGB in units of a white surface in
   * its daylight; added over the lit globe as light, so a night side glows and a day side gains it.
   */
  glow?: [number, number, number];
  /**
   * True look: plume grains above the limb (plumes.ts `plumeGrains`, turned by true-body.ts to the world's pole in
   * the drawing's frame), in units of its radius from its centre, added as light at `brightness` 0..1.
   */
  plumes?: { points: Float32Array; brightness: number };
  /**
   * True look: the world's air as a shell round it, lit by the star (gpu-air-shell.ts, VP-A3): its shape in its own
   * radius and its look-up tables (air-luts.ts), keyed so they are uploaded once.
   */
  air?: { key: string; shape: AirShape; luts: AirLuts };
  /**
   * True look: the world's clouds and haze alone (true-air.ts `AirLook.clouds`: premultiplied colour, alpha their
   * reflectance), drawn lit over the surface a hair above it until the weather op gives each deck's height (VP-A4).
   */
  clouds?: { key: string; texture: TrueTexture };
  /**
   * True look: the body's own shape instead of a sphere (small-body.ts `smallBodyMesh`: an asteroid's figure, two lobes
   * of a contact binary), in units of `radius`, its long axis on x and spin axis on z. Keyed: built once.
   */
  mesh?: { key: string; positions: Float32Array; normals: Float32Array; index: Uint32Array };
  /** True look: how much the world's spin flattens it (true-shape.ts, TA6), drawn along its pole, volume kept. */
  flatten?: number;
  /** The world's north pole in camera space, for `flatten` when no `frame` is given. */
  pole?: Vec3;
  /**
   * True look: the world's aurora shells (aurora.ts `exposeAurora`, drawn by gpu-aurora.ts as on the Body globe), keyed
   * by world; `pole` its spin pole in the drawing's frame (the plan's magnetic axis is in the globe's own frame, z the pole).
   */
  aurora?: { key: string; draw: AuroraDraw; pole: Vec3 };
  /**
   * True look: the bodies that may shadow this world, where they really are, in its drawn radii (eclipse.ts
   * `shadeFor`), and its star's angular radius. Its rings' shadow and its own on its rings need nothing more.
   */
  shade?: Shade;
  /** True look: the sun's glint off its liquid ground, spread by this wind (m/s; a stand-in when left out) at this index. */
  glint?: { wind_m_s?: number; index?: number };
}

/** True look: the star's light. `at` a camera-relative point (the star is in the frame), else `from` a direction. */
export interface GpuLight {
  at?: Vec3;
  from?: Vec3;
  /** Light from everywhere, as a share of the star's (a trace when left out): a shaded chart globe gets more. */
  ambient?: number;
  /** Keep the chart's ground behind instead of black space (3D style: shaded, but a chart). */
  ground?: boolean;
}

export interface GpuRing {
  /** Camera-relative points of the orbit (float32 values), closed. */
  points: Vec3[];
  colour: string;
  /** Drawn over everything, not hidden behind (lines already trimmed to what faces the viewer: a globe's graticule). */
  over?: boolean;
}

/** A haze of single-pixel points (a shell such as the Oort cloud), camera-relative like everything else. */
export interface GpuDots {
  points: Vec3[];
  colour: string;
  /** Added as light at this strength, 0..1 (the cosmic web in True: crowded matter glows brighter); left out, opaque. */
  glow?: number;
  /** Pixels across (1 when left out). */
  size?: number;
}

/**
 * A flat picture lying in the xy plane (V9t: a galaxy's disc, one layer of its thickness), added to what is behind
 * it as light. `heights` bends it: (n+1)² heights in display units over the square, row 0 at +y, for a warp.
 */
export interface GpuSheet {
  key: string;
  version: number;
  texture: { width: number; data: Uint8ClampedArray };
  /** Camera-relative centre. */
  rel: Vec3;
  /** Half the square's side, display units. */
  half: number;
  /** Brightness this layer adds, 0..1. */
  weight: number;
  heights?: { n: number; z: Float32Array; version: number };
}

/** A camera-facing glow added as light (a galaxy's bulge): a square picture `radius` from centre to edge. */
export interface GpuGlow {
  key: string;
  version: number;
  texture: { width: number; data: Uint8ClampedArray };
  rel: Vec3;
  radius: number;
}

/**
 * The sky's stars behind a True view (VP2): unit directions in the drawing's frame, and each star's flux at the eye as
 * a share of the flux that lights the exposure's white surface, per colour channel. Each is drawn as one pixel whose
 * light is π · share / Ω_pixel (a point's flux in one pixel against a white Lambert surface's radiance), times `gain`
 * (1 as the eye sees; a long exposure lifts it). A new `key` uploads the directions and shares.
 */
export interface GpuStars {
  key: string;
  dirs: Float32Array;
  share: Float32Array;
  gain?: number;
}

/** The brightest a star's pixel may be at a long exposure, as a share of white. */
export const LONG_CLIP = 4;

export interface Gpu {
  readonly backend: "webgpu" | "webgl2";
  /**
   * The GPU's own time for the last frames drawn, ms, when the view asked for timing and the device has timestamp
   * queries (VP4: the governor's signal); null otherwise, and the view times its own work instead.
   */
  readonly gpuMs: number | null;
  draw(o: { items: GpuItem[]; rings: GpuRing[]; dots?: GpuDots[]; sheets?: GpuSheet[]; glows?: GpuGlow[]; forward: Vec3; up: Vec3; fovY: number; width: number; height: number; ground: string; /** True look: worlds lit by their star against black space. */ light?: GpuLight; /** Black space with no light (a galaxy in True). */ space?: boolean; /** Seen straight on, sizes kept whatever the distance (the size comparison): the view's width and height in display units round the forward axis. */ ortho?: { width: number; height: number; near: number; far: number }; /** True: draw through the shared finish (finish.ts): high range, the eye's glare, exposure, AgX, dither. */ finish?: { exposure?: number; glare?: boolean; /** The tier's glare knobs and render scale, and the running mean's frame index at rest (finish.ts FinishFrame). */ rungs?: number; glare_scale?: number; render_scale?: number; accumulate?: number }; /** True: segments round each globe (the recipe's `true-bodies` sphere_detail; default 64). */ sphere_detail?: number; /** Sharpen at rest: shift the picture by this much of a pixel (still.ts restJitter), x right and y up. */ jitter?: [number, number]; /** True: the sky's stars behind everything (VP2, stars-behind.ts). */ stars?: GpuStars; /** Perspective near and far planes, display units (default 1e-3 and 1e4): a close-up brings the near plane down under its height. */ near?: number; far?: number }): void;
  dispose(): void;
}

export async function createGpu(canvas: HTMLCanvasElement, opts: { timing?: boolean } = {}): Promise<Gpu | null> {
  let renderer: WebGPURenderer;
  try {
    renderer = new WebGPURenderer({ canvas, antialias: true, alpha: false, trackTimestamp: !!opts.timing });
    await renderer.init();
  } catch {
    return null;
  }
  const backend = (renderer.backend as { isWebGPUBackend?: boolean }).isWebGPUBackend ? "webgpu" : "webgl2";
  // Frame timing: each frame's timestamp queries are read back after it (a frame late), so the pool never fills.
  const timed = !!opts.timing && !!(renderer.backend as { trackTimestamp?: boolean }).trackTimestamp;
  let gpuMs: number | null = null;
  let resolving = false;
  let unread = 0;
  const resolveTiming = () => {
    if (!timed) return;
    unread++;
    if (resolving) return;
    resolving = true;
    // The frames since the last read-back are read together: their mean is the frame's time.
    const n = unread;
    unread = 0;
    renderer
      .resolveTimestampsAsync("render")
      .then((d) => {
        if (typeof d === "number" && d > 0) gpuMs = d / n;
      })
      .catch(() => undefined)
      .finally(() => (resolving = false));
  };
  // The shared finish, made on first use (a view that never asks for it never builds its passes).
  let finish: Finish | null = null;
  const scene = new Scene();
  // The sky's stars: one geometry per sky, its colours rewritten when the pixel's size or the gain changes.
  const starGeom = new BufferGeometry();
  const starMat = new PointsMaterial({ size: 1, sizeAttenuation: false, vertexColors: true, blending: AdditiveBlending, depthWrite: false, transparent: true });
  const starPoints = new Points(starGeom, starMat);
  starPoints.renderOrder = -1;
  starPoints.frustumCulled = false;
  let starSig = "";
  const camera = new PerspectiveCamera(50, 1, 1e-3, 1e4);
  const flat = new OrthographicCamera(-1, 1, 1, -1, 1e-3, 1e4);
  const turned = new Matrix4();
  const sphere = new SphereGeometry(1, 32, 16);
  // True look: a finer sphere with its poles on z, as the globe's (true-look.ts `equirect`).
  let globe = new SphereGeometry(1, 64, 32).rotateX(Math.PI / 2);
  let globeDetail = 64;
  // Close-up: a globe that fills the screen many times over shows its limb's facets at 64 segments, so it takes a finer one.
  const fineGlobe = new SphereGeometry(1, 512, 256).rotateX(Math.PI / 2);
  const textures = new Map<string, DataTexture>();
  const shells = new Map<string, AirShell>();
  // Aurora shells, one set per world (each holds its own light and centre).
  const auroras = new Map<string, AuroraGpu>();
  // Clouds over a surface: lit like the ground, premultiplied, so the ground shows through where they are thin.
  const cloudMats = new Map<string, MeshStandardNodeMaterial>();
  const cloudMat = (c: NonNullable<GpuItem["clouds"]>, id: string) => {
    const hit = cloudMats.get(c.key);
    if (hit) return hit;
    const map = new DataTexture(c.texture.data.slice(), c.texture.width, c.texture.height, RGBAFormat);
    map.colorSpace = SRGBColorSpace;
    map.magFilter = LinearFilter;
    map.minFilter = LinearFilter;
    map.needsUpdate = true;
    textures.set(c.key, map);
    const m = new MeshStandardNodeMaterial({ color: 0xffffff, map, roughness: 1, metalness: 0, transparent: true, depthWrite: false });
    // Clouds lie in the same shadows as the ground under them.
    shadeOf(id).wear(m);
    // The texture is premultiplied already: add its light, dim what is behind by its alpha.
    m.blending = CustomBlending;
    m.blendSrc = OneFactor;
    m.blendDst = OneMinusSrcAlphaFactor;
    m.blendSrcAlpha = OneFactor;
    m.blendDstAlpha = OneMinusSrcAlphaFactor;
    cloudMats.set(c.key, m);
    return m;
  };
  const meshGeoms = new Map<string, BufferGeometry>();
  const meshGeom = (mesh: NonNullable<GpuItem["mesh"]>) => {
    let g = meshGeoms.get(mesh.key);
    if (!g) {
      g = new BufferGeometry();
      g.setAttribute("position", new Float32BufferAttribute(mesh.positions, 3));
      g.setAttribute("normal", new Float32BufferAttribute(mesh.normals, 3));
      g.setIndex(Array.from(mesh.index));
      meshGeoms.set(mesh.key, g);
    }
    return g;
  };
  const versions = new Map<string, number>();
  // Each world's shadow uniforms (gpu-shade.ts), shared by its surface, cloud and ring materials; set every frame.
  const shades = new Map<string, ShadeState>();
  const shadeOf = (id: string) => shades.get(id) ?? (shades.set(id, makeShade()), shades.get(id)!);
  const worn = new WeakSet<object>();
  const litMats = new Map<string, MeshStandardNodeMaterial>();
  // VP3: a world with cube face pictures wears a bake sized to its disc on screen (texture-ladder.ts), with relief.
  const baker = createTextureBaker(renderer);
  const bakedSize = new Map<string, number>();
  const ladderTier = backend === "webgpu" ? "gpu-high" : "gpu-basic";
  const bakedMat = (it: GpuItem, disc_px: number) => {
    const c = it.surface?.texture.faces;
    if (!c || !c.radius_m || !it.surface) return null;
    const finest = Math.max(...c.faces.map((f) => f?.n ?? 0));
    if (!finest) return null;
    // No finer than four texels a tile sample (beyond that the bake adds nothing the sampler does not).
    const want = Math.min(faceSize(disc_px, ladderTier), Math.max(64, 4 * finest));
    if (want <= 0) return null;
    // Keep a bake within a factor of two of what is wanted, so a world drifting on screen is not baked again.
    const had = bakedSize.get(it.surface.key);
    const size = had && had >= want / 2 && had <= want * 2 ? had : want;
    bakedSize.set(it.surface.key, size);
    const b = baker.get(it.surface.key, c, size, c.radius_m);
    baker.trim(BAKE_BUDGET[ladderTier]);
    b.setPatch(it.surface.texture.patch ?? null);
    // Shadows and the glint (gpu-shade.ts): the bake's colour cube carries where the ground is liquid in its alpha.
    if (!worn.has(b.material)) {
      worn.add(b.material);
      shadeOf(it.id).wear(b.material, cubeTexture(b.colour.texture, normalize(positionLocal)).w);
    }
    return b.material;
  };
  const litMat = (it: GpuItem) => {
    // One material per world: its shadows are its own.
    const key = it.surface?.key ?? `plain:${it.colour}:${it.id}`;
    const hit = litMats.get(key);
    if (hit) {
      const map = textures.get(key);
      const v = it.surface?.version ?? 0;
      if (map && it.surface && versions.get(key) !== v && map.image.width === it.surface.texture.width && map.image.height === it.surface.texture.height) {
        (map.image.data as Uint8Array).set(it.surface.texture.data);
        map.needsUpdate = true;
        versions.set(key, v);
      }
      return hit;
    }
    let map: DataTexture | null = null;
    if (it.surface) {
      const t = it.surface.texture;
      map = new DataTexture(t.data.slice(), t.width, t.height, RGBAFormat);
      map.colorSpace = SRGBColorSpace;
      map.magFilter = LinearFilter;
      map.minFilter = LinearFilter;
      map.needsUpdate = true;
      textures.set(key, map);
      versions.set(key, it.surface.version ?? 0);
    }
    const m = new MeshStandardNodeMaterial({ color: map ? 0xffffff : new Color(it.colour), map, roughness: 1, metalness: 0 });
    shadeOf(it.id).wear(m);
    litMats.set(key, m);
    return m;
  };
  // A star's face: one texture and material per star, its pixels replaced when the moment drawn changes.
  const faces = new Map<string, { tex: DataTexture; mat: SpriteMaterial; version: number; width: number }>();
  const faceMat = (f: NonNullable<GpuItem["face"]>) => {
    let e = faces.get(f.key);
    if (e && e.width !== f.width) {
      e.tex.dispose();
      e.mat.dispose();
      faces.delete(f.key);
      e = undefined;
    }
    if (!e) {
      const tex = new DataTexture(new Uint8Array(f.width * f.width * 4), f.width, f.width, RGBAFormat);
      tex.colorSpace = SRGBColorSpace;
      tex.magFilter = LinearFilter;
      tex.minFilter = LinearFilter;
      e = { tex, mat: new SpriteMaterial({ map: tex, transparent: true, depthWrite: false }), version: -1, width: f.width };
      faces.set(f.key, e);
    }
    if (e.version !== f.version) {
      // Rows turned over: the face is drawn top row first, a texture's row 0 is the bottom.
      const n = f.width;
      const dst = e.tex.image.data as Uint8Array;
      for (let j = 0; j < n; j++) dst.set(f.data.subarray((n - 1 - j) * n * 4, (n - j) * n * 4), j * n * 4);
      e.tex.needsUpdate = true;
      e.version = f.version;
    }
    return e.mat;
  };
  // A body's own light in a mapped band, as grey (linear, so the steps of the log scale read evenly once encoded).
  const greys = new Map<number, MeshBasicMaterial>();
  const greyMat = (v: number) => {
    const k = Math.round(Math.max(0, Math.min(1, v)) * 64);
    return greys.get(k) ?? (greys.set(k, new MeshBasicMaterial({ color: new Color().setScalar(k / 64) })), greys.get(k)!);
  };
  // Pictures added as light (sheets and glows): a texture per key, remade when its version changes; rows turned over.
  const pics = new Map<string, { tex: DataTexture; version: number; width: number }>();
  const picture = (key: string, version: number, t: { width: number; data: Uint8ClampedArray }) => {
    let e = pics.get(key);
    if (e && e.width !== t.width) (e.tex.dispose(), pics.delete(key), (e = undefined));
    if (!e) {
      const tex = new DataTexture(new Uint8Array(t.width * t.width * 4), t.width, t.width, RGBAFormat);
      tex.colorSpace = SRGBColorSpace;
      tex.magFilter = LinearFilter;
      tex.minFilter = LinearFilter;
      e = { tex, version: -1, width: t.width };
      pics.set(key, e);
    }
    if (e.version !== version) {
      const n = t.width;
      const dst = e.tex.image.data as Uint8Array;
      for (let j = 0; j < n; j++) dst.set(t.data.subarray((n - 1 - j) * n * 4, (n - j) * n * 4), j * n * 4);
      e.tex.needsUpdate = true;
      e.version = version;
    }
    return e.tex;
  };
  const lightMats = new Map<string, MeshBasicMaterial>();
  const lightMat = (key: string, tex: DataTexture, weight: number) => {
    const k = `${key}|${Math.round(weight * 1000)}`;
    let m = lightMats.get(k);
    if (!m) {
      m = new MeshBasicMaterial({ map: tex, transparent: true, blending: AdditiveBlending, depthWrite: false, side: DoubleSide, color: new Color().setScalar(weight) });
      lightMats.set(k, m);
    }
    m.map = tex;
    return m;
  };
  const glowMats = new Map<string, SpriteMaterial>();
  const sheetGeoms = new Map<string, { geom: PlaneGeometry; version: number; half: number }>();
  const sheetGeom = (s: GpuSheet) => {
    const n = s.heights?.n ?? 1;
    const key = `${s.key}|${n}`;
    let e = sheetGeoms.get(key);
    const version = s.heights?.version ?? 0;
    if (e && (e.version !== version || e.half !== s.half)) (e.geom.dispose(), sheetGeoms.delete(key), (e = undefined));
    if (!e) {
      const geom = new PlaneGeometry(2 * s.half, 2 * s.half, n, n);
      if (s.heights) {
        // PlaneGeometry's vertices run row by row from +y down, as the heights do.
        const pos = geom.getAttribute("position");
        for (let v = 0; v < pos.count; v++) pos.setZ(v, s.heights.z[v] ?? 0);
        pos.needsUpdate = true;
      }
      e = { geom, version, half: s.half };
      sheetGeoms.set(key, e);
    }
    return e.geom;
  };
  const torus = new TorusGeometry(1.8, 0.08, 6, 48);
  // A world's rings: an annulus in units of its radius, its texture's u running from the inner edge out.
  const ringSets = new Map<string, { geom: RingGeometry; tex: DataTexture; mat: MeshBasicNodeMaterial }>();
  const zAxis = new Vector3(0, 0, 1);
  const ringSet = (r: NonNullable<GpuItem["rings"]>, id: string) => {
    const hit = ringSets.get(r.key);
    if (hit) return hit;
    const geom = new RingGeometry(r.inner, r.outer, 256, 1);
    const pos = geom.getAttribute("position");
    const uv = geom.getAttribute("uv");
    for (let v = 0; v < pos.count; v++) uv.setXY(v, (Math.hypot(pos.getX(v), pos.getY(v)) - r.inner) / (r.outer - r.inner), 0.5);
    uv.needsUpdate = true;
    const tex = new DataTexture(new Uint8Array(r.data), r.width, 1, RGBAFormat);
    tex.colorSpace = SRGBColorSpace;
    tex.magFilter = LinearFilter;
    tex.minFilter = LinearFilter;
    tex.needsUpdate = true;
    // Lit by hand (ringLight in rings.ts), not by the scene's lights: a ring is a layer of particles, not a surface.
    // The world's shadow lies across it (gpu-shade.ts), soft at its edge by the star's disc.
    const mat = new MeshBasicNodeMaterial({ transparent: true, side: DoubleSide, depthWrite: false });
    const t = textureNode(tex, uvNode());
    mat.colorNode = vec4(t.xyz.mul(vec3(shadeOf(id).ringLight())), t.w);
    const e = { geom, tex, mat };
    ringSets.set(r.key, e);
    return e;
  };
  const mats = new Map<string, MeshBasicMaterial>();
  const lineMats = new Map<string, LineBasicMaterial>();
  const mat = (c: string) => mats.get(c) ?? (mats.set(c, new MeshBasicMaterial({ color: new Color(c) })), mats.get(c)!);
  const dotMats = new Map<string, PointsMaterial>();
  const dotMat = (d: GpuDots) => {
    const key = `${d.colour}|${d.glow ?? ""}|${d.size ?? 1}`;
    const hit = dotMats.get(key);
    if (hit) return hit;
    const m = new PointsMaterial({ color: new Color(d.colour), size: d.size ?? 1, sizeAttenuation: false });
    if (d.glow !== undefined) {
      m.transparent = true;
      m.opacity = d.glow;
      m.blending = AdditiveBlending;
      m.depthWrite = false;
    }
    dotMats.set(key, m);
    return m;
  };
  // Hot glow and plume grains: light added to what is behind, cached by colour in steps of 1/1024.
  const addMats = new Map<string, MeshBasicMaterial | PointsMaterial>();
  const glowMat = (c: [number, number, number]) => {
    const q = c.map((v) => Math.round(Math.max(0, Math.min(1, v)) * 1024));
    const k = `g|${q.join(",")}`;
    let m = addMats.get(k) as MeshBasicMaterial | undefined;
    if (!m) addMats.set(k, (m = new MeshBasicMaterial({ color: new Color(q[0]! / 1024, q[1]! / 1024, q[2]! / 1024), transparent: true, blending: AdditiveBlending, depthWrite: false })));
    return m;
  };
  const plumeMat = (b: number) => {
    const q = Math.round(Math.max(0, Math.min(1, b)) * 64);
    const k = `p|${q}`;
    let m = addMats.get(k) as PointsMaterial | undefined;
    if (!m) addMats.set(k, (m = new PointsMaterial({ color: new Color().setScalar(1), size: 1.5, sizeAttenuation: false, transparent: true, opacity: q / 64, blending: AdditiveBlending, depthWrite: false })));
    return m;
  };
  const lineMat = (c: string, over = false) => {
    const key = `${c}|${over}`;
    return lineMats.get(key) ?? (lineMats.set(key, new LineBasicMaterial({ color: new Color(c), depthTest: !over })), lineMats.get(key)!);
  };
  return {
    backend,
    get gpuMs() {
      return gpuMs;
    },
    draw(o) {
      const detail = Math.max(16, Math.min(256, Math.round(o.sphere_detail ?? 64)));
      if (detail !== globeDetail) {
        globe.dispose();
        globe = new SphereGeometry(1, detail, Math.max(8, detail >> 1)).rotateX(Math.PI / 2);
        globeDetail = detail;
      }
      scene.clear();
      // Space is black in the true look: a physical choice, not a theme colour.
      scene.background = new Color((o.light && !o.light.ground) || o.space ? 0x000000 : o.ground);
      if (o.light) {
        // The star lights its worlds with no fall-off across the frame (the distances are drawn on a log scale);
        // a trace of ambient keeps a night side from vanishing against space.
        if (o.light.at) {
          const l = new PointLight(0xffffff, 3, 0, 0);
          l.position.set(o.light.at[0], o.light.at[1], o.light.at[2]);
          scene.add(l);
        } else if (o.light.from) {
          const l = new DirectionalLight(0xffffff, 3);
          l.position.set(o.light.from[0], o.light.from[1], o.light.from[2]);
          scene.add(l);
        }
        scene.add(new AmbientLight(0xffffff, o.light.ambient ?? 0.04));
      }
      for (const it of o.items) {
        if (o.light && it.face) {
          const sp = new Sprite(faceMat(it.face));
          sp.position.set(it.rel[0], it.rel[1], it.rel[2]);
          sp.scale.setScalar((2 * it.radius) / it.face.disc);
          scene.add(sp);
          continue;
        }
        const lit = !!o.light && !it.shines && it.emit === undefined;
        // On screen: the disc's width in pixels (perspective; the ortho view's from its width).
        const dist = Math.hypot(it.rel[0], it.rel[1], it.rel[2]);
        const disc_px = o.ortho ? (2 * it.radius * o.width) / o.ortho.width : dist > it.radius ? (it.radius / dist / Math.tan(o.fovY / 2)) * o.height : o.height;
        const toStarOf = (): Vec3 => (o.light!.at ? [o.light!.at[0] - it.rel[0], o.light!.at[1] - it.rel[1], o.light!.at[2] - it.rel[2]] : (o.light!.from ?? [0, 0, 1]));
        if (lit) {
          const rs = it.rings ? ringSet(it.rings, it.id) : null;
          shadeOf(it.id).set({ rel: it.rel, radius: it.radius, toStar: toStarOf(), ...(it.shade ? { shade: it.shade } : {}), rings: rs && it.rings ? { normal: it.rings.normal, inner: it.rings.inner, outer: it.rings.outer, tex: rs.tex } : null, glint: it.glint ?? null });
        }
        const surfaceMat = lit && it.surface?.texture.faces ? bakedMat(it, disc_px) : null;
        const m = new Mesh(it.mesh ? meshGeom(it.mesh) : o.light || it.frame ? (dist < 1.5 * it.radius ? fineGlobe : globe) : sphere, it.emit !== undefined ? greyMat(it.emit) : lit ? (surfaceMat ?? litMat(it)) : mat(it.colour));
        m.position.set(it.rel[0], it.rel[1], it.rel[2]);
        m.scale.setScalar(it.radius);
        if (it.frame) {
          const [x, y, z] = it.frame;
          m.quaternion.setFromRotationMatrix(turned.makeBasis(new Vector3(...x), new Vector3(...y), new Vector3(...z)));
        } else if (it.flatten && it.pole) m.quaternion.setFromUnitVectors(zAxis, new Vector3(...it.pole).normalize());
        if (it.flatten) {
          // The globe's poles are on its own z: shorter along it, wider round it, the volume kept.
          const s = flattenScale(it.flatten);
          m.scale.set(it.radius * s.equator, it.radius * s.equator, it.radius * s.pole);
        }
        scene.add(m);
        // A hot world's own light: a second skin added over the globe (hot-glow.ts).
        if (o.light && it.glow && Math.max(...it.glow) > 0) {
          const gm = new Mesh(globe, glowMat(it.glow));
          gm.scale.setScalar(1.0005);
          m.add(gm);
        }
        // Plume grains over the limb, already turned to the world's pole (plumes.ts, true-body.ts).
        if (o.light && it.plumes && it.plumes.brightness > 0 && it.plumes.points.length) {
          const pg = new BufferGeometry();
          pg.setAttribute("position", new Float32BufferAttribute(it.plumes.points, 3));
          const pp = new Points(pg, plumeMat(it.plumes.brightness));
          pp.position.copy(m.position);
          pp.scale.setScalar(it.radius);
          scene.add(pp);
        }
        if (lit && it.clouds) {
          const cm = new Mesh(dist < 1.5 * it.radius ? fineGlobe : globe, cloudMat(it.clouds, it.id));
          cm.position.copy(m.position);
          cm.quaternion.copy(m.quaternion);
          cm.scale.copy(m.scale).multiplyScalar(CLOUD_LIFT);
          cm.renderOrder = 1;
          scene.add(cm);
        }
        if (o.light && it.air && !it.shines && it.emit === undefined) {
          let sh = shells.get(it.air.key);
          if (!sh) shells.set(it.air.key, (sh = makeAirShell(it.air.shape, it.air.luts)));
          const toStar: Vec3 = o.light.at ? [o.light.at[0] - it.rel[0], o.light.at[1] - it.rel[1], o.light.at[2] - it.rel[2]] : (o.light.from ?? [0, 0, 1]);
          // The point light's strength (3), as the lit globe gets.
          sh.set(it.rel, it.radius, toStar, 3);
          const am = new Mesh(dist < 1.5 * it.radius ? fineGlobe : globe, sh.material);
          am.position.copy(m.position);
          am.scale.setScalar(it.radius * sh.top);
          am.renderOrder = 1;
          scene.add(am);
        }
        if (o.light && it.aurora && !it.shines && it.emit === undefined) {
          let au = auroras.get(it.aurora.key);
          if (!au) auroras.set(it.aurora.key, (au = makeAurora()));
          const t = o.light.at ? [o.light.at[0] - it.rel[0], o.light.at[1] - it.rel[1], o.light.at[2] - it.rel[2]] : (o.light.from ?? [0, 0, 1]);
          const tl = Math.hypot(t[0]!, t[1]!, t[2]!) || 1;
          const p = it.aurora.pole;
          const turn = new Matrix4().compose(new Vector3(it.rel[0], it.rel[1], it.rel[2]), new Quaternion().setFromUnitVectors(zAxis, new Vector3(p[0], p[1], p[2]).normalize()), new Vector3(it.radius, it.radius, it.radius));
          au.set(it.aurora.draw, [t[0]! / tl, t[1]! / tl, t[2]! / tl], turn, it.rel);
          scene.add(au.group);
        }
        if (o.light && it.rings) {
          const rs = ringSet(it.rings, it.id);
          const n = it.rings.normal;
          const toLight: Vec3 = o.light.at ? [o.light.at[0] - it.rel[0], o.light.at[1] - it.rel[1], o.light.at[2] - it.rel[2]] : (o.light.from ?? [0, 0, 1]);
          const toEye: Vec3 = [-it.rel[0], -it.rel[1], -it.rel[2]];
          // The point light's strength (3) over π, as a lit world's diffuse surface gets.
          shadeOf(it.id).uRingScale.value = (3 / Math.PI) * ringLight(n, toLight, toEye);
          const rm = new Mesh(rs.geom, rs.mat);
          rm.position.copy(m.position);
          rm.scale.setScalar(it.radius);
          rm.quaternion.copy(new Quaternion().setFromUnitVectors(zAxis, new Vector3(it.rings.normal[0], it.rings.normal[1], it.rings.normal[2]).normalize()));
          scene.add(rm);
        }
        if (it.ring && !(o.light && it.rings)) {
          const r = new Mesh(torus, mat(it.colour));
          r.position.copy(m.position);
          r.scale.setScalar(it.radius);
          scene.add(r);
        }
      }
      for (const ring of o.rings) {
        const g = new BufferGeometry();
        g.setAttribute("position", new Float32BufferAttribute(ring.points.flat(), 3));
        const line = new Line(g, lineMat(ring.colour, ring.over));
        if (ring.over) line.renderOrder = 1;
        scene.add(line);
      }
      for (const s of o.sheets ?? []) {
        const m = new Mesh(sheetGeom(s), lightMat(s.key, picture(s.key, s.version, s.texture), s.weight));
        m.position.set(s.rel[0], s.rel[1], s.rel[2]);
        scene.add(m);
      }
      for (const g of o.glows ?? []) {
        const tex = picture(g.key, g.version, g.texture);
        let m = glowMats.get(g.key);
        if (!m) glowMats.set(g.key, (m = new SpriteMaterial({ map: tex, transparent: true, blending: AdditiveBlending, depthWrite: false })));
        m.map = tex;
        const sp = new Sprite(m);
        sp.position.set(g.rel[0], g.rel[1], g.rel[2]);
        sp.scale.setScalar(2 * g.radius);
        scene.add(sp);
      }
      for (const d of o.dots ?? []) {
        const g = new BufferGeometry();
        g.setAttribute("position", new Float32BufferAttribute(d.points.flat(), 3));
        scene.add(new Points(g, dotMat(d)));
      }
      if (o.stars && !o.ortho) {
        // One drawn pixel's angle: drawn smaller than the view (the tier's render scale), each spans more sky.
        const pixel = o.fovY / Math.max(1, o.height) / Math.max(0.25, Math.min(1, o.finish?.render_scale ?? 1));
        const k = (Math.PI / (pixel * pixel)) * (o.stars.gain ?? 1);
        const sig = `${o.stars.key}|${k}`;
        if (sig !== starSig) {
          if (!starSig.startsWith(`${o.stars.key}|`)) {
            // Far out along each direction, inside the camera's far plane: behind every body, never in front.
            const pos = new Float32Array(o.stars.dirs.length);
            for (let i = 0; i < pos.length; i++) pos[i] = o.stars.dirs[i]! * 9e3;
            starGeom.setAttribute("position", new Float32BufferAttribute(pos, 3));
          }
          const col = new Float32Array(o.stars.share.length);
          // A long exposure clips each star at a few times white, as a photograph's sensor fills: lifted, the nearest
          // bright stars would otherwise outshine the system's own star and glare over it.
          const cap = (o.stars.gain ?? 1) > 1 ? LONG_CLIP : Infinity;
          for (let i = 0; i < col.length; i++) col[i] = Math.min(cap, o.stars.share[i]! * k);
          starGeom.setAttribute("color", new Float32BufferAttribute(col, 3));
          starSig = sig;
        }
        scene.add(starPoints);
      }
      const cam = o.ortho ? flat : camera;
      if (o.ortho) {
        flat.left = -o.ortho.width / 2;
        flat.right = o.ortho.width / 2;
        flat.top = o.ortho.height / 2;
        flat.bottom = -o.ortho.height / 2;
        flat.near = o.ortho.near;
        flat.far = o.ortho.far;
      } else {
        camera.fov = (o.fovY * 180) / Math.PI;
        camera.near = o.near ?? 1e-3;
        camera.far = o.far ?? 1e4;
        camera.aspect = o.width / Math.max(1, o.height);
      }
      cam.position.set(0, 0, 0);
      cam.up.set(o.up[0], o.up[1], o.up[2]);
      cam.lookAt(new Vector3(o.forward[0], o.forward[1], o.forward[2]));
      // A sub-pixel shift of the projection for the running mean at rest; cleared otherwise.
      if (o.jitter && (o.jitter[0] !== 0 || o.jitter[1] !== 0)) cam.setViewOffset(o.width, o.height, -o.jitter[0], o.jitter[1], o.width, o.height);
      else cam.clearViewOffset();
      cam.updateProjectionMatrix();
      renderer.setSize(o.width, o.height, false);
      if (o.finish) (finish ??= startFinish(renderer)).render(scene, cam, { exposure: o.finish.exposure ?? 1, pixel_deg: o.ortho ? 0 : (o.fovY * 180) / Math.PI / Math.max(1, o.height), glare: !o.ortho && o.finish.glare !== false, rungs: o.finish.rungs, glare_scale: o.finish.glare_scale, render_scale: o.finish.render_scale, accumulate: o.finish.accumulate });
      else renderer.render(scene, cam);
      resolveTiming();
      scene.traverse((x) => (x instanceof Line || x instanceof Points) && x !== starPoints && x.geometry.dispose());
    },
    dispose() {
      sphere.dispose();
      globe.dispose();
      fineGlobe.dispose();
      torus.dispose();
      textures.forEach((t) => t.dispose());
      shells.forEach((x) => x.dispose());
      auroras.forEach((x) => x.dispose());
      cloudMats.forEach((m) => m.dispose());
      meshGeoms.forEach((g) => g.dispose());
      faces.forEach((e) => (e.tex.dispose(), e.mat.dispose()));
      litMats.forEach((m) => m.dispose());
      baker.dispose();
      mats.forEach((m) => m.dispose());
      greys.forEach((m) => m.dispose());
      pics.forEach((e) => e.tex.dispose());
      lightMats.forEach((m) => m.dispose());
      glowMats.forEach((m) => m.dispose());
      sheetGeoms.forEach((e) => e.geom.dispose());
      ringSets.forEach((e) => (e.geom.dispose(), e.tex.dispose(), e.mat.dispose()));
      lineMats.forEach((m) => m.dispose());
      dotMats.forEach((m) => m.dispose());
      addMats.forEach((m) => m.dispose());
      finish?.dispose();
      starGeom.dispose();
      starMat.dispose();
      renderer.dispose();
    },
  };
}
