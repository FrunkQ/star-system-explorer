// Starts a kernel in a Web Worker and opens a session for one seat. The GM
// shell and the player app both start this way. `base` is the URL of the
// folder that holds kernel/worker.js and kernel/sse_wasm.js (copied there by
// scripts/build-shell.sh).
import { KernelClient, browserPort } from "../../kernel-client/src/index.ts";
import { Session } from "./session.ts";
import type { Seat } from "./seat.ts";
import type { ViewSession } from "./session.ts";

export async function openBrowserSession(opts: { base: string; seat: Seat; universe: { id: string; seed: string }; packs?: unknown[] }): Promise<ViewSession> {
  const kernel = () => {
    const worker = new Worker(new URL("kernel/worker.js", opts.base), { type: "module" });
    return new KernelClient(browserPort(worker), { jsUrl: new URL("kernel/sse_wasm.js", opts.base).href });
  };
  const client = kernel();
  // Trials (physics and pack previews) run on a second kernel, started on first use.
  const session = await Session.open(client, opts.seat, opts.universe, opts.packs ?? [], kernel);
  await client.measure();
  return session;
}
