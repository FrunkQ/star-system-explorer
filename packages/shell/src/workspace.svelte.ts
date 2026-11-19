// The workspace state shared by the views of one screen. Reactive, so a
// selection made on the chart reaches the card and the ruler.
import type { AppliedEdit, ChangeRequest, DeriveRef, Workspace } from "../../ui-contract/src/index.ts";

export function createWorkspace(now_yr: number): Workspace {
  const ws: Workspace = $state({
    selection: null,
    now_yr,
    mode: "edit",
    applied: {},
    revision: 0,
    undo: [],
    redo: [],
    screen: "system",
    bench: null,
    requests: [],
    select(address: string | null) {
      ws.selection = address;
    },
    openFlow(flow: string, from: { node?: string; ref?: DeriveRef; label?: string } = {}) {
      ws.bench = { flow, ...from };
      ws.screen = "physics";
    },
    fileRequest(r: ChangeRequest) {
      const i = ws.requests.findIndex((x) => x.id === r.id);
      if (i >= 0) ws.requests[i] = r;
      else ws.requests.push(r);
    },
    notePhysics() {
      ws.revision += 1;
    },
    setTime(yr: number) {
      ws.now_yr = yr;
    },
    noteApplied(e: AppliedEdit, as: "edit" | "undo" | "redo" = "edit") {
      ws.applied[`${e.address}#${e.field}`] = { value: e.after.value, display: e.after.display, event_id: e.event_id };
      if (as === "edit") {
        ws.undo.push(e);
        ws.redo = [];
      } else if (as === "undo") ws.redo.push(e);
      else ws.undo.push(e);
      ws.revision += 1;
    },
  });
  return ws;
}
