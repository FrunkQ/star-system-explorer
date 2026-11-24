// Ship manifest and crew roster (track MC): entries read from the record, who is aboard where, a person's own age
// with cold sleep stopping the clock, life support days from the stores aboard at each kind of being's rates, and
// the base manifest pack itself.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { ABOARD, aboard, ageNow, crewAboard, entryFrom, entryId, lifeSupport, lifeSupportFrom, massKg, startingValues, storeOf, type AboardEntry } from "../../packages/ui-contract/src/manifest.ts";
import type { Json } from "../../packages/ui-contract/src/json.ts";
import { shipLog, type RecordRead } from "../../packages/ui-contract/src/constructs.ts";

const YR = 31_557_600;
const pack = JSON.parse(readFileSync(fileURLToPath(new URL("../../packs/base-manifest/pack.json", import.meta.url)), "utf8")) as Json;
const ls = lifeSupportFrom([pack]);
const SHIP = "u:k/sys:1/ship:1";

function rec(kept: Record<string, { value: Json; history?: { at_s: number; value: Json }[] }>): RecordRead {
  return {
    address: "x",
    notes: [],
    actors: [],
    kept: Object.entries(kept).map(([kind, k]) => ({ kind, value: k.value, history: (k.history ?? [{ at_s: 0, value: k.value }]).map((h) => ({ event: "e", writer: "gm", ...h })) })),
  };
}
function person(name: string, kept: Record<string, Json>, extra: Partial<AboardEntry> = {}): AboardEntry {
  const r = rec(Object.fromEntries(Object.entries({ [ABOARD.on]: SHIP, ...kept }).map(([k, v]) => [k, { value: v }])));
  return { ...entryFrom({ address: `u:k/actor:${entryId(name, [])}`, name, kind: "person" }, r), ...extra };
}
function lot(name: string, what: string, amount: number, unit: string): AboardEntry {
  return entryFrom({ address: `u:k/actor:${entryId(name, [])}`, name, kind: "cargo" }, rec({ [ABOARD.on]: { value: SHIP }, [ABOARD.what]: { value: what }, [ABOARD.amount]: { value: amount }, [ABOARD.unit]: { value: unit } }));
}

test("the base manifest pack keeps every kind the manifest uses, writable by the GM, with cited rates", () => {
  const kept = (pack as { provides: { other: { record: { kept: { id: string; writers: string[] }[] } } } }).provides.other.record.kept;
  for (const k of Object.values(ABOARD)) assert.ok(kept.some((x) => x.id === k && x.writers.includes("gm")), k);
  const human = ls.rates.human!;
  assert.ok(human.src && ls.sources?.[human.src], "the human rates name a source the pack lists");
  assert.ok(human.oxygen_kg_day! > 0 && human.water_kg_day! > 0 && human.food_kg_day! > 0);
});

test("an entry reads its kept values; unknown states read as awake", () => {
  const e = person("Ada Moss", { [ABOARD.role]: "pilot", [ABOARD.state]: "cold_sleep", [ABOARD.being]: "human", [ABOARD.age]: 34 });
  assert.equal(e.role, "pilot");
  assert.equal(e.state, "cold_sleep");
  assert.equal(e.age_yr, 34);
  assert.equal(person("Odd", { [ABOARD.state]: "dozing" }).state, "awake");
});

test("aboard sorts people from cargo, and only lists what is at that place", () => {
  const es = [person("Zed", { [ABOARD.role]: "captain" }), person("Ann", { [ABOARD.role]: "pilot" }), lot("Grain", "grain", 40, "t"), { ...person("Elsewhere", {}), on: "u:k/sys:2/ship:9" }];
  const a = aboard(es, SHIP);
  assert.deepEqual(a.people.map((p) => p.name), ["Ann", "Zed"]);
  assert.deepEqual(a.cargo.map((c) => c.name), ["Grain"]);
});

