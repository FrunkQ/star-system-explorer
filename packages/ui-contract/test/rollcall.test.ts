// Roll call (SH1): who was in flight on a day, aboard what and where. A made-up campaign checks each rule; the real
// "Sol today" crews (RS9) check the look-up against the pack's own step list, never against a figure from a book.
import { test } from "node:test";
import assert from "node:assert/strict";
import { existsSync, readFileSync, readdirSync, statSync } from "node:fs";
import { join } from "node:path";
import { countAt, countSteps, indexRoll, rollCall, rollSentence, sourceFromCrews, sourceFromManifest, stepEvent, type RollLog, type RollSource } from "../src/rollcall.ts";
import { constant } from "../src/sheet.ts";
import type { AboardEntry } from "../src/manifest.ts";

const DAY = constant("day");
const HOME = "u:k/sys:1/body:3";
const MOON = "u:k/sys:1/body:3/body:0";
const STAR = "u:k/sys:1";
const T0 = 1_000 * DAY;
const names: Record<string, string> = { [HOME]: "Tarn", [MOON]: "Skerry", [STAR]: "Kestrel's star", "u:k/sys:1/body:4": "Hollow" };
const nameOf = (a: string) => names[a] ?? a;

// Made up: the Heron flies three to Skerry, two land in the Wren; the Petrel hops; the Tern sits in orbit, empty.
const logs: RollLog[] = [
  { construct: "heron", name: "Heron", pieces: [
    { from_s: T0, to_s: T0 + 2 * DAY, state: { kind: "orbit", around: HOME }, mark: "recorded" },
    { from_s: T0 + 2 * DAY, to_s: T0 + 4 * DAY, state: { kind: "course" }, mark: "estimated", why: "big burns only" },
    { from_s: T0 + 4 * DAY, to_s: T0 + 9 * DAY, state: { kind: "orbit", around: MOON }, mark: "recorded" },
    { from_s: T0 + 9 * DAY, state: { kind: "landed", body: HOME }, mark: "recorded" },
  ] },
  { construct: "wren", name: "Wren", pieces: [
    { from_s: T0, to_s: T0 + 5 * DAY, state: { kind: "docked", to: "heron" }, mark: "recorded" },
    { from_s: T0 + 5 * DAY, to_s: T0 + 6 * DAY, state: { kind: "landed", body: MOON }, mark: "recorded" },
    { from_s: T0 + 6 * DAY, state: { kind: "gone", how: "impacted" }, mark: "recorded" },
  ] },
  { construct: "tern", name: "Tern", gcat: "S1", pieces: [{ from_s: 0, state: { kind: "orbit", around: HOME }, mark: "recorded" }] },
  { construct: "lark", name: "Lark", pieces: [{ from_s: 0, state: { kind: "orbit", around: STAR }, mark: "recorded" }] },
];
const source: RollSource = {
  people: [
    { id: "p:ada", name: "Ada Quill" },
    { id: "p:bo", name: "Bo Fenn" },
    { id: "p:cy", name: "Cy Marr" },
    { id: "p:dot", name: "Dot", being: "dog" },
    { id: "p:eli", name: "Eli Sand" },
    { id: "p:fay", name: "Fay Lowe" },
    { id: "p:gil", name: "Gil Oake" },
  ],
  stays: [
    { person: "p:ada", from_s: T0, to_s: T0 + 9 * DAY, on: "heron", craft: "Heron", role: "commander", mark: "recorded", flight: true },
    { person: "p:bo", from_s: T0, to_s: T0 + 5 * DAY, on: "heron", craft: "Heron", role: "pilot", mark: "recorded", flight: true },
    { person: "p:bo", from_s: T0 + 5 * DAY, to_s: T0 + 6 * DAY, on: "wren", craft: "Wren", role: "pilot", mark: "recorded", flight: true },
    { person: "p:bo", from_s: T0 + 6 * DAY, to_s: T0 + 9 * DAY, on: "heron", craft: "Heron", role: "pilot", mark: "recorded", flight: true },
    { person: "p:cy", from_s: T0 + 5 * DAY, to_s: T0 + 6 * DAY, on: "wren", craft: "Wren", role: "engineer", mark: "estimated", why: "the landing hour is not recorded", flight: true },
    { person: "p:dot", from_s: T0 + 30 * DAY, to_s: T0 + 31 * DAY, on: "tern", craft: "Tern", mark: "recorded", flight: true },
    { person: "p:eli", from_s: T0 + 40 * DAY, to_s: T0 + 40 * DAY + 900, on: null, craft: "Petrel", mark: "recorded", flight: true, suborbital: true, peak_km: 90 },
    { person: "p:fay", from_s: T0 + 40 * DAY, to_s: T0 + 40 * DAY + 900, on: null, craft: "Shrike", mark: "recorded", flight: true, suborbital: true, peak_km: null },
    { person: "p:gil", from_s: T0 + 60 * DAY, to_s: T0 + 60 * DAY + 70, on: null, craft: "Gannet", mark: "recorded", flight: true, reached_space: false, reached_why: "lost in the climb" },
  ],
  logs,
  edge_km: 100,
  listed_from_km: 80,
  home: HOME,
};
const ix = indexRoll(source);
const day = (n: number) => [T0 + n * DAY, T0 + (n + 1) * DAY] as const;

