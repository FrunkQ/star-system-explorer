// GH9: every collision leaves its mark (galaxy histories design 9.4; Alex, 22 Nov: "each collision should leave its
// mark. High speed ones can literally punch a hole. slower ones will stay around forever orbiting"). The kernel (GH3)
// writes dated marks into a looked-at galaxy's record; this draws marks and nothing else. Each mark is a formula over
// the disc (a bend, a wave, a ring, a lopsided or two-armed brightness), so it costs a few numbers and needs no
// particles. Marks overlap and add, and each fades by its own time, so scrubbing the time ruler shows a galaxy being
// damaged and healing. The shapes are stand-ins for the disc's real response and are in the fudge register.
import { constant } from "../../ui-contract/src/index.ts";
import type { GalaxyMark } from "../../ui-contract/src/index.ts";

/** Stand-in shapes for each kind of mark (fudge register, "galaxy-marks.ts SCAR_SHAPES"). Lengths in disc scale
 * lengths unless named otherwise. */
export const SCAR_SHAPES = {
  /** The warp starts at this radius and lifts the edge by up to this share of the radius (the Milky Way's outer disc
   * rises 1 to 3 kpc by 16 kpc, about a tenth). */
  warp: { start_rd: 3.5, max_rise: 0.15 },
  /** Ripples: up to 0.3 kpc high (a few hundred parsecs), first wavelength 1.5 scale lengths (several kpc), wound
   * tighter as stars drift out of step: the wavelength halves every `wind_gyr`. */
  ripples: { max_kpc: 0.3, wavelength_rd: 1.5, wind_gyr: 0.5, max_wind: 3 },
  /** The outer disc tilts against the inner by up to this. */
  tilt: { max_deg: 10, from_rd: 3 },
  /** A collisional ring expands at about the Cartwheel's measured 50 to 90 km/s; its width is a share of its radius
   * and its middle is emptied by up to `hole` (Lynds and Toomre 1976; Appleton and Struck-Marcell 1996). */
  ring: { expand_kms: 70, width: 0.12, hole: 0.7 },
  /** One-armed (lopsided) and two-armed (driven) brightness waves, at most these contrasts. */
  lopsided: { max: 0.35 },
  driven_arms: { max: 0.6 },
  /** Thin and thick disc scale heights in kpc (the Milky Way's, Bland-Hawthorn and Gerhard 2016). */
  heights: { thin_kpc: 0.3, thick_kpc: 0.9 },
  /** Knots of new stars along the disturbed side: up to this many. */
  knots: { max: 24 },
} as const;

/** How far 1 km/s goes in 1 Gyr, in kpc (about 1.02). */
const kpcPerGyrPerKms = () => (1e3 * 1e9 * constant("yr_julian")) / (1e3 * constant("pc"));

/** A mark's strength at a time (the kernel's `Mark::strength_at`): nothing before it began, then falling by e over
 * its fading time; a mark with no fading time lasts for good. */
export function markStrengthAt(m: GalaxyMark, t_gyr: number): number {
  if (t_gyr < m.began_gyr) return 0;
  return m.fades_gyr && m.fades_gyr > 0 ? m.strength * Math.exp(-(t_gyr - m.began_gyr) / m.fades_gyr) : m.strength;
}

/** The angle a mark faces: the record keeps no direction, so it is read from the culprit's address (stand-in). */
function phaseOf(m: GalaxyMark): number {
  let h = 2166136261;
  for (const ch of `${m.caused_by}|${m.kind}|${m.began_gyr}`) h = Math.imul(h ^ ch.charCodeAt(0), 16777619) >>> 0;
  return (h / 4294967296) * 2 * Math.PI;
}

/** Many weak marks add like chances, so the whole never passes 1: 1 − Π(1 − s). */
const union = (xs: number[]) => 1 - xs.reduce((a, s) => a * (1 - Math.min(1, Math.max(0, s))), 1);

/** A disc bends or brightens one way at a time: the strongest live mark of a kind sets its direction and size
 * (older, weaker ones of the same kind have mostly been overwritten by it). */
