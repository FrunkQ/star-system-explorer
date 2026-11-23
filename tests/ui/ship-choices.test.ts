// V8: the ship pickers list the stock ships, then every ship a stacked pack lists (the Traveller pack's standard
// ships first). A stock ship goes to the kernel by its id, a pack ship whole.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { STOCK_SHIPS, packDriveWords, shipChoices, shipToSend } from "../../packages/ui-contract/src/route.ts";
import { VOYAGE_SHIPS } from "../../packages/ui-contract/src/voyage.ts";

const TRAVELLER = JSON.parse(readFileSync(new URL("../../packs/traveller/pack.json", import.meta.url), "utf8"));

test("stock ships first, then the pack's, each sent the right way", () => {
  const rows = shipChoices(VOYAGE_SHIPS, [TRAVELLER]);
  assert.equal(rows.length, VOYAGE_SHIPS.length + 18);
  assert.deepEqual(rows.slice(0, VOYAGE_SHIPS.length).map((r) => r.id), VOYAGE_SHIPS.map((s) => s.id));
  assert.equal(shipToSend(rows[0]), "jump-courier");
  const courier = rows.find((r) => r.id === "traveller.courier")!;
  assert.equal(courier.pack, "sse.traveller");
  assert.equal(typeof shipToSend(courier), "object");
  assert.match(courier.drive, /^Jump-2, a pack rule \(fictional\); 2 g sustained/);
});

test("no packs leaves the stock list; a pack ship with a stock id replaces it", () => {
  assert.deepEqual(shipChoices(STOCK_SHIPS, []).map((r) => r.id), STOCK_SHIPS.map((s) => s.id));
  const pack = { id: "p", provides: { other: { nav_ships: { ships: { "fusion-torch": { name: "Our torch", accel_g: 3, sustained: true, exhaust_km_s: 1000, wet_t: 10, dry_t: 5 } } } } } };
  const rows = shipChoices(STOCK_SHIPS, [pack]);
  assert.equal(rows.length, STOCK_SHIPS.length);
  assert.equal(rows.find((r) => r.id === "fusion-torch")!.name, "Our torch");
  assert.equal(packDriveWords({ name: "x", accel_g: 0.5, sustained: false, exhaust_km_s: 4, wet_t: 2, dry_t: 1 }), "0.5 g");
});
