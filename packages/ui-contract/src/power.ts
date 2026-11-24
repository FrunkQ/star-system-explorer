import { constant } from "./sheet.ts";
import { sayNumber } from "./figures.ts";
import type { FlowRow } from "./flow.ts";

// A ship's power books (RK9, kernel `ship_power` and the `power` a voyage
// carries when the ship has fitted engines): what the sources make, what the
// loads and engines take, and any power its stated drive needs that nothing
// gives it ("from nowhere"). The kernel does every sum; this only words them.

export interface PowerFlow {
  from: string;
  to: string;
  amount_w: number;
  kind: "electric" | "heat" | "jet" | "fuel" | string;
  /** For heat: the temperature it leaves the radiators at, K. */
  reject_k?: number;
}

export interface PowerEngine {
  name: string;
  count: number;
  energy_from: "fuel" | "electric" | "beam" | "sunlight";
  thrust_n: number;
  exhaust_km_s: number;
  electric_w: number;
  jet_w: number;
  nowhere_w: number;
}

export interface PowerLedger {
  flows: PowerFlow[];
  made_w: number;
  wanted_w: number;
  spare_w: number;
  engines: PowerEngine[];
  thrust_n: number;
  heat_by_k: Record<string, number>;
  nowhere_w: number;
  why: string[];
}

export interface PowerBooks {
  flux_w_m2: number;
  accel_g: number;
  exhaust_km_s: number;
  ledger: PowerLedger;
  balance: { id: string; demand: number; supply: number; ok: boolean; says?: string };
  fictional: boolean;
  why: string[];
  /** Asked with a panel rule: each temperature's panels, sized by the kernel's `sse.ship.radiator_area` block (RK3). */
  radiators?: PowerRadiator[];
}

/** One temperature's radiator panels, as the kernel sized them. */
export interface PowerRadiator {
  reject_k: number;
  heat_w: number;
  area_m2: number;
  block: string;
  inputs: Record<string, number>;
}

/** Watts in the unit a reader would use: W, kW, MW, GW. */
export function wattsText(w: number): string {
  const a = Math.abs(w);
  const [d, u] = a >= 1e9 ? [1e9, "GW"] : a >= 1e6 ? [1e6, "MW"] : a >= 1e3 ? [1e3, "kW"] : [1, "W"];
  const v = w / d;
  return `${sayNumber(v, 3, { decimals: Math.abs(v) >= 100 ? 0 : Math.abs(v) >= 10 ? 1 : 2 })} ${u}`;
}

/** One line on the ship's power as the voyage flies it. */
export function powerWords(p: PowerBooks): string {
  const l = p.ledger;
  const made = `${wattsText(l.made_w)} made`;
  const spare = l.spare_w >= 0 ? `${wattsText(l.spare_w)} to spare` : `${wattsText(-l.spare_w)} short, so the electric drive throttles back`;
  const heat = Object.values(l.heat_by_k).reduce((a, b) => a + b, 0);
  return `${made}, ${spare}; ${wattsText(heat)} of heat to shed; ${gText(p.accel_g)} from its engines`;
}

/** What the books cannot account for, in words (none when they close). */
export function nowhereWords(p: PowerBooks): string | null {
  if (!p.fictional) return null;
  return `${wattsText(p.ledger.nowhere_w)} from nowhere: the stated drive carries more power than the ship gives it. It flies on its stated numbers, marked fictional.`;
}

// ---- The Power card (RK13): the books as a Flow view, and five readings ----

/** The three columns of the Power card, as plan section 16 names them. */
export const POWER_COLUMNS = ["Comes from", "Used by", "Ends up as"] as const;
export const NOWHERE = "Nowhere";
export const JET = "Jet";
export const FUEL_NODE = "Fuel";

/** The end node for heat leaving at a temperature. */
export const radiatorNode = (k: number): string => `Radiators at ${Math.round(k)} K`;
const plantLoss = (source: string): string => `Lost in ${source}`;

/**
 * The kernel's rows say where power goes through the ship's electric bus. The
 * card reads in three columns, so each draw on the bus is shared among the
 * sources in proportion to what each makes (a common bus mixes them), a plant's
 * own heat is a part that uses power ("Lost in …"), and heat ends at a radiator
 * per temperature. Players get no "Nowhere" rows: it is an honesty mark.
 */
