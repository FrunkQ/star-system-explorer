// Ships and stations on the chart room (V3 gap 13): templates group by kind,
// a scene's constructs are found by their kind or tag and drop out when
// retired, a construct names its template, and fact words read plainly.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { byKind, constructsIn, kindOf, kindWords, shipLog, templateOf, writerWords, type ConstructTemplate, type RecordRead } from "../../packages/ui-contract/src/constructs.ts";
import type { HistoryEvent } from "../../packages/ui-contract/src/session.ts";
import { railOf, RAIL_MAX } from "../../packages/ui-contract/src/screen.ts";
import { stackProvides } from "../../packages/ui-contract/src/sector.ts";
import { fieldWords, valueWords } from "../../packages/views/constructs/src/words.ts";
import type { PlayerScene } from "../../packages/ui-contract/src/player-scene.ts";

const root = fileURLToPath(new URL("../../", import.meta.url));
const json = (p: string) => JSON.parse(readFileSync(root + p, "utf8"));
const traveller = json("packs/traveller/pack.json");
const templates: ConstructTemplate[] = traveller.provides.other.templates.templates.map((t: ConstructTemplate) => ({ ...t, pack: "sse.traveller", version: "0.1.0" }));

test("the pack's templates group by kind, in words", () => {
  const g = byKind(templates);
  assert.deepEqual(g.map((x) => x.kind), ["ship", "station", "site"]);
  assert.deepEqual(g.map((x) => x.words), ["Ships", "Stations", "Sites"]);
  assert.equal(kindWords("facility"), "Facilities");
  assert.ok(g[0]!.templates.some((t) => t.id === "courier"));
  const labels = g[0]!.templates.map((t) => t.label);
  assert.deepEqual(labels, [...labels].sort());
});

test("a scene's constructs: by kind or tag, retired ones gone", () => {
  const scene: PlayerScene = {
    audience: "gm",
    scope: "u:k/sys:1",
    subjects: [
      { address: "u:k/sys:1", view: "gm", facts: [], tags: [] },
      { address: "u:k/sys:1/body:3", view: "gm", facts: [], tags: [] },
      { address: "u:k/sys:1/body:3/station:1", view: "gm", facts: [{ field: "template.id", value: "highport_a" }, { field: "template.pack", value: "sse.traveller" }], tags: ["construct"] },
      { address: "u:k/sys:1/ship:2", view: "gm", facts: [{ field: "template.id", value: "courier" }], tags: [] },
      { address: "u:k/sys:1/ship:1", view: "gm", facts: [{ field: "retired", value: true }], tags: ["construct"] },
      { address: "u:k/sys:1/faction:1", view: "gm", facts: [], tags: ["construct"] },
    ],
  };
  const got = constructsIn(scene, ["ship", "station", "site"]);
  assert.deepEqual(got.map((s) => s.address), ["u:k/sys:1/body:3/station:1", "u:k/sys:1/faction:1", "u:k/sys:1/ship:2"]);
  assert.equal(kindOf("u:k/sys:1/body:3/station:1"), "station");
  assert.equal(templateOf(got[0]!, templates)?.id, "highport_a");
  assert.equal(templateOf(got[2]!, templates)?.id, "courier");
});

test("fact words: plain labels, units from the suffix", () => {
  assert.deepEqual(fieldWords("traveller.cargo_dt"), { label: "cargo", unit: "dT" });
  assert.deepEqual(fieldWords("construct.crew_max"), { label: "crew max" });
  assert.deepEqual(fieldWords("sensor.passive_rf.range_m"), { label: "passive rf range", unit: "m" });
  assert.equal(valueWords(1.4959787e13), "1.5 × 10^13");
  assert.equal(valueWords(4), "4");
  assert.equal(valueWords(true), "yes");
});

test("Ships and stations is on the rail only while a pack gives templates, and the rails stay in bounds", () => {
  const docs = ["workspace", "physics", "packs", "reference", "screens", "dataviews", "symbols", "body", "compare", "life", "styles", "route", "window", "stories", "cutaway", "explore", "voyages", "scale-galaxy", "scale-cluster", "scale-web", "sector", "codes", "constructs"];
  const screens = Object.fromEntries(docs.map((d) => [d, json(`packages/shell/src/documents/${d}.gm.json`)]));
  const at = (scale: "system" | "world", packs: unknown[]) => railOf(screens, scale, (n) => stackProvides(packs, n)).flatMap((g) => g.lines.map((l) => l.screen));
  assert.ok(!at("system", []).includes("constructs"));
  assert.ok(at("system", [traveller]).includes("constructs"));
  assert.ok(at("world", [traveller]).includes("constructs"));
  for (const s of ["system", "world"] as const) assert.ok(at(s, [traveller]).length <= RAIL_MAX, `${s}: ${at(s, [traveller]).join(", ")}`);
});

// V3 gap 16: the log is the history and the record, read back. The record and
// history below are the kernel's replies for a trader made, renamed, a crew
// member lost, a note, then scrapped (sse-kernel, 23 Nov).
test("the ship's log reads the history and the record, oldest first, with who and why", () => {
  const history: HistoryEvent[] = [
    { id: "a", kind: "template.create", at_yr: 0, evidence: "authored", subject: ["u:trav/sys:0/ship:1"] },
    { id: "b", kind: "authored.set", at_yr: 100, evidence: "authored", subject: ["u:trav/sys:0/ship:1"] },
    { id: "c", kind: "template.retire", at_yr: 400, evidence: "authored", subject: ["u:trav/sys:0/ship:1"] },
  ];
  const record: RecordRead = {
    address: "u:trav/sys:0/ship:1",
    actors: [],
    notes: [{ at_s: 300, by: "gm", id: "n", text: "refuelled at Halt" }],
    kept: [
      { kind: "construct.crew", value: 3, history: [
        { at_s: 0, event: "e1", reason: "made from template trader", value: 6, writer: "template.trader" },
        { at_s: 200, delta: -3, event: "e2", reason: "one lost on Halt", value: 3, writer: "gm" },
      ] },
      { kind: "ship.readiness", value: 0, history: [
        { at_s: 0, event: "e3", reason: "made from template trader", value: 1, writer: "template.trader" },
        { at_s: 350, event: "e4", reason: "derelict", value: 0, writer: "effect.traveller.status.derelict" },
      ] },
    ],
  };
  const log = shipLog(history, record, (s) => s, (k) => fieldWords(k).label);
  assert.deepEqual(log.map((e) => e.what), ["made", "edited", "kept", "note", "kept", "retired"]);
  const crew = log[2]!;
  assert.equal(crew.text, `${fieldWords("construct.crew").label}: 3 (-3)`);
  assert.equal(crew.who, "the GM");
  assert.equal(crew.why, "one lost on Halt");
  // A pack trigger is named as the rule it is; the template's opening values are part of "Made".
  assert.equal(log[4]!.who, "rule traveller.status.derelict");
  assert.ok(!log.some((e) => e.who.startsWith("template")));
  assert.equal(writerWords("template.far_trader"), "template far trader");
  // No record (a seat): the history alone.
  assert.equal(shipLog(history, null, (s) => s).length, 3);
});
