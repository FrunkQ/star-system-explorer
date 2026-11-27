// The ship builder's model (CM14a, compound ships design 8A.4, 8B.1 and rulings 14A.2 to 14A.4, with Alex's two modes
// of 27 Nov 11:47): a ship is palette parts clipped at their connection points, kept as a recipe. This file lays a
// recipe out as the designer's pieces (so the plans and the 3D model draw it exactly as they draw any ship), sums what
// the parts carry into a ship record the kernel judges, sets a plan's budgets beside what is built, and turns a ship
// the old generator laid out into parts (so the catalogue opens in the builder until every ship is saved as parts).
//
// Nothing here is physics: placing is geometry, totals are sums of the parts' own figures, propellant mass is a tank's
// room times the sheet's density, and every judgement (what it can do at a world, its power books) is the kernel's.

import { stretchPart, type Part, type PartKind, type PartsPalette } from "../../ui-contract/src/index.ts";
import { constant } from "../../ui-contract/src/index.ts";
import { driveFacts, type Propellants, type ShipRecord } from "./ship.ts";
import { needsOf, type DesignRules, type DesignShip, type Piece, type ShipLayout, type SystemKind } from "./ship-design.ts";

/** Where a part sits on the one it clips to: in front of it, behind it, on its hull, or inside its bay. */
export type BuildPlace = "front" | "behind" | "hull" | "bay";

/** One part of a build: a palette part, resized, clipped to another (none for the root), turned and painted. */
export interface BuiltPart {
  key: string;
  part: string;
  length_m?: number;
  radius_m?: number;
  on?: string;
  place?: BuildPlace;
  /** On the hull: the share along the carrier from its stern (0) to its nose (1), and the clock (0 up, 90 to +y). */
  along?: number;
  clock_deg?: number;
  /** The builder's turn about the joint (a recipe value, never a kernel fact). */
  turn_deg?: number;
  /** What a tank holds (a base-nav fuel id); missing is the ship's first fuel. */
  fuel?: string;
  /** A finish or a colour (#rrggbb). */
  paint?: string;
  /** A drive whose figures its part does not give yet (a tech advance's): the push and exhaust speed stated by the
   * record it came from, with that record named. */
  drive?: { thrust_n: number; exhaust_km_s: number; src: string };
}

/** A build: everything needed to rebuild the same ship anywhere. Saved with the construct in place of the old style recipe. */
export interface BuildRecipe {
  format: "sse.build/1";
  name: string;
  parts: BuiltPart[];
  /** The ship's fuels, first is the one tanks hold unless a tank says otherwise. */
  fuels?: string[];
  crew?: number;
  cargo_t?: number;
  /** The catalogue ship it was opened from. */
  from?: string;
  /** Structure no part counts yet (tank walls, frame, wiring): carried over from the catalogue record a build was
   * opened from, until the parts' own masses are read, and said so. */
  structure_t?: number;
  /** Build to a plan: the record whose needs set the budgets (Alex: "fix the basic capabilities of a ship"). */
  plan?: DesignShip;
}

/** A part placed: its resized palette part, where its stern sits on the axis, and its centre line off the axis. */
export interface PlacedPart {
  key: string;
  part: Part;
  x0: number;
  y: number;
  z: number;
  /** Panels and dishes: which way they reach (degrees, clock), from where they meet the carrier. */
  roll_deg: number;
  built: BuiltPart;
}

/** A connection point not yet used, where the builder offers a clip. */
export interface FreePoint {
  key: string;
  place: BuildPlace;
  /** Where it is, in the layout frame (after the stern is moved to 0). */
  at: [number, number, number];
  size_m: number;
  /** Hull points: the share along and clock a part dropped there takes. */
  along?: number;
  clock_deg?: number;
  label: string;
}

const SYSTEM_OF: Record<PartKind, SystemKind> = {
  stage: "tanks",
  engine: "drive",
  tank: "tanks",
  habitat: "crew",
  node: "spine",
  truss: "spine",
  power: "sensors",
  solar: "wings",
  radiator: "radiators",
  bay: "bay",
  pod: "cargo",
  sensor: "sensors",
  dish: "sensors",
  shield: "shield",
  legs: "legs",
};

const FLAT: readonly string[] = ["panel", "dish"];

