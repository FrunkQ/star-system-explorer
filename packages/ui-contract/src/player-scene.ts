// Player scenes (H6.7, H8.1): what the kernel's scene builder (K9,
// sse-observe) hands a view after the data-view filter. The shape mirrors
// `sse_observe::Scene` exactly, so a view written against it works whether
// the scene comes from the kernel op, a subscription delta (V4) or a fixture.
// Views only turn these facts into words and marks through a vocabulary
// layer (pack data); they never compute physics.
import type { Json } from "./json.ts";

export interface PlayerFact {
  field: string;
  value?: Json;
  /** A range shown instead of a value (band precision). */
  band?: [number, number];
  /** Text shown in place of a value this audience has not been given. */
  placeholder?: string;
  /** Honesty marks (authored, impossible, fictional), present only when the data view shows them. */
  marks?: string[];
}

export interface PlayerSubject {
  address: string;
  /** The data view applied to this subject ("gm" on the GM seat). */
  view: string;
  facts: PlayerFact[];
  tags: string[];
}

export interface PlayerScene {
  audience: string;
  scope: string;
  subjects: PlayerSubject[];
}

/** How one field reads in one vocabulary layer (H8.3 vocab): label, unit, scale, digits, sentence. */
export interface FieldWords {
  label: string;
  /** SI value is divided by this before display. */
  scale?: number;
  unit?: string;
  digits?: number;
  /** Shown as a percentage of 1. */
  percent?: boolean;
  /** Sentence for prose views; `{name}` and `{text}` are filled in. */
  sentence?: string;
  /** Sentence when the value is not given; `{name}` and `{placeholder}` are filled in. */
  unknown?: string;
}

/** A vocabulary layer: words for fields, kinds and tags. Pack data (`sse.vocab/0`). */
export interface Vocab {
  schema: "sse.vocab/0";
  id: string;
  fields: Record<string, FieldWords>;
  kinds?: Record<string, string>;
  tags?: Record<string, string>;
}

export type FactText =
  | { state: "value"; label: string; text: string; number?: number }
  | { state: "band"; label: string; text: string; number: number }
  | { state: "unknown"; label: string; text: string };

/** Fields matched exactly, else by the longest `prefix.*` entry, else a label made from the name. */
export function wordsFor(vocab: Vocab, field: string): FieldWords {
  const exact = vocab.fields[field];
  if (exact) return exact;
  let best: FieldWords | undefined;
  let len = -1;
  for (const [k, w] of Object.entries(vocab.fields)) {
    if (k.endsWith(".*") && field.startsWith(k.slice(0, -1)) && k.length > len) {
      best = w;
      len = k.length;
    }
  }
  return best ?? { label: field.split(".").pop()!.replace(/_/g, " ") };
}

function sigText(v: number, digits: number): string {
  if (v === 0 || !Number.isFinite(v)) return String(v);
  return String(Number(v.toPrecision(digits)));
}

/** One fact as text in a vocabulary layer. */
export function describeFact(vocab: Vocab, f: PlayerFact): FactText {
  const w = wordsFor(vocab, f.field);
  const label = w.label;
  const num = (v: number) => {
    const x = (w.percent ? v * 100 : v) / (w.scale ?? 1);
    return { x, t: sigText(x, w.digits ?? 3) };
  };
  const unit = w.percent ? " %" : w.unit ? ` ${w.unit}` : "";
  if (f.placeholder !== undefined && f.value === undefined && !f.band) return { state: "unknown", label, text: f.placeholder };
  if (f.band) {
    const a = num(f.band[0]);
    const b = num(f.band[1]);
    return { state: "band", label, text: `${a.t} to ${b.t}${unit}`, number: (a.x + b.x) / 2 };
  }
  const v = f.value;
  if (typeof v === "number") {
    const n = num(v);
    return { state: "value", label, text: `${n.t}${unit}`, number: n.x };
  }
  if (Array.isArray(v)) return { state: "value", label, text: v.map(String).join(", ") };
  if (typeof v === "string") {
    if (f.field === "kind") return { state: "value", label, text: vocab.kinds?.[v] ?? v.replace(/-/g, " ") };
    return { state: "value", label, text: v };
  }
  return { state: "value", label, text: v === null || v === undefined ? "" : JSON.stringify(v) };
}

export function fact(s: PlayerSubject, field: string): PlayerFact | undefined {
  return s.facts.find((f) => f.field === field);
}

/** The subject's name as the audience knows it. */
export function subjectName(s: PlayerSubject): string {
  const n = fact(s, "name");
  return typeof n?.value === "string" ? n.value : "Unnamed object";
}

/** Parent address (a moon's planet), or null at the top of the scope. */
export function parentOf(address: string, scope: string): string | null {
  const i = address.lastIndexOf("/");
  const p = i > 0 ? address.slice(0, i) : null;
  return p && p !== scope && p.startsWith(scope) ? p : null;
}

/** A sentence about one fact, for prose views. */
export function sentence(vocab: Vocab, s: PlayerSubject, f: PlayerFact): string | null {
  const w = wordsFor(vocab, f.field);
  const d = describeFact(vocab, f);
  const fill = (t: string) => t.replace("{name}", subjectName(s)).replace("{text}", d.text).replace("{placeholder}", f.placeholder ?? "");
  if (d.state === "unknown") return w.unknown ? fill(w.unknown) : null;
  return w.sentence ? fill(w.sentence) : null;
}

/** Why a scene is not one a view can draw, or null. */
export function checkPlayerScene(s: unknown): string | null {
  const x = s as PlayerScene;
  if (!x || typeof x !== "object" || typeof x.audience !== "string" || typeof x.scope !== "string" || !Array.isArray(x.subjects)) return "not a scene (audience, scope, subjects)";
  for (const sub of x.subjects) {
    if (typeof sub.address !== "string" || !Array.isArray(sub.facts) || !Array.isArray(sub.tags)) return `subject ${String(sub.address)} is malformed`;
    if (!sub.address.startsWith(x.scope)) return `subject ${sub.address} is outside the scope ${x.scope}`;
  }
  return null;
}
