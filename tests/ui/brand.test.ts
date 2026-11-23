// Branding and join words on player views (V3 gap 25): a seat's brand is
// checked (name, raster logo only, size), and join words come from the
// stacked packs' vocabulary, topmost first, line by line.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { DEFAULT_JOIN, checkBrand, joinLine, joinWordsOf } from "../../packages/ui-contract/src/brand.ts";
import { addSeat, checkSeats, defaultPlan } from "../../packages/ui-contract/src/seats.ts";

const root = fileURLToPath(new URL("../../", import.meta.url));
const traveller = JSON.parse(readFileSync(root + "packs/traveller/pack.json", "utf8"));

test("a brand is a name and, at most, a small raster logo", () => {
  const png = "data:image/png;base64,iVBORw0KGgo=";
  assert.deepEqual(checkBrand({ name: "Vesh Navy", logo: png }), []);
  assert.equal(checkBrand({ name: " " }).length, 1);
  assert.equal(checkBrand({ name: "x".repeat(41) }).length, 1);
  assert.equal(checkBrand({ name: "V", logo: "data:image/svg+xml;base64,PHN2Zz4=" }).length, 1, "no SVG: it can carry script");
  assert.equal(checkBrand({ name: "V", logo: "https://example.com/logo.png" }).length, 1, "no outside addresses");
  assert.equal(checkBrand({ name: "V", logo: "data:image/png;base64," + "A".repeat(90_001) }).length, 1);
  const p = addSeat(defaultPlan(), { label: "Vesh", brand: { name: "Vesh Navy", logo: png } });
  assert.deepEqual(p.seats.find((s) => s.label === "Vesh")!.brand, { name: "Vesh Navy", logo: png });
  assert.deepEqual(checkSeats(p), []);
  const bad = { ...p, seats: p.seats.map((s) => (s.label === "Vesh" ? { ...s, brand: { name: "Vesh", logo: "data:text/html;base64,PGI+" } } : s)) };
  assert.equal(checkSeats(bad).length, 1);
});

test("join words come from the stacked packs' vocabulary, topmost first, line by line", () => {
  assert.deepEqual(joinWordsOf([]), DEFAULT_JOIN);
  const t = joinWordsOf([traveller]);
  assert.notEqual(t.joining, DEFAULT_JOIN.joining);
  assert.equal(joinLine(t.joining, { seat: "Bridge", brand: "Vesh Navy" }), "Calling the ship's computer for Vesh Navy…");
  assert.equal(joinLine(t.joining, { seat: "Bridge" }), "Calling the ship's computer for Bridge…");
  const top = { provides: { other: { vocabulary: { join: { waiting: "Stand by." } } } } };
  const both = joinWordsOf([traveller, top]);
  assert.equal(both.waiting, "Stand by.");
  assert.equal(both.joining, t.joining, "a line the top pack leaves out keeps the one below");
});

test("quick overrides lay a seat's live switches over a screen's parameters, only where the view has them", async () => {
  const { withLive, LIVE_OVERRIDES } = await import("../../packages/ui-contract/src/brand.ts");
  const declared = [{ name: "labels" }, { name: "mark_scale" }];
  assert.deepEqual(withLive({ labels: true, mark_scale: 2 }, declared, { labels: false }), { labels: false, mark_scale: 2 });
  assert.deepEqual(withLive({ fields: [] }, [{ name: "fields" }], { labels: false }), { fields: [] }, "a card has no labels switch");
  assert.deepEqual(withLive({ labels: true }, declared, undefined), { labels: true });
  assert.ok(LIVE_OVERRIDES.some((o) => o.param === "labels" && o.on === false));
});