function strongest(ms: { s: number; phase: number }[]): { amp: number; phase: number } {
  const m = ms[0];
  return m ? { amp: m.s, phase: m.phase } : { amp: 0, phase: 0 };
}

export interface Ring {
  r_kpc: number;
  width_kpc: number;
  strength: number;
  caused_by: string;
}

export interface Ripple {
  amp_kpc: number;
  wavelength_kpc: number;
  phase: number;
}

/** What the marks do to the disc at one time. */
export interface DiscScars {
  t_gyr: number;
  disc_kpc: number;
  warp: { rise: number; phase: number; start_kpc: number };
  ripples: Ripple[];
  tilt_deg: number;
  tilt_phase: number;
  /** 0 to 1: how much of a thick disc the history has made. */
  thick: number;
  rings: Ring[];
  lopsided: { amp: number; phase: number };
  arms: { amp: number; phase: number };
  knots: { r_kpc: number; phi: number; size: number }[];
  /** Seen only in other bands: a gas trail in radio, a warm side of the hot halo in X-rays. */
  gas_trail: number;
  warm_side: { amp: number; phase: number };
  /** The marks alive now, strongest first, for words and pointing at the culprit. */
  live: { mark: GalaxyMark; strength: number }[];
}

/** Gather every mark alive at `t_gyr` into the disc's shape. `floor` drops marks too faint to see. */
export function scarsAt(marks: readonly GalaxyMark[], t_gyr: number, disc_kpc: number, floor = 0.02): DiscScars {
  const rd = Math.max(disc_kpc, 1e-3);
  const live = marks.map((mark) => ({ mark, strength: markStrengthAt(mark, t_gyr) })).filter((l) => l.strength > floor);
  live.sort((a, b) => b.strength - a.strength || a.mark.began_gyr - b.mark.began_gyr);
  const of = (kind: string) => live.filter((l) => l.mark.kind === kind);
  const vec = (kind: string) => strongest(of(kind).map((l) => ({ s: l.strength, phase: phaseOf(l.mark) })));

  const w = vec("warp");
  const tilt = vec("tilt");
  const S = SCAR_SHAPES;
  // Ripples from several passes add, but their sum stays a few hundred parsecs: each takes its share of the union.
  const rip = of("ripples").slice(0, 4);
  const ripSum = rip.reduce((a, l) => a + l.strength, 0);
  const ripAll = union(rip.map((l) => l.strength));
  const ripples = rip.map((l) => {
    const age = t_gyr - l.mark.began_gyr;
    return { amp_kpc: (S.ripples.max_kpc * ripAll * l.strength) / ripSum, wavelength_kpc: (S.ripples.wavelength_rd * rd) / Math.min(S.ripples.max_wind, 1 + age / S.ripples.wind_gyr), phase: phaseOf(l.mark) };
  });
  const rings = of("ring")
    .map((l) => {
      const r = Math.max(0.2 * rd, (l.mark.shape.r_kpc ?? 0) * 0.25) + S.ring.expand_kms * kpcPerGyrPerKms() * (t_gyr - l.mark.began_gyr);
      return { r_kpc: r, width_kpc: S.ring.width * r, strength: l.strength, caused_by: l.mark.caused_by };
    })
    .filter((r) => r.r_kpc < 12 * rd);
  // Knots of new stars on the disturbed side, placed from the culprit's address so they hold still between redraws.
  const burst = of("burst_knots");
  const lop = vec("lopsided");
  const knots: DiscScars["knots"] = [];
  for (const l of burst) {
    const n = Math.round(S.knots.max * l.strength);
    let h = Math.floor(phaseOf(l.mark) * 1e6) >>> 0;
    const rnd = () => ((h = Math.imul(h ^ (h >>> 15), 2246822519) + 0x9e3779b9) >>> 0) / 4294967296;
    for (let i = 0; i < n; i++) knots.push({ r_kpc: rd * (0.8 + 3 * rnd()), phi: lop.phase + (rnd() - 0.5) * Math.PI, size: 0.4 + rnd() });
  }
  return {
    t_gyr,
    disc_kpc: rd,
    warp: { rise: S.warp.max_rise * w.amp, phase: w.phase, start_kpc: S.warp.start_rd * rd },
    ripples,
    tilt_deg: S.tilt.max_deg * tilt.amp,
    tilt_phase: tilt.phase,
    thick: of("thick_disc")[0]?.strength ?? 0,
    rings,
    lopsided: { amp: S.lopsided.max * lop.amp, phase: lop.phase },
    arms: { amp: S.driven_arms.max * vec("driven_arms").amp, phase: vec("driven_arms").phase },
    knots,
    gas_trail: union(of("gas_trail").map((l) => l.strength)),
    warm_side: vec("warm_side"),
    live,
  };
}

