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
  | { kind: "stack_packs"; packs: Json[] }
  /**
   * The GM's Scanned button: give `audience` the next data view up for one
   * subject, or take it back on a second tap (K9 scan_toggle, chronicled as
   * an observe.grant). Only a seat with GM authority applies it.
   */
  | { kind: "scan_toggle"; address: string; audience: string; at_yr: number }
  /**
   * Add a system to the universe (V13, kernel `adopt`): an imported V3 save or one grown by formation. GM only. It is
   * an intent so that it travels to players' screens like every other change (a replica that never saw it had no copy
   * of the system: the landing run-through, 24 Nov).
   */
  | { kind: "adopt"; source: { import_v3: string } | { formation: Json }; occurred_at_s?: number }
  /**
   * A write to the record that is not a field: a flight log entry, a kept value (fuel, readiness), a construct made
   * from a template or taken out of play, a person or cargo added, a note. An intent so it
   * travels to players' screens like every other change: written straight to the kernel, the players' copy fell out of
   * step with the GM's at the next change and stopped following (party run-through, 24 Nov). GM only.
   */
  | { kind: "record"; op: RecordOp; ask: Json }
  /**
   * Grant an audience a data view of one subject and everything under it
   * (K9 grant, chronicled as observe.grant: "Vesh learned Tarn"). `view`
   * null takes the grant back, so the subject falls back to its parent, its
   * tags and the defaults. `default` sets the audience's starting view
   * instead (address: the universe root). Only the GM applies it.
   */
  | { kind: "grant"; address: string; audience: string; view: string | null; default?: boolean; at_yr: number }
  /**
   * Set many fields of many subjects as one authored event (kernel
   * set_authored_batch): a paste places its bodies this way, so one Undo
   * takes the whole paste back. `evidence` "catalogued" (with a `source`)
   * marks values read from a real catalogue. Impossible and possible values
   * go in separate intents. Only the GM applies it.
   */
  | { kind: "set_values"; edits: { address: string; field: string; value: Json; impossible?: boolean }[]; at_yr: number; evidence?: "authored" | "catalogued"; source?: string }
  /** Take a subject and everything under it out of the universe from `at_yr` on (Cut, or Undo of a paste). Its facts are kept. */
  | { kind: "withdraw"; address: string; at_yr: number }
  /** Bring a withdrawn subject back from `at_yr` on, with every fact it had (Redo of a paste, Undo of a cut). */
  | { kind: "restore"; address: string; at_yr: number }
  /**
   * Type a name for one layer of a thing (NM2 rename): a typed name always wins and never follows its parent. `text`
   * null takes the typed name away, so the drawn one shows again. Only the GM applies it.
   */
  | { kind: "rename"; address: string; layer: string; text: string | null; at_yr: number }
  /**
   * Choose the scheme that names one layer here and everything beneath (NM2 set_naming; `of_kind` for one kind of
   * thing only). `scheme` null takes the setting away, so the one above applies again. Names not yet kept change at
   * once; `rename_fixed` draws kept ones afresh too, dated `at_yr`. Only the GM applies it.
   */
  | { kind: "set_naming"; address: string; layer: string; scheme: string | null; of_kind?: string; rename_fixed?: boolean; at_yr: number }
  /**
   * Who knows a thing (SV1 set_known, secrets and visibility design 5): put it, or one fact on it (`field`), at a
   * landmark for one audience, over a scope (this, with what is inside it, or a list), as dated observe.grant events
   * that undo as one. A null landmark on a field drops its exception, so the fact follows its thing again. `from_yr`
   * holds the news back until that campaign time. Only the GM applies it.
   */
  /** Put back exactly what one set_known changed (SV1 undo_known), by the id its reply gave, as one dated event.
   * `at_s` (whole seconds, as text), when given, is the exact time and wins over `at_yr`. */
  | { kind: "undo_known"; undo: string; at_yr: number; at_s?: string }
  | { kind: "set_known"; address: string; audience: string; landmark: import("./known.ts").Landmark | null; field?: string; scope: import("./known.ts").KnownScope; from_yr?: number; at_yr: number; at_s?: string };

export type IntentKind = Intent["kind"];
/** The record writes the `record` intent carries (each a kernel op). */
export const RECORD_OPS = ["log_flight", "set_kept", "create", "retire", "add_actor", "add_note"] as const;
export type RecordOp = (typeof RECORD_OPS)[number];

export const INTENT_KINDS: readonly IntentKind[] = ["set_value", "adopt_physics", "stack_packs", "scan_toggle", "grant", "set_values", "withdraw", "restore", "rename", "set_naming", "set_known", "undo_known", "adopt", "record"];

