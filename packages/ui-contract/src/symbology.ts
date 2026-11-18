// Symbology (H8.4A, build plan V12): how facts become marks, separate from
// the theme. A symbology document binds facts (object kind, class, tags,
// numbers) to visual channels (shape, fill pattern, colour, size, outline,
// ring, glyph). Two base grammars ship (diagrammatic, colour-coded); others
// inherit from one or mix them channel by channel. A theme says what it can
// carry, and a stated rule collapses what it cannot (a one-colour theme turns
// colours into patterns). Views ask "mark for this object" and draw what
// comes back; they never decide what a class looks like. Presentation only:
// nothing here reaches the kernel or changes the state hash.

export type Channel = "shape" | "fill_pattern" | "colour" | "size" | "outline" | "ring" | "glyph" | "tone";
export const CHANNELS: readonly Channel[] = ["shape", "fill_pattern", "colour", "size", "outline", "ring", "glyph", "tone"];

export type Shape = "disc" | "star" | "diamond" | "square" | "triangle";
/** Shapes a planet-like body can take (the star shape is kept for stars). */
export const BODY_SHAPES: readonly Shape[] = ["disc", "square", "diamond", "triangle"];
export type Pattern = "solid" | "hollow" | "hatch" | "stipple" | "band" | "cross" | "grid" | "dash";
/** Patterns in the order a collapsed colour channel takes them (the stated rule). */
export const PATTERNS: readonly Pattern[] = ["solid", "hatch", "stipple", "band", "cross", "grid", "dash", "hollow"];

export interface Mark {
  shape: Shape;
  fill_pattern: Pattern;
  /** A colour (#RRGGBB) for the fill, or null to draw in the theme's ink. */
  colour: string | null;
  /** Radius in chart pixels before zoom. */
  size: number;
  /** Which theme ink draws the outline and patterns. */
  tone: "ink" | "muted" | "accent";
  outline: "none" | "thin" | "double";
  ring: boolean;
  /** A short mark drawn beside the body (a letter or symbol), or null. */
  glyph: string | null;
}

/** What a mark is chosen from: the object's kind, classes (C1 ids, base first), tags and numbers. */
export interface MarkSubject {
  kind: string;
  classes: readonly string[];
  tags: readonly string[];
  numbers?: Readonly<Record<string, number>>;
}

export type Scale =
  /** Keys are values of the source; a key ending in "*" matches a prefix ("planet/*"). */
  | { table: Record<string, string | number | boolean> }
  /** Ordered steps for a number: the first step whose `upto` is above the value. */
  | { thresholds: [number, string | number | boolean][] };

export interface Binding {
  /** Object kind (star, planet, moon, belt, station, ...) or "*". */
  applies_to: string;
  channel: Channel;
  /** "kind", "class" (the first of the object's classes the scale knows), "tag:<prefix>", or "number:<name>". */
  from: string;
  scale: Scale;
  fallback?: string | number | boolean | null;
  /** Only in force under a theme limit: "one-colour" bindings stand in for a colour the theme cannot carry. */
  when?: "one-colour";
}

export interface SymbologyDoc {
  schema: "sse.symbology/1";
  id: string;
  name: string;
  description?: string;
  inherits?: string;
  /** Channels taken whole from other symbologies (mixing is per channel). */
  mix?: Partial<Record<Channel, string>>;
  bindings: Binding[];
  /** "auto", or the legend's entries in hand order (source values). */
  legend?: "auto" | string[];
  /** True while a table holds stand-in values that must be replaced (for example V3's colours, not yet read). */
  placeholder?: boolean | string;
}

/** What a theme can carry (H8.4A), and its own palette for "palette:N" colours. */
export interface ThemeCapability {
  supports: { colours: number | "full"; patterns: boolean; shapes: boolean };
  palette?: readonly string[];
}

const BASE: Mark = { shape: "disc", fill_pattern: "solid", colour: null, size: 4, tone: "ink", outline: "none", ring: false, glyph: null };

/** The bindings a symbology uses after `inherits` and `mix`. Throws on a loop or a missing document. */
export function resolveSymbology(id: string, registry: readonly SymbologyDoc[], seen: string[] = []): Binding[] {
  if (seen.includes(id)) throw new Error(`symbology loop: ${[...seen, id].join(" > ")}`);
  const doc = registry.find((d) => d.id === id);
  if (!doc) throw new Error(`no symbology ${id}`);
  let out: Binding[] = doc.inherits ? resolveSymbology(doc.inherits, registry, [...seen, id]) : [];
  for (const [ch, src] of Object.entries(doc.mix ?? {})) {
    const from = resolveSymbology(src!, registry, [...seen, id]).filter((b) => b.channel === ch);
    out = [...out.filter((b) => b.channel !== ch), ...from];
  }
  for (const b of doc.bindings) out = [...out.filter((x) => !(x.channel === b.channel && x.applies_to === b.applies_to)), b];
  return out;
}

