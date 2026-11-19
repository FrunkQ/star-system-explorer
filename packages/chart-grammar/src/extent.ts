// How far a drawn mark reaches from its centre, so a view can place a
// selection ring or a label without knowing what the mark looks like.
import type { Mark } from "../../ui-contract/src/index.ts";

export function markExtent(m: Mark): number {
  if (m.shape === "star") return m.size * 3.3;
  if (m.ring) return m.size * 2;
  if (m.shape === "diamond" || m.shape === "triangle") return m.size * 1.3;
  return m.outline === "double" ? m.size + 3 : m.size;
}

/** The pattern a mark lays over its fill, if any, so a view can lay the same pattern over a shape of its own size. */
export function markPattern(m: Mark): string | null {
  return m.fill_pattern !== "solid" && m.fill_pattern !== "hollow" ? m.fill_pattern : null;
}
