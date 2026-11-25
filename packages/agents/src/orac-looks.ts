// Orac's targets for how SSE looks and what players learn (A2 part 2):
// style documents, player screens, symbologies and data views. As in part 1,
// each check is the checker the GM's own editor uses (checkStyle on the
// Styles screen, checkScreen in the Screens composer, checkSymbology on the
// Symbols screen, checkSetup on Player data) and each accept is the GM's own
// path (keep the style, hand the screen to the players page, keep the
// symbology, apply the setup as the campaign's player-data pack). New
// documents get new ids ("mine." for styles, screens and symbologies), so a
// shipped document is never overwritten.
import {
  checkScreen,
  checkSetup,
  checkStyle,
  checkSymbology,
  CHANNELS,
  controlLines,
  LAYOUT_AREAS,
  type DataView,
  type ObserveSetup,
  type ScreenDoc,
  type StyleDoc,
  type SymbologyDoc,
  type ViewModule,
} from "../../ui-contract/src/index.ts";
import type { Entry, OracTarget } from "./orac.ts";

type AnyView = Pick<ViewModule<unknown, unknown>, "id" | "params" | "switchable"> & { label?: string; scene?: "system" | "player" };
const obj = (x: unknown): x is Record<string, unknown> => !!x && typeof x === "object" && !Array.isArray(x);
const newId = (id: unknown, taken: readonly string[], what: string): string[] => {
  if (typeof id !== "string" || !id) return [`a ${what} needs an id`];
  const e: string[] = [];
  if (!id.startsWith("mine.")) e.push(`a new ${what} id starts with "mine."`);
  if (taken.includes(id)) e.push(`there is already a ${what} ${id}`);
  return e;
};

/** A view's parameters as Orac reads them: name, type, range or options, default, meaning, cost. */
export function paramLines(v: AnyView): string {
  return v.params
    .map((p) => {
      const range = p.range ? ` ${p.range[0]} to ${p.range[1]}` : p.options ? ` one of ${p.options.join(" | ")}` : "";
      const cost = p.cost ? `; cost ${p.cost} when on` : p.cost_options ? `; cost by option ${JSON.stringify(p.cost_options)}` : "";
      return `- ${p.name} (${p.type}${range}${p.unit ? `, ${p.unit}` : ""}; default ${JSON.stringify(p.default)}${cost}): ${p.meaning}`;
    })
    .join("\n");
}

