// Records and On this day (SH5): questions over the roll's own stays and count. A made-up campaign checks each rule
// (legs joined into whole stays, stays still going left out and said, the last empty day, each year's day); the real
// "Sol today" pack is checked only against itself (the records agree with its count and its stays), never a book.
import { test } from "node:test";
import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { indexRoll, countSteps, sourceFromCrews, type RollSource } from "../src/rollcall.ts";
import { onThisDay, rollRecords, wholeStays } from "../src/rollrecords.ts";
import { GREGORIAN, parseDate } from "../src/calendar.ts";
import { constant } from "../src/sheet.ts";

const DAY = constant("day");
const at = (text: string, h = 12) => parseDate(GREGORIAN, text)! + (h / 24) * DAY;
const leg = (person: string, from: number, to: number | null, craft = "Heron") => ({ person, from_s: from, to_s: to, on: null, craft, mark: "recorded" as const, flight: true });
const source: RollSource = {
  people: [
    { id: "ada", name: "Quill, Ada" },
    { id: "bo", name: "Fenn, Bo" },
    { id: "cy", name: "Marr, Cy" },
  ],
  stays: [
    // Ada: up in the Heron, a long stay on the Lark, down in the Heron: one whole stay.
    leg("ada", at("3 March 2001"), at("4 March 2001"), "Heron"),
    leg("ada", at("4 March 2001"), at("1 June 2001"), "Lark"),
    leg("ada", at("1 June 2001"), at("1 June 2001", 18), "Heron"),
    // Bo: two short stays, more in all than Ada's one.
    leg("bo", at("3 March 2001", 13), at("30 May 2001")),
    leg("bo", at("3 March 2002"), at("1 June 2002")),
    // Cy: still up.
    leg("cy", at("3 March 2003"), null),
  ],
  logs: [],
  edge_km: 100,
};
const ix = indexRoll(source);
const steps = countSteps(ix);

test("legs that follow on are one stay; separate flights are not", () => {
  const w = wholeStays(ix);
  assert.equal(w.filter((s) => s.person === "ada").length, 1);
  assert.equal(w.filter((s) => s.person === "bo").length, 2);
});

test("records: most at once, the last empty day, longest stay, most in all, and what is left out", () => {
  const r = Object.fromEntries(rollRecords(ix, steps, DAY).map((x) => [x.key, x]));
  assert.equal(r.most!.n, 2);
  assert.equal(r.most!.at_s, at("3 March 2001", 13));
  assert.equal(r.longest!.person!.id, "ada");
  assert.equal(r.longest!.n, at("1 June 2001", 18) - at("3 March 2001"));
  assert.equal(r.total!.person!.id, "bo");
  assert.match(r.total!.how, /Stays still going are left out/);
  // Nobody up between Bo's return in 2002 and Cy's launch in 2003: the day shown is the last whole empty day.
  assert.equal(r.empty!.at_s, at("3 March 2003") - DAY);
  assert.equal(r.empty!.to_s, at("3 March 2003"));
});

test("On this day: each year's same day, who was up, who went up and came down; empty years left out", () => {
  const span = { from_s: at("1 January 2001"), to_s: at("31 December 2004") };
  const y = onThisDay(ix, steps, GREGORIAN, at("3 March 2004"), span);
  assert.deepEqual(y.map((x) => [x.year, x.up, x.went, x.came]), [[2001, 2, 2, 0], [2002, 1, 1, 0], [2003, 1, 1, 0], [2004, 1, 0, 0]]);
  // Moving between craft is not coming down.
  const june = onThisDay(ix, steps, GREGORIAN, at("1 June 2004"), span);
  assert.deepEqual(june.find((x) => x.year === 2001), { year: 2001, at_s: at("1 June 2001"), up: 1, went: 0, came: 1 });
  // 29 February is only in the years that have one.
  assert.deepEqual(onThisDay(ix, steps, GREGORIAN, at("29 February 2004"), span).map((x) => x.year), [2004]);
});

const PACK = join(import.meta.dirname, "../../../store-packs/sol-today");
const has = existsSync(join(PACK, "crews.json"));
test("Sol today: the records agree with the pack's own count and stays", { skip: !has && "crews.json is not on this branch yet" }, () => {
  const crews = JSON.parse(readFileSync(join(PACK, "crews.json"), "utf8"));
  const real = indexRoll(sourceFromCrews(crews, []));
  const s: [number, number][] = crews.in_space;
  const r = Object.fromEntries(rollRecords(real, s, DAY).map((x) => [x.key, x]));
  assert.equal(r.most!.n, Math.max(...s.map(([, n]) => n)));
  const w = wholeStays(real).filter((x) => x.to_s !== null);
  assert.equal(r.longest!.n, Math.max(...w.map((x) => x.to_s! - x.from_s)));
  assert.ok(r.total!.n >= r.longest!.n);
  assert.ok(r.empty!.to_s! > r.empty!.at_s);
});
