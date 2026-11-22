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

/**
 * One kernel call as the performance readout sees it (PF1). Times are read
 * on the page and in the worker, never inside the kernel, and never enter a
 * hash. `kernel_ms` is the kernel's own time; `total_ms` adds the trip to
 * the worker and any wait behind earlier calls.
 */
export interface CallTiming {
  op: string;
  /** The request's `detail` or `level`, when it names one (Orbit, Survey, ...). */
  detail?: string;
  /** A job (start, steps and finish together) rather than one call. */
  job: boolean;
  total_ms: number;
  kernel_ms?: number;
  bytes_in: number;
  bytes_out: number;
  /** The kernel's WebAssembly memory after the call, bytes. */
  wasm_bytes?: number;
}

let observer: ((t: CallTiming) => void) | null = null;

/** Watch every kernel call from every client on this page; null stops. Off by default: unwatched calls cost nothing extra. */
export function setCallObserver(fn: ((t: CallTiming) => void) | null): void {
  observer = fn;
}

const clock = () => (typeof performance !== "undefined" ? performance.now() : 0);
// Every caller writes the top-level op first, so a match near the start is enough for a readout
// (and a pack load of several megabytes is never scanned whole).
const opOf = (req: string) => /"op"\s*:\s*"([a-z_]+)"/.exec(req.slice(0, 256))?.[1] ?? "unknown";
const detailOf = (req: string) => /"(?:detail|level)"\s*:\s*"([A-Za-z_-]+)"/.exec(req.slice(0, 512))?.[1];

type Done = { response: string; ms?: number; mem?: number };

/** Typed client for a kernel behind a worker port. */
export class KernelClient {
  private port: Port;
  private next = 1;
  private pending = new Map<number, { resolve: (d: Done) => void; progress?: (d: number, t: number) => void }>();
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
          p.resolve(m);
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
    const watch = observer;
    const t0 = watch ? clock() : 0;
    const done = await new Promise<Done>((resolve) => {
      this.pending.set(id, { resolve });
      this.send({ kind: "call", id, request });
    });
    if (watch) watch({ op: opOf(request), detail: detailOf(request), job: false, total_ms: clock() - t0, kernel_ms: done.ms, bytes_in: request.length, bytes_out: done.response.length, wasm_bytes: done.mem });
    return done.response;
  }

  async call(request: Request): Promise<Response> {
    return JSON.parse(await this.callRaw(JSON.stringify(request))) as Response;
  }

  /** Run a long job with progress in work units and cancellation. */
  async runJob(job: JobSpec, opts: JobOptions = {}): Promise<Response> {
    await this.ready;
    const id = this.next++;
    const watch = observer;
    const t0 = watch ? clock() : 0;
    const text = new Promise<Done>((resolve) => {
      this.pending.set(id, { resolve, progress: opts.onProgress });
    });
    opts.signal?.addEventListener("abort", () => this.send({ kind: "cancel", id }), { once: true });
    this.send({ kind: "job", id, job, budget: opts.budget ?? 10_000 });
    const done = await text;
    if (watch) watch({ op: `job:${job.kind}`, job: true, total_ms: clock() - t0, kernel_ms: done.ms, bytes_in: 0, bytes_out: done.response.length, wasm_bytes: done.mem });
    return JSON.parse(done.response) as Response;
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
