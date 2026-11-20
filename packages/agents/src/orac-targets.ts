// Orac's targets over the chart room's own machinery (A2 part 1): physics
// change requests, lenses and atom issue drafts. Each check is the one the
// GM's own edits go through (the kernel for physics, checkLens for lenses);
// each accept is the GM's own path (the Physics screen's request queue, the
// lens list). Orac adds no validator of its own where SSE has one.
import { checkLens, LAYERS, TOOLS, previewRequest, type AtomInfo, type ChangeRequest, type Lens, type ViewSession, type Workspace } from "../../ui-contract/src/index.ts";
import { ATOM_ISSUE_FORM, atomIssueText, atomIssueUrl, checkAtomIssue, type AtomIssue, type Entry, type OracTarget } from "./orac.ts";

const num = (x: unknown): x is number => typeof x === "number" && Number.isFinite(x);
const short = (v: number) => String(Number(v.toPrecision(6)));

/**
 * Physics: a parameter change to a flow in the stack, filed as an Orac change
 * request. The check is the kernel's (previewRequest loads the candidate
 * campaign pack on the scratch kernel); a request it rejects is never filed.
 * Accepting files it on the Physics screen, where the GM sees its diff and
 * effect and adopts or declines it as any other request.
 */
export function physicsTarget(session: ViewSession, ws: Workspace): OracTarget<ChangeRequest> {
  let n = 0;
  return {
    kind: "physics",
    what: "A change to the parameters of one physics flow. It applies to every body the flow serves. Use only flows, nodes and parameters from the catalogue (need flow:<id> for its nodes and parameters).",
    form: '{"flow": "<flow id>", "edits": [{"node": "<node id>", "param": "<parameter name>", "after": <new number>}]}',
    async check(doc, made) {
      const d = doc as { flow?: unknown; edits?: unknown };
      if (typeof d.flow !== "string") return { problems: ["flow must be a flow id from the catalogue"] };
      const f = session.flowDoc(d.flow)?.doc;
      if (!f) return { problems: [`there is no flow ${d.flow} in the stack`] };
      if (!Array.isArray(d.edits) || !d.edits.length) return { problems: ["edits must list at least one change"] };
      const problems: string[] = [];
      const edits: ChangeRequest["edits"] = [];
      for (const e of d.edits as { node?: unknown; param?: unknown; after?: unknown }[]) {
        const node = f.nodes.find((x) => x.id === e?.node);
        if (!node) {
          problems.push(`flow ${f.id} has no node ${String(e?.node)}; its nodes are ${f.nodes.map((x) => x.id).join(", ")}`);
          continue;
        }
        const before = node.params?.[String(e.param)];
        if (before === undefined) {
          problems.push(`node ${node.id} has no parameter ${String(e.param)}; it has ${Object.keys(node.params ?? {}).join(", ") || "none"}`);
          continue;
        }
        if (!num(e.after)) problems.push(`${node.id}.${String(e.param)} must be a number`);
        else if (e.after === before) problems.push(`${node.id}.${String(e.param)} is already ${short(before)}`);
        else edits.push({ node: node.id, param: String(e.param), before, after: e.after });
      }
      if (problems.length) return { problems };
      const r: ChangeRequest = { id: `orac.${Date.now().toString(36)}.${++n}`, title: made.title?.trim() || `Change ${f.id}`, author: "orac", rationale: made.rationale?.trim() ?? "", flow: f.id, edits, status: "draft" };
      const p = await previewRequest(session, ws, r, null);
      return p.ok ? { problems: [], value: r } : { problems: p.problems };
    },
    summary: (r) => `${r.flow}: ${r.edits.map((e) => `${e.node}.${e.param} ${short(e.before)} → ${short(e.after)}`).join(", ")}`,
    async accept(r) {
      ws.fileRequest(r);
      return null;
    },
  };
}

/** The flows in the stack as catalogue entries: what each publishes on the list, nodes and parameters in full. */
/**
 * The atoms the kernel knows, for the catalogue: parameter ranges and units,
 * so Orac drafts inside them instead of learning them from refusals. Empty
 * when the session cannot say (an older kernel).
 */
export async function loadAtoms(session: ViewSession): Promise<readonly AtomInfo[]> {
  const r = await session.atoms?.();
  return r?.ok ? r.atoms : [];
}

function paramText(name: string, value: number, atom: AtomInfo | undefined): string {
  const p = atom?.params.find((x) => x.name === name);
  if (!p) return `${name}=${short(value)}`;
  const range = p.range ? `, allowed ${short(p.range[0])} to ${short(p.range[1])}` : "";
  const dim = p.dim && p.dim !== "1" ? ` ${p.dim}` : "";
  return `${name}=${short(value)}${dim} (${p.description}${range})`;
}