test("a day's roll call groups people under their craft and craft under places, nearest first", () => {
  const [a, b] = day(5);
  const rc = rollCall(ix, a, b, a + 3600, { nameOf });
  assert.deepEqual(rc.groups.map((g) => g.label), ["At Skerry"]);
  const g = rc.groups[0]!;
  assert.deepEqual(g.crafts.map((c) => c.name), ["Heron", "Wren"]);
  const wren = g.crafts.find((c) => c.name === "Wren")!;
  assert.equal(wren.where, "on the surface");
  assert.deepEqual(wren.people.map((r) => r.name).sort(), ["Bo Fenn", "Cy Marr"]);
  assert.equal(rc.people, 3);
  assert.equal(rc.crafts, 2);
  // Bo's three stays are one flight: day 6 of 9.
  const bo = wren.people.find((r) => r.person === "p:bo")!;
  assert.equal(bo.day, 6);
  assert.equal(bo.of, 9);
  // An estimated stay keeps its mark and why.
  const cy = wren.people.find((r) => r.person === "p:cy")!;
  assert.equal(cy.mark, "estimated");
  assert.match(cy.why ?? "", /not recorded/);
});

test("a docked craft is placed where its host is, and says so", () => {
  const [a, b] = day(1);
  const rc = rollCall(ix, a, b, a, { nameOf });
  assert.deepEqual(rc.groups.map((g) => g.label), ["Around Tarn"]);
  assert.deepEqual(rc.groups[0]!.crafts.map((c) => [c.name, c.where]), [["Heron", "in orbit"]]);
  const wrenAlone = rollCall(ix, a, b, a, { nameOf }).also;
  assert.ok(wrenAlone.some((x) => x.label === "Around Tarn" && x.count >= 2), "the Tern and the docked Wren fly empty");
});

test("on the way: a course is grouped under where it goes, in pencil when estimated", () => {
  const [a, b] = day(3);
  const rc = rollCall(ix, a, b, a, { nameOf });
  assert.equal(rc.groups[0]!.label, "On the way to Skerry");
  assert.equal(rc.groups[0]!.crafts[0]!.mark, "estimated");
});

test("kinds of being are never assumed human, and the sentence says so", () => {
  const [a, b] = day(30);
  const rc = rollCall(ix, a, b, a, { nameOf });
  assert.equal(rc.people, 1);
  assert.equal(rc.humans, 0);
  assert.equal(rc.groups[0]!.crafts[0]!.people[0]!.being, "dog");
  assert.match(rollSentence(rc), /^1 was in space, aboard 1 craft\./);
  const [c, d] = day(5);
  assert.match(rollSentence(rollCall(ix, c, d, c, { nameOf })), /^3 people were in space, aboard 2 craft\./);
});

test("the edge of space is a number: below it a hop is listed and not counted; an unknown height is counted and said", () => {
  const [a, b] = day(40);
  const rc = rollCall(ix, a, b, a, { nameOf });
  const rows = rc.groups.flatMap((g) => g.crafts.flatMap((c) => c.people));
  const eli = rows.find((r) => r.person === "p:eli")!;
  const fay = rows.find((r) => r.person === "p:fay")!;
  assert.match(eli.below_edge ?? "", /80 km line, below the 100 km edge/);
  assert.equal(fay.height_unknown, true);
  assert.equal(rc.people, 1);
  // Move the edge to 80 km and the hop counts.
  assert.equal(rollCall(ix, a, b, a, { nameOf, edge_km: 80 }).people, 2);
  assert.equal(rc.groups[0]!.label, "Around Tarn");
  assert.match(rc.groups[0]!.crafts.find((c) => c.name === "Petrel")!.where, /up to 90 km/);
});