/** A part as built: the palette's, stretched and widened to the recipe's size by its own law. */
export function partAsBuilt(b: BuiltPart, palette: PartsPalette): { part: Part; note?: string } | null {
  const p = palette.parts.find((x) => x.id === b.part);
  if (!p) return null;
  if (b.length_m === undefined && b.radius_m === undefined) return { part: p };
  return stretchPart(p, { ...(b.length_m !== undefined ? { length_m: b.length_m } : {}), ...(b.radius_m !== undefined ? { radius_m: b.radius_m } : {}) });
}

const rad = (d: number) => (d * Math.PI) / 180;

/** Lay a recipe out: the root on the axis, each part where its place puts it on its carrier, the stern moved to 0. Parts
 * whose carrier is missing, or that come before it, are left out and named. */
export function placeBuild(r: BuildRecipe, palette: PartsPalette): { placed: PlacedPart[]; notes: string[] } {
  const notes: string[] = [];
  const placed: PlacedPart[] = [];
  const byKey = new Map<string, PlacedPart>();
  for (const b of r.parts) {
    const got = partAsBuilt(b, palette);
    if (!got) {
      notes.push(`${b.key}: the part ${b.part} is not on the palette (a pack that is not loaded?), so it is left out.`);
      continue;
    }
    if (got.note) notes.push(got.note);
    const p = got.part;
    const L = p.shape.length_m;
    const R = p.shape.radius_m;
    const flat = FLAT.includes(p.shape.hint);
    const carrier = b.on === undefined ? undefined : byKey.get(b.on);
    let at: PlacedPart;
    if (b.on === undefined) at = { key: b.key, part: p, x0: 0, y: 0, z: 0, roll_deg: b.turn_deg ?? 0, built: b };
    else if (!carrier) {
      notes.push(`${b.key} is clipped to ${b.on}, which is not in the build before it, so it is left out.`);
      continue;
    } else {
      const c = carrier.part.shape;
      const place = b.place ?? "front";
      if (place === "front") at = { key: b.key, part: p, x0: carrier.x0 + c.length_m, y: carrier.y, z: carrier.z, roll_deg: 0, built: b };
      else if (place === "behind") at = { key: b.key, part: p, x0: carrier.x0 - L, y: carrier.y, z: carrier.z, roll_deg: 0, built: b };
      else if (place === "bay") {
        at = { key: b.key, part: p, x0: carrier.x0 + (c.length_m - L) / 2, y: carrier.y, z: carrier.z, roll_deg: 0, built: b };
        if (L > c.length_m || R > c.radius_m) notes.push(`${b.key} is bigger than the bay it sits in (${p.name} in ${carrier.part.name}): the bay wants stretching.`);
      } else {
        const clock = (b.clock_deg ?? 0) + (b.turn_deg ?? 0) * 0;
        const along = Math.min(1, Math.max(0, b.along ?? 0.5));
        const surface = c.radius_m + (flat ? 0 : R);
        const xc = carrier.x0 + along * c.length_m;
        // Clock 0 is up (+z) and 90 is towards +y.
        const y = carrier.y + surface * Math.sin(rad(clock));
        const z = carrier.z + surface * Math.cos(rad(clock));
        at = { key: b.key, part: p, x0: flat ? xc - L / 2 : xc - L / 2, y, z, roll_deg: clock + (b.turn_deg ?? 0), built: b };
      }
    }
    placed.push(at);
    byKey.set(b.key, at);
  }
  const min = Math.min(0, ...placed.map((q) => q.x0));
  for (const q of placed) q.x0 -= min;
  return { placed, notes };
}

/** The placed parts as the designer's pieces, so `shipViews`, `shipMesh` and the call-outs draw a build as any ship.
 * A lathe's profile becomes one body per step; a panel reaches out from its root at its clock; a dish is a short body. */
