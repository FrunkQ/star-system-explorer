// Landing run-through (blocker 4, gap 9): the party's ship names its nav ship
// among the picker's rows, a voyage starts where it is (else at the GM's picked
// world), and Voyages' names sit in rows without overlapping.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { IN_SYSTEM_SHIPS, labelLanes } from "../../packages/ui-contract/src/voyage.ts";
import { shipChoices } from "../../packages/ui-contract/src/route.ts";
import { navShipOf, partyLander, voyageStart, voyageLogEntries, type PartyShip } from "../../packages/ui-contract/src/party.ts";
import { Session } from "../../packages/ui-contract/src/session.ts";
import { gmSeat, tableSeat } from "../../packages/ui-contract/src/seat.ts";
import type { PackDoc } from "../../packages/ui-contract/src/physics.ts";
import type { SystemVoyagePlan } from "../../packages/ui-contract/src/voyage.ts";
import { nodeKernel } from "../../packages/ui-contract/test/node-kernel.ts";

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

// V8 flight log: a flown voyage written into the party ship's log on the record, and read back where it ended.

test("a flown voyage goes into the ship's flight log: leaving, the legs, arriving, parked at the end", async () => {
  const base: PackDoc = JSON.parse(readFileSync(new URL("../../packs/base-physics/pack.json", import.meta.url), "utf8"));
  const nav = JSON.parse(readFileSync(new URL("../fixtures/nav/sol.json", import.meta.url), "utf8")) as { bodies: { address: string; name: string; radius_m: number }[] };
  const earth = nav.bodies.find((b) => b.name === "Earth")!.address;
  const mars = nav.bodies.find((b) => b.name === "Mars")!.address;
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, gmSeat(), { id: "sol", seed: "party log" }, [base], k.scratch);
    const r = await s.planVoyage({ nav, ship: "fusion-torch", from: earth, to: mars, depart_s: 63_115_200_000 });
    assert.ok(r.ok, JSON.stringify(r));
    const plan = r.plan as SystemVoyagePlan;
    const entries = voyageLogEntries(plan, nav.bodies);
    assert.equal(entries.length, 2 + plan.voyage!.legs.length + 1, "leave, each leg, arrive, park");
    const ship = "u:sol/construct:beowulf";
    for (const e of entries) {
      const w = await s.logFlight({ construct: ship, at_s: 0, ...e });
      assert.ok(w.ok, JSON.stringify(w));
    }
    const arrive = 63_115_200_000 + Math.round(plan.voyage!.home_days * 86_400);
    const l = await s.flightLog({ construct: ship, where_at: arrive + 86_400, nav });
    assert.ok(l.ok, JSON.stringify(l));
    const at = l.at as { kind: string; with?: string; words: string };
    assert.equal(at.kind, "orbit", at.words);
    assert.equal(at.with, mars);
    const mid = await s.flightLog({ construct: ship, where_at: arrive - 3600, nav });
    assert.ok(mid.ok);
    assert.equal((mid.at as { kind: string }).kind, "course");
    const p = await Session.open(k.client, tableSeat(), { id: "sol", seed: "party log" }, [base], k.scratch);
    const refused = await p.logFlight({ construct: ship, at_s: 0, name: "x" });
    assert.equal(refused.ok, false, "players do not write the log");
  } finally {
    await k.close();
  }
});
