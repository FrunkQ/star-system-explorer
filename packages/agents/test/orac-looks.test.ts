// A2 part 2 acceptance: styles, player screens, symbologies and data views.
// Each draft goes through the checker the GM's own editor uses; a draft
// with a problem goes back with that checker's words and is never offered;
// accepting is the only thing that keeps, opens or applies anything.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import type { ObserveSetup, ScreenDoc, StyleDoc, SymbologyDoc } from "../../ui-contract/src/index.ts";
import { CARD_PARAMS, CHART_PARAMS, READOUT_PARAMS, TABLE_PARAMS, TEXT_PARAMS } from "../../views/player-stack/src/schemas.ts";
import { asker, emptyAgents, type AgentsState } from "../src/agents.ts";
import { dataViewEntries, dataViewTarget, lookEntries, screenTarget, styleTarget, symbologyTarget } from "../src/orac-looks.ts";
import { runOrac } from "../src/orac.ts";
// @ts-expect-error plain JS test helper
import { mockProvider } from "./mock-provider.mjs";

const root = new URL("../../../", import.meta.url);
const json = (p: string) => JSON.parse(readFileSync(new URL(p, root), "utf8"));
const fence = (o: unknown) => "```json\n" + JSON.stringify(o) + "\n```";
const VIEWS = [
  { id: "sse.view.chart", label: "Chart", scene: "player" as const, params: CHART_PARAMS },
  { id: "sse.view.card", label: "Card", scene: "player" as const, params: CARD_PARAMS },
  { id: "sse.view.table", label: "Table", scene: "player" as const, params: TABLE_PARAMS },
  { id: "sse.view.text", label: "Text", scene: "player" as const, params: TEXT_PARAMS },
  { id: "sse.view.readout", label: "Readout", scene: "player" as const, params: READOUT_PARAMS },
];
const STYLES: StyleDoc[] = json("packages/shell/src/documents/styles.json").styles;
const SYMBOLOGIES: SymbologyDoc[] = ["diagrammatic", "colour-coded", "mixed"].map((n) => json(`packages/shell/src/documents/symbology.${n}.json`));
const SETUP: ObserveSetup = json("packages/shell/src/documents/kestrel.player-data.pack.json").provides.other.observe;
const linkedTo = (base: string): AgentsState => ({ ...emptyAgents(), providers: [{ id: "x", kind: "other", label: "X", base }], links: { building: { provider: "x", model: "mock/large" } } });
const made = { kind: "", doc: null };