export function buildPieces(placed: readonly PlacedPart[]): Piece[] {
  const out: Piece[] = [];
  for (const q of placed) {
    const s = q.part.shape;
    const system = SYSTEM_OF[q.part.kind];
    const label = q.part.name;
    if (s.hint === "panel") {
      // A panel's span across, from the surface outward (the palette gives it, or twice its radius as a stand-in).
      const span = s.span_m ?? Math.max(4 * s.radius_m, s.length_m);
      // Its plane turned so it reaches straight out along its clock: Piece.roll_deg puts the span along
      // (cos roll, -sin roll) in (y, z), and outward at clock c is (sin c, cos c), so roll = c - 90.
      out.push({ system, shape: "panel", x0: q.x0, x1: q.x0 + s.length_m, r0: s.radius_m, r1: s.radius_m, y: q.y, z: q.z, span, sweep: 0, roll_deg: q.roll_deg - 90, label, part: q.key, member: q.key });
      continue;
    }
    const prof = s.hint === "lathe" && s.profile && s.profile.length > 1 ? s.profile : [[0, 1], [1, 1]] as [number, number][];
    for (let i = 1; i < prof.length; i++) {
      const [a, ra] = prof[i - 1]!;
      const [b, rb] = prof[i]!;
      if (b <= a) continue;
      out.push({ system, shape: "body", x0: q.x0 + a * s.length_m, x1: q.x0 + b * s.length_m, r0: ra * s.radius_m, r1: rb * s.radius_m, y: q.y, z: q.z, label, part: q.key, member: q.key });
    }
  }
  return out;
}

/** Connection points not yet taken: the free front and back of each part on the axis line it sits on, and its hull for
 * radial parts (offered at four clocks, plus where its own hull points are). Bay mouths with nothing inside. */
export function freePoints(r: BuildRecipe, placed: readonly PlacedPart[]): FreePoint[] {
  const out: FreePoint[] = [];
  for (const q of placed) {
    const s = q.part.shape;
    const used = (place: BuildPlace) => r.parts.some((b) => b.on === q.key && (b.place ?? "front") === place);
    for (const c of q.part.points) {
      if (c.place === "front" && !used("front") && !(q.built.on && q.built.place === "behind")) out.push({ key: q.key, place: "front", at: [q.x0 + s.length_m, q.y, q.z], size_m: c.size_m, label: `In front of ${q.part.name}` });
      if (c.place === "behind" && !used("behind") && !(q.built.on && q.built.place === "front")) out.push({ key: q.key, place: "behind", at: [q.x0, q.y, q.z], size_m: c.size_m, label: `Behind ${q.part.name}` });
      if (c.place === "bay" && !used("bay")) out.push({ key: q.key, place: "bay", at: [q.x0 + s.length_m / 2, q.y, q.z], size_m: c.size_m, label: `Inside ${q.part.name}` });
    }
    // An engine's only point is its top; an engine at the root has its top free.
    if (q.part.kind === "engine" && !q.built.on && !used("front")) out.push({ key: q.key, place: "front", at: [q.x0 + s.length_m, q.y, q.z], size_m: q.part.points[0]?.size_m ?? 2 * s.radius_m, label: `In front of ${q.part.name}` });
    if (s.hint === "lathe" || s.hint === "truss" || s.hint === "box")
      for (const clock of [0, 90, 180, 270]) {
        const taken = r.parts.some((b) => b.on === q.key && b.place === "hull" && Math.abs(((b.clock_deg ?? 0) - clock + 540) % 360 - 180) < 30 && Math.abs((b.along ?? 0.5) - 0.5) < 0.2);
        if (taken) continue;
        out.push({ key: q.key, place: "hull", at: [q.x0 + s.length_m / 2, q.y + s.radius_m * Math.sin(rad(clock)), q.z + s.radius_m * Math.cos(rad(clock))], size_m: 0.8, along: 0.5, clock_deg: clock, label: `On ${q.part.name}'s hull, ${clockWords(clock)}` });
      }
  }
  // Points clip only where nothing sits already: a front on the axis taken by the stern of another part is not free.
  return out;
}

function clockWords(c: number): string {
  const n = ((c % 360) + 360) % 360;
  return n < 45 || n >= 315 ? "on top" : n < 135 ? "to the right" : n < 225 ? "underneath" : "to the left";
}

