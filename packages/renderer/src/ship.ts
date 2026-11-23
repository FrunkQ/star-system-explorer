// V3 gap item 14, part 1: ships drawn from their own physics. A hull is not a model someone uploads; it is built
// from what the ship record says it carries: propellant mass over the propellant's density gives tank volume, the
// shield's areal density over the crew module's skin gives its mass, and the drive's thrust and exhaust speed give
// the plume. The same record that plans a voyage (base-nav `ships`, sse-nav `Ship`) draws the ship.
//
// Drive physics is the relativistic rocket (Ackeret 1946), which is the classical one at chemical speeds:
//   thrust F = m0 a, mass flow mdot = F / (gamma v_e), jet power P = mdot (gamma - 1) c^2,
//   delta-v = c tanh((v_e / c) ln(m0 / m1)), full-thrust burn t = (m0 - m1) / mdot.
// Constants come from the master data sheet; densities come from the sheet's substance rows (phases.density_liquid),
// passed in by the view, never typed here.
import { constant } from "../../ui-contract/src/sheet.ts";

/** A ship as NAV reads it (ui-contract NavShip), with the fuels the base-nav record lists. */
export interface ShipRecord {
  name: string;
  accel_g: number;
  sustained: boolean;
  exhaust_km_s: number;
  wet_t: number;
  dry_t: number;
  fuels?: string[];
  shield_g_cm2?: number;
  jump?: { tank_t?: number };
}

/** One part of a fuel as stored: its share of the fuel's mass and its density, with the sheet's source. */
export interface PropellantPart {
  name: string;
  share: number;
  kg_m3: number;
  src: string;
}

/** How a fuel is stored, by base-nav fuel id. A fuel with no entry has an unknown tank size, and says so. */
export type Propellants = Readonly<Record<string, readonly PropellantPart[]>>;

export interface DriveFacts {
  thrust_n: number;
  mass_flow_kg_s: number;
  jet_power_w: number;
  delta_v_m_s: number;
  burn_s: number;
  /** Kinetic energy each atomic mass unit of exhaust leaves with, in eV. */
  ev_per_u: number;
  /** Wavelength of one photon carrying that energy (h c / E), in metres: the shortest light the jet's energy can
   * make, one atomic mass unit at a time, where it stops in gas or on a surface. */
  photon_m: number;
}

const sq = (x: number) => x * x;

/** The drive's facts from the ship record. Thrust is taken at full tanks, as base-nav quotes accel_g. */
export function driveFacts(s: ShipRecord): DriveFacts {
  const c = constant("c");
  const m0 = s.wet_t * 1e3;
  const m1 = s.dry_t * 1e3;
  const ve = s.exhaust_km_s * 1e3;
  const beta = ve / c;
  const gamma = 1 / Math.sqrt(1 - sq(beta));
  // gamma - 1 without cancellation at chemical speeds.
  const gm1 = (sq(beta) * sq(gamma)) / (gamma + 1);
  const thrust = m0 * s.accel_g * constant("g0");
  const mdot = thrust / (gamma * ve);
  const power = mdot * gm1 * sq(c);
  const dv = m1 > 0 && m0 > m1 ? c * Math.tanh(beta * Math.log(m0 / m1)) : 0;
  const ePerU = gm1 * constant("m_u") * sq(c);
  return {
    thrust_n: thrust,
    mass_flow_kg_s: mdot,
    jet_power_w: power,
    delta_v_m_s: dv,
    burn_s: mdot > 0 ? (m0 - m1) / mdot : 0,
    ev_per_u: ePerU / constant("e"),
    photon_m: (constant("h") * c) / ePerU,
  };
}

/** The band light of a wavelength falls in: a label read off the number, never a gate. */
export function bandOf(m: number): string {
  if (m < 1e-11) return "gamma rays";
  if (m < 1e-8) return "X-rays";
  if (m < 3.8e-7) return "ultraviolet";
  if (m < 7.5e-7) return "visible light";
  if (m < 1e-3) return "infrared";
  return "radio";
}

export type HullPartKind = "engine" | "tank" | "crew";

export interface HullPart {
  kind: HullPartKind;
  name: string;
  /** Along the axis from the nozzle's exit, in metres. */
  from_m: number;
  to_m: number;
  radius_m: number;
  mass_t?: number;
  volume_m3?: number;
  /** True when the sheet does not give this part's size, so it is drawn at a stand-in length. */
  stand_in?: boolean;
  src?: string;
}

export interface Hull {
  name: string;
  radius_m: number;
  length_m: number;
  parts: HullPart[];
  /** Mass the crew module's shield needs, against the ship's dry mass. */
  shield_t: number;
  notes: string[];
}

