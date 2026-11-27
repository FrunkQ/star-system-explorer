// Compound ships, CM7 (design `handover/design/SSE4-compound-ships-design.md` section 8.2, accepted by Alex 27 Nov 2026
// with his rulings in section 14A): the designer places each construct of a compound by its mount. Every construct is
// laid out on its own exactly as before (designShip, or any layout with the same pieces); this file only places those
// layouts relative to each other and sizes bays from what they hold. It reads the mount facts the kernel writes (CM1,
// crates/sse-trigger/src/mount.rs) and never feeds anything back: the kernel's figures make the drawing, never the
// reverse (design section 12).
//
// Frames: every layout runs along x from its stern (0) to its nose (length_m), y to the side, z up. A construct's
// clock round its carrier's axis is measured from up (+z, clock 0) towards +y (clock 90), so clock 180 is the belly,
// the kernel's reading for an orbiter's tank (design 8.3). A construct on the hull or outside is turned so its own up
// faces away from the carrier: it meets the hull with its belly. `turn_deg` (the builder's turn about the connection,
// kept in the recipe, never a kernel fact) turns it further.
//
// The five places (design 4.2):
//   front: its stern at the carrier's nose, on the axis;   behind: its nose at the carrier's stern, on the axis;
//   hull: at `along_m` from the carrier's nose and `clock_deg` round, its belly touching the carrier's outer surface
//         there (the surface is found along that clock line, so a clustered tank ring is met where it really is);
//   outside: as the hull, `stand_off_m` further out, with the arm or tether drawn;
//   inside: in its bay, centred, several in one bay set round each layer and layer after layer along it.
// Fit notes warn, never refuse (design 8.2 item 5): too big for its bay or its door, strap-ons that would turn the
// stack, a drive at a carrier's nose with something in front, constructs that overlap, a mount whose station is not
// given (drawn at a stated place).
import type { DesignBay, Piece, ShipLayout } from "./ship-design.ts";

export type MountPlace = "front" | "behind" | "inside" | "hull" | "outside";

/** The mount, as the kernel's facts on the carried construct (`mount.*`), plus the builder's turn. */
export interface MountFacts {
  on: string;
  place: MountPlace;
  port?: string;
  bay?: string;
  along_m?: number;
  clock_deg?: number;
  stand_off_m?: number;
  bond?: string;
  lines?: string[];
  /** The builder's turn about the connection (recipe, CM14); not a kernel fact. */
  turn_deg?: number;
}

/** A named port on a construct (`port.<id>.*`). Where it sits on the hull is the kernel's when given. */
export interface PortFacts {
  id: string;
  place: MountPlace;
  size_m?: number;
  lines?: string[];
  along_m?: number;
  clock_deg?: number;
}

/** A bay part (`part.<id>.bay_length_m`, `bay_radius_m`, `bay_door_m`). A size not given is worked out from what it holds. */
export interface BayFacts {
  id: string;
  length_m?: number;
  radius_m?: number;
  door_m?: number;
}

/** base-ships `ship_design.compound`: arrangement for pictures, authored, in the fudge register. */
export interface CompoundRules {
  bay_clearance: number;
  bay_per_layer: number;
  port_ring_share: number;
  tether_share: number;
}

/** One construct of a compound. */
export interface CompoundMember {
  address: string;
  name: string;
  mount?: MountFacts;
  ports?: PortFacts[];
  bays?: BayFacts[];
}

/** Where one construct ended up: its layout's frame turned by `roll_deg` about x, then moved by `offset`. */
export interface PlacedMember {
  address: string;
  name: string;
  carrier?: string;
  place?: MountPlace;
  offset: [number, number, number];
  roll_deg: number;
  length_m: number;
}

/** A port drawn as a ring on its construct's hull, in the compound's frame. */
export interface PortMark {
  member: string;
  id: string;
  place: MountPlace;
  at: [number, number, number];
  /** Which way the ring faces (a unit vector). */
  normal: [number, number, number];
  r_m: number;
  thick_m: number;
  lines: string[];
  /** The construct mounted at it, if any. */
  used_by?: string;
}

