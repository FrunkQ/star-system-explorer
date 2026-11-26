// The three desks as data (TM1): the records, shows(), the shell's parts, the rail's desks and the per-desk memory keys.
import { test } from "node:test";
import assert from "node:assert/strict";
import { DESKS, SHIPPED_DESKS, SHELL_PARTS, checkDesk, checkDesks, deskDoc, deskKeeps, deskMemoryKey, deskWords, isDesk, partShows, shellPart, shows } from "../src/desk.ts";
import { DESK_WORDS } from "../src/actions.ts";
import { eyesShow } from "../src/known.ts";
import { checkScreen, railOf, type ScreenDoc } from "../src/screen.ts";
import { controlLines } from "../src/part-controls.ts";

test("the three shipped desks are valid records, in the switch's order, with the design's three answers", () => {
  assert.deepEqual(SHIPPED_DESKS.map((d) => d.id), [...DESKS]);
  for (const d of SHIPPED_DESKS) assert.deepEqual(checkDesk(d), [], d.id);
  assert.deepEqual(SHIPPED_DESKS.map((d) => [d.label, d.writes, d.time]), [
    ["Explore", "try", "look"],
    ["Build", "campaign", "set"],
    ["Run", "play", "clock"],
  ]);
  // The keys are distinct, and the menu's desk words come from the same records.
  assert.equal(new Set(SHIPPED_DESKS.map((d) => d.key)).size, 3);
  assert.deepEqual(DESK_WORDS, { explore: "Explore", build: "Build", run: "Run" });
});

test("only Build keeps changes today, and each desk opens with its lens (Explore: the device's)", () => {
  assert.deepEqual(DESKS.filter(deskKeeps), ["build"]);
  assert.equal(deskDoc("explore").lens, undefined);
  assert.equal(deskDoc("build").lens, "building");
  assert.equal(deskDoc("run").lens, "running");
});

test("a desk record is checked word by word", () => {
  assert.deepEqual(checkDesk(null), ["a desk must be an object"]);
  const bad = checkDesk({ schema: "sse.desk/2", id: "play", label: "", key: "rr", hint: "x", line: "y", writes: "keep", time: "now", lens: "" });
  assert.equal(bad.length, 7);
  assert.match(bad.join("; "), /unknown desk play/);
  assert.ok(isDesk("run") && !isDesk("Run") && !isDesk(undefined));
});

test("shows(): left out means every desk; a list means those desks only", () => {
  for (const d of DESKS) assert.ok(shows({}, d) && shows(undefined, d));
  assert.deepEqual(DESKS.filter((d) => shows({ desks: ["build", "run"] }, d)), ["build", "run"]);
  assert.equal(deskWords({}), "");
  assert.equal(deskWords({ desks: ["build", "run"] }), "Build and Run");
  assert.equal(deskWords({ desks: ["run"] }), "Run");
  assert.deepEqual(checkDesks(undefined, "x"), []);
  assert.equal(checkDesks([], "x").length, 1);
  assert.deepEqual(checkDesks(["run", "play", "run"], "x"), ["x: unknown desk play", "x: a desk is listed twice"]);
});

