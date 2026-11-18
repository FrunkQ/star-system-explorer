// The session: the one door from any interface to the kernel. Views and
// shells receive a ViewSession and never see the kernel client behind it.
// Only this file and host-browser.ts may import the kernel client (checked by
// tests/ui/boundary.test.ts).
import type { KernelClient } from "../../kernel-client/src/index.ts";
import { encodeRequest } from "./json.ts";
import { checkIntent, intentToRequest, secondsToYears, yearsToSeconds, type Intent } from "./intent.ts";
import { decide, type Seat } from "./seat.ts";
import { providerOf, type FlowDoc, type PackDoc } from "./physics.ts";

export type IntentResult =
  /** For adopt_physics, `event_id` is "stack:" and the new stack hash: the kernel does not yet chronicle stack changes. */
  | { status: "applied"; event_id: string; state_hash: string; affected: string[] }
  | { status: "pending"; intent: Intent }
  | { status: "refused"; reason: string }
  | { status: "error"; code: string; message: string };

export interface HistoryEvent {
  id: string;
  kind: string;
  at_yr: number;
  evidence: string;
  subject: string[];
}

/** One input to a derivation, as the kernel takes it (H5.2 quantities with units). */
export interface InputValue {
  value: number;
  unit?: string;
  source?: "catalogued" | "resolved" | "derived" | "authored" | "authored-impossible";
}

/** What to derive: a flow and its inputs. Also the address of a "why?". */
export interface DeriveRef {
  flow: string;
  inputs: Record<string, InputValue>;
}

export interface Derived {
  /** SI value. */
  value: number;
  dim: string;
  validity: string;
  fictional: boolean;
  publisher: string;
}

/** The kernel's explanation bundle (H6.5), as shown by a "why?". */
export interface Explanation {
  subject: string;
  value: { value: number; dim: number[] };
  formula_display: string;
  lenses: { curious?: string; science?: string; engine?: string };
  inputs: { port: string; quantity: string; source: string; value: { value: number; dim: number[] }; explanation_ref: string | null }[];
  params: Record<string, number>;
  references: string[];
  validity: string;
  out_of_range: string[];
  publisher: Record<string, string>;
  fictional: boolean;
}

export interface DeviceEdge {
  tier: string;
  unitsPerSecond: number;
}

/** What a view or shell may do. Nothing here can reach the kernel another way. */
export interface ViewSession {
  readonly seat: Readonly<Seat>;
  readonly universe: Readonly<{ id: string; seed: string; root: string }>;
  send(intent: Intent): Promise<IntentResult>;
  history(address: string): Promise<HistoryEvent[]>;
  stateHash(at_yr?: number): Promise<string>;
  kernelVersion(): Promise<string>;
  /** Derived values of a flow (read only; no event). */
  derive(ref: DeriveRef): Promise<{ ok: true; values: Record<string, Derived> } | { ok: false; message: string }>;
  /** Why one derived quantity has its value. */
  explain(ref: DeriveRef, kind: string): Promise<{ ok: true; explanation: Explanation } | { ok: false; message: string }>;
  /** Flows the loaded pack stack offers. */
  flows(): readonly string[];
  device(): DeviceEdge | undefined;
  /** Intents waiting for the GM (V5 carries them; here they wait locally). */
  pending(): readonly Intent[];
  /** The packs in force, bottom to top, and the hash the kernel gave the stack. */
  stack(): StackInfo;
  /** The flow document the stack uses for `flow` and the pack that provides it. */
  flowDoc(flow: string): { doc: FlowDoc; pack: string } | undefined;
  /**
   * Try a candidate campaign pack without adopting it: the kernel validates
   * the stack it would make and derives each ref under it, then the stack in
   * force is restored. Read only; nothing is chronicled.
   */
  trial(pack: PackDoc | null, refs: DeriveRef[]): Promise<TrialResult>;
}

export interface StackInfo {
  base: readonly PackDoc[];
  /** The campaign's own physics pack over the base, if one is adopted. */
  campaign: PackDoc | null;
  hash: string;
}

export type TrialResult =
  | { ok: true; stack_hash: string; results: ({ ok: true; values: Record<string, Derived> } | { ok: false; message: string })[] }
  | { ok: false; report: string; issues: string[] };

type Raw = { type: string; [k: string]: unknown };

export class Session implements ViewSession {
  readonly #client: KernelClient;
  readonly #pending: Intent[] = [];
  #flows: string[] = [];
  #base: PackDoc[] = [];
  #campaign: PackDoc | null = null;
  #stackHash = "";
  /** Requests run one after another, so a trial's temporary stack is never seen by another call. */
  #queue: Promise<unknown> = Promise.resolve();
  readonly seat: Readonly<Seat>;
  readonly universe: Readonly<{ id: string; seed: string; root: string }>;

  private constructor(client: KernelClient, seat: Seat, universe: { id: string; seed: string; root: string }) {
    this.#client = client;
    this.seat = Object.freeze({ ...seat });
    this.universe = Object.freeze(universe);
  }

  /** Open a universe on a kernel and hand back a session for one seat. */
  static async open(client: KernelClient, seat: Seat, universe: { id: string; seed: string }, packs: unknown[] = []): Promise<Session> {
    let flows: string[] = [];
    let hash = "";
    if (packs.length) {
      const st = await call(client, { op: "load_stack", packs });
      if (st.type !== "stack_loaded") throw new Error(`kernel rejected the pack stack: ${String(st.report ?? st.message ?? JSON.stringify(st))}`);
      flows = st.flows as string[];
      hash = String(st.stack_hash);
    }
    const r = await call(client, { op: "open_universe", id: universe.id, seed: universe.seed });
    if (r.type !== "universe_opened") throw new Error(`kernel would not open the universe: ${JSON.stringify(r)}`);
    const s = new Session(client, seat, { ...universe, root: String(r.root) });
    s.#flows = flows;
    s.#base = structuredClone(packs) as PackDoc[];
    s.#stackHash = hash;
    return s;
  }