export interface CompoundLayout extends ShipLayout {
  root: string;
  members: PlacedMember[];
  ports: PortMark[];
}

// ---- reading the kernel's facts ----

type FactValue = number | string | boolean | readonly unknown[] | null | undefined;
type Facts = Readonly<Record<string, FactValue>>;

const num = (v: FactValue) => (typeof v === "number" && Number.isFinite(v) ? v : undefined);
const text = (v: FactValue) => (typeof v === "string" ? v : undefined);
const list = (v: FactValue): string[] => (Array.isArray(v) ? v.filter((x): x is string => typeof x === "string") : typeof v === "string" && v ? v.split(/[,\s]+/).filter(Boolean) : []);
const PLACES: readonly MountPlace[] = ["front", "behind", "inside", "hull", "outside"];
const placeOf = (v: FactValue) => PLACES.find((p) => p === v);

/** The mount from a construct's facts, or none when it is not carried. An RS4 dock (`docked.to`) reads as a latch on
 * the hull, as the kernel reads it. */
export function mountFromFacts(f: Facts): MountFacts | undefined {
  const on = text(f["mount.on"]);
  const place = placeOf(f["mount.place"]);
  if (on && place) {
    const m: MountFacts = { on, place, lines: list(f["mount.lines"]) };
    const port = text(f["mount.port"]);
    const bay = text(f["mount.bay"]);
    const bond = text(f["mount.bond"]);
    const along = num(f["mount.station.along_m"]);
    const clock = num(f["mount.station.clock_deg"]);
    const off = num(f["mount.station.stand_off_m"]);
    if (port) m.port = port;
    if (bay) m.bay = bay;
    if (bond) m.bond = bond;
    if (along !== undefined) m.along_m = along;
    if (clock !== undefined) m.clock_deg = clock;
    if (off !== undefined) m.stand_off_m = off;
    return m;
  }
  const docked = text(f["docked.to"]);
  return docked ? { on: docked, place: "hull", bond: "latch", lines: [] } : undefined;
}

/** A construct's ports from its `port.<id>.*` facts, in id order. */
export function portsFromFacts(f: Facts): PortFacts[] {
  const ids = [...new Set(Object.keys(f).flatMap((k) => (k.startsWith("port.") && k.endsWith(".place") ? [k.slice(5, -6)] : [])))].sort();
  return ids.flatMap((id) => {
    const place = placeOf(f[`port.${id}.place`]);
    if (!place) return [];
    const p: PortFacts = { id, place, lines: list(f[`port.${id}.lines`]) };
    const size = num(f[`port.${id}.size_m`]);
    const along = num(f[`port.${id}.station.along_m`]);
    const clock = num(f[`port.${id}.station.clock_deg`]);
    if (size !== undefined) p.size_m = size;
    if (along !== undefined) p.along_m = along;
    if (clock !== undefined) p.clock_deg = clock;
    return [p];
  });
}

/** A construct's bays from its `part.<id>.bay_*` facts, in id order. */
export function baysFromFacts(f: Facts): BayFacts[] {
  const ids = [...new Set(Object.keys(f).flatMap((k) => { const m = /^part\.(.+)\.bay_(length_m|radius_m|door_m|holds_kg)$/.exec(k); return m ? [m[1]!] : []; }))].sort();
  return ids.map((id) => {
    const b: BayFacts = { id };
    const l = num(f[`part.${id}.bay_length_m`]);
    const r = num(f[`part.${id}.bay_radius_m`]);
    const d = num(f[`part.${id}.bay_door_m`]);
    if (l !== undefined) b.length_m = l;
    if (r !== undefined) b.radius_m = r;
    if (d !== undefined) b.door_m = d;
    return b;
  });
}

// ---- geometry ----

