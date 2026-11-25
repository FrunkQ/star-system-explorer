// Reach it (editing design section 8): change a result by moving one true input until the result lands on the figure
// asked for. The search runs on a scratch what-if branch the kernel resolves (BC1): each try is one set_value there
// and one `resolved_system {branch}`, and the branch is thrown away at the end, so nothing reaches the campaign until
// the answer is set as an ordinary change (one Undo). The world stays wholly physical and gets no mark. When no value
// in the input's range gets there, it says how close it can come (and the caller offers Pin it). The kernel's own sweep
// (K1) will replace the search with one call; the answer is the same.
import type { SystemResolved } from "./make.ts";
import type { ViewSession } from "./session.ts";
import type { Json } from "./json.ts";

export interface ReachAsk {
  system: string;
  /** The body moved, and the field of the input moved (one handle's field). */
  address: string;
  field: string;
  at_yr: number;
  /** The input's range to search, in the field's own units; `log` searches it by ratio (sizes, masses). */
  lo: number;
  hi: number;
  log?: boolean;
  /** The figure wanted, and how to read the result from a resolve (the surface's temperature, say). */
  target: number;
  read: (r: SystemResolved) => number | null;
  /** How near counts as there, as a share of the target (default a thousandth), and the most tries (default 24). */
  within?: number;
  tries?: number;
}

export type Reached =
  | { ok: true; value: number; got: number; tries: number }
  | { ok: false; why: string; closest?: { value: number; got: number }; tries: number };

/**
 * Find the input value that gives the result asked for, by bisection over the input's range (the result is taken to
 * move one way as the input does, as a surface's warmth does with its distance from the star; when the ends do not
 * straddle the target, the nearer end is how close it can come).
 */
export async function reachIt(session: ViewSession, ask: ReachAsk): Promise<Reached> {
  const opened = await session.openBranch("Reach it (scratch)");
  if (!opened.ok) return { ok: false, why: opened.message, tries: 0 };
  const branch = opened.branch;
  let tries = 0;
  const at = async (v: number): Promise<number | null> => {
    tries++;
    const r = await session.branchSend(branch, { kind: "set_value", address: ask.address, field: ask.field, value: v, at_yr: ask.at_yr });
    if (r.status !== "applied") return null;
    const res = await session.resolvedSystem(ask.system, undefined, branch);
    return res.ok ? ask.read(res.result) : null;
  };
  const mid = (a: number, b: number) => (ask.log ? Math.sqrt(a * b) : (a + b) / 2);
  const near = (got: number) => Math.abs(got - ask.target) <= (ask.within ?? 1e-3) * Math.abs(ask.target);
  try {
    let [a, b] = [ask.lo, ask.hi];
    let [fa, fb] = [await at(a), await at(b)];
    if (fa === null || fb === null) return { ok: false, why: "the kernel could not work the result out at the ends of the range", tries };
    if ((fa - ask.target) * (fb - ask.target) > 0) {
      const closest = Math.abs(fa - ask.target) < Math.abs(fb - ask.target) ? { value: a, got: fa } : { value: b, got: fb };
      return { ok: false, why: "no value in the range gets there", closest, tries };
    }
    for (let k = 0; k < (ask.tries ?? 24); k++) {
      const m = mid(a, b);
      const fm = await at(m);
      if (fm === null) return { ok: false, why: "the kernel could not work the result out on the way", tries };
      if (near(fm)) return { ok: true, value: m, got: fm, tries };
      if ((fa - ask.target) * (fm - ask.target) <= 0) [b, fb] = [m, fm];
      else [a, fa] = [m, fm];
    }
    const best = Math.abs(fa - ask.target) < Math.abs(fb - ask.target) ? { value: a, got: fa } : { value: b, got: fb };
    return { ok: true, ...best, tries };
  } finally {
    await session.discardBranch(branch);
  }
}

/**
 * The exact answer for a value not yet set (editing design section 3, "on a pause, one exact answer"): a scratch branch
 * gets `edits` in order (a what-if's own edits first, then the held value), the kernel resolves that line (BC1), and the
 * branch is thrown away. Nothing reaches the campaign.
 */
export async function resolveWith(session: ViewSession, ask: { system: string; at_yr: number; edits: { address: string; field: string; value: Json }[] }): Promise<{ ok: true; result: SystemResolved } | { ok: false; why: string }> {
  const opened = await session.openBranch("Held value (scratch)");
  if (!opened.ok) return { ok: false, why: opened.message };
  try {
    for (const e of ask.edits) {
      const r = await session.branchSend(opened.branch, { kind: "set_value", address: e.address, field: e.field, value: e.value, at_yr: ask.at_yr });
      if (r.status !== "applied") return { ok: false, why: r.status === "refused" ? r.reason : r.status === "error" ? r.message : "not applied" };
    }
    const res = await session.resolvedSystem(ask.system, undefined, opened.branch);
    return res.ok ? { ok: true, result: res.result } : { ok: false, why: "the kernel could not resolve it" };
  } finally {
    await session.discardBranch(opened.branch);
  }
}

/** One of a body's results from a resolve, by the results sheet's key ("gravity_m_s2"); null when not a number. */
export const resultFrom =
  (address: string, key: string) =>
  (r: SystemResolved): number | null => {
    const v = (r.surfaces.find((s) => s.address === address) as Record<string, unknown> | undefined)?.[key];
    return typeof v === "number" ? v : null;
  };

/** The surface's temperature now for one body, from a resolve (the kernel's `t_surface_now_k`). */
export const surfaceKelvin = (address: string) => resultFrom(address, "t_surface_now_k");
