// One units engine (ED1a, editing design section 3C): every figure on every screen is said by `say`, and every
// typed figure is read back by `readFigure`, from one table, figures-said.json ("How figures are said"). The kernel
// holds SI; this is display only, so nothing here changes a hash. Every unit factor comes from the master data
// sheet's Units table (the generated copy in ../generated/reference-constants.json): none is typed here. A rung
// whose unit the sheet does not hold yet is skipped, and `waitingUnits()` lists them.
//
// Pure and deterministic: no locale, no clock, the same value reads the same on every device.
import doc from "../generated/reference-constants.json" with { type: "json" };
import table from "./figures-said.json" with { type: "json" };
import type { Depth } from "./depth.ts";

// ---------------------------------------------------------------------------------------------------- the table

interface UnitRow {
  id: string;
  symbol: string;
  name: string;
  measures: string;
  factor: number;
  offset?: number;
}

export interface Rung {
  unit: string;
  /** Where this rung takes over, in its own unit. The first rung has none. */
  from?: number;
  /** Write "thousand", "million" above the table's plain_below (exponents at Engine depth). */
  words?: boolean;
  /** SI prefixes this unit is written with, by symbol. */
  prefixes?: string[];
  /** An exponent once the number leaves the comfortable range. */
  exponent?: boolean;
  /** An angle as degrees, minutes and seconds. */
  dms?: boolean;
}

export interface Reading {
  id: string;
  family: string;
  rungs: Rung[];
}

interface KindRow {
  measures: string;
  aliases?: Record<string, string>;
  say: Record<string, Reading[]>;
}

interface UnitLook {
  symbol?: string;
  word?: string;
  word_one?: string;
  tight?: boolean;
}

interface Table {
  comfort: [number, number];
  plain_below: number;
  stickiness: number;
  words: { word: string; exp: number }[];
  prefixes: { symbol: string; name: string; exp: number }[];
  prefixable: string[];
  gram: { unit: string; symbol: string; name: string; exp: number };
  families: { id: string; name: string }[];
  classes: { id: string; name: string }[];
  units: Record<string, UnitLook>;
  kinds: Record<string, KindRow>;
}

const T = table as unknown as Table;
const ROWS = new Map<string, UnitRow>((doc as unknown as { unit_rows: UnitRow[] }).unit_rows.map((u) => [u.id, u]));

/** The kinds of quantity the engine says: length, mass, time, temperature and the rest. */
export type FigureKind = string;
export const FIGURE_KINDS: readonly string[] = Object.keys(T.kinds);
/** The scale classes a unit choice is remembered for (star, planet, moon, construct...). */
export const SCALE_CLASSES: readonly { id: string; name: string }[] = T.classes;
export const UNIT_FAMILIES: readonly { id: string; name: string }[] = T.families;

/** What a figure is about: its scale class and, where it matters, which figure of it ("orbit", "size", "mass"). */
export interface About {
  class?: string;
  what?: string;
}

function kindRow(kind: FigureKind): KindRow {
  const k = T.kinds[kind];
  if (!k) throw new Error(`the units engine has no kind "${kind}"`);
  return k;
}

const has = (unit: string): boolean => ROWS.has(unit);

/** The table's key that answers for this kind and about: "planet.orbit", then "*.orbit", then "planet", then "default". */
function sayKey(k: KindRow, about: About | undefined): string {
  const c = about?.class;
  const w = about?.what;
  for (const key of [c && w ? `${c}.${w}` : "", w ? `*.${w}` : "", c ?? ""]) if (key && k.say[key]) return key;
  return "default";
}

/** The readings a click steps through for this kind and about, in click order, leaving out any with no unit on the sheet. */
export function readingsFor(kind: FigureKind, about?: About): Reading[] {
  const k = kindRow(kind);
  return (k.say[sayKey(k, about)] ?? [])
    .map((r) => ({ ...r, rungs: r.rungs.filter((g) => has(g.unit)) }))
    .filter((r) => r.rungs.length > 0);
}

/** The reading after `current` in click order, round again at the end. */
export function nextReading(kind: FigureKind, about: About | undefined, current: string): string {
  const rs = readingsFor(kind, about);
  const i = rs.findIndex((r) => r.id === current);
  return rs[(i + 1) % rs.length]!.id;
}

