/**
 * The performance readout (PF1, design/SSE4-performance-and-minimum-spec.md).
 *
 * One recorder for the page: every kernel call (through the kernel client's
 * observer), the renderer's frame work, the page's frame interval, clicks and
 * long tasks, grouped into the big chunks of perf-chunks.json. Off by
 * default; switched on from More, "Performance readout". While it is off,
 * nothing is recorded except a handful of start-up stamps.
 *
 * Times are read on the page and in the worker, never inside the kernel, and
 * never enter a hash. Nothing leaves the device: a report is made only when
 * the person copies or saves it, and it holds timings and the device's own
 * description, never a universe, a seed or a name.
 */
import { setCallObserver, type CallTiming } from "../../kernel-client/src/index.ts";
import chunksDoc from "./perf-chunks.json" with { type: "json" };

export interface PerfChunk {
  id: string;
  name: string;
  meaning: string;
  budget_ms: number;
  sources: string[];
  /** Engine scenarios the bench times for this chunk (PF2, tests/harness/bench.js). */
  bench?: string[];
}

export interface PerfChunksDoc {
  schema: "sse.perf-chunks/1";
  chunks: PerfChunk[];
}

export const PERF_CHUNKS: PerfChunksDoc = chunksDoc as PerfChunksDoc;

/** Where a source with no chunk goes; it keeps its own name inside. */
export const OTHER_CHUNK: PerfChunk = { id: "other", name: "Other", meaning: "Anything not yet given a chunk in perf-chunks.json.", budget_ms: 300, sources: [] };

export interface Stats {
  n: number;
  median: number;
  p95: number;
  max: number;
}

/** Nearest-rank percentiles over the kept samples. Empty: zeros. */
export function stats(xs: readonly number[]): Stats {
  if (!xs.length) return { n: 0, median: 0, p95: 0, max: 0 };
  const s = [...xs].sort((a, b) => a - b);
  const at = (q: number) => s[Math.min(s.length - 1, Math.max(0, Math.ceil(q * s.length) - 1))]!;
  return { n: s.length, median: at(0.5), p95: at(0.95), max: s[s.length - 1]! };
}

export type Verdict = "within" | "near" | "over";

/** Against a chunk's budget: within, near (over three quarters of it) or over. */
export function verdict(p95: number, budget: number): Verdict {
  return p95 > budget ? "over" : p95 > budget * 0.75 ? "near" : "within";
}

export interface SourceSummary {
  source: string;
  detail?: string;
  calls: number;
  last_ms: number;
  median_ms: number;
  p95_ms: number;
  max_ms: number;
  /** The kernel's own time (median), when the source is a kernel call. */
  kernel_median_ms?: number;
}

export interface ChunkSummary {
  id: string;
  name: string;
  meaning: string;
  budget_ms: number;
  verdict: Verdict;
  calls: number;
  median_ms: number;
  p95_ms: number;
  max_ms: number;
  sources: SourceSummary[];
}

interface Series {
  source: string;
  detail?: string;
  calls: number;
  last: number;
  total: number[];
  kernel: number[];
}

export interface RecordExtra {
  detail?: string;
  kernel_ms?: number;
  wasm_bytes?: number;
}

const round = (x: number) => (x >= 100 ? Math.round(x) : Math.round(x * 10) / 10);

export class PerfRecorder {
  readonly doc: PerfChunksDoc;
  /** Samples kept per source (a ring; calls counts every one). */
  readonly keep: number;
  #on = false;
  #series = new Map<string, Series>();
  #byId = new Map<string, PerfChunk>();
  #chunkOf = new Map<string, PerfChunk>();
  #stamps: Record<string, number> = {};
  #longTasks = 0;
  #wasm = 0;
  #listeners = new Set<(on: boolean) => void>();
  /** Bumped on every record, so a view can redraw only when something changed. */
  version = 0;

  constructor(doc: PerfChunksDoc = PERF_CHUNKS, keep = 240) {
    this.doc = doc;
    this.keep = keep;
    for (const c of doc.chunks) {
      this.#byId.set(c.id, c);
      for (const s of c.sources) this.#chunkOf.set(s, c);
    }
  }

  get enabled(): boolean {
    return this.#on;
  }

