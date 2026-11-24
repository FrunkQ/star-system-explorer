// The GPU path: three.js WebGPURenderer (one shader source; WebGL2 when the
// device has no WebGPU). It is handed camera-relative positions only, already
// narrowed from f64 (frames.ts), and draws with the camera at the origin, so
// no large number ever reaches the GPU (rendering rule 3).
import { AdditiveBlending, AmbientLight, BufferGeometry, DoubleSide, Quaternion, RingGeometry, PlaneGeometry, Color, DataTexture, DirectionalLight, Float32BufferAttribute, Line, LineBasicMaterial, LinearFilter, Matrix4, Mesh, MeshBasicMaterial, MeshStandardMaterial, OrthographicCamera, PerspectiveCamera, PointLight, Points, PointsMaterial, RGBAFormat, Scene, SphereGeometry, Sprite, SpriteMaterial, SRGBColorSpace, TorusGeometry, Vector3 } from "three";
import { WebGPURenderer } from "three/webgpu";
import type { Vec3 } from "./frames.ts";
import type { TrueTexture } from "./true-look.ts";
import { ringLight } from "./rings.ts";

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
   * True look: the body's own shape instead of a sphere (small-body.ts `smallBodyMesh`: an asteroid's figure, two lobes
   * of a contact binary), in units of `radius`, its long axis on x and spin axis on z. Keyed: built once.
   */
  mesh?: { key: string; positions: Float32Array; normals: Float32Array; index: Uint32Array };
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

export interface Gpu {
  readonly backend: "webgpu" | "webgl2";
  draw(o: { items: GpuItem[]; rings: GpuRing[]; dots?: GpuDots[]; sheets?: GpuSheet[]; glows?: GpuGlow[]; forward: Vec3; up: Vec3; fovY: number; width: number; height: number; ground: string; /** True look: worlds lit by their star against black space. */ light?: GpuLight; /** Black space with no light (a galaxy in True). */ space?: boolean; /** Seen straight on, sizes kept whatever the distance (the size comparison): the view's width and height in display units round the forward axis. */ ortho?: { width: number; height: number; near: number; far: number } }): void;
  dispose(): void;
}

export async function createGpu(canvas: HTMLCanvasElement): Promise<Gpu | null> {
  let renderer: WebGPURenderer;
  try {
    renderer = new WebGPURenderer({ canvas, antialias: true, alpha: false });
    await renderer.init();
  } catch {
    return null;
  }
  const backend = (renderer.backend as { isWebGPUBackend?: boolean }).isWebGPUBackend ? "webgpu" : "webgl2";
  const scene = new Scene();
  const camera = new PerspectiveCamera(50, 1, 1e-3, 1e4);
  const flat = new OrthographicCamera(-1, 1, 1, -1, 1e-3, 1e4);
  const turned = new Matrix4();
  const sphere = new SphereGeometry(1, 32, 16);
  // True look: a finer sphere with its poles on z, as the globe's (true-look.ts `equirect`).
  const globe = new SphereGeometry(1, 64, 32).rotateX(Math.PI / 2);
  const textures = new Map<string, DataTexture>();
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
  const litMats = new Map<string, MeshStandardMaterial>();
  const litMat = (it: GpuItem) => {
    const key = it.surface?.key ?? `plain:${it.colour}`;
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
    const m = new MeshStandardMaterial({ color: map ? 0xffffff : new Color(it.colour), map, roughness: 1, metalness: 0 });
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
  const ringSets = new Map<string, { geom: RingGeometry; tex: DataTexture; mat: MeshBasicMaterial }>();
  const zAxis = new Vector3(0, 0, 1);
  const ringSet = (r: NonNullable<GpuItem["rings"]>) => {
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
    const mat = new MeshBasicMaterial({ map: tex, transparent: true, side: DoubleSide, depthWrite: false });
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
  const lineMat = (c: string, over = false) => {
    const key = `${c}|${over}`;
    return lineMats.get(key) ?? (lineMats.set(key, new LineBasicMaterial({ color: new Color(c), depthTest: !over })), lineMats.get(key)!);
  };
  return {
    backend,
    draw(o) {
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
        const m = new Mesh(it.mesh ? meshGeom(it.mesh) : o.light || it.frame ? globe : sphere, it.emit !== undefined ? greyMat(it.emit) : lit ? litMat(it) : mat(it.colour));
        m.position.set(it.rel[0], it.rel[1], it.rel[2]);
        m.scale.setScalar(it.radius);
        if (it.frame) {
          const [x, y, z] = it.frame;
          m.quaternion.setFromRotationMatrix(turned.makeBasis(new Vector3(...x), new Vector3(...y), new Vector3(...z)));
        }
        scene.add(m);
        if (o.light && it.rings) {
          const rs = ringSet(it.rings);
          const n = it.rings.normal;
          const toLight: Vec3 = o.light.at ? [o.light.at[0] - it.rel[0], o.light.at[1] - it.rel[1], o.light.at[2] - it.rel[2]] : (o.light.from ?? [0, 0, 1]);
          const toEye: Vec3 = [-it.rel[0], -it.rel[1], -it.rel[2]];
          // The point light's strength (3) over π, as a lit world's diffuse surface gets.
          rs.mat.color.setScalar((3 / Math.PI) * ringLight(n, toLight, toEye));
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
        camera.aspect = o.width / Math.max(1, o.height);
      }
      cam.position.set(0, 0, 0);
      cam.up.set(o.up[0], o.up[1], o.up[2]);
      cam.lookAt(new Vector3(o.forward[0], o.forward[1], o.forward[2]));
      cam.updateProjectionMatrix();
      renderer.setSize(o.width, o.height, false);
      renderer.render(scene, cam);
      scene.traverse((x) => (x instanceof Line || x instanceof Points) && x.geometry.dispose());
    },
    dispose() {
      sphere.dispose();
      globe.dispose();
      torus.dispose();
      textures.forEach((t) => t.dispose());
      meshGeoms.forEach((g) => g.dispose());
      faces.forEach((e) => (e.tex.dispose(), e.mat.dispose()));
      litMats.forEach((m) => m.dispose());
      mats.forEach((m) => m.dispose());
      greys.forEach((m) => m.dispose());
      pics.forEach((e) => e.tex.dispose());
      lightMats.forEach((m) => m.dispose());
      glowMats.forEach((m) => m.dispose());
      sheetGeoms.forEach((e) => e.geom.dispose());
      ringSets.forEach((e) => (e.geom.dispose(), e.tex.dispose(), e.mat.dispose()));
      lineMats.forEach((m) => m.dispose());
      dotMats.forEach((m) => m.dispose());
      renderer.dispose();
    },
  };
}