/** Every unit the table names that the master data sheet does not hold yet (those rungs are skipped). */
export function waitingUnits(): string[] {
  const out = new Set<string>();
  for (const k of Object.values(T.kinds)) {
    for (const rs of Object.values(k.say)) for (const r of rs) for (const g of r.rungs) if (!has(g.unit)) out.add(g.unit);
    for (const id of Object.values(k.aliases ?? {})) if (!has(id)) out.add(id);
  }
  return [...out].sort();
}

// ---------------------------------------------------------------------------------- remembered choices (per class)

/** Which reading the user last chose, per kind and what it is about ("length|planet.orbit": "si"). Saved with the campaign. */
export type UnitChoices = Readonly<Record<string, string>>;

export const choiceKey = (kind: FigureKind, about?: About): string =>
  `${kind}|${about?.class ?? "*"}${about?.what ? `.${about.what}` : ""}`;

/** The reading to use: the remembered choice if it still exists, otherwise the first. */
export function chosenReading(choices: UnitChoices | undefined, kind: FigureKind, about?: About): string {
  const rs = readingsFor(kind, about);
  const want = choices?.[choiceKey(kind, about)];
  return (rs.find((r) => r.id === want) ?? rs[0])?.id ?? "";
}

/** Choices with this one remembered; `null` puts the scheme's default back ("Use the scheme's"). */
export function withChoice(choices: UnitChoices | undefined, kind: FigureKind, about: About | undefined, reading: string | null): UnitChoices {
  const out: Record<string, string> = { ...(choices ?? {}) };
  const key = choiceKey(kind, about);
  if (reading === null || reading === readingsFor(kind, about)[0]?.id) delete out[key];
  else out[key] = reading;
  return out;
}

// ------------------------------------------------------------------------------------------- a card's facts

/** A card fact as the units engine reads it: a number in a unit the sheet holds. */
export interface FactLike {
  field: string;
  value: unknown;
  unit?: string;
  source?: string;
  digits?: number;
  /** The scene's own text: a typed or catalogued value's figures are counted from it ("500,000 km" has one). */
  display?: string;
}

/** A fact's unit as the sheet holds it: its kind, and how to take it to SI. */
export interface FactUnit {
  kind: FigureKind;
  unit: string;
  factor: number;
  offset: number;
}

const FACT_ABOUT = ((T as unknown as { fact_about?: { field: string; what: string }[] }).fact_about ?? []).map((r) => ({ re: new RegExp(r.field), what: r.what }));

/** What a card's fact is about, from its field (the table's `fact_about`): a_au is an orbit, radius_earth a size. */
export function factAbout(field: string, cls: string): About {
  const what = FACT_ABOUT.find((r) => r.re.test(field))?.what;
  return { class: cls, ...(what ? { what } : {}) };
}

/**
 * The sheet's unit for a symbol a scene writes ("AU", "M⊕", "K", "g", "μT"), with the kind it measures; null for a
 * unit the sheet does not hold ("dex") or none. A symbol that is a unit's own (g is the gee, d the day) wins over a
 * prefixed or aliased reading of it (g as a gram, d as degrees).
 */
export function factUnit(symbol: string | undefined): FactUnit | null {
  const s = symbol?.trim().replace(/μ/g, "µ");
  if (!s) return null;
  const of = (kind: FigureKind, r: Resolved): FactUnit => ({ kind, unit: r.id, factor: r.factor, offset: r.offset });
  for (const kind of FIGURE_KINDS) {
    const k = kindRow(kind);
    const u = [...ROWS.values()].find((x) => x.measures === k.measures && (x.symbol === s || look(x.id).symbol === s));
    if (u) return of(kind, { factor: u.factor, offset: u.offset ?? 0, id: u.id });
  }
  for (const kind of FIGURE_KINDS) {
    const r = resolveIn(kindRow(kind), s);
    if (r) return of(kind, r);
  }
  return null;
}

/** Strip float noise from a stored value before counting its figures (0.10700000000000001 is 0.107). */
const clean = (v: number): string => String(Number(v.toPrecision(12)));

/**
 * A card's fact said by the units engine, or null when it is not a number in a unit the sheet holds (a name, a dex,
 * a ratio): the caller then shows the scene's own text. `cls` is the body's scale class; `choices` the remembered
 * readings. A value someone set or a catalogue gave keeps its own figures (up to six); a computed one shows three.
 */
