// The site Worker's relay room (worker/index.js) with a stand-in Durable
// Object state: messages reach every other peer, `to` reaches one, a leaver
// is announced, and nothing is stored. The end-to-end run through workerd is
// `wrangler dev` plus `node tests/ui/p2p.e2e.mjs --relay <url>` (V4.md).
import { test } from "node:test";
import assert from "node:assert/strict";
// @ts-expect-error plain JS Worker module
import worker, { SignalRoom } from "../../worker/index.js";

type Sock = { tag: string; sent: string[]; send(s: string): void; close(): void };
function fakeState() {
  const socks: Sock[] = [];
  return {
    socks,
    acceptWebSocket(ws: Sock, tags: string[]) {
      ws.tag = tags[0]!;
      socks.push(ws);
    },
    getWebSockets(tag?: string) {
      return socks.filter((s) => tag === undefined || s.tag === tag);
    },
    getTags(ws: Sock) {
      return [ws.tag];
    },
  };
}
const sock = (tag: string): Sock => ({ tag, sent: [], send(s) { this.sent.push(s); }, close() {} });

test("the relay room forwards, addresses and announces leavers", () => {
  const st = fakeState();
  const room = new SignalRoom(st);
  const [gm, a, b] = ["gm", "a", "b"].map((t) => {
    const s = sock(t);
    st.acceptWebSocket(s, [t]);
    return s;
  });
  room.webSocketMessage(a, JSON.stringify({ t: "hello", from: "spoofed" }));
  assert.deepEqual(JSON.parse(gm!.sent[0]!), { t: "hello", from: "a" }, "the room names the sender");
  assert.equal(b!.sent.length, 1);
  assert.equal(a!.sent.length, 0, "never echoed");
  room.webSocketMessage(gm, JSON.stringify({ t: "manifest", to: "a" }));
  assert.equal(a!.sent.length, 1);
  assert.equal(b!.sent.length, 1, "addressed to a only");
  room.webSocketMessage(gm, "not json");
  room.webSocketMessage(gm, "[1]");
  assert.equal(a!.sent.length, 1, "junk is dropped");
  room.webSocketClose(b);
  assert.deepEqual(JSON.parse(gm!.sent.at(-1)!), { t: "left", from: "b" });
});

test("the Worker serves the site and sends only well-formed room joins to rooms", async () => {
  const env = { ASSETS: { fetch: async () => new Response("site") }, ROOMS: { idFromName: (n: string) => n, get: () => ({ fetch: async () => new Response("room") }) } };
  assert.equal(await (await worker.fetch(new Request("https://x/"), env)).text(), "site");
  assert.equal((await worker.fetch(new Request("https://x/signal/abcd1234?peer=p1"), env)).status, 426);
  const ws = { headers: { Upgrade: "websocket" } };
  assert.equal((await worker.fetch(new Request("https://x/signal/abcd1234?peer=BAD!", ws), env)).status, 400);
  assert.equal(await (await worker.fetch(new Request("https://x/signal/abcd1234?peer=p1", ws), env)).text(), "room");
  assert.equal(await (await worker.fetch(new Request("https://x/signal/../etc", ws), env)).text(), "site");
});
