// A kernel in a Node worker thread, for tests.
import { readFileSync } from "node:fs";
import { Worker } from "node:worker_threads";
import { KernelClient, type Port } from "../../kernel-client/src/index.ts";

const root = new URL("../../../", import.meta.url);

/** A kernel, plus `scratch`, which starts a second one for trials (Session.open's scratch). */
export function nodeKernel(): { client: KernelClient; scratch: () => KernelClient; close: () => Promise<number> } {
  const workers: Worker[] = [];
  const jsUrl = new URL("packages/kernel-client/wasm/sse_wasm.js", root).href;
  const wasmBytes = readFileSync(new URL("packages/kernel-client/wasm/sse_wasm_bg.wasm", root));
  const start = () => {
    const worker = new Worker(new URL("packages/kernel-client/src/worker.ts", root));
    workers.push(worker);
    const port: Port = { postMessage: (m) => worker.postMessage(m), onMessage: (cb) => worker.on("message", cb) };
    return new KernelClient(port, { jsUrl, wasmBytes });
  };
  const client = start();
  return { client, scratch: start, close: async () => (await Promise.all(workers.map((w) => w.terminate())))[0]! };
}
