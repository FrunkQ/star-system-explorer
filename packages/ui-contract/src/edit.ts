// The one way a view changes something: send an intent for the current seat
// and, if it is applied, record it in the workspace so it can be undone.
// Undo and redo are themselves intents (the chronicle only grows): a
// correction inked over the earlier one.
import type { Json } from "./json.ts";
import type { AppliedEdit, Workspace } from "./view.ts";
import type { SceneFact, SystemScene } from "./scene.ts";
import type { IntentResult, ViewSession } from "./session.ts";
import type { Intent } from "./intent.ts";

/**
 * Where a typed value goes: a fact read back from the universe is held at its kernel field in SI (SceneFact.kernel),
 * so the value sent is scaled there; the fixture's own facts take the value as typed, at their own field.
 */
export function factEdit(f: SceneFact, value: Json): { field: string; value: Json; before: Json } {
  const k = f.kernel;
  const si = (v: Json) => (k && typeof v === "number" ? v * k.per_unit : v);
  return { field: k?.field ?? f.field, value: si(value), before: si(f.value) };
}

export async function commitEdit(
  session: ViewSession,
  ws: Workspace,
  change: { address: string; field: string; value: Json; display: string; before: { value: Json; display: string } },
): Promise<IntentResult> {
  const intent = { kind: "set_value" as const, address: change.address, field: change.field, value: change.value, at_yr: ws.now_yr };
  // A what-if branch is open: the edit goes to it, and the main line does not move.
  const b = ws.branch;
  if (b) {
    const r = await session.branchSend(b.id, intent);
    if (r.status === "applied") {
      b.edits.push({ address: change.address, field: change.field, before: change.before, after: { value: change.value, display: change.display }, event_id: r.event_id });
      b.applied[`${change.address}#${change.field}`] = { value: change.value, display: change.display, event_id: r.event_id };
      ws.revision += 1;
    }
    return r;
  }
  const r = await session.send(intent);
  if (r.status === "applied")
    ws.noteApplied({ address: change.address, field: change.field, before: change.before, after: { value: change.value, display: change.display }, event_id: r.event_id });
  return r;
}

async function reverse(session: ViewSession, ws: Workspace, from: "undo" | "redo"): Promise<IntentResult | null> {
  // Undo and redo work on the main line; a what-if branch is kept or thrown away whole.
  if (ws.branch) return null;
  const stack = from === "undo" ? ws.undo : ws.redo;
  const e = stack[stack.length - 1];
  if (!e) return null;
  // A paste or a cut is kept as the presence of its top bodies: taking it back withdraws them, or brings them back.
  const r =
    e.field === FIELDS
      ? await session.send({ kind: "set_values", edits: Object.entries(e.before.value as Record<string, Json>).map(([k, value]) => ({ ...splitKey(k, e.address), value })), at_yr: ws.now_yr })
      : e.field === PRESENCE
      ? await sendAll(session, (e.before.value as string[] | null) ? (e.before.value as string[]).map((a) => ({ kind: "restore" as const, address: a, at_yr: ws.now_yr })) : (e.after.value as string[]).map((a) => ({ kind: "withdraw" as const, address: a, at_yr: ws.now_yr })))
      : await session.send({ kind: "set_value", address: e.address, field: e.field, value: e.before.value, at_yr: ws.now_yr });
  if (r.status === "applied") {
    stack.pop();
    const back: AppliedEdit = { address: e.address, field: e.field, before: e.after, after: e.before, event_id: r.event_id };
    ws.noteApplied(back, from);
  }
  return r;
}

/**
 * The undo stack's field for a paste or a cut: before and after hold the top
 * bodies' addresses where they are present, null where they are not, and the
 * address is the first of them.
 */
export const PRESENCE = "presence";

/**
 * The undo stack's field for several fields set as one change (the Air box's mix, the sheet, the system sheet):
 * before and after hold `{"<address>#<field>": value}` records; `address` is the first body written.
 */
export const FIELDS = "fields";
const splitKey = (k: string, fallback: string) => {
  const i = k.lastIndexOf("#");
  return i < 0 ? { address: fallback, field: k } : { address: k.slice(0, i), field: k.slice(i + 1) };
};

/**
 * Several fields as one change: one authored event (set_values), one Undo. Each edit is of `address` unless it names
 * its own (the system sheet writes many bodies). On a what-if, one edit each.
 */
export async function commitFields(
  session: ViewSession,
  ws: Workspace,
  change: { address: string; edits: { address?: string; field: string; value: Json; before: Json }[]; display: string; before_display: string; evidence?: "authored" | "catalogued"; source?: string },
): Promise<IntentResult> {
  const at = (e: { address?: string }) => e.address ?? change.address;
  const b = ws.branch;
  if (b) {
    let last: IntentResult = { status: "refused", reason: "nothing to send" };
    for (const e of change.edits) {
      last = await session.branchSend(b.id, { kind: "set_value", address: at(e), field: e.field, value: e.value, at_yr: ws.now_yr });
      if (last.status !== "applied") return last;
      b.edits.push({ address: at(e), field: e.field, before: { value: e.before, display: "" }, after: { value: e.value, display: change.display }, event_id: last.event_id });
      b.applied[`${at(e)}#${e.field}`] = { value: e.value, display: change.display, event_id: last.event_id };
    }
    ws.revision += 1;
    return last;
  }
  const r = await session.send({ kind: "set_values", edits: change.edits.map((e) => ({ address: at(e), field: e.field, value: e.value })), at_yr: ws.now_yr, ...(change.evidence ? { evidence: change.evidence } : {}), ...(change.source ? { source: change.source } : {}) });
  if (r.status === "applied") {
    const rec = (k: "before" | "value") => Object.fromEntries(change.edits.map((e) => [`${at(e)}#${e.field}`, e[k]])) as Record<string, Json>;
    ws.noteApplied({ address: change.address, field: FIELDS, before: { value: rec("before"), display: change.before_display }, after: { value: rec("value"), display: change.display }, event_id: r.event_id });
  }
  return r;
}