const RAD = Math.PI / 180;
/** Up (+z) turned to clock `deg`: (y, z) = (sin, cos). */
const clockDir = (deg: number): [number, number] => [Math.sin(deg * RAD), Math.cos(deg * RAD)];
/** Turn a (y, z) point so that up goes to clock `deg`. */
const turn = (y: number, z: number, deg: number): [number, number] => {
  if (!deg) return [y, z];
  const c = Math.cos(deg * RAD);
  const s = Math.sin(deg * RAD);
  return [y * c + z * s, -y * s + z * c];
};
const solid = (p: Piece) => p.shape === "body" && p.system !== "plume" && p.system !== "tether";
const radiusAt = (p: Piece, x: number) => (p.x1 > p.x0 ? p.r0 + ((p.r1 - p.r0) * (x - p.x0)) / (p.x1 - p.x0) : Math.max(p.r0, p.r1));
/** A panel's tip at its root's two ends, in (y, z). */
const panelTip = (p: Piece): [number, number] => {
  const a = (p.roll_deg ?? 0) * RAD;
  const s = p.span ?? 0;
  return [p.y + s * Math.cos(a), p.z - s * Math.sin(a)];
};

/** How far a layout reaches from its axis, wings and radiators included. */
export function reachOf(l: ShipLayout): number {
  let r = 0;
  for (const p of l.pieces) {
    if (p.system === "plume") continue;
    if (p.shape === "body") r = Math.max(r, Math.hypot(p.y, p.z) + Math.max(p.r0, p.r1));
    else r = Math.max(r, Math.hypot(p.y, p.z), Math.hypot(...panelTip(p)));
  }
  return r;
}

/** How far below its own axis a layout reaches (its belly, -z), once turned by `deg`. */
function bellyOf(l: ShipLayout, deg: number): number {
  let b = 0;
  for (const p of l.pieces) {
    if (p.system === "plume") continue;
    const [, z] = turn(p.y, p.z, deg);
    if (p.shape === "body") b = Math.max(b, p.r0 - z, p.r1 - z);
    else {
      const [ty, tz] = panelTip(p);
      b = Math.max(b, -z, -turn(ty, tz, deg)[1]);
    }
  }
  return b;
}

/** Where the carrier's outer surface is at `x` along the clock line `deg`, from its axis. Exact for its bodies of
 * revolution: the farthest crossing of the ray from the axis with any body covering x. */
function hullReach(l: ShipLayout, x: number, deg: number): number | undefined {
  const [uy, uz] = clockDir(deg);
  let best: number | undefined;
  for (const p of l.pieces) {
    if (!solid(p) || p.system === "legs" || x < p.x0 || x > p.x1) continue;
    const r = radiusAt(p, x);
    const along = p.y * uy + p.z * uz;
    const perp = Math.abs(p.y * uz - p.z * uy);
    if (perp >= r) continue;
    const out = along + Math.sqrt(r * r - perp * perp);
    if (out > 0 && (best === undefined || out > best)) best = out;
  }
  return best;
}

interface Frame {
  o: [number, number, number];
  roll: number;
}
/** Carrier frame after child frame: R(a + b) p + R(a) o_b + o_a. */
const compose = (a: Frame, b: Frame): Frame => {
  const [y, z] = turn(b.o[1], b.o[2], a.roll);
  return { o: [a.o[0] + b.o[0], a.o[1] + y, a.o[2] + z], roll: a.roll + b.roll };
};
const movePiece = (p: Piece, f: Frame, member: string): Piece => {
  const [y, z] = turn(p.y, p.z, f.roll);
  const q: Piece = { ...p, x0: p.x0 + f.o[0], x1: p.x1 + f.o[0], y: y + f.o[1], z: z + f.o[2], member };
  if (p.shape === "panel") {
    const roll = (p.roll_deg ?? 0) + f.roll;
    if (roll) q.roll_deg = roll;
    else delete q.roll_deg;
  }
  return q;
};

const words = (m: number) => (m >= 10 ? `${Math.round(m)} m` : `${Math.round(m * 10) / 10} m`);