export function sayFact(f: FactLike, cls: string, opts: SayOptions & { choices?: UnitChoices } = {}): (Said & { kind: FigureKind; about: About }) | null {
  if (typeof f.value !== "number" || !Number.isFinite(f.value)) return null;
  const u = factUnit(f.unit);
  if (!u) return null;
  const about = factAbout(f.field, cls);
  const si = f.value * u.factor + u.offset;
  const kept = (f.source === "authored" || f.source === "catalogued") && opts.depth !== "curious";
  const { choices, ...rest } = opts;
  const said = say(si, u.kind, about, {
    ...rest,
    reading: opts.reading ?? chosenReading(choices, u.kind, about),
    ...(kept ? { source: "catalogued" as const, sig: Math.min(6, f.digits ?? significantFigures(/[-+−]?[\d,]*\.?\d+/.exec(f.display ?? "")?.[0] ?? clean(f.value))) } : {}),
  });
  return { ...said, kind: u.kind, about };
}

/**
 * Typed text for a fact read back in the fact's own unit (what an edit sends): any unit of its kind, words and
 * prefixes included ("412 thousand km" for an orbit held in AU). A bare number is taken in `shownUnit` (the unit on
 * screen), else the fact's.
 */
export function readFact(f: FactLike, text: string, shownUnit?: string): { ok: true; value: number; sig: number } | { ok: false; why: string } | null {
  const u = factUnit(f.unit);
  if (!u) return null;
  const r = readFigure(text, u.kind, { unit: shownUnit ?? u.unit });
  if (!r.ok) return { ok: false, why: r.why };
  return { ok: true, value: (r.si - u.offset) / u.factor, sig: r.sig };
}

// --------------------------------------------------------------------------------------------- numbers as text

const SUP: Record<string, string> = { "0": "⁰", "1": "¹", "2": "²", "3": "³", "4": "⁴", "5": "⁵", "6": "⁶", "7": "⁷", "8": "⁸", "9": "⁹", "-": "⁻", "+": "⁺" };
const UNSUP: Record<string, string> = Object.fromEntries(Object.entries(SUP).map(([a, b]) => [b, a]));
const sup = (s: string): string => s.replace(/./g, (c) => SUP[c] ?? c);

function group(int: string): string {
  return int.length <= 3 ? int : int.replace(/\B(?=(\d{3})+(?!\d))/g, ",");
}

/** Digits and exponent of |x| to `sig` significant figures: "1234", 2 means 1.234 × 10². */
function digitsOf(x: number, sig: number): { digits: string; exp: number } {
  const [m, e] = Math.abs(x).toExponential(Math.max(0, sig - 1)).split("e");
  return { digits: m!.replace(".", ""), exp: Number(e) };
}

/** |x| written out to `sig` significant figures as a plain decimal, grouped by thousands. */
function plain(x: number, sig: number, keepZeros: boolean): string {
  if (x === 0) return "0";
  const { digits, exp } = digitsOf(x, sig);
  let int: string;
  let frac: string;
  if (exp >= 0) {
    const d = digits.padEnd(exp + 1, "0");
    int = d.slice(0, exp + 1);
    frac = d.slice(exp + 1);
  } else {
    int = "0";
    frac = "0".repeat(-exp - 1) + digits;
  }
  if (!keepZeros) frac = frac.replace(/0+$/, "");
  return `${x < 0 ? "−" : ""}${group(int)}${frac ? `.${frac}` : ""}`;
}

function exponential(x: number, sig: number, keepZeros: boolean): string {
  const { digits, exp } = digitsOf(x, sig);
  let frac = digits.slice(1);
  if (!keepZeros) frac = frac.replace(/0+$/, "");
  return `${x < 0 ? "−" : ""}${digits[0]}${frac ? `.${frac}` : ""} × 10${sup(String(exp))}`;
}

/** How many significant figures a typed number carries ("0.01671022" is 7, "149.6" is 4, "1,200" is 2). */
export function significantFigures(typed: string): number {
  const m = typed.replace(/[,\s]/g, "").replace(/^[+−-]/, "").match(/^(\d*)(?:\.(\d*))?/);
  if (!m) return 3;
  const int = m[1] ?? "";
  const frac = m[2];
  const all = (int + (frac ?? "")).replace(/^0+/, "");
  if (frac === undefined) return Math.max(1, all.replace(/0+$/, "").length);
  return Math.max(1, all.length);
}

// --------------------------------------------------------------------------------------------------------- say

/** Which rung of which reading said a figure: "scientific/1". Held through a drag so the figure does not jump. */
export type RungRef = string;

