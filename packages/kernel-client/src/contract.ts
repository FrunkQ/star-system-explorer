// The JavaScript half of the contract test. Same expansion rules as
// crates/sse-kernel/src/contract.rs; responses must match the golden lines
// byte for byte.
import type { KernelClient } from "./client.ts";

type Json = null | boolean | number | string | Json[] | { [k: string]: Json };

function expand(v: Json, packs: Record<string, Json>, branch: string): Json {
  if (typeof v === "string") {
    if (v === "$branch") return branch;
    if (v.startsWith("$pack:")) {
      const p = packs[v.slice(6)];
      if (p === undefined) throw new Error(`no fixture pack ${v}`);
      return p;
    }
    return v;
  }
  if (Array.isArray(v)) return v.map((x) => expand(x, packs, branch));
  if (v && typeof v === "object") return Object.fromEntries(Object.entries(v).map(([k, x]) => [k, expand(x, packs, branch)]));
  return v;
}

export async function runContract(client: KernelClient, requests: Json[], packs: Record<string, Json>): Promise<string[]> {
  const out: string[] = [];
  let branch = "";
  for (const r of requests) {
    const resp = await client.callRaw(JSON.stringify(expand(r, packs, branch)));
    const parsed = JSON.parse(resp);
    if (parsed.type === "branch_opened") branch = parsed.branch;
    out.push(resp);
  }
  return out;
}
