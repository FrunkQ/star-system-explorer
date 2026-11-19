// The v4 site's Worker (H9.2): serves the built site from dist/ and relays
// signalling for player rooms. A room is one Durable Object that forwards
// messages between the sockets joined to it (the GM and the players' screens)
// and stores nothing: no campaign data lives on the server.
//
//   GET /signal/<room>?peer=<id>  (WebSocket)  join a room as <id>
//   Messages are JSON objects; the room adds `from`. One with `to` goes to that
//   peer only; others go to every other peer. When a peer leaves, the others
//   hear { t: "left", from }.

const ROOM = /^\/signal\/([a-z0-9]{4,40})$/;
const PEER = /^[a-z0-9-]{1,40}$/;
const MAX_PEERS = 32;
const MAX_BYTES = 4 * 1024 * 1024;

export default {
  async fetch(request, env) {
    const url = new URL(request.url);
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