test("age moves on while awake and stops in cold sleep and at death", () => {
  const base = person("Ada", { [ABOARD.age]: 30 });
  assert.equal(ageNow(base, 5 * YR), 35);
  // Asleep from year 1 to year 4, then awake: three years lost to sleep.
  const slept: AboardEntry = { ...base, states: [{ event: "a", at_s: 1 * YR, value: "cold_sleep", writer: "gm" }, { event: "b", at_s: 4 * YR, value: "awake", writer: "gm" }] };
  assert.equal(ageNow(slept, 5 * YR), 32);
  const died: AboardEntry = { ...base, states: [{ event: "a", at_s: 2 * YR, value: "dead", writer: "gm" }] };
  assert.equal(ageNow(died, 10 * YR), 32);
  assert.equal(ageNow(person("No age", {}), YR), undefined);
});

test("life support days are the stores over the crew's daily use, less what is recycled", () => {
  const crew = [person("A", {}), person("B", {}), person("C", { [ABOARD.state]: "cold_sleep" })];
  const stores = [lot("Water tanks", "water", 1, "t"), lot("Rations", "rations", 360, "kg"), lot("Air", "oxygen", 82, "kg")];
  const r = lifeSupport(crew, stores, ls);
  const h = ls.rates.human!;
  assert.equal(r.drawing, 2, "the sleeper draws nothing");
  const water = r.stores.find((s) => s.store === "water")!;
  assert.equal(water.kg, 1000);
  assert.ok(Math.abs(water.days! - 1000 / (2 * h.water_kg_day!)) < 1e-9);
  assert.equal(r.stores.find((s) => s.store === "food")!.kg, 360, "rations are food by the pack's words");
  assert.equal(r.first?.store, "oxygen");
  // Recycling half the oxygen and water doubles how long they last; food is not recyclable.
  const half = lifeSupport(crew, stores, ls, 0.5);
  assert.ok(Math.abs(half.stores.find((s) => s.store === "water")!.days! - 2 * water.days!) < 1e-9);
  assert.equal(half.stores.find((s) => s.store === "food")!.days, r.stores.find((s) => s.store === "food")!.days);
});

test("a kind of being with no rates is named apart, never given a human's", () => {
  const r = lifeSupport([person("Vesh one", { [ABOARD.being]: "vesh" }), person("Vesh two", { [ABOARD.being]: "vesh" }), person("Ada", {})], [lot("Water", "water", 100, "kg")], ls);
  assert.deepEqual(r.unrated, [{ being: "vesh", count: 2 }]);
  assert.equal(r.drawing, 1);
});

test("stores in counts or volumes are left out of the sums and named", () => {
  const r = lifeSupport([person("A", {})], [lot("Food crates", "food", 12, "crates")], ls);
  assert.deepEqual(r.unweighed, ["Food crates"]);
  assert.equal(r.stores.find((s) => s.store === "food")!.days, 0);
  assert.equal(massKg(2, "t"), 2000);
  assert.equal(massKg(2, "m3"), undefined);
  assert.equal(storeOf("Grain", ls.store_words), undefined);
});

test("new entries get unique ids and only the values given", () => {
  assert.equal(entryId("Ada Moss", []), "ada-moss");
  assert.equal(entryId("Ada Moss", ["u:k/actor:ada-moss", "u:k/actor:ada-moss-2"]), "ada-moss-3");
  assert.equal(entryId("!!!", []), "entry");
  assert.deepEqual(startingValues("person", SHIP, { role: " pilot ", being: "", age_yr: 30 }), [[ABOARD.on, SHIP], [ABOARD.role, "pilot"], [ABOARD.age, 30]]);
  assert.deepEqual(startingValues("cargo", SHIP, { what: "grain", amount: 40, unit: "t" }), [[ABOARD.on, SHIP], [ABOARD.what, "grain"], [ABOARD.amount, 40], [ABOARD.unit, "t"]]);
});

test("people aboard: the roster's count when there is one, else the template's starting figure", () => {
  const ship = { address: SHIP, facts: [{ field: "construct.crew", value: 1 }, { field: "traveller.crew", value: 4 }] };
  assert.equal(crewAboard(ship, null), 1);
  assert.equal(crewAboard(ship, []), 1);
  assert.equal(crewAboard(ship, [person("A", {}), person("B", { [ABOARD.state]: "cold_sleep" }), person("C", { [ABOARD.state]: "dead" })]), 2);
});