/** Machine-readable description of an intent a view may send (H8.1 `intents`). */
export interface IntentSchema {
  kind: IntentKind;
  meaning: string;
}

/**
 * Years to kernel seconds: whole years exactly, the fraction to the nearest
 * second (U11: edits and the campaign clock can fall on a day or an hour).
 */
export function yearsToSeconds(yr: number): bigint {
  if (!Number.isFinite(yr)) throw new Error("time must be finite");
  const whole = Math.floor(yr);
  return BigInt(whole) * YEAR_S + BigInt(Math.round((yr - whole) * Number(YEAR_S)));
}

const YEAR_MIN = Number(YEAR_S) / 60;

/**
 * Kernel seconds to years. A number read from kernel JSON may have lost its
 * last bits above 2^53 seconds (about 285 million years); rounding to the
 * nearest minute recovers whole years exactly (the error is a few seconds)
 * and keeps days and hours.
 */
export function secondsToYears(s: number | bigint): number {
  if (typeof s === "bigint") {
    const whole = s / YEAR_S;
    return Number(whole) + Number(s - whole * YEAR_S) / Number(YEAR_S);
  }
  const minutes = Math.round(s / 60);
  const years = Math.floor(minutes / YEAR_MIN);
  return years + (minutes - years * YEAR_MIN) / YEAR_MIN;
}

