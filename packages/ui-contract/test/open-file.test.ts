// V3 gap 28: one Open door names every file by looking inside it; Explorers files are checked before opening.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { sniffFile, explorersUrl, fetchExplorers, sha256Hex, creditWords } from "../src/index.ts";

test("each kind of file is named by what is inside, whatever it is called", () => {
  const save = JSON.stringify({ schema: "sse.campaign-save/1", title: "Kestrel", saved_at: "2026-11-23T04:00:00Z", root: "u:x", export: { events: [1, 2, 3] } });
  assert.equal(sniffFile("anything.json", save).kind, "v4-campaign");
  assert.match(sniffFile("x", save).what, /Kestrel.*3 events/);
  assert.equal(sniffFile("clip.json", JSON.stringify({ sseClip: 2, nodes: [{}, {}] })).kind, "v4-clip");
  assert.equal(sniffFile("clip.json", JSON.stringify({ sseClip: 1 })).kind, "v3-clip");
  assert.equal(sniffFile("p.json", JSON.stringify({ format: "sse.pack/1", id: "warm", version: "1.0.0" })).kind, "pack");
  assert.equal(sniffFile("d.json", JSON.stringify({ kind: "sse4-diagnostic" })).kind, "diagnostic");
  const sol = readFileSync(new URL("../../shell/public/samples/synthetic-sol.json", import.meta.url), "utf8");
  assert.equal(sniffFile("starmap.json", sol).kind, "v3-save");
  assert.equal(sniffFile("world.ubox", null).kind, "other-app");
  assert.equal(sniffFile("sol.sc", null).kind, "other-app");
  assert.equal(sniffFile("old.sse.zip", null, new Uint8Array([0x50, 0x4b, 0x03, 0x04, 0])).kind, "v3-save");
  assert.equal(sniffFile("notes.txt", "hello").kind, "unknown");
  assert.equal(sniffFile("list.json", "[1,2]").kind, "unknown");
});

test("only Explorers' own V4 listings open from a link", () => {
  assert.ok(explorersUrl("https://explorers.starsystemx.com/api/v4/warm-worlds-1.0.0"));
  assert.equal(explorersUrl("http://explorers.starsystemx.com/api/v4/a"), null);
  assert.equal(explorersUrl("https://explorers.starsystemx.com.evil.example/api/v4/a"), null);
  assert.equal(explorersUrl("https://evil.example/api/v4/a"), null);
  assert.equal(explorersUrl("https://explorers.starsystemx.com/api/v3/a"), null);
  assert.equal(explorersUrl("https://explorers.starsystemx.com/api/v4/../admin"), null);
});

test("an Explorers file opens only when its bytes match the listing", async () => {
  const body = new TextEncoder().encode(JSON.stringify({ format: "sse.pack/1", id: "warm", version: "1.0.0" }));
  const sum = await sha256Hex(body);
  const listing = { sha256: sum, source: { site: "explorers", title: "Warm worlds", url: "https://explorers.starsystemx.com/i/warm", creditChain: ["Ann", "Bo"] }, pack: { id: "warm", version: "1.0.0" } };
  const fake = (l: unknown, bytes: Uint8Array, header: string | null, status = 200) =>
    (async (u: string) => (String(u).endsWith("/file") ? new Response(bytes as BodyInit, { headers: header ? { "x-sse-sha256": header } : {} }) : new Response(JSON.stringify(l), { status }))) as unknown as typeof fetch;
  const url = "https://explorers.starsystemx.com/api/v4/warm-1.0.0";
  const ok = await fetchExplorers(url, fake(listing, body, sum));
  assert.ok(ok.ok && ok.text.includes("warm"));
  const bad = await fetchExplorers(url, fake({ ...listing, sha256: "0".repeat(64) }, body, sum));
  assert.ok(!bad.ok && /sha256/.test(bad.why));
  const badHeader = await fetchExplorers(url, fake(listing, body, "f".repeat(64)));
  assert.ok(!badHeader.ok);
  const none = await fetchExplorers(url, fake({}, body, null, 404));
  assert.ok(!none.ok && none.noShelf);
  assert.equal(creditWords(listing.source.creditChain), "Made by Ann, then Bo");
});
