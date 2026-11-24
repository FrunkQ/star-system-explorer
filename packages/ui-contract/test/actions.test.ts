// The one action list (V3 gap 38): the object menu, the Build tools and More read it; the menu follows the desk.
import { test } from "node:test";
import assert from "node:assert/strict";
import { OBJECT_ACTIONS, actionById, keyWords, listWords, menuFor, type ActionTarget } from "../src/actions.ts";

const tarn: ActionTarget = { address: "u:k/sys:1/body:3", kind: "planet", label: "Tarn" };
const belt: ActionTarget = { address: "u:k/sys:1/belt:0", kind: "belt", label: "The Shoal" };
const moon: ActionTarget = { address: "u:k/sys:1/body:3/body:0", kind: "moon", label: "Skerry" };

test("every action has a unique id, a desk and words", () => {
  const ids = OBJECT_ACTIONS.map((a) => a.id);
  assert.equal(new Set(ids).size, ids.length);
  for (const a of OBJECT_ACTIONS) {
    assert.ok(a.desks.length, a.id);
    assert.ok(a.label(tarn).length, a.id);
  }
});

test("Explore offers looking and the table; nothing that changes the universe", () => {
  const m = menuFor(tarn, { desk: "explore", sharing: true, canDescribe: true });
  assert.deepEqual(m.items.map((i) => i.id), ["open", "describe", "link", "send", "copy", "land"]);
  assert.ok(m.items.every((i) => !i.why));
  assert.equal(m.items[0]!.label, "Open Tarn");
  assert.deepEqual(m.elsewhere, [
    { desk: "build", words: "On Build: Paste here, Add here and Cut" },
    { desk: "run", words: "On Run: Voyage" },
  ]);
});

test("Build offers changing, and says why an action cannot be done", () => {
  const m = menuFor(tarn, { desk: "build", canCut: false });
  assert.deepEqual(m.items.map((i) => i.id), ["open", "describe", "link", "copy", "paste", "add", "cut"]);
  assert.match(m.items.find((i) => i.id === "cut")!.why!, /Only a body added to this campaign/);
  assert.match(m.items.find((i) => i.id === "describe")!.why!, /agent key/);
  assert.equal(m.items.find((i) => i.id === "paste")!.why, undefined);
  // A what-if branch holds pasting and adding until it is kept.
  const b = menuFor(tarn, { desk: "build", canCut: true, branch: true });
  assert.match(b.items.find((i) => i.id === "add")!.why!, /what-if/);
});

test("Run offers ships and the table", () => {
  const m = menuFor(moon, { desk: "run" });
  assert.deepEqual(m.items.map((i) => i.id), ["open", "describe", "link", "send", "voyage", "land"]);
  assert.match(m.items.find((i) => i.id === "send")!.why!, /Share with the players first/);
  assert.equal(m.items.find((i) => i.id === "land")!.label, "Land on Skerry");
  // Add here is for stars and planets: a moon's Build line leaves it out.
  assert.ok(!m.elsewhere.find((e) => e.desk === "build")!.words.includes("Add"));
});

test("a belt is an area: no close-up, no landing, no link of its own", () => {
  const m = menuFor(belt, { desk: "run", sharing: true });
  assert.deepEqual(m.items.map((i) => i.id), ["describe"]);
});

test("keys and lists read as words", () => {
  assert.equal(keyWords(actionById("cut")!.key!), "Ctrl+X");
  assert.equal(listWords(["a"]), "a");
  assert.equal(listWords(["a", "b", "c"]), "a, b and c");
});