function lookup(scale: Scale, v: string | number | undefined): string | number | boolean | undefined {
  if (v === undefined) return undefined;
  if ("thresholds" in scale) {
    if (typeof v !== "number") return undefined;
    return scale.thresholds.find(([upto]) => v < upto)?.[1] ?? scale.thresholds[scale.thresholds.length - 1]?.[1];
  }
  const k = String(v);
  if (k in scale.table) return scale.table[k];
  const pre = Object.keys(scale.table)
    .filter((x) => x.endsWith("*") && k.startsWith(x.slice(0, -1)))
    .sort((a, b) => b.length - a.length)[0];
  return pre === undefined ? undefined : scale.table[pre];
}

/** The value a binding reads, and what its scale makes of it. */
function evaluate(b: Binding, s: MarkSubject): { key: string | undefined; out: string | number | boolean | undefined } {
  if (b.from === "class") {
    for (const c of s.classes) {
      const out = lookup(b.scale, c);
      if (out !== undefined) return { key: c, out };
    }
    return { key: undefined, out: undefined };
  }
  let v: string | number | undefined;
  if (b.from === "kind") v = s.kind;
  else if (b.from.startsWith("tag:")) {
    const prefix = b.from.slice(4);
    v = s.tags.find((t) => t.startsWith(prefix))?.slice(prefix.length);
  } else if (b.from.startsWith("number:")) v = s.numbers?.[b.from.slice(7)];
  return { key: v === undefined ? undefined : String(v), out: lookup(b.scale, v) };
}

export interface Marker {
  readonly id: string;
  readonly bindings: readonly Binding[];
  /** The mark for one object, already fitted to the theme. */
  mark(s: MarkSubject): Mark;
  /** The stated rule in force, in words, when the theme cannot carry a channel (null when it can). */
  readonly collapse: string | null;
}

/**
 * A marker for a symbology under a theme. The theme's capability rules:
 * - one colour: the colour channel collapses. If the symbology binds no fill
 *   pattern, each colour in the colour table takes the next pattern in
 *   PATTERNS order; otherwise the colours are dropped (patterns already say it).
 *   Bindings marked `when: "one-colour"` (class codes) come into force, so
 *   classes that share a pattern still differ.
 * - no patterns: patterns become solid; no shapes: shapes become discs.
 * - "palette:N" colours take the theme's palette (wrapping), else the ink.
 */
export function markerFor(id: string, registry: readonly SymbologyDoc[], theme: ThemeCapability): Marker {
  const oneColour = theme.supports.colours !== "full" && theme.supports.colours <= 1;
  const bindings = resolveSymbology(id, registry).filter((b) => b.when === undefined || (b.when === "one-colour" && oneColour));
  const patternBound = bindings.some((b) => b.channel === "fill_pattern");
  const colourOrder: string[] = [];
  for (const b of bindings) if (b.channel === "colour" && "table" in b.scale) for (const v of Object.values(b.scale.table)) if (typeof v === "string" && !colourOrder.includes(v)) colourOrder.push(v);
  const collapse = !oneColour ? null : patternBound ? "One-colour theme: colours are dropped; patterns and shapes carry the classes." : "One-colour theme: each colour becomes a pattern, in the legend's order (solid, hatch, stipple, band, cross, grid, dash, hollow).";
  const paint = (c: string): string | null => {
    const p = /^palette:(\d+)$/.exec(c);
    if (!p) return c;
    const pal = theme.palette ?? [];
    return pal.length ? pal[Number(p[1]) % pal.length]! : null;
  };
  return {
    id,
    bindings,
    collapse,
    mark(s) {
      const m: Record<string, unknown> = { ...BASE };
      for (const b of bindings) {
        if (b.applies_to !== "*" && b.applies_to !== s.kind) continue;
        const { out } = evaluate(b, s);
        const v = out !== undefined ? out : b.fallback;
        if (v !== undefined) m[b.channel] = v;
      }
      const mk = m as unknown as Mark;
      if (mk.colour) {
        if (oneColour) {
          if (!patternBound) {
            const i = colourOrder.indexOf(mk.colour);
            mk.fill_pattern = PATTERNS[(i < 0 ? 0 : i) % PATTERNS.length]!;
          }
          mk.colour = null;
        } else mk.colour = paint(mk.colour);
      }
      if (!theme.supports.patterns && mk.fill_pattern !== "hollow") mk.fill_pattern = "solid";
      if (!theme.supports.shapes) mk.shape = "disc";
      return mk;
    },
  };
}

export interface LegendEntry {
  /** What the entry means ("Ocean world", "Ringed"). */
  label: string;
  /** The source value (a class id, tag value or kind). */
  key: string;
  channel: Channel;
  mark: Mark;
}

