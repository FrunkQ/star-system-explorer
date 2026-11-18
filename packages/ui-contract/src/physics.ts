// Physics change requests (H13, build plan U3). A change to the physics is a
// document: an edited copy of a flow, carried in the campaign's own pack with
// an explicit `replaces`, so the base pack is never touched and every value
// keeps saying which pack published it. Orac (A2) will file the same
// documents; until then the GM files them from the flow editor, and a
// fixture stands in for Orac. Nothing here talks to the kernel: the session
// validates and previews (Session.trial) and adoption is an intent.
import type { Json } from "./json.ts";

/** The parts of an `sse.flow/1` document the editor reads. Other fields are kept as they are. */
export interface FlowDoc {
  format: "sse.flow/1";
  id: string;
  version: string;
  hook?: string;
  inputs: { name: string; kind: string }[];
  nodes: { id: string; atom: string; params?: Record<string, number> }[];
  wires: { from: string; to: string }[];
  publishes: { from: string; kind: string }[];
  [other: string]: unknown;
}

/** The parts of an `sse.pack/1` manifest the editor reads. */
export interface PackDoc {
  format: string;
  id: string;
  version: string;
  kind: string[];
  kernel_range: string;
  licence: string;
  fictional?: boolean;
  replaces?: { flow: string; pack: string; range: string }[];
  provides?: { flows?: FlowDoc[]; [other: string]: unknown };
  [other: string]: unknown;
}

/** One parameter of one node, before and after. */
export interface ParamEdit {
  node: string;
  param: string;
  before: number;
  after: number;
}

export type RequestStatus = "draft" | "adopted" | "declined" | "superseded";

/**
 * A proposed physics change, shown as a diff with its effects before anyone
 * accepts it. `author` is "gm" or "orac"; `rationale` is the author's reason
 * in plain words.
 */
export interface ChangeRequest {
  id: string;
  title: string;
  author: "gm" | "orac";
  rationale: string;
  flow: string;
  edits: ParamEdit[];
  status: RequestStatus;
  /** True while the request comes from a fixture document standing in for Orac. */
  fixture?: boolean;
}

/** Where the campaign's own physics lives in the stack (always the top pack). */
export const CAMPAIGN_PACK = "campaign.physics";

/** Every parameter a flow sets, as node → name → value. */
export function flowParams(f: FlowDoc): { node: string; param: string; value: number }[] {
  return f.nodes.flatMap((n) => Object.entries(n.params ?? {}).map(([param, value]) => ({ node: n.id, param, value })));
}

/** A copy of the flow with the edits applied. Unknown nodes are an error. */
export function editFlow(f: FlowDoc, edits: readonly ParamEdit[]): FlowDoc {
  const out = structuredClone(f);
  for (const e of edits) {
    const n = out.nodes.find((x) => x.id === e.node);
    if (!n) throw new Error(`flow ${f.id} has no node ${e.node}`);
    n.params = { ...(n.params ?? {}), [e.param]: e.after };
  }
  return out;
}

/** Parameter differences from `a` to `b` (same flow id). Structure changes are listed by text. */
export function diffFlow(a: FlowDoc, b: FlowDoc): { params: ParamEdit[]; structure: string[] } {
  const params: ParamEdit[] = [];
  const structure: string[] = [];
  const key = (n: string, p: string) => `${n}\u0000${p}`;
  const before = new Map(flowParams(a).map((x) => [key(x.node, x.param), x.value]));
  for (const x of flowParams(b)) {
    const was = before.get(key(x.node, x.param));
    if (was === undefined) structure.push(`parameter ${x.node}.${x.param} added`);
    else if (was !== x.value) params.push({ node: x.node, param: x.param, before: was, after: x.value });
    before.delete(key(x.node, x.param));
  }
  for (const k of before.keys()) structure.push(`parameter ${k.replace("\u0000", ".")} removed`);
  const ids = (f: FlowDoc) => f.nodes.map((n) => `${n.id}:${n.atom}`).join(",");
  if (ids(a) !== ids(b)) structure.push("nodes changed");
  const ws = (f: FlowDoc) => f.wires.map((w) => `${w.from}>${w.to}`).join(",");
  if (ws(a) !== ws(b)) structure.push("wires changed");
  return { params, structure };
}

/** The pack in `packs` (bottom to top) whose flow a stack uses for `flow`: the highest provider. */
export function providerOf(packs: readonly PackDoc[], flow: string): { pack: PackDoc; doc: FlowDoc } | undefined {
  for (let i = packs.length - 1; i >= 0; i--) {
    const doc = packs[i]!.provides?.flows?.find((f) => f.id === flow);
    if (doc) return { pack: packs[i]!, doc };
  }
  return undefined;
}

