// A visiting universe for postcards from elsewhere (PC4 part 2, design section 4): a second kernel, started on the
// first visit that needs one, holding nothing of this page's campaign. A postcard that carries its world as a clip is
// laid into a system of its own there, as authored facts, and the physics works the rest out again; its front is then
// drawn from that system exactly as a campaign's own postcard is drawn from the campaign. Opening one never changes
// the campaign: the campaign's kernel is never written to.
import { clipAsSystem, gmSeat, type Json, type Postcard, type Session, type ViewSession } from "../../../ui-contract/src/index.ts";
import { openBrowserSession } from "../../../ui-contract/src/host-browser.ts";

/** The visiting universe: one for the page, never kept, never shared. */
export const VISIT_UNIVERSE = { id: "visit", seed: "postcard visits" };

/** A postcard laid into the visiting universe: the session it lives in, its system and the camera's body there. */
export interface Laid {
  session: ViewSession;
  system: string;
  about: string;
  kind: string;
  label: string;
}

// One kernel per universe visited (the visiting universe, or a postcard's own), for the page's life.
const kernels = new Map<string, Promise<Session>>();
const laid = new Map<string, Promise<Laid | string>>();
let next = 1;

function visitingSession(universe: { id: string; seed: string }, packs: readonly unknown[]): Promise<Session> {
  const key = JSON.stringify([universe.id, universe.seed]);
  let s = kernels.get(key);
  if (!s) {
    s = openBrowserSession({ base: document.baseURI, seat: gmSeat(), universe, packs: [...packs] });
    kernels.set(key, s);
    s.catch(() => kernels.delete(key));
  }
  return s;
}

async function send(s: Session, edits: readonly { address: string; field: string; value: Json; impossible?: boolean }[]): Promise<string | null> {
  const possible = edits.filter((e) => !e.impossible);
  const impossible = edits.filter((e) => e.impossible);
  for (const batch of [possible, impossible].filter((b) => b.length)) {
    const r = await s.send({ kind: "set_values", edits: batch.map((e) => ({ ...e })), at_yr: 0, evidence: "authored" });
    if (r.status !== "applied") return `Its world could not be laid out here: ${r.status === "refused" ? r.reason : r.status === "error" ? r.message : "it waits for approval"}.`;
  }
  return null;
}

/** Lay a postcard's clip into the visiting universe (once per postcard for the page's life). A string says why not. */
export function layVisit(p: Postcard, packs: readonly unknown[]): Promise<Laid | string> {
  const w = p.world;
  if (w.how !== "clip") return Promise.resolve("Only a postcard that carries its world can be visited for now.");
  const hit = laid.get(p.id);
  if (hit) return hit;
  const job = (async (): Promise<Laid | string> => {
    const node = w.about ?? w.clip.nodes.find((n) => n.kind === "planet" || n.kind === "moon")?.id ?? w.clip.root;
    const n = w.clip.nodes.find((x) => x.id === node);
    const name = w.clip.events.find((e) => e.node === node && e.field === "name")?.value;
    const label = typeof name === "string" ? name : (n?.kind ?? "world");
    const o = w.origin;
    let s: Session;
    try {
      s = await visitingSession(o ? o.universe : VISIT_UNIVERSE, packs);
    } catch (e) {
      return `The visiting kernel did not start: ${e instanceof Error ? e.message : String(e)}`;
    }
    // Made with its origin: the same universe and addresses, so the world is the same world, ground and all.
    if (o && o.nodes[node]) {
      const why = await send(s, o.facts);
      if (why) return why;
      return { session: s, system: o.system, about: o.nodes[node]!, kind: n?.kind ?? "planet", label };
    }
    // Without one, the clip is laid into a system of its own and the physics works the rest out again.
    if (!w.age_s) return "It does not say how old its star is, so its worlds cannot be worked out again here.";
    const system = `u:${VISIT_UNIVERSE.id}/sys:${next++}`;
    const rs = await s.resolvedSystem(system);
    const taken = rs.ok ? Object.keys(rs.result.facts) : [];
    const { edits, addressOf } = clipAsSystem(w.clip, system, taken, w.age_s);
    const why = await send(s, edits);
    if (why) return why;
    const about = addressOf.get(node);
    if (!about || !n) return "It does not say which world it shows.";
    return { session: s, system, about, kind: n.kind, label };
  })();
  laid.set(p.id, job);
  job.then((r) => typeof r === "string" && laid.delete(p.id));
  return job;
}
