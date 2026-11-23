// V3 gap item 14, stage 2: a ship designed from its parts. Each system's size comes from what it must do (tanks
// from propellant over density, quarters from crew, holds from cargo, radiators from the drive's waste heat), using
// rules from the base-ships pack. A style (also pack data) only arranges the parts and sets the look; it never
// changes a size. A design is kept as a recipe (ship, style, seed, hand overrides), never as a mesh, and is drawn
// from that recipe on any machine: a side view and a top view with call-outs here, a 3D model in the next stage.
import { constant } from "../../ui-contract/src/index.ts";
import type { SkinSpec } from "./ship-mesh.ts";
import { driveFacts, plumeShape, tanksOf, type DriveFacts, type Propellants, type ShipRecord, type TankNeed } from "./ship.ts";

export type SystemKind = "sensors" | "crew" | "cargo" | "tanks" | "radiators" | "drive" | "spine" | "plume";

/** A ship as the designer reads it: the NAV record plus the fields the travel thread is adding. */
export interface DesignShip extends ShipRecord {
  crew?: number;
  cargo_t?: number;
  /** Waste heat the drive leaves in the ship, per watt of jet. Overrides the pack's rule for the ship's fuel. */
  waste_per_jet?: number;
}

/** base-ships `ship_design`. */
export interface DesignRules {
  crew: { habitable_m3_per_person: number; net_to_gross: number };
  cargo: { stowage_kg_m3: number };
  radiator: { emissivity: number; two_sided: boolean };
  drive_heat: { fuels: string[]; waste_per_jet: number; radiator_k: number; why: string }[];
}

/** base-ships `ship_styles` entry. Ranges are [low, high]; a remix picks inside them. */
export interface ShipStyle {
  name: string;
  order: SystemKind[];
  tank_cluster: number[];
  tank_fineness: [number, number];
  drive_gap_r: [number, number];
  nose_r?: [number, number];
  /** A radiator wing's span over its chord. */
  radiator_aspect: [number, number];
  spine_width_r: number;
  sensor_r: number;
  hull: "open" | "skin";
  /** The look of each surface in 3D (ship-mesh.ts SkinSpec). */
  skin?: SkinSpec;
}

/** Sizes set by hand win over the rules, and the words say so. */
export interface DesignOverrides {
  crew_m3?: number;
  cargo_m3?: number;
  radiator_m2?: number;
}

/** What each system must hold, before any style arranges it. */
export interface Needs {
  drive: DriveFacts;
  tanks: TankNeed[];
  crew: number;
  crew_m3: number;
  cargo_m3: number;
  waste_w: number;
  radiator_k: number;
  /** One face's area of all radiator panels together. */
  radiator_m2: number;
  heat_why: string;
  by_hand: (keyof DesignOverrides)[];
  notes: string[];
}

export function needsOf(s: DesignShip, store: Propellants, rules: DesignRules, over: DesignOverrides = {}): Needs {
  const drive = driveFacts(s);
  const { tanks, notes } = tanksOf(s, store);
  const crew = s.crew ?? 0;
  const crew_m3 = over.crew_m3 ?? (crew * rules.crew.habitable_m3_per_person) / rules.crew.net_to_gross;
  const cargo_m3 = over.cargo_m3 ?? ((s.cargo_t ?? 0) * 1e3) / rules.cargo.stowage_kg_m3;
  // The drive's heat follows its energy source, the first fuel the record lists (antimatter, not the hydrogen it heats).
  const source = s.fuels?.[0];
  const heat = source === undefined ? undefined : rules.drive_heat.find((h) => h.fuels.includes(source));
  const ratio = s.waste_per_jet ?? heat?.waste_per_jet ?? 0;
  const waste_w = ratio * drive.jet_power_w;
  const T = heat?.radiator_k ?? 0;
  const faces = rules.radiator.two_sided ? 2 : 1;
  const flux = faces * rules.radiator.emissivity * constant("sigma_SB") * T ** 4;
  const radiator_m2 = over.radiator_m2 ?? (waste_w > 0 && flux > 0 ? waste_w / flux : 0);
  if (s.crew === undefined) notes.push("The ship record gives no crew, so no quarters are drawn.");
  const by_hand = (Object.keys(over) as (keyof DesignOverrides)[]).filter((k) => over[k] !== undefined);
  return { drive, tanks, crew, crew_m3, cargo_m3, waste_w, radiator_k: T, radiator_m2, heat_why: heat?.why ?? "", by_hand, notes };
}