/** Height of the disc's mid-plane above the halo's plane at (r, φ), kpc: warp, ripples and the outer tilt. */
export function heightAt(sc: DiscScars, r: number, phi: number): number {
  let z = 0;
  if (r > sc.warp.start_kpc) {
    // An integral sign seen edge-on: up on one side, down on the other, rising as the square past its start.
    const u = (r - sc.warp.start_kpc) / Math.max(r, 1e-6);
    z += sc.warp.rise * r * u * u * Math.sin(phi - sc.warp.phase);
  }
  for (const q of sc.ripples) z += q.amp_kpc * Math.min(1, r / (4 * sc.disc_kpc)) * Math.sin((2 * Math.PI * r) / q.wavelength_kpc + q.phase) * Math.cos(phi - q.phase);
  const from = SCAR_SHAPES.tilt.from_rd * sc.disc_kpc;
  if (r > from) z += Math.tan((sc.tilt_deg * Math.PI) / 180) * (r - from) * Math.sin(phi - sc.tilt_phase);
  return z;
}

/** Brightness of the disc at (r, φ) against the undisturbed disc (1): rings and their emptied middles, a lopsided
 * side, driven arms. */
export function brightnessAt(sc: DiscScars, r: number, phi: number): number {
  let b = 1 + sc.lopsided.amp * Math.cos(phi - sc.lopsided.phase) + sc.arms.amp * Math.cos(2 * (phi - sc.arms.phase));
  for (const g of sc.rings) {
    const d = (r - g.r_kpc) / Math.max(g.width_kpc, 1e-3);
    b += 1.5 * g.strength * Math.exp(-0.5 * d * d);
    if (r < g.r_kpc - g.width_kpc) b *= 1 - SCAR_SHAPES.ring.hole * g.strength * Math.min(1, (g.r_kpc - r) / g.r_kpc);
  }
  return Math.max(0, b);
}

export interface Frame {
  cx: number;
  cy: number;
  /** Pixels per kpc. */
  k: number;
  outer_kpc: number;
}

const f1 = (v: number) => v.toFixed(1);

/** Face-on Chart marks in a disc frame: each ring with its emptied middle, the lopsided and driven brightness as a
 * contour of the disc's edge, and the knots. Paths only; the view inks them. */
export function faceOnPaths(sc: DiscScars, f: Frame, steps = 96): { rings: { d: string; hole: string; strength: number }[]; edge: string; knots: { x: number; y: number; r: number }[] } {
  const at = (r: number, phi: number) => `${f1(f.cx + r * f.k * Math.cos(phi))} ${f1(f.cy - r * f.k * Math.sin(phi))}`;
  const circle = (r: number) => `M${at(r, 0)} ` + Array.from({ length: steps }, (_, i) => `L${at(r, (2 * Math.PI * (i + 1)) / steps)}`).join(" ") + " Z";
  // A ring that has run off the disc has nothing left to light.
  const rings = sc.rings.filter((g) => g.r_kpc < f.outer_kpc).map((g) => ({ d: circle(g.r_kpc), hole: circle(Math.max(0, g.r_kpc - g.width_kpc)), strength: g.strength }));
  // The visible edge: where the disc's light falls to a set level, pushed out where the marks brighten it.
  const edgeR = (phi: number) => Math.min(f.outer_kpc, sc.disc_kpc * 4 * (1 + 0.6 * (brightnessAt({ ...sc, rings: [] }, 0, phi) - 1)));
  const edge = Array.from({ length: steps + 1 }, (_, i) => {
    const phi = (2 * Math.PI * i) / steps;
    return `${i ? "L" : "M"}${at(edgeR(phi), phi)}`;
  }).join(" ") + " Z";
  const knots = sc.knots.filter((k) => k.r_kpc < f.outer_kpc).map((k) => ({ x: f.cx + k.r_kpc * f.k * Math.cos(k.phi), y: f.cy - k.r_kpc * f.k * Math.sin(k.phi), r: 1 + 1.5 * k.size }));
  return { rings, edge, knots };
}