/** Hull proportions: authored, in the fudge register. */
export const HULL_SHAPE = { fineness: 4, crew_length: 1, engine_length: 1.2, bell_exit: 0.8, throat: 0.25, stand_in_kg_m3: 100 } as const;

/** Build a hull from the record: one tank per stored part of each fuel, a crew module whose skin is the shield,
 * and an engine at the back. The hull's radius is the one at which the tanks make a cylinder `fineness` diameters long. */
export function hullFromShip(s: ShipRecord, store: Propellants): Hull {
  const notes: string[] = [];
  const fuels = s.fuels?.length ? s.fuels : ["propellant"];
  const prop_t = Math.max(0, s.wet_t - s.dry_t);
  const tanks: { name: string; t: number; part?: PropellantPart }[] = [];
  // Several fuels share the propellant mass equally unless the stored parts say otherwise (fudge register).
  for (const f of fuels) {
    const parts = store[f];
    const t = prop_t / fuels.length;
    if (!parts?.length) {
      tanks.push({ name: f, t });
      notes.push(`How ${f} is stored is not on the master data sheet yet, so its tank is drawn at a stand-in size.`);
    } else for (const p of parts) tanks.push({ name: p.name, t: t * p.share, part: p });
  }
  if (s.jump?.tank_t) {
    const parts = store["hydrogen"];
    if (parts?.length) for (const p of parts) tanks.push({ name: `jump fuel (${p.name})`, t: s.jump.tank_t * p.share, part: p });
    else tanks.push({ name: "jump fuel", t: s.jump.tank_t });
  }
  // A fuel the sheet cannot size is drawn at a stand-in density (fudge register), marked as such.
  const vol = (k: { t: number; part?: PropellantPart }) => (k.t * 1e3) / (k.part ? k.part.kg_m3 : HULL_SHAPE.stand_in_kg_m3);
  const volume = Math.max(1, tanks.reduce((a, k) => a + vol(k), 0));
  const r = Math.cbrt(volume / (2 * Math.PI * HULL_SHAPE.fineness));
  const parts: HullPart[] = [];
  let at = 0;
  const engine = HULL_SHAPE.engine_length * r;
  parts.push({ kind: "engine", name: "engine", from_m: 0, to_m: engine, radius_m: HULL_SHAPE.bell_exit * r });
  at = engine;
  for (const k of tanks) {
    const v = k.part ? vol(k) : undefined;
    const len = vol(k) / (Math.PI * r * r);
    parts.push({ kind: "tank", name: k.name, from_m: at, to_m: at + len, radius_m: r, mass_t: k.t, volume_m3: v, stand_in: v === undefined, src: k.part?.src });
    at += len;
  }
  const crewLen = HULL_SHAPE.crew_length * r;
  const sigma = (s.shield_g_cm2 ?? 0) * 10; // g cm-2 to kg m-2
  const skin = 2 * Math.PI * r * crewLen + 2 * Math.PI * r * r;
  const shield_t = (sigma * skin) / 1e3;
  parts.push({ kind: "crew", name: "crew module", from_m: at, to_m: at + crewLen, radius_m: r, mass_t: shield_t || undefined });
  at += crewLen;
  if (shield_t > s.dry_t) notes.push(`Its shield alone would weigh ${round(shield_t)} t, more than the ship's dry mass of ${round(s.dry_t)} t.`);
  return { name: s.name, radius_m: r, length_m: at, parts, shield_t, notes };
}

/** The hull's outline as (along, radius) pairs from the nozzle to the nose, for a lathe (three.js LatheGeometry
 * takes these as x = radius, y = along). The bell narrows to its throat, then the hull runs at full radius. */
export function hullProfile(h: Hull): [number, number][] {
  const e = h.parts[0]!;
  const r = h.radius_m;
  const out: [number, number][] = [[0, 0], [0, e.radius_m]];
  const steps = 8;
  // A bell: radius falls from the exit to the throat along a parabola.
  for (let i = 1; i <= steps; i++) {
    const u = i / steps;
    out.push([u * e.to_m * 0.8, HULL_SHAPE.throat * r + (e.radius_m - HULL_SHAPE.throat * r) * sq(1 - u)]);
  }
  out.push([e.to_m, HULL_SHAPE.throat * r], [e.to_m, r], [h.length_m, r], [h.length_m, 0]);
  return out;
}

export interface PlumeShape {
  /** Length of the Chart glyph in hull radii: grows by one radius per factor of ten in jet power above 1 MW. */
  length_r: number;
  /** Half-angle of the Chart glyph's cone. */
  half_angle_deg: number;
}

