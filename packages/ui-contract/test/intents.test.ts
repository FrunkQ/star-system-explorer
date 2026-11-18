// U0, second check: an edit made through the GM seat produces exactly the
// event the same intent produces from a player seat with auto-approve, and a
// player seat without it changes nothing until the GM approves.
import { test } from "node:test";
import assert from "node:assert/strict";
import { Session } from "../src/session.ts";
import { gmSeat, tableSeat, type Seat } from "../src/seat.ts";
import type { Intent } from "../src/intent.ts";
import { nodeKernel } from "./node-kernel.ts";

const universe = { id: "kestrel", seed: "kestrel reach campaign" };
const tarn = "u:kestrel/sys:1/body:3";
const intent: Intent = { kind: "set_value", address: tarn, field: "mass_earth", value: 0.9, at_yr: 4_610_000_000 };

async function withSeat<T>(seat: Seat, f: (s: Session) => Promise<T>): Promise<T> {
  const k = nodeKernel();
  try {
    return await f(await Session.open(k.client, seat, universe));
  } finally {
    await k.close();
  }
}

test("GM edit and auto-approved player intent give the same event and state hash", async () => {
  const gm = await withSeat(gmSeat(), (s) => s.send(intent));
  const sensors: Seat = {
    ...tableSeat(),
    id: "sensors",
    label: "Sensors",
    may_send: ["set_value"],
    auto_approve: [{ intent: "set_value", scope: ["u:kestrel/sys:1"] }],
  };
  const player = await withSeat(sensors, (s) => s.send(intent));
  assert.equal(gm.status, "applied");
  assert.deepEqual(player, gm);
});

test("a player intent outside its auto-approve scope waits for the GM and changes nothing", async () => {
  const seat: Seat = { ...tableSeat(), may_send: ["set_value"], auto_approve: [{ intent: "set_value", scope: ["u:kestrel/sys:2"] }] };
  await withSeat(seat, async (s) => {
    const before = await s.stateHash();
    const r = await s.send(intent);
    assert.equal(r.status, "pending");
    assert.deepEqual(s.pending(), [intent]);
    assert.equal(await s.stateHash(), before);
    assert.deepEqual(await s.history(tarn), []);
  });
});

test("a seat that may not send an intent is refused; a malformed intent is refused", async () => {
  await withSeat(tableSeat(), async (s) => {
    assert.equal((await s.send(intent)).status, "refused");
  });
  await withSeat(gmSeat(), async (s) => {
    const r = await s.send({ ...intent, field: "Mass!" });
    assert.equal(r.status, "refused");
  });
});

test("applied edits come back in history at the ruler's time", async () => {
  await withSeat(gmSeat(), async (s) => {
    const r = await s.send(intent);
    assert.equal(r.status, "applied");
    const h = await s.history("u:kestrel/sys:1");
    assert.equal(h.length, 1);
    assert.equal(h[0]!.at_yr, 4_610_000_000);
    assert.equal(h[0]!.evidence, "authored");
    assert.equal(h[0]!.id, r.status === "applied" ? r.event_id : "");
  });
});