/** Points round a port's ring (its centre line), in the compound's frame: drawn as an ellipse in the plans and as a
 * tube in 3D, both from these. */
export function ringPoints(p: PortMark, n = 24): [number, number, number][] {
  const [nx, ny, nz] = p.normal;
  // Two directions across the ring's face.
  const a: [number, number, number] = Math.abs(nx) < 0.9 ? [0, -nz, ny] : [-ny, nx, 0];
  const la = Math.hypot(...a) || 1;
  const e1 = a.map((v) => v / la) as [number, number, number];
  const e2: [number, number, number] = [ny * e1[2] - nz * e1[1], nz * e1[0] - nx * e1[2], nx * e1[1] - ny * e1[0]];
  return Array.from({ length: n }, (_, i) => {
    const t = (2 * Math.PI * i) / n;
    const c = Math.cos(t) * p.r_m;
    const s = Math.sin(t) * p.r_m;
    return [p.at[0] + c * e1[0] + s * e2[0], p.at[1] + c * e1[1] + s * e2[1], p.at[2] + c * e1[2] + s * e2[2]];
  });
}

// ---- bays ----

/** How several carried constructs sit in one bay: up to `per` round each layer, layers along the bay. */
function arrangement(n: number, rules: CompoundRules): { per: number; layers: number } {
  const per = Math.max(1, Math.min(n, Math.floor(rules.bay_per_layer)));
  return { per, layers: Math.ceil(n / per) };
}
const ringRadius = (per: number, r: number) => (per <= 1 ? 0 : r / Math.sin(Math.PI / per));

/** Which of a carrier's bays a carried construct goes in: the one its mount names, else the only one, else the first. */
function bayFor(m: MountFacts, bays: readonly BayFacts[]): string {
  if (m.bay && bays.some((b) => b.id === m.bay)) return m.bay;
  return bays[0]?.id ?? m.bay ?? "bay";
}

/** Size a carrier's bays from what each holds (design 8.2 item 4: capabilities drive form). A size the kernel gives
 * (`bay_length_m`, `bay_radius_m`) wins; one it leaves out is the largest held construct's, set round and along by
 * `arrangement`, times the clearance. A carrier with things inside and no bay part gets one, and a note says so. */
export function sizeBays(carrier: CompoundMember, held: readonly { member: CompoundMember; layout: ShipLayout }[], rules: CompoundRules, notes: string[] = []): DesignBay[] {
  const bays = carrier.bays ?? [];
  const groups = new Map<string, { member: CompoundMember; layout: ShipLayout }[]>();
  for (const h of held) {
    if (h.member.mount?.place !== "inside") continue;
    const id = bayFor(h.member.mount, bays);
    groups.set(id, [...(groups.get(id) ?? []), h]);
  }
  if (!bays.length && groups.size) notes.push(`${carrier.name} carries ${held.map((h) => h.member.name).join(", ")} inside but has no bay part; a bay is drawn, sized from what it holds.`);
  const ids = [...new Set([...bays.map((b) => b.id), ...groups.keys()])];
  return ids.map((id) => {
    const given = bays.find((b) => b.id === id);
    const inside = groups.get(id) ?? [];
    const { per, layers } = arrangement(inside.length, rules);
    const len = Math.max(0, ...inside.map((h) => h.layout.length_m));
    const r = Math.max(0, ...inside.map((h) => reachOf(h.layout)));
    const length_m = given?.length_m ?? layers * len * rules.bay_clearance;
    const radius_m = given?.radius_m ?? (ringRadius(per, r) + r) * rules.bay_clearance;
    const label = inside.length === 0 ? "bay, empty" : inside.length === 1 ? `bay, holds ${inside[0]!.member.name}` : `bay, holds ${inside.length}`;
    return { id, length_m: Math.max(length_m, 1e-3), radius_m: Math.max(radius_m, 1e-3), label };
  });
}

// ---- the compound ----

