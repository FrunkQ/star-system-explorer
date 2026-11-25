// Names carried (SH7): the memorials from a pack's memorials.json, the names they carry and from when, and where the
// small bodies are on a date. A made-up record checks each rule; the orbit solve is checked against the two-body laws
// it must keep (Kepler's period and the vis-viva energy), never against a pinned figure; the real "Sol today" record
// (RS9 part 3) is checked for its own rules.
import { test } from "node:test";
import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { carriedLine, namedAt, sharedSinceWhy, namesCarried, orbitAt, sinceWords } from "../src/namescarried.ts";
import { constant } from "../src/sheet.ts";

const src = [{ text: "Record one", url: "https://example.org/1" }, { text: "Record two" }];
const crews = { people: [{ id: "p:ada", name: "Quill, Ada *Jo" }, { id: "p:bo", name: "Fenn, Bo" }] };
const AU = constant("au");
const el = { a_m: 2.7 * AU, e: 0.21, i_deg: 7.5, node_deg: 120, peri_deg: 40, m0_deg: 15, epoch_s: 1000, frame: "heliocentric ecliptic J2000" };
const doc = {
  memorials: [
    { id: "m2", kind: "named_small_body", name: "4021 Fenn", people: ["p:bo"], since_s: 5000, since_date: "2001-02-03", where: { elements: el }, sources: src },
    { id: "m1", kind: "named_small_body", name: "812 Quill", people: ["p:ada", "p:bo"], since_s: null, since_date: null, since_why: "the citation is not yet read", where: { elements: el }, sources: src },
    { id: "m3", kind: "named_small_body", name: "", sources: src },
  ],
};

test("memorials in number order, named for pack people, unnamed rows dropped", () => {
  const m = namesCarried(doc, crews);
  assert.deepEqual(m.map((c) => c.name), ["812 Quill", "4021 Fenn"]);
  assert.equal(carriedLine(m[0]!), "812 Quill, for Quill, Ada Jo and Fenn, Bo.");
  assert.equal(carriedLine(m[1]!), "4021 Fenn, for Fenn, Bo.");
});

test("a name is carried from its date; an unknown date is said, not guessed", () => {
  const [quill, fenn] = namesCarried(doc, crews);
  assert.equal(namedAt(fenn!, 4999), false);
  assert.equal(namedAt(fenn!, 5000), true);
  assert.equal(namedAt(quill!, 0), true);
  assert.equal(sinceWords(fenn!, (d) => `[${d}]`), "Named [2001-02-03].");
  assert.equal(sinceWords(quill!, String), "The naming date is not given here: the citation is not yet read.");
  // Said once for the list when every row lacks a date for the same reason; never when any row has one.
  assert.equal(sharedSinceWhy([quill!]), "the citation is not yet read");
  assert.equal(sharedSinceWhy([quill!, fenn!]), null);
});

test("the orbit keeps Kepler's period and the vis-viva energy", () => {
  const GM = constant("GM_sun");
  const period = 2 * Math.PI * Math.sqrt(el.a_m ** 3 / GM);
  const p0 = orbitAt(el, el.epoch_s);
  const p1 = orbitAt(el, el.epoch_s + period);
  assert.ok(Math.hypot(p1.x - p0.x, p1.y - p0.y, p1.z - p0.z) < 1e-6 * el.a_m, "back where it was after one period");
  for (const t of [0, 0.13, 0.5, 0.77].map((f) => el.epoch_s + f * period)) {
    const p = orbitAt(el, t);
    assert.ok(p.r_m >= el.a_m * (1 - el.e) * (1 - 1e-12) && p.r_m <= el.a_m * (1 + el.e) * (1 + 1e-12), "between perihelion and aphelion");
    const h = 1;
    const a = orbitAt(el, t - h);
    const b = orbitAt(el, t + h);
    const v2 = ((b.x - a.x) ** 2 + (b.y - a.y) ** 2 + (b.z - a.z) ** 2) / (2 * h) ** 2;
    assert.ok(Math.abs(v2 / (GM * (2 / p.r_m - 1 / el.a_m)) - 1) < 1e-6, "vis-viva");
  }
  // The inclination is the tilt of the orbit's plane: the highest point above the ecliptic is r sin i at most.
  const zmax = Math.max(...Array.from({ length: 64 }, (_, k) => Math.abs(orbitAt(el, el.epoch_s + (k / 64) * period).z / orbitAt(el, el.epoch_s + (k / 64) * period).r_m)));
  assert.ok(zmax <= Math.sin(el.i_deg * (Math.PI / 180)) + 1e-9 && zmax > 0.9 * Math.sin(el.i_deg * (Math.PI / 180)));
});

const PACK = join(import.meta.dirname, "../../../store-packs/sol-today");
const has = existsSync(join(PACK, "memorials.json")) && existsSync(join(PACK, "crews.json"));
test("Sol today: every memorial has two sources, every person is in the crews, every orbit is placed", { skip: !has && "memorials.json is not on this branch yet" }, () => {
  const raw = JSON.parse(readFileSync(join(PACK, "memorials.json"), "utf8"));
  const c = JSON.parse(readFileSync(join(PACK, "crews.json"), "utf8"));
  const ids = new Set(c.people.map((p: { id: string }) => p.id));
  const m = namesCarried(raw, c);
  assert.equal(m.length, raw.memorials.length);
  for (const x of m) {
    assert.ok(x.sources.length >= 2, `${x.id}: two sources`);
    for (const f of x.for) assert.ok(!f.person || ids.has(f.person), `${x.id}: ${f.person} in the crews`);
    assert.ok(x.since_s !== null || x.since_why, `${x.id}: a date or a reason`);
    if (x.kind === "named_small_body") {
      assert.ok(x.elements, `${x.id}: elements`);
      const r = orbitAt(x.elements!, x.elements!.epoch_s).r_m;
      assert.ok(r > 0 && Number.isFinite(r), `${x.id}: placed`);
    }
  }
});
