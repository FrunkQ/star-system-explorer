// Orac's target for triggers and rules (A2 part 6, track J): "when this
// happens, do that", such as a sensor moving a body up the knowledge ladder or
// a tag setting a ship's readiness. The check is the kernel's: the draft goes
// into the campaign's own triggers pack and the stack is tried on the scratch
// kernel, which loads triggers with `sse_trigger::Rules::from_packs` and
// refuses a bad id, a rule declared twice, a rule with no effects, a grant to
// an observer audience without pairs, an unknown field, in its own words.
// Accepting stacks that pack like any pack the GM adds, chronicled.
import { stackPacks, type PackDoc, type ViewSession, type Workspace } from "../../ui-contract/src/index.ts";
import type { Entry, OracTarget } from "./orac.ts";
import { CONDITIONS } from "./orac-rules.ts";

export const TRIGGERS_PACK = "campaign.triggers";

export interface TriggerDoc {
  id: string;
  phase?: number;
  when: Record<string, unknown>;
  scope?: Record<string, unknown>;
  effects: { effect: string; [k: string]: unknown }[];
  approval?: unknown;
  explain?: string;
}
/** A trigger draft: the rule, and any new tags it names that no pack declares yet. */
export type TriggerDraft = TriggerDoc & { tags?: { id: string; label: string; about?: string }[] };

const obj = (x: unknown): x is Record<string, unknown> => !!x && typeof x === "object" && !Array.isArray(x);
const bump = (v: string) => {
  const [a = 0, b = 0, c = 0] = v.split(".").map(Number);
  return `${a}.${b}.${c + 1}`;
};
const packsOf = (session: ViewSession): PackDoc[] => {
  const s = session.stack();
  return [...s.base, ...(s.campaign ? [s.campaign] : [])];
};
const rulesOf = (p: PackDoc): TriggerDoc[] => {
  const t = (p.provides?.other as { triggers?: { rules?: TriggerDoc[] } } | undefined)?.triggers;
  return Array.isArray(t?.rules) ? t.rules : [];
};
const whenWords = (w: Record<string, unknown>) => {
  const [k, v] = Object.entries(w)[0] ?? ["?", {}];
  const o = (v ?? {}) as Record<string, unknown>;
  if (k === "on_event") return `on event ${String(o.kind)}`;
  if (k === "on_intent") return `on intent ${String(o.kind)}`;
  if (k === "on_time") return `every ${String(o.every_s)} s`;
  if (k === "on_condition") return `when its condition holds${o.hold_s ? ` for ${String(o.hold_s)} s` : ""}`;
  return k;
};

