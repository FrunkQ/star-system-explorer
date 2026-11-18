// The session: the one door from any interface to the kernel. Views and
// shells receive a ViewSession and never see the kernel client behind it.
// Only this file and host-browser.ts may import the kernel client (checked by
// tests/ui/boundary.test.ts).
import type { KernelClient } from "../../kernel-client/src/index.ts";
import { encodeRequest } from "./json.ts";
import { checkIntent, intentToRequest, secondsToYears, yearsToSeconds, type Intent } from "./intent.ts";
import { decide, type Seat } from "./seat.ts";

export type IntentResult =
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
  device(): DeviceEdge | undefined;
  /** Intents waiting for the GM (V5 carries them; here they wait locally). */
  pending(): readonly Intent[];
}

type Raw = { type: string; [k: string]: unknown };

export class Session implements ViewSession {
  readonly #client: KernelClient;
  readonly #pending: Intent[] = [];
  readonly seat: Readonly<Seat>;
  readonly universe: Readonly<{ id: string; seed: string; root: string }>;

  private constructor(client: KernelClient, seat: Seat, universe: { id: string; seed: string; root: string }) {
    this.#client = client;
    this.seat = Object.freeze({ ...seat });
    this.universe = Object.freeze(universe);
  }

  /** Open a universe on a kernel and hand back a session for one seat. */
  static async open(client: KernelClient, seat: Seat, universe: { id: string; seed: string }): Promise<Session> {
    const r = await call(client, { op: "open_universe", id: universe.id, seed: universe.seed });
    if (r.type !== "universe_opened") throw new Error(`kernel would not open the universe: ${JSON.stringify(r)}`);
    return new Session(client, seat, { ...universe, root: String(r.root) });
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
    const r = await call(this.#client, intentToRequest(intent));
    if (r.type === "authored")
      return { status: "applied", event_id: String(r.event_id), state_hash: String(r.state_hash), affected: r.affected as string[] };
    if (r.type === "error") return { status: "error", code: String(r.code), message: String(r.message) };
    return { status: "error", code: "unexpected", message: JSON.stringify(r) };
  }

  async history(address: string): Promise<HistoryEvent[]> {
    const r = await call(this.#client, { op: "history", address });
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
    const r = await call(this.#client, { op: "state_hash", at_s });
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
