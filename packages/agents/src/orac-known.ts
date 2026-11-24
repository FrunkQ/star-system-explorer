// Orac's bulk reveals (SV7, secrets and visibility design 11): "tell the crew
// everything they would have seen coming in to Tarn" becomes one change
// request the GM accepts, never a reveal by itself. The draft names an
// audience, a landmark (GM only, Heard of, Known), the things, and optionally
// one fact. The check reads the kernel's own `known_to` for every thing at the
// campaign's now: an audience the kernel does not list, a thing Orac was not
// given, or a change that would change nothing comes back in words. Accept
// sends `set_known` (the same intent the "Known to" eye sends), as one event
// with one undo for a list, or one per thing when what is inside each goes too.
import { LANDMARKS, LANDMARK_WORDS, fieldLandmark, yearsToSeconds, type Intent, type KnownAudience, type KnownLandmark as Landmark, type ViewSession, type Workspace } from "../../ui-contract/src/index.ts";
import type { Entry, OracTarget } from "./orac.ts";

export interface RevealDraft {
  audience: string;
  landmark: Landmark;
  /** Addresses, from `things`. */
  things: string[];
  /** One fact only (an exception), instead of the whole thing. */
  field?: string;
  /** Each thing and everything inside it (moons, bases, ships at it). */
  with_inside?: boolean;
  /** Hold the news back until this campaign year. */
  from_yr?: number;
  explain?: string;
}
export interface RevealPlan {
  doc: RevealDraft;
  label: string;
  /** Each thing with where it stands now for that audience. */
  before: { address: string; name: string; from: Landmark }[];
}
export interface Place {
  address: string;
  name: string;
}

const obj = (x: unknown): x is Record<string, unknown> => !!x && typeof x === "object" && !Array.isArray(x);
const sentence = (t: string) => {
  const x = t.trim();
  return `${x[0]!.toUpperCase()}${x.slice(1)}${/[.!?]$/.test(x) ? "" : "."}`;
};
const at = (ws: Workspace) => {
  const at_yr = ws.campaign_now_yr ?? ws.now_yr;
  return { at_yr, at_s: String(yearsToSeconds(at_yr)) };
};

/** The intents accept sends: one set_known over a list, or one per thing with what is inside it. */
export function revealIntents(d: RevealDraft, when: { at_yr: number; at_s: string }): Intent[] {
  const base = { kind: "set_known" as const, audience: d.audience, landmark: d.landmark, ...(d.field ? { field: d.field } : {}), ...(d.from_yr !== undefined ? { from_yr: d.from_yr } : {}), ...when };
  if (d.with_inside) return d.things.map((address) => ({ ...base, address, scope: "children" as const }));
  return [{ ...base, address: d.things[0]!, scope: d.things.length === 1 ? ("this" as const) : { list: d.things } }];
}

export function revealTarget(session: ViewSession, ws: Workspace, places: () => readonly Place[]): OracTarget<RevealPlan> {
  return {
    kind: "reveal",
    what: "A reveal or a hide: who knows what, changed for many things at once (\"tell the crew about every world they passed\", \"the Vesh have heard of the base\"). It moves things for one audience to a landmark: gm_only (they know nothing of it), heard_of (they know it is there) or known (all their view shows). It can be one fact only (field) instead of the whole thing, and can include what is inside each thing. It never changes a fact, only who knows it; nothing is revealed until the GM accepts. Read the things first (need things).",
    form: [
      '{"audience": "everyone" | "<audience id>", "landmark": "gm_only" | "heard_of" | "known", "things": ["<address>"], "field": "<one fact, optional>", "with_inside": true | false, "from_yr": <campaign year the news arrives, optional>, "explain": "<one plain sentence: why they learn it>"}',
      '"everyone" is the whole table. A cover story (telling them something untrue) is not a reveal: say so instead.',
    ].join("\n"),
    async check(doc) {
      if (!obj(doc)) return { problems: ["a reveal is one JSON object"] };
      const d = doc as unknown as RevealDraft;
      const problems: string[] = [];
      if (typeof d.audience !== "string" || !d.audience) problems.push("audience names who learns it (\"everyone\" for the table)");
      else if (d.audience === "gm") problems.push("the GM knows everything already: name a player audience");
      if (!LANDMARKS.includes(d.landmark)) problems.push(`landmark is one of ${LANDMARKS.join(", ")}`);
      if (!Array.isArray(d.things) || !d.things.length || d.things.some((t) => typeof t !== "string")) problems.push("things lists the addresses (need things)");
      if (d.field !== undefined && (typeof d.field !== "string" || !d.field)) problems.push("field names one fact, or leave it out");
      if (d.from_yr !== undefined && !Number.isFinite(d.from_yr)) problems.push("from_yr is a campaign year");
      if (problems.length) return { problems };
      const given = new Map(places().map((p) => [p.address, p.name]));
      const unknown = d.things.filter((t) => !given.has(t));
      if (unknown.length) return { problems: [`${unknown.join(", ")}: not among the things (need things); use their addresses exactly`] };
      const when = at(ws);
      const before: RevealPlan["before"] = [];
      let label = d.audience;
      for (const address of [...new Set(d.things)]) {
        const r = await session.knownTo(address, d.field, BigInt(when.at_s));
        if (!r.ok) return { problems: [`${given.get(address)}: ${r.message}`] };
        const a: KnownAudience | undefined = r.value.audiences.find((x) => x.audience === d.audience);
        if (!a) return { problems: [`"${d.audience}" is not an audience here: use one of ${r.value.audiences.map((x) => `${x.audience} (${x.label})`).join(", ")}`] };
        label = a.label;
        before.push({ address, name: given.get(address)!, from: d.field ? fieldLandmark(a, d.field) : a.landmark });
      }
      if (!d.with_inside && before.every((b) => b.from === d.landmark)) return { problems: [`nothing would change: ${label} already has every one of them at ${LANDMARK_WORDS[d.landmark]}`] };
      return { problems: [], value: { doc: { ...d, things: before.map((b) => b.address) }, label, before } };
    },
    summary: ({ doc: d, label, before }) => {
      const list = before.map((b) => (b.from === d.landmark ? `${b.name} (already ${LANDMARK_WORDS[b.from]})` : `${b.name} (${LANDMARK_WORDS[b.from]} to ${LANDMARK_WORDS[d.landmark]})`)).join(", ");
      const what = d.field ? `${d.field} of ` : "";
      const inside = d.with_inside ? ", with what is inside each" : "";
      const from = d.from_yr !== undefined ? ` The news arrives in year ${d.from_yr}.` : "";
      const undo = d.with_inside && before.length > 1 ? `${before.length} changes, each undone on its own` : "one change with one undo";
      return `${label}: ${what}${list}${inside}.${from}${d.explain ? ` ${sentence(d.explain)}` : ""} Accept sends it as ${undo}; no fact changes, only who knows it.`;
    },
    async accept({ doc }) {
      for (const i of revealIntents(doc, at(ws))) {
        const r = await session.send(i);
        if (r.status !== "applied") return r.status === "refused" ? r.reason : r.status === "error" ? r.message : "waiting for approval";
      }
      ws.revision++;
      return null;
    },
  };
}

/** The things Orac may name in a reveal, by address, for the catalogue. */
export function placeEntries(places: readonly Place[]): Entry[] {
  return [{ key: "things", line: `the ${places.length} things on the chart, by address (for reveals)`, full: places.map((p) => `${p.address}: ${p.name}`).join("\n") || "none" }];
}
