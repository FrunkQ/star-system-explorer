// Sharing a session with players (V4): the GM shell records every applied
// intent from the moment the campaign opens, and serves players once the GM
// shares. A room is a random id; the players' link opens players.html with it.
// The GM serves both at once: BroadcastChannel for other windows and screens
// of this computer, and the site's relay (worker/index.js) for other devices.
import { GmHost, observeOf, addSeat, broadcastHub, defaultPlan, discoverIntent, joinLink, multiHub, relayHub, removeSeat, startIntents, type Audience, type HostState, type Hub, type PlayerRequest, type SeatPlan, type Session } from "../../ui-contract/src/index.ts";

declare const __SSE_APP__: string;
export const APP_VERSION: string = typeof __SSE_APP__ === "string" ? __SSE_APP__ : "dev";

export interface Share {
  readonly host: GmHost;
  readonly state: HostState & { room: string | null; link: string | null; waiting: PlayerRequest[]; plan: SeatPlan };
  start(): string;
  setClock(yr: number): void;
  /** V10: add a seat with its own audience; a fog-of-war audience's starting view is chronicled at once. */
  addSeat(o: Parameters<typeof addSeat>[1]): Promise<string | null>;
  removeSeat(id: string): void;
  /** Mark a body (and what orbits it) discovered by an audience, or take it back. */
  discover(audience: Audience, address: string, known: boolean, at_yr: number): Promise<string | null>;
  /** Join links of a seat: one per screen it opens. */
  links(seatId: string): { screen: string | null; href: string }[];
}

function roomId(): string {
  const b = new Uint8Array(6);
  crypto.getRandomValues(b);
  return [...b].map((x) => x.toString(36).padStart(2, "0")).join("").slice(0, 10);
}

export async function createShare(session: Session, opening: { universe: { id: string; seed: string }; packs: unknown[]; clock_yr: number; views: () => string[] }, hubFor: (room: string, id: string) => Hub = (room, id) => multiHub(id, [broadcastHub(room, id), relayHub(document.baseURI, room, id)])): Promise<Share> {
  const run = roomId();
  const host = new GmHost(session, null, { run, kernel: await session.kernelVersion(), app: APP_VERSION, universe: opening.universe, packs: opening.packs, opening_hash: await session.stateHash(), clock_yr: opening.clock_yr, seats: defaultPlan().seats });
  const st = $state({ ...host.state(), room: null as string | null, link: null as string | null, waiting: [] as PlayerRequest[], plan: defaultPlan() });
  const root = `u:${opening.universe.id}`;
  const failed = (r: Awaited<ReturnType<Session["send"]>>) => (r.status === "applied" ? null : r.status === "refused" ? r.reason : r.status === "error" ? r.message : "not applied");
  host.onChange = (s) => Object.assign(st, s, { waiting: [...host.requests()] });
  let timer: ReturnType<typeof setInterval> | null = null;
  return {
    host,
    state: st,
    start() {
      if (st.link) return st.link;
      const room = roomId();
      host.attach(hubFor(room, "gm"));
      timer ??= setInterval(() => host.beat(), 2000);
      addEventListener("pagehide", () => host.close());
      // Data-view state: the players' screens may look through the campaign's data views.
      host.setState({ data_views: opening.views() });
      st.room = room;
      st.link = joinLink(document.baseURI, room);
      return st.link;
    },
    setClock(yr: number) {
      host.setState({ clock_yr: yr });
    },
    async addSeat(o) {
      const next = addSeat($state.snapshot(st.plan) as SeatPlan, o);
      const added = next.audiences.filter((a) => !st.plan.audiences.some((b) => b.id === a.id));
      for (const i of startIntents({ ...next, audiences: added }, root, opening.clock_yr)) {
        const bad = failed(await session.send(i));
        if (bad) return bad;
      }
      st.plan = next;
      host.setState({ seats: next.seats });
      return null;
    },
    removeSeat(id: string) {
      const next = removeSeat($state.snapshot(st.plan) as SeatPlan, id);
      st.plan = next;
      host.setState({ seats: next.seats });
    },
    async discover(audience, address, known, at_yr) {
      return failed(await session.send(discoverIntent(audience, address, known, observeOf(session.stack().base)?.setup.campaign.before ?? "long-range", at_yr)));
    },
    links(seatId: string) {
      const seat = st.plan.seats.find((x) => x.id === seatId);
      if (!st.room || !seat) return [];
      const screens = seat.screens?.length ? seat.screens : [null];
      return screens.map((screen) => ({ screen, href: joinLink(document.baseURI, st.room!, seat.id, screen ?? undefined) }));
    },
  };
}