/** What the parts carry, summed: a ledger, never a judgement. Figures a part does not give are named, not guessed. */
export interface BuildTotals {
  /** Tonnes of everything but propellant: each part's dry or own mass. */
  dry_t: number;
  /** Tonnes of propellant: stages' own loads, and tanks' room times the sheet's density for what they hold. */
  propellant_t: number;
  /** Room in tanks by what they hold, m3. */
  tank_m3: Record<string, number>;
  habitat_m3: number;
  cargo_m3: number;
  radiator_m2: number;
  solar_m2: number;
  electric_w: number;
  thrust_n: number;
  /** Thrust-weighted exhaust speed of the engines, km/s. */
  exhaust_km_s: number;
  /** Drives whose figures are a tech advance's, read at the setting's level by the kernel. */
  advances: string[];
  length_m: number;
  across_m: number;
  /** What the sum could not count, in words. */
  missing: string[];
}

const kg = (p: Part): number | undefined => {
  const f = p.figures;
  if (f.dry_t) return f.dry_t.v * 1e3;
  if (f.mass_t) return f.mass_t.v * 1e3;
  if (f.mass_kg) return f.mass_kg.v;
  return undefined;
};

/** Density of a fuel as stored, from its parts by mass (the sheet's), or undefined if the sheet cannot size it. */
export function fuelDensity(store: Propellants, fuel: string): number | undefined {
  const parts = store[fuel];
  if (!parts?.length) return undefined;
  const specific = parts.reduce((a, x) => a + x.share / x.kg_m3, 0);
  return specific > 0 ? 1 / specific : undefined;
}

export function buildTotals(r: BuildRecipe, placed: readonly PlacedPart[], store: Propellants): BuildTotals {
  const t: BuildTotals = { dry_t: 0, propellant_t: 0, tank_m3: {}, habitat_m3: 0, cargo_m3: 0, radiator_m2: 0, solar_m2: 0, electric_w: 0, thrust_n: 0, exhaust_km_s: 0, advances: [], length_m: 0, across_m: 0, missing: [] };
  const fuel0 = r.fuels?.[0] ?? "propellant";
  let thrustV = 0;
  const noMass: string[] = [];
  for (const q of placed) {
    const p = q.part;
    const f = p.figures;
    const m = kg(p);
    if (m !== undefined) t.dry_t += m / 1e3;
    else if (p.kind !== "truss" || Object.keys(f).length) noMass.push(p.name);
    if (f.propellant_t) t.propellant_t += f.propellant_t.v;
    const vol = f.volume_m3?.v ?? 0;
    if (p.kind === "tank") {
      const fuel = q.built.fuel ?? fuel0;
      t.tank_m3[fuel] = (t.tank_m3[fuel] ?? 0) + vol;
      const rho = fuelDensity(store, fuel);
      if (rho !== undefined) t.propellant_t += (vol * rho) / 1e3;
      else t.missing.push(`${p.name} holds ${fuel}, which the data sheet cannot size, so its load is not counted.`);
    }
    if (p.kind === "habitat") t.habitat_m3 += vol;
    if (p.kind === "pod") t.cargo_m3 += vol;
    if (p.kind === "radiator") t.radiator_m2 += f.area_m2?.v ?? 0;
    if (p.kind === "solar") t.solar_m2 += f.area_m2?.v ?? 0;
    t.electric_w += f.electric_w?.v ?? f.electric_w_at_1_au?.v ?? 0;
    if (p.kind === "engine") {
      if (q.built.drive) {
        t.thrust_n += q.built.drive.thrust_n;
        thrustV += q.built.drive.thrust_n * q.built.drive.exhaust_km_s;
        t.missing.push(`${p.name}: its push and exhaust speed are the ${q.built.drive.src}'s, until the drive's own figures are read.`);
      } else if (p.advance) t.advances.push(p.advance);
      else if (f.thrust_n && f.exhaust_km_s) {
        t.thrust_n += f.thrust_n.v;
        thrustV += f.thrust_n.v * f.exhaust_km_s.v;
      } else t.missing.push(`${p.name} gives no thrust in its record, so it adds none.`);
    }
  }
  t.exhaust_km_s = t.thrust_n > 0 ? thrustV / t.thrust_n : 0;
  if (r.structure_t) {
    t.dry_t += r.structure_t;
    t.missing.push(`${r.structure_t >= 10 ? Math.round(r.structure_t) : Math.round(r.structure_t * 10) / 10} t of structure is carried over from the catalogue record it was opened from, until the parts' own masses are read.`);
  }
  if (noMass.length && !r.structure_t) t.missing.push(`No mass given for ${[...new Set(noMass)].join(", ")}: counted as nothing until their sizing blocks read them (tank walls: sse.ship.tank_wall_mass).`);
  if (t.advances.length) t.missing.push(`${[...new Set(t.advances)].join(", ")}: the drive's figures are the tech advance's at the setting's level, which this sum does not read yet.`);
  const bodies = buildPieces(placed);
  t.length_m = Math.max(0, ...bodies.map((b) => b.x1));
  t.across_m = Math.max(0, ...bodies.map((b) => 2 * (Math.hypot(b.y, b.z) + (b.shape === "panel" ? Math.abs(b.span ?? 0) : Math.max(b.r0, b.r1)))));
  return t;
}

