// The Names page (NM6, naming design 4.12): which name schemes the stack carries and which one names each layer by
// default, read from the pack documents themselves, and a live sample from the kernel's `name_sample` (keyed draws,
// the same on every machine). Names never feed physics; nothing here writes anything.
import type { PackDoc } from "./physics.ts";

export interface SampleAsk {
  scheme: string;
  kind: string;
  layer?: string;
  count?: number;
  start?: number;
  seed?: string;
  parent?: string;
  facts?: Record<string, string>;
}

export interface NamePart {
  text: string;
  from: string;
  means?: string;
  source?: string;
}

export interface SampledName {
  scheme: string;
  form: string;
  follows: boolean;
  parts: NamePart[];
  /** How to say it ("KHAZ-rak"); empty when the scheme does not say. */
  say: string;
  ipa: string;
  /** The name in its own script, when it has one. */
  native?: string;
  /** Unusual sounds and how to make them ("q": "a k made far back in the throat"). */
  hints?: Record<string, string>;
  text: string;
}

export interface NameSample {
  scheme: string;
  kind: string;
  layer: string;
  count: number;
  digest: string;
  names: SampledName[];
  avoid_hits: unknown;
}

export interface SchemeInfo {
  id: string;
  label: string;
  /** catalogue, list, invented or real: what sort of scheme it is. */
  sort: string;
  about: string;
  script?: string;
  /** The pack it comes from (a later pack's scheme with the same id replaces an earlier one). */
  pack: string;
  /** The kinds of thing it has its own forms for. */
  kinds: string[];
  /** For a mix: the schemes it draws from, with their weights. */
  mix?: [string, number][];
}

export interface NamesOnStack {
  schemes: SchemeInfo[];
  /** Layer to scheme, as the stack's packs set them (a later pack wins), with the pack that set it. */
  defaults: { layer: string; scheme: string; pack: string }[];
}

type RawScheme = { id: string; label?: string; kind?: string; about?: string; script?: string; forms?: Record<string, unknown>; mix?: { of?: [string, number][] } };
const namesOf = (p: PackDoc) => ((p.provides as { other?: { names?: { schemes?: RawScheme[]; defaults?: Record<string, string> } } } | undefined)?.other?.names ?? null);

/** The schemes and defaults of a stack, bottom pack first, as the kernel gathers them (NM1, NM2). */
export function namesOnStack(packs: readonly PackDoc[]): NamesOnStack {
  const schemes = new Map<string, SchemeInfo>();
  const defaults = new Map<string, { scheme: string; pack: string }>();
  for (const p of packs) {
    const n = namesOf(p);
    if (!n) continue;
    for (const s of n.schemes ?? []) {
      schemes.delete(s.id);
      schemes.set(s.id, {
        id: s.id,
        label: s.label ?? s.id,
        sort: s.kind ?? (s.mix ? "mix" : "list"),
        about: s.about ?? "",
        ...(s.script ? { script: s.script } : {}),
        pack: p.id,
        kinds: Object.keys(s.forms ?? {}).filter((k) => k !== "default"),
        ...(s.mix?.of ? { mix: s.mix.of } : {}),
      });
    }
    for (const [layer, scheme] of Object.entries(n.defaults ?? {})) defaults.set(layer, { scheme, pack: p.id });
  }
  // A mix has the kinds of the schemes it draws from.
  for (const s of schemes.values()) if (s.mix && !s.kinds.length) s.kinds = [...new Set(s.mix.flatMap(([id]) => schemes.get(id)?.kinds ?? []))];
  return { schemes: [...schemes.values()], defaults: [...defaults].map(([layer, d]) => ({ layer, ...d })) };
}

/** The kinds a sample offers, in the order people reach for them, then the rest as the scheme lists them. */
export const SAMPLE_KINDS = ["planet", "moon", "star", "settlement", "peak", "sea", "crater", "continent"];
export function sampleKinds(s: SchemeInfo): string[] {
  const first = SAMPLE_KINDS.filter((k) => s.kinds.includes(k));
  return first.length ? first : ["planet"];
}