export function powerFlowRows(p: PowerBooks, o: { gm: boolean; ship?: unknown }): FlowRow[] {
  const flows = p.ledger.flows;
  const tempOf = heatTemperatures(p, o.ship);
  const sources = flows.filter((f) => f.to === "electric" && f.amount_w > 0);
  const made = sources.reduce((a, f) => a + f.amount_w, 0);
  const sourceIds = new Set(sources.map((f) => f.from));
  const rows: FlowRow[] = [];
  for (const f of flows) {
    if (!(f.amount_w > 0)) continue;
    if (f.to === "electric") continue;
    if (f.from === "electric") {
      for (const s of sources) rows.push({ from: s.from, to: f.to, amount: (f.amount_w * s.amount_w) / made, kind: "electric" });
      continue;
    }
    if (f.from === "Nowhere") {
      if (o.gm) rows.push({ from: NOWHERE, to: f.to, amount: f.amount_w, kind: f.kind, dashed: true });
      continue;
    }
    if (f.from === "propellant") {
      rows.push({ from: FUEL_NODE, to: f.to, amount: f.amount_w, kind: "fuel" });
      continue;
    }
    if (f.to === "jet") {
      rows.push({ from: f.from, to: JET, amount: f.amount_w, kind: "jet" });
      continue;
    }
    if (f.to === "radiators") {
      const k = f.reject_k ?? tempOf(f.from);
      const end = k === undefined ? RADIATORS : radiatorNode(k);
      const heat = (from: string, to: string): FlowRow => (k === undefined ? { from, to, amount: f.amount_w, kind: "heat" } : { from, to, amount: f.amount_w, kind: "heat", k });
      if (sourceIds.has(f.from)) rows.push(heat(f.from, plantLoss(f.from)), heat(plantLoss(f.from), end));
      else rows.push(heat(f.from, end));
      continue;
    }
    rows.push({ from: f.from, to: f.to, amount: f.amount_w, kind: f.kind });
  }
  return rows;
}

/** The end node for heat whose temperature the books do not say. */
export const RADIATORS = "Radiators";

/**
 * The temperature each part's heat leaves at. The kernel's rows name the part
 * and its ledger sums heat by temperature; the ship's record says each part's
 * `reject_k`. A part with none goes to the one temperature no stated part
 * explains (the kernel's default), when there is exactly one; otherwise none.
 */
export function heatTemperatures(p: PowerBooks, ship: unknown): (part: string) => number | undefined {
  const keys = Object.keys(p.ledger.heat_by_k).map(Number);
  const stated = new Map<string, number | undefined>();
  const s = (ship ?? {}) as { power?: { id?: string; kind?: string; reject_k?: number }[]; loads?: { name?: string; reject_k?: number }[]; engines?: { engine?: unknown }[] };
  for (const x of s.power ?? []) stated.set(x.id || x.kind || "", x.reject_k);
  for (const x of s.loads ?? []) stated.set(x.name ?? "", x.reject_k);
  for (const m of s.engines ?? []) if (m.engine && typeof m.engine === "object") {
    const e = m.engine as { name?: string; reject_k?: number };
    stated.set(e.name || "engine", e.reject_k);
  }
  const explicit = new Set([...stated.values()].filter((k): k is number => k !== undefined).map((k) => Math.round(k)));
  const rest = keys.filter((k) => !explicit.has(k));
  const fallback = keys.length === 1 ? keys[0] : rest.length === 1 ? rest[0] : undefined;
  return (part) => {
    const k = stated.get(part);
    return k !== undefined ? Math.round(k) : fallback;
  };
}

/** Column of a Power card node: sources, fuel and Nowhere; the parts; jet and radiators. */
export function powerColumn(rows: readonly FlowRow[]): (id: string) => number {
  const ends = new Set(rows.filter((r) => r.to === JET || r.to === RADIATORS || r.to.startsWith("Radiators at ")).map((r) => r.to));
  const feeds = new Set(rows.map((r) => r.to));
  return (id) => (ends.has(id) ? 2 : id === NOWHERE || id === FUEL_NODE || !feeds.has(id) ? 0 : 1);
}

/** Order in a column: Nowhere on top, then sources as listed; engines first among the parts; jet, then hottest radiators. */
export function powerRank(p: PowerBooks): (id: string) => number {
  const engines = p.ledger.engines.map((e) => e.name);
  return (id) => {
    if (id === NOWHERE) return -2;
    if (id === FUEL_NODE) return -1;
    if (id === JET) return 0;
    const k = /^Radiators at (\d+) K$/.exec(id);
    if (k) return 1e5 - Number(k[1]);
    if (id === RADIATORS) return 1e5;
    if (id.startsWith("Lost in ")) return 9e4;
    const e = engines.indexOf(id);
    return e >= 0 ? e : 100;
  };
}

/** Radiator panels as the base-ships pack has them (`ship_design.radiator`). */
export interface RadiatorRule {
  emissivity: number;
  two_sided: boolean;
}

