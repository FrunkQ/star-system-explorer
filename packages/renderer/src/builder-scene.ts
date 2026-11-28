// The ship builder's working view (CM14b, to the mock-up Alex accepted: sse4/handover-screens/CM-builder-mockup/):
// the build in 3D, part by part so a part can be pressed and painted; the blueprint room behind it with the side, top
// and front drawings cast on its walls by a camera that never moves (ruling 14A.2: the blueprint does not turn with
// the model); and the glowing connection points while a part is being placed. Geometry and lookups only: the pieces
// come from ship-build.ts through the same ship-mesh.ts every ship uses, and nothing here works out a figure.
import { AdditiveBlending, AmbientLight, BufferAttribute, BufferGeometry, CanvasTexture, Color, DataTexture, DirectionalLight, DoubleSide, Float32BufferAttribute, Group, HemisphereLight, LinearMipmapLinearFilter, LineBasicMaterial, LineSegments, Mesh, MeshBasicMaterial, MeshStandardMaterial, type Object3D, PerspectiveCamera, PlaneGeometry, Raycaster, RepeatWrapping, RGBAFormat, Scene, Sprite, SpriteMaterial, SRGBColorSpace, Vector2, Vector3 } from "three";
import { WebGPURenderer } from "three/webgpu";
import { panelTexture, shipMesh, type SkinSpec, type Surface } from "./ship-mesh.ts";
import type { ShipLayout } from "./ship-design.ts";
import { sayNumber } from "../../ui-contract/src/index.ts";

/** A connection point on offer, coloured by its kind as the mock-up's legend. */
export interface BuilderMarker {
  at: [number, number, number];
  kind: "end" | "hull" | "outside" | "bay";
}

export interface BuilderShow {
  layout: ShipLayout;
  /** The part being placed, drawn as a see-through ghost where it would go. */
  ghost?: string | null;
  selected?: readonly string[];
  /** Part key to a colour (#rrggbb) for its hull and tanks. */
  paints?: Readonly<Record<string, string>>;
  markers?: readonly BuilderMarker[];
  hover?: number | null;
  wire?: boolean;
  /** Reframe the camera on the build (a ship opened, or "Seen from" pressed). */
  frame?: boolean;
}

export interface BuilderCamera {
  yaw_deg: number;
  pitch_deg: number;
  zoom: number;
  pan: [number, number];
}

export interface BuilderScene {
  readonly backend: "webgpu" | "webgl2";
  show(s: BuilderShow): void;
  draw(c: BuilderCamera, width: number, height: number): void;
  /** What is under a point of the canvas (CSS pixels): a connection point first, then a part. */
  pick(x: number, y: number, width: number, height: number): { marker: number | null; part: string | null };
  /** Where a point of the build falls on the canvas, in CSS pixels. */
  project(p: [number, number, number], width: number, height: number): [number, number];
  dispose(): void;
}

const ACCENT = new Color("#7FB2FF");
const MARK: Record<BuilderMarker["kind"], number> = { end: 0x7fb2ff, hull: 0x6fd39a, outside: 0xf2b35b, bay: 0xc3a6ff };
const fmt = (n: number) => sayNumber(n, 0, { decimals: n >= 100 ? 0 : 1 });
const niceStep = (x: number) => {
  const p = 10 ** Math.floor(Math.log10(x));
  const n = x / p;
  return (n < 1.5 ? 1 : n < 3.5 ? 2 : n < 7.5 ? 5 : 10) * p;
};

function ringTexture(): CanvasTexture {
  const c = document.createElement("canvas");
  c.width = c.height = 64;
  const x = c.getContext("2d")!;
  x.strokeStyle = "#fff";
  x.lineWidth = 7;
  x.beginPath();
  x.arc(32, 32, 22, 0, 7);
  x.stroke();
  x.fillStyle = "rgba(255,255,255,.35)";
  x.beginPath();
  x.arc(32, 32, 10, 0, 7);
  x.fill();
  const t = new CanvasTexture(c);
  t.colorSpace = SRGBColorSpace;
  return t;
}

