// The lander on the ground (landing run-through LR6): the ship that came down, standing where it touched down, in
// frame from the walk and drone views on Land, with its lamps lit at night. The surface's own GPU path draws it
// (gpu-surface.ts `setLander`); this file builds the shape and places it. The light on it is the surface's: the hull is
// lit by the same sun, sky and exposure as the ground through the `paint` hooks, so nothing here tone-maps or picks a
// display colour. The lamps are emitters whose radiance comes in the surface's own units (the star's light at the
// top of the air is 1), so the eye's adaptation shows them faint by day and bright at night.
//
// The stock shape is a generic lander: a squat body on four splayed legs with pads, a cabin and a beacon on top,
// sized from its height alone. Where the lander is the party's own ship (LR6b), the caller passes the ship's model
// (ship-mesh.ts `shipMesh`, from its design) and its style's albedos as `ship`: it stands on its tail, its drive
// down, at its own length, its lamps on its sides near the ground.
import { BoxGeometry, BufferAttribute, BufferGeometry, CylinderGeometry, Group, Mesh, SphereGeometry, Vector3, DoubleSide, type Material } from "three";
import { MeshBasicNodeMaterial, type Node } from "three/webgpu";
import { uniform, vec3, vec4 } from "three/tsl";
import { LAMP_DISC, LAMP_SPILL, LANDER_HULL_ALBEDO } from "../../ui-contract/src/index.ts";
import type { ShipMesh, Surface } from "./ship-mesh.ts";

/** A ship's own model for the ground: its mesh (x along the ship from the stern, metres) and each surface's albedo. */
export interface LanderShip {
  mesh: ShipMesh;
  albedo: Partial<Record<Surface, readonly [number, number, number]>>;
}

/** Where the lander stands and how it looks, in the site's frame (the same metres as the surface's meshes). */
export interface LanderPose {
  /** East and north of the site, metres. */
  at_m: [number, number];
  /** The ground's height there, metres, from the patch (so it stands on the ground and does not float). */
  ground_m: number;
  /** Which way its front faces, degrees from north toward east. */
  heading_deg: number;
  /** Its height from pads to top, metres. */
  height_m: number;
  /** The party's ship's own model (LR6b), stood on its tail and scaled to `height_m` (its length); absent or null: the stock shape. */
  ship?: LanderShip | null;
  /** The lamps' radius, metres; absent: the stock shape's (LAMP_DISC of its height). A ship's own model keeps lamps the size its mass gives the stock shape. */
  lamp_m?: number;
  /** Lamps on. */
  lights: boolean;
  /** The lamps' radiance in the surface's units (the star's top-of-air light is 1), linear RGB; absent: 0. */
  lamp?: [number, number, number];
}

/** How the surface paints: a lit surface of an albedo, an emitter of a radiance, and the diagram looks' ink. */
export interface LanderPaint {
  lit(albedo: Node<"vec3">): Node<"vec4">;
  glow(radiance: Node<"vec3">): Node<"vec4">;
  ink: Node<"vec4">;
}

export interface LanderOnGround {
  readonly group: Group;
  set(pose: LanderPose | null): void;
  /** Each frame: the scene is drawn about the eye, so the lander is placed relative to it; the look picks the paint. */
  place(eye: readonly [number, number, number], look: "true" | "3d" | "wireframe"): void;
  /** The lamps' places in the site's frame, for tests and marks. */
  lamps(): [number, number, number][];
  dispose(): void;
}

/** The stock shape's parts, in units of its height (pads at z 0, top at z 1). */
const BODY = { r: 0.32, z0: 0.3, z1: 0.62 };
const CABIN = { r: 0.2, z0: 0.62, z1: 0.88 };
const LEG = { r: 0.022, foot: 0.52, pad: 0.07 };
const LAMP_Z = 0.36;
// The lamps' size and their light back on the hull are the landing's numbers (ui-contract landing.ts).
const SPILL = LAMP_SPILL;
const BEACON_Z = 0.94;

