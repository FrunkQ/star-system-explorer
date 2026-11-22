// The Reference screen's reading of the master data sheet (matter-pack design section 8, MP10): everything comes
// from the kernel's `read_matter`, so a pack's or a campaign's edit shows the moment it is loaded. Nothing here is
// typed: values, units, sources, marks and group wording are the sheet's. This module only words and lays them out.
import type { Json } from "./json.ts";

/** A value on the sheet: the number in its unit, with its source and how it was checked. */
export interface SheetValue {
  v: number;
  unit?: string;
  src?: string;
  also?: string[];
  uncertainty?: number;
  check?: string;
  note?: string;
  /** Exact by definition (the SI's defining constants, the IAU's conventions). */
  exact?: boolean;
}

/** One row of the Constants page. */
export interface ConstantRow {
  id: string;
  name: string;
  group: string;
  value: SheetValue;
  note?: string;
  derived_from?: string;
  /** "theoretical" is the only row mark (DM1): the amount is measured, its nature is not known. */
  mark?: string;
  mark_note?: string;
}

/** One row of the Units page. */
export interface UnitRow {
  id: string;
  name: string;
  symbol: string;
  measures: string;
  group: string;
  factor?: { v: number; src?: string };
  /** A factor worked out from constants, as a formula over their ids. */
  factor_from?: string;
  offset?: number;
}

export interface SheetSource {
  id: string;
  text: string;
  doi?: string;
  url?: string;
  status?: string;
}

export interface PageGroup {
  id: string;
  name: string;
  about: string;
  notes?: { text: string; src: string; also?: string[] }[];
}

/** One row read alone: the row, the packs that set it, its sources, and which blocks read or skip it. */
export interface RowDetail<R> {
  id: string;
  table: string;
  row: R;
  packs: string[];
  sources: SheetSource[];
  used_by: string[];
  skipped_by: string[];
}

/** The kernel door the screen needs (a ViewSession has it). */
export interface MatterReader {
  readMatter(ask: { table?: string; id?: string }): Promise<{ ok: true; value: Json } | { ok: false; code: string; message: string }>;
}

export interface ConstantsPage {
  groups: PageGroup[];
  rows: ConstantRow[];
  units: UnitRow[];
}

/** Read the Constants and Units pages: the listing, then each row (cheap: the sheet is in memory in the kernel). */
export async function readConstantsPage(k: MatterReader): Promise<{ ok: true; page: ConstantsPage } | { ok: false; message: string }> {
  const l = await k.readMatter({});
  if (!l.ok) return { ok: false, message: l.message };
  const list = l.value as unknown as { constants?: string[]; units?: string[]; groups?: Record<string, PageGroup> };
  const one = async <R>(table: string, id: string): Promise<R | null> => {
    const r = await k.readMatter({ table, id });
    return r.ok ? ((r.value as unknown as RowDetail<R>).row ?? null) : null;
  };
  const rows = (await Promise.all((list.constants ?? []).map((id) => one<ConstantRow>("constants", id)))).filter((r): r is ConstantRow => r !== null);
  const units = (await Promise.all((list.units ?? []).map((id) => one<UnitRow>("units", id)))).filter((r): r is UnitRow => r !== null);
  // The sheet's own order of groups is the order it lists them in; any group a row names that the listing lacks
  // still gets a section, so no row is ever hidden.
  const groups = Object.values(list.groups ?? {});
  for (const r of rows) if (!groups.some((g) => g.id === r.group)) groups.push({ id: r.group, name: r.group, about: "" });
  return { ok: true, page: { groups: orderGroups(groups), rows, units } };
}

/** Nature first, then conventions, factors, cosmos, then anything a pack adds. */
function orderGroups(gs: PageGroup[]): PageGroup[] {
  const at = (id: string) => {
    const i = ["nature", "convention", "factor", "cosmos"].indexOf(id);
    return i < 0 ? 99 : i;
  };
  return [...gs].sort((a, b) => at(a.id) - at(b.id) || a.id.localeCompare(b.id));
}

