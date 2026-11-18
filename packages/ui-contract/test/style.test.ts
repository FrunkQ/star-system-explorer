// V11 acceptance (H8.9) at the contract level: every player view exposes a
// described schema with a few promoted settings; style documents inherit,
// and an out-of-range or over-budget document is rejected with reasons; the
// shipped styles pass; the tier table matches the kernel's.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { checkStyle, resolveStyle, settingCost, settingsCost, promotedOf, settingsDiff, TIER_RUNS_UP_TO, type StyleDoc } from "../src/style.ts";
import { checkScreen } from "../src/screen.ts";
import { CARD_PARAMS, CHART_PARAMS, READOUT_PARAMS, TABLE_PARAMS, TEXT_PARAMS } from "../../views/player-stack/src/schemas.ts";
import type { ParamSpec } from "../src/view.ts";

const VIEWS = [
  { id: "sse.view.chart", params: CHART_PARAMS },
  { id: "sse.view.card", params: CARD_PARAMS },
  { id: "sse.view.table", params: TABLE_PARAMS },
  { id: "sse.view.text", params: TEXT_PARAMS },
  { id: "sse.view.readout", params: READOUT_PARAMS },
];
const SHIPPED: StyleDoc[] = JSON.parse(readFileSync(new URL("../../shell/src/documents/styles.json", import.meta.url), "utf8")).styles;

export function schemaProblems(id: string, params: readonly ParamSpec[]): string[] {
  const out: string[] = [];
  const promoted = params.filter((p) => p.promoted).length;
  if (promoted > 5) out.push(`${id} promotes ${promoted} settings (at most 5)`);
  if (params.length >= 3 && promoted < 1) out.push(`${id} promotes nothing`);
  for (const p of params) {
    if (!/^[a-z][a-z0-9_]*$/.test(p.name)) out.push(`${id}.${p.name}: name`);
    if (!p.meaning || p.meaning.length < 8) out.push(`${id}.${p.name}: needs a meaning`);
    if ((p.type === "number" || p.type === "integer") && !p.range && p.promoted) out.push(`${id}.${p.name}: a promoted number needs a range`);
    if (p.type === "enum" && !p.options?.length) out.push(`${id}.${p.name}: enum without options`);
  }
  return out;
}

test("every player view exposes a described schema with a few promoted settings", () => {
  for (const v of VIEWS) {
    assert.ok(v.params.length >= 4, `${v.id} has ${v.params.length} settings`);
    assert.deepEqual(schemaProblems(v.id, v.params), []);
    // Defaults are valid against the view's own schema.
    const r = checkScreen({ schema: "sse.screen/1", id: "t", label: "t", audience: "players", layout: "page", theme: "", symbology: "", vocab: "plain", items: [{ view: v.id, area: "1" }] }, [{ ...v, version: "0", tiers: [], inputs: [], mount: () => ({ update() {}, dispose() {} }) }]);
    assert.equal(r.ok, true, v.id);
  }
});

test("the shipped styles pass on a standard device; the orrery is over budget on a basic one", () => {
  for (const d of SHIPPED) {
    const c = checkStyle(d.id, SHIPPED, VIEWS, "standard");
    assert.ok(c.ok, `${d.id}: ${c.ok ? "" : c.errors.join("; ")}`);
  }
  const orrery = checkStyle("sse.style.chart.orrery", SHIPPED, VIEWS, "basic");
  assert.ok(!orrery.ok && /over budget: animate cost medium, and a basic device runs up to light/.test(orrery.errors.join()));
  const ok = checkStyle("sse.style.chart.orrery", SHIPPED, VIEWS, "capable");
  assert.ok(ok.ok && ok.cost === "medium" && ok.by[0] === "animate");
});

test("styles inherit; a tweak is a small difference", () => {
  const r = resolveStyle("sse.style.chart.table-screen", SHIPPED);
  assert.deepEqual(r.chain, ["sse.style.chart.table-screen", "sse.style.chart.plain"]);
  assert.equal(r.settings.labels, true);
  assert.equal(r.settings.label_size, 21);
  const d = settingsDiff(resolveStyle("sse.style.chart.plain", SHIPPED).settings, r.settings);
  assert.deepEqual(d.map((x) => x.name), ["label_size", "mark_scale", "show_scale"]);
});

