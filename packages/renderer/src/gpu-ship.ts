// V3 gap item 14, stage 3: draw a designed ship in 3D (WebGPU, else WebGL2). The buffers come from ship-mesh.ts,
// built once from the layout; this only wraps them in materials from the style's skin, lights them with one star
// and turns the camera. Hot radiators glow with the colour of a body at their temperature (Planck, through the
// sky's thermal colour table), dimmed by the style of display rather than physics: the glow is there to be seen.
import { AdditiveBlending, AmbientLight, BufferAttribute, BufferGeometry, Color, DataTexture, DirectionalLight, DoubleSide, LatheGeometry, LinearMipmapLinearFilter, Mesh, MeshBasicMaterial, MeshStandardMaterial, PerspectiveCamera, RepeatWrapping, RGBAFormat, Scene, SRGBColorSpace, Vector2, Vector3 } from "three";
import { WebGPURenderer } from "three/webgpu";
import { panelTexture, shipMesh, type ShipMesh, type SkinSpec, type Surface } from "./ship-mesh.ts";
import { starRgb } from "./sky.ts";
import type { ShipLayout } from "./ship-design.ts";

export interface ShipView3D {
  readonly backend: "webgpu" | "webgl2";
  readonly mesh: ShipMesh;
  /** Turn the camera: yaw and pitch in degrees about the ship's centre; `zoom` 1 frames the whole ship, or with
   * `frame: "hull"` the ship without its radiators (which may be kilometres across). */
  draw(o: { yaw_deg: number; pitch_deg: number; zoom?: number; frame?: "all" | "hull" | "front"; /** Slide across the view, in framed radii: [right, up]. */ pan?: [number, number]; width: number; height: number; ground: string; glow?: boolean }): void;
  dispose(): void;
}