async function sendAll(session: ViewSession, intents: Intent[]): Promise<IntentResult> {
  let last: IntentResult = { status: "refused", reason: "nothing to send" };
  for (const i of intents) {
    last = await session.send(i);
    if (last.status !== "applied") return last;
  }
  return last;
}

/**
 * Place bodies (a paste) as one change: the possible values in one authored
 * event, any impossible ones in a second, and one entry on the undo stack, so
 * one Undo withdraws them all. `roots` are the top bodies placed; `words`
 * says what was placed, for the undo list.
 */
export async function placeBodies(
  session: ViewSession,
  ws: Workspace,
  p: { edits: { address: string; field: string; value: Json; impossible?: boolean }[]; roots: string[]; words: string; evidence?: "authored" | "catalogued"; source?: string },
): Promise<IntentResult> {
  if (ws.branch) return { status: "refused", reason: "a what-if takes value edits only: keep it or throw it away, then paste" };
  const possible = p.edits.filter((e) => !e.impossible);
  const impossible = p.edits.filter((e) => e.impossible);
  const extra = { ...(p.evidence ? { evidence: p.evidence } : {}), ...(p.source ? { source: p.source } : {}) };
  const batches: Intent[] = [possible, impossible].filter((b) => b.length).map((edits) => ({ kind: "set_values" as const, edits, at_yr: ws.now_yr, ...(edits === possible ? extra : {}) }));
  const r = await sendAll(session, batches);
  if (r.status === "applied") ws.noteApplied({ address: p.roots[0]!, field: PRESENCE, before: { value: null, display: "not here" }, after: { value: p.roots, display: p.words }, event_id: r.event_id });
  return r;
}

/** Take bodies out (a cut), as one entry on the undo stack; Undo brings them back with every fact they had. */
export async function withdrawBodies(session: ViewSession, ws: Workspace, p: { roots: string[]; words: string }): Promise<IntentResult> {
  if (ws.branch) return { status: "refused", reason: "a what-if takes value edits only: keep it or throw it away, then cut" };
  const r = await sendAll(session, p.roots.map((a) => ({ kind: "withdraw" as const, address: a, at_yr: ws.now_yr })));
  if (r.status === "applied") ws.noteApplied({ address: p.roots[0]!, field: PRESENCE, before: { value: p.roots, display: p.words }, after: { value: null, display: "not here" }, event_id: r.event_id });
  return r;
}

export const undoEdit = (session: ViewSession, ws: Workspace) => reverse(session, ws, "undo");
export const redoEdit = (session: ViewSession, ws: Workspace) => reverse(session, ws, "redo");

// ------------------------------------------------------------ what-if branches (U4)

/** Open a what-if branch from now. Edits made while it is open go to it. */
export async function openWhatIf(session: ViewSession, ws: Workspace, label: string): Promise<string | null> {
  if (ws.branch) return "A what-if is already open: keep it or throw it away first.";
  const r = await session.openBranch(label);
  if (!r.ok) return r.message;
  ws.branch = { id: r.branch, label, from_yr: ws.now_yr, edits: [], applied: {} };
  ws.revision += 1;
  return null;
}

/** Throw the branch away. The main line is exactly as it was. */
export async function dropWhatIf(session: ViewSession, ws: Workspace): Promise<void> {
  const b = ws.branch;
  if (!b) return;
  await session.discardBranch(b.id);
  ws.branch = null;
  ws.revision += 1;
}

/**
 * Keep the branch: its edits are sent on the main line, in order, as the
 * GM's own intents, so they replicate to every player's screen (V4) and each
 * can be undone. The kernel's branch is then dropped; it has served as the
 * preview. Returns the first problem, if any edit is refused.
 */
export async function keepWhatIf(session: ViewSession, ws: Workspace): Promise<string | null> {
  const b = ws.branch;
  if (!b) return null;
  await session.discardBranch(b.id);
  ws.branch = null;
  for (const e of b.edits) {
    const r = await commitEdit(session, ws, { address: e.address, field: e.field, value: e.after.value, display: e.after.display, before: e.before });
    if (r.status !== "applied") return r.status === "refused" ? r.reason : r.status === "error" ? r.message : "sent to the GM to approve";
  }
  ws.revision += 1;
  return null;
}

/** One difference between the main line and a what-if, as the card shows it. */
export interface Difference {
  address: string;
  label: string;
  field: string;
  field_label: string;
  before: string;
  after: string;
  /** Set on the branch by the GM, or derived from what was set. */
  how: "set" | "follows";
}

/** Control versus changed (H6.3): every fact whose shown value differs between two scenes of the same system. */
export function compareScenes(main: SystemScene, branch: SystemScene, set: Record<string, unknown> = {}): Difference[] {
  const out: Difference[] = [];
  const by = new Map(main.subjects.map((s) => [s.address, s]));
  for (const s of branch.subjects) {
    const m = by.get(s.address);
    if (!m) continue;
    const mf = new Map<string, SceneFact>(m.facts.map((f) => [f.field, f]));
    for (const f of s.facts) {
      const g = mf.get(f.field);
      if (!g || g.display === f.display) continue;
      out.push({ address: s.address, label: s.label, field: f.field, field_label: f.label, before: g.display, after: f.display, how: `${s.address}#${f.field}` in set ? "set" : "follows" });
    }
  }
  return out;
}
