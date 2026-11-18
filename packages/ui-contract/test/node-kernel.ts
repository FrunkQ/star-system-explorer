// A kernel in a Node worker thread, for tests.
import { readFileSync } from "node:fs";
import { Worker } from "node:worker_threads";
import { KernelClient, type Port } from "../../kernel-client/src/index.ts";

const root = new URL("../../../", import.meta.url);

export function nodeKernel(): { client: KernelClient; close: () => Promise<number> } {
  const worker = new Worker(new URL("packages/kernel-client/src/worker.ts", root));
  const port: Port = { postMessage: (m) => worker.postMessage(m), onMessage: (cb) => worker.on("message", cb) };
  const jsUrl = new URL("packages/kernel-client/wasm/sse_wasm.js", root).href;
  const wasmBytes = readFileSync(new URL("packages/kernel-client/wasm/sse_wasm_bg.wasm", root));
  return { client: new KernelClient(port, { jsUrl, wasmBytes }), close: () => worker.terminate() };
}