export function createLander(paint: LanderPaint): LanderOnGround {
  const group = new Group();
  group.visible = false;
  // Painted hull (a light thermal finish), darker legs and pads; albedos, not display colours.
  const uLamp = uniform(new Vector3(0, 0, 0));
  const uSpill = uniform(new Vector3(0, 0, 0));
  const finish = (alb: Node<"vec3">) => paint.lit(alb).add(paint.glow(alb.mul(uSpill).div(Math.PI)).mul(vec4(1, 1, 1, 0)));
  const hullTrue = new MeshBasicNodeMaterial();
  hullTrue.colorNode = finish(vec3(LANDER_HULL_ALBEDO, LANDER_HULL_ALBEDO * 0.97, LANDER_HULL_ALBEDO * 0.9));
  const legTrue = new MeshBasicNodeMaterial();
  legTrue.colorNode = finish(vec3(0.28, 0.27, 0.26));
  const lampMat = new MeshBasicNodeMaterial();
  lampMat.colorNode = paint.glow(uLamp);
  const inkMat = new MeshBasicNodeMaterial({ wireframe: false });
  inkMat.colorNode = paint.ink;
  const inkWire = new MeshBasicNodeMaterial({ wireframe: true });
  inkWire.colorNode = paint.ink;

  const geoms: BufferGeometry[] = [];
  const parts: { mesh: Mesh; lit: Material; lamp?: boolean }[] = [];
  const stock = new Group();
  const add = (g: BufferGeometry, lit: Material, at: [number, number, number], rot?: [number, number, number], lamp = false) => {
    geoms.push(g);
    const m = new Mesh(g, lit);
    m.position.set(...at);
    if (rot) m.rotation.set(...rot);
    m.frustumCulled = false;
    stock.add(m);
    parts.push({ mesh: m, lit, lamp });
  };
  // Cylinders stand along y in three.js; turned to stand along z (up in the site's frame).
  const up: [number, number, number] = [Math.PI / 2, 0, 0];
  add(new CylinderGeometry(BODY.r, BODY.r * 1.08, BODY.z1 - BODY.z0, 16), hullTrue, [0, 0, (BODY.z0 + BODY.z1) / 2], up);
  add(new CylinderGeometry(CABIN.r * 0.7, CABIN.r, CABIN.z1 - CABIN.z0, 12), hullTrue, [0, 0, (CABIN.z0 + CABIN.z1) / 2], up);
  add(new BoxGeometry(0.16, 0.05, 0.12), legTrue, [0, CABIN.r * 0.85, (CABIN.z0 + CABIN.z1) / 2]);
  const lampPlaces: [number, number, number][] = [];
  for (let i = 0; i < 4; i++) {
    const a = Math.PI / 4 + (i * Math.PI) / 2;
    const c = Math.cos(a);
    const s = Math.sin(a);
    // A leg from the body's rim down and out to its pad.
    const top = new Vector3(c * BODY.r * 0.9, s * BODY.r * 0.9, BODY.z0 + 0.05);
    const foot = new Vector3(c * LEG.foot, s * LEG.foot, LEG.pad * 0.3);
    const mid = top.clone().add(foot).multiplyScalar(0.5);
    const len = top.distanceTo(foot);
    const leg = new CylinderGeometry(LEG.r, LEG.r, len, 6);
    const dir = foot.clone().sub(top).normalize();
    const m = new Mesh(leg, legTrue);
    m.position.copy(mid);
    m.quaternion.setFromUnitVectors(new Vector3(0, 1, 0), dir);
    m.frustumCulled = false;
    geoms.push(leg);
    stock.add(m);
    parts.push({ mesh: m, lit: legTrue });
    add(new CylinderGeometry(LEG.pad, LEG.pad, LEG.pad * 0.3, 10), legTrue, [foot.x, foot.y, LEG.pad * 0.15], up);
    // A floodlamp on the body between each pair of legs, looking down and out.
    const b = a + Math.PI / 4;
    const lp: [number, number, number] = [Math.cos(b) * BODY.r * 1.09, Math.sin(b) * BODY.r * 1.09, LAMP_Z];
    lampPlaces.push(lp);
    add(new SphereGeometry(LAMP_DISC, 8, 6), lampMat, lp, undefined, true);
  }
  lampPlaces.push([0, 0, BEACON_Z]);
  add(new SphereGeometry(LAMP_DISC, 8, 6), lampMat, [0, 0, BEACON_Z], undefined, true);
  group.add(stock);

  // The ship's own model, when set: built once per ship, its parts painted like the stock shape's.
  const own = new Group();
  group.add(own);
  let ownParts: { mesh: Mesh; lit: Material; lamp?: boolean }[] = [];
  let ownLamps: [number, number, number][] = [];
  let ownOf: LanderShip | null = null;
  const dropOwn = () => {
    for (const p of ownParts) {
      p.mesh.geometry.dispose();
      if (!p.lamp) p.lit.dispose();
      own.remove(p.mesh);
    }
    ownParts = [];
    ownLamps = [];
  };
  const buildOwn = (sh: LanderShip) => {
    const [lo, hi] = sh.mesh.bounds;
    const len = hi[0] - lo[0];
    if (!(len > 0)) return;
    // Ship x (stern to bow) becomes up; its y and z the ground's east and north: a turn of the axes, so faces keep their winding.
    for (const [surface, b] of Object.entries(sh.mesh.surfaces) as [Surface, NonNullable<ShipMesh["surfaces"][Surface]>][]) {
      const n = b.positions.length / 3;
      const pos = new Float32Array(n * 3);
      const nor = new Float32Array(n * 3);
      for (let i = 0; i < n; i++) {
        pos[3 * i] = b.positions[3 * i + 1]! / len;
        pos[3 * i + 1] = b.positions[3 * i + 2]! / len;
        pos[3 * i + 2] = (b.positions[3 * i]! - lo[0]) / len;
        nor[3 * i] = b.normals[3 * i + 1]!;
        nor[3 * i + 1] = b.normals[3 * i + 2]!;
        nor[3 * i + 2] = b.normals[3 * i]!;
      }
      const g = new BufferGeometry();
      g.setAttribute("position", new BufferAttribute(pos, 3));
      g.setAttribute("normal", new BufferAttribute(nor, 3));
      g.setIndex(new BufferAttribute(b.indices, 1));
      const a = sh.albedo[surface] ?? [LANDER_HULL_ALBEDO, LANDER_HULL_ALBEDO, LANDER_HULL_ALBEDO];
      const lit = new MeshBasicNodeMaterial({ side: DoubleSide });
      lit.colorNode = finish(vec3(a[0], a[1], a[2]));
      const m = new Mesh(g, lit);
      m.frustumCulled = false;
      own.add(m);
      ownParts.push({ mesh: m, lit });
    }
    // Lamps on the hull's sides a little above the drive, looking out, and a beacon on the bow.
    const [hlo, hhi] = sh.mesh.hull_bounds;
    const r = Math.max(hhi[1] - hlo[1], hhi[2] - hlo[2]) / 2 / len;
    const z = Math.min(0.5, (hlo[0] - lo[0]) / len + 0.06);
    for (let i = 0; i < 4; i++) {
      const b = Math.PI / 2 + (i * Math.PI) / 2;
      ownLamps.push([Math.cos(b) * r * 1.05, Math.sin(b) * r * 1.05, z]);
    }
    ownLamps.push([0, 0, (hhi[0] - lo[0]) / len]);
    for (const lp of ownLamps) {
      const g = new SphereGeometry(1, 8, 6);
      const m = new Mesh(g, lampMat);
      m.position.set(...lp);
      m.frustumCulled = false;
      own.add(m);
      ownParts.push({ mesh: m, lit: lampMat, lamp: true });
    }
  };

  let pose: LanderPose | null = null;
  let look = "";
  const paintFor = (l: string) => {
    for (const p of parts) p.mesh.material = l === "true" ? p.lit : l === "wireframe" ? inkWire : inkMat;
    for (const p of ownParts) p.mesh.material = l === "true" ? p.lit : l === "wireframe" ? inkWire : inkMat;
  };
  return {
    group,
    set(p) {
      pose = p;
      const sh = p?.ship ?? null;
      if (sh !== ownOf) {
        dropOwn();
        ownOf = sh;
        if (sh) buildOwn(sh);
        look = "";
      }
      stock.visible = !ownParts.length;
      own.visible = !!ownParts.length;
      group.visible = !!p && p.height_m > 0;
      if (!p) return;
      const s = p.height_m;
      group.scale.set(s, s, s);
      // Heading from north toward east; the shape's front (the hatch) faces +y, north, at heading 0.
      group.rotation.set(0, 0, (-p.heading_deg * Math.PI) / 180);
      const on = p.lights && !!p.lamp;
      uLamp.value.set(on ? p.lamp![0] : 0, on ? p.lamp![1] : 0, on ? p.lamp![2] : 0);
      // A ship's lamps keep their own size in metres, whatever the ship's length; on a hull longer than the stock
      // shape the same lamps light less of it (the spill falls as their size over the hull's, squared).
      const lr = (p.lamp_m ?? LAMP_DISC * s) / s;
      uSpill.value.copy(uLamp.value).multiplyScalar(SPILL * Math.min(1, (lr / LAMP_DISC) ** 2));
      for (const q of parts) if (q.lamp) q.mesh.visible = on;
      for (const q of ownParts) if (q.lamp) (q.mesh.visible = on, q.mesh.scale.setScalar(lr));
    },
    place(eye, l) {
      if (!pose) return;
      group.position.set(pose.at_m[0] - eye[0], pose.at_m[1] - eye[1], pose.ground_m - eye[2]);
      if (l !== look) ((look = l), paintFor(l));
    },
    lamps() {
      if (!pose) return [];
      const p = pose;
      const t = (-p.heading_deg * Math.PI) / 180;
      return (ownParts.length ? ownLamps : lampPlaces).map(([x, y, z]) => [p.at_m[0] + (x * Math.cos(t) - y * Math.sin(t)) * p.height_m, p.at_m[1] + (x * Math.sin(t) + y * Math.cos(t)) * p.height_m, p.ground_m + z * p.height_m]);
    },
    dispose() {
      dropOwn();
      for (const g of geoms) g.dispose();
      for (const m of [hullTrue, legTrue, lampMat, inkMat, inkWire]) m.dispose();
    },
  };
}