export function triggerTarget(session: ViewSession, ws: Workspace): OracTarget<{ doc: TriggerDoc; added: PackDoc[] }> {
  return {
    kind: "trigger",
    what: "A trigger: a rule that fires when something happens (an event, an intent, a condition held for a time, a clock) and has effects (grant knowledge to an audience, set a kept value, add a note, tag or untag, send an intent, set a pin, create from a template, retire). Effects that create, retire or pin ask the GM first by default. Read a shipped one first (need trigger:<id>).",
    form: [
      '{"id": "<lower-case.id>", "when": {"on_condition": {"condition": <condition>, "hold_s": <seconds>}} | {"on_event": {"kind": "<event kind>", "tag": "<tag>"}} | {"on_intent": {"kind": "<intent kind>"}} | {"on_time": {"every_s": <seconds>, "from_s": <seconds>}}, "scope": {"tag": "<tag>"} | {"addresses": ["<address>"]} | {"under": "<address>"} | {"pairs": {"observer_tag": "<tag>"}}, "effects": [{"effect": "grant", "audience": "<audience>" | "{observer.<field>}", "ladder": ["<rung>"]} | {"effect": "set_kept", "kind": "<quantity kind>", "value": <v>, "reason": "<why>"} | {"effect": "add_note", "text": "<text>"} | {"effect": "tag", "tag": "<tag>"} | {"effect": "untag", "tag": "<tag>"} | {"effect": "send_intent", "kind": "<intent kind>", "payload": {...}}], "explain": "<one plain sentence>"}',
      CONDITIONS + ' In a pairs scope, fields may also be "pair.distance_m" and "observer.<field>".',
      'Every tag a trigger names must be declared by a pack (need tags). To name a new one, add "tags": [{"id": "<tag>", "label": "<words>", "about": "<what it means>"}] beside the rule.',
    ].join("\n"),
    async check(doc) {
      if (!obj(doc)) return { problems: ["a trigger is one JSON object"] };
      const { tags: fresh, ...rule } = doc as unknown as TriggerDraft;
      const d = rule as TriggerDoc;
      const problems: string[] = [];
      if (typeof d.id !== "string" || !d.id) problems.push("a trigger needs an id");
      if (!obj(d.when)) problems.push("when says what fires it");
      if (!Array.isArray(d.effects) || !d.effects.length) problems.push("effects must list at least one effect");
      if (typeof d.id === "string" && packsOf(session).flatMap(rulesOf).some((r) => r.id === d.id)) problems.push(`there is already a trigger ${d.id}`);
      if (fresh !== undefined && !Array.isArray(fresh)) problems.push("tags is a list of {id, label}");
      if (problems.length) return { problems };
      const st = session.stack();
      const mine = st.added.find((p) => p.id === TRIGGERS_PACK);
      const pack = { format: "sse.pack/1", id: TRIGGERS_PACK, version: bump(mine?.version ?? "0.0.0"), kind: ["game-system"], kernel_range: "^0.1", licence: "Apache-2.0", provides: { other: { triggers: { rules: [...(mine ? rulesOf(mine) : []), d] }, ...withTags(mine, fresh ?? []) } } } as unknown as PackDoc;
      const added = [...st.added.filter((p) => p.id !== TRIGGERS_PACK), pack];
      const t = await session.trial({ added }, async () => true);
      return t.ok ? { problems: [], value: { doc: d, added } } : { problems: t.issues.length ? t.issues : [t.report] };
    },
    summary: ({ doc: d }) => `${d.id}: ${whenWords(d.when)}, ${d.effects.map((e) => e.effect.replace("_", " ")).join(", ")}${d.explain ? `. ${d.explain}` : ""}. Accept adds it to the campaign's triggers.`,
    async accept({ added }) {
      const r = await stackPacks(session, ws, added);
      return r.status === "applied" ? null : r.status === "refused" ? r.reason : r.status === "error" ? r.message : "waiting for approval";
    },
  };
}

type TagRow = { id: string; label?: string; about?: string };
const tagsOf = (p: PackDoc | undefined): TagRow[] => {
  const t = (p?.provides?.other as { tags?: { tags?: TagRow[] } } | undefined)?.tags;
  return Array.isArray(t?.tags) ? t.tags : [];
};
/** The campaign triggers pack's tag declarations, with the draft's new tags added. */
function withTags(mine: PackDoc | undefined, fresh: TagRow[]) {
  const tags = [...tagsOf(mine), ...fresh];
  return tags.length ? { tags: { about: "Tags the campaign's own triggers and templates name (TG1): labels only.", groups: [], tags } } : {};
}

/** The triggers and tags in force, for the catalogue. */
export function triggerEntries(session: ViewSession): Entry[] {
  const packs = packsOf(session);
  const tags = packs.flatMap(tagsOf);
  return [
    { key: "tags", line: `the ${tags.length} tags packs declare`, full: tags.map((t) => `${t.id}: ${t.label ?? ""}${t.about ? ` (${t.about})` : ""}`).join("\n") || "none" },
    ...packs.flatMap(rulesOf).map((r) => ({ key: `trigger:${r.id}`, line: `${r.id}: ${whenWords(r.when)}${r.explain ? `, ${r.explain}` : ""}`, full: JSON.stringify(r) })),
  ];
}

export const TEMPLATES_PACK = "campaign.templates";

