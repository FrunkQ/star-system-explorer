// Starts a kernel in a Web Worker and opens a session for one seat. The GM
// shell and the player app both start this way. `base` is the URL of the
// folder that holds kernel/worker.js and kernel/sse_wasm.js (copied there by
// scripts/build-shell.sh).
import { KernelClient, browserPort } from "../../kernel-client/src/index.ts";
import { Session } from "./session.ts";
import type { Seat } from "./seat.ts";
import { perf } from "./perf.ts";

export async function openBrowserSession(opts: { base: string; seat: Seat; universe: { id: string; seed: string }; packs?: unknown[] }): Promise<Session> {
  const kernel = () => {
    const worker = new Worker(new URL("kernel/worker.js", opts.base), { type: "module" });
    return new KernelClient(browserPort(worker), { jsUrl: new URL("kernel/sse_wasm.js", opts.base).href });
  };
  // Start-up stamps for the performance readout (PF1): ms since the page began, read outside the kernel.
  const client = kernel();
  await client.info();
  perf.stamp("startup.kernel", performance.now());
  // Trials (physics and pack previews) run on a second kernel, started on first use.
  const session = await Session.open(client, opts.seat, opts.universe, opts.packs ?? [], kernel);
  perf.stamp("startup.session", performance.now());
  await client.measure();
  perf.stamp("startup.measure", performance.now());
  return session;
}
