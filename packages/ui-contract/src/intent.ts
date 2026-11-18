// Intents (H8.1, H8.8). Every change anyone makes, the GM included, is an
// intent. An approved intent becomes exactly one authored event; the
// translation below is the only one, so a GM edit and the same intent from a
// player seat cannot produce different events.
import type { Json } from "./json.ts";

/** Seconds in a Julian year, the unit the time ruler works in. */
export const YEAR_S = 31_557_600n;

export type Intent =
  /** Set one field of one subject, as of a universe time. */
  | { kind: "set_value"; address: string; field: string; value: Json; at_yr: number; impossible?: boolean }
  /**
   * Adopt a campaign physics pack over the base stack (null: back to the base
   * physics as shipped). Carries the change requests it adopts, by id. Only a
   * seat with GM authority applies it; others send it to the GM.
   */
  | { kind: "adopt_physics"; pack: Json | null; requests: string[] }
  /**
   * Set the packs stacked above the ones the session opened with (Packs
   * screen), bottom to top. The campaign's own physics pack stays on top.
   * Only a seat with GM authority applies it.
   */
  | { kind: "stack_packs"; packs: Json[] };

export type IntentKind = Intent["kind"];

export const INTENT_KINDS: readonly IntentKind[] = ["set_value", "adopt_physics", "stack_packs"];

/** Machine-readable description of an intent a view may send (H8.1 `intents`). */
export interface IntentSchema {
  kind: IntentKind;
  meaning: string;
}

/** Whole years to kernel seconds, exactly. */
export function yearsToSeconds(yr: number): bigint {
  if (!Number.isFinite(yr)) throw new Error("time must be finite");
  return BigInt(Math.round(yr)) * YEAR_S;
}

/**
 * Kernel seconds to whole years. A number read from kernel JSON may have lost
 * its last bits above 2^53 seconds (about 285 million years); rounding to the
 * nearest year recovers it exactly, since the error is a few seconds.
 */
export function secondsToYears(s: number | bigint): number {
  return typeof s === "bigint" ? Number(s / YEAR_S) : Math.round(s / Number(YEAR_S));
}

/** Why an intent is malformed, or null when it is well formed. */
export function checkIntent(i: Intent): string | null {
  if (i.kind === "stack_packs") {
    if (!Array.isArray(i.packs)) return "packs must be a list";
    for (const p of i.packs as { id?: unknown; format?: unknown }[]) {
      if (!p || typeof p !== "object" || typeof p.id !== "string" || p.format !== "sse.pack/1") return "each pack must be an sse.pack/1 document";
    }
    return null;
  }
  if (i.kind === "adopt_physics") {
    if (i.pack === null) return null;
    const p = i.pack as { id?: unknown; format?: unknown };
    if (typeof p !== "object" || Array.isArray(p) || typeof p.id !== "string") return "physics pack must be a pack document";
    if (p.format !== "sse.pack/1") return "physics pack must be sse.pack/1";
    return null;
  }
  if (i.kind !== "set_value") return `unknown intent kind ${(i as { kind: string }).kind}`;
  if (!i.address.startsWith("u:")) return "address must start with u:";
  if (!/^[a-z][a-z0-9_.]*$/.test(i.field)) return "field must be a lower-case name";
  if (!Number.isFinite(i.at_yr)) return "time must be finite";
  return null;
}

/**
 * The kernel request an approved intent becomes. Internal to the session.
 * `stack` is the pack stack in force: the packs the session opened with,
 * the packs added above them, and the campaign's own physics pack.
 */
export function intentToRequest(i: Intent, stack: { opening: readonly unknown[]; added: readonly unknown[]; campaign: unknown } = { opening: [], added: [], campaign: null }) {
  const top = (c: unknown) => (c === null || c === undefined ? [] : [c]);
  if (i.kind === "adopt_physics") return { op: "load_stack" as const, packs: [...stack.opening, ...stack.added, ...top(i.pack)] };
  if (i.kind === "stack_packs") return { op: "load_stack" as const, packs: [...stack.opening, ...i.packs, ...top(stack.campaign)] };
  return {
    op: "set_authored" as const,
    edit: {
      address: i.address,
      field: i.field,
      value: i.value,
      occurred_at_s: yearsToSeconds(i.at_yr),
      impossible: i.impossible ?? false,
    },
  };
}