/** The build as a ship record the kernel reads (NAV's ship): masses from the totals, acceleration from the thrust. */
export function buildRecord(r: BuildRecipe, t: BuildTotals): DesignShip {
  const dry = t.dry_t + (r.cargo_t ?? 0);
  const wet = dry + t.propellant_t;
  const accel = wet > 0 ? t.thrust_n / (wet * 1e3 * constant("g0")) : 0;
  return {
    name: r.name,
    accel_g: accel,
    sustained: false,
    exhaust_km_s: t.exhaust_km_s,
    wet_t: Math.max(wet, 1e-3),
    dry_t: Math.max(dry, 1e-3),
    ...(r.fuels?.length ? { fuels: r.fuels } : {}),
    ...(r.crew !== undefined ? { crew: r.crew } : {}),
    ...(r.cargo_t !== undefined ? { cargo_t: r.cargo_t } : {}),
  };
}

/** The build laid out as a designer layout, so the plans, call-outs and 3D model draw it. */
export function buildLayout(r: BuildRecipe, palette: PartsPalette, store: Propellants, rules: DesignRules): ShipLayout & { totals: BuildTotals; placed: PlacedPart[]; record: DesignShip } {
  const { placed, notes } = placeBuild(r, palette);
  const totals = buildTotals(r, placed, store);
  const record = buildRecord(r, totals);
  const pieces = buildPieces(placed);
  return { name: r.name, style: "built", seed: 0, pieces, length_m: totals.length_m, across_m: totals.across_m, needs: needsOf(record, store, rules), notes: [...notes, ...totals.missing], totals, placed, record };
}

/** One budget bar: what the plan needs, what is built, and what is left (negative when the build overshoots: steer, never stop). */
export interface Budget {
  id: string;
  label: string;
  unit: string;
  need: number;
  built: number;
  left: number;
}

/** Build to a plan: the plan's needs (the designer's own needsOf, from the record) beside what is built. */
export function budgets(plan: DesignShip, t: BuildTotals, store: Propellants, rules: DesignRules): Budget[] {
  const n = needsOf(plan, store, rules);
  const out: Budget[] = [];
  const add = (id: string, label: string, unit: string, need: number, built: number) => out.push({ id, label, unit, need, built, left: need - built });
  // Tank room as one bar: the plan's needs are by what each holds (the sheet's names), the build's by fuel.
  const need = n.tanks.reduce((a, k) => a + k.volume_m3, 0);
  if (need > 0) add("tanks", `Tanks (${n.tanks.map((k) => k.name).join(", ")})`, "m3", need, Object.values(t.tank_m3).reduce((a, v) => a + v, 0));
  if (n.crew_m3 > 0) add("habitat", "Room for the crew", "m3", n.crew_m3, t.habitat_m3);
  if (n.cargo_m3 > 0) add("cargo", "Cargo room", "m3", n.cargo_m3, t.cargo_m3);
  if (n.radiator_m2 > 0) add("radiator", "Radiators", "m2", n.radiator_m2, t.radiator_m2);
  add("thrust", "Thrust", "N", n.drive.thrust_n, t.thrust_n);
  return out;
}

const GENERIC: Partial<Record<SystemKind, PartKind>> = { tanks: "tank", crew: "habitat", cargo: "pod", radiators: "radiator", wings: "solar", sensors: "dish", spine: "truss", bay: "bay" };

/** Open a ship the old generator laid out as parts (convert on open, coordinator 27 Nov 11:50): each body or panel
 * becomes the generic palette part of its kind at its own size, clipped in a line along the axis, with the rest on the
 * hull where they sat; the drive is the tech part for its fuel where one fits, or the generic chemical drive. What has
 * no part yet (shields, legs, chutes, scoops, plumes) is named. Saved, it never needs the generator again. */