/** Why an intent is malformed, or null when it is well formed. */
export function checkIntent(i: Intent): string | null {
  if (i.kind === "record") {
    if (!(RECORD_OPS as readonly string[]).includes(i.op)) return `record writes are ${RECORD_OPS.join(" or ")}`;
    if (!i.ask || typeof i.ask !== "object" || Array.isArray(i.ask) || "op" in (i.ask as object)) return "a record write's ask is an object without its own op";
    return null;
  }
  if (i.kind === "adopt") {
    const src = i.source as Record<string, unknown> | null;
    if (!src || typeof src !== "object") return "adopt needs a source";
    const keys = Object.keys(src);
    if (keys.length !== 1 || !(typeof src.import_v3 === "string" || (keys[0] === "formation" && src.formation && typeof src.formation === "object"))) return "adopt's source is one import_v3 save or one formation request";
    if (i.occurred_at_s !== undefined && !Number.isFinite(i.occurred_at_s)) return "time must be finite";
    return null;
  }
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
  if (i.kind === "scan_toggle") {
    if (!i.address.startsWith("u:")) return "address must start with u:";
    if (!/^[a-z0-9][a-z0-9._-]*$/.test(i.audience)) return "audience must be a lower-case name";
    if (!Number.isFinite(i.at_yr)) return "time must be finite";
    return null;
  }
  if (i.kind === "grant") {
    if (!i.address.startsWith("u:")) return "address must start with u:";
    if (!/^[a-z0-9][a-z0-9._-]*$/.test(i.audience)) return "audience must be a lower-case name";
    if (i.view !== null && (typeof i.view !== "string" || !i.view)) return "view must be a data view id, or null to take the grant back";
    if (i.default && i.view === null) return "a starting view cannot be taken back, only replaced";
    if (!Number.isFinite(i.at_yr)) return "time must be finite";
    return null;
  }
  if (i.kind === "set_values") {
    if (!Array.isArray(i.edits) || i.edits.length === 0) return "a batch needs at least one edit";
    for (const e of i.edits) {
      if (typeof e.address !== "string" || !e.address.startsWith("u:")) return "address must start with u:";
      if (typeof e.field !== "string" || !/^[a-z][a-z0-9_.-]*$/.test(e.field)) return "field must be a lower-case name";
    }
    const imp = i.edits.filter((e) => e.impossible).length;
    if (imp && imp !== i.edits.length) return "impossible and possible values go in separate batches";
    if (i.evidence !== undefined && i.evidence !== "authored" && i.evidence !== "catalogued") return "evidence must be authored or catalogued";
    if (i.evidence === "catalogued" && (imp || !i.source?.trim())) return "catalogued values need a source and cannot be impossible";
    if (!Number.isFinite(i.at_yr)) return "time must be finite";
    return null;
  }
  if (i.kind === "withdraw" || i.kind === "restore") {
    if (!i.address.startsWith("u:")) return "address must start with u:";
    if (!Number.isFinite(i.at_yr)) return "time must be finite";
    return null;
  }
  if (i.kind === "rename" || i.kind === "set_naming") {
    if (!i.address.startsWith("u:")) return "address must start with u:";
    if (!/^[a-z0-9_]+$/.test(i.layer)) return "a layer is lower-case letters, digits and _";
    if (i.kind === "rename" && i.text !== null && (typeof i.text !== "string" || !i.text.trim())) return "a name needs some text, or null to take it away";
    if (i.kind === "set_naming" && i.scheme !== null && (typeof i.scheme !== "string" || !i.scheme)) return "a scheme id, or null to take the setting away";
    if (!Number.isFinite(i.at_yr)) return "time must be finite";
    return null;
  }
  if ((i.kind === "undo_known" || i.kind === "set_known") && i.at_s !== undefined && !/^-?\d{1,19}$/.test(i.at_s)) return "at_s is whole seconds, written out";
  if (i.kind === "undo_known") return typeof i.undo === "string" && i.undo ? (Number.isFinite(i.at_yr) ? null : "time must be finite") : "undo needs the id a set_known gave";
  if (i.kind === "set_known") {
    if (!i.address.startsWith("u:")) return "address must start with u:";
    if (!/^[a-z0-9][a-z0-9._-]*$/.test(i.audience)) return "audience must be a lower-case name";
    if (i.landmark !== null && !["gm_only", "heard_of", "known"].includes(i.landmark)) return "landmark is gm_only, heard_of or known";
    if (i.landmark === null && !i.field) return "only a fact's exception can be dropped; a thing always has a landmark";
    if (i.field !== undefined && !/^[a-z][a-z0-9_.-]*$/.test(i.field)) return "field must be a lower-case name";
    if (typeof i.scope === "object" && (!Array.isArray(i.scope.list) || !i.scope.list.length || i.scope.list.some((a) => !a.startsWith("u:")))) return "a list scope names addresses";
    if (typeof i.scope === "string" && i.scope !== "this" && i.scope !== "children") return "scope is this, children or a list";
    if (!Number.isFinite(i.at_yr) || (i.from_yr !== undefined && !Number.isFinite(i.from_yr))) return "time must be finite";
    return null;
  }
  if (i.kind !== "set_value") return `unknown intent kind ${(i as { kind: string }).kind}`;
  if (!i.address.startsWith("u:")) return "address must start with u:";
  if (!/^[a-z][a-z0-9_.-]*$/.test(i.field)) return "field must be a lower-case name";
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
  if (i.kind === "scan_toggle") return { op: "scan_toggle" as const, address: i.address, audience: i.audience, at_s: yearsToSeconds(i.at_yr) };
  if (i.kind === "grant") return { op: "grant" as const, address: i.address, audience: i.audience, ...(i.view !== null ? { view: i.view } : {}), ...(i.default ? { default: true } : {}), at_s: yearsToSeconds(i.at_yr) };
  if (i.kind === "set_values")
    return {
      op: "set_authored_batch" as const,
      edits: i.edits.map((e) => ({ address: e.address, field: e.field, value: e.value, ...(e.impossible ? { impossible: true } : {}) })),
      occurred_at_s: yearsToSeconds(i.at_yr),
      ...(i.evidence ? { evidence: i.evidence } : {}),
      ...(i.source ? { source: i.source } : {}),
    };
  if (i.kind === "withdraw" || i.kind === "restore") return { op: i.kind, address: i.address, at_s: yearsToSeconds(i.at_yr) };
  if (i.kind === "rename") return { op: "rename" as const, address: i.address, layer: i.layer, ...(i.text !== null ? { text: i.text.trim() } : {}), at_s: yearsToSeconds(i.at_yr) };
  if (i.kind === "set_naming")
    return { op: "set_naming" as const, address: i.address, layer: i.layer, ...(i.scheme !== null ? { scheme: i.scheme } : {}), ...(i.of_kind ? { kind: i.of_kind } : {}), ...(i.rename_fixed ? { rename_fixed: true } : {}), at_s: yearsToSeconds(i.at_yr) };
  if (i.kind === "record") return { op: i.op, ...(i.ask as Record<string, Json>) };
  if (i.kind === "adopt") return { op: "adopt" as const, ...(i.source as Record<string, Json>), ...(i.occurred_at_s !== undefined ? { occurred_at_s: i.occurred_at_s } : {}) };
  if (i.kind === "undo_known") return { op: "undo_known" as const, undo: i.undo, at_s: i.at_s !== undefined ? BigInt(i.at_s) : yearsToSeconds(i.at_yr) };
  if (i.kind === "set_known")
    return {
      op: "set_known" as const,
      address: i.address,
      audience: i.audience,
      landmark: i.landmark,
      ...(i.field ? { field: i.field } : {}),
      scope: i.scope,
      ...(i.from_yr !== undefined ? { from_s: yearsToSeconds(i.from_yr) } : {}),
      // Exact seconds when given (the campaign's now): a year in floating point is only good to about half a minute.
      at_s: i.at_s !== undefined ? BigInt(i.at_s) : yearsToSeconds(i.at_yr),
    };
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
