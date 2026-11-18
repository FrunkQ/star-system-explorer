import type { Cost, JobSpec, Request, Response, TierDef } from "../../contracts/src/index.ts";
import type { FromWorker, Port, ToWorker } from "./messages.ts";

export type { Port } from "./messages.ts";

export interface JobOptions {
  /** Work units per step; smaller means more responsive cancellation. */
  budget?: number;
  onProgress?: (done: number, total: number) => void;
  signal?: AbortSignal;
}

export interface Capability {
  tier: string;
  unitsPerSecond: number;
  runsUpTo: Cost;
}

export interface Edge {
  ok: boolean;
  /** Why not, and what would lift it. Shown to the user at the edge. */
  reason?: string;
}

const COST_ORDER: Cost[] = ["trivial", "light", "medium", "heavy"];

/** Typed client for a kernel behind a worker port. */
export class KernelClient {
  private port: Port;
  private next = 1;
  private pending = new Map<number, { resolve: (s: string) => void; progress?: (d: number, t: number) => void }>();
  private ready: Promise<void>;
  private hello?: Extract<Response, { type: "hello" }>;
  capability?: Capability;

  constructor(port: Port, init: { jsUrl: string; wasmBytes?: Uint8Array }) {
    this.port = port;
    this.ready = new Promise((resolve, reject) => {
      port.onMessage((raw) => {
        const m = raw as FromWorker;
        if (m.kind === "ready") return resolve();
        if (m.kind === "init_failed") return reject(new Error(m.message));
        const p = this.pending.get(m.id);
        if (!p) return;
        if (m.kind === "progress") p.progress?.(m.done, m.total);
        else {
          this.pending.delete(m.id);
          p.resolve(m.response);
        }
      });
    });
    this.send({ kind: "init", ...init });
  }

  private send(m: ToWorker) {
    this.port.postMessage(m);
  }

  /** Send a raw JSON request; resolves with the kernel's JSON text unchanged. */
  async callRaw(request: string): Promise<string> {
    await this.ready;
    const id = this.next++;
    return new Promise((resolve) => {
      this.pending.set(id, { resolve });
      this.send({ kind: "call", id, request });
    });
  }

  async call(request: Request): Promise<Response> {
    return JSON.parse(await this.callRaw(JSON.stringify(request))) as Response;
  }

  /** Run a long job with progress in work units and cancellation. */
  async runJob(job: JobSpec, opts: JobOptions = {}): Promise<Response> {
    await this.ready;
    const id = this.next++;
    const text = new Promise<string>((resolve) => {
      this.pending.set(id, { resolve, progress: opts.onProgress });
    });
    opts.signal?.addEventListener("abort", () => this.send({ kind: "cancel", id }), { once: true });
    this.send({ kind: "job", id, job, budget: opts.budget ?? 10_000 });
    return JSON.parse(await text) as Response;
  }

  /** Kernel facts: version, ops with cost classes, tiers. */
  async info() {
    if (!this.hello) {
      const r = await this.call({ op: "hello" });
      if (r.type !== "hello") throw new Error("kernel did not say hello");
      this.hello = r;
    }
    return this.hello;
  }

  /**
   * Measure this device once in deterministic work units per second. The
   * wall clock is read here, outside the kernel, and never enters a hash.
   */
  async measure(units = 20_000): Promise<Capability> {
    const info = await this.info();
    const t0 = performance.now();
    const r = await this.runJob({ kind: "benchmark", units }, { budget: units });
    const seconds = Math.max((performance.now() - t0) / 1000, 1e-6);
    if (r.type !== "job_done") throw new Error("benchmark failed");
    const ups = units / seconds;
    let best: TierDef = info.tiers[0]!;
    for (const t of info.tiers) if (ups >= t.min_units_per_second) best = t;
    this.capability = { tier: best.tier, unitsPerSecond: ups, runsUpTo: best.runs_up_to };
    return this.capability;
  }

  /** Can this device run work of a cost class locally? If not, say why. */
  can(cost: Cost): Edge {
    const c = this.capability;
    if (!c) return { ok: false, reason: "device not measured yet; call measure() first" };
    if (COST_ORDER.indexOf(cost) <= COST_ORDER.indexOf(c.runsUpTo)) return { ok: true };
    const need = this.hello?.tiers.find((t) => COST_ORDER.indexOf(t.runs_up_to) >= COST_ORDER.indexOf(cost));
    return {
      ok: false,
      reason:
        `Beyond this device's range: ${cost} work needs the ${need?.tier ?? "highest"} tier ` +
        `(${need ? Math.round(need.min_units_per_second).toLocaleString("en") : "?"} units/s); this device measured ` +
        `${Math.round(c.unitsPerSecond).toLocaleString("en")} units/s (${c.tier}). ` +
        `When connected, the GM's machine can send recorded results instead.`,
    };
  }
}

/** Port over a browser Web Worker. */
export function browserPort(w: Worker): Port {
  return { postMessage: (m) => w.postMessage(m), onMessage: (cb) => w.addEventListener("message", (e) => cb(e.data)) };
}