export function flowEntries(session: ViewSession, atoms: readonly AtomInfo[] = []): Entry[] {
  return session.flows().flatMap((id) => {
    const f = session.flowDoc(id);
    if (!f) return [];
    const d = f.doc;
    const full = [
      `pack ${f.pack}; inputs: ${d.inputs.map((i) => `${i.name} (${i.kind})`).join(", ") || "none"}`,
      ...d.nodes.map((n) => {
        const atom = atoms.find((a) => a.id === n.atom);
        return `node ${n.id}: atom ${n.atom}${atom?.explain.curious ? ` (${atom.explain.curious})` : ""}${n.params && Object.keys(n.params).length ? `; parameters ${Object.entries(n.params).map(([k, v]) => paramText(k, v, atom)).join(", ")}` : ""}`;
      }),
      `wires: ${d.wires.map((w) => `${w.from} -> ${w.to}`).join(", ")}`,
      `publishes: ${d.publishes.map((p) => `${p.kind} (from ${p.from})`).join(", ")}`,
    ].join("\n");
    return [{ key: `flow:${id}`, line: `publishes ${d.publishes.map((p) => p.kind).join(", ")}; ${d.nodes.length} nodes`, full }];
  });
}

/**
 * Lenses: what the GM sees for the job in hand (H8.6). The check is the lens
 * checker the Lens panel uses; ids must be new ("mine.*"), so a shipped lens
 * is never overwritten. Accepting saves it with the GM's own lenses.
 */
export function lensTarget(o: { taken: () => readonly string[]; fields: () => Record<string, string[]>; save: (l: Lens) => void }): OracTarget<Lens> {
  return {
    kind: "lens",
    what: "A lens: which layers the chart draws, which fields each card shows and how densely, and which tools are on the bar. It never changes data.",
    form: [
      '{"schema": "sse.lens/0", "id": "mine.<short-name>", "label": "<name>", "description": "<one line>", "cards": [{"subject_kind": "<kind or *>", "fields": ["<field>", ...], "density": "line" | "compact" | "full"}], "layers": [<layers>], "tools": [<tools>], "cost": "light"}',
      `Layers: ${LAYERS.map((l) => l.id).join(", ")}. Tools: ${TOOLS.map((t) => t.id).join(", ")}. Leave "fields" out for every field. Fields by kind are in the catalogue (need fields:<kind>).`,
    ].join("\n"),
    async check(doc) {
      const problems = checkLens(doc);
      const l = doc as Lens;
      if (typeof l.id === "string" && !l.id.startsWith("mine.")) problems.push('a new lens id starts with "mine."');
      if (typeof l.id === "string" && o.taken().includes(l.id)) problems.push(`there is already a lens ${l.id}`);
      const known = o.fields();
      for (const c of Array.isArray(l.cards) ? l.cards : []) {
        const allowed = c?.subject_kind === "*" ? [...new Set(Object.values(known).flat())] : known[c?.subject_kind];
        if (!allowed) continue;
        for (const f of c.fields ?? []) if (!allowed.includes(f)) problems.push(`no ${c.subject_kind} has a field ${f}`);
      }
      return problems.length ? { problems } : { problems: [], value: { ...l, origin: "mine" } };
    },
    summary: (l) => `${l.label}: ${l.layers.length} layers, ${l.cards.length} card ${l.cards.length === 1 ? "rule" : "rules"}, tools ${l.tools.join(", ") || "none"}`,
    async accept(l) {
      o.save(l);
      return null;
    },
  };
}

export function lensEntries(lenses: readonly Lens[], fields: Record<string, string[]>): Entry[] {
  return [
    ...lenses.map((l) => ({ key: `lens:${l.id}`, line: `${l.label}: ${l.description ?? ""}`.trim(), full: JSON.stringify({ ...l, origin: undefined }) })),
    ...Object.entries(fields).map(([k, f]) => ({ key: `fields:${k}`, line: `the fields a ${k} card can show`, full: f.join(", ") })),
  ];
}

/**
 * A new physics block that SSE does not have, drafted as an issue in the
 * issue-spec-test-code form. Accepting opens GitHub's new-issue page with the
 * draft filled in; nothing is sent until the GM presses Submit there.
 */
export function atomIssueTarget(open: (url: string) => void): OracTarget<AtomIssue> {
  return {
    kind: "atom_issue",
    what: "A new physics block (atom) SSE does not have yet, drafted as an issue for the GM to read and send. Every reference needs a DOI or an address.",
    form: ATOM_ISSUE_FORM,
    async check(doc) {
      const problems = checkAtomIssue(doc);
      return problems.length ? { problems } : { problems: [], value: doc as AtomIssue };
    },
    summary: (a) => `${a.title}: ${atomIssueText(a).split("\n")[1]?.slice(0, 160) ?? ""}`,
    async accept(a) {
      open(atomIssueUrl(a));
      return null;
    },
  };
}
