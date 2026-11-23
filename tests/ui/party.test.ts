// Landing run-through (blocker 4, gap 9): the party's ship names its nav ship
// among the picker's rows, a voyage starts where it is (else at the GM's picked
// world), and Voyages' names sit in rows without overlapping.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { IN_SYSTEM_SHIPS, labelLanes } from "../../packages/ui-contract/src/voyage.ts";
import { shipChoices } from "../../packages/ui-contract/src/route.ts";
import { navShipOf, partyLander, voyageStart, type PartyShip } from "../../packages/ui-contract/src/party.ts";

const TRAVELLER = JSON.parse(readFileSync(new URL("../../packs/traveller/pack.json", import.meta.url), "utf8"));

test("a construct's nav ship: a pack ship whole, a stock ship by id, else none", () => {
  const rows = shipChoices(IN_SYSTEM_SHIPS, [TRAVELLER]);
  const far = navShipOf("traveller.far_trader", rows);
  assert.equal(typeof far, "object");
  assert.equal((far as { name: string }).name.length > 0, true);
  assert.equal(navShipOf("fusion-torch", rows), "fusion-torch");
  assert.equal(navShipOf("no-such", rows), null);
  assert.equal(navShipOf(undefined, rows), null);
  const p: PartyShip = { ship: "u:k/sys:sun/body:earth/ship:1", name: "Beowulf", nav: far!, at: "u:k/sys:sun/body:earth" };
  assert.deepEqual(partyLander(p), { ship: far });
});

test("a voyage starts at the party's ship, else the picked world (or the world a moon is at), else the fallback", () => {
  const places = ["u:k/sys:sun/body:mercury", "u:k/sys:sun/body:earth", "u:k/sys:sun/body:mars"];
  const party: PartyShip = { ship: "u:k/sys:sun/body:mars/ship:1", name: "B", nav: "fusion-torch", at: places[2]! };
  assert.equal(voyageStart(places, party, places[1], places[0]!), places[2]);
  assert.equal(voyageStart(places, null, places[1], places[0]!), places[1]);
  assert.equal(voyageStart(places, null, "u:k/sys:sun/body:earth/body:moon", places[0]!), places[1]);
  assert.equal(voyageStart(places, { ...party, at: "u:k/sys:other" }, null, places[0]!), places[0]);
  assert.equal(voyageStart(places, null, "u:k/sys:sun/body:earthling", places[0]!), places[0]);
});

test("names along the axis never overlap; the start and the goal always get one", () => {
  const items = [
    { x: 100, width: 40 },
    { x: 110, width: 40 },
    { x: 120, width: 40, first: true },
    { x: 130, width: 40 },
    { x: 300, width: 40 },
  ];
  const lanes = labelLanes(items, 2);
  assert.equal(lanes[2], 0);
  assert.equal(lanes[4], 0);
  for (let i = 0; i < items.length; i++)
    for (let j = i + 1; j < items.length; j++) {
      if (lanes[i] === null || lanes[i] !== lanes[j]) continue;
      const [a, b] = [items[i]!, items[j]!];
      assert.ok(Math.abs(a.x - b.x) >= (a.width + b.width) / 2 + 6, `${i} and ${j} overlap`);
    }
  assert.ok(lanes.includes(null), "one too many for two rows is drawn only on hover");
});
