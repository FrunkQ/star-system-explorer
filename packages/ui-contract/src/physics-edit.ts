// The physics workflow over the session and workspace: draft a parameter
// change, preview it, adopt it (an intent), withdraw it. Views call these and
// nothing else, so the flow editor and the change-request panel cannot
// diverge, and Orac's requests (A2) go through the same path as the GM's.
import type { Workspace } from "./view.ts";
import type { DeriveRef, Derived, IntentResult, ViewSession } from "./session.ts";
import type { SystemScene } from "./scene.ts";
import { checkRequest, packFor, type ChangeRequest, type PackDoc } from "./physics.ts";

/** The id of the GM's open draft for a flow. */
export const draftId = (flow: string) => `gm-draft.${flow}`;

/** Requests adopted now, in the order they were adopted. */
export function adopted(ws: Workspace): ChangeRequest[] {
  return ws.requests.filter((r) => r.status === "adopted");
}

/**
 * Change one parameter in the GM's draft for the flow (making the draft if
 * there is none). `before` is the value in force. Setting a parameter back
 * to the value in force removes it from the draft.
 */
export function draftParam(session: ViewSession, ws: Workspace, flow: string, node: string, param: string, value: number): ChangeRequest | null {
  const doc = session.flowDoc(flow)?.doc;
  const before = doc?.nodes.find((n) => n.id === node)?.params?.[param];
  if (before === undefined) return null;
  const id = draftId(flow);
  const cur = ws.requests.find((r) => r.id === id && r.status === "draft");
  const edits = (cur?.edits ?? []).filter((e) => !(e.node === node && e.param === param));
  if (value !== before) edits.push({ node, param, before, after: value });
  const r: ChangeRequest = cur
    ? { ...cur, edits }
    : { id, title: `Change ${flow.replace(/^sse\.base\./, "").replace(/-/g, " ")}`, author: "gm", rationale: "", flow, edits, status: "draft" };
  ws.fileRequest(r);
  return r;
}

/** The campaign pack the stack would carry with `r` adopted on top of what is adopted now. */
export function candidateFor(session: ViewSession, ws: Workspace, r: ChangeRequest | null, without?: string): PackDoc | null {
  const base = session.stack().base;
  const keep = adopted(ws).filter((x) => x.id !== without && x.id !== r?.id);
  return packFor(base, r ? [...keep, r] : keep);
}

export interface Preview {
  ok: boolean;
  /** Schema or kernel validation problems; a request with any never reaches Accept. */
  problems: string[];
  /** Before and after values of each published kind, for the ref. */
  values: { kind: string; before: Derived | null; after: Derived | null }[];
}

/** Validate a request and show its effect on one set of inputs, without adopting it. */
export async function previewRequest(session: ViewSession, ws: Workspace, r: ChangeRequest, ref: DeriveRef | null): Promise<Preview> {
  const bad = checkRequest(r, session.stack().base);
  if (bad) return { ok: false, problems: [bad], values: [] };
  let pack: PackDoc | null;
  try {
    pack = candidateFor(session, ws, r);
  } catch (e) {
    return { ok: false, problems: [String((e as Error).message)], values: [] };
  }
  const t = await session.trial({ campaign: pack }, async (p) => (ref ? p.derive(ref) : null));
  if (!t.ok) return { ok: false, problems: (t.issues.length ? t.issues : [t.report]).map(readable), values: [] };
  if (!ref || !t.result) return { ok: true, problems: [], values: [] };
  const now = await session.derive(ref);
  const after = t.result;
  if (!after.ok) return { ok: false, problems: [after.message], values: [] };
  const kinds = [...new Set([...Object.keys(now.ok ? now.values : {}), ...Object.keys(after.values)])].sort();
  return { ok: true, problems: [], values: kinds.map((kind) => ({ kind, before: now.ok ? (now.values[kind] ?? null) : null, after: after.values[kind] ?? null })) };
}

async function adoptSet(session: ViewSession, ws: Workspace, set: ChangeRequest[]): Promise<IntentResult> {
  let pack: PackDoc | null;
  try {
    pack = packFor(session.stack().base, set);
  } catch (e) {
    return { status: "refused", reason: String((e as Error).message) };
  }
  return session.send({ kind: "adopt_physics", pack: pack as never, requests: set.map((x) => x.id) });
}

/** Adopt a request: the campaign pack is rebuilt with it on top, as one intent. */
export async function adoptRequest(session: ViewSession, ws: Workspace, id: string): Promise<IntentResult> {
  const r = ws.requests.find((x) => x.id === id);
  if (!r) return { status: "refused", reason: `no request ${id}` };
  const bad = checkRequest(r, session.stack().base);
  if (bad) return { status: "refused", reason: bad };
  const res = await adoptSet(session, ws, [...adopted(ws).filter((x) => x.id !== id), r]);
  if (res.status === "applied") {
    // An adopted GM draft gets its own id, so the next edit starts a new draft.
    const keepId = r.id === draftId(r.flow) ? `${r.id}.${ws.requests.length}` : r.id;
    ws.requests = ws.requests.filter((x) => x.id !== r.id);
    ws.fileRequest({ ...r, id: keepId, status: "adopted" });
    for (const o of ws.requests) if (o.status === "draft" && o.flow === r.flow && o.author === "orac" && overlaps(o, r)) ws.fileRequest({ ...o, status: "superseded" });
    ws.notePhysics();
  }
  return res;
}

/** Withdraw an adopted request: the campaign pack is rebuilt without it. */
export async function withdrawRequest(session: ViewSession, ws: Workspace, id: string): Promise<IntentResult> {
  const r = ws.requests.find((x) => x.id === id && x.status === "adopted");
  if (!r) return { status: "refused", reason: `request ${id} is not adopted` };
  const res = await adoptSet(session, ws, adopted(ws).filter((x) => x.id !== id));
  if (res.status === "applied") {
    ws.fileRequest({ ...r, status: "declined" });
    ws.notePhysics();
  }
  return res;
}

/** Decline a draft. Nothing changes in the physics. */
export function declineRequest(ws: Workspace, id: string): void {
  const r = ws.requests.find((x) => x.id === id);
  if (r && r.status === "draft") ws.fileRequest({ ...r, status: "declined" });
}

function overlaps(a: ChangeRequest, b: ChangeRequest): boolean {
  return a.edits.some((e) => b.edits.some((f) => f.node === e.node && f.param === e.param));
}

/**
 * Real inputs to show a flow with: the bench's own, else those of a derived
 * fact of the selected subject, else of the first subject the flow serves.
 */
export function benchRef(scene: SystemScene, ws: Workspace, flow: string): { ref: DeriveRef; label: string } | null {
  if (ws.bench?.flow === flow && ws.bench.ref) return { ref: ws.bench.ref, label: ws.bench.label ?? "" };
  const subjects = [...scene.subjects].sort((a, b) => Number(b.address === ws.selection) - Number(a.address === ws.selection));
  for (const s of subjects) {
    const f = s.facts.find((x) => x.why?.flow === flow);
    if (f?.why) return { ref: { flow, inputs: f.why.inputs }, label: s.label };
  }
  return null;
}

/** Kernel validation text with its Rust debug forms turned into words ("Some((0.25, 0.67))" → "0.25 to 0.6667"). */
export function readable(message: string): string {
  const n = (x: string) => String(Number(Number(x).toPrecision(4)));
  return message.replace(/Some\(\((-?[\d.e+-]+), (-?[\d.e+-]+)\)\)/g, (_, a: string, b: string) => `${n(a)} to ${n(b)}`);
}