/** "Designation", "Common", "Local (vesh)" for a layer id. */
export function layerWords(layer: string): string {
  if (layer === "designation") return "Designation";
  if (layer === "common") return "Common";
  const local = /^local_(.+)$/.exec(layer);
  if (local) return `Local (${local[1]!.replace(/_/g, " ")})`;
  return layer[0]!.toUpperCase() + layer.slice(1).replace(/_/g, " ");
}

/** What sort of scheme, in a few words. */
export function sortWords(sort: string): string {
  return ({ catalogue: "catalogue designations", list: "from a word list", invented: "an invented language", real: "real languages", mix: "a mix" } as Record<string, string>)[sort] ?? sort;
}

/** The meanings of a drawn name's parts, joined: "dark + water". Empty when no part carries one. */
export function meaningWords(n: SampledName): string {
  return n.parts.flatMap((p) => (p.means ? [p.means] : [])).join(" + ");
}

/** True when the say-it line tells a reader something the name itself does not (not just the name split up). */
export function saysMore(n: SampledName): boolean {
  const flat = (x: string) => x.replace(/[\s-]/g, "").toLowerCase();
  return !!n.say && (flat(n.say) !== flat(n.text) || /[A-Z]{2}/.test(n.say));
}

/** One layer's name on a real thing (kernel `read_names`, NM2). */
export interface ShownName {
  layer: string;
  text: string;
  /** typed (a person wrote it; it always wins), fixed (drawn once and kept) or drawn (worked out when read). */
  source: "typed" | "fixed" | "drawn";
  scheme?: string;
  named?: Omit<SampledName, "text">;
  /** Under a typed name, the name it would have otherwise (GM only). */
  original?: string;
}

export interface NamesRead {
  address: string;
  kind: string;
  names: ShownName[];
}

/** Where a name came from, in words. */
export function sourceWords(n: ShownName, schemeLabel: (id: string) => string = (id) => id): string {
  if (n.source === "typed") return "typed";
  const from = n.scheme ? ` from ${schemeLabel(n.scheme)}` : "";
  return n.source === "fixed" ? `drawn${from}, kept` : `drawn${from}`;
}

/** Layers in the order people read them: designation, common, then the rest by id. */
export function layerOrder(a: { layer: string }, b: { layer: string }): number {
  const at = (l: string) => (l === "designation" ? 0 : l === "common" ? 1 : 2);
  return at(a.layer) - at(b.layer) || a.layer.localeCompare(b.layer);
}

/** The name layers a lens can label the chart with: the usual three, then any other the stack's defaults name. */
export function chartNameLayers(defaults: readonly { layer: string }[] = []): { id: string; label: string }[] {
  const out = [
    { id: "", label: "Common name, else designation" },
    { id: "designation", label: "Designation only" },
    { id: "local", label: "Local names" },
  ];
  for (const d of defaults) if (d.layer !== "common" && !out.some((o) => o.id === d.layer)) out.push({ id: d.layer, label: `${layerWords(d.layer)} names` });
  return out;
}

/**
 * Each subject's name in one layer, for a chart a lens labels by that layer (NM6 layer choice). Read through the
 * session's read_names, so a player seat gets only the layers its view discloses. A subject with no name in the
 * layer is left out (the chart keeps its usual label). Layer "" or "common" needs no reading: the scene already has
 * the common name, else the designation.
 */
export async function namesInLayer(session: { readNames(address: string): Promise<{ ok: true; value: NamesRead } | { ok: false; code: string; message: string }> }, addresses: readonly string[], layer: string): Promise<Map<string, string>> {
  const out = new Map<string, string>();
  if (!layer || layer === "common") return out;
  const read = await Promise.all(addresses.map((a) => session.readNames(a)));
  read.forEach((r, i) => {
    const n = r.ok ? r.value.names.find((x) => x.layer === layer) : undefined;
    if (n?.text) out.set(addresses[i]!, n.text);
  });
  return out;
}

/** A scene with its labels taken from `labels` where it has one (names never change anything else). */
export function relabel<S extends { subjects: readonly T[] }, T extends { address: string; label: string }>(scene: S, labels: ReadonlyMap<string, string>): S {
  if (!labels.size) return scene;
  return { ...scene, subjects: scene.subjects.map((s) => (labels.has(s.address) ? { ...s, label: labels.get(s.address)! } : s)) };
}