export interface SayOptions {
  depth?: Depth;
  /** The reading to use (the remembered choice); the first when absent. */
  reading?: string;
  /** The rung the figure was last said on: boundaries are a little sticky, so a figure does not flicker. */
  prev?: RungRef;
  /** Keep this rung whatever the value (during one drag). */
  hold?: RungRef;
  /** Where the value came from: computed shows three figures; catalogued and typed what their source gave. */
  source?: "computed" | "catalogued" | "typed";
  /** Significant figures for a catalogued or typed value (see `significantFigures`). */
  sig?: number;
}

export interface Said {
  /** The whole figure as one string: "about 384 thousand km". */
  text: string;
  /** The number part: "384", "3.2 × 10¹⁹". */
  number: string;
  /** The unit part, which a click steps on: "thousand km", "Earth masses", "µbar". */
  unit: string;
  approx: boolean;
  reading: string;
  family: string;
  rung: RungRef;
  /** The sheet's id for the rung's unit ("au", "degc"): a bare number typed over this figure is read in it. */
  unitId: string;
}

function look(id: string): UnitLook {
  return T.units[id] ?? {};
}

function unitLabel(id: string, depth: Depth, one: boolean, prefix: string): string {
  const row = ROWS.get(id)!;
  const l = look(id);
  if (depth === "curious" && !prefix && l.word) return one && l.word_one ? l.word_one : l.word;
  return prefix + (l.symbol ?? row.symbol);
}

function pickRung(rs: Rung[], si: number, readingId: string, opts: SayOptions): number {
  const inUnit = (g: Rung): number => Math.abs((si - (ROWS.get(g.unit)!.offset ?? 0)) / ROWS.get(g.unit)!.factor);
  const hold = opts.hold?.split("/");
  if (hold && hold[0] === readingId) {
    const i = Number(hold[1]);
    if (i >= 0 && i < rs.length) return i;
  }
  let i = 0;
  for (let j = 1; j < rs.length; j++) if (inUnit(rs[j]!) >= (rs[j]!.from ?? 0)) i = j;
  const prev = opts.prev?.split("/");
  if (prev && prev[0] === readingId) {
    const p = Number(prev[1]);
    const s = T.stickiness;
    if (p === i - 1 && inUnit(rs[i]!) < (rs[i]!.from ?? 0) * (1 + s)) return p;
    if (p === i + 1 && p < rs.length && inUnit(rs[p]!) >= (rs[p]!.from ?? 0) * (1 - s)) return p;
  }
  return i;
}

function sayDms(deg: number, sig: number): string {
  const a = Math.abs(deg);
  let d = Math.floor(a);
  let m = Math.floor((a - d) * 60);
  const secDigits = Math.max(0, sig - 5);
  let s = Number(((a - d - m / 60) * 3600).toFixed(secDigits));
  if (s >= 60) {
    s -= 60;
    m += 1;
  }
  if (m >= 60) {
    m -= 60;
    d += 1;
  }
  return `${deg < 0 ? "−" : ""}${d}° ${m}′ ${s.toFixed(secDigits)}″`;
}

/**
 * A value in SI as the words every screen shows. `kind` is the quantity ("length", "mass", "time"...), `about` what
 * it is about ({ class: "moon", what: "orbit" }), which picks the established unit: AU for a planet's orbit, thousand
 * km for a moon's, Earth masses for a planet, Sun masses for a star.
 */
