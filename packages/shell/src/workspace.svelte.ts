// The workspace state shared by the views of one screen. Reactive, so a
// selection made on the chart reaches the card and the ruler.
import { withChoice, type AppliedEdit, type ChangeRequest, type DeriveRef, type Workspace } from "../../ui-contract/src/index.ts";
import { emptyPlaybook, followTime, stoppedAt, withRate } from "../../ui-contract/src/index.ts";

const wallNow = () => (typeof performance !== "undefined" ? performance.now() : 0);

export function createWorkspace(now_yr: number): Workspace {
  const ws: Workspace = $state({
    selection: null,
    now_yr,
    campaign_now_yr: now_yr,
    campaign_epoch_yr: now_yr,
    clock: stoppedAt(now_yr, now_yr),
    mode: "edit",
    applied: {},
    revision: 0,
    undo: [],
    redo: [],
    screen: "system",
    bench: null,
    requests: [],
    branch: null,
    adding: null,
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
      ws.clock = followTime(ws.clock ?? stoppedAt(yr, ws.campaign_epoch_yr ?? yr), yr, ws.campaign_epoch_yr ?? yr, wallNow());
    },
    setRate(rate: number) {
      const c = followTime(ws.clock ?? stoppedAt(ws.now_yr, ws.campaign_epoch_yr ?? ws.now_yr), ws.now_yr, ws.campaign_epoch_yr ?? ws.now_yr, wallNow());
      ws.clock = withRate(c, rate, wallNow());
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
    unit_choices: {},
    could_quiet: [],
    setUnitChoice(kind, about, reading) {
      ws.unit_choices = withChoice(ws.unit_choices, kind, about, reading);
    },
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
