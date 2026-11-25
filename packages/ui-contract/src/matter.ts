// The Reference screen's reading of the master data sheet (matter-pack design section 8, MP10): everything comes
// from the kernel's `read_matter`, so a pack's or a campaign's edit shows the moment it is loaded. Nothing here is
// typed: values, units, sources, marks and group wording are the sheet's. This module only words and lays them out.
import type { Json } from "./json.ts";
import { roundTo, drawn } from "./figures.ts";

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
  if (a >= 1e-3 && a < 1e5) return group3(full ? String(v) : String(roundTo(v, 4)));
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

/** One origin in the sheet's one legend (big_bang ... human_made), in its three wordings. */
export interface OriginKind {
  id: string;
  physics: string;
  plain: string;
  story: string;
}

/** One element row as the Elements page reads it. */
export interface ElementRow {
  id: string;
  symbol: string;
  name: string;
  z: number;
  atomic_weight?: SheetValue;
  no_standard_weight?: boolean;
  abundance_protosolar?: SheetValue & { sigma?: number };
  origin?: { shares: Record<string, number>; src?: string; also?: string[]; check?: string; note?: string; argued?: string };
  origin_story?: { physics?: string; plain?: string; story?: string };
  /** A pack's honesty mark on the row, when it adds or changes an element (authored, fictional, hypothetical). */
  mark?: string;
  fictional?: boolean;
  hypothetical?: boolean;
}

export interface ElementsPage {
  elements: ElementRow[];
  legend: OriginKind[];
}

/** The legend's reading order: the oldest process first, people last. */
const ORIGIN_ORDER = ["big_bang", "cosmic_rays", "low_mass_stars", "massive_stars", "white_dwarfs", "neutron_star_mergers", "human_made"];

/** Read the Elements page: the listing (for its ids and the origin legend), then each element's row. */
export async function readElementsPage(k: MatterReader): Promise<{ ok: true; page: ElementsPage } | { ok: false; message: string }> {
  const l = await k.readMatter({});
  if (!l.ok) return { ok: false, message: l.message };
  const list = l.value as unknown as { elements?: string[]; origin_legend?: OriginKind[] | Record<string, OriginKind> };
  const rows = await Promise.all(
    (list.elements ?? []).map(async (id) => {
      const r = await k.readMatter({ table: "elements", id });
      return r.ok ? ((r.value as unknown as RowDetail<ElementRow>).row ?? null) : null;
    }),
  );
  const elements = rows.filter((r): r is ElementRow => r !== null).sort((a, b) => a.z - b.z);
  // The kernel keys the legend by id; the page reads it oldest process first, as the sheet lists it, then any a pack adds.
  const raw = Array.isArray(list.origin_legend) ? list.origin_legend : Object.values(list.origin_legend ?? {});
  const at = (id: string) => (ORIGIN_ORDER.includes(id) ? ORIGIN_ORDER.indexOf(id) : ORIGIN_ORDER.length);
  const legend = [...raw].sort((a, b) => at(a.id) - at(b.id) || a.id.localeCompare(b.id));
  return { ok: true, page: { elements, legend } };
}

/**
 * Where an element sits in the usual 18-column table: [row, column], rows 1 to 7 for the periods, 9 and 10 for the
 * lanthanides and actinides set below, null past 118 (those go in the campaign row). Layout only.
 */
export function tablePosition(z: number): [number, number] | null {
  if (!Number.isInteger(z) || z < 1 || z > 118) return null;
  if (z === 1) return [1, 1];
  if (z === 2) return [1, 18];
  if (z <= 4) return [2, z - 2];
  if (z <= 10) return [2, z + 8];
  if (z <= 12) return [3, z - 10];
  if (z <= 18) return [3, z];
  if (z <= 36) return [4, z - 18];
  if (z <= 54) return [5, z - 36];
  if (z <= 56) return [6, z - 54];
  if (z <= 71) return [9, z - 54];
  if (z <= 86) return [6, z - 68];
  if (z <= 88) return [7, z - 86];
  if (z <= 103) return [10, z - 86];
  return [7, z - 100];
}

/** The theme token that colours an origin: one legend for the whole app. */
export const originToken = (id: string): string => `--sse-origin-${id.replace(/_/g, "-")}`;

/** An element's shares in the legend's order, largest first within it, each with its fraction. */
export function originShares(e: Pick<ElementRow, "origin">, legend: readonly OriginKind[]): { kind: OriginKind; share: number }[] {
  const shares = e.origin?.shares ?? {};
  const known = legend.filter((k) => (shares[k.id] ?? 0) > 0).map((kind) => ({ kind, share: shares[kind.id]! }));
  // A share whose origin the legend lacks still shows, in the faint ink, so nothing is hidden.
  for (const [id, share] of Object.entries(shares)) if (share > 0 && !legend.some((k) => k.id === id)) known.push({ kind: { id, physics: id, plain: id.replace(/_/g, " "), story: id }, share });
  return known.sort((a, b) => b.share - a.share);
}

/** A CSS gradient that fills a bar in proportion to the shares, in the legend's colours. */
export function originGradient(parts: readonly { kind: OriginKind; share: number }[]): string {
  const total = parts.reduce((s, p) => s + p.share, 0) || 1;
  let at = 0;
  const stops: string[] = [];
  for (const p of parts) {
    const c = `var(${originToken(p.kind.id)}, var(--sse-ink-faint))`;
    const to = at + (p.share / total) * 100;
    stops.push(`${c} ${drawn(at, 2)}%`, `${c} ${drawn(to, 2)}%`);
    at = to;
  }
  return stops.length ? `linear-gradient(90deg, ${stops.join(", ")})` : "var(--sse-ink-faint)";
}

/** "Merging neutron stars, about 94 parts in 100": a share in words. */
export function shareWords(p: { kind: OriginKind; share: number }, first: boolean): string {
  const n = Math.round(p.share * 100);
  const who = p.kind.plain.charAt(0).toUpperCase() + p.kind.plain.slice(1);
  return n >= 100 ? `${who}, all of it` : first ? `${who}, about ${n} parts in 100` : `${who}, ${n}`;
}
