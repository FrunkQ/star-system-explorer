// Sharing a session with players (V4): the GM shell records every applied
// intent from the moment the campaign opens, and serves players once the GM
// shares. A room is a random id; the players' link opens players.html with it.
// The GM serves both at once: BroadcastChannel for other windows and screens
// of this computer, and the site's relay (worker/index.js) for other devices.
import { GmHost, broadcastHub, multiHub, relayHub, sharedTableSeat, type HostState, type Hub, type PlayerRequest, type Session } from "../../ui-contract/src/index.ts";

declare const __SSE_APP__: string;
export const APP_VERSION: string = typeof __SSE_APP__ === "string" ? __SSE_APP__ : "dev";

export interface Share {
  readonly host: GmHost;
  readonly state: HostState & { room: string | null; link: string | null; waiting: PlayerRequest[] };
  start(): string;
  setClock(yr: number): void;
}

function roomId(): string {
  const b = new Uint8Array(6);
  crypto.getRandomValues(b);
  return [...b].map((x) => x.toString(36).padStart(2, "0")).join("").slice(0, 10);
}

export async function createShare(session: Session, opening: { universe: { id: string; seed: string }; packs: unknown[]; clock_yr: number; views: () => string[] }, hubFor: (room: string, id: string) => Hub = (room, id) => multiHub(id, [broadcastHub(room, id), relayHub(document.baseURI, room, id)])): Promise<Share> {
  const run = roomId();
  const host = new GmHost(session, null, { run, kernel: await session.kernelVersion(), app: APP_VERSION, universe: opening.universe, packs: opening.packs, opening_hash: await session.stateHash(), clock_yr: opening.clock_yr, seats: [sharedTableSeat()] });
  const st = $state({ ...host.state(), room: null as string | null, link: null as string | null, waiting: [] as PlayerRequest[] });
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
      st.link = new URL(`players.html?join=${room}`, document.baseURI).href;
      return st.link;
    },
    setClock(yr: number) {
      host.setState({ clock_yr: yr });
    },
  };
}
