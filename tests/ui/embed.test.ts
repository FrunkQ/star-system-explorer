// A player view inside a virtual tabletop (V3 gap 26): `?embed=1`, only for
// tabletops the GM listed on the seat, matched exactly; the bridge talks only
// to that tabletop and hears only from it.
import { test } from "node:test";
import assert from "node:assert/strict";
import { EmbedBridge, embedAsk, embedAsked, embedMode, embedOrigin, embedOrigins, framingOrigin } from "../../packages/ui-contract/src/embed.ts";
import { addSeat, checkSeats, defaultPlan } from "../../packages/ui-contract/src/seats.ts";

test("a tabletop address is an exact origin, or says why not", () => {
  assert.deepEqual(embedOrigin("https://vtt.example.com"), { ok: true, origin: "https://vtt.example.com" });
  assert.deepEqual(embedOrigin(" https://VTT.example.com:8443/ "), { ok: true, origin: "https://vtt.example.com:8443" });
  assert.deepEqual(embedOrigin("http://localhost:30000"), { ok: true, origin: "http://localhost:30000" });
  for (const bad of ["http://vtt.example.com", "https://*.example.com", "https://vtt.example.com/game/1", "https://a:b@vtt.example.com", "vtt.example.com", "javascript:alert(1)", ""]) {
    assert.equal(embedOrigin(bad).ok, false, bad);
  }
  const r = embedOrigins(["https://a.example", "", "https://a.example", "ftp://b"]);
  assert.deepEqual(r.origins, ["https://a.example"]);
  assert.equal(r.problems.length, 1);
});

test("only an allowed tabletop shows the view; anything else is refused with a reason", () => {
  const ok = ["https://vtt.example.com"];
  assert.equal(embedAsked("?join=abcd&embed=1"), true);
  assert.equal(embedAsked("?join=abcd"), false);
  assert.deepEqual(embedMode(false, "https://evil.example", ok), { mode: "page" });
  assert.deepEqual(embedMode(true, "unframed", ok), { mode: "page" });
  assert.deepEqual(embedMode(true, "https://vtt.example.com", ok), { mode: "embedded", origin: "https://vtt.example.com" });
  for (const [framing, allowed] of [["https://vtt.example.com.evil.example", ok], ["https://evil.example", ok], [null, ok], ["https://vtt.example.com", undefined], ["https://vtt.example.com", []]] as const) {
    const m = embedMode(true, framing, allowed as string[] | undefined);
    assert.equal(m.mode, "refused", String(framing));
    assert.ok(m.mode === "refused" && m.reason.length > 20);
  }
  const top = {};
  assert.equal(framingOrigin({ self: top, top, location: {} }, ""), "unframed");
  assert.equal(framingOrigin({ self: {}, top, location: { ancestorOrigins: { length: 1, 0: "https://vtt.example.com" } } }, "https://other.example/x"), "https://vtt.example.com");
  assert.equal(framingOrigin({ self: {}, top, location: {} }, "https://vtt.example.com/game"), "https://vtt.example.com");
  assert.equal(framingOrigin({ self: {}, top, location: {} }, ""), null);
});

test("the bridge posts only to the allowed origin and hears only the framing tabletop", () => {
  const sent: [unknown, string][] = [];
  const parent = { postMessage: (m: unknown, o: string) => void sent.push([m, o]) };
  let hear: ((e: { origin: string; source: unknown; data: unknown }) => void) | null = null;
  const b = new EmbedBridge("https://vtt.example.com", parent, (f) => ((hear = f), () => (hear = null)));
  b.send({ source: "sse4", v: 1, type: "ready", seat: "Crew", audience: "everyone" });
  assert.deepEqual(sent.map((x) => x[1]), ["https://vtt.example.com"]);
  const asks: string[] = [];
  b.onAsk((a) => asks.push(a.address));
  const focus = { source: "vtt", type: "focus", address: "u:k/sys:1/body:3" };
  hear!({ origin: "https://vtt.example.com", source: parent, data: focus });
  hear!({ origin: "https://evil.example", source: parent, data: focus });
  hear!({ origin: "https://vtt.example.com", source: {}, data: focus });
  hear!({ origin: "https://vtt.example.com", source: parent, data: { source: "vtt", type: "set_value", address: "x" } });
  assert.deepEqual(asks, ["u:k/sys:1/body:3"]);
  b.close();
  assert.equal(hear, null);
  assert.equal(embedAsk("focus"), null);
});

test("a seat carries its tabletops, and the seat check holds them to exact origins", () => {
  const p = addSeat(defaultPlan(), { label: "Crew", embedOrigins: ["https://vtt.example.com"] });
  const seat = p.seats.find((s) => s.label === "Crew")!;
  assert.deepEqual(seat.embed_origins, ["https://vtt.example.com"]);
  assert.deepEqual(checkSeats(p), []);
  const bad = { ...p, seats: p.seats.map((s) => (s.label === "Crew" ? { ...s, embed_origins: ["https://*.example.com", "https://VTT.example.com"] } : s)) };
  assert.equal(checkSeats(bad).length, 2);
  assert.equal(addSeat(defaultPlan(), { label: "Wall" }).seats.find((s) => s.label === "Wall")!.embed_origins, undefined);
});
