// The v4 site's Worker (H9.2): serves the built site from dist/ and relays
// signalling for player rooms. A room is one Durable Object that forwards
// messages between the sockets joined to it (the GM and the players' screens)
// and stores nothing: no campaign data lives on the server.
//
//   GET /signal/<room>?peer=<id>  (WebSocket)  join a room as <id>
//   Messages are JSON objects; the room adds `from`. One with `to` goes to that
//   peer only; others go to every other peer. When a peer leaves, the others
//   hear { t: "left", from }.
//
// It also fetches community sector data for the Traveller pack's importer
// (handoff 7A part 4) on the user's request, so browsers need no cross-origin
// access, and caches it for a day. Only these reads, by name, are passed on:
//
//   GET /tmap/sectors                         the sector list (JSON)
//   GET /tmap/sector?sector=<name>[&subsector=<A-P>]   T5 tab-delimited worlds
//   GET /tmap/metadata?sector=<name>          sector metadata (XML)
//
// Every answer carries the data's attribution. No campaign data is sent.

const ROOM = /^\/signal\/([a-z0-9]{4,40})$/;
const PEER = /^[a-z0-9-]{1,40}$/;
const MAX_PEERS = 32;
const MAX_BYTES = 4 * 1024 * 1024;

const TMAP = "https://travellermap.com";
const SECTOR = /^[A-Za-z0-9 '\-.]{1,40}$/;
const SUBSECTOR = /^[A-P]$/;
const DAY_S = 86400;
export const ATTRIBUTION = "Sector data from the Traveller Map (travellermap.com); see that site for its terms of use and the trademark notice.";

/** The upstream URL for a /tmap/ request, or an error. */
export function tmapUpstream(url) {
  const q = url.searchParams;
  const sector = q.get("sector") ?? "";
  switch (url.pathname) {
    case "/tmap/sectors":
      return { url: `${TMAP}/api/universe?requireData=1` };
    case "/tmap/sector": {
      if (!SECTOR.test(sector)) return { error: "a sector name is letters, digits, spaces and ' - . only" };
      const sub = q.get("subsector");
      if (sub !== null && !SUBSECTOR.test(sub)) return { error: "a subsector is one letter A to P" };
      const u = new URL(`${TMAP}/api/sec`);
      u.searchParams.set("sector", sector);
      u.searchParams.set("type", "TabDelimited");
      if (sub) u.searchParams.set("subsector", sub);
      return { url: u.toString() };
    }
    case "/tmap/metadata": {
      if (!SECTOR.test(sector)) return { error: "a sector name is letters, digits, spaces and ' - . only" };
      const u = new URL(`${TMAP}/api/metadata`);
      u.searchParams.set("sector", sector);
      return { url: u.toString() };
    }
    default:
      return { error: "unknown sector data request" };
  }
}

async function tmap(request, deps) {
  const plain = (status, text) =>
    new Response(text, { status, headers: { "content-type": "text/plain; charset=utf-8", "x-attribution": ATTRIBUTION } });
  if (request.method !== "GET") return plain(405, "GET only");
  const up = tmapUpstream(new URL(request.url));
  if (up.error) return plain(400, up.error);
  const cache = deps.cache;
  const key = new Request(up.url);
  const hit = cache ? await cache.match(key) : undefined;
  if (hit) return hit;
  let res;
  try {
    res = await deps.fetch(up.url, { headers: { "user-agent": "StarSystemExplorer-v4", accept: "*/*" } });
  } catch {
    return plain(502, "the Traveller Map could not be reached");
  }
  if (!res.ok) return plain(res.status === 404 ? 404 : 502, res.status === 404 ? "no such sector" : `the Traveller Map answered ${res.status}`);
  const out = new Response(await res.text(), {
    status: 200,
    headers: {
      "content-type": res.headers.get("content-type") ?? "text/plain; charset=utf-8",
      "cache-control": `public, max-age=${DAY_S}`,
      "x-attribution": ATTRIBUTION,
    },
  });
  if (cache) await cache.put(key, out.clone());
  return out;
}

export default {
  async fetch(request, env, ctx, deps) {
    const url = new URL(request.url);
    if (url.pathname.startsWith("/tmap/")) {
      return tmap(request, deps ?? { fetch: (u, o) => fetch(u, o), cache: typeof caches !== "undefined" ? caches.default : undefined });
    }
    const m = ROOM.exec(url.pathname);
    if (!m) return env.ASSETS.fetch(request);
    if (request.headers.get("Upgrade") !== "websocket") return new Response("expected a WebSocket", { status: 426 });
    const peer = url.searchParams.get("peer") ?? "";
    if (!PEER.test(peer)) return new Response("bad peer id", { status: 400 });
    return env.ROOMS.get(env.ROOMS.idFromName(m[1])).fetch(request);
  },
};

export class SignalRoom {
  constructor(state) {
    this.state = state;
  }

  async fetch(request) {
    const peer = new URL(request.url).searchParams.get("peer");
    const sockets = this.state.getWebSockets();
    if (sockets.length >= MAX_PEERS) return new Response("room is full", { status: 429 });
    // A peer that reconnects replaces its old socket.
    for (const old of this.state.getWebSockets(peer)) old.close(1000, "replaced");
    const pair = new WebSocketPair();
    this.state.acceptWebSocket(pair[1], [peer]);
    return new Response(null, { status: 101, webSocket: pair[0] });
  }

  webSocketMessage(ws, data) {
    if (typeof data !== "string" || data.length > MAX_BYTES) return;
    let msg;
    try {
      msg = JSON.parse(data);
    } catch {
      return;
    }
    if (!msg || typeof msg !== "object" || Array.isArray(msg)) return;
    const from = this.state.getTags(ws)[0];
    const out = JSON.stringify({ ...msg, from });
    const to = typeof msg.to === "string" ? msg.to : null;
    for (const other of to ? this.state.getWebSockets(to) : this.state.getWebSockets()) {
      if (other === ws) continue;
      try {
        other.send(out);
      } catch {
        // a closing socket; its close handler tells the room
      }
    }
  }

  webSocketClose(ws) {
    const from = this.state.getTags(ws)[0];
    const out = JSON.stringify({ t: "left", from });
    for (const other of this.state.getWebSockets()) if (other !== ws) try { other.send(out); } catch {}
  }

  webSocketError(ws) {
    this.webSocketClose(ws);
  }
}