/** The Chart glyph for a plume (schematic, in the fudge register): its length reads the jet power on a log scale. */
export function plumeShape(d: DriveFacts): PlumeShape {
  const decades = Math.log10(Math.max(d.jet_power_w, 1e6) / 1e6);
  return { length_r: 1 + decades, half_angle_deg: 12 };
}

/** Side-view SVG paths for the Chart style: hull outline, part dividers and the plume, nozzle to the left.
 * `size` is the frame [width, height] in pixels; the drawing fits with a margin. */
export function shipPaths(h: Hull, d: DriveFacts, size: [number, number]): { hull: string; dividers: string; plume: string; scale: number } {
  const p = plumeShape(d);
  const plumeLen = p.length_r * h.radius_m;
  const total = h.length_m + plumeLen;
  const scale = Math.min((size[0] * 0.9) / total, (size[1] * 0.8) / (2 * Math.max(h.radius_m, plumeLen * Math.tan((p.half_angle_deg * Math.PI) / 180))));
  const x0 = (size[0] - total * scale) / 2 + plumeLen * scale;
  const cy = size[1] / 2;
  const X = (a: number) => (x0 + a * scale).toFixed(1);
  const Y = (rr: number) => (cy - rr * scale).toFixed(1);
  const prof = hullProfile(h);
  const top = prof.map(([a, rr], i) => `${i ? "L" : "M"}${X(a)} ${Y(rr)}`).join(" ");
  const bottom = [...prof].reverse().map(([a, rr]) => `L${X(a)} ${Y(-rr)}`).join(" ");
  // The step from the engine to the first tank is already in the outline.
  const dividers = h.parts.slice(2).map((q) => `M${X(q.from_m)} ${Y(q.radius_m)} L${X(q.from_m)} ${Y(-q.radius_m)}`).join(" ");
  const e = h.parts[0]!.radius_m;
  const spread = e + plumeLen * Math.tan((p.half_angle_deg * Math.PI) / 180);
  const plume = `M${X(0)} ${Y(e)} L${X(-plumeLen)} ${Y(spread)} L${X(-plumeLen)} ${Y(-spread)} L${X(0)} ${Y(-e)} Z`;
  return { hull: `${top} ${bottom} Z`, dividers, plume, scale };
}

const round = (x: number) => (x >= 100 ? Math.round(x).toLocaleString("en-GB") : x >= 10 ? x.toFixed(0) : x.toPrecision(2));

function siWords(x: number, unit: string): string {
  const steps: [number, string][] = [[1e18, "E"], [1e15, "P"], [1e12, "T"], [1e9, "G"], [1e6, "M"], [1e3, "k"], [1, ""]];
  for (const [f, p] of steps) if (Math.abs(x) >= f) return `${round(x / f)} ${p}${unit}`;
  return `${x.toPrecision(2)} ${unit}`;
}

function timeWords(s: number): string {
  const day = constant("day");
  if (s < 120) return `${round(s)} seconds`;
  if (s < 7200) return `${round(s / 60)} minutes`;
  if (s < 2 * day) return `${round(s / 3600)} hours`;
  return `${round(s / day)} days`;
}

/** Words for the text tier and the Curious depth. */
export function narrateShip(h: Hull, d: DriveFacts): string {
  const tanks = h.parts.filter((p) => p.kind === "tank");
  const sized = tanks.filter((p) => !p.stand_in);
  const lines = [
    `${h.name}: ${round(h.length_m)} m long and ${round(2 * h.radius_m)} m across, built round ${round(sized.reduce((a, p) => a + (p.volume_m3 ?? 0), 0))} m³ of tanks.`,
    `Thrust ${siWords(d.thrust_n, "N")}; the jet carries ${siWords(d.jet_power_w, "W")} and the tanks last ${timeWords(d.burn_s)} at full thrust, for ${round(d.delta_v_m_s / 1e3)} km/s of change in speed.`,
    `Each hydrogen atom's weight of exhaust carries ${siWords(d.ev_per_u, "eV")}: where the jet stops, that is enough for light up to ${bandOf(d.photon_m)}.`,
  ];
  if (h.shield_t > 0) lines.push(`The crew module's shield weighs ${round(h.shield_t)} t.`);
  return [...lines, ...h.notes].join(" ");
}

export const SHIP_REFS = {
  rocket: "Ackeret, J. 1946, Zur Theorie der Raketen, Helvetica Physica Acta 19, 103 (the relativistic rocket equation)",
  tsiolkovsky: "Tsiolkovsky, K. E. 1903, Exploration of the Universe with Reaction Machines, Nauchnoye Obozreniye 5",
  photon: "Planck: a photon of energy E has wavelength h c / E, with h and c from the master data sheet",
} as const;