  /** Run `f` after every request already queued, and before any queued after it. */
  #serial<T>(f: () => Promise<T>): Promise<T> {
    const next = this.#queue.then(f, f);
    this.#queue = next.catch(() => undefined);
    return next;
  }

  #call(request: unknown): Promise<Raw> {
    return this.#serial(() => call(this.#client, request));
  }

  flows(): readonly string[] {
    return [...this.#flows];
  }

  stack(): StackInfo {
    return { base: structuredClone(this.#base), campaign: structuredClone(this.#campaign), hash: this.#stackHash };
  }

  flowDoc(flow: string) {
    const p = providerOf(this.#campaign ? [...this.#base, this.#campaign] : this.#base, flow);
    return p ? { doc: structuredClone(p.doc), pack: p.pack.id } : undefined;
  }

  trial(pack: PackDoc | null, refs: DeriveRef[]): Promise<TrialResult> {
    const restore = this.#campaign ? [...this.#base, this.#campaign] : this.#base;
    return this.#serial(async () => {
      const st = await call(this.#client, { op: "load_stack", packs: pack ? [...this.#base, pack] : this.#base });
      if (st.type !== "stack_loaded") {
        return { ok: false as const, report: String(st.report ?? st.message ?? ""), issues: (st.issues as string[] | undefined) ?? [] };
      }
      try {
        const results = [];
        for (const ref of refs) {
          const r = await call(this.#client, { op: "derive", flow: ref.flow, inputs: ref.inputs });
          results.push(r.type === "derived" ? { ok: true as const, values: r.values as Record<string, Derived> } : { ok: false as const, message: String(r.message ?? JSON.stringify(r)) });
        }
        return { ok: true as const, stack_hash: String(st.stack_hash), results };
      } finally {
        const back = await call(this.#client, { op: "load_stack", packs: restore });
        if (back.type !== "stack_loaded") throw new Error("could not restore the pack stack after a trial");
      }
    });
  }

  async derive(ref: DeriveRef) {
    const r = await this.#call({ op: "derive", flow: ref.flow, inputs: ref.inputs });
    if (r.type === "derived") return { ok: true as const, values: r.values as Record<string, Derived> };
    return { ok: false as const, message: String(r.message ?? JSON.stringify(r)) };
  }

  async explain(ref: DeriveRef, kind: string) {
    const r = await this.#call({ op: "explain", flow: ref.flow, inputs: ref.inputs, kind });
    if (r.type === "explanation") return { ok: true as const, explanation: { ...(r.explanation as Explanation), fictional: Boolean(r.fictional) } };
    return { ok: false as const, message: String(r.message ?? JSON.stringify(r)) };
  }

  async send(intent: Intent): Promise<IntentResult> {
    const bad = checkIntent(intent);
    if (bad) return { status: "refused", reason: bad };
    const d = decide(this.seat, intent);
    if (d.action === "refuse") return { status: "refused", reason: d.reason };
    if (d.action === "ask_gm") {
      this.#pending.push(intent);
      return { status: "pending", intent };
    }
    const r = await this.#call(intentToRequest(intent, this.#base));
    if (intent.kind === "adopt_physics") {
      if (r.type !== "stack_loaded") return { status: "refused", reason: String(r.report ?? r.message ?? JSON.stringify(r)) };
      this.#campaign = intent.pack === null ? null : (structuredClone(intent.pack) as unknown as PackDoc);
      this.#flows = r.flows as string[];
      this.#stackHash = String(r.stack_hash);
      return { status: "applied", event_id: `stack:${this.#stackHash}`, state_hash: await this.stateHash(), affected: [] };
    }
    if (r.type === "authored")
      return { status: "applied", event_id: String(r.event_id), state_hash: String(r.state_hash), affected: r.affected as string[] };
    if (r.type === "error") return { status: "error", code: String(r.code), message: String(r.message) };
    return { status: "error", code: "unexpected", message: JSON.stringify(r) };
  }

  async history(address: string): Promise<HistoryEvent[]> {
    const r = await this.#call({ op: "history", address });
    if (r.type !== "history") return [];
    return (r.events as { id: string; kind: string; occurred_at_s: number; evidence: string; subject: string[] }[]).map((e) => ({
      id: e.id,
      kind: e.kind,
      at_yr: secondsToYears(e.occurred_at_s),
      evidence: e.evidence,
      subject: e.subject,
    }));
  }

  async stateHash(at_yr?: number): Promise<string> {
    const at_s = at_yr === undefined ? 9_223_372_036_854_775_807n : yearsToSeconds(at_yr);
    const r = await this.#call({ op: "state_hash", at_s });
    return r.type === "state_hash" ? String(r.hash) : "";
  }

  async kernelVersion(): Promise<string> {
    return (await this.#client.info()).kernel_version;
  }

  device(): DeviceEdge | undefined {
    const c = this.#client.capability;
    return c ? { tier: c.tier, unitsPerSecond: c.unitsPerSecond } : undefined;
  }

  pending(): readonly Intent[] {
    return [...this.#pending];
  }
}

async function call(client: KernelClient, request: unknown): Promise<Raw> {
  // Times are bigints in the protocol types and are written as exact digits.
  return JSON.parse(await client.callRaw(encodeRequest(request))) as Raw;
}
