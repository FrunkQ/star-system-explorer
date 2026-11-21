// The GPU path: three.js WebGPURenderer (one shader source; WebGL2 when the
// device has no WebGPU). It is handed camera-relative positions only, already
// narrowed from f64 (frames.ts), and draws with the camera at the origin, so
// no large number ever reaches the GPU (rendering rule 3).
import { BufferGeometry, Color, Float32BufferAttribute, Line, LineBasicMaterial, Mesh, MeshBasicMaterial, PerspectiveCamera, Points, PointsMaterial, Scene, SphereGeometry, TorusGeometry, Vector3 } from "three";
import { WebGPURenderer } from "three/webgpu";
import type { Vec3 } from "./frames.ts";

export interface GpuItem {
  id: string;
  /** Camera-relative centre (float32 values). */
  rel: Vec3;
  radius: number;
  colour: string;
  ring?: boolean;
  selected?: boolean;
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

export interface Gpu {
  readonly backend: "webgpu" | "webgl2";
  draw(o: { items: GpuItem[]; rings: GpuRing[]; dots?: GpuDots[]; forward: Vec3; up: Vec3; fovY: number; width: number; height: number; ground: string }): void;
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
      scene.background = new Color(o.ground);
      for (const it of o.items) {
        const m = new Mesh(sphere, mat(it.colour));
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
      torus.dispose();
      mats.forEach((m) => m.dispose());
      lineMats.forEach((m) => m.dispose());
      dotMats.forEach((m) => m.dispose());
      renderer.dispose();
    },
  };
}
