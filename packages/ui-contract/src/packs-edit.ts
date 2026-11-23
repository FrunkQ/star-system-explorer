// The Packs screen's workflow (U5, H5 stacking rules): see the stack in
// force, check it with the kernel, and add or remove a pack with a preview of
// every derived value it would change before anyone accepts. Adding or
// removing is one intent (stack_packs) through the session.
import type { Workspace } from "./view.ts";
import type { IntentResult, ViewSession } from "./session.ts";
import type { SystemScene } from "./scene.ts";
import { sig } from "./derivation.ts";
import type { PackDoc } from "./physics.ts";
import { readable } from "./physics-edit.ts";

export interface PackSummary {
  id: string;
  version: string;
  kinds: string[];
  fictional: boolean;
  licence: string;
  flows: string[];
  quantity_kinds: number;
  replaces: string[];
  /** Content the kernel carries but does not interpret yet (rules, data views, screens, skins...). */
  other: string[];
}

export function packSummary(p: PackDoc): PackSummary {
  const prov = (p.provides ?? {}) as { flows?: { id: string }[]; quantity_kinds?: unknown[]; other?: Record<string, unknown> };
  return {
    id: p.id,
    version: p.version,
    kinds: p.kind,
    fictional: !!p.fictional,
    licence: p.licence,
    flows: (prov.flows ?? []).map((f) => f.id),
    quantity_kinds: (prov.quantity_kinds ?? []).length,
    replaces: (p.replaces ?? []).map((r) => `${r.flow} from ${r.pack} ${r.range}`),
    other: Object.keys(prov.other ?? {}),
  };
}

export interface ValueChange {
  address: string;
  subject: string;
  field: string;
  label: string;
  before: string;
  after: string;
  fictional: boolean;
}

export interface StackPreview {
  ok: boolean;
  problems: string[];
  stack_hash?: string;
  changes: ValueChange[];
  /** What the added pack says about itself (ND6), when asked for: its sources, fudges and reproduction checks. */
  report?: PackReport | null;
}

/** One pack's own account of itself (kernel op pack_reports, ND6), worked out when the stack loaded. */
export interface PackReport {
  pack: string;
  version: string;
  licence: string;
  sources: { id: string; text: string; doi?: string; url?: string; licence?: string }[];
  fudges: {
    id: string;
    what: string;
    stands_in_for: string;
    replace_with?: string;
    flows: string[];
    source?: string;
    /** "declared" by the pack, or "model_number": a fitted or assumed number the kernel found in its flows. */
    from: "declared" | "model_number";
    number?: { value?: number; unit?: string; grade?: string; source?: string };
  }[];
  reproduction: {
    name: string;
    source: string;
    where?: string;
    flow: string;
    pass: boolean;
    rows: { kind: string; unit?: string; published: number; got: number | null; tolerance: number; off: number | null; pass: boolean; error?: string }[];
  }[];
  /** Every check within its tolerance (true when there are none). */
  reproduced: boolean;
}

/** A source's link: its DOI when it has one, else its address. */
export function sourceLink(s: { doi?: string; url?: string }): string | null {
  if (s.doi) return `https://doi.org/${s.doi.replace(/^https?:\/\/(dx\.)?doi\.org\//, "")}`;
  return s.url ?? null;
}

/** A unit id as people write it: "m_s2" is m/s², "w_m2" is W/m², "1" is nothing. */
export function unitWords(u: string | undefined): string {
  if (!u || u === "1" || u === "one") return "";
  const sup = (x: string) => x.replace(/(\d+)$/, (d) => [...d].map((c) => "⁰¹²³⁴⁵⁶⁷⁸⁹"[Number(c)]).join(""));
  const [top, ...under] = u.split("_").map((x) => sup(x === "w" ? "W" : x === "k" ? "K" : x));
  return under.length ? `${top}/${under.join("/")}` : top!;
}

/** "0.2 %" for a relative closeness. */
export function percentWords(x: number | null | undefined): string {
  if (x === null || x === undefined || !Number.isFinite(x)) return "–";
  const p = x * 100;
  return `${p >= 10 ? Math.round(p) : Number(p.toPrecision(2))} %`;
}

/** One line for a pack's report: "2 sources, 2 fudges, 1 of 1 checks reproduced". */
export function reportWords(r: PackReport): string {
  const n = (k: number, one: string, many: string) => `${k} ${k === 1 ? one : many}`;
  const passed = r.reproduction.filter((c) => c.pass).length;
  return [
    n(r.sources.length, "source", "sources"),
    n(r.fudges.length, "fudge", "fudges"),
    r.reproduction.length ? `${passed} of ${n(r.reproduction.length, "check", "checks")} reproduced` : "no reproduction checks",
  ].join(", ");
}

/**
 * What a candidate set of added packs would do: the kernel validates the
 * stack, then every derived value on the scene is derived again under it.
 */
export async function previewStack(session: ViewSession, scene: SystemScene, added: readonly PackDoc[], opts: { report?: string } = {}): Promise<StackPreview> {
  const facts = scene.subjects.flatMap((s) => s.facts.filter((f) => f.source === "derived" && f.why && f.scale).map((f) => ({ s, f })));
  const t = await session.trial({ added }, async (p) => {
    const out: ValueChange[] = [];
    for (const { s, f } of facts) {
      const r = await p.derive({ flow: f.why!.flow, inputs: f.why!.inputs });
      const d = r.ok ? r.values[f.why!.kind] : undefined;
      const after = d ? `${sig(d.value / f.scale!, f.digits ?? 3)}${f.unit ? ` ${f.unit}` : ""}` : "not derived";
      if (after !== f.display || (d?.fictional ?? false) !== !!f.fictional)
        out.push({ address: s.address, subject: s.label, field: f.field, label: f.label, before: f.display, after, fictional: d?.fictional ?? false });
    }
    let report: PackReport | null | undefined;
    if (opts.report) {
      const r = await p.packReports(opts.report);
      report = r.ok ? ((r.reports[0] as PackReport | undefined) ?? null) : null;
    }
    return { changes: out, report };
  });
  if (!t.ok) return { ok: false, problems: (t.issues.length ? t.issues : [t.report]).map(readable), changes: [] };
  return { ok: true, problems: [], stack_hash: t.stack_hash, changes: t.result.changes, ...(t.result.report !== undefined ? { report: t.result.report } : {}) };
}

/** Set the added packs (bottom to top). One intent; everything re-derives. */
export async function stackPacks(session: ViewSession, ws: Workspace, added: readonly PackDoc[]): Promise<IntentResult> {
  const r = await session.send({ kind: "stack_packs", packs: added as never });
  if (r.status === "applied") ws.notePhysics();
  return r;
}

/** Why a document is not a pack the stack could take, or null (schema only; the kernel checks the rest). */
export function checkPackDoc(x: unknown, inStack: readonly string[]): string | null {
  const p = x as PackDoc;
  if (!p || typeof p !== "object" || Array.isArray(p)) return "not a JSON object";
  if (p.format !== "sse.pack/1") return "not an sse.pack/1 document";
  if (typeof p.id !== "string" || !/^[a-z0-9-]+\.[a-z0-9.-]+$/.test(p.id)) return "pack id must be publisher.name";
  if (inStack.includes(p.id)) return `${p.id} is already in the stack`;
  if (!Array.isArray(p.kind) || !p.kind.length) return "pack must say what kind it is";
  if (typeof p.version !== "string" || typeof p.kernel_range !== "string" || typeof p.licence !== "string") return "pack needs version, kernel_range and licence";
  return null;
}
