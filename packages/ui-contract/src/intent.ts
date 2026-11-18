// Intents (H8.1, H8.8). Every change anyone makes, the GM included, is an
// intent. An approved intent becomes exactly one authored event; the
// translation below is the only one, so a GM edit and the same intent from a
// player seat cannot produce different events.
import type { Json } from "./json.ts";

/** Seconds in a Julian year, the unit the time ruler works in. */
export const YEAR_S = 31_557_600n;

export type Intent =
  /** Set one field of one subject, as of a universe time. */
  | { kind: "set_value"; address: string; field: string; value: Json; at_yr: number; impossible?: boolean };

export type IntentKind = Intent["kind"];

export const INTENT_KINDS: readonly IntentKind[] = ["set_value"];

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
  if (i.kind !== "set_value") return `unknown intent kind ${(i as { kind: string }).kind}`;
  if (!i.address.startsWith("u:")) return "address must start with u:";
  if (!/^[a-z][a-z0-9_.]*$/.test(i.field)) return "field must be a lower-case name";
  if (!Number.isFinite(i.at_yr)) return "time must be finite";
  return null;
}

/** The kernel request an approved intent becomes. Internal to the session. */
export function intentToRequest(i: Intent) {
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
