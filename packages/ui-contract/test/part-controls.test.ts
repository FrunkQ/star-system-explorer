// AT13 (atlas design 11A.4): a part declares its controls; a screen item switches them; "none" makes a picture.
import { test } from "node:test";
import assert from "node:assert/strict";
import { checkItemControls, controlLines, controlOn, isPicture, resolveControls, type PartControl } from "../src/part-controls.ts";
import { checkScreen, type ScreenDoc } from "../src/screen.ts";
import { CARD_CONTROLS, CARD_PARAMS, CHART_CONTROLS, CHART_PARAMS, TABLE_CONTROLS, TABLE_PARAMS, TEXT_PARAMS } from "../../views/player-stack/src/schemas.ts";

const TABLE: PartControl[] = TABLE_CONTROLS;
const VIEWS = [
  { id: "sse.view.chart", params: CHART_PARAMS, switchable: CHART_CONTROLS },
  { id: "sse.view.card", params: CARD_PARAMS, switchable: CARD_CONTROLS },
  { id: "sse.view.table", params: TABLE_PARAMS, switchable: TABLE_CONTROLS },
  { id: "sse.view.text", params: TEXT_PARAMS },
] as never[];
const screen = (items: ScreenDoc["items"]): ScreenDoc => ({ schema: "sse.screen/1", id: "t", label: "T", audience: "players", layout: "grid", theme: "chart", symbology: "s", vocab: "plain", items });

test("unstated controls take the part's default; all and none switch every one", () => {
  assert.deepEqual(resolveControls(TABLE, undefined), { sort: true, select: true });
  assert.deepEqual(resolveControls(TABLE, { sort: false }), { sort: false, select: true });
  assert.deepEqual(resolveControls(TABLE, "none"), { sort: false, select: false });
  assert.deepEqual(resolveControls([{ id: "print", words: "Print", kind: "button", default: false }], "all"), { print: true });
  assert.equal(controlOn(undefined, "sort"), true, "a host that says nothing leaves a control on");
  assert.equal(controlOn({ sort: false }, "sort"), false);
});

test("a part with every control off is a picture; none is always one", () => {
  assert.equal(isPicture(TABLE, { sort: false, select: false }), true);
  assert.equal(isPicture(TABLE, { sort: false }), false);
  assert.equal(isPicture(undefined, "none"), true);
  assert.equal(isPicture(undefined, undefined), false, "a part with no controls is not made inert unless asked");
});

test("a screen naming a control its part has not declared is refused, with what it has", () => {
  assert.deepEqual(checkItemControls(TABLE, { filter: true }), ["no control filter (it has sort, select)"]);
  assert.deepEqual(checkItemControls(undefined, { sort: true }), ["no control sort (it has none)"]);
  assert.deepEqual(checkItemControls(TABLE, { sort: "yes" }), ["control sort must be true or false"]);
  assert.equal(checkItemControls(TABLE, ["sort"]).length, 1);
  const bad = checkScreen(screen([{ view: "sse.view.table", area: "a", controls: { filter: true } }]), VIEWS);
  assert.equal(bad.ok, false);
  assert.match((bad as { errors: string[] }).errors[0]!, /sse.view.table: no control filter/);
});

test("checked mounts carry each part's controls and whether it is a picture", () => {
  const c = checkScreen(screen([
    { view: "sse.view.chart", area: "a", controls: "none" },
    { view: "sse.view.table", area: "b", controls: { sort: false } },
    { view: "sse.view.card", area: "c" },
    { view: "sse.view.text", area: "d" },
  ]), VIEWS);
  assert.equal(c.ok, true);
  if (!c.ok) return;
  assert.deepEqual(c.screen.mounts.map((m) => [m.module.id, m.controls, m.picture]), [
    ["sse.view.chart", { select: false }, true],
    ["sse.view.table", { sort: false, select: true }, false],
    ["sse.view.card", { open: true }, false],
    ["sse.view.text", {}, false],
  ]);
});

test("the catalogue line names each control, its kind and default", () => {
  assert.equal(controlLines(TABLE), "- sort (button, on by default): Sort by a column, pressing its heading\n- select (link, on by default): Pick a world by its name in the table");
});
