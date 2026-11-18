// The workspace state shared by the views of one screen. Reactive, so a
// selection made on the chart reaches the card and the ruler.
import type { Json, Workspace } from "../../ui-contract/src/index.ts";

export function createWorkspace(now_yr: number): Workspace {
  const ws: Workspace = $state({
    selection: null,
    now_yr,
    mode: "edit",
    applied: {},
    revision: 0,
    select(address: string | null) {
      ws.selection = address;
    },
    setTime(yr: number) {
      ws.now_yr = yr;
    },
    noteApplied(address: string, field: string, value: Json, display: string, event_id: string) {
      ws.applied[`${address}#${field}`] = { value, display, event_id };
      ws.revision += 1;
    },
  });
  return ws;
}