export interface PowerReading {
  id: "thrust" | "spare" | "radiators" | "visible";
  label: string;
  /** The figure, in a reader's unit. */
  value: string;
  words: string;
  /** The formula block that worked the figure out, when the kernel did. */
  block?: string;
}

/** Newtons in the unit a reader would use. */
export function newtonsText(n: number): string {
  const a = Math.abs(n);
  return a >= 1e6 ? `${sayNumber(n / 1e6, 3, { decimals: 2 })} MN` : a >= 1e3 ? `${sayNumber(n / 1e3, 3, { decimals: 1 })} kN` : a >= 1 ? `${sayNumber(n, 3, { decimals: 1 })} N` : `${sayNumber(n * 1e3, 3, { decimals: 1 })} mN`;
}

/** An acceleration in g, as milli-g when small. */
export function gText(g: number): string {
  return g < 0.001 ? `${sayNumber(g * 1000, 3, { decimals: 3 })} milli-g` : `${sayNumber(g, 3, { decimals: 3 })} g`;
}

/** Square metres in the unit a reader would use. */
export function areaText(m2: number): string {
  return m2 >= 1e6 ? `${sayNumber(m2 / 1e6, 3, { decimals: 2 })} km²` : m2 >= 100 ? `${sayNumber(m2, 3, { decimals: 0 })} m²` : `${sayNumber(m2, 3, { decimals: 1 })} m²`;
}

/** Panel area to shed `w` watts at `k` kelvin: A = W / (faces × ε × σ T⁴). */
export function radiatorArea(w: number, k: number, rule: RadiatorRule): number {
  const flux = (rule.two_sided ? 2 : 1) * rule.emissivity * constant("sigma_SB") * k ** 4;
  return flux > 0 ? w / flux : 0;
}

/** How far a craft's output spreads: at this distance it lands as this many W/m² (inverse square, all directions). */
export const VISIBLE_AT_M = 1e9;

/**
 * The card's plain readings (plan 16): thrust, power to spare, a radiator per
 * temperature (the hot and the cool ones), how visible the craft is. Visibility
 * is the light and heat it gives off spread over a sphere a million km across;
 * the range a given sensor sees it at waits for the sensor fit (RK14).
 */
export function powerReadings(p: PowerBooks, rule: RadiatorRule): PowerReading[] {
  const l = p.ledger;
  const out: PowerReading[] = [];
  const jet = l.engines.reduce((a, e) => a + e.jet_w, 0);
  out.push(
    l.thrust_n > 0
      ? { id: "thrust", label: "Thrust", value: newtonsText(l.thrust_n), words: `from a ${wattsText(jet)} jet: ${gText(p.accel_g)} for this ship` }
      : { id: "thrust", label: "Thrust", value: "none", words: "no engine is firing" },
  );
  out.push(
    l.spare_w >= 0
      ? { id: "spare", label: "Power to spare", value: wattsText(l.spare_w), words: l.made_w > 0 ? `of ${wattsText(l.made_w)} made` : "nothing on board makes electricity" }
      : { id: "spare", label: "Power to spare", value: "none", words: `${wattsText(-l.spare_w)} short, so the electric engines get less than they ask for` },
  );
  const temps = Object.keys(l.heat_by_k)
    .map(Number)
    .filter((k) => (l.heat_by_k[String(k)] ?? 0) > 0)
    .sort((a, b) => b - a);
  temps.forEach((k, i) => {
    const w = l.heat_by_k[String(k)]!;
    const label = temps.length === 1 ? "Radiators" : i === 0 ? "Hot radiators" : i === temps.length - 1 ? "Cool radiators" : "Warm radiators";
    // The kernel's block when it sized them; the same sum here for an answer asked without the panel rule.
    const sized = p.radiators?.find((r) => r.reject_k === k);
    out.push({ id: "radiators", label, value: areaText(sized ? sized.area_m2 : radiatorArea(w, k, rule)), ...(sized ? { block: sized.block } : {}), words: `at ${k} K for ${wattsText(w)}, emissivity ${rule.emissivity}${rule.two_sided ? ", both faces" : ""}` });
  });
  if (!temps.length) out.push({ id: "radiators", label: "Radiators", value: "none", words: "no heat to shed" });
  const given = jet + Object.values(l.heat_by_k).reduce((a, b) => a + b, 0);
  const at = given / (4 * Math.PI * VISIBLE_AT_M ** 2);
  out.push({
    id: "visible",
    label: "How visible",
    value: wattsText(given),
    words: `given off as heat and exhaust; a million km away that is ${at.toExponential(1)} W/m², ${given > 0 ? `about ${(at / constant("S_sun")).toExponential(0)} of sunlight at Earth` : "nothing to see"}`,
  });
  return out;
}