export interface TemplateDraft {
  id: string;
  kind: string;
  label: string;
  facts?: Record<string, unknown>;
  kept?: Record<string, unknown>;
  tags?: string[];
  programs?: string[];
  explain?: string;
  /** New tags it names that no pack declares yet. */
  declare?: TagRow[];
}

const templatesOf = (p: PackDoc | undefined): Omit<TemplateDraft, "declare">[] => {
  const t = (p?.provides?.other as { templates?: { templates?: Omit<TemplateDraft, "declare">[] } } | undefined)?.templates;
  return Array.isArray(t?.templates) ? t.templates : [];
};

/**
 * Object templates (J5, J6): what a created object starts with, its tags, and
 * the programs it may run. A program Orac drafts runs only on an object whose
 * template lists it, so this is how a new autopilot or patrol gets a body. The
 * kernel's template load refuses a bad id or kind, an id declared twice, a
 * missing label, a fact create sets itself, an undeclared tag; a kept value
 * must list `template.<id>` among its writers.
 */
export function templateTarget(session: ViewSession, ws: Workspace): OracTarget<{ doc: TemplateDraft; added: PackDoc[] }> {
  return {
    kind: "template",
    what: "An object template: a kind of thing the GM or a trigger can create (a patrol ship, a mining drone, a beacon), with the facts and kept values it starts with, its tags and the programs it may run. A program runs only on an object whose template lists it. Read the programs (need program:<id>), the templates in force (need template:<id>) and the tags (need tags) first.",
    form: [
      '{"id": "<lower-case-id>", "kind": "<one lower-case word: ship, station, drone...>", "label": "<name>", "facts": {"<fact>": <value>}, "kept": {"<kept kind>": <value>}, "tags": ["<tag>"], "programs": ["<program id>"], "explain": "<one plain sentence>", "declare": [{"id": "<new tag>", "label": "<words>"}]}',
      "Tags must be declared by a pack; put any new ones in declare. A kept value must be a kept kind whose writers include template.<id>. Leave facts out that the object's physics will give it.",
    ].join("\n"),
    async check(doc) {
      if (!obj(doc)) return { problems: ["a template is one JSON object"] };
      const { declare, ...d } = doc as unknown as TemplateDraft;
      const problems: string[] = [];
      if (typeof d.id !== "string" || !d.id) problems.push("a template needs an id");
      if (typeof d.kind !== "string" || !d.kind) problems.push("a template needs a kind");
      if (typeof d.id === "string" && packsOf(session).some((p) => templatesOf(p).some((t) => t.id === d.id))) problems.push(`there is already a template ${d.id}`);
      if (declare !== undefined && !Array.isArray(declare)) problems.push("declare is a list of {id, label}");
      if (problems.length) return { problems };
      const st = session.stack();
      const mine = st.added.find((p) => p.id === TEMPLATES_PACK);
      const pack = { format: "sse.pack/1", id: TEMPLATES_PACK, version: bump(mine?.version ?? "0.0.0"), kind: ["game-system"], kernel_range: "^0.1", licence: "Apache-2.0", provides: { other: { templates: { templates: [...templatesOf(mine), d] }, ...withTags(mine, declare ?? []) } } } as unknown as PackDoc;
      const added = [...st.added.filter((p) => p.id !== TEMPLATES_PACK), pack];
      const t = await session.trial({ added }, async () => true);
      return t.ok ? { problems: [], value: { doc: d, added } } : { problems: t.issues.length ? t.issues : [t.report] };
    },
    summary: ({ doc: d }) => `${d.label} (${d.kind})${d.tags?.length ? `, tagged ${d.tags.join(", ")}` : ""}${d.programs?.length ? `, may run ${d.programs.join(", ")}` : ""}${d.explain ? `. ${d.explain}` : ""}. Accept adds it to the campaign's templates, so objects can be made from it.`,
    async accept({ added }) {
      const r = await stackPacks(session, ws, added);
      return r.status === "applied" ? null : r.status === "refused" ? r.reason : r.status === "error" ? r.message : "waiting for approval";
    },
  };
}
