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

test("sector data: only named reads pass, answers carry attribution and are cached", async () => {
  const { tmapUpstream, ATTRIBUTION } = await import("../../worker/index.js");
  const u = (p: string) => tmapUpstream(new URL(`https://v4.example${p}`));
  assert.equal(u("/tmap/sector?sector=Spinward Marches&subsector=C").url, "https://travellermap.com/api/sec?sector=Spinward+Marches&type=TabDelimited&subsector=C");
  assert.equal(u("/tmap/metadata?sector=Deneb").url, "https://travellermap.com/api/metadata?sector=Deneb");
  assert.match(u("/tmap/sectors").url, /api\/universe/);
  assert.ok(u("/tmap/sector?sector=../../etc").error);
  assert.ok(u("/tmap/sector?sector=Deneb&subsector=Q").error);
  assert.ok(u("/tmap/anything?url=https://evil.example").error);

  const calls: string[] = [];
  const store = new Map<string, Response>();
  const deps = {
    fetch: async (url: string) => {
      calls.push(url);
      return url.includes("Nowhere") ? new Response("", { status: 404 }) : new Response("Sector\tSS\tHex\n", { status: 200, headers: { "content-type": "text/plain" } });
    },
    cache: {
      match: async (r: Request) => store.get(r.url)?.clone(),
      put: async (r: Request, res: Response) => void store.set(r.url, res),
    },
  };
  const get = (p: string, method = "GET") => worker.fetch(new Request(`https://v4.example${p}`, { method }), {}, {}, deps);
  const a = await get("/tmap/sector?sector=Deneb");
  assert.equal(a.status, 200);
  assert.equal(a.headers.get("x-attribution"), ATTRIBUTION);
  assert.match(await a.text(), /^Sector/);
  await get("/tmap/sector?sector=Deneb");
  assert.equal(calls.length, 1, "the second read comes from the cache");
  assert.equal((await get("/tmap/sector?sector=Nowhere")).status, 404);
  assert.equal((await get("/tmap/sector?sector=Deneb", "POST")).status, 405);
  assert.equal((await get("/tmap/sector?sector=<script>")).status, 400);
});

test("the client asks the Worker by name and keeps the attribution", async () => {
  const { fetchSector, sectorPath } = await import("../../packages/ui-contract/src/sector-fetch.ts");
  assert.equal(sectorPath("Spinward Marches", "C"), "/tmap/sector?sector=Spinward+Marches&subsector=C");
  const deps = { fetch: async () => new Response("Sector\tHex\n", { status: 200 }), cache: undefined };
  const via = (u: string) => worker.fetch(new Request(`https://v4.example${u}`), {}, {}, deps);
  const s = await fetchSector(via, "Deneb", "B");
  assert.equal(s.source, "Deneb, subsector B");
  assert.match(s.attribution, /travellermap\.com/);
  const none = { fetch: async () => new Response("", { status: 404 }), cache: undefined };
  await assert.rejects(fetchSector((u: string) => worker.fetch(new Request(`https://v4.example${u}`), {}, {}, none), "Nowhere"), /no sector called Nowhere/);
});