/** One row with its sources and readers, for its card. */
export async function readRow<R>(k: MatterReader, table: string, id: string): Promise<RowDetail<R> | null> {
  const r = await k.readMatter({ table, id });
  return r.ok ? (r.value as unknown as RowDetail<R>) : null;
}

/** One source by id (kernel `read_matter` on the sources table), or null when the kernel cannot give it. */
export async function readSource(k: MatterReader, id: string): Promise<SheetSource | null> {
  const r = await k.readMatter({ table: "sources", id });
  return r.ok ? (((r.value as unknown as { row?: SheetSource }).row ?? null) as SheetSource | null) : null;
}

const SUP: Record<string, string> = { "-": "⁻", "0": "⁰", "1": "¹", "2": "²", "3": "³", "4": "⁴", "5": "⁵", "6": "⁶", "7": "⁷", "8": "⁸", "9": "⁹" };
const sup = (s: string) => [...s].map((c) => SUP[c] ?? c).join("");

/** Digits in threes either side of the point, with thin spaces, as the sheet's printed sources set them. */
function group3(digits: string): string {
  const [i, f] = digits.split(".");
  const neg = i!.startsWith("-");
  const int = (neg ? i!.slice(1) : i!).replace(/\B(?=(\d{3})+(?!\d))/g, " ");
  const frac = f ? f.replace(/(\d{3})(?=\d)/g, "$1 ") : "";
  return `${neg ? "−" : ""}${int}${frac ? `.${frac}` : ""}`;
}

/**
 * A value as the page prints it. `full` keeps every digit the sheet gives (Science and Engine); otherwise four
 * significant figures (Curious). Large and small numbers go to "× 10ⁿ"; whole numbers below a trillion stay whole.
 */
export function formatValue(v: number, full = true): string {
  if (!Number.isFinite(v)) return String(v);
  if (v === 0) return "0";
  const a = Math.abs(v);
  if (Number.isInteger(v) && a < 1e12 && (full || a < 1e5)) return group3(String(v));
  if (a >= 1e-3 && a < 1e5) return group3(full ? String(v) : String(Number(v.toPrecision(4))));
  const [m, e] = (full ? v.toExponential() : v.toExponential(3)).split("e");
  const mant = full ? m! : String(Number(m));
  return `${group3(mant)} × 10${sup(String(Number(e)))}`;
}

/** "m3 kg-1 s-2" as "m³ kg⁻¹ s⁻²"; a pure number ("1") has no unit. */
export function formatUnit(u: string | undefined): string {
  if (!u || u === "1") return "";
  return u
    .split(/\s+/)
    .map((t) => t.replace(/^([A-Za-zµ°Ω☉⊕/]+)(-?\d+)$/, (_, b: string, p: string) => b + sup(p)))
    .join(" ");
}

/** A source the sheet names only by id ("clowe2006", "codata2018") as words: "Clowe 2006", "CODATA 2018". */
export function sourceName(id: string): string {
  return id.replace(/^([a-z]+)[-_]?(\d{4})([a-z\d]*)$/, (_, n: string, y: string, s: string) => `${n.length <= 6 && /^(codata|iau|si|nist|iupac)$/.test(n) ? n.toUpperCase() : n[0]!.toUpperCase() + n.slice(1)} ${y}${s ? ` ${s.toUpperCase()}` : ""}`);
}

/** A short note for the row's last column at Science depth: how the number is known. */
export function rowNote(r: ConstantRow): string {
  if (r.value.exact) return "exact";
  if (r.derived_from) return "worked out";
  if (typeof r.value.uncertainty === "number" && r.value.uncertainty > 0) return `± ${formatValue(r.value.uncertainty)}`;
  return r.value.src ? sourceName(r.value.src) : "";
}

