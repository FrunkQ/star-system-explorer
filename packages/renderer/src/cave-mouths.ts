// Lava tube openings on the ground (third audit, TRUE6): the kernel's cave survey at a landing site (sse-surface
// caves.rs, op `survey_caves`) cut into the ground the walker stands on. Nothing is invented: each stretch of tube
// whose roof has fallen (a skylight, a pit chain or an open rille) is a trench of the kernel's width, its floor at
// the kernel's passage floor; where a standing roof meets a fallen one the trench ends in the cave's mouth. Going
// inside is parked (interior sites, about 4.2); this draws the holes and says where they are.

/** One stretch of fallen roof, in the site's frame (x east, y north of the site, metres). */
export interface CaveOpening {
  passage: string;
  /** "skylight", "pit chain" or "rille": what the kernel calls a run of fallen roof. */
  kind: string;
  ax: number;
  ay: number;
  bx: number;
  by: number;
  width_m: number;
  /** How far below the ground the passage floor lies at each end, m (the kernel's ground less its floor). */
  depth_a_m: number;
  depth_b_m: number;
  speculative: boolean;
}

type Node = { x_m?: number; y_m?: number; ground_m?: number; floor_m?: number };
type Segment = { from?: number; to?: number; width_m?: number; roofed?: boolean; open?: string | null };
type PassageRec = { id?: string; nodes?: Node[]; segments?: Segment[]; speculative?: boolean };

/** The openings in a cave survey (the `caves` of a `caves_surveyed` reply), or none when it holds none. */
export function caveOpenings(survey: unknown): CaveOpening[] {
  const ps = (survey as { passages?: PassageRec[] } | null)?.passages;
  if (!Array.isArray(ps)) return [];
  const out: CaveOpening[] = [];
  for (const p of ps) {
    const nodes = p.nodes ?? [];
    for (const s of p.segments ?? []) {
      if (s.roofed !== false) continue;
      const a = nodes[s.from ?? -1];
      const b = nodes[s.to ?? -1];
      const w = Number(s.width_m);
      if (!a || !b || !(w > 0)) continue;
      const depth = (n: Node) => Math.max(0, Number(n.ground_m ?? 0) - Number(n.floor_m ?? 0));
      out.push({
        passage: String(p.id ?? ""),
        kind: s.open ?? "skylight",
        ax: Number(a.x_m ?? 0),
        ay: Number(a.y_m ?? 0),
        bx: Number(b.x_m ?? 0),
        by: Number(b.y_m ?? 0),
        width_m: w,
        depth_a_m: depth(a),
        depth_b_m: depth(b),
        speculative: !!p.speculative,
      });
    }
  }
  return out;
}

/** Where a point lies against an opening: its distance off the opening's line, m, and how far along it, 0 to 1. */
function against(o: CaveOpening, x: number, y: number): { off: number; u: number } {
  const dx = o.bx - o.ax;
  const dy = o.by - o.ay;
  const l2 = dx * dx + dy * dy;
  const u = l2 > 0 ? Math.max(0, Math.min(1, ((x - o.ax) * dx + (y - o.ay) * dy) / l2)) : 0;
  return { off: Math.hypot(x - (o.ax + u * dx), y - (o.ay + u * dy)), u };
}

/**
 * How far the ground at a point is cut down by the openings, m (0 outside them). Inside an opening the ground
 * falls to the passage floor; a fallen roof leaves its broken edge as a near-vertical wall, drawn as a lip one
 * twentieth of the width wide so a mesh can show it (fudge register).
 */
export function caveCut(openings: readonly CaveOpening[], x: number, y: number): number {
  let cut = 0;
  for (const o of openings) {
    const half = o.width_m / 2;
    if (x < Math.min(o.ax, o.bx) - half || x > Math.max(o.ax, o.bx) + half || y < Math.min(o.ay, o.by) - half || y > Math.max(o.ay, o.by) + half) continue;
    const { off, u } = against(o, x, y);
    if (off >= half) continue;
    const lip = Math.max(0.5, o.width_m / 20);
    const edge = Math.min(1, (half - off) / lip);
    cut = Math.max(cut, edge * (o.depth_a_m + u * (o.depth_b_m - o.depth_a_m)));
  }
  return cut;
}

/** The opening nearest a point, with its distance to the rim (0 inside it) and its compass bearing, degrees. */
export function nearestOpening(openings: readonly CaveOpening[], x: number, y: number): { o: CaveOpening; d_m: number; bearing_deg: number; depth_m: number } | null {
  let best: { o: CaveOpening; d_m: number; bearing_deg: number; depth_m: number } | null = null;
  for (const o of openings) {
    const { off, u } = against(o, x, y);
    const d = Math.max(0, off - o.width_m / 2);
    if (best && d >= best.d_m) continue;
    const cx = o.ax + u * (o.bx - o.ax);
    const cy = o.ay + u * (o.by - o.ay);
    const brg = ((Math.atan2(cx - x, cy - y) * 180) / Math.PI + 360) % 360;
    best = { o, d_m: d, bearing_deg: brg, depth_m: o.depth_a_m + u * (o.depth_b_m - o.depth_a_m) };
  }
  return best;
}

/** One line for the ground's HUD: the nearest opening, what it is, how wide and deep, and where. */
export function openingLine(n: ReturnType<typeof nearestOpening>): string {
  if (!n) return "";
  const m = (v: number) => (v >= 1000 ? `${(v / 1000).toFixed(1)} km` : `${Math.round(v)} m`);
  const what = `${n.o.kind} ${m(n.o.width_m)} across, ${m(n.depth_m)} deep${n.o.speculative ? " (speculative)" : ""}`;
  return n.d_m < 1 ? `over a ${what}` : `${what}, ${m(n.d_m)} at ${Math.round(n.bearing_deg)}°`;
}