test("the shell's parts say which desks each is on now", () => {
  const ids = SHELL_PARTS.map((p) => p.id);
  assert.equal(new Set(ids).size, ids.length);
  for (const p of SHELL_PARTS) {
    assert.ok(p.words.length, p.id);
    assert.deepEqual(checkDesks(p.desks, p.id), []);
    for (const c of p.controls ?? []) assert.deepEqual(checkDesks(c.desks, `${p.id}.${c.id}`), []);
  }
  // The eyes are Build's and Run's (the SV ruling); the depth dial and the Workshop door are off Run (TM2b) and the rail
  // folds there (TM2c, design 3.1).
  assert.deepEqual(SHELL_PARTS.filter((p) => p.desks).map((p) => [p.id, p.desks]), [
    ["depth", ["explore", "build"]],
    ["rail-open", ["explore", "build"]],
    ["workshop-door", ["explore", "build"]],
    ["eyes", ["build", "run"]],
    ["ruler", ["explore", "build"]],
    ["clock", ["run"]],
    ["light", ["explore", "build"]],
  ]);
  // The stage's edit tools are Build's; Select and Keyboard shortcuts are off Run; the ruler is on every desk until TM3.
  assert.deepEqual(DESKS.filter((d) => partShows("tools", d, "cut")), ["build"]);
  assert.deepEqual(DESKS.filter((d) => partShows("tools", d, "select")), ["explore", "build"]);
  assert.deepEqual(DESKS.filter((d) => partShows("tools", d, "keys")), ["explore", "build"]);
  // On Run the clock takes the ruler's place, and Light goes with the ruler (TM3a, design 7.3); Sound stays.
  assert.deepEqual(DESKS.map((d) => [partShows("ruler", d), partShows("clock", d)]), [[true, false], [true, false], [false, true]]);
  assert.ok(DESKS.every((d) => partShows("sound", d)));
  // On Run the status line says whether the campaign is kept, without the seat or the engine's numbers.
  assert.deepEqual(["seat", "kept", "engine"].map((c) => partShows("status-line", "run", c)), [false, true, false]);
  assert.ok(["seat", "kept", "engine"].every((c) => partShows("status-line", "explore", c) && partShows("status-line", "build", c)));
  assert.throws(() => shellPart("stepper"), /no shell part stepper/);
  assert.throws(() => partShows("tools", "run", "measure"), /no control measure/);
  // The eyes read their desks from the part, as they did by name.
  assert.deepEqual(DESKS.map((d) => eyesShow(d, true)), [false, true, true]);
  assert.equal(eyesShow(undefined, true), false);
  assert.equal(eyesShow("build", false), false);
});

test("a part's control line names its desks when it is not on all three", () => {
  assert.equal(controlLines(shellPart("tools").controls!.filter((c) => c.id === "select" || c.id === "copy")), "- select (button, on by default, on Explore and Build only): Select\n- copy (button, on by default, on Build only): Copy");
});

const screens: Record<string, Pick<ScreenDoc, "rail">> = {
  sky: { rail: [{ scale: "world", group: "look", label: "Sky from here", order: 1 }] },
  life: { rail: [{ scale: "world", group: "understand", label: "Life", order: 1, desks: ["explore", "build"] }] },
};

test("a rail entry with desks is listed on those desks only; without a desk the rail lists everything", () => {
  const lines = (desk?: "explore" | "build" | "run") => railOf(screens, "world", () => true, desk).flatMap((g) => g.lines.map((l) => l.screen));
  assert.deepEqual(lines(), ["sky", "life"]);
  assert.deepEqual(lines("build"), ["sky", "life"]);
  assert.deepEqual(lines("run"), ["sky"]);
});

test("a screen's rail entry with a bad desks field is refused", () => {
  const doc = { schema: "sse.screen/1", id: "s", label: "S", audience: "gm", layout: "grid", theme: "t", symbology: "s", vocab: "plain", items: [], rail: [{ scale: "world", group: "look", label: "S", order: 1, desks: ["play"] }] } as unknown as ScreenDoc;
  const r = checkScreen(doc, []);
  assert.ok(!r.ok && r.errors.includes("rail world: unknown desk play"), JSON.stringify(r));
});

test("each desk remembers its view and ruler span per level, and its depth per desk", () => {
  assert.equal(deskMemoryKey("run", "view", "system"), "desk.run.system.view");
  assert.equal(deskMemoryKey("build", "ruler-span", "galaxy"), "desk.build.galaxy.ruler-span");
  assert.equal(deskMemoryKey("run", "depth"), "desk.run.depth");
  assert.throws(() => deskMemoryKey("run", "view"), /per level/);
  assert.throws(() => deskMemoryKey("run", "depth", "world"), /per desk/);
  // The keys never collide with the device's existing desk keys ("desk", "desk-lens").
  assert.ok(!["desk", "desk-lens"].includes(deskMemoryKey("explore", "depth")));
});
