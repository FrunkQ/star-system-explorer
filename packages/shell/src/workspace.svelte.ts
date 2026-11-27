// The workspace state shared by the views of one screen. Reactive, so a
// selection made on the chart reaches the card and the ruler.
import { withChoice, type AppliedEdit, type ChangeRequest, type DeriveRef, type Workspace } from "../../ui-contract/src/index.ts";
import { untrack } from "svelte";
import { emptyPlaybook, followTime, stoppedAt, withRate, YEAR_SECONDS } from "../../ui-contract/src/index.ts";

const wallNow = () => (typeof performance !== "undefined" ? performance.now() : 0);

export function createWorkspace(start_yr: number): Workspace {
  // The one place the time in view, still kept in years (TP1 ledger; TP2 and TP3 move it to whole seconds), becomes the
  // picture clock's seconds since the campaign's epoch, with how finely a year that size can say it.
  const sinceEpoch = () => {
    const [yr, epoch] = [ws.now_yr, ws.campaign_epoch_yr ?? ws.now_yr];
    return { s: (yr - epoch) * YEAR_SECONDS, grain_s: Math.abs(yr) * Number.EPSILON * 4 * YEAR_SECONDS };
  };
  const follow = () => {
    const t = sinceEpoch();
    return followTime(untrack(() => ws.clock) ?? stoppedAt(t.s), t.s, t.grain_s, wallNow());
  };
  const ws: Workspace = $state({
    selection: null,
    now_yr: start_yr,
    campaign_now_yr: start_yr,
    campaign_epoch_yr: start_yr,
    clock: stoppedAt(0),
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
      ws.clock = follow();
    },
    followClock() {
      const c = follow();
      untrack(() => (ws.clock = c));
    },
    setRate(rate: number) {
      ws.clock = withRate(untrack(follow), rate, wallNow());
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