/** One solid of the layout. Axis x runs from the stern (0) to the nose; y is to the side, z is up.
 * A "body" is a solid of revolution about a line parallel to x through (y, z), radius r0 at x0 and r1 at x1.
 * A "panel" is flat in the x-y plane: from x0 to x1 at its root (y), reaching `span` outward (sign of y), its tip
 * swept back by `sweep` metres. */
export interface Piece {
  system: SystemKind;
  shape: "body" | "panel";
  x0: number;
  x1: number;
  r0: number;
  r1: number;
  y: number;
  z: number;
  span?: number;
  sweep?: number;
  label?: string;
}

export interface ShipLayout {
  name: string;
  style: string;
  seed: number;
  pieces: Piece[];
  length_m: number;
  /** Widest extent across, radiators included. */
  across_m: number;
  needs: Needs;
  /** What this arrangement could not do well, in words. */
  notes: string[];
}

/** Recipe: everything needed to rebuild the same design anywhere. */
export interface DesignRecipe {
  style: string;
  seed: number;
  overrides?: DesignOverrides;
}

/** A small deterministic generator (mulberry32) for remix choices. */
function rng(seed: number): () => number {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

const between = (r: () => number, [a, b]: [number, number]) => a + (b - a) * r();
const pick = <T>(r: () => number, xs: readonly T[]) => xs[Math.min(xs.length - 1, Math.floor(r() * xs.length))]!;
/** Radius of a cylinder `fineness` diameters long that holds `v`. */
const radiusFor = (v: number, fineness: number) => Math.cbrt(Math.max(v, 1e-6) / (2 * Math.PI * fineness));

/** Lay a ship out in a style. The same ship, style and seed always give the same layout. */
export function designShip(s: DesignShip, store: Propellants, rules: DesignRules, styleId: string, style: ShipStyle, seed: number, over: DesignOverrides = {}): ShipLayout {
  const n = needsOf(s, store, rules, over);
  const r = rng(seed);
  const cluster = pick(r, style.tank_cluster);
  const fineness = between(r, style.tank_fineness);
  const gapR = between(r, style.drive_gap_r);
  const noseR = style.nose_r ? between(r, style.nose_r) : 0;
  const aspect = between(r, style.radiator_aspect);
  // Each of the two wings carries half the area; its chord follows from the area and the style's aspect.
  const wingChord = n.radiator_m2 > 0 ? Math.sqrt(n.radiator_m2 / 2 / aspect) : 0;
  const pieces: Piece[] = [];
  const notes: string[] = [];
  const tankV = n.tanks.reduce((a, t) => a + t.volume_m3, 0);
  const skin = style.hull === "skin";

  // The hull radius: for a skin, one body round everything; for a spine, the tank cluster's outer radius.
  const allV = tankV + n.crew_m3 + n.cargo_m3;
  const R = skin ? radiusFor(allV, fineness) : 0;
  const tankR = skin ? R : radiusFor(Math.max(...n.tanks.map((t) => t.volume_m3), 1) / cluster, fineness);
  const ring = cluster === 1 ? 0 : tankR / Math.sin(Math.PI / cluster);
  const outer = skin ? R : ring + tankR;
  const crewR = skin ? R : radiusFor(n.crew_m3, 0.5);
  const cargoR = skin ? R : radiusFor(n.cargo_m3, 0.5);
  const big = Math.max(outer, n.crew_m3 > 0 ? crewR : 0, n.cargo_m3 > 0 ? cargoR : 0);

  // Walk from the nose aft (x falling), then shift so the stern is at 0.
  let x = 0;
  // A body of length `len` ending at x, radius `fore` at its front and `aft` at its back.
  const body = (system: SystemKind, len: number, fore: number, aft: number, y = 0, z = 0, label?: string) => {
    pieces.push({ system, shape: "body", x0: x - len, x1: x, r0: aft, r1: fore, y, z, ...(label ? { label } : {}) });
  };
  let radiatorSlot: [number, number] | null = null;
  for (const sys of style.order) {
    if (sys === "sensors") {
      const len = skin ? noseR * R : style.sensor_r * big;
      body("sensors", len, skin ? 0 : style.sensor_r * big * 0.6, skin ? R : style.sensor_r * big * 0.6, 0, 0, "sensors");
      x -= len;
    } else if (sys === "crew" && n.crew_m3 > 0) {
      const len = n.crew_m3 / (Math.PI * crewR * crewR);
      body("crew", len, crewR, crewR, 0, 0, `crew quarters, ${n.crew} aboard`);
      x -= len;
    } else if (sys === "cargo" && n.cargo_m3 > 0) {
      const len = n.cargo_m3 / (Math.PI * cargoR * cargoR);
      body("cargo", len, cargoR, cargoR, 0, 0, "cargo hold");
      x -= len;
    } else if (sys === "tanks") {
      for (const t of n.tanks) {
        const len = t.volume_m3 / (cluster * Math.PI * tankR * tankR);
        for (let k = 0; k < cluster; k++) {
          const a = (2 * Math.PI * k) / cluster + Math.PI / 4;
          body("tanks", len, tankR, tankR, ring * Math.cos(a), ring * Math.sin(a), k === 0 ? `${t.name} tank${cluster > 1 ? `s (${cluster})` : ""}` : undefined);
        }
        x -= len;
      }
    } else if (sys === "radiators" && n.radiator_m2 > 0) {
      const len = Math.max(2 * big, wingChord);
      radiatorSlot = [x - len, x];
      x -= len;
    } else if (sys === "drive") {
      if (!skin && gapR > 0) {
        const len = gapR * big;
        x -= len;
      }
      const len = 1.2 * big;
      body("drive", len, 0.35 * big, 0.8 * big, 0, 0, "drive");
      x -= len;
    }
  }
  const length = -x;
  for (const p of pieces) (p.x0 += length), (p.x1 += length);
  // The spine: a truss from the drive to the nose, drawn behind the modules.
  if (!skin) pieces.unshift({ system: "spine", shape: "body", x0: 0.6 * big, x1: length, r0: style.spine_width_r * big, r1: style.spine_width_r * big, y: 0, z: 0, label: "spine" });
  // Radiators: two wings, each half of the area. A spine carries them flat off its middle; a skin sweeps them back
  // as fins along the aft half.
  if (n.radiator_m2 > 0) {
    // A skin has no slot: its fins sit along the aft hull, their chord at most 60% of the hull, so a big area
    // becomes a long span; the sweep stops the tips at the stern, clear of the plume.
    const skinChord = Math.min(wingChord, 0.6 * (length - 1.2 * big));
    const [a, b] = radiatorSlot ? [radiatorSlot[0] + length, radiatorSlot[0] + length + wingChord] : [1.2 * big, 1.2 * big + skinChord];
    const chord = b - a;
    const span = n.radiator_m2 / 2 / chord;
    const sweep = skin ? Math.min(span * Math.tan((40 * Math.PI) / 180), a) : 0;
    const root = skin ? R : style.spine_width_r * big;
    if (skin && span > 10 * length)
      notes.push(`A smooth hull has nowhere to put this much radiator: its fins would reach ${lengthWords(span)} out on each side. Ships with drives this hot carry radiators on booms.`);
    for (const side of [1, -1])
      pieces.push({ system: "radiators", shape: "panel", x0: a, x1: b, r0: 0, r1: 0, y: side * root, z: 0, span: side * span, sweep, ...(side > 0 ? { label: `radiators at ${Math.round(n.radiator_k)} K` } : {}) });
  }
  const across = Math.max(2 * big, ...pieces.filter((p) => p.shape === "panel").map((p) => 2 * (Math.abs(p.y) + Math.abs(p.span ?? 0))));
  return { name: s.name, style: styleId, seed, pieces, length_m: length, across_m: across, needs: n, notes };
}

/** A handful of remixes: the same ship and style, different choices inside the style's ranges. */
export function remixes(s: DesignShip, store: Propellants, rules: DesignRules, styleId: string, style: ShipStyle, count: number, first = 1, over: DesignOverrides = {}): ShipLayout[] {
  return Array.from({ length: count }, (_, i) => designShip(s, store, rules, styleId, style, first + i, over));
}

export interface ViewPath {
  system: SystemKind;
  d: string;
}

export interface Callout {
  system: SystemKind;
  text: string;
  anchor: [number, number];
  label: [number, number];
  /** Labels above the drawing hang from their baseline; below, from their top. */
  above: boolean;
}

export interface ShipView {
  paths: ViewPath[];
  callouts: Callout[];
}

/** Side view (looking along +y: x across, z up) and top view (looking down: x across, y up), at one scale that fits
 * both in frames of `size` pixels with room above and below for call-outs. The drive's plume is drawn behind the stern. */
export function shipViews(l: ShipLayout, size: [number, number], fit: "all" | "hull" = "all"): { side: ShipView; top: ShipView; scale: number; plume_m: number; clipped: boolean } {
  const [W, H] = size;
  const big = Math.max(...l.pieces.filter((p) => p.shape === "body" && p.system !== "spine").map((p) => Math.max(p.r0, p.r1) + Math.hypot(p.y, p.z)));
  const plume_m = plumeShape(l.needs.drive).length_r * big;
  // "hull" fits the ship's own bodies and lets wide radiators run off the frame; "all" fits everything.
  const bodies = l.pieces.filter((p) => p.shape === "body");
  const hullLen = Math.max(...bodies.map((p) => p.x1)) - Math.min(0, ...bodies.map((p) => p.x0));
  const total = (fit === "hull" ? hullLen : l.length_m) + plume_m;
  const band = 0.56; // the drawing's share of the height; the rest is call-outs
  const scale = Math.min((W * 0.92) / total, (H * band) / (fit === "hull" ? 2 * big : Math.max(l.across_m, 2 * big)));
  const clipped = fit === "hull" && (l.across_m > 2 * big * 1.01 || l.length_m > hullLen * 1.01);
  const x0 = (W - total * scale) / 2 + plume_m * scale;
  const cy = H / 2;
  const X = (x: number) => x0 + x * scale;
  const f = (v: number) => v.toFixed(1);
  const view = (lat: "y" | "z"): ShipView => {
    const paths: ViewPath[] = [];
    const anchors = new Map<SystemKind, { at: [number, number]; text: string }>();
    const Y = (v: number) => cy - v * scale;
    for (const p of l.pieces) {
      let d: string;
      let at: [number, number];
      if (p.shape === "body") {
        const c = lat === "y" ? p.y : p.z;
        d = `M${f(X(p.x0))} ${f(Y(c + p.r0))} L${f(X(p.x1))} ${f(Y(c + p.r1))} L${f(X(p.x1))} ${f(Y(c - p.r1))} L${f(X(p.x0))} ${f(Y(c - p.r0))} Z`;
        at = [X((p.x0 + p.x1) / 2), Y(c)];
      } else if (lat === "y") {
        const s = p.span ?? 0;
        const sw = p.sweep ?? 0;
        d = `M${f(X(p.x0))} ${f(Y(p.y))} L${f(X(p.x1))} ${f(Y(p.y))} L${f(X(p.x1 - sw))} ${f(Y(p.y + s))} L${f(X(p.x0 - sw))} ${f(Y(p.y + s))} Z`;
        at = [X((p.x0 + p.x1) / 2 - sw / 2), Y(p.y + s / 2)];
      } else {
        // Edge on: a panel is a thin line in the side view.
        const t = Math.max(0.6 / scale, 0.02 * big);
        d = `M${f(X(p.x0))} ${f(Y(t))} L${f(X(p.x1))} ${f(Y(t))} L${f(X(p.x1))} ${f(Y(-t))} L${f(X(p.x0))} ${f(Y(-t))} Z`;
        at = [X((p.x0 + p.x1) / 2), Y(0)];
      }
      paths.push({ system: p.system, d });
      // An anchor off the frame (radiators running off a "hull" picture) is pulled to the frame's edge.
      at = [Math.min(W - 4, Math.max(4, at[0])), Math.min(cy + (H * band) / 2, Math.max(cy - (H * band) / 2, at[1]))];
      if (p.label && !anchors.has(p.system)) anchors.set(p.system, { at, text: p.label });
      else if (p.label && p.system === "tanks") anchors.set(`tanks:${p.label}` as SystemKind, { at, text: p.label });
    }
    // The plume, from the drive's exit.
    const drive = l.pieces.find((p) => p.system === "drive");
    if (drive) {
      const e = drive.r0;
      const spread = e + plume_m * Math.tan((plumeShape(l.needs.drive).half_angle_deg * Math.PI) / 180);
      paths.unshift({ system: "plume", d: `M${f(X(0))} ${f(Y(e))} L${f(X(-plume_m))} ${f(Y(spread))} L${f(X(-plume_m))} ${f(Y(-spread))} L${f(X(0))} ${f(Y(-e))} Z` });
    }
    // Call-outs: sorted along the ship, alternating above and below, spread evenly so labels never overlap.
    const list = [...anchors.entries()].sort((a, b) => a[1].at[0] - b[1].at[0]);
    const up = list.filter((_, i) => i % 2 === 0);
    const down = list.filter((_, i) => i % 2 === 1);
    // Each label sits over its anchor, pushed along just enough to clear its neighbour, then pulled back inside.
    const place = (row: typeof list, above: boolean): Callout[] => {
      const widths = row.map(([, a]) => a.text.length * 5.6 + 10);
      const xs = row.map(([, a]) => a.at[0]);
      for (let i = 0; i < xs.length; i++) xs[i] = Math.max(xs[i]!, widths[i]! / 2, i ? xs[i - 1]! + (widths[i - 1]! + widths[i]!) / 2 : 0);
      for (let i = xs.length - 1; i >= 0; i--) xs[i] = Math.min(xs[i]!, W - widths[i]! / 2, i < xs.length - 1 ? xs[i + 1]! - (widths[i + 1]! + widths[i]!) / 2 : W);
      return row.map(([sys, a], i) => ({
        system: (sys.startsWith("tanks:") ? "tanks" : sys) as SystemKind,
        text: a.text,
        anchor: a.at,
        label: [xs[i]!, above ? H * 0.1 : H * 0.92],
        above,
      }));
    };
    // Draw order: plume, spine and radiators behind the bodies.
    const rank = (s: SystemKind) => (s === "plume" ? 0 : s === "spine" ? 1 : s === "radiators" ? 2 : 3);
    paths.sort((p, q) => rank(p.system) - rank(q.system));
    return { paths, callouts: [...place(up, true), ...place(down, false)] };
  };
  return { side: view("z"), top: view("y"), scale, plume_m, clipped };
}

const words = (x: number) => (x >= 1e4 ? Math.round(x).toLocaleString("en-GB") : x >= 100 ? Math.round(x).toString() : x >= 10 ? x.toFixed(0) : x.toPrecision(2));

function lengthWords(m: number): string {
  return m >= 2000 ? `${words(m / 1000)} km` : `${words(m)} m`;
}

function powerWords(w: number): string {
  for (const [f, u] of [[1e15, "PW"], [1e12, "TW"], [1e9, "GW"], [1e6, "MW"], [1e3, "kW"]] as const) if (w >= f) return `${words(w / f)} ${u}`;
  return `${words(w)} W`;
}

/** Words for the design, for the text tier and the call-outs' "why". */
export function narrateDesign(l: ShipLayout, style: ShipStyle): string {
  const n = l.needs;
  const out = [`${l.name}, ${style.name.toLowerCase()}: ${lengthWords(l.length_m)} long and ${lengthWords(l.across_m)} across.`];
  if (n.crew > 0) out.push(`Quarters for ${n.crew} hold ${words(n.crew_m3)} m³.`);
  if (n.cargo_m3 > 0) out.push(`The hold takes ${words(n.cargo_m3)} m³.`);
  out.push(`Tanks hold ${words(n.tanks.reduce((a, t) => a + t.volume_m3, 0))} m³ of propellant.`);
  if (n.radiator_m2 > 0) {
    const area = n.radiator_m2 >= 1e6 ? `${words(n.radiator_m2 / 1e6)} km²` : `${words(n.radiator_m2)} m²`;
    out.push(`Its drive leaves ${powerWords(n.waste_w)} of waste heat, so its radiators span ${area} at ${Math.round(n.radiator_k)} K.`);
  } else out.push("Its drive carries its own heat away in the exhaust, so it needs no big radiators.");
  if (n.by_hand.length) out.push(`Set by hand: ${n.by_hand.map((k) => k.replace("_m3", "").replace("_m2", "")).join(", ")}.`);
  return [...out, ...n.notes, ...l.notes].join(" ");
}