/**
 * Evaluate a sheet formula ("c yr_julian", "au 648000 / pi", "2 pi^5 k_B^4 / (15 h^3 c^2)", "1 - Omega_m") with
 * the sheet's own values: juxtaposition multiplies, ^ binds tightest. Returns null for anything it cannot read, so
 * the page then shows the formula itself. Display arithmetic only; physics never reads this.
 */
export function evalFormula(f: string, values: ReadonlyMap<string, number>): number | null {
  const toks = f.match(/\d+(?:\.\d+)?(?:e[+-]?\d+)?|[A-Za-z_][A-Za-z0-9_]*|[()^*/+-]/g);
  if (!toks || toks.join("") !== f.replace(/\s+/g, "")) return null;
  let i = 0;
  const peek = () => toks[i];
  const atom = (): number | null => {
    const t = toks[i++];
    if (t === undefined) return null;
    if (t === "(") {
      const v = sum();
      if (toks[i++] !== ")") return null;
      return v;
    }
    if (t === "-") {
      const v = power();
      return v === null ? null : -v;
    }
    if (/^\d/.test(t)) return Number(t);
    if (t === "pi") return Math.PI;
    return values.get(t) ?? null;
  };
  const power = (): number | null => {
    const b = atom();
    if (b === null) return null;
    if (peek() === "^") {
      i++;
      const e = power();
      return e === null ? null : b ** e;
    }
    return b;
  };
  const product = (): number | null => {
    let v = power();
    while (v !== null) {
      const t = peek();
      if (t === "*" || t === "/") {
        i++;
        const r = power();
        if (r === null) return null;
        v = t === "*" ? v * r : v / r;
      } else if (t !== undefined && t !== ")" && t !== "+" && t !== "-" && t !== "^") {
        const r = power();
        if (r === null) return null;
        v *= r;
      } else break;
    }
    return v;
  };
  const sum = (): number | null => {
    let v = product();
    while (v !== null && (peek() === "+" || peek() === "-")) {
      const t = toks[i++];
      const r = product();
      if (r === null) return null;
      v = t === "+" ? v + r : v - r;
    }
    return v;
  };
  const v = sum();
  return v !== null && i === toks.length && Number.isFinite(v) ? v : null;
}

/** A unit's size in the SI unit of what it measures, worked out from the sheet when it rests on constants. */
export function unitFactor(u: UnitRow, values: ReadonlyMap<string, number>): number | null {
  if (u.factor) return u.factor.v;
  return u.factor_from ? evalFormula(u.factor_from, values) : null;
}

/** The SI unit each measure is counted in: the unit of factor one with no offset, SI's own before working ones. */
export function siUnits(units: readonly UnitRow[]): Map<string, UnitRow> {
  const out = new Map<string, UnitRow>();
  const rank = (g: string) => (g === "si-base" ? 0 : g === "si-named" ? 1 : 2);
  for (const u of [...units].sort((a, b) => rank(a.group) - rank(b.group))) {
    if (u.factor?.v === 1 && !u.offset && !out.has(u.measures)) out.set(u.measures, u);
  }
  return out;
}

/** A measure's id as words: "magnetic_field" as "magnetic field". */
export const measureWords = (m: string): string => m.replace(/_/g, " ");

/** The two numbers at the top of the Constants page: a change of units cannot move them, a change of physics does. */
export function dimensionless(values: ReadonlyMap<string, number>): { id: string; name: string; v: number; shown: string }[] {
  const out: { id: string; name: string; v: number; shown: string }[] = [];
  const alpha = values.get("alpha");
  if (alpha) out.push({ id: "alpha", name: "Fine-structure constant", v: alpha, shown: `1 / ${(1 / alpha).toFixed(3)}` });
  const G = values.get("G");
  const mp = values.get("m_p");
  const hbar = values.get("hbar");
  const c = values.get("c");
  if (G && mp && hbar && c) {
    const v = (G * mp * mp) / (hbar * c);
    out.push({ id: "alpha_G", name: "Gravitational coupling, G mₚ² / ħc", v, shown: formatValue(v, false) });
  }
  return out;
}