test("styles: the view's own schema and the device tier decide; accept keeps it", async () => {
  const kept: StyleDoc[] = [];
  const t = styleTarget({ registry: () => STYLES, views: () => VIEWS, tier: () => "basic", keep: (d) => kept.push(d) });
  const base = STYLES.find((s) => s.view === "sse.view.chart")!;
  const bad = await t.check({ schema: "sse.style/1", id: base.id, name: "x", view: "sse.view.chart", settings: { glow: 3 } }, made);
  assert.ok(bad.problems.some((p) => /starts with "mine\."/.test(p)) && bad.problems.some((p) => /already a style/.test(p)));
  const unknown = await t.check({ schema: "sse.style/1", id: "mine.glow", name: "Glow", view: "sse.view.chart", extends: base.id, settings: { glow: 3 } }, made);
  assert.deepEqual(unknown.problems, ["unknown parameter glow"]);
  const good = await t.check({ schema: "sse.style/1", id: "mine.big", name: "Big marks", view: "sse.view.chart", extends: base.id, settings: { mark_scale: 2 } }, made);
  assert.deepEqual(good.problems, []);
  assert.equal(kept.length, 0, "nothing kept before accept");
  await t.accept(good.value!);
  assert.equal(kept[0]!.id, "mine.big");
  assert.match(lookEntries({ views: VIEWS, styles: STYLES, symbologies: [] }).find((e) => e.key === "view:sse.view.chart")!.full, /- mark_scale \(number/);
});

test("screens: the composer's checks, theme and symbology names; accept opens it", async () => {
  const opened: ScreenDoc[] = [];
  const t = screenTarget({ views: () => VIEWS, taken: () => ["sse.screen.players.bridge"], dataViews: () => SETUP.views.map((v) => v.id), styles: () => ({ registry: STYLES, tier: "standard" }), themes: () => ["chart-room", "green-phosphor"], symbologies: () => SYMBOLOGIES.map((s) => s.id), open: (d) => opened.push(d) });
  const doc = { schema: "sse.screen/1", id: "mine.sensors", label: "Sensors", audience: "players", layout: "grid", theme: "green-phosphor", symbology: SYMBOLOGIES[0]!.id, vocab: "plain", data_view: "scanned", items: [{ view: "sse.view.chart", area: "a" }, { view: "sse.view.table", area: "b" }] };
  assert.deepEqual((await t.check(doc, made)).problems, []);
  const bad = await t.check({ ...doc, theme: "lcars", data_view: "deep", items: [{ view: "sse.view.map", area: "a" }, { view: "sse.view.text", area: "z" }] }, made);
  assert.ok(bad.problems.some((p) => /no theme lcars/.test(p)));
  assert.ok(bad.problems.some((p) => /no data view deep/.test(p)));
  assert.ok(bad.problems.some((p) => /no view module sse\.view\.map/.test(p)));
  assert.ok(bad.problems.some((p) => /has no area z/.test(p)));
  const gm = await t.check({ ...doc, audience: "gm" }, made);
  assert.ok(gm.problems.some((p) => /audience/.test(p)), "Orac drafts player screens only");
  assert.equal(opened.length, 0);
});

test("symbologies: checkSymbology decides; accept keeps it", async () => {
  const kept: SymbologyDoc[] = [];
  const t = symbologyTarget({ registry: () => SYMBOLOGIES, keep: (d) => kept.push(d) });
  const bad = await t.check({ schema: "sse.symbology/1", id: "mine.warm", name: "Warm", inherits: SYMBOLOGIES[0]!.id, bindings: [{ applies_to: "*", channel: "colour", from: "class", scale: { table: { "planet/terrestrial": "red" } } }] }, made);
  assert.deepEqual(bad.problems, ["colour red must be #RRGGBB or palette:N"]);
  const good = await t.check({ schema: "sse.symbology/1", id: "mine.warm", name: "Warm", inherits: SYMBOLOGIES[0]!.id, bindings: [{ applies_to: "*", channel: "colour", from: "class", scale: { table: { "planet/terrestrial": "#C05030" } } }] }, made);
  assert.deepEqual(good.problems, []);
  assert.equal(kept.length, 0);
  await t.accept(good.value!);
  assert.equal(kept[0]!.id, "mine.warm");
});

test("data views: Player data's setup check, never a GM-only tier; accept applies the whole setup", async () => {
  const applied: ObserveSetup[] = [];
  const t = dataViewTarget({ setup: () => SETUP, apply: async (s) => (applied.push(s), null) });
  const view = { id: "orbits-only", label: "Orbits only", show_tiers: ["obvious"], tags: { mode: "none" }, unknown_style: "omit" };
  const gmOnly = await t.check({ ...view, show_tiers: ["obvious", "secret"] }, made);
  assert.deepEqual(gmOnly.problems, ["view orbits-only would show GM-only facts to players"]);
  const twice = await t.check({ ...view, id: "scanned" }, made);
  assert.deepEqual(twice.problems, ["two views are called scanned"]);
  const override = await t.check({ ...view, overrides: [{ target: { kind: "field", id: "warp_signature" }, show: true }] }, made);
  assert.match(override.problems[0]!, /no such field or tag/);
  const good = await t.check(view, made);
  assert.deepEqual(good.problems, []);
  assert.equal(SETUP.views.length, 5, "the campaign's setup is not touched by checking");
  assert.equal(applied.length, 0);
  assert.equal(await t.accept(good.value!), null);
  assert.deepEqual(applied[0]!.views.map((v) => v.id), [...SETUP.views.map((v) => v.id), "orbits-only"]);
  assert.match(dataViewEntries(SETUP)[0]!.full, /tiers in order: obvious, scanned, surveyed, secret/);
});

test("Orac drafts a screen: the composer's problems go back, the fixed one is offered, nothing opens before accept", async () => {
  const opened: ScreenDoc[] = [];
  const target = screenTarget({ views: () => VIEWS, taken: () => [], dataViews: () => SETUP.views.map((v) => v.id), styles: () => ({ registry: STYLES, tier: "standard" }), themes: () => ["chart-room", "green-phosphor"], symbologies: () => SYMBOLOGIES.map((s) => s.id), open: (d) => opened.push(d) });
  const screen = (area: string) => ({ schema: "sse.screen/1", id: "mine.helm", label: "Helm", audience: "players", layout: "stack", theme: "green-phosphor", symbology: SYMBOLOGIES[0]!.id, vocab: "plain", items: [{ view: "sse.view.chart", area: "1" }, { view: "sse.view.readout", area, params: { field: "surface.temp_mean_k", style: "gauge" } }] });
  const p = await mockProvider({ replies: [fence({ note: "A helm screen.", make: [{ kind: "screen", title: "Helm", doc: screen("b") }] }), fence({ note: "Fixed the area.", make: [{ kind: "screen", title: "Helm", doc: screen("2") }] })] });
  try {
    const r = await runOrac({ request: "A green-screen helm display: the chart, then a temperature gauge.", targets: [target], catalogue: lookEntries({ views: VIEWS, styles: STYLES, symbologies: SYMBOLOGIES }), ask: asker(linkedTo(p.base), "building") });
    assert.equal(r.ok, true, r.message);
    assert.match(r.rounds[0]!.problems[0]!, /layout stack has no area b/);
    assert.equal(r.passed.length, 1);
    assert.equal(opened.length, 0);
    const o = r.passed[0]!.value!;
    assert.match(o.target.summary(o.value), /^Helm: stack layout, chart in 1, readout in 2/);
    await o.target.accept(o.value);
    assert.equal(opened[0]!.id, "mine.helm");
  } finally {
    await p.close();
  }
});