export function fromLayout(l: ShipLayout, ship: DesignShip, palette: PartsPalette): { recipe: BuildRecipe; notes: string[] } {
  const notes: string[] = [];
  const parts: BuiltPart[] = [];
  const pick = (kind: PartKind) => palette.parts.find((p) => p.kind === kind && p.year === undefined && !p.advance);
  const fuel = ship.fuels?.[0];
  const drivePart = palette.parts.find((p) => p.kind === "engine" && p.advance && fuel && driveFor(fuel) === p.advance) ?? palette.parts.find((p) => p.advance === "chemical-rocket");
  const axial = l.pieces.filter((p) => p.shape === "body" && Math.hypot(p.y, p.z) < 1e-6 && p.system !== "plume").sort((a, b) => a.x0 - b.x0);
  const skipped = new Set<string>();
  let n = 0;
  const keyOf = new Map<Piece, string>();
  let last: string | undefined;
  for (const p of axial) {
    const kind = p.system === "drive" ? "engine" : GENERIC[p.system];
    const part = kind === "engine" ? drivePart : kind ? pick(kind) : undefined;
    if (!part) {
      skipped.add(p.system);
      continue;
    }
    const key = `p${++n}`;
    const drive = kind === "engine" && !part.figures.thrust_n ? { drive: { thrust_n: ship.accel_g * ship.wet_t * 1e3 * constant("g0"), exhaust_km_s: ship.exhaust_km_s, src: `catalogue ship ${ship.name}` } } : {};
    // A part that comes in fixed sizes keeps its own; a shape takes the size the old layout gave it.
    const size = part.scale.law === "fixed" ? {} : { length_m: Math.max(0.1, p.x1 - p.x0), radius_m: Math.max(0.05, Math.max(p.r0, p.r1)) };
    parts.push({ key, part: part.id, ...size, ...(last ? { on: last, place: "front" as const } : {}), ...drive });
    keyOf.set(p, key);
    last = key;
  }
  const carrierAt = (x: number) => axial.find((a) => keyOf.has(a) && a.x0 <= x && x <= a.x1) ?? axial.find((a) => keyOf.has(a));
  for (const p of l.pieces) {
    if (axial.includes(p) || p.system === "plume") continue;
    const kind = GENERIC[p.system];
    const part = kind ? pick(kind) : undefined;
    const mid = (p.x0 + p.x1) / 2;
    const c = carrierAt(mid);
    if (!part || !c) {
      skipped.add(p.system);
      continue;
    }
    const clock = p.shape === "panel" ? ((p.span ?? 1) >= 0 ? 90 : 270) : (Math.atan2(p.y, p.z) * 180) / Math.PI;
    const size = part.scale.law === "fixed" ? {} : { length_m: Math.max(0.1, p.x1 - p.x0), radius_m: Math.max(0.05, p.shape === "panel" ? part.shape.radius_m : Math.max(p.r0, p.r1)) };
    parts.push({ key: `p${++n}`, part: part.id, ...size, on: keyOf.get(c)!, place: "hull", along: (mid - c.x0) / Math.max(c.x1 - c.x0, 1e-6), clock_deg: Math.round(((clock % 360) + 360) % 360) });
  }
  if (skipped.size) notes.push(`No palette part yet for ${[...skipped].join(", ")}, so ${skipped.size > 1 ? "they are" : "it is"} left out of the build.`);
  return { recipe: { format: "sse.build/1", name: ship.name, parts, structure_t: Math.max(0, ship.dry_t - (ship.cargo_t ?? 0)), ...(ship.fuels ? { fuels: [...ship.fuels] } : {}), ...(ship.crew !== undefined ? { crew: ship.crew } : {}), ...(ship.cargo_t !== undefined ? { cargo_t: ship.cargo_t } : {}), plan: ship }, notes };
}

/** The tech advance whose drive burns a base-nav fuel, as the palette's tech parts are named. */
function driveFor(fuel: string): string | undefined {
  if (fuel === "hydrogen") return "nuclear-thermal";
  if (fuel.includes("fusion") || fuel === "deuterium" || fuel === "helium-3") return "fusion-torch";
  if (fuel === "xenon") return "ion-thruster";
  return undefined;
}
