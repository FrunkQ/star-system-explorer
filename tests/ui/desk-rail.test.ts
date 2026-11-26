// TM2 part 1 (three desks design section 5, question 7): the rail on each desk, from the shipped screen documents.
// On Run an entry is listed if it answers a question asked at the table (where are we, what is it like, how do we get
// there, what do they see); one that explains, sifts or sets up is off Run. Explore and Build list everything.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readdirSync, readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { checkScreen, railOf, RAIL_SCALES, type ScreenDoc } from "../../packages/ui-contract/src/screen.ts";
import { DESKS, type Desk } from "../../packages/ui-contract/src/desk.ts";

const root = fileURLToPath(new URL("../../", import.meta.url));
const dirs = ["packages/shell/src/documents", "packages/shell/src/atlas"];
const screens: Record<string, ScreenDoc> = {};
for (const d of dirs) for (const f of readdirSync(root + d).filter((n) => n.endsWith(".gm.json"))) screens[f.replace(/\.gm\.json$/, "")] = JSON.parse(readFileSync(`${root}${d}/${f}`, "utf8"));

const labels = (scale: (typeof RAIL_SCALES)[number], desk?: Desk) => railOf(screens, scale, () => true, desk).flatMap((g) => g.lines.map((l) => l.label));

test("every shipped rail entry's desks are well formed", () => {
  for (const [id, doc] of Object.entries(screens)) {
    const r = checkScreen({ ...doc, items: [] }, []);
    const desk = r.ok ? [] : r.errors.filter((e) => /desk/.test(e));
    assert.deepEqual(desk, [], id);
  }
});

test("Explore and Build list every entry, as before", () => {
  for (const s of RAIL_SCALES) {
    assert.deepEqual(labels(s, "explore"), labels(s), s);
    assert.deepEqual(labels(s, "build"), labels(s), s);
  }
});

test("Run leaves off Sizes, Life, Stories, Roll call, The physics, Look up and Postcards, and keeps the table's instruments", () => {
  const OFF_RUN = ["Sizes", "Life", "Stories", "Roll call", "The physics", "Look up", "Postcards"];
  for (const s of RAIL_SCALES) {
    const run = labels(s, "run");
    assert.deepEqual(run, labels(s).filter((l) => !OFF_RUN.includes(l)), s);
  }
  assert.deepEqual([...labels("system", "run")].sort(), ["Atlas", "Chart", "Hazard map", "Route", "Ships and stations", "Sky from here", "Switchboard", "Voyages"]);
  for (const l of ["Globe", "Cutaway", "Sky from here", "Land", "Route to here", "Codes", "Atlas", "Switchboard"]) assert.ok(labels("world", "run").includes(l), l);
  // Every off-Run entry is still on the other two desks.
  for (const d of DESKS.filter((x) => x !== "run")) for (const s of RAIL_SCALES) for (const l of OFF_RUN) if (labels(s).includes(l)) assert.ok(labels(s, d).includes(l), `${d} ${s} ${l}`);
});