/** Lay out a compound: each construct designed by `design` (carried ones first, so a carrier's bays are sized from
 * what they hold), then placed by its mount. Members whose carrier is not in the list are drawn beside the compound,
 * with a note. */
export function designCompound(members: readonly CompoundMember[], design: (m: CompoundMember, bays: DesignBay[]) => ShipLayout, rules: CompoundRules): CompoundLayout {
  const notes: string[] = [];
  const kids = childrenOf(members);
  const layouts = new Map<string, ShipLayout>();
  const visit = (m: CompoundMember, path: readonly string[]) => {
    if (layouts.has(m.address)) return;
    if (path.includes(m.address)) return;
    const children = kids.get(m.address) ?? [];
    for (const c of children) visit(c, [...path, m.address]);
    const held = children.filter((c) => c.mount?.place === "inside").map((c) => ({ member: c, layout: layouts.get(c.address)! }));
    layouts.set(m.address, design(m, sizeBays(m, held, rules, notes)));
  };
  for (const m of members) visit(m, []);
  return placeCompound(members, layouts, rules, notes);
}

function childrenOf(members: readonly CompoundMember[]): Map<string, CompoundMember[]> {
  const kids = new Map<string, CompoundMember[]>();
  const known = new Set(members.map((m) => m.address));
  for (const m of members) if (m.mount && known.has(m.mount.on)) kids.set(m.mount.on, [...(kids.get(m.mount.on) ?? []), m]);
  for (const v of kids.values()) v.sort((a, b) => (a.address < b.address ? -1 : a.address > b.address ? 1 : 0));
  return kids;
}

/** Place laid-out constructs by their mounts. The first member with no carrier in the list is the root, at the
 * origin; the result is moved so the compound's stern is at x = 0, as for one ship. */
