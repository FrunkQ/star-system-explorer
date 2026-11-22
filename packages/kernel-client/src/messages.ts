import type { JobSpec } from "../../contracts/src/index.ts";

/** Main thread to worker. */
export type ToWorker =
  | { kind: "init"; jsUrl: string; wasmBytes?: Uint8Array }
  | { kind: "call"; id: number; request: string }
  | { kind: "job"; id: number; job: JobSpec; budget: number }
  | { kind: "cancel"; id: number };

/**
 * Worker to main thread. Responses carry the kernel's JSON text unchanged,
 * plus two measurements read outside the kernel (PF1): `ms`, the time the
 * kernel itself spent on it (a job: all its steps), and `mem`, the size of the
 * kernel's WebAssembly memory afterwards in bytes. Neither enters a hash.
 */
export type FromWorker =
  | { kind: "ready" }
  | { kind: "init_failed"; message: string }
  | { kind: "response"; id: number; response: string; ms?: number; mem?: number }
  | { kind: "progress"; id: number; done: number; total: number };

/** Anything that can carry messages both ways (Web Worker, Node worker thread). */
export interface Port {
  postMessage(m: unknown): void;
  onMessage(cb: (m: unknown) => void): void;
}
