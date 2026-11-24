// Draw a station in 3D (WebGPU, else WebGL2): the buffers from station-mesh.ts in materials from the base-ships
// `station_skin`, lit by one star, the camera turned about the station. Built once; later draws only move the camera.
import { AmbientLight, BufferAttribute, BufferGeometry, Color, DataTexture, DirectionalLight, DoubleSide, LinearMipmapLinearFilter, Mesh, MeshStandardMaterial, PerspectiveCamera, RepeatWrapping, RGBAFormat, Scene, SRGBColorSpace, Vector3 } from "three";
import { WebGPURenderer } from "three/webgpu";
import { panelTexture, type SurfaceLook } from "./ship-mesh.ts";
import { stationMesh, type StationMesh } from "./station-mesh.ts";
import type { StationLayout, StationSurface } from "./station-design.ts";

/** base-ships `station_skin`. */
export interface StationSkin {
  panel_m: number;
  patchiness: number;
  wear: number;
  surfaces: Record<StationSurface, SurfaceLook>;
}

export interface StationView3D {
  readonly backend: "webgpu" | "webgl2";
  readonly mesh: StationMesh;
  /** Yaw and pitch in degrees about the station; `zoom` 1 frames it all, or with `frame: "core"` the station
   * without its solar and radiator arrays (which may be kilometres across). */
  draw(o: { yaw_deg: number; pitch_deg: number; zoom?: number; frame?: "all" | "core"; pan?: [number, number]; width: number; height: number; ground: string }): void;
  dispose(): void;
}

export async function mountStation(canvas: HTMLCanvasElement, l: StationLayout, skin: StationSkin, o: { segments: number; seed: number }): Promise<StationView3D | null> {
  let renderer: WebGPURenderer;
  try {
    renderer = new WebGPURenderer({ canvas, antialias: true, alpha: false, logarithmicDepthBuffer: true });
    await renderer.init();
  } catch {
    return null;
  }
  const backend = (renderer.backend as { isWebGPUBackend?: boolean }).isWebGPUBackend ? "webgpu" : "webgl2";
  const mesh = stationMesh(l, { segments: o.segments, panel_m: skin.panel_m });
  const scene = new Scene();
  const px = 128;
  const tex = new DataTexture(panelTexture(o.seed, px, skin), px, px, RGBAFormat);
  tex.wrapS = tex.wrapT = RepeatWrapping;
  tex.repeat.set(0.25, 0.25);
  tex.colorSpace = SRGBColorSpace;
  tex.generateMipmaps = true;
  tex.minFilter = LinearMipmapLinearFilter;
  tex.anisotropy = 8;
  tex.needsUpdate = true;
  const materials: MeshStandardMaterial[] = [];
  const geometries: BufferGeometry[] = [];
  for (const [name, b] of Object.entries(mesh.surfaces) as [StationSurface, NonNullable<StationMesh["surfaces"][StationSurface]>][]) {
    const g = new BufferGeometry();
    g.setAttribute("position", new BufferAttribute(b.positions, 3));
    g.setAttribute("normal", new BufferAttribute(b.normals, 3));
    g.setAttribute("uv", new BufferAttribute(b.uvs, 2));
    g.setIndex(new BufferAttribute(b.indices, 1));
    const s = skin.surfaces[name] ?? skin.surfaces.hull;
    // Arrays are thin sheets seen from both sides.
    const m = new MeshStandardMaterial({ color: new Color(...s.albedo), roughness: s.roughness, metalness: s.metalness, map: tex, ...(name === "solar" || name === "radiators" ? { side: DoubleSide } : {}) });
    materials.push(m);
    geometries.push(g);
    scene.add(new Mesh(g, m));
  }
  const sun = new DirectionalLight(0xffffff, 3);
  sun.position.set(0.5, 0.35, 0.8);
  scene.add(sun, new AmbientLight(0xffffff, 0.25));
  const sphere = ([lo, hi]: StationMesh["bounds"]) => ({
    centre: new Vector3((lo[0] + hi[0]) / 2, (lo[1] + hi[1]) / 2, (lo[2] + hi[2]) / 2),
    radius: Math.max(1, 0.5 * Math.hypot(hi[0] - lo[0], hi[1] - lo[1], hi[2] - lo[2])),
  });
  const whole = sphere(mesh.bounds);
  const core = sphere(mesh.core_bounds);
  const camera = new PerspectiveCamera(35, 1, core.radius * 1e-2, whole.radius * 20);
  return {
    backend,
    mesh,
    draw(v) {
      renderer.setSize(v.width, v.height, false);
      camera.aspect = v.width / v.height;
      const { centre, radius } = v.frame === "core" ? core : whole;
      const zoom = v.zoom ?? 1;
      const dist = radius / Math.sin((camera.fov * Math.PI) / 360) / zoom;
      const yaw = (v.yaw_deg * Math.PI) / 180;
      const pitch = (v.pitch_deg * Math.PI) / 180;
      const [px2, py2] = v.pan ?? [0, 0];
      const right = new Vector3(-Math.sin(yaw), Math.cos(yaw), 0);
      const up = new Vector3(-Math.sin(pitch) * Math.cos(yaw), -Math.sin(pitch) * Math.sin(yaw), Math.cos(pitch));
      const look = centre.clone().addScaledVector(right, (px2 * radius) / zoom).addScaledVector(up, (py2 * radius) / zoom);
      camera.position.set(look.x + dist * Math.cos(pitch) * Math.cos(yaw), look.y + dist * Math.cos(pitch) * Math.sin(yaw), look.z + dist * Math.sin(pitch));
      camera.up.set(0, 0, 1);
      camera.lookAt(look);
      camera.updateProjectionMatrix();
      scene.background = new Color(v.ground);
      renderer.render(scene, camera);
    },
    dispose() {
      for (const g of geometries) g.dispose();
      for (const m of materials) m.dispose();
      tex.dispose();
      renderer.dispose();
    },
  };
}

