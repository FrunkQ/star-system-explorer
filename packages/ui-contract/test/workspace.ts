// A plain (non-reactive) workspace for tests; the shell's is the same shape with Svelte state.
import type { AppliedEdit, Workspace } from "../src/view.ts";
import type { ChangeRequest } from "../src/physics.ts";

export function plainWorkspace(now_yr = 4_610_000_000): Workspace {
  const ws: Workspace = {
    selection: null, now_yr, mode: "edit", applied: {}, revision: 0, undo: [], redo: [],
    screen: "system", bench: null, requests: [], branch: null,
    select: () => {}, setTime: () => {},
    openFlow(flow, from = {}) { ws.bench = { flow, ...from }; ws.screen = "physics"; },
    fileRequest(r: ChangeRequest) { const i = ws.requests.findIndex((x) => x.id === r.id); if (i >= 0) ws.requests[i] = r; else ws.requests.push(r); },
    notePhysics() { ws.revision++; },
    noteApplied(e: AppliedEdit, as = "edit") {
      ws.applied[`${e.address}#${e.field}`] = { value: e.after.value, display: e.after.display, event_id: e.event_id };
      if (as === "edit") { ws.undo.push(e); ws.redo = []; } else if (as === "undo") ws.redo.push(e); else ws.undo.push(e);
      ws.revision++;
    },
  };
  return ws;
}
