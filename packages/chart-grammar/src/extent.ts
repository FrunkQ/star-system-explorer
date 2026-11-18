// How far a drawn mark reaches from its centre, so a view can place a
// selection ring or a label without knowing what the mark looks like.
import type { Mark } from "../../ui-contract/src/index.ts";

export function markExtent(m: Mark): number {
  if (m.shape === "star") return m.size * 3.3;
  if (m.ring) return m.size * 2;
  if (m.shape === "diamond" || m.shape === "triangle") return m.size * 1.3;
  return m.outline === "double" ? m.size + 3 : m.size;
}
