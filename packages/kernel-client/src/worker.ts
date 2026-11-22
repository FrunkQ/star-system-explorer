// Worker entry: owns one kernel instance. Works as a browser module worker
// and as a Node worker thread. The kernel is loaded on the first message, so
// this file has no static import of the generated wasm glue.
import type { Request, Response } from "../../contracts/src/index.ts";
import type { FromWorker, Port, ToWorker } from "./messages.ts";

interface KernelLike {
  handle(request: string): string;
}

const yieldToEvents = () => new Promise<void>((r) => setTimeout(r, 0));
// Display only (PF1): how long the kernel took, read here, never inside it.
const now = () => (typeof performance !== "undefined" ? performance.now() : 0);

async function serve(port: Port) {
  let kernel: KernelLike | undefined;
  let memory: { buffer: ArrayBuffer } | undefined;
  const mem = () => memory?.buffer.byteLength;
  const cancelled = new Set<number>();
  const send = (m: FromWorker) => port.postMessage(m);
  const call = (req: Request): Response => JSON.parse(kernel!.handle(JSON.stringify(req))) as Response;

  port.onMessage(async (raw) => {
    const m = raw as ToWorker;
    if (m.kind === "init") {
      try {
        const glue = await import(m.jsUrl);
        const exports = await glue.default(m.wasmBytes ? { module_or_path: m.wasmBytes } : undefined);
        memory = exports?.memory;
        kernel = new glue.Kernel() as KernelLike;
        send({ kind: "ready" });
      } catch (e) {
        send({ kind: "init_failed", message: String(e) });
      }
      return;
    }
    if (!kernel) return;
    if (m.kind === "cancel") {
      cancelled.add(m.id);
      return;
    }
    if (m.kind === "call") {
      const t0 = now();
      const response = kernel.handle(m.request);
      send({ kind: "response", id: m.id, response, ms: now() - t0, mem: mem() });
      return;
    }
    // A long job runs in budgeted steps; between steps the worker yields to
    // its event loop, which is when a cancel message can arrive.
    let spent = 0;
    const t0 = now();
    const started = call({ op: "start_job", job: m.job });
    spent += now() - t0;
    if (started.type !== "job_started") {
      send({ kind: "response", id: m.id, response: JSON.stringify(started) });
      return;
    }
    const job = started.job;
    for (;;) {
      if (cancelled.delete(m.id)) {
        send({ kind: "response", id: m.id, response: kernel.handle(JSON.stringify({ op: "cancel_job", job })), ms: spent, mem: mem() });
        return;
      }
      const s0 = now();
      const text = kernel.handle(JSON.stringify({ op: "step_job", job, budget: m.budget }));
      spent += now() - s0;
      const r = JSON.parse(text) as Response;
      if (r.type === "job_progress") {
        send({ kind: "progress", id: m.id, done: r.done_units, total: r.total_units });
        await yieldToEvents();
        continue;
      }
      send({ kind: "response", id: m.id, response: text, ms: spent, mem: mem() });
      return;
    }
  });
}

const isNode = typeof process !== "undefined" && !!process.versions?.node;
if (isNode) {
  const { parentPort } = await import("node:worker_threads");
  if (parentPort) {
    const p = parentPort;
    serve({ postMessage: (m) => p.postMessage(m), onMessage: (cb) => p.on("message", cb) });
  }
} else {
  const g = globalThis as unknown as { postMessage(m: unknown): void; addEventListener(t: string, cb: (e: MessageEvent) => void): void };
  serve({ postMessage: (m) => g.postMessage(m), onMessage: (cb) => g.addEventListener("message", (e) => cb(e.data)) });
}
