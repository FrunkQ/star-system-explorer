// PB13 (playbook and switchboard design 7.4): layouts are screen documents the GM arranges; boxes move, swap and go.
import { test } from "node:test";
import assert from "node:assert/strict";
import { addBox, arrangementOf, blankLayout, copyLayout, keepLayout, layoutIdFor, layoutsFrom, moveBox, rearrange, removeBox, sorted, swapBoxes } from "../src/layouts.ts";
import { showOf } from "../src/playbook.ts";
import type { ScreenDoc } from "../src/screen.ts";

const bridge: ScreenDoc = {
  schema: "sse.screen/1",
  id: "sse.screen.players.bridge",
  label: "Bridge",
  audience: "players",
  layout: "grid",
  theme: "chart-room",
  symbology: "s",
  vocab: "plain",
  items: [
    { view: "sse.view.chart", area: "a" },
    { view: "sse.view.card", area: "b" },
    { view: "sse.view.table", area: "c", controls: { sort: false } },
  ],
  rail: [],
};
const views = (d: ScreenDoc) => sorted(d).map((i) => i.view.replace("sse.view.", ""));

test("a new layout is a blank copy with its own mine. id; a shipped one is edited as a copy", () => {
  const b = blankLayout(bridge, "Wall chart", []);
  assert.equal(b.id, "mine.wall-chart");
  assert.equal(b.items.length, 0);
  assert.equal(arrangementOf(b), "blank");
  assert.equal(layoutIdFor("Wall chart", ["mine.wall-chart"]), "mine.wall-chart-2");
  const c = copyLayout(bridge, []);
  assert.equal(c.id, "mine.bridge");
  assert.equal(c.rail, undefined, "a made layout is not on the GM's rail");
  assert.equal(copyLayout({ ...bridge, id: "mine.x" }, []).id, "mine.x", "a made layout keeps its id");
});

test("boxes are added on the next free area, moved, swapped and taken off; areas stay put", () => {
  let d = addBox(blankLayout(bridge, "W", []), "sse.view.chart");
  assert.deepEqual(d.items, [{ view: "sse.view.chart", area: "1" }]);
  assert.equal(addBox(d, "sse.view.card"), d, "One picture holds one box");
  d = rearrange(bridge, "report");
  assert.equal(d.layout, "page");
  assert.deepEqual(d.items.map((i) => i.area), ["1", "2", "3"]);
  assert.deepEqual(d.items[2]!.controls, { sort: false }, "a box keeps its controls when the arrangement changes");
  assert.deepEqual(views(moveBox(d, 2, 0)), ["table", "chart", "card"]);
  assert.deepEqual(views(swapBoxes(d, 0, 2)), ["table", "card", "chart"]);
  assert.deepEqual(views(removeBox(d, 1)), ["chart", "table"]);
  assert.deepEqual(removeBox(d, 1).items.map((i) => i.area), ["1", "2"]);
  assert.equal(rearrange(bridge, "one").items.length, 1, "extra boxes go when there is no room");
});

test("layouts from a save: only screen documents with mine. ids, each once; keeping replaces by id", () => {
  const mine = { ...bridge, id: "mine.a" };
  assert.deepEqual(layoutsFrom([mine, mine, bridge, { schema: "x" }, null]).map((d) => d.id), ["mine.a"]);
  assert.deepEqual(layoutsFrom(undefined), []);
  const kept = keepLayout([mine], { ...mine, label: "Renamed" });
  assert.equal(kept.length, 1);
  assert.equal(kept[0]!.label, "Renamed");
  assert.equal(keepLayout(kept, { ...mine, id: "mine.b" }).length, 2);
});

test("a Show step that names a layout sends it with the place", () => {
  const p = { id: "p", title: "Wreck", to: "table", steps: [] };
  const bm = [{ schema: "sse.bookmark/1", id: "b1", name: "Wreck", scope: "u:k/sys:1" }] as never;
  assert.deepEqual(showOf(p as never, { show: { bookmark: "b1", layout: "mine.wall" } }, bm), { scope: "u:k/sys:1", title: "Wreck", screen: "mine.wall" });
  assert.equal("screen" in showOf(p as never, { show: { bookmark: "b1" } }, bm)!, false);
});