export function say(si: number, kind: FigureKind, about?: About, opts: SayOptions = {}): Said {
  const depth = opts.depth ?? "science";
  const rs = readingsFor(kind, about);
  const reading = rs.find((r) => r.id === opts.reading) ?? rs[0];
  if (!reading) throw new Error(`the units engine has no unit on the sheet for ${kind}`);
  const source = opts.source ?? "computed";
  const keep = source !== "computed";
  const sig = Math.max(1, opts.sig ?? 3);
  const approx = depth === "curious" && source === "computed";
  if (!Number.isFinite(si)) {
    return { text: String(si), number: String(si), unit: "", approx: false, reading: reading.id, family: reading.family, rung: `${reading.id}/0`, unitId: reading.rungs[0]!.unit };
  }
  const ri = pickRung(reading.rungs, si, reading.id, opts);
  const rung = reading.rungs[ri]!;
  const row = ROWS.get(rung.unit)!;
  const x = (si - (row.offset ?? 0)) / row.factor;
  const ax = Math.abs(x);
  const lo = T.comfort[0];
  let mult = "";
  let prefix = "";
  let exp10 = 0;
  if (rung.prefixes?.length && x !== 0) {
    const ps = T.prefixes.filter((p) => rung.prefixes!.includes(p.symbol));
    let best = { symbol: "", exp: 0 };
    if (ax >= 1) {
      for (const p of ps) if (p.exp > best.exp && ax / 10 ** p.exp >= 1) best = p;
    } else if (ax < lo) {
      const neg = ps.filter((p) => p.exp < 0).sort((a, b) => b.exp - a.exp);
      best = neg.find((p) => ax / 10 ** p.exp >= 1) ?? neg[neg.length - 1] ?? best;
    }
    prefix = best.symbol;
    exp10 = best.exp;
  } else if (rung.words && depth !== "engine" && ax >= T.plain_below) {
    let best: { word: string; exp: number } | undefined;
    for (const w of T.words) if (ax / 10 ** w.exp >= 1) best = w;
    if (best) {
      mult = best.word;
      exp10 = best.exp;
    }
  }
  let y = x / 10 ** exp10;
  const ay = Math.abs(y);
  const useExp =
    y !== 0 &&
    !rung.dms &&
    ((rung.exponent && (ay >= 1e4 || ay < lo / 100)) || (rung.words && depth === "engine" && ay >= T.plain_below) || ay >= 1e15 || ay < 1e-4);
  // An exponent is written on the plain unit: "3.83 × 10²⁶ W", never "3.83 × 10¹¹ PW".
  if (useExp) {
    y = x;
    mult = "";
    prefix = "";
  }
  let number: string;
  if (rung.dms) number = sayDms(y, keep ? sig : 6);
  else if (useExp) number = exponential(y, sig, keep);
  else number = plain(y, sig, keep);
  const one = number === "1" && !mult;
  const label = rung.dms ? "" : unitLabel(rung.unit, depth, one, prefix);
  const unit = [mult, label].filter(Boolean).join(" ");
  const tight = look(rung.unit).tight && !mult;
  const body = rung.dms ? number : unit ? `${number}${tight ? "" : " "}${unit}` : number;
  return { text: `${approx ? "about " : ""}${body}`, number, unit, approx, reading: reading.id, family: reading.family, rung: `${reading.id}/${ri}`, unitId: rung.unit };
}

/** `say(...).text`. */
export const sayText = (si: number, kind: FigureKind, about?: About, opts?: SayOptions): string => say(si, kind, about, opts).text;

// ------------------------------------------------------------------------------------------------------- read back

export type FigureRead =
  | { ok: true; si: number; unit: string; sig: number; terms: number }
  | { ok: false; why: string; as?: { kind: FigureKind; si: number } };

function normalise(text: string): string {
  return text
    .trim()
    .replace(/^(about|roughly|~|≈)\s*/i, "")
    .replace(/[−–]/g, "-")
    .replace(/10([⁻⁺]?[⁰¹²³⁴⁵⁶⁷⁸⁹]+)/g, (_, e: string) => `10^${e.replace(/./g, (c) => UNSUP[c] ?? c)}`)
    .replace(/(\d)\s*[×x*·]\s*10\s*\^\s*([+-]?\d+)/g, "$1e$2")
    .replace(/(^|[^\d.])10\s*\^\s*([+-]?\d+)/g, "$11e$2")
    .replace(/\s+/g, " ");
}

const NUM = /[+-]?(?:\d{1,3}(?:,\d{3})+|\d+)(?:\.\d*)?(?:[eE][+-]?\d+)?|[+-]?\.\d+(?:[eE][+-]?\d+)?/y;

interface Resolved {
  factor: number;
  offset: number;
  id: string;
}