export function placeCompound(members: readonly CompoundMember[], layouts: ReadonlyMap<string, ShipLayout>, rules: CompoundRules, notes: string[] = []): CompoundLayout {
  const known = new Set(members.map((m) => m.address));
  const kids = childrenOf(members);
  const roots = members.filter((m) => !m.mount || !known.has(m.mount.on));
  for (const m of roots) if (m.mount) notes.push(`${m.name} is mounted on something not in this picture; it is drawn beside the rest.`);
  if (!roots.length && members.length) notes.push("Every construct here is carried by another, round in a loop; the first is drawn as the root.");
  const order = roots.length ? roots : members.slice(0, 1);
  const pieces: Piece[] = [];
  const placed: PlacedMember[] = [];
  const ports: PortMark[] = [];
  const frames = new Map<string, Frame>();
  let besideY = 0;

  const portMarks = (m: CompoundMember, l: ShipLayout, f: Frame) => {
    const list = m.ports ?? [];
    const loose = list.filter((p) => (p.place === "hull" || p.place === "outside") && p.clock_deg === undefined);
    const big = reachOf(l);
    for (const p of list) {
      const size = p.size_m ?? 0.25 * big;
      let at: [number, number, number];
      let normal: [number, number, number];
      if (p.place === "front") (at = [l.length_m, 0, 0]), (normal = [1, 0, 0]);
      else if (p.place === "behind") (at = [0, 0, 0]), (normal = [-1, 0, 0]);
      else if (p.place === "inside") {
        const bay = l.pieces.find((q) => q.system === "bay");
        (at = [bay ? bay.x1 : l.length_m / 2, bay?.y ?? 0, bay?.z ?? 0]), (normal = [1, 0, 0]);
      } else {
        const k = loose.indexOf(p);
        const clock = p.clock_deg ?? (360 * k) / Math.max(1, loose.length);
        const x = l.length_m - (p.along_m ?? l.length_m / 2);
        const reach = hullReach(l, x, clock) ?? big;
        const [uy, uz] = clockDir(clock);
        (at = [x, reach * uy, reach * uz]), (normal = [0, uy, uz]);
      }
      const [ay, az] = turn(at[1], at[2], f.roll);
      const [ny, nz] = turn(normal[1], normal[2], f.roll);
      const user = members.find((c) => c.mount?.on === m.address && c.mount.port === p.id);
      ports.push({
        member: m.address,
        id: p.id,
        place: p.place,
        at: [at[0] + f.o[0], ay + f.o[1], az + f.o[2]],
        normal: [normal[0], ny, nz],
        r_m: size / 2,
        thick_m: rules.port_ring_share * size,
        lines: p.lines ?? [],
        ...(user ? { used_by: user.address } : {}),
      });
    }
    if (loose.length && list.length) notes.push(`Where ${m.name}'s ${loose.length === 1 ? "hull port sits is" : "hull ports sit is"} not given, so ${loose.length === 1 ? "it is" : "they are"} drawn spread round its middle.`);
  };

  const place = (m: CompoundMember, f: Frame, path: readonly string[]) => {
    const l = layouts.get(m.address);
    if (!l || frames.has(m.address) || path.includes(m.address)) return;
    frames.set(m.address, f);
    for (const p of l.pieces) pieces.push(movePiece(p, f, m.address));
    placed.push({ address: m.address, name: m.name, ...(m.mount && known.has(m.mount.on) ? { carrier: m.mount.on, place: m.mount.place } : {}), offset: f.o, roll_deg: f.roll, length_m: l.length_m });
    portMarks(m, l, f);
    const children = kids.get(m.address) ?? [];
    // Hull and outside constructs with no clock are spread round evenly, in address order.
    const unclocked = children.filter((c) => (c.mount!.place === "hull" || c.mount!.place === "outside") && c.mount!.clock_deg === undefined && !(m.ports ?? []).some((p) => p.id === c.mount!.port && p.clock_deg !== undefined));
    const inBays = new Map<string, CompoundMember[]>();
    for (const c of children) if (c.mount!.place === "inside") {
      const id = bayFor(c.mount!, m.bays ?? []);
      inBays.set(id, [...(inBays.get(id) ?? []), c]);
    }
    const axial = { front: 0, behind: 0 };
    for (const c of children) {
      const cl = layouts.get(c.address);
      if (!cl) continue;
      const mt = c.mount!;
      const port = (m.ports ?? []).find((p) => p.id === mt.port);
      const t = mt.turn_deg ?? 0;
      let local: Frame;
      if (mt.place === "front" || mt.place === "behind") {
        axial[mt.place]++;
        if (axial[mt.place] > 1) notes.push(`${m.name} has more than one construct ${mt.place === "front" ? "in front" : "behind"}; ${c.name} is drawn where the first one is.`);
        local = { o: [mt.place === "front" ? l.length_m : -cl.length_m, 0, 0], roll: t };
        const drive = l.pieces.find((p) => p.system === "drive");
        if (mt.place === "front" && drive && drive.x1 > 0.9 * l.length_m) notes.push(`${c.name} is in front of ${m.name}, whose drive is at its nose.`);
      } else if (mt.place === "inside") {
        const id = bayFor(mt, m.bays ?? []);
        const bay = l.pieces.find((p) => p.system === "bay" && p.part === id) ?? l.pieces.find((p) => p.system === "bay");
        const group = inBays.get(id) ?? [c];
        const k = group.indexOf(c);
        const { per, layers } = arrangement(group.length, rules);
        const r = Math.max(...group.map((g) => reachOf(layouts.get(g.address) ?? cl)));
        const rho = ringRadius(per, r);
        const a = (360 * (k % per)) / per;
        const [uy, uz] = clockDir(a);
        if (!bay) {
          notes.push(`${m.name} has no bay drawn for ${c.name}; it is drawn at ${m.name}'s middle.`);
          local = { o: [(l.length_m - cl.length_m) / 2, 0, 0], roll: t };
        } else {
          const slot = (bay.x1 - bay.x0) / layers;
          const xc = bay.x1 - (Math.floor(k / per) + 0.5) * slot;
          local = { o: [xc - cl.length_m / 2, bay.y + rho * uy, bay.z + rho * uz], roll: t };
          const given = (m.bays ?? []).find((b) => b.id === id);
          if (cl.length_m > slot * 1.0001) notes.push(`${c.name} is ${words(cl.length_m)} long, longer than its room in ${m.name}'s bay (${words(slot)}).`);
          if (rho + reachOf(cl) > Math.max(bay.r0, bay.r1) * 1.0001) notes.push(`${c.name} is wider than ${m.name}'s bay.`);
          if (given?.door_m !== undefined && 2 * reachOf(cl) > given.door_m) notes.push(`${m.name}'s bay door is ${words(given.door_m)} across, too small for ${c.name} (${words(2 * reachOf(cl))}) to leave.`);
        }
      } else {
        const k = unclocked.indexOf(c);
        const clock = mt.clock_deg ?? port?.clock_deg ?? (360 * k) / Math.max(1, unclocked.length);
        const along = mt.along_m ?? port?.along_m;
        if (along === undefined) notes.push(`Where ${c.name} sits along ${m.name} is not given; it is drawn at the middle.`);
        const xc = l.length_m - (along ?? l.length_m / 2);
        const reach = hullReach(l, xc, clock);
        if (reach === undefined) notes.push(`${c.name} is ${words(along ?? l.length_m / 2)} from ${m.name}'s nose, past its hull there; it is drawn off its widest part.`);
        const r0 = reach ?? reachOf(l);
        const off = mt.place === "outside" ? mt.stand_off_m ?? 0 : 0;
        if (mt.place === "outside" && mt.stand_off_m === undefined) notes.push(`${c.name} is held outside ${m.name} with no stand-off given; it is drawn touching.`);
        const roll = clock + t;
        const d = r0 + off + bellyOf(cl, t);
        const [uy, uz] = clockDir(clock);
        local = { o: [xc - cl.length_m / 2, d * uy, d * uz], roll };
        if (off > 0) {
          // The arm or tether: a thin strip from the hull out to the carried construct, along the clock line.
          const w = Math.max(rules.tether_share * reachOf(cl), 1e-3);
          pieces.push(movePiece({ system: "tether", shape: "panel", x0: xc - w, x1: xc + w, r0: 0, r1: 0, y: r0 * uy, z: r0 * uz, span: off, roll_deg: clock - 90, label: `tether to ${c.name}` }, f, m.address));
        }
      }
      place(c, compose(f, local), [...path, m.address]);
    }
    // Thrust on the hull (design 6.4): fixed strap-ons with drives must balance round the axis.
    // Only a fixed bond takes thrust; a latch (the hull's usual bond) does not, so a docked craft never counts.
    const pushers = children.filter((c) => (c.mount!.place === "hull" || c.mount!.place === "outside") && c.mount!.bond === "fixed" && layouts.get(c.address)?.pieces.some((p) => p.system === "drive"));
    if (pushers.length) {
      let sy = 0;
      let sz = 0;
      let total = 0;
      for (const c of pushers) {
        const thrust = layouts.get(c.address)!.needs.drive.thrust_n || 1;
        const k = unclocked.indexOf(c);
        const port = (m.ports ?? []).find((p) => p.id === c.mount!.port);
        const [uy, uz] = clockDir(c.mount!.clock_deg ?? port?.clock_deg ?? (360 * k) / Math.max(1, unclocked.length));
        (sy += thrust * uy), (sz += thrust * uz), (total += thrust);
      }
      if (Math.hypot(sy, sz) > 0.05 * total) notes.push(`The engines on ${m.name}'s hull are not balanced round its axis: firing together they would turn the stack.`);
    }
  };

  for (const r of order) {
    const l = layouts.get(r.address);
    place(r, { o: [0, besideY, 0], roll: 0 }, []);
    if (l) besideY += 2.5 * reachOf(l);
  }
  for (const m of members) if (!layouts.has(m.address)) notes.push(`${m.name} has no layout, so it is not drawn.`);

  // Overlaps between constructs that are not carrier and carried (a carried one inside its bay is meant to be there).
  const box = new Map<string, [number[], number[]]>();
  for (const p of pieces) {
    if (!solid(p) || p.system === "bay" || p.system === "spine" || !p.member) continue;
    const r = Math.max(p.r0, p.r1);
    const b = box.get(p.member) ?? [[Infinity, Infinity, Infinity], [-Infinity, -Infinity, -Infinity]];
    const lo = [p.x0, p.y - r, p.z - r];
    const hi = [p.x1, p.y + r, p.z + r];
    for (let d = 0; d < 3; d++) (b[0][d] = Math.min(b[0][d]!, lo[d]!)), (b[1][d] = Math.max(b[1][d]!, hi[d]!));
    box.set(p.member, b);
  }
  const related = (a: string, b: string) => members.some((m) => (m.address === a && m.mount?.on === b) || (m.address === b && m.mount?.on === a));
  const names = new Map(members.map((m) => [m.address, m.name]));
  const keys = [...box.keys()];
  for (let i = 0; i < keys.length; i++)
    for (let j = i + 1; j < keys.length; j++) {
      const [a, b] = [keys[i]!, keys[j]!];
      if (related(a, b)) continue;
      const [alo, ahi] = box.get(a)!;
      const [blo, bhi] = box.get(b)!;
      // A small overlap is the box's corners round a cylinder; a fifth of the smaller one's size is a real clash.
      const over = [0, 1, 2].map((d) => Math.min(ahi[d]!, bhi[d]!) - Math.max(alo[d]!, blo[d]!));
      const size = [0, 1, 2].map((d) => Math.min(ahi[d]! - alo[d]!, bhi[d]! - blo[d]!));
      if (over.every((o, d) => o > 0.2 * size[d]!)) notes.push(`${names.get(a)} and ${names.get(b)} overlap.`);
    }

  // Stern at 0, as for one ship.
  const xs = pieces.filter(solid).map((p) => p.x0);
  const shift = xs.length ? -Math.min(...xs) : 0;
  for (const p of pieces) (p.x0 += shift), (p.x1 += shift);
  for (const m of placed) m.offset = [m.offset[0] + shift, m.offset[1], m.offset[2]];
  for (const p of ports) p.at = [p.at[0] + shift, p.at[1], p.at[2]];
  const length = Math.max(0, ...pieces.filter(solid).map((p) => p.x1));

  // The plume belongs to the stern-most drive: its construct's pieces go first and its needs stand for the compound's.
  const drives = pieces.filter((p) => p.system === "drive");
  const sternDrive = drives.length ? drives.reduce((a, p) => (p.x0 < a.x0 ? p : a)) : undefined;
  const stern = sternDrive?.member ?? order[0]?.address ?? "";
  pieces.sort((a, b) => Number(b.member === stern) - Number(a.member === stern));
  const rootAddr = order[0]?.address ?? "";
  const rootLayout = layouts.get(rootAddr) ?? layouts.get(stern)!;
  const sternLayout = layouts.get(stern) ?? rootLayout;
  let across = 0;
  for (const p of pieces) {
    if (p.system === "plume") continue;
    if (p.shape === "body") across = Math.max(across, 2 * Math.max(Math.abs(p.y), Math.abs(p.z)) + 2 * Math.max(p.r0, p.r1));
    else {
      const [ty, tz] = panelTip(p);
      across = Math.max(across, 2 * Math.max(Math.abs(p.y), Math.abs(p.z), Math.abs(ty), Math.abs(tz)));
    }
  }
  return {
    name: rootLayout?.name ?? "",
    style: rootLayout?.style ?? "",
    seed: rootLayout?.seed ?? 0,
    pieces,
    length_m: length,
    across_m: across,
    needs: sternLayout!.needs,
    notes: [...notes, ...[...layouts.values()].flatMap((l) => l.notes)],
    root: rootAddr,
    members: placed,
    ports,
  };
}
