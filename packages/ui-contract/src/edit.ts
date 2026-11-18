// The one way a view changes something: send an intent for the current seat
// and, if it is applied, record it in the workspace so it can be undone.
// Undo and redo are themselves intents (the chronicle only grows): a
// correction inked over the earlier one.
import type { Json } from "./json.ts";
import type { AppliedEdit, Workspace } from "./view.ts";
import type { IntentResult, ViewSession } from "./session.ts";

export async function commitEdit(
  session: ViewSession,
  ws: Workspace,
  change: { address: string; field: string; value: Json; display: string; before: { value: Json; display: string } },
): Promise<IntentResult> {
  const r = await session.send({ kind: "set_value", address: change.address, field: change.field, value: change.value, at_yr: ws.now_yr });
  if (r.status === "applied")
    ws.noteApplied({ address: change.address, field: change.field, before: change.before, after: { value: change.value, display: change.display }, event_id: r.event_id });
  return r;
}

async function reverse(session: ViewSession, ws: Workspace, from: "undo" | "redo"): Promise<IntentResult | null> {
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
