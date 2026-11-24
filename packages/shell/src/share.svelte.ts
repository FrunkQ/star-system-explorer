// Sharing a session with players (V4): the GM shell records every applied
// intent from the moment the campaign opens, and serves players once the GM
// shares. A room is a random id; the players' link opens players.html with it.
// The GM serves both at once: BroadcastChannel for other windows and screens
// of this computer, and the site's relay (worker/index.js) for other devices.
import { GmHost, SealedHost, sealedLink, observeOf, addSeat, broadcastHub, defaultPlan, discoverIntent, joinLink, multiHub, relayHub, removeSeat, startIntents, type Audience, type HostState, type Hub, type Json, type PlayerRequest, type SeatPlan, type Session, type Shown } from "../../ui-contract/src/index.ts";

declare const __SSE_APP__: string;
export const APP_VERSION: string = typeof __SSE_APP__ === "string" ? __SSE_APP__ : "dev";

export interface Share {
  readonly host: GmHost;
  readonly state: HostState & { room: string | null; link: string | null; waiting: PlayerRequest[]; plan: SeatPlan; shown: Shown };
  /** Share, showing the table `first` (the place in view when the GM shares). */
  start(first?: Shown): string;
  /** Send the table a place, or turn their screens off. Nothing reaches them until it is sent. */
  show(s: Shown): void;
  setClock(yr: number): void;
  /** V10: add a seat with its own audience; a fog-of-war audience's starting view is chronicled at once. */
  addSeat(o: Parameters<typeof addSeat>[1]): Promise<string | null>;
  removeSeat(id: string): void;
  /** Quick overrides (V3 gap 25): set a seat's live switches now; they go to its screens at once and are never saved. */
  setLive(id: string, live: Record<string, boolean>): void;
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

export async function createShare(session: Session, opening: { universe: { id: string; seed: string }; packs: unknown[]; opening_export?: Json; clock_yr: number; views: () => string[]; scope: string }, hubFor: (room: string, id: string) => Hub = (room, id) => multiHub(id, [broadcastHub(room, id), relayHub(document.baseURI, room, id)])): Promise<Share> {
  const run = roomId();
  const host = new GmHost(session, null, { run, kernel: await session.kernelVersion(), app: APP_VERSION, universe: opening.universe, packs: opening.packs, ...(opening.opening_export !== undefined ? { opening_export: opening.opening_export } : {}), opening_hash: await session.stateHash(), clock_yr: opening.clock_yr, seats: defaultPlan().seats });
  const st = $state({ ...host.state(), room: null as string | null, link: null as string | null, waiting: [] as PlayerRequest[], plan: defaultPlan(), shown: { scope: opening.scope } as Shown });
  const root = `u:${opening.universe.id}`;
  const failed = (r: Awaited<ReturnType<Session["send"]>>) => (r.status === "applied" ? null : r.status === "refused" ? r.reason : r.status === "error" ? r.message : "not applied");
  host.onChange = (s) => Object.assign(st, s, { waiting: [...host.requests()] });
  let timer: ReturnType<typeof setInterval> | null = null;
  // V10 part 2: each sealed seat has its own room, served finished scenes only.
  const sealed = new Map<string, { host: SealedHost; room: string }>();
  let clock = opening.clock_yr;
  function serveSealed() {
    if (!st.room) return;
    for (const seat of st.plan.seats) {
      if (!seat.sealed || sealed.has(seat.id)) continue;
      const room = roomId();
      sealed.set(seat.id, { room, host: new SealedHost(session, host, hubFor(room, "gm"), { seat: $state.snapshot(seat) as typeof seat, scope: "scope" in st.shown ? st.shown.scope : opening.scope, app: APP_VERSION, clock_yr: clock, data_views: opening.views() }) });
    }
  }
  return {
    host,
    state: st,
    start(first?: Shown) {
      if (st.link) return st.link;
      if (first) st.shown = first;
      host.setState({ shown: JSON.parse(JSON.stringify(st.shown)) as Shown });
      const room = roomId();
      host.attach(hubFor(room, "gm"));
      timer ??= setInterval(() => (host.beat(), sealed.forEach((x) => x.host.beat())), 2000);
      addEventListener("pagehide", () => (host.close(), sealed.forEach((x) => x.host.close())));
      // Data-view state: the players' screens may look through the campaign's data views.
      host.setState({ data_views: opening.views() });
      st.room = room;
      st.link = joinLink(document.baseURI, room);
      serveSealed();
      return st.link;
    },
    show(s: Shown) {
      st.shown = s;
      host.setState({ shown: s });
      if ("scope" in s) sealed.forEach((x) => void x.host.setScope(s.scope));
    },
    setClock(yr: number) {
      clock = yr;
      host.setState({ clock_yr: yr });
      sealed.forEach((x) => x.host.setClock(yr));
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
      serveSealed();
      return null;
    },
    setLive(id: string, live: Record<string, boolean>) {
      const plan = $state.snapshot(st.plan) as SeatPlan;
      const seats = plan.seats.map((s) => {
        if (s.id !== id) return s;
        const { live: _old, ...rest } = s;
        return Object.keys(live).length ? { ...rest, live: { ...live } } : rest;
      });
      st.plan = { ...plan, seats };
      host.setState({ seats });
    },
    removeSeat(id: string) {
      const next = removeSeat($state.snapshot(st.plan) as SeatPlan, id);
      sealed.get(id)?.host.close();
      sealed.delete(id);
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
      const own = sealed.get(seat.id);
      return screens.map((screen) => ({ screen, href: own ? sealedLink(document.baseURI, own.room, seat.id, screen ?? undefined) : joinLink(document.baseURI, st.room!, seat.id, screen ?? undefined) }));
    },
  };
}
