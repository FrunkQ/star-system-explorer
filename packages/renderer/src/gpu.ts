// The GPU path: three.js WebGPURenderer (one shader source; WebGL2 when the
// device has no WebGPU). It is handed camera-relative positions only, already
// narrowed from f64 (frames.ts), and draws with the camera at the origin, so
// no large number ever reaches the GPU (rendering rule 3).
import { AdditiveBlending, AmbientLight, BufferGeometry, DoubleSide, PlaneGeometry, Color, DataTexture, DirectionalLight, Float32BufferAttribute, Line, LineBasicMaterial, LinearFilter, Mesh, MeshBasicMaterial, MeshStandardMaterial, PerspectiveCamera, PointLight, Points, PointsMaterial, RGBAFormat, Scene, SphereGeometry, Sprite, SpriteMaterial, SRGBColorSpace, TorusGeometry, Vector3 } from "three";
import { WebGPURenderer } from "three/webgpu";
import type { Vec3 } from "./frames.ts";
import type { TrueTexture } from "./true-look.ts";

export interface GpuItem {
  id: string;
  /** Camera-relative centre (float32 values). */
  rel: Vec3;
  radius: number;
  colour: string;
  ring?: boolean;
  selected?: boolean;
  /** True look: the world's own surface (true-look.ts), keyed so it is uploaded once. */
  surface?: { key: string; texture: TrueTexture };
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
}

/** True look: the star's light. `at` a camera-relative point (the star is in the frame), else `from` a direction. */
export interface GpuLight {
  at?: Vec3;
  from?: Vec3;
}

export interface GpuRing {
  /** Camera-relative points of the orbit (float32 values), closed. */
  points: Vec3[];
  colour: string;
}

/** A haze of single-pixel points (a shell such as the Oort cloud), camera-relative like everything else. */
export interface GpuDots {
  points: Vec3[];
  colour: string;
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
  draw(o: { items: GpuItem[]; rings: GpuRing[]; dots?: GpuDots[]; sheets?: GpuSheet[]; glows?: GpuGlow[]; forward: Vec3; up: Vec3; fovY: number; width: number; height: number; ground: string; /** True look: worlds lit by their star against black space. */ light?: GpuLight; /** Black space with no light (a galaxy in True). */ space?: boolean }): void;
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
  const sphere = new SphereGeometry(1, 32, 16);
  // True look: a finer sphere with its poles on z, as the globe's (true-look.ts `equirect`).
  const globe = new SphereGeometry(1, 64, 32).rotateX(Math.PI / 2);
  const textures = new Map<string, DataTexture>();
  const litMats = new Map<string, MeshStandardMaterial>();
  const litMat = (it: GpuItem) => {
    const key = it.surface?.key ?? `plain:${it.colour}`;
    const hit = litMats.get(key);
    if (hit) return hit;
    let map: DataTexture | null = null;
    if (it.surface) {
      const t = it.surface.texture;
      map = new DataTexture(t.data, t.width, t.height, RGBAFormat);
      map.colorSpace = SRGBColorSpace;
      map.magFilter = LinearFilter;
      map.minFilter = LinearFilter;
      map.needsUpdate = true;
      textures.set(key, map);
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
  const mats = new Map<string, MeshBasicMaterial>();
  const lineMats = new Map<string, LineBasicMaterial>();
  const mat = (c: string) => mats.get(c) ?? (mats.set(c, new MeshBasicMaterial({ color: new Color(c) })), mats.get(c)!);
  const dotMats = new Map<string, PointsMaterial>();
  const dotMat = (c: string) => dotMats.get(c) ?? (dotMats.set(c, new PointsMaterial({ color: new Color(c), size: 1, sizeAttenuation: false })), dotMats.get(c)!);
  const lineMat = (c: string) => lineMats.get(c) ?? (lineMats.set(c, new LineBasicMaterial({ color: new Color(c) })), lineMats.get(c)!);
  return {
    backend,
    draw(o) {
      scene.clear();
      // Space is black in the true look: a physical choice, not a theme colour.
      scene.background = new Color(o.light || o.space ? 0x000000 : o.ground);
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
        scene.add(new AmbientLight(0xffffff, 0.04));
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
        const m = new Mesh(o.light ? globe : sphere, it.emit !== undefined ? greyMat(it.emit) : lit ? litMat(it) : mat(it.colour));
        m.position.set(it.rel[0], it.rel[1], it.rel[2]);
        m.scale.setScalar(it.radius);
        scene.add(m);
        if (it.ring) {
          const r = new Mesh(torus, mat(it.colour));
          r.position.copy(m.position);
          r.scale.setScalar(it.radius);
          scene.add(r);
        }
      }
      for (const ring of o.rings) {
        const g = new BufferGeometry();
        g.setAttribute("position", new Float32BufferAttribute(ring.points.flat(), 3));
        scene.add(new Line(g, lineMat(ring.colour)));
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
        scene.add(new Points(g, dotMat(d.colour)));
      }
      camera.fov = (o.fovY * 180) / Math.PI;
      camera.aspect = o.width / Math.max(1, o.height);
      camera.position.set(0, 0, 0);
      camera.up.set(o.up[0], o.up[1], o.up[2]);
      camera.lookAt(new Vector3(o.forward[0], o.forward[1], o.forward[2]));
      camera.updateProjectionMatrix();
      renderer.setSize(o.width, o.height, false);
      renderer.render(scene, camera);
      scene.traverse((x) => (x instanceof Line || x instanceof Points) && x.geometry.dispose());
    },
    dispose() {
      sphere.dispose();
      globe.dispose();
      torus.dispose();
      textures.forEach((t) => t.dispose());
      faces.forEach((e) => (e.tex.dispose(), e.mat.dispose()));
      litMats.forEach((m) => m.dispose());
      mats.forEach((m) => m.dispose());
      greys.forEach((m) => m.dispose());
      pics.forEach((e) => e.tex.dispose());
      lightMats.forEach((m) => m.dispose());
      glowMats.forEach((m) => m.dispose());
      sheetGeoms.forEach((e) => e.geom.dispose());
      lineMats.forEach((m) => m.dispose());
      dotMats.forEach((m) => m.dispose());
      renderer.dispose();
    },
  };
}