/** Edge-on Chart profile: looking along the disc from far off, the mid-plane line (warp, ripples, tilt), and the thin
 * and thick discs' envelopes. Heights are drawn `stretch` times larger than lengths so a 300 pc ripple shows. */
export function edgeOnPaths(sc: DiscScars, size: [number, number], outer_kpc: number, stretch = 5, steps = 160): { mid: string; thin: string; thick: string; stretch: number } {
  const [W, H] = size;
  const k = (W * 0.46) / outer_kpc;
  const X = (x: number) => W / 2 + x * k;
  const Y = (z: number) => H / 2 - z * k * stretch;
  // Seen from far along the y axis: x > 0 is φ = 0, x < 0 is φ = π. Brighter, longer disc where the marks brighten it.
  const zAt = (x: number) => heightAt(sc, Math.abs(x), x >= 0 ? 0 : Math.PI);
  const xs = Array.from({ length: steps + 1 }, (_, i) => -outer_kpc + (2 * outer_kpc * i) / steps);
  const mid = xs.map((x, i) => `${i ? "L" : "M"}${f1(X(x))} ${f1(Y(zAt(x)))}`).join(" ");
  // An envelope that thins outward (an exponential disc's light falls off), around the bent mid-plane.
  const env = (h: number) => {
    const thick = (x: number) => h * Math.max(0.15, Math.exp(-Math.abs(x) / (3 * sc.disc_kpc)));
    const top = xs.map((x, i) => `${i ? "L" : "M"}${f1(X(x))} ${f1(Y(zAt(x) + thick(x)))}`);
    const bottom = [...xs].reverse().map((x) => `L${f1(X(x))} ${f1(Y(zAt(x) - thick(x)))}`);
    return [...top, ...bottom, "Z"].join(" ");
  };
  return { mid, thin: env(SCAR_SHAPES.heights.thin_kpc), thick: env(SCAR_SHAPES.heights.thick_kpc * (0.3 + 0.7 * sc.thick)), stretch };
}

const KIND_WORDS: Record<string, string> = {
  warp: "a warped edge",
  ripples: "ripples across the disc",
  tilt: "an outer disc tilted against the inner",
  thick_disc: "a thick disc of old stars",
  driven_arms: "two strong arms driven by a passing companion",
  ring: "a ring of new stars round an emptied middle, where something punched straight through",
  burst_knots: "knots of new stars",
  lopsided: "a lopsided disc, heavier on one side",
  gas_trail: "a trail of gas behind a companion (radio)",
  warm_side: "one side of the hot halo brighter (X-rays)",
  fresh_gas: "fresh gas from a meal",
};

/** Words for the scars at this time, strongest first, a few at most. */
export function scarWords(sc: DiscScars, most = 4): string[] {
  const seen = new Map<string, { s: number; n: number; since: number }>();
  for (const l of sc.live) {
    const e = seen.get(l.mark.kind) ?? { s: 0, n: 0, since: Infinity };
    seen.set(l.mark.kind, { s: Math.max(e.s, l.strength), n: e.n + 1, since: Math.min(e.since, l.mark.began_gyr) });
  }
  return [...seen.entries()]
    .filter(([k]) => KIND_WORDS[k])
    .sort((a, b) => b[1].s - a[1].s || a[0].localeCompare(b[0]))
    .slice(0, most)
    .map(([k, e]) => `${KIND_WORDS[k]}${e.n > 1 ? ` (${e.n} encounters)` : ""}, from ${(sc.t_gyr - e.since).toFixed(1)} Gyr ago`);
}