/**
 * The legend for a marker, generated from its bindings: one entry per value
 * a binding draws differently, limited to `present` subjects when given, in
 * the symbology's hand order when it has one. Every entry's mark is the one
 * the chart draws for that value, so the legend cannot disagree with the marks.
 */
export function legendFor(marker: Marker, opts: { present?: readonly MarkSubject[]; names?: Readonly<Record<string, string>>; order?: readonly string[]; kind?: string } = {}): LegendEntry[] {
  const kind = opts.kind ?? "planet";
  const out: LegendEntry[] = [];
  const seen = new Set<string>();
  const add = (key: string, channel: Channel, subject: MarkSubject) => {
    const sig = `${channel}\u0000${key}`;
    if (seen.has(sig)) return;
    seen.add(sig);
    out.push({ key, channel, label: opts.names?.[key] ?? readableKey(key), mark: marker.mark(subject) });
  };
  for (const b of marker.bindings) {
    if (b.applies_to !== kind && b.applies_to !== "*") continue;
    // Sizes are magnitudes, not categories: the legend lists what kind a body is.
    if (b.channel === "size" || b.channel === "tone") continue;
    if (!(b.from === "class" || b.from.startsWith("tag:")) || !("table" in b.scale)) continue;
    const keys = opts.present
      ? opts.present.filter((s) => s.kind === kind).map((s) => evaluate(b, s).key).filter((k): k is string => k !== undefined && lookup(b.scale, k) !== undefined)
      : Object.keys(b.scale.table).filter((k) => !k.endsWith("*"));
    for (const k of keys) {
      const subject: MarkSubject = b.from === "class" ? { kind, classes: [k], tags: [] } : { kind, classes: [], tags: [`${b.from.slice(4)}${k}`] };
      add(k, b.channel, subject);
    }
  }
  // One entry per value: a class drawn by several channels is one line.
  const merged = new Map<string, LegendEntry>();
  for (const e of out) if (!merged.has(e.key)) merged.set(e.key, e);
  const list = [...merged.values()];
  const order = opts.order ?? [];
  return list.sort((a, b) => (order.indexOf(a.key) + 1 || 1e9) - (order.indexOf(b.key) + 1 || 1e9));
}

function readableKey(k: string): string {
  const s = k.replace(/^[a-z]+\//, "").replace(/-/g, " ");
  return s.charAt(0).toUpperCase() + s.slice(1);
}

/** Two marks look the same to the eye (used to prove a grammar distinguishes classes). */
export function sameLook(a: Mark, b: Mark): boolean {
  return a.shape === b.shape && a.fill_pattern === b.fill_pattern && a.colour === b.colour && a.ring === b.ring && a.glyph === b.glyph && a.outline === b.outline;
}

/** Why a symbology document is malformed, or null. */
export function checkSymbology(d: SymbologyDoc, registry: readonly SymbologyDoc[]): string | null {
  if (d.schema !== "sse.symbology/1") return "not an sse.symbology/1 document";
  if (!/^[a-z0-9][a-z0-9._-]*$/.test(d.id)) return "symbology id must be lower-case text";
  for (const b of d.bindings) {
    if (!CHANNELS.includes(b.channel)) return `unknown channel ${b.channel}`;
    if (b.when !== undefined && b.when !== "one-colour") return `binding condition ${String(b.when)} is not one-colour`;
    if (!(b.from === "kind" || b.from === "class" || b.from.startsWith("tag:") || b.from.startsWith("number:"))) return `binding source ${b.from} is not kind, class, tag: or number:`;
    if (b.channel === "colour" && "table" in b.scale) {
      for (const v of Object.values(b.scale.table)) if (typeof v !== "string" || !/^(#[0-9A-Fa-f]{6}|palette:\d+)$/.test(v)) return `colour ${String(v)} must be #RRGGBB or palette:N`;
    }
  }
  try {
    resolveSymbology(d.id, [...registry.filter((x) => x.id !== d.id), d]);
  } catch (e) {
    return (e as Error).message;
  }
  return null;
}

/**
 * What a GM shell offers views about symbols: the symbologies it knows, the
 * one in force under the current theme, and (for the symbols screen only)
 * a way to choose another or keep an edited copy. Presentation state: it
 * lives with the workspace on this device, never in the universe.
 */
export interface SymbologyHost {
  registry(): readonly SymbologyDoc[];
  chosen(): string;
  marker(): Marker;
  theme(): ThemeCapability;
  /** Plain names for class ids, from the classification pack. */
  names(): Readonly<Record<string, string>>;
  choose(id: string): void;
  /** Keep an edited symbology (replaces one with the same id); null removes every edited copy. */
  keep(doc: SymbologyDoc | null): void;
  /** Themes to preview the symbols under: a name, the theme's tokens as a style, and what it can carry. */
  previews?(): readonly { id: string; name: string; style: string; capability: ThemeCapability }[];
}
