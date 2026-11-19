// The draft, check and fix loop, in the shape of Ovid Next's builder
// (ovid: browser/src/agent/builder.ts::buildFlow, "Naso's loop"): one fixed
// system turn (so a provider can cache it), one request turn, then rounds.
// Each round the model either asks to read catalogue entries (a lookup, which
// does not use up a try, at most three a run) or answers with one JSON object.
// The answer's candidates are checked by the host's own validators, every
// problem at once, and sent back; the model fixes and answers again, up to
// `tries` fixes. The loop never applies anything: it hands back the
// candidates that passed and, apart, the ones that did not, so a candidate
// with a problem can never reach an accept button.
//
// Nothing here knows about physics or screens. Orac (orac.ts) supplies the
// targets; the loop only talks, reads JSON and counts.
import type { ChatMessage } from "./agents.ts";
import { readJson, readLookup } from "./lifted.ts";

/** One call to the model. The adapter (agents.ts chat) estimates, cleans keys and enforces the day's cap. */
export type Ask = (turns: readonly ChatMessage[]) => Promise<{ ok: true; text: string; usd?: number; tokens_in: number; tokens_out: number } | { ok: false; message: string }>;

/** One thing the model made: a kind the host knows and its document. */
export interface Made {
  kind: string;
  doc: unknown;
  title?: string;
  rationale?: string;
}

/** A made thing after checking. `problems` empty means it passed every check the host has. */
export interface Checked<T = unknown> {
  made: Made;
  problems: string[];
  /** What the host turned it into when it passed (a change request, a lens...). */
  value?: T;
}

export interface Missing {
  does: string;
  closest?: string;
  why_not?: string;
}

export interface Round {
  round: number;
  lookup?: { need: string[]; find: string | null };
  problems: string[];
  note: string;
}

export interface LoopResult<T = unknown> {
  ok: boolean;
  note: string;
  /** Passed every check: these, and only these, may be offered for acceptance. */
  passed: Checked<T>[];
  /** Still failing after the last fix: shown with their problems, never with an accept button. */
  failed: Checked<T>[];
  missing: Missing[];
  rounds: Round[];
  calls: number;
  /** US dollars, when every call reported or had a listed price; null once one did not (never shown as zero). */
  usd: number | null;
  tokens_in: number;
  tokens_out: number;
  message: string;
}

export interface LoopOptions<T> {
  system: string;
  request: string;
  ask: Ask;
  /** Check one made thing: every problem at once, in words the model and the GM can act on. */
  check: (m: Made) => Promise<{ problems: string[]; value?: T }>;
  /** Answer a lookup: the full entries asked for by key, or the nearest entries to some plain words. */
  lookup?: (need: readonly string[], find: string | null) => string;
  /** Fixes after the first answer (Ovid's default is three; at most six). */
  tries?: number;
  /** A limit for this run in US dollars: no call is started that the spend so far plus the last call's cost would pass. */
  cap?: number;
  onRound?: (r: Round) => void;
}

const LOOKUPS = 3;

export function readAnswer(text: string): { note: string; make: Made[]; missing: Missing[] } | null {
  const v = readJson(text);
  if (!v || !Array.isArray(v.make)) return null;
  const make: Made[] = v.make
    .filter((m): m is Record<string, unknown> => !!m && typeof m === "object")
    .map((m) => ({ kind: String(m.kind ?? ""), doc: m.doc, ...(typeof m.title === "string" ? { title: m.title } : {}), ...(typeof m.rationale === "string" ? { rationale: m.rationale } : {}) }));
  const missing: Missing[] = Array.isArray(v.missing)
    ? v.missing.filter((m): m is Record<string, unknown> => !!m && typeof m === "object" && typeof m.does === "string").map((m) => ({ does: String(m.does), ...(typeof m.closest === "string" ? { closest: m.closest } : {}), ...(typeof m.why_not === "string" ? { why_not: m.why_not } : {}) }))
    : [];
  return { note: typeof v.note === "string" ? v.note : "", make, missing };
}

export async function draftLoop<T>(o: LoopOptions<T>): Promise<LoopResult<T>> {
  const tries = Math.max(0, Math.min(6, o.tries ?? 3));
  const turns: ChatMessage[] = [
    { role: "system", content: o.system },
    { role: "user", content: o.request },
  ];
  const rounds: Round[] = [];
  let calls = 0;
  let usd: number | null = 0;
  let tokens_in = 0;
  let tokens_out = 0;
  let lastCost = 0;
  let lookups = 0;
  let note = "";
  let passed: Checked<T>[] = [];
  let failed: Checked<T>[] = [];
  let missing: Missing[] = [];
  let message = "";
  let capped = false;
  for (let round = 1; round <= tries + 1; round++) {
    if (o.cap !== undefined && calls > 0 && usd !== null && usd + lastCost > o.cap) {
      capped = true;
      break;
    }
    const r = await o.ask(turns);
    if (!r.ok) {
      message = r.message;
      break;
    }
    calls++;
    tokens_in += r.tokens_in;
    tokens_out += r.tokens_out;
    lastCost = r.usd ?? lastCost;
    usd = usd === null || r.usd === undefined ? null : usd + r.usd;
    turns.push({ role: "assistant", content: r.text });

    const look = readLookup(r.text);
    if (look && lookups < LOOKUPS && o.lookup) {
      lookups++;
      round--;
      const rr: Round = { round: round + 1, lookup: look, problems: [], note: "" };
      rounds.push(rr);
      o.onRound?.(rr);
      turns.push({ role: "user", content: `${o.lookup(look.need, look.find)}\nNow answer with the note and what you make.` });
      continue;
    }
    const a = readAnswer(r.text);
    let problems: string[];
    if (!a) problems = ['Your answer was not one JSON object with "note" and "make".'];
    else {
      note = a.note;
      missing = a.missing;
      const checked: Checked<T>[] = [];
      for (const m of a.make) {
        const c = await o.check(m);
        checked.push({ made: m, problems: c.problems, ...(c.value !== undefined ? { value: c.value } : {}) });
      }
      passed = checked.filter((c) => !c.problems.length);
      failed = checked.filter((c) => c.problems.length);
      problems = failed.flatMap((c, i) => c.problems.map((p) => `${c.made.kind} ${c.made.title ? `"${c.made.title}"` : `#${i + 1}`}: ${p}`));
      if (!a.make.length && !a.missing.length) problems = ['Your answer made nothing. Make at least one thing, or say what is missing under "missing".'];
    }
    const rr: Round = { round, problems, note: a?.note ?? "" };
    rounds.push(rr);
    o.onRound?.(rr);
    if (!problems.length) break;
    if (round <= tries) turns.push({ role: "user", content: `The check found these problems (every one at once):\n${problems.map((p) => `- ${p}`).join("\n")}\nSend the whole corrected answer again, in the same form.` });
  }
  const fixes = rounds.filter((r) => !r.lookup).length;
  const ok = !message && !capped && failed.length === 0 && (passed.length > 0 || missing.length > 0);
  if (!message)
    message = ok
      ? `Drafted in ${fixes} ${fixes === 1 ? "try" : "tries"}; every draft passed its checks.`
      : capped
        ? `Stopped before the next try: it would pass this run's limit of $${o.cap}.`
        : failed.length
          ? `After ${fixes} tries ${failed.length === 1 ? "one draft still has" : `${failed.length} drafts still have`} problems. ${failed.length === 1 ? "It is" : "They are"} shown with the problems, and cannot be accepted.`
          : "No usable draft came back.";
  return { ok, note, passed, failed, missing, rounds, calls, usd, tokens_in, tokens_out, message };
}
