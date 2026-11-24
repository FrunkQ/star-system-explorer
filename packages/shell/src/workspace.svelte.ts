// The workspace state shared by the views of one screen. Reactive, so a
// selection made on the chart reaches the card and the ruler.
import type { AppliedEdit, ChangeRequest, DeriveRef, Workspace } from "../../ui-contract/src/index.ts";
import { emptyPlaybook } from "../../ui-contract/src/index.ts";

export function createWorkspace(now_yr: number): Workspace {
  const ws: Workspace = $state({
    selection: null,
    now_yr,
    campaign_now_yr: now_yr,
    campaign_epoch_yr: now_yr,
    mode: "edit",
    applied: {},
    revision: 0,
    undo: [],
    redo: [],
    screen: "system",
    bench: null,
    requests: [],
    branch: null,
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
    setCampaignNow(yr: number) {
      ws.campaign_now_yr = yr;
    },
    party: null,
    setParty(p) {
      ws.party = p;
    },
    bookmarks: [],
    playbook: emptyPlaybook(),
    overlay_anchors: {},
    setOverlayAnchor(id, a) {
      const next = { ...(ws.overlay_anchors ?? {}) };
      if (a) next[id] = a;
      else delete next[id];
      ws.overlay_anchors = next;
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
