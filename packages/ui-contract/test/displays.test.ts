// PB9 (playbook and switchboard design 3.2): screens on this computer and Split.
import { test } from "node:test";
import assert from "node:assert/strict";
import { SPLITS, displayLink, displayMap, displaysFrom, isSplit, partLinks, screenKey } from "../src/displays.ts";

const join = "https://v4.example/players.html?join=room1&seat=bridge";

test("a display's link is the role's join link, local and named; whole has no split", () => {
  const u = new URL(displayLink(join, "Wall TV"));
  assert.equal(u.searchParams.get("join"), "room1");
  assert.equal(u.searchParams.get("seat"), "bridge");
  assert.equal(u.searchParams.get("local"), "1");
  assert.equal(u.searchParams.get("name"), "Wall TV");
  assert.equal(u.searchParams.has("split"), false);
  assert.equal(new URL(displayLink(displayLink(join, "W", "two"), "W", "whole")).searchParams.has("split"), false);
});

test("a split window's parts are unsplit join links, numbered, each named; a whole one is itself", () => {
  const parts = partLinks(displayLink(join, "Wall TV", "three"));
  assert.equal(parts.length, 3);
  assert.deepEqual(parts.map((p) => new URL(p).searchParams.get("part")), ["1", "2", "3"]);
  assert.deepEqual(parts.map((p) => new URL(p).searchParams.get("name")), ["Wall TV 1", "Wall TV 2", "Wall TV 3"]);
  assert.ok(parts.every((p) => !new URL(p).searchParams.has("split") && new URL(p).searchParams.get("local") === "1"));
  const whole = displayLink(join, "Wall TV");
  assert.deepEqual(partLinks(whole), [whole]);
  for (const s of Object.keys(SPLITS)) assert.equal(SPLITS[s as keyof typeof SPLITS].areas.join(" ").match(/p\d/g)!.length >= SPLITS[s as keyof typeof SPLITS].parts, true);
  assert.equal(isSplit("four"), true);
  assert.equal(isSplit("toString"), false);
});

test("displays get plain, distinct labels and a map in their real positions", () => {
  const ds = displaysFrom([
    { label: "", left: 0, top: 0, width: 1920, height: 1080, isPrimary: true },
    { label: "LG TV", left: 1920, top: 0, width: 3840, height: 2160 },
    { label: "LG TV", left: -1280, top: 200, width: 1280, height: 1024 },
  ]);
  assert.deepEqual(ds.map((d) => d.label), ["This screen", "LG TV", "LG TV 2"]);
  const m = displayMap(ds);
  assert.equal(m[2]!.x, 0, "the leftmost display starts the map");
  assert.ok(m[1]!.x > m[0]!.x);
  assert.ok(Math.max(...m.map((b) => b.x + b.w)) <= 100.0001);
});

test("each part of a split window keeps its own screen id", () => {
  assert.equal(screenKey("r", null), "sse4.screen.r");
  assert.equal(screenKey("r", "2"), "sse4.screen.r.2");
  assert.equal(screenKey(null, "9"), "sse4.screen.alone");
});