test("fuel gauges read the kept levels, full by default, and plan from them", async () => {
  const { fuelGauges, fuelNow, FUEL_FIELDS } = await import("../../packages/ui-contract/src/index.ts");
  const courier = { wet_t: 420, dry_t: 400, fuels: ["deuterium"], jump: { tank_t: 19.838, fuel: "hydrogen" } };
  const none = () => undefined;
  const g = fuelGauges(courier, none);
  assert.deepEqual(g.map((x) => [x.tank, x.fuel, x.full_t, x.share]), [["propellant", "deuterium", 20, 1], ["jump", "hydrogen", 19.838, 1]]);
  const kept = (f: string) => (f === FUEL_FIELDS.jump ? 9.919 : f === FUEL_FIELDS.propellant ? 99 : undefined);
  const h = fuelGauges(courier, kept);
  assert.equal(h[0]!.now_t, 20, "more than the tank holds reads as full");
  assert.equal(h[1]!.share, 0.5);
  const now = fuelNow(courier, kept);
  assert.equal(now.propellant_t, 20);
  assert.equal(now.jump!.fuel_t, 9.919);
  assert.deepEqual(fuelGauges({ wet_t: 10, dry_t: 10 }, none), [], "no tanks, no gauge");
});

test("the ship's log reads who came aboard, left and changed state while aboard", async () => {
  const { aboardLog } = await import("../../packages/ui-contract/src/index.ts");
  const ch = (at_s: number, value: string, reason?: string) => ({ event: `e${at_s}`, at_s, value, writer: "gm", ...(reason ? { reason } : {}) });
  const ship = "u:1/c:ship";
  const entries = [
    { address: "u:1/actor:ada", name: "Ada", kind: "person", state: "awake", moves: [ch(10, ship)], states: [ch(10, "awake"), ch(20, "cold_sleep"), ch(40, "awake")] },
    { address: "u:1/actor:parts", name: "parts", kind: "cargo", amount: 3, unit: "t", state: "awake", moves: [ch(5, "u:1/c:yard"), ch(12, ship), ch(30, "u:1/c:lark", "sold")], states: [] },
    // A state change elsewhere is not this ship's.
    { address: "u:1/actor:bo", name: "Bo", kind: "person", state: "dead", moves: [ch(1, ship), ch(2, "u:1/c:lark")], states: [ch(1, "awake"), ch(3, "dead")] },
  ] as never;
  const names: Record<string, string> = { "u:1/c:yard": "the yard", "u:1/c:lark": "Brass Lark" };
  const log = aboardLog(ship, entries, (s: number) => s, (a: string) => names[a] ?? a);
  assert.deepEqual(log.map((e) => [e.at_yr, e.text, e.why]), [
    [1, "Bo came aboard", undefined],
    [2, "Bo left for Brass Lark", undefined],
    [10, "Ada came aboard", undefined],
    [12, "parts (3 t) came aboard from the yard", undefined],
    [20, "Ada is in cold sleep", undefined],
    [30, "parts (3 t) left for Brass Lark", "sold"],
    [40, "Ada is awake", undefined],
  ]);
});

test("a refit's log line says what changed, from what to what (MC6)", () => {
  const made = { id: "a", kind: "template.create", at_yr: 1, evidence: "authored", subject: ["s"], changes: [{ field: "jump_rating", value: 2 }, { field: "name", value: "Kestrel" }] };
  const refit = { id: "b", kind: "authored.set", at_yr: 2, evidence: "authored", subject: ["s"], changes: [{ field: "jump_rating", value: 3 }, { field: "construct.design", value: { style: "x" } }, { field: "picture.own", value: null, large: true }, { field: "picture.shown", value: "picture:0" }] };
  const plain = { id: "c", kind: "authored.set", at_yr: 3, evidence: "authored", subject: ["s"] };
  const log = shipLog([refit, made, plain], null, (s) => s, (k) => ({ jump_rating: "Jump drive", "construct.design": "Design", "picture.own": "Pictures" })[k] ?? k);
  assert.equal(log[1]!.text, "Changed: Jump drive 2 → 3; Design changed; Pictures changed; picture.shown picture 1");
  assert.equal(log[2]!.text, "Facts edited", "an event without its changes keeps the old words");
});
