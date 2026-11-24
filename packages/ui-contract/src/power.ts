// A ship's power books (RK9, kernel `ship_power` and the `power` a voyage
// carries when the ship has fitted engines): what the sources make, what the
// loads and engines take, and any power its stated drive needs that nothing
// gives it ("from nowhere"). The kernel does every sum; this only words them.

export interface PowerFlow {
  from: string;
  to: string;
  amount_w: number;
  kind: "electric" | "heat" | "jet" | "fuel" | string;
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
}

/** Watts in the unit a reader would use: W, kW, MW, GW. */
export function wattsText(w: number): string {
  const a = Math.abs(w);
  const [d, u] = a >= 1e9 ? [1e9, "GW"] : a >= 1e6 ? [1e6, "MW"] : a >= 1e3 ? [1e3, "kW"] : [1, "W"];
  const v = w / d;
  return `${Math.abs(v) >= 100 ? v.toFixed(0) : Math.abs(v) >= 10 ? v.toFixed(1) : v.toFixed(2)} ${u}`;
}

/** One line on the ship's power as the voyage flies it. */
export function powerWords(p: PowerBooks): string {
  const l = p.ledger;
  const made = `${wattsText(l.made_w)} made`;
  const spare = l.spare_w >= 0 ? `${wattsText(l.spare_w)} to spare` : `${wattsText(-l.spare_w)} short, so the electric drive throttles back`;
  const heat = Object.values(l.heat_by_k).reduce((a, b) => a + b, 0);
  return `${made}, ${spare}; ${wattsText(heat)} of heat to shed; ${p.accel_g < 0.001 ? `${(p.accel_g * 1000).toFixed(3)} milli-g` : `${p.accel_g.toFixed(3)} g`} from its engines`;
}

/** What the books cannot account for, in words (none when they close). */
export function nowhereWords(p: PowerBooks): string | null {
  if (!p.fictional) return null;
  return `${wattsText(p.ledger.nowhere_w)} from nowhere: the stated drive carries more power than the ship gives it. It flies on its stated numbers, marked fictional.`;
}