/** Views, styles and symbologies as catalogue entries. */
export function lookEntries(o: { views: readonly AnyView[]; styles: readonly StyleDoc[]; symbologies: readonly SymbologyDoc[] }): Entry[] {
  return [
    // AT13: a part's own controls are a column of the catalogue; a screen item's `controls` switches them.
    ...o.views.map((v) => ({
      key: `view:${v.id}`,
      line: `${v.label ?? v.id}${v.scene === "player" ? " (player screens)" : ""}; ${v.params.length} settings${v.switchable?.length ? `, ${v.switchable.length} controls` : ""}`,
      full: `settings:\n${paramLines(v)}${v.switchable?.length ? `\ncontrols (a screen item's "controls": "none", "all", or {"<id>": true|false}):\n${controlLines(v.switchable)}` : ""}`,
    })),
    ...o.styles.map((s) => ({ key: `style:${s.id}`, line: `${s.name} for ${s.view}${s.description ? `: ${s.description}` : ""}`, full: JSON.stringify(s) })),
    ...o.symbologies.map((s) => ({ key: `symbology:${s.id}`, line: `${s.name}${s.description ? `: ${s.description}` : ""}`, full: JSON.stringify(s) })),
  ];
}

/**
 * Styles (H8.9): a named set of settings for one view, which may extend
 * another style. Checked against the view's schema and this device's tier.
 */
export function styleTarget(o: { registry: () => readonly StyleDoc[]; views: () => readonly AnyView[]; tier: () => string; keep: (d: StyleDoc) => void }): OracTarget<StyleDoc> {
  return {
    kind: "style",
    what: "A style: named settings for one view (a chart, a card, a readout...), which may extend another style so a tweak is a small difference. Use only settings the view has (need view:<id>).",
    form: '{"schema": "sse.style/1", "id": "mine.<short-name>", "name": "<name>", "description": "<one line>", "view": "<view id>", "extends": "<style id, optional>", "settings": {"<setting>": <value>}, "promote": ["<setting offered as a plain control>", ...]}',
    async check(doc) {
      if (!obj(doc)) return { problems: ["a style is one JSON object"] };
      const d = doc as unknown as StyleDoc;
      const problems = newId(d.id, o.registry().map((s) => s.id), "style");
      if (typeof d.name !== "string" || !d.name.trim()) problems.push("a style needs a name");
      if (!obj(d.settings)) problems.push("settings must be an object of setting names and values");
      if (problems.length) return { problems };
      const c = checkStyle(d.id, [...o.registry(), d], o.views(), o.tier());
      return c.ok ? { problems: [], value: d } : { problems: c.errors };
    },
    summary: (d) => `${d.name} for ${d.view}${d.extends ? `, on top of ${d.extends}` : ""}: ${Object.entries(d.settings).map(([k, v]) => `${k} ${JSON.stringify(v)}`).join(", ") || "no changes"}`,
    async accept(d) {
      o.keep(d);
      return null;
    },
  };
}

/**
 * Player screens (H8.3): a document that puts player views in a layout's
 * areas, dressed with a theme and seen through a data view. The check is
 * the composer's (every view, area, setting, style and data view). Accepting
 * hands it to the players page on this device, as the composer's Show to
 * players does.
 */
export function screenTarget(o: { views: () => readonly AnyView[]; taken: () => readonly string[]; dataViews: () => readonly string[] | undefined; styles: () => { registry: readonly StyleDoc[]; tier: string } | undefined; themes: () => readonly string[]; symbologies: () => readonly string[]; open: (d: ScreenDoc) => void }): OracTarget<ScreenDoc> {
  return {
    kind: "screen",
    what: "A player screen: player views placed in the areas of one layout, with a theme, symbols and the data view players see it through. Use only player views from the catalogue (need view:<id> for their settings).",
    form: [
      '{"schema": "sse.screen/1", "id": "mine.<short-name>", "label": "<name>", "description": "<one line>", "audience": "players", "layout": "<layout>", "theme": "<theme>", "symbology": "<symbology id>", "vocab": "plain" | "physics" | "story", "data_view": "<data view id, optional>", "items": [{"view": "<view id>", "area": "<area>", "params": {<settings>}, "style": "<style id, optional>", "controls": "none" | "all" | {"<control id>": true | false} (optional; control ids from the catalogue entry of the view)}]}',
      `Layouts and their areas: ${Object.entries(LAYOUT_AREAS).map(([k, a]) => `${k} (${a.join(", ")})`).join("; ")}. Each area holds one item. An item with "controls": "none" is a picture (a wall screen, a stream): nothing on it can be pressed.`,
    ].join("\n"),
    async check(doc) {
      if (!obj(doc)) return { problems: ["a screen is one JSON object"] };
      const d = doc as unknown as ScreenDoc;
      const problems = newId(d.id, o.taken(), "screen");
      const themes = o.themes();
      if (themes.length && !themes.includes(d.theme)) problems.push(`there is no theme ${String(d.theme)}; the themes are ${themes.join(", ")}`);
      const sy = o.symbologies();
      if (sy.length && !sy.includes(d.symbology)) problems.push(`there is no symbology ${String(d.symbology)}; they are ${sy.join(", ")}`);
      if (!["plain", "physics", "story"].includes(d.vocab)) problems.push("vocab is plain, physics or story");
      if (d.audience !== "players" && !(typeof d.audience === "string" && d.audience.startsWith("player:"))) problems.push('a player screen\'s audience is "players" or "player:<seat>"');
      const c = checkScreen(d, o.views() as never, o.dataViews(), o.styles());
      if (!c.ok) problems.push(...c.errors);
      return problems.length ? { problems } : { problems: [], value: d };
    },
    summary: (d) => `${d.label}: ${d.layout} layout, ${d.items.map((i) => `${i.view.replace(/^sse\.view\./, "")} in ${i.area}`).join(", ")}; theme ${d.theme}${d.data_view ? `, seen through ${d.data_view}` : ""}. Accept opens it on the players page on this device.`,
    async accept(d) {
      o.open(d);
      return null;
    },
  };
}

/**
 * Symbologies (H8.4A): how facts become marks (shape, pattern, colour, size,
 * outline, ring, glyph), inheriting or mixing the shipped grammars. Accepting
 * keeps it on this device, where the Symbols screen can choose it.
 */
export function symbologyTarget(o: { registry: () => readonly SymbologyDoc[]; keep: (d: SymbologyDoc) => void }): OracTarget<SymbologyDoc> {
  return {
    kind: "symbology",
    what: "A symbology: how facts (kind, class, tags, numbers) become marks on charts. Inherit a shipped one and change only the bindings you need (need symbology:<id> to read one).",
    form: [
      '{"schema": "sse.symbology/1", "id": "mine.<short-name>", "name": "<name>", "description": "<one line>", "inherits": "<symbology id>", "mix": {"<channel>": "<symbology id>"}, "bindings": [{"applies_to": "<kind or *>", "channel": "<channel>", "from": "kind" | "class" | "tag:<prefix>" | "number:<name>", "scale": {"table": {"<value>": <mark value>}} | {"thresholds": [[<upto>, <mark value>], ...]}, "fallback": <mark value>}]}',
      `Channels: ${CHANNELS.join(", ")}. Colours are #RRGGBB or palette:N; shapes disc, star, diamond, square, triangle; patterns solid, hollow, hatch, stipple, band, cross, grid, dash.`,
    ].join("\n"),
    async check(doc) {
      if (!obj(doc)) return { problems: ["a symbology is one JSON object"] };
      const d = doc as unknown as SymbologyDoc;
      const problems = newId(d.id, o.registry().map((s) => s.id), "symbology");
      if (typeof d.name !== "string" || !d.name.trim()) problems.push("a symbology needs a name");
      if (!Array.isArray(d.bindings)) problems.push("bindings must be a list (empty to change nothing)");
      if (problems.length) return { problems };
      const bad = checkSymbology(d, o.registry());
      return bad ? { problems: [bad] } : { problems: [], value: d };
    },
    summary: (d) => `${d.name}${d.inherits ? `, from ${d.inherits}` : ""}: ${d.bindings.map((b) => `${b.channel} by ${b.from}${b.applies_to !== "*" ? ` for ${b.applies_to}` : ""}`).join(", ") || "inherited as is"}. Accept keeps it; choose it on the Symbols screen.`,
    async accept(d) {
      o.keep(d);
      return null;
    },
  };
}

/**
 * Data views (H6.7): what an audience is shown of each body. Orac drafts one
 * new view; the check is Player data's (checkSetup on the campaign's setup
 * with the view added), and accepting applies it as the campaign's
 * player-data pack, so it is chronicled like the GM's own edit.
 */
export function dataViewTarget(o: { setup: () => ObserveSetup | null; apply: (s: ObserveSetup) => Promise<string | null> }): OracTarget<{ view: DataView; setup: ObserveSetup }> {
  const merged = (v: DataView): ObserveSetup | null => {
    const s = o.setup();
    return s ? { ...structuredClone(s), views: [...s.views.map((x) => structuredClone(x)), v] } : null;
  };
  return {
    kind: "data_view",
    what: "A data view: what players are shown of each body. It shows facts up to chosen tiers of the disclosure ladder, a choice of tags, and how unknown facts read. Fields and tags and the tier each sits at are in the catalogue (need disclosure).",
    form: '{"id": "<lower-case-words-with-hyphens>", "label": "<name>", "description": "<one line>", "show_tiers": ["<tier>", ...], "tags": {"mode": "all" | "none" | "listed", "list": ["<tag>", ...]}, "overrides": [{"target": {"kind": "field" | "tag", "id": "<id>"}, "show": true | false, "precision": {"as": "exact"} | {"as": "rounded", "digits": <n>} | {"as": "band", "step": <n>, "half_width": <n>} | {"as": "hidden"}}], "unknown_style": "placeholder" | "omit", "show_honesty": false}',
    async check(doc) {
      if (!obj(doc)) return { problems: ["a data view is one JSON object"] };
      const v = doc as unknown as DataView;
      const problems: string[] = [];
      if (!Array.isArray(v.show_tiers)) problems.push("show_tiers must be a list of tiers");
      if (!obj(v.tags) || !["all", "none", "listed"].includes(String((v.tags as { mode?: unknown }).mode))) problems.push("tags.mode is all, none or listed");
      if (!["placeholder", "omit"].includes(v.unknown_style)) problems.push("unknown_style is placeholder or omit");
      if (typeof v.label !== "string") problems.push("a data view needs a label");
      const s = merged(v);
      if (!s) problems.push("this campaign has no player-data setup to add a view to");
      if (problems.length || !s) return { problems };
      const known = new Set([...Object.keys(s.disclosure.fields), ...Object.keys(s.disclosure.tags)]);
      for (const ov of v.overrides ?? []) if (!known.has(ov?.target?.id)) problems.push(`override for ${String(ov?.target?.id)}: there is no such field or tag in the disclosure map`);
      const bad = checkSetup(s);
      if (bad) problems.push(bad);
      return problems.length ? { problems } : { problems: [], value: { view: v, setup: s } };
    },
    summary: ({ view: v }) => `${v.label}: shows ${v.show_tiers.join(", ") || "no tiers"}, tags ${v.tags.mode}${v.tags.list?.length ? ` (${v.tags.list.join(", ")})` : ""}, unknowns ${v.unknown_style === "omit" ? "left out" : "shown as placeholders"}. Accept applies it to the campaign's player data.`,
    accept: (x) => o.apply(x.setup),
  };
}

/** The campaign's disclosure map and data views as catalogue entries. */
export function dataViewEntries(setup: ObserveSetup | null): Entry[] {
  if (!setup) return [];
  const ladder = setup.disclosure.ladder?.rungs?.map((r) => r[0]) ?? ["obvious", "scanned", "surveyed", "secret"];
  return [
    { key: "disclosure", line: `fields and tags, each at a tier (${ladder.join(", ")}; the last is GM only)`, full: `tiers in order: ${ladder.join(", ")} (never show the last to players)\nfields: ${Object.entries(setup.disclosure.fields).map(([k, t]) => `${k}=${t}`).join(", ")}\ntags: ${Object.entries(setup.disclosure.tags).map(([k, t]) => `${k}=${t}`).join(", ")}` },
    ...setup.views.map((v) => ({ key: `data_view:${v.id}`, line: `${v.label}${v.description ? `: ${v.description}` : ""}`, full: JSON.stringify(v) })),
  ];
}
