// The one way a view changes something: send an intent for the current seat
// and, if it is applied, record it in the workspace so it can be undone.
// Undo and redo are themselves intents (the chronicle only grows): a
// correction inked over the earlier one.
import type { Json } from "./json.ts";
import type { AppliedEdit, Workspace } from "./view.ts";
import type { SceneFact, SystemScene } from "./scene.ts";
import type { IntentResult, ViewSession } from "./session.ts";

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
  const r = await session.send({ kind: "set_value", address: e.address, field: e.field, value: e.before.value, at_yr: ws.now_yr });
  if (r.status === "applied") {
    stack.pop();
    const back: AppliedEdit = { address: e.address, field: e.field, before: e.after, after: e.before, event_id: r.event_id };
    ws.noteApplied(back, from);
  }
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
