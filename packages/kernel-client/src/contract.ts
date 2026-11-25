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

// What the golden keeps of a response line: the hello op list and the unknown-op message are masked, so a new op
// changes no golden line (crates/sse-kernel/src/contract.rs `mask_ops`, which checks the lists against ops()).
const HELLO = '{"type":"hello"';
const OPS_FROM = '"ops":[';
const OPS_TO = '],"tiers":';
const OPS_MASK = '"ops":"every op, checked against ops()"';
const UNKNOWN_FROM = "unknown variant `nonsense`, expected one of ";
const UNKNOWN_TO = " at line ";
const UNKNOWN_MASK = "every op, checked against ops()";

// The reach report's per-block lists go to tests/golden/contract-reach.txt (checked natively), one block per line.
const REACH = '{"type":"reach"';
const REACH_FROM = '"follows":[';
const REACH_MASK = '"lists":"one line per block in tests/golden/contract-reach.txt"';

export function maskOps(line: string): string {
  if (line.startsWith(REACH)) {
    const s = line.indexOf(REACH_FROM);
    if (s >= 0 && line.endsWith("]}")) return line.slice(0, s) + REACH_MASK + "}";
  }
  if (line.startsWith(HELLO)) {
    const s = line.indexOf(OPS_FROM);
    const e = line.indexOf(OPS_TO);
    if (s >= 0 && e >= 0 && s < e) return line.slice(0, s) + OPS_MASK + line.slice(e + 1);
  }
  const u = line.indexOf(UNKNOWN_FROM);
  if (u >= 0) {
    const s = u + UNKNOWN_FROM.length;
    const e = line.indexOf(UNKNOWN_TO, s);
    if (e >= 0) return line.slice(0, s) + UNKNOWN_MASK + line.slice(e);
  }
  return line;
}

export async function runContract(client: KernelClient, requests: Json[], packs: Record<string, Json>): Promise<string[]> {
  const out: string[] = [];
  let branch = "";
  for (const r of requests) {
    const resp = await client.callRaw(JSON.stringify(expand(r, packs, branch)));
    const parsed = JSON.parse(resp);
    if (parsed.type === "branch_opened") branch = parsed.branch;
    out.push(maskOps(resp));
  }
  return out;
}