test("out-of-range, wrong-type, unknown and mis-inherited documents are rejected with reasons", () => {
  const base = (settings: Record<string, unknown>, extra: Partial<StyleDoc> = {}): StyleDoc[] => [...SHIPPED, { schema: "sse.style/1", id: "x", name: "x", view: "sse.view.chart", settings: settings as StyleDoc["settings"], ...extra }];
  const range = checkStyle("x", base({ mark_scale: 9 }), VIEWS, "capable");
  assert.ok(!range.ok && /mark_scale: must be within 0.5 to 2.5/.test(range.errors.join()));
  const type = checkStyle("x", base({ labels: "yes" }), VIEWS, "capable");
  assert.ok(!type.ok && /labels: must be true or false/.test(type.errors.join()));
  const unknown = checkStyle("x", base({ sparkle: true }), VIEWS, "capable");
  assert.ok(!unknown.ok && /unknown parameter sparkle/.test(unknown.errors.join()));
  const promote = checkStyle("x", base({}, { promote: ["sparkle"] }), VIEWS, "capable");
  assert.ok(!promote.ok && /promotes sparkle/.test(promote.errors.join()));
  const other = checkStyle("x", base({}, { extends: "sse.style.card.brief" }), VIEWS, "capable");
  assert.ok(!other.ok && /for another view/.test(other.errors.join()));
  const loop = checkStyle("x", base({}, { extends: "x" }), VIEWS, "capable");
  assert.ok(!loop.ok && /loop/.test(loop.errors.join()));
  // Inherited values are checked too: a bad base fails its children.
  const kids: StyleDoc[] = [...base({ label_size: 99 }), { schema: "sse.style/1", id: "y", name: "y", view: "sse.view.chart", extends: "x", settings: {} }];
  assert.equal(checkStyle("y", kids, VIEWS, "capable").ok, false);
});

test("a screen naming a style is checked through it, including the device budget", () => {
  const screen = (style: string, params?: Record<string, unknown>) => ({ schema: "sse.screen/1" as const, id: "s", label: "s", audience: "players" as const, layout: "page" as const, theme: "", symbology: "", vocab: "plain" as const, items: [{ view: "sse.view.chart", area: "1", style, ...(params ? { params: params as never } : {}) }] });
  const mods = VIEWS.map((v) => ({ ...v, version: "0", tiers: [], inputs: [], mount: () => ({ update() {}, dispose() {} }) }));
  const ok = checkScreen(screen("sse.style.chart.table-screen"), mods, undefined, { registry: SHIPPED, tier: "basic" });
  assert.ok(ok.ok && ok.screen.mounts[0]!.params.label_size === 21);
  const over = checkScreen(screen("sse.style.chart.orrery"), mods, undefined, { registry: SHIPPED, tier: "basic" });
  assert.ok(!over.ok && /over budget/.test(over.errors.join()));
  // The item's own params go on top of the style.
  const tweak = checkScreen(screen("sse.style.chart.table-screen", { label_size: 12 }), mods, undefined, { registry: SHIPPED, tier: "basic" });
  assert.ok(tweak.ok && tweak.screen.mounts[0]!.params.label_size === 12);
  const plainOver = checkScreen({ ...screen("x"), items: [{ view: "sse.view.chart", area: "1", params: { animate: true } }] }, mods, undefined, { registry: SHIPPED, tier: "basic" });
  assert.ok(!plainOver.ok && /over budget/.test(plainOver.errors.join()));
  assert.ok(!checkScreen(screen("sse.style.card.brief"), mods, undefined, { registry: SHIPPED, tier: "basic" }).ok, "a card style on a chart item");
  assert.ok(!checkScreen(screen("nope"), mods, undefined, { registry: SHIPPED, tier: "basic" }).ok);
});

test("cost rules: booleans when on, numbers above default, enums by option", () => {
  const b: ParamSpec = { name: "a", type: "boolean", default: false, meaning: "an animation", cost: "medium" };
  assert.equal(settingCost(b, false), null);
  assert.equal(settingCost(b, true), "medium");
  const n: ParamSpec = { name: "n", type: "integer", default: 10, meaning: "particle count", cost: "heavy", range: [0, 100] };
  assert.equal(settingCost(n, 10), null);
  assert.equal(settingCost(n, 50), "heavy");
  const e: ParamSpec = { name: "q", type: "enum", default: "low", options: ["low", "high"], meaning: "quality", cost_options: { high: "heavy" } };
  assert.equal(settingCost(e, "high"), "heavy");
  assert.deepEqual(settingsCost([b, n, e], { a: true, n: 50 }), { cost: "heavy", by: ["n"] });
  assert.deepEqual(promotedOf({ params: CHART_PARAMS }).map((p) => p.name), ["labels", "mark_scale", "animate"]);
  assert.deepEqual(promotedOf({ params: CHART_PARAMS }, SHIPPED.find((d) => d.id === "sse.style.chart.orrery")).map((p) => p.name), ["label_size", "mark_scale", "animate"]);
});

test("the tier budgets are the kernel's tier table", () => {
  const rs = readFileSync(new URL("../../../crates/sse-kernel/src/protocol.rs", import.meta.url), "utf8");
  const kernel = Object.fromEntries([...rs.matchAll(/tier: "(\w+)"\.into\(\),[\s\S]*?runs_up_to: Cost::(\w+)/g)].map((m) => [m[1], m[2]!.toLowerCase()]));
  assert.deepEqual(kernel, TIER_RUNS_UP_TO);
});