function resolveIn(k: KindRow, s: string): Resolved | undefined {
  const rows = [...ROWS.values()].filter((u) => u.measures === k.measures);
  const as = (u: UnitRow, f = 1): Resolved => ({ factor: u.factor * f, offset: u.offset ?? 0, id: u.id });
  const exact = (q: string, ci: boolean): UnitRow | undefined => {
    const eq = (a: string | undefined): boolean => !!a && (ci ? a.toLowerCase() === q.toLowerCase() : a === q);
    const alias = Object.entries(k.aliases ?? {}).find(([a]) => eq(a))?.[1];
    if (alias && ROWS.get(alias)?.measures === k.measures) return ROWS.get(alias);
    return rows.find((u) => {
      const l = look(u.id);
      return eq(u.id) || eq(u.symbol) || eq(u.name) || eq(`${u.name}s`) || eq(l.symbol) || eq(l.word) || eq(l.word_one);
    });
  };
  const hit = exact(s, false) ?? (s.length > 2 ? exact(s, true) : undefined);
  if (hit) return as(hit);
  const g = T.gram;
  const kg = ROWS.get(g.unit);
  if (kg?.measures === k.measures && (s === g.symbol || s === g.name || s === `${g.name}s`)) return as(kg, 10 ** g.exp);
  for (const p of [...T.prefixes].sort((a, b) => b.symbol.length - a.symbol.length)) {
    for (const [pre, ci] of [[p.symbol, false], [p.name, true]] as const) {
      const lower = ci ? s.toLowerCase() : s;
      if (!lower.startsWith(ci ? pre.toLowerCase() : pre) || lower.length === pre.length) continue;
      const rest = s.slice(pre.length);
      if (kg?.measures === k.measures && (rest === g.symbol || (ci && (rest === g.name || rest === `${g.name}s`)))) {
        return as(kg, 10 ** (p.exp + g.exp));
      }
      const base = rows.find((u) => T.prefixable.includes(u.id) && (ci ? rest === u.name || rest === `${u.name}s` : rest === (look(u.id).symbol ?? u.symbol)));
      if (base) return as(base, 10 ** p.exp);
    }
  }
  return undefined;
}

function readIn(kind: FigureKind, src: string, defaultUnit: string | undefined): FigureRead {
  const k = kindRow(kind);
  let pos = 0;
  let si = 0;
  let terms = 0;
  let sig = 0;
  let first = "";
  let offsetUsed = false;
  while (pos < src.length) {
    while (src[pos] === " ") pos++;
    if (pos >= src.length) break;
    NUM.lastIndex = pos;
    const m = NUM.exec(src);
    if (!m) return { ok: false, why: `"${src.slice(pos)}" is not a number` };
    pos = NUM.lastIndex;
    const numText = m[0];
    let n = Number(numText.replace(/,/g, ""));
    const next = src.slice(pos).search(/[+-]?(?:\d|\.\d)/);
    const end = next < 0 ? src.length : pos + next;
    let unitText = src.slice(pos, end).trim();
    pos = end;
    const w = T.words.find((x) => unitText === x.word || unitText.startsWith(`${x.word} `));
    if (w) {
      n *= 10 ** w.exp;
      unitText = unitText.slice(w.word.length).trim();
    }
    const u = unitText ? resolveIn(k, unitText) : defaultUnit && ROWS.has(defaultUnit) ? { factor: ROWS.get(defaultUnit)!.factor, offset: ROWS.get(defaultUnit)!.offset ?? 0, id: defaultUnit } : undefined;
    if (!u) return { ok: false, why: unitText ? `"${unitText}" is not a unit of ${kind}` : `a ${kind} needs a unit` };
    if (u.offset !== 0) offsetUsed = true;
    si += n * u.factor + u.offset;
    terms++;
    sig = Math.max(sig, significantFigures(numText));
    if (!first) first = u.id;
  }
  if (terms === 0) return { ok: false, why: "nothing to read" };
  if (offsetUsed && terms > 1) return { ok: false, why: "a temperature is one figure" };
  return { ok: true, si, unit: first, sig, terms };
}

/**
 * A typed figure read back into SI, exactly as typed: "149.6 million km", "1.2 Gm", "3.2 × 10¹⁹ kg", "23d 26m",
 * "20 °C", "0.005 AU". A bare number takes `unit` (the unit on screen). A figure of another kind ("300 d" typed for an
 * orbit) is refused with `as` set, so the caller can turn a period into a distance by physics.
 */
export function readFigure(text: string, kind: FigureKind, opts: { unit?: string } = {}): FigureRead {
  const src = normalise(text);
  const got = readIn(kind, src, opts.unit);
  if (got.ok) return got;
  for (const other of FIGURE_KINDS) {
    if (other === kind || kindRow(other).measures === kindRow(kind).measures) continue;
    const r = readIn(other, src, undefined);
    if (r.ok) return { ok: false, why: `that is a ${other}, not a ${kind}`, as: { kind: other, si: r.si } };
  }
  return got;
}
