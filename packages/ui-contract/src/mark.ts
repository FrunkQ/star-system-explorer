// Marks (H8.4A). A view asks "mark for this object" and draws what comes
// back; it never decides by itself what a planet class looks like. This is a
// minimal, table-driven evaluator of a symbology document; V12 replaces it
// with the full symbology engine (mixing, legend, theme capability rules).
import type { SceneSubject } from "./scene.ts";

export type Channel = "shape" | "fill_pattern" | "size" | "tone" | "outline";

export interface Mark {
  shape: "disc" | "ring" | "star" | "diamond" | "square";
  fill_pattern: "solid" | "hollow" | "hatch" | "stipple" | "band" | "cross";
  /** Radius in chart pixels before zoom. */
  size: number;
  tone: "ink" | "muted" | "accent";
  outline: "none" | "thin" | "double";
}

export interface Binding {
  applies_to: string;
  channel: Channel;
  /** "kind", or "tag:<prefix>" to take the value of the subject's first tag with that prefix. */
  from: string;
  table: Record<string, string | number>;
  fallback: string | number;
}

export interface Symbology {
  id: string;
  name: string;
  bindings: Binding[];
}

const BASE: Mark = { shape: "disc", fill_pattern: "solid", size: 4, tone: "ink", outline: "none" };

function source(s: SceneSubject, from: string): string | undefined {
  if (from === "kind") return s.kind;
  if (from.startsWith("tag:")) {
    const prefix = from.slice(4);
    const t = s.tags.find((x) => x.startsWith(prefix));
    return t?.slice(prefix.length);
  }
  return undefined;
}

export function markFor(s: SceneSubject, sym: Symbology): Mark {
  const m: Record<string, string | number> = { ...BASE };
  for (const b of sym.bindings) {
    if (b.applies_to !== "*" && b.applies_to !== s.kind) continue;
    const v = source(s, b.from);
    m[b.channel] = v !== undefined && v in b.table ? b.table[v]! : b.fallback;
  }
  return m as unknown as Mark;
}
