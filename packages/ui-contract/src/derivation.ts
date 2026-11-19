// Interim scene assembly (until K9 builds scenes in the kernel): apply the
// session's authored values to a scene, then ask the kernel to derive what
// a derivation document names. Which flow serves which object, and which
// facts feed which inputs, is data; swapping the toy flow for the real
// physics pack is an edit to that document, not to code. Nothing here
// computes physics: values come from the kernel, and only unit scaling for
// display happens here.
import type { Json } from "./json.ts";
import type { SceneFact, SceneSubject, SystemScene } from "./scene.ts";
import type { InputValue, ViewSession } from "./session.ts";

export interface InputBinding {
  input: string;
  /** "fact": a fact of the subject (or of the system's star, derived ones included); "const": a fixed value; "now": the ruler's time in years, divided by `scale`. */
  from: "fact" | "const" | "now";
  scale?: number;
  subject?: "self" | "star";
  field?: string;
  value?: number;
  /** Kernel unit id the value is in (au, m_sun, m_earth, ...). */
  unit?: string;
}

export interface OutputBinding {
  field: string;
  label: string;
  /** Quantity kind the flow publishes. */
  kind: string;
  /** Shown value = SI value / scale, in `unit`. */
  scale: number;
  unit: string;
  digits: number;
  /** Not derived for subjects with any of these tags (e.g. no surface temperature for a gas giant). */
  except_tags?: string[];
}

export interface DerivationRule {
  applies_to: string;
  flow: string;
  inputs: InputBinding[];
  outputs: OutputBinding[];
}

export interface DerivationDoc {
  schema: "sse.derivations/0";
  id: string;
  note?: string;
  rules: DerivationRule[];
}

export type Applied = Record<string, { value: Json; display: string; event_id: string }>;

/** Scene with the session's authored values in place (source "authored"). */
export function applyEdits(scene: SystemScene, applied: Applied): SystemScene {
  const subjects = scene.subjects.map((s) => {
    const facts = s.facts.map((f) => {
      const a = applied[`${s.address}#${f.field}`];
      return a ? { ...f, value: a.value, display: a.display, source: "authored" as const } : f;
    });
    const name = applied[`${s.address}#name`];
    const a_au = applied[`${s.address}#a_au`];
    return {
      ...s,
      label: name ? String(name.value) : s.label,
      orbit: s.orbit && a_au && typeof a_au.value === "number" ? { ...s.orbit, a_au: a_au.value } : s.orbit,
      facts,
      stamps: Object.keys(applied).some((k) => k.startsWith(`${s.address}#`)) && !s.stamps.includes("authored") ? [...s.stamps, "authored"] : s.stamps,
    };
  });
  return { ...scene, subjects };
}

/** Significant-figure text without exponent for everyday magnitudes. */
export function sig(v: number, digits: number): string {
  if (v === 0 || !Number.isFinite(v)) return String(v);
  const mag = Math.floor(Math.log10(Math.abs(v)));
  const dp = Math.max(0, digits - 1 - mag);
  return dp > 0 ? v.toFixed(dp) : String(Math.round(v / 10 ** (mag - digits + 1)) * 10 ** (mag - digits + 1));
}

function bindInputs(rule: DerivationRule, s: SceneSubject, star: SceneSubject | undefined, now_yr: number): Record<string, InputValue> | string {
  const out: Record<string, InputValue> = {};
  for (const b of rule.inputs) {
    if (b.from === "now") {
      out[b.input] = { value: now_yr / (b.scale ?? 1), ...(b.unit ? { unit: b.unit } : {}), source: "authored" };
      continue;
    }
    if (b.from === "const") {
      out[b.input] = { value: b.value!, ...(b.unit ? { unit: b.unit } : {}), source: "authored" };
      continue;
    }
    const subj = b.subject === "star" ? star : s;
    const f = subj?.facts.find((x) => x.field === b.field);
    if (!f || typeof f.value !== "number") return `${s.label} has no number for ${b.field}`;
    out[b.input] = { value: f.value, ...(b.unit ? { unit: b.unit } : {}), source: f.source === "authored" ? "authored" : f.source === "derived" ? "derived" : "resolved" };
  }
  return out;
}

/** Ask the kernel for every derived fact the document names; returns a new scene. */
export async function deriveScene(session: Pick<ViewSession, "derive" | "stack">, scene: SystemScene, doc: DerivationDoc): Promise<SystemScene> {
  // The star is derived first so its derived facts (luminosity now) can feed its planets.
  const basePacks = new Set(session.stack().base.map((p) => p.id));
  const order = [...scene.subjects].sort((a, b) => Number(b.kind === "star" && !b.orbit) - Number(a.kind === "star" && !a.orbit));
  let star: SceneSubject | undefined;
  const done = new Map<string, SceneSubject>();
  for (const s of order) {
    let cur = s;
    for (const rule of doc.rules.filter((r) => r.applies_to === s.kind)) {
      const inputs = bindInputs(rule, cur, star, scene.now_yr);
      if (typeof inputs === "string") continue;
      const ref = { flow: rule.flow, inputs };
      const r = await session.derive(ref);
      if (!r.ok) continue;
      const facts = [...cur.facts];
      for (const o of rule.outputs) {
        const d = r.values[o.kind];
        if (!d || o.except_tags?.some((t) => cur.tags.includes(t))) continue;
        const v = d.value / o.scale;
        const text = sig(v, o.digits);
        const fact: SceneFact = {
          field: o.field,
          label: o.label,
          value: Number(text),
          unit: o.unit,
          display: o.unit ? `${text} ${o.unit}` : text,
          editable: false,
          source: "derived",
          why: { flow: rule.flow, inputs, kind: o.kind },
          validity: d.validity,
          fictional: d.fictional,
          publisher: d.publisher,
          changed: !basePacks.has(d.publisher.split("/")[0]!),
          scale: o.scale,
          digits: o.digits,
        };
        const i = facts.findIndex((f) => f.field === o.field);
        if (i >= 0) facts[i] = fact;
        else facts.push(fact);
      }
      cur = { ...cur, facts };
    }
    if (cur.facts.some((f) => f.changed) && !cur.stamps.includes("changed-physics")) cur = { ...cur, stamps: [...cur.stamps, "changed-physics"] };
    if (cur.facts.some((f) => f.fictional) && !cur.stamps.includes("fictional")) cur = { ...cur, stamps: [...cur.stamps, "fictional"] };
    done.set(cur.address, cur);
    if (cur.kind === "star" && !cur.orbit) star = cur;
  }
  return { ...scene, subjects: scene.subjects.map((s) => done.get(s.address)!) };
}
