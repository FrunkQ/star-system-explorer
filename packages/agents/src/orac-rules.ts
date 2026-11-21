// Orac's targets for the game layer (A2 part 3, track J): programs, the
// state machines that drive a program seat (J6). The check is the kernel's:
// the candidate goes into the campaign's own programs pack and the whole
// stack is tried on the scratch kernel (`trial`), which refuses an unknown
// start or state, a transition to nowhere, `{pick}` without a pick, an
// unknown param, an intent no pack declares, and the rest of J6's load
// refusals, in its own words. Accepting stacks that pack like any pack the
// GM adds, chronicled as a stack change. Nothing runs until the GM attaches
// the program to an object whose template lists it.
import { stackPacks, type PackDoc, type ViewSession, type Workspace } from "../../ui-contract/src/index.ts";
import type { Entry, OracTarget } from "./orac.ts";

/** The pack that carries the campaign's own programs. */
export const PROGRAMS_PACK = "campaign.programs";

export interface ProgramDoc {
  id: string;
  label?: string;
  params?: Record<string, unknown>;
  start: string;
  states: { id: string; pick?: unknown; act?: { kind: string; payload?: unknown }; transitions?: unknown[] }[];
  budget?: { max_steps: number };
  explain?: string;
}

type Other = { programs?: { programs?: ProgramDoc[]; limit?: number }; intents?: { kinds?: { kind: string; meaning?: string; fields?: { name: string; type: string }[] }[] }; templates?: unknown; record?: unknown };
const other = (p: PackDoc) => (p.provides?.other ?? {}) as Other;
const obj = (x: unknown): x is Record<string, unknown> => !!x && typeof x === "object" && !Array.isArray(x);

/** Every pack in force, bottom to top, the campaign's own last. */
function packsOf(session: ViewSession): PackDoc[] {
  const s = session.stack();
  return [...s.base, ...(s.campaign ? [s.campaign] : [])];
}

export function programsPack(programs: readonly ProgramDoc[], version: string): PackDoc {
  return { format: "sse.pack/1", id: PROGRAMS_PACK, version, kind: ["game-system"], kernel_range: "^0.1", licence: "Apache-2.0", provides: { other: { programs: { programs: structuredClone(programs) } } } } as unknown as PackDoc;
}

const bump = (v: string) => {
  const [a = 0, b = 0, c = 0] = v.split(".").map(Number);
  return `${a}.${b}.${c + 1}`;
};

/** The candidate stack with `doc` added to the campaign's programs pack. */
function withProgram(session: ViewSession, doc: ProgramDoc): { added: PackDoc[] } {
  const st = session.stack();
  const mine = st.added.find((p) => p.id === PROGRAMS_PACK);
  const list = [...(other(mine ?? ({} as PackDoc)).programs?.programs ?? []), doc];
  return { added: [...st.added.filter((p) => p.id !== PROGRAMS_PACK), programsPack(list, bump(mine?.version ?? "0.0.0"))] };
}

export const CONDITIONS =
  'Conditions (K8): {"if": "all"|"any", "of": [...]}, {"if": "not", "p": {...}}, {"if": "has", "field": f}, {"if": "range", "field": f, "min": n, "max": n}, {"if": "is", "field": f, "value": v}, {"if": "compare", "left": f, "op": "lt"|"le"|"gt"|"ge", "right": f, "factor": n}, {"if": "tag", "tag": t}. In a program, a field may also be "param.<name>" and, in transitions, "pick.found".';

export function programTarget(session: ViewSession, ws: Workspace): OracTarget<{ doc: ProgramDoc; added: PackDoc[] }> {
  return {
    kind: "program",
    what: "A program: a small state machine that drives a program seat (an autopilot, a patrol, a mining drone). Each state may pick a subject among what the seat has sensed and send one intent; transitions move it on. It acts only on what its seat's audience has been shown, and only through intents a pack declares (need intents). It runs once the GM attaches it to an object whose template lists it.",
    form: [
      '{"id": "<lower-case-id>", "label": "<name>", "params": {"<name>": <value>}, "start": "<state id>", "states": [{"id": "<state>", "pick": {"where": <condition>, "by": "<number field>", "order": "max"|"min"}, "act": {"kind": "<intent kind>", "payload": {"<field>": "{self}" | "{pick}" | "{param.<name>}" | <value>}}, "transitions": [{"when": <condition>, "to": "<state>"}]}], "budget": {"max_steps": <n>}, "explain": "<one plain sentence>"}',
      CONDITIONS,
    ].join("\n"),
    async check(doc) {
      if (!obj(doc)) return { problems: ["a program is one JSON object"] };
      const d = doc as unknown as ProgramDoc;
      const problems: string[] = [];
      if (typeof d.id !== "string" || !d.id) problems.push("a program needs an id");
      if (typeof d.start !== "string") problems.push("start must name the first state");
      if (!Array.isArray(d.states) || !d.states.length) problems.push("states must list at least one state");
      const taken = packsOf(session).flatMap((p) => other(p).programs?.programs ?? []).map((p) => p.id);
      if (typeof d.id === "string" && taken.includes(d.id)) problems.push(`there is already a program ${d.id}`);
      if (problems.length) return { problems };
      const cand = withProgram(session, d);
      const t = await session.trial(cand, async () => true);
      return t.ok ? { problems: [], value: { doc: d, added: cand.added } } : { problems: t.issues.length ? t.issues : [t.report] };
    },
    summary: ({ doc: d }) => `${d.label ?? d.id}: ${d.states.map((s) => `${s.id}${s.act ? ` (${s.act.kind})` : ""}`).join(" → ")}, starting at ${d.start}${d.explain ? `. ${d.explain}` : ""}. Accept adds it to the campaign's programs; attach it to an object to run it.`,
    async accept({ added }) {
      const r = await stackPacks(session, ws, added);
      return r.status === "applied" ? null : r.status === "refused" ? r.reason : r.status === "error" ? r.message : "waiting for approval";
    },
  };
}

/** The intents, programs and templates in force, for the catalogue. */
export function ruleEntries(session: ViewSession): Entry[] {
  const packs = packsOf(session);
  const kinds = packs.flatMap((p) => other(p).intents?.kinds ?? []);
  const programs = packs.flatMap((p) => other(p).programs?.programs ?? []);
  const templates = packs.flatMap((p) => {
    const t = other(p).templates as { templates?: { id: string; label?: string }[] } | undefined;
    return t?.templates ?? [];
  });
  return [
    { key: "intents", line: kinds.length ? `the intents packs declare: ${kinds.map((k) => k.kind).join(", ")}` : "no pack declares intents yet, so a program has nothing to send", full: kinds.map((k) => `${k.kind}: ${k.meaning ?? ""}; fields ${(k.fields ?? []).map((f) => `${f.name} (${f.type})`).join(", ") || "none"}`).join("\n") || "none" },
    ...programs.map((p) => ({ key: `program:${p.id}`, line: `${p.label ?? p.id}${p.explain ? `: ${p.explain}` : ""}`, full: JSON.stringify(p) })),
    ...templates.map((t) => ({ key: `template:${t.id}`, line: `object template ${t.label ?? t.id}`, full: JSON.stringify(t) })),
  ];
}
