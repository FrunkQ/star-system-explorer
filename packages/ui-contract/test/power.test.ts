// RK9: a ship's power books in words, and as a voyage carries them.
import { test } from "node:test";
import assert from "node:assert/strict";
import { nowhereWords, powerWords, wattsText, type PowerBooks } from "../src/index.ts";
import { readFileSync } from "node:fs";
import { Session } from "../src/session.ts";
import { gmSeat } from "../src/seat.ts";
import type { PackDoc } from "../src/physics.ts";
import type { SystemVoyagePlan } from "../src/voyage.ts";
import { nodeKernel } from "./node-kernel.ts";

const books = (over: Partial<PowerBooks["ledger"]> = {}, fictional = false): PowerBooks => ({
  flux_w_m2: 1361,
  accel_g: 0.000135,
  exhaust_km_s: 20,
  ledger: { flows: [], made_w: 1e5, wanted_w: 1e5, spare_w: 0, engines: [], thrust_n: 530, heat_by_k: { "300": 2.5e4 }, nowhere_w: 0, why: [], ...over },
  balance: { id: "power-books", demand: 5.3e6, supply: 1e5, ok: !fictional },
  fictional,
  why: [],
});

test("watts read in the unit a reader would use", () => {
  assert.equal(wattsText(950), "950 W");
  assert.equal(wattsText(2600), "2.60 kW");
  assert.equal(wattsText(5.2e6), "5.20 MW");
  assert.equal(wattsText(47e6), "47.0 MW");
});

test("the power line says made, spare or short, heat and the drive", () => {
  assert.match(powerWords(books({ spare_w: 6e3 })), /100 kW made, 6\.00 kW to spare; 25\.0 kW of heat to shed; 0\.135 milli-g/);
  assert.match(powerWords(books({ spare_w: -1.4e3 })), /1\.40 kW short, so the electric drive throttles back/);
});

test("books that do not close say what came from nowhere; books that close say nothing", () => {
  assert.equal(nowhereWords(books()), null);
  assert.match(nowhereWords(books({ nowhere_w: 5.2e6 }, true))!, /^5\.20 MW from nowhere/);
});

// Against the real kernel: a Dawn-like solar ship and SSE3's Hall tug in Sol (NAV fixture).

const root = new URL("../../../", import.meta.url);
const base: PackDoc = JSON.parse(readFileSync(new URL("packs/base-physics/pack.json", root), "utf8"));
const hall = {
  name: "Hall tug", accel_g: 0.05, sustained: true, exhaust_km_s: 20, wet_t: 400, dry_t: 200,
  power: [{ id: "plant", kind: "fission-reactor", electric_w: 1e5 }],
  engines: [{ engine: { name: "Hall thruster", thrust_n: 530, exhaust_km_s: 20, energy_from: "electric", electric_w: 1e5 } }],
};

test("a voyage carries the ship's power books, and a drive given too little says so", async () => {
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, gmSeat(), { id: "power", seed: "power books" }, [base], k.scratch);
    const r = await s.planVoyage({ fixture: "sol", ship: hall, from: "Earth", to: "Mars" });
    assert.ok(r.ok, JSON.stringify(r));
    const p = (r.plan as SystemVoyagePlan).power;
    assert.ok(p, "fitted engines bring the books");
    assert.equal(p.fictional, true);
    assert.match(nowhereWords(p)!, /^5\.\d\d MW from nowhere/);
    assert.match(powerWords(p), /100 kW made/);
    const typed = await s.planVoyage({ fixture: "sol", ship: "fusion-torch", from: "Earth", to: "Mars" });
    assert.ok(typed.ok);
    assert.equal((typed.plan as SystemVoyagePlan).power, undefined, "a typed ship has no books");
  } finally {
    await k.close();
  }
});