/** Build and draw a ship once; later draws only move the camera. `segments` comes from the device tier. */
export async function mountShip(canvas: HTMLCanvasElement, l: ShipLayout, skin: SkinSpec, o: { segments: number; seed: number; /** See-through ground (the builder draws its fixed blueprint behind the model, ruling 14A.2): `draw` with ground "transparent". */ clear?: boolean; /** An aerobraking pass at its hottest (ship-abilities passGlow): radiators folded away, the shield glowing at its face temperature, the bow shock ahead. */ pass?: { shield_k: number; standoff_rn: number; shock_curve_rn: number; glow: number } }): Promise<ShipView3D | null> {
  let renderer: WebGPURenderer;
  try {
    // Kilometre radiators beside metre-scale modules need a logarithmic depth buffer to sort cleanly.
    renderer = new WebGPURenderer({ canvas, antialias: true, alpha: o.clear === true, logarithmicDepthBuffer: true });
    await renderer.init();
  } catch {
    return null;
  }
  const backend = (renderer.backend as { isWebGPUBackend?: boolean }).isWebGPUBackend ? "webgpu" : "webgl2";
  const mesh = shipMesh(l, { segments: o.segments, panel_m: skin.panel_m });
  const scene = new Scene();
  const px = 128;
  const tex = new DataTexture(panelTexture(o.seed, px, skin), px, px, RGBAFormat);
  tex.wrapS = tex.wrapT = RepeatWrapping;
  // The texture covers 4 x 4 panels; the buffers' coordinates count panels.
  tex.repeat.set(0.25, 0.25);
  tex.colorSpace = SRGBColorSpace;
  // Kilometre radiators carry thousands of panels: mipmaps keep the seams from shimmering at a distance.
  tex.generateMipmaps = true;
  tex.minFilter = LinearMipmapLinearFilter;
  tex.anisotropy = 8;
  tex.needsUpdate = true;
  const heat = l.needs.radiator_k;
  const materials: MeshStandardMaterial[] = [];
  const glowMaterials: MeshBasicMaterial[] = [];
  const geometries: BufferGeometry[] = [];
  for (const [name, b] of Object.entries(mesh.surfaces) as [Surface, NonNullable<ShipMesh["surfaces"][Surface]>][]) {
    // In a pass the radiators are folded behind the shield (ship-abilities notes), so they are not drawn.
    if (o.pass && name === "radiators") continue;
    const g = new BufferGeometry();
    g.setAttribute("position", new BufferAttribute(b.positions, 3));
    g.setAttribute("normal", new BufferAttribute(b.normals, 3));
    g.setAttribute("uv", new BufferAttribute(b.uvs, 2));
    g.setIndex(new BufferAttribute(b.indices, 1));
    const s = (name === "shield" ? skin.surfaces.shield : skin.surfaces[name]) ?? skin.surfaces.hull;
    const m = new MeshStandardMaterial({ color: new Color(...s.albedo), roughness: s.roughness, metalness: s.metalness, map: tex });
    if (name === "shield" && o.pass) {
      m.emissive = new Color(...starRgb(o.pass.shield_k));
      m.userData.glow = 0.4 + 1.6 * o.pass.glow;
    }
    if (name === "radiators" && heat > 0) {
      m.emissive = new Color(...starRgb(heat));
      m.emissiveMap = tex;
      m.userData.glow = Math.min(2, 0.6 * (heat / 1500) ** 4);
    }
    materials.push(m);
    geometries.push(g);
    scene.add(new Mesh(g, m));
  }
  // The bow shock: a paraboloid shell standing off the shield's nose, drawn as glowing gas (a display choice).
  const shieldPiece = l.pieces.find((p) => p.system === "shield");
  if (o.pass && shieldPiece && l.fit?.shield) {
    const rn = l.fit.shield.nose_r;
    const front = shieldPiece.x1 + o.pass.standoff_rn * rn;
    const rs = o.pass.shock_curve_rn * rn;
    const reach = 1.6 * shieldPiece.r0;
    const pts = Array.from({ length: 24 }, (_, i) => {
      const r = (reach * i) / 23;
      return new Vector2(r, front - (r * r) / (2 * rs));
    });
    const g = new LatheGeometry(pts, o.segments);
    // The lathe turns about its own y axis; the ship's axis is x.
    g.rotateZ(-Math.PI / 2);
    const m = new MeshBasicMaterial({ color: new Color(...starRgb(Math.min(12000, 1.6 * o.pass.shield_k))), transparent: true, opacity: 0.15 + 0.45 * o.pass.glow, blending: AdditiveBlending, side: DoubleSide, depthWrite: false });
    geometries.push(g);
    scene.add(new Mesh(g, m));
    glowMaterials.push(m);
  }
  const sun = new DirectionalLight(0xffffff, 3);
  sun.position.set(0.4, 0.8, 0.45);
  scene.add(sun, new AmbientLight(0xffffff, 0.25));
  const sphere = ([lo, hi]: ShipMesh["bounds"]) => ({
    centre: new Vector3((lo[0] + hi[0]) / 2, (lo[1] + hi[1]) / 2, (lo[2] + hi[2]) / 2),
    radius: 0.5 * Math.hypot(hi[0] - lo[0], hi[1] - lo[1], hi[2] - lo[2]),
  });
  const whole = sphere(mesh.bounds);
  const hull = sphere(mesh.hull_bounds);
  const front = Number.isFinite(mesh.front_bounds[0][0]) ? sphere(mesh.front_bounds) : hull;
  const camera = new PerspectiveCamera(35, 1, hull.radius * 1e-2, whole.radius * 20);
  return {
    backend,
    mesh,
    draw(v) {
      renderer.setSize(v.width, v.height, false);
      camera.aspect = v.width / v.height;
      const { centre, radius } = v.frame === "hull" ? hull : v.frame === "front" ? front : whole;
      const dist = radius / Math.sin((camera.fov * Math.PI) / 360) / (v.zoom ?? 1);
      const yaw = (v.yaw_deg * Math.PI) / 180;
      const pitch = (v.pitch_deg * Math.PI) / 180;
      // Slide the point looked at across the view: right is along -sin(yaw), cos(yaw); up is the tilted vertical.
      const [px, py] = v.pan ?? [0, 0];
      const right = new Vector3(-Math.sin(yaw), Math.cos(yaw), 0);
      const up = new Vector3(-Math.sin(pitch) * Math.cos(yaw), -Math.sin(pitch) * Math.sin(yaw), Math.cos(pitch));
      const at = centre.clone().addScaledVector(right, (px * radius) / (v.zoom ?? 1)).addScaledVector(up, (py * radius) / (v.zoom ?? 1));
      camera.position.set(at.x + dist * Math.cos(pitch) * Math.cos(yaw), at.y + dist * Math.cos(pitch) * Math.sin(yaw), at.z + dist * Math.sin(pitch));
      camera.up.set(0, 0, 1);
      camera.lookAt(at);
      camera.updateProjectionMatrix();
      for (const m of materials) m.emissiveIntensity = v.glow === false ? 0 : ((m.userData.glow as number | undefined) ?? 0);
      scene.background = v.ground === "transparent" ? null : new Color(v.ground);
      renderer.render(scene, camera);
    },
    dispose() {
      for (const g of geometries) g.dispose();
      for (const m of materials) m.dispose();
      for (const m of glowMaterials) m.dispose();
      tex.dispose();
      renderer.dispose();
    },
  };
}