function textPlane(text: string, h: number): Mesh {
  const c = document.createElement("canvas");
  const x = c.getContext("2d")!;
  const font = '500 64px "IBM Plex Mono", ui-monospace, monospace';
  x.font = font;
  const w = Math.ceil(x.measureText(text).width) + 20;
  c.width = w;
  c.height = 90;
  x.font = font;
  x.fillStyle = "#A9CBFF";
  x.textBaseline = "middle";
  x.fillText(text, 10, 46);
  const t = new CanvasTexture(c);
  t.colorSpace = SRGBColorSpace;
  const g = new PlaneGeometry((h * w) / 90, h);
  g.translate((h * w) / 90 / 2, 0, 0);
  return new Mesh(g, new MeshBasicMaterial({ map: t, transparent: true, depthWrite: false, opacity: 0.85, side: DoubleSide }));
}

/** Mount the builder's view on a canvas. Returns null where the device draws no 3D. */
export async function mountBuilder(canvas: HTMLCanvasElement, skin: SkinSpec, o: { segments: number }): Promise<BuilderScene | null> {
  let renderer: WebGPURenderer;
  try {
    renderer = new WebGPURenderer({ canvas, antialias: true, alpha: true, logarithmicDepthBuffer: true });
    await renderer.init();
  } catch {
    return null;
  }
  renderer.autoClear = false;
  renderer.setClearColor(0x000000, 0);
  renderer.setPixelRatio(Math.min(globalThis.devicePixelRatio ?? 1, 2));
  const backend = (renderer.backend as { isWebGPUBackend?: boolean }).isWebGPUBackend ? "webgpu" : "webgl2";

  const px = 128;
  const tex = new DataTexture(panelTexture(1, px, skin), px, px, RGBAFormat);
  tex.wrapS = tex.wrapT = RepeatWrapping;
  tex.repeat.set(0.25, 0.25);
  tex.colorSpace = SRGBColorSpace;
  tex.generateMipmaps = true;
  tex.minFilter = LinearMipmapLinearFilter;
  tex.needsUpdate = true;
  const ring = ringTexture();

  const scene = new Scene();
  scene.add(new HemisphereLight(0xcfe0ff, 0x1a1d24, 0.9));
  const sun = new DirectionalLight(0xfff4e6, 2.6);
  sun.position.set(0.5, 0.7, 0.6);
  scene.add(sun, new AmbientLight(0xffffff, 0.2));
  const bg = new Scene();
  const marks = new Scene();
  const cam = new PerspectiveCamera(35, 1, 0.05, 1e6);
  const bgCam = new PerspectiveCamera(35, 1, 0.05, 1e6);

  const ghostMat = new MeshBasicMaterial({ color: ACCENT, transparent: true, opacity: 0.38, depthWrite: false });
  const shadowFill = new MeshBasicMaterial({ color: ACCENT, transparent: true, opacity: 0.06, depthWrite: false, side: DoubleSide });
  const lineMat = (opacity: number) => new LineBasicMaterial({ color: ACCENT, transparent: true, opacity, depthWrite: false });

  let ship = new Group();
  let room = new Group();
  let markers: Sprite[] = [];
  let parts: { key: string; mesh: Mesh }[] = [];
  const owned: { dispose(): void }[] = [];
  const disposeAll = () => {
    for (const d of owned.splice(0)) d.dispose();
  };
  let centre = new Vector3();
  let radius = 10;
  let box = { lo: new Vector3(-5, -3, -3), hi: new Vector3(5, 3, 3) };
  let size = { w: 1, h: 1 };
  let roomFor = "";

  const geometryOf = (b: NonNullable<ReturnType<typeof shipMesh>["surfaces"][Surface]>) => {
    const g = new BufferGeometry();
    g.setAttribute("position", new BufferAttribute(b.positions, 3));
    g.setAttribute("normal", new BufferAttribute(b.normals, 3));
    g.setAttribute("uv", new BufferAttribute(b.uvs, 2));
    g.setIndex(new BufferAttribute(b.indices, 1));
    owned.push(g);
    return g;
  };

  function buildRoom(): void {
    bg.remove(room);
    room = new Group();
    bg.add(room);
    // The build's frame: x along the ship (nose to the viewer's right), y away from the viewer, z up.
    const lo = box.lo;
    const hi = box.hi;
    const len = hi.x - lo.x;
    const hy = hi.y - lo.y;
    const hz = hi.z - lo.z;
    const cx = (lo.x + hi.x) / 2;
    const R = Math.max(hy, hz) / 2 + Math.max(2, len * 0.22);
    const pad = Math.max(2, len * 0.2);
    const X0 = lo.x - pad;
    const X1 = hi.x + pad;
    const Y1 = R;
    const Y0 = -R;
    const Z0 = -R;
    const step = niceStep(Math.max(len, hy, hz) / 14);
    // A grid on a wall: (u, v) in [u0, u1] x [v0, v1] mapped into the room, every fifth line stronger, with its frame.
    const grid = (u0: number, u1: number, v0: number, v1: number, at: (u: number, v: number) => number[]) => {
      const minor: number[] = [];
      const major: number[] = [];
      for (let u = Math.ceil(u0 / step) * step; u <= u1; u += step) (Math.round(u / step) % 5 ? minor : major).push(...at(u, v0), ...at(u, v1));
      for (let v = Math.ceil(v0 / step) * step; v <= v1; v += step) (Math.round(v / step) % 5 ? minor : major).push(...at(u0, v), ...at(u1, v));
      const frame = [...at(u0, v0), ...at(u1, v0), ...at(u1, v0), ...at(u1, v1), ...at(u1, v1), ...at(u0, v1), ...at(u0, v1), ...at(u0, v0)];
      const g = new Group();
      for (const [pts, op] of [[minor, 0.07], [major, 0.17], [frame, 0.3]] as [number[], number][]) {
        const bgeo = new BufferGeometry();
        bgeo.setAttribute("position", new Float32BufferAttribute(pts, 3));
        const m = lineMat(op);
        owned.push(bgeo, m);
        g.add(new LineSegments(bgeo, m));
      }
      return g;
    };
    // The build flattened onto a wall: its own parts, drawn as a faint fill.
    const flat = () => {
      const g = new Group();
      for (const p of parts) g.add(new Mesh(p.mesh.geometry, shadowFill));
      return g;
    };
    const th = Math.max(0.6, len * 0.036);
    // A label lying on a wall, facing `face`, its letters' up along `up`.
    const label = (t: string, at: Vector3, face: Vector3, up: Vector3) => {
      const m = textPlane(t, th);
      owned.push(m.geometry, m.material as MeshBasicMaterial, (m.material as MeshBasicMaterial).map!);
      m.position.copy(at);
      m.up.copy(up);
      m.lookAt(at.clone().add(face));
      return m;
    };
    const Z = new Vector3(0, 0, 1);
    // The far wall: the side drawing (y flattened).
    const side = flat();
    side.scale.y = 0.0005;
    side.position.y = Y1 - 0.05;
    const sg = grid(X0, X1, Z0, R, (u, v) => [u, Y1, v]);
    const sl = label(`SIDE · ${fmt(len)} m long · ${fmt(hz)} m high`, new Vector3(X0 + th, Y1 - 0.1, R - th * 1.2), new Vector3(0, -1, 0), Z);
    // The floor: the top drawing (z flattened).
    const top = flat();
    top.scale.z = 0.0005;
    top.position.z = Z0 + 0.05;
    const tg = grid(X0, X1, Y0, Y1, (u, v) => [u, v, Z0]);
    const tl = label(`TOP · ${fmt(len)} m long · ${fmt(hy)} m across`, new Vector3(X0 + th, Y1 - th * 1.6, Z0 + 0.1), Z, new Vector3(0, 1, 0));
    // The end wall: the front drawing (x flattened), at the far end from the viewer.
    const end = flat();
    end.scale.x = 0.0005;
    end.position.x = X0 + 0.05;
    const eg = grid(Y0, Y1, Z0, R, (u, v) => [X0, u, v]);
    const el = label(`FRONT · ${fmt(Math.max(hy, hz))} m across`, new Vector3(X0 + 0.1, Y0 + th, R - th * 1.2), new Vector3(1, 0, 0), Z);
    room.add(side, sg, sl, top, tg, tl, end, eg, el);
    // The fixed three-quarter camera that frames the room's corner from the viewer's side (the -y side, past the nose).
    const c = new Vector3(cx, 0, 0);
    const rr = 0.5 * Math.hypot(X1 - X0, 2 * R, 2 * R);
    bgCam.aspect = Math.max(0.3, size.w / Math.max(1, size.h));
    bgCam.near = rr * 0.01;
    bgCam.far = rr * 10;
    bgCam.up.copy(Z);
    const half = (bgCam.fov * Math.PI) / 360;
    const fit = Math.min(half, Math.atan(Math.tan(half) * bgCam.aspect));
    const D = (rr / Math.sin(fit)) * 0.92;
    bgCam.position.copy(c).addScaledVector(new Vector3(0.62, -0.69, 0.38).normalize(), D);
    bgCam.lookAt(c);
    bgCam.updateProjectionMatrix();
  }

  return {
    backend,
    show(s) {
      scene.remove(ship);
      disposeAll();
      ship = new Group();
      scene.add(ship);
      parts = [];
      const sel = new Set(s.selected ?? []);
      const keys = [...new Set(s.layout.pieces.map((p) => p.member ?? ""))];
      for (const key of keys) {
        const pieces = s.layout.pieces.filter((p) => (p.member ?? "") === key);
        if (!pieces.length) continue;
        const m = shipMesh({ ...s.layout, pieces }, { segments: o.segments, panel_m: skin.panel_m });
        for (const [name, b] of Object.entries(m.surfaces) as [Surface, NonNullable<(typeof m.surfaces)[Surface]>][]) {
          const look = (name === "shield" ? skin.surfaces.shield : skin.surfaces[name]) ?? skin.surfaces.hull;
          const ghost = key === s.ghost;
          const paint = s.paints?.[key];
          const mat = ghost ? ghostMat : new MeshStandardMaterial({ color: paint && (name === "hull" || name === "tanks") ? new Color(paint) : new Color(...look.albedo), roughness: look.roughness, metalness: look.metalness, map: tex, wireframe: s.wire === true });
          if (!ghost) {
            owned.push(mat);
            if (sel.has(key)) ((mat as MeshStandardMaterial).emissive = ACCENT.clone()), ((mat as MeshStandardMaterial).emissiveIntensity = 0.35);
          }
          const mesh = new Mesh(geometryOf(b), mat);
          mesh.userData.key = ghost ? null : key;
          ship.add(mesh);
          if (!ghost) parts.push({ key, mesh });
        }
      }
      ship.updateMatrixWorld(true);
      // Frame on the build without its ghost, so the camera holds still while a part is tried at each point.
      const lo = new Vector3(Infinity, Infinity, Infinity);
      const hi = new Vector3(-Infinity, -Infinity, -Infinity);
      for (const p of s.layout.pieces) {
        if (p.member === s.ghost) continue;
        const r = p.shape === "body" ? Math.max(p.r0, p.r1) : Math.abs(p.span ?? 0);
        lo.min(new Vector3(Math.min(p.x0, p.x1), p.y - r, p.z - r));
        hi.max(new Vector3(Math.max(p.x0, p.x1), p.y + r, p.z + r));
      }
      if (!Number.isFinite(lo.x)) lo.set(-5, -3, -3), hi.set(5, 3, 3);
      box = { lo, hi };
      const shape = `${[...lo.toArray(), ...hi.toArray()].map((v) => Math.round(v * 100))}|${parts.length}|${s.wire}`;
      if (s.frame || roomFor !== shape) {
        centre = new Vector3((lo.x + hi.x) / 2, (lo.y + hi.y) / 2, (lo.z + hi.z) / 2);
        radius = 0.5 * Math.hypot(hi.x - lo.x, hi.y - lo.y, hi.z - lo.z);
      }
      if (roomFor !== shape) ((roomFor = shape), buildRoom());
      // The connection points: rings that always face the camera and draw over the build.
      for (const m of markers) marks.remove(m);
      markers = [];
      const ringSize = Math.max(0.5, (hi.x - lo.x) * 0.03);
      (s.markers ?? []).forEach((mk, i) => {
        const sm = new SpriteMaterial({ map: ring, color: MARK[mk.kind], depthTest: false, transparent: true, blending: i === s.hover ? AdditiveBlending : undefined });
        owned.push(sm);
        const sp = new Sprite(sm);
        const k = i === s.hover ? 1.5 : 1;
        sp.scale.set(ringSize * k, ringSize * k, 1);
        sp.position.set(...mk.at);
        sp.userData.index = i;
        marks.add(sp);
        markers.push(sp);
      });
    },
    draw(c, width, height) {
      size = { w: width, h: height };
      renderer.setSize(width, height, false);
      cam.aspect = width / Math.max(1, height);
      if (Math.abs(bgCam.aspect - Math.max(0.3, cam.aspect)) > 0.01) buildRoom();
      const half = (cam.fov * Math.PI) / 360;
      const fit = Math.min(half, Math.atan(Math.tan(half) * cam.aspect));
      const dist = (radius / Math.sin(fit)) * 1.15 / c.zoom;
      const yaw = (c.yaw_deg * Math.PI) / 180;
      const pitch = (c.pitch_deg * Math.PI) / 180;
      // z up, as every ship view: yaw about z from +x, pitch above the x-y plane.
      const dir = new Vector3(Math.cos(pitch) * Math.cos(yaw), Math.cos(pitch) * Math.sin(yaw), Math.sin(pitch));
      const right = new Vector3(-Math.sin(yaw), Math.cos(yaw), 0);
      const up = new Vector3().crossVectors(dir, right).normalize();
      const at = centre.clone().addScaledVector(right, (c.pan[0] * radius) / c.zoom).addScaledVector(up, (c.pan[1] * radius) / c.zoom);
      cam.position.copy(at).addScaledVector(dir, dist);
      cam.near = Math.max(dist - radius * 4, radius * 1e-3);
      cam.far = dist + radius * 8;
      cam.up.copy(up);
      cam.lookAt(at);
      cam.updateProjectionMatrix();
      renderer.clear();
      renderer.render(bg, bgCam);
      renderer.clearDepth();
      renderer.render(scene, cam);
      renderer.clearDepth();
      renderer.render(marks, cam);
    },
    pick(x, y, width, height) {
      const ray = new Raycaster();
      ray.setFromCamera(new Vector2((x / width) * 2 - 1, -(y / height) * 2 + 1), cam);
      const hit = ray.intersectObjects(markers as Object3D[], false)[0];
      if (hit) return { marker: hit.object.userData.index as number, part: null };
      const ph = ray.intersectObjects(parts.map((p) => p.mesh), false)[0];
      return { marker: null, part: (ph?.object.userData.key as string | undefined) ?? null };
    },
    project(p, width, height) {
      const v = new Vector3(...p).project(cam);
      return [((v.x + 1) / 2) * width, ((1 - v.y) / 2) * height];
    },
    dispose() {
      disposeAll();
      for (const m of [ghostMat, shadowFill]) m.dispose();
      tex.dispose();
      ring.dispose();
      renderer.dispose();
    },
  };
}