/**
 * The campaign pack that carries the given flows over the base stack. Each
 * flow replaces the base pack's flow of the same id, explicitly. Returns null
 * when there is nothing to carry (the base physics as shipped).
 */
export function campaignPack(base: readonly PackDoc[], flows: readonly FlowDoc[], version = "0.1.0"): PackDoc | null {
  if (!flows.length) return null;
  const top = base[base.length - 1];
  const replaces = flows.map((f) => {
    const p = providerOf(base, f.id);
    if (!p) throw new Error(`no base pack provides ${f.id}`);
    return { flow: f.id, pack: p.pack.id, range: `^${p.pack.version}` };
  });
  return {
    format: "sse.pack/1",
    id: CAMPAIGN_PACK,
    version,
    kind: ["physics"],
    kernel_range: top?.kernel_range ?? "*",
    licence: "Apache-2.0",
    replaces,
    provides: { flows: [...flows] },
  };
}

/**
 * The campaign pack that results from adopting `requests` (in order) over the
 * base stack: each flow edited by every adopted request that touches it.
 */
export function packFor(base: readonly PackDoc[], requests: readonly Pick<ChangeRequest, "flow" | "edits">[]): PackDoc | null {
  const flows = new Map<string, FlowDoc>();
  for (const r of requests) {
    const cur = flows.get(r.flow) ?? providerOf(base, r.flow)?.doc;
    if (!cur) throw new Error(`no base pack provides ${r.flow}`);
    flows.set(r.flow, editFlow(cur, r.edits));
  }
  return campaignPack(base, [...flows.values()]);
}

/** Why a request cannot be shown for acceptance, or null. Schema checks only; the kernel validates the rest. */
export function checkRequest(r: ChangeRequest, base: readonly PackDoc[]): string | null {
  if (!/^[a-z0-9][a-z0-9._-]*$/.test(r.id)) return "request id must be lower-case text";
  if (!r.title.trim()) return "request needs a title";
  const p = providerOf(base, r.flow);
  if (!p) return `no pack in the stack provides ${r.flow}`;
  if (!r.edits.length) return "request changes nothing";
  for (const e of r.edits) {
    const n = p.doc.nodes.find((x) => x.id === e.node);
    if (!n) return `flow ${r.flow} has no node ${e.node}`;
    if (!Number.isFinite(e.after)) return `${e.node}.${e.param} must be a number`;
  }
  return null;
}

/** Columns for drawing a flow left to right: inputs, then nodes by depth, then what it publishes. */
export function flowColumns(f: FlowDoc): string[][] {
  const depth = new Map<string, number>();
  const nodeOf = (port: string) => port.split(".")[0]!;
  const ids = f.nodes.map((n) => n.id);
  // Longest path from the inputs; flows are acyclic (K4 rejects cycles).
  for (let pass = 0; pass < ids.length + 1; pass++) {
    for (const id of ids) {
      const ups = f.wires.filter((w) => nodeOf(w.to) === id && nodeOf(w.from) !== "in").map((w) => nodeOf(w.from));
      depth.set(id, ups.length ? Math.max(...ups.map((u) => (depth.get(u) ?? 0) + 1)) : 0);
    }
  }
  const cols: string[][] = [];
  for (const id of ids) (cols[depth.get(id)!] ??= []).push(id);
  return cols.filter(Boolean);
}

/** Text of a parameter value as the editor shows it. */
export function paramText(v: number): string {
  return Number.isInteger(v) ? String(v) : String(Number(v.toPrecision(6)));
}

export type { Json };

const NAMED_SI: Record<string, string> = {
  "kg s^-3": "W/m²",
  "kg m^2 s^-3": "W",
  "kg m^2 s^-2": "J",
  "m s^-2": "m/s²",
  "m s^-1": "m/s",
  "kg m^-3": "kg/m³",
  "kg m^-1 s^-2": "Pa",
  "1": "",
};

/** A kernel SI dimension string ("kg s^-3") as people read it ("W/m²"). */
export function siText(dim: string): string {
  return NAMED_SI[dim] ?? dim.replace(/\^(-?\d+)/g, (_, p: string) => p.replace(/-/g, "⁻").replace(/\d/g, (d) => "⁰¹²³⁴⁵⁶⁷⁸⁹"[Number(d)]!));
}