  setEnabled(on: boolean): void {
    if (on === this.#on) return;
    this.#on = on;
    for (const l of this.#listeners) l(on);
  }

  /** Told when the readout is switched on or off. */
  onToggle(fn: (on: boolean) => void): () => void {
    this.#listeners.add(fn);
    return () => this.#listeners.delete(fn);
  }

  chunkOf(source: string): PerfChunk {
    return this.#chunkOf.get(source) ?? OTHER_CHUNK;
  }

  /** One measurement. Does nothing while the readout is off. */
  record(source: string, ms: number, x: RecordExtra = {}): void {
    if (!this.#on || !(ms >= 0) || !Number.isFinite(ms)) return;
    const key = x.detail ? `${source}|${x.detail}` : source;
    let s = this.#series.get(key);
    if (!s) this.#series.set(key, (s = { source, detail: x.detail, calls: 0, last: 0, total: [], kernel: [] }));
    s.calls++;
    s.last = ms;
    s.total.push(ms);
    if (s.total.length > this.keep) s.total.shift();
    if (x.kernel_ms !== undefined && Number.isFinite(x.kernel_ms)) {
      s.kernel.push(x.kernel_ms);
      if (s.kernel.length > this.keep) s.kernel.shift();
    }
    if (x.wasm_bytes) this.#wasm = x.wasm_bytes;
    this.version++;
  }

  /** A kernel call, as the kernel client reports it. */
  recordCall(t: CallTiming): void {
    this.record(t.op, t.total_ms, { detail: t.detail, kernel_ms: t.kernel_ms, wasm_bytes: t.wasm_bytes });
  }

  /** A start-up stage, kept even while the readout is off (a few numbers, once per visit). */
  stamp(name: string, ms: number): void {
    if (Number.isFinite(ms)) this.#stamps[name] = ms;
  }

  stamps(): Readonly<Record<string, number>> {
    return this.#stamps;
  }

  longTask(): void {
    if (!this.#on) return;
    this.#longTasks++;
    this.version++;
  }

  get longTasks(): number {
    return this.#longTasks;
  }

  /** The kernel's WebAssembly memory after the last call, bytes (0: not seen yet). */
  get wasmBytes(): number {
    return this.#wasm;
  }

  clear(): void {
    this.#series.clear();
    this.#longTasks = 0;
    this.version++;
  }

  /** The chunks that have samples, in the document's order, each with its sources slowest first. */
  summary(): ChunkSummary[] {
    const groups = new Map<string, Series[]>();
    for (const s of this.#series.values()) {
      const c = this.chunkOf(s.source);
      groups.set(c.id, [...(groups.get(c.id) ?? []), s]);
    }
    const order = [...this.doc.chunks, OTHER_CHUNK];
    const out: ChunkSummary[] = [];
    for (const c of order) {
      const ss = groups.get(c.id);
      if (!ss) continue;
      const sources = ss
        .map((s): SourceSummary => {
          const t = stats(s.total);
          const k = s.kernel.length ? stats(s.kernel) : null;
          return { source: s.source, ...(s.detail ? { detail: s.detail } : {}), calls: s.calls, last_ms: round(s.last), median_ms: round(t.median), p95_ms: round(t.p95), max_ms: round(t.max), ...(k ? { kernel_median_ms: round(k.median) } : {}) };
        })
        .sort((a, b) => b.p95_ms - a.p95_ms || a.source.localeCompare(b.source));
      // The median pools every kept sample; the 95th percentile is the worst op's own, so one slow kind of call
      // (a single resolved_system among forty quick edits) is never averaged away. The verdict follows it.
      const all = stats(ss.flatMap((s) => s.total));
      const p95 = Math.max(...sources.map((s) => s.p95_ms));
      out.push({ id: c.id, name: c.name, meaning: c.meaning, budget_ms: c.budget_ms, verdict: verdict(p95, c.budget_ms), calls: ss.reduce((n, s) => n + s.calls, 0), median_ms: round(all.median), p95_ms: p95, max_ms: round(all.max), sources });
    }
    return out;
  }
}

/** The page's one recorder. */
export const perf = new PerfRecorder();

// The kernel client is watched only while the readout is on.
perf.onToggle((on) => setCallObserver(on ? (t) => perf.recordCall(t) : null));

export interface DeviceDescription {
  user_agent: string;
  platform?: string;
  cores?: number;
  /** navigator.deviceMemory: rounded down by the browser, at most 8 in Chromium. */
  memory_gb?: number;
  screen: string;
  pixel_ratio: number;
  /** The kernel's measured tier and speed (K6). */
  tier?: string;
  units_per_second?: number;
  gpu: { backend: "webgpu" | "webgl2" | "none"; adapter?: string };
  reduced_motion: boolean;
}

/** What this device says about itself. Browser only; asks the GPU once. */
export async function describeDevice(edge?: { tier: string; unitsPerSecond: number }): Promise<DeviceDescription> {
  const nav = (typeof navigator !== "undefined" ? navigator : {}) as Navigator & { deviceMemory?: number; gpu?: { requestAdapter(): Promise<{ info?: { vendor?: string; architecture?: string; device?: string; description?: string } } | null> }; userAgentData?: { platform?: string } };
  let gpu: DeviceDescription["gpu"] = { backend: "none" };
  try {
    const a = await nav.gpu?.requestAdapter();
    if (a) {
      const i = a.info ?? {};
      gpu = { backend: "webgpu", adapter: [i.vendor, i.architecture, i.device, i.description].filter(Boolean).join(" ") || undefined };
    }
  } catch {
    /* no WebGPU here */
  }
  if (gpu.backend === "none" && typeof document !== "undefined") {
    try {
      const gl = document.createElement("canvas").getContext("webgl2");
      if (gl) {
        const ext = gl.getExtension("WEBGL_debug_renderer_info");
        gpu = { backend: "webgl2", adapter: ext ? String(gl.getParameter(ext.UNMASKED_RENDERER_WEBGL)) : undefined };
        gl.getExtension("WEBGL_lose_context")?.loseContext();
      }
    } catch {
      /* no WebGL2 either */
    }
  }
  const scr = typeof screen !== "undefined" ? `${screen.width}×${screen.height}` : "unknown";
  return {
    user_agent: nav.userAgent ?? "unknown",
    ...(nav.userAgentData?.platform ? { platform: nav.userAgentData.platform } : {}),
    ...(nav.hardwareConcurrency ? { cores: nav.hardwareConcurrency } : {}),
    ...(nav.deviceMemory ? { memory_gb: nav.deviceMemory } : {}),
    screen: scr,
    pixel_ratio: typeof devicePixelRatio !== "undefined" ? devicePixelRatio : 1,
    ...(edge ? { tier: edge.tier, units_per_second: Math.round(edge.unitsPerSecond) } : {}),
    gpu,
    reduced_motion: typeof matchMedia !== "undefined" && matchMedia("(prefers-reduced-motion: reduce)").matches,
  };
}

/** JavaScript heap in use, MB (Chromium only; elsewhere undefined). */
export function jsHeapMb(): number | undefined {
  const m = (typeof performance !== "undefined" ? (performance as Performance & { memory?: { usedJSHeapSize: number } }).memory : undefined)?.usedJSHeapSize;
  return m ? Math.round(m / 1e5) / 10 : undefined;
}

/**
 * Watch the page while the readout is on: the frame interval (one
 * requestAnimationFrame callback a frame, which does no drawing of its own),
 * clicks and keys to the next paint, and main-thread tasks over 50 ms.
 * Returns the stop.
 */
export function watchPage(r: PerfRecorder = perf): () => void {
  if (typeof window === "undefined") return () => {};
  let raf = 0;
  let last = -1;
  const tick = (t: number) => {
    // A gap over a second is a hidden tab or a pause, not a frame.
    if (last >= 0 && t - last < 1000) r.record("page.frame", t - last);
    last = t;
    raf = requestAnimationFrame(tick);
  };
  raf = requestAnimationFrame(tick);
  const observers: PerformanceObserver[] = [];
  const observe = (type: string, fn: (e: PerformanceEntry) => void, extra: Record<string, unknown> = {}) => {
    try {
      if (!PerformanceObserver.supportedEntryTypes?.includes(type)) return;
      const o = new PerformanceObserver((list) => list.getEntries().forEach(fn));
      o.observe({ type, buffered: false, ...extra } as PerformanceObserverInit);
      observers.push(o);
    } catch {
      /* this browser does not offer it */
    }
  };
  observe("longtask", () => r.longTask());
  // Event timing: one entry per interaction (a click is pointerdown, pointerup and click; keep the first of each).
  observe(
    "event",
    (e) => {
      const ev = e as PerformanceEntry & { interactionId?: number };
      if (ev.interactionId && (e.name === "pointerdown" || e.name === "keydown" || e.name === "click")) r.record("page.interaction", e.duration, { detail: e.name === "keydown" ? "key" : "click" });
    },
    { durationThreshold: 16 },
  );
  return () => {
    cancelAnimationFrame(raf);
    observers.forEach((o) => o.disconnect());
  };
}

export interface PerfReport {
  schema: "sse.perf-report/1";
  /** What the person called this device ("my phone"), if anything. */
  label: string;
  /** "app": the readout in the chart room; "bench": the fixed workloads on /harness/. */
  source: "app" | "bench";
  /** The day it was made (no time, no place). */
  made_on: string;
  app: { kernel?: string; site?: string };
  device: DeviceDescription;
  startup_ms: Record<string, number>;
  chunks: ChunkSummary[];
  memory: { wasm_mb?: number; js_heap_mb?: number };
  long_tasks: number;
}

/** A report of what this page has measured, for the person to copy or save. */
export function perfReport(o: { recorder?: PerfRecorder; device: DeviceDescription; label?: string; kernel?: string; site?: string; today?: string }): PerfReport {
  const r = o.recorder ?? perf;
  const stamps = Object.fromEntries(Object.entries(r.stamps()).map(([k, v]) => [k, round(v)]));
  return {
    schema: "sse.perf-report/1",
    label: o.label ?? "",
    source: "app",
    made_on: o.today ?? new Date().toISOString().slice(0, 10),
    app: { ...(o.kernel ? { kernel: o.kernel } : {}), ...(o.site ? { site: o.site } : {}) },
    device: o.device,
    startup_ms: stamps,
    chunks: r.summary(),
    memory: { ...(r.wasmBytes ? { wasm_mb: Math.round(r.wasmBytes / 1e5) / 10 } : {}), ...(jsHeapMb() !== undefined ? { js_heap_mb: jsHeapMb() } : {}) },
    long_tasks: r.longTasks,
  };
}
