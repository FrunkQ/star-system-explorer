// Marks for the GM's system scene (H8.4A). The engine is symbology.ts; this
// adapts a scene subject to what a symbology reads: its kind, its classes
// (tags "class:<C1 class id>", base first) and its other tags.
import type { SceneSubject } from "./scene.ts";
import type { MarkSubject } from "./symbology.ts";
import type { PlayerSubject } from "./player-scene.ts";

export type { Mark } from "./symbology.ts";

export function subjectOf(s: Pick<SceneSubject, "kind" | "tags">): MarkSubject {
  return { kind: s.kind, classes: s.tags.filter((t) => t.startsWith("class:")).map((t) => t.slice(6)), tags: s.tags };
}

/** A player subject as a symbology reads it: kind and classes come from facts the audience was given. */
export function playerSubjectOf(s: PlayerSubject): MarkSubject {
  const fact = (f: string) => s.facts.find((x) => x.field === f)?.value;
  const k = String(fact("kind") ?? "");
  const kind = k === "planetary-body" ? "planet" : k;
  const c = fact("class");
  const classes = Array.isArray(c) ? c.map(String) : c === undefined || c === null ? [] : [String(c)];
  const size = fact("size_class");
  return { kind, classes, tags: [...s.tags, ...(typeof size === "string" ? [`size:${size}`] : [])] };
}
