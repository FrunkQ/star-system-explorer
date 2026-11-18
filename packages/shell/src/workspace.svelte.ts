// The workspace state shared by the views of one screen. Reactive, so a
// selection made on the chart reaches the card and the ruler.
import type { AppliedEdit, Workspace } from "../../ui-contract/src/index.ts";

export function createWorkspace(now_yr: number): Workspace {
  const ws: Workspace = $state({
    selection: null,
    now_yr,
    mode: "edit",
    applied: {},
    revision: 0,
    undo: [],
    redo: [],
    select(address: string | null) {
      ws.selection = address;
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
