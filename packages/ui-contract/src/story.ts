// The story finder (U8): the pure parts. The kernel's `sift` runs sifters
// (declarative patterns, data from packs or the GM) over the campaign and
// returns hits with their evidence and significance; this file only words
// them and turns one into a brief for a storyteller.

export interface SifterDoc {
  id: string;
  label: string;
  applies_to?: string[];
  pattern: unknown;
  significance: unknown;
}

export interface StoryEvidence { address: string; field: string; value: unknown; event: string | null }

export interface StoryHit {
  sifter: string;
  label: string;
  subject: string;
  evidence: StoryEvidence[];
  significance: { score: number; persistence_s: number; reach: number; reversibility: number; rarity_p: number | null; components: [number, number, number, number] };
}

/** `audience`: sift only what that audience's scene shows (event patterns are skipped and named in `skipped`). */
export interface SiftAsk { scope?: string; at_s?: number; sifters?: (SifterDoc | string)[]; audience?: string }
export interface SiftResult { scope: string; sifters: string[]; hits: StoryHit[]; audience?: string; skipped?: string[] }

export const SIGNIFICANCE_PARTS = ["lasts", "reaches", "can't be undone", "rare"] as const;

/** A fact's value in words. */
export function valueWords(v: unknown): string {
  if (Array.isArray(v)) return v.map(valueWords).join(", ");
  if (v === true) return "yes";
  if (v === false) return "no";
  if (typeof v === "number") return Number.isInteger(v) ? String(v) : v.toPrecision(3);
  if (v && typeof v === "object") return JSON.stringify(v);
  return String(v);
}

/** A field name in words: "settlements.count" → "settlements count", "tag.garden" → "tagged garden". */
export function fieldWords(f: string): string {
  if (f.startsWith("tag.")) return `tagged ${f.slice(4).replace(/-/g, " ")}`;
  if (f === "gm.notes") return "GM's note";
  return f.replace(/[._]/g, " ");
}

/** A brief a storyteller (Minerva, or the GM) can start from: what, where, why it matters, the evidence. */
export function storyBrief(h: StoryHit, name: (address: string) => string): string {
  const s = h.significance;
  const lines = [
    `Story hook: ${h.label}, at ${name(h.subject)}.`,
    `Why it matters (score ${(s.score * 100).toFixed(0)} of 100): ${SIGNIFICANCE_PARTS.map((p, i) => `${p} ${Math.round(s.components[i]! * 100)}`).join(", ")}.`,
    "Evidence:",
    ...h.evidence.map((e) => `- ${name(e.address)}: ${fieldWords(e.field)} = ${valueWords(e.value)}${e.event ? ` (set by event ${e.event.slice(0, 10)})` : ""}`),
    "Write a short scene or rumour the players could meet, true to this evidence.",
  ];
  return lines.join("\n");
}