test("a flight that never reached space is listed on its day and never counted", () => {
  const [a, b] = day(60);
  const rc = rollCall(ix, a, b, a, { nameOf });
  assert.equal(rc.people, 0);
  const gil = rc.groups.flatMap((g) => g.crafts.flatMap((c) => c.people)).find((r) => r.person === "p:gil")!;
  assert.match(gil.below_edge ?? "", /^did not reach space: lost in the climb$/);
  assert.equal(countAt(countSteps(ix, 100), T0 + 60 * DAY + 1), 0);
});

test("an empty day names the last person down and the next up; before anyone it says so", () => {
  const [a, b] = day(20);
  const rc = rollCall(ix, a, b, a, { nameOf });
  assert.equal(rc.people, 0);
  assert.equal(rc.empty?.last?.name, "Ada Quill");
  assert.equal(rc.empty?.next?.name, "Dot");
  const before = rollCall(ix, T0 - 10 * DAY, T0 - 9 * DAY, T0 - 10 * DAY, { nameOf });
  assert.equal(before.empty?.before_first, true);
  assert.equal(before.empty?.next?.at_s, T0);
});

test("event steps go to the next and previous dated event", () => {
  assert.equal(stepEvent(ix, T0 + 1, 1), T0 + 2 * DAY);
  assert.equal(stepEvent(ix, T0 + 2 * DAY, -1), T0);
  assert.equal(stepEvent(ix, T0 + 100 * DAY, 1), null);
});

test("the count steps follow the edge", () => {
  const s100 = countSteps(ix, 100);
  const s80 = countSteps(ix, 80);
  assert.equal(countAt(s100, T0 + 1), 2);
  assert.equal(countAt(s100, T0 + 5 * DAY + 1), 3);
  assert.equal(countAt(s100, T0 + 40 * DAY + 1), 1);
  assert.equal(countAt(s80, T0 + 40 * DAY + 1), 2);
  assert.equal(countAt(s100, T0 + 50 * DAY), 0);
});

test("a campaign's own manifest is a source: in flight is read from the log", () => {
  const entry = (address: string, name: string, moves: [number, string][]): AboardEntry => ({ address, name, kind: "person", state: "awake", states: [], moves: moves.map(([at_s, value]) => ({ event: `e${at_s}`, at_s, value, writer: "gm" })) });
  const src = sourceFromManifest([entry("p:ada", "Ada Quill", [[T0 - DAY, "heron"], [T0 + 12 * DAY, "u:k/sys:1/body:3/site:port"]])], logs, HOME);
  const m = indexRoll(src);
  // Before launch the Heron has no piece: Ada is aboard but not in flight.
  assert.equal(rollCall(m, T0 - DAY, T0 - 1, T0 - DAY, { nameOf }).people, 0);
  assert.equal(rollCall(m, ...day(4), T0 + 4 * DAY, { nameOf }).groups[0]!.label, "At Skerry");
  // Landed at home on day 9: aboard, not in flight.
  assert.equal(rollCall(m, ...day(10), T0 + 10 * DAY, { nameOf }).people, 0);
});

// ---- The real pack, once RS9's crews are on main ----

const PACK = join(import.meta.dirname, "../../../store-packs/sol-today");
const hasCrews = existsSync(join(PACK, "crews.json"));
function readLogs(dir: string): unknown[] {
  const out: unknown[] = [];
  for (const f of readdirSync(dir).sort()) {
    const p = join(dir, f);
    if (statSync(p).isDirectory()) out.push(...readLogs(p));
    else if (f.endsWith(".json")) out.push(JSON.parse(readFileSync(p, "utf8")));
  }
  return out;
}

test("Sol today: every crewed leg's craft has a flight log, and the look-up's count is the pack's own step list", { skip: !hasCrews && "crews.json is not on this branch yet" }, () => {
  const crews = JSON.parse(readFileSync(join(PACK, "crews.json"), "utf8"));
  const src = sourceFromCrews(crews, readLogs(join(PACK, "logs")) as never);
  const real = indexRoll(src);
  for (const s of src.stays) if (s.on) assert.ok(real.logs.has(s.on), `no log for ${s.on}`);
  // The pack lists from its own lowest line; with the edge there the look-up must agree at every step.
  const mine = countSteps(real, src.listed_from_km ?? null);
  const theirs = crews.in_space as [number, number][];
  for (const [t] of theirs) assert.equal(countAt(mine, t), countAt(theirs, t), `count at ${t}`);
  // And a roll call at each step's instant counts the same people (a window of one second).
  for (const [t, n] of theirs.filter((_, i) => i % 7 === 0)) assert.equal(rollCall(real, t, t + 1, t, { edge_km: src.listed_from_km ?? null }).people, n, `roll call at ${t}`);
});
