// Reveal on arrival: the offer when the party's ship comes to a world, and the set_known its press sends.
import { test } from "node:test";
import assert from "node:assert/strict";
import { arrivalOffer, arrivalReveal } from "../src/arrival.ts";
import { checkIntent } from "../src/intent.ts";
import type { PartyShip } from "../src/party.ts";
import type { KnownAudience } from "../src/known.ts";

const earth = "u:kestrel/sys:sol/body:3";
const mars = "u:kestrel/sys:sol/body:4";
const ship = (at: string): PartyShip => ({ ship: "u:kestrel/sys:sol/construct:1", name: "Beowulf", nav: "far-trader", at });
const row = (landmark: KnownAudience["landmark"], top: boolean): KnownAudience => ({ audience: "everyone", label: "The table", rung: landmark, rung_label: landmark, landmark, top } as KnownAudience);

test("arriving at a world the table has only heard of offers to tell them", () => {
  const o = arrivalOffer(ship(earth), ship(mars), "Mars", row("heard_of", false));
  assert.deepEqual(o, { world: mars, line: "Beowulf is at Mars. Tell the table about it?", action: "Tell the table" });
});

test("no offer when the ship has not moved, is at a whole system, or the table knows the world", () => {
  assert.equal(arrivalOffer(ship(mars), ship(mars), "Mars", null), null);
  assert.equal(arrivalOffer(ship(earth), ship("u:kestrel/sys:sol"), "Sol", null), null);
  assert.equal(arrivalOffer(ship(earth), ship(mars), "Mars", row("known", true)), null);
  assert.equal(arrivalOffer(ship(earth), null, "Mars", null), null);
});

test("known short of the top rung still offers; a first party ship at a world offers; an empty name says this world", () => {
  assert.ok(arrivalOffer(ship(earth), ship(mars), "Mars", row("known", false)));
  assert.ok(arrivalOffer(null, ship(mars), "Mars", null));
  assert.match(arrivalOffer(null, ship(mars), " ", null)!.line, /at this world\./);
});

test("the press is one valid set_known for the table at the campaign's now", () => {
  const o = arrivalOffer(ship(earth), ship(mars), "Mars", null)!;
  const i = arrivalReveal(o, "everyone", { at_yr: 2026.9, at_s: "63912345678" });
  assert.deepEqual(i, { kind: "set_known", address: mars, audience: "everyone", landmark: "known", scope: "this", at_yr: 2026.9, at_s: "63912345678" });
  assert.equal(checkIntent(i), null);
});
