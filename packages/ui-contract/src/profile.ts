// The Profile (RB3, ring and belt editing design sections 4.2 and 6): an amount along a line, the editing grammar's
// fifteenth part. A ring or belt seen edge to edge: distance from the parent across, how much is there upward, as
// zones (spans) that always sum to the whole. It is the Bar with a second direction: a join drags sideways to move an
// edge between two zones, a zone drags up or down to make it denser or thinner, and the rest give way in proportion,
// so shaping never changes how much there is (design question 2; only the "How much" handle changes the total).
//
// It reads the kernel's `ring_structure_is` answer in the shape agreed with "Give planetary rings real structure"
// (design section 9). Presentation arithmetic only: every amount here is the kernel's, moved about inside the ring;
// a held shape is said in pencil until the kernel answers for it (RB1 writes authored zones in the ask's shape).
// The part is general: the same functions would edit a disc's gas with distance or an air's layers with height.

/** Why a zone is where it is: history's draw, a sculptor's mark, a catalogue, or the GM. */
export interface ZoneCause {
  kind: "base" | "seeded" | "resonance" | "embedded" | "catalogued" | "authored";
  /** The sculptor (a moon round a world, a planet round a star): the ask's `sculptors[].id`. */
  sculptor?: string;
  /** A resonance's order, read as first order m:(m − 1): Mimas 2:1 is m = 2, Janus 7:6 is m = 7. */
  m?: number;
}

/** One zone of the kernel's answer: a share of the ring's mass over a span of distance. */
export interface RingZone {
  inner_m: number;
  outer_m: number;
  mass_kg: number;
  tau: number;
  sigma_kg_m2: number;
  albedo: number;
  /** Linear 0..1 (not sRGB): `srgb` turns it into a colour to draw. */
  rgb: [number, number, number];
  name?: string;
  cause: ZoneCause;
}

/** A sculptor's mark on the ring: an edge it holds, a gap round it, a wave train. */
export interface RingMark {
  kind: "edge" | "gap" | "wave";
  /** The sculptor's id in the ask (its name is also matched, while the kernel answers with names). */
  sculptor: string;
  /** Where it is, in metres from the parent's centre. */
  r_m: number;
  half_width_m?: number;
  /** A resonance's order, as for a zone's cause; gaps carry none. */
  m?: number | null;
}

/** The kernel's answer `ring_structure_is` (design section 9). Zone masses sum to `mass_kg`. */
export interface RingStructure {
  inner_m: number;
  outer_m: number;
  mass_kg: number;
  zones: RingZone[];
  marks: RingMark[];
  /** The fine profile, sampled evenly from lo to hi: optical depth per sample and the zone it falls in (−1 in a gap). */
  profile: { lo_m: number; hi_m: number; tau: number[]; zone: number[] };
  why: string[];
}

/** A body that shapes the ring (the ask's `sculptors`): moons today, planets round a star later. */
export interface Sculptor {
  id: string;
  name: string;
  a_m: number;
  mass_kg: number;
}

/** A zone as the Profile shows and moves it. */
export interface ProfileSpan {
  name: string;
  inner_m: number;
  outer_m: number;
  mass_kg: number;
  /** Its share of the ring's mass (all spans sum to one). */
  share: number;
  sigma_kg_m2: number;
  tau: number;
  rgb: [number, number, number];
  cause: ZoneCause;
  /** The GM has shaped this zone (its edge or its height): inked, and written as authored. */
  yours: boolean;
}

/** An authored zone in the ask's shape (design section 9): what RB1 writes for a shaped ring. */
export interface AuthoredZone {
  name?: string;
  inner_m: number;
  outer_m: number;
  share: number;
  source: "authored";
}

/** The face of a zone between two distances (one side). */
export const annulus = (inner_m: number, outer_m: number): number => Math.PI * (outer_m * outer_m - inner_m * inner_m);

/** The answer's zones as spans, inner first, each with its share of the whole. */
export function profileSpans(ring: RingStructure): ProfileSpan[] {
  const total = ring.zones.reduce((t, z) => t + z.mass_kg, 0);
  return [...ring.zones]
    .sort((a, b) => a.inner_m - b.inner_m)
    .map((z, i) => ({
      name: z.name ?? `Zone ${i + 1}`,
      inner_m: z.inner_m,
      outer_m: z.outer_m,
      mass_kg: z.mass_kg,
      share: total > 0 ? z.mass_kg / total : 0,
      sigma_kg_m2: z.sigma_kg_m2,
      tau: z.tau,
      rgb: z.rgb,
      cause: z.cause,
      yours: z.cause.kind === "authored",
    }));
}

/** The sum of the spans' masses: what shaping keeps. */
export const profileMass = (spans: readonly ProfileSpan[]): number => spans.reduce((t, s) => t + s.mass_kg, 0);

/** The sculptor a cause or mark names, by its id or (while the kernel answers with names) its name. */
export const sculptorOf = (ref: string | undefined, sculptors: readonly Sculptor[]): Sculptor | undefined => sculptors.find((s) => s.id === ref) ?? sculptors.find((s) => s.name === ref);

/** A sculptor's name (the reference itself when the ask does not list it). */
const nameOf = (ref: string | undefined, sculptors: readonly Sculptor[]): string => sculptorOf(ref, sculptors)?.name ?? ref ?? "a moon";

/** The small tag that says a zone's cause: "Mimas 2:1", "cleared by Pan", "as generated", "measured", "drawn by you". */
export function causeTag(cause: ZoneCause, sculptors: readonly Sculptor[]): string {
  switch (cause.kind) {
    case "resonance":
      return cause.m && cause.m > 1 ? `${nameOf(cause.sculptor, sculptors)} ${cause.m}:${cause.m - 1}` : `held by ${nameOf(cause.sculptor, sculptors)}`;
    case "embedded":
      return `cleared by ${nameOf(cause.sculptor, sculptors)}`;
    case "catalogued":
      return "measured";
    case "authored":
      return "drawn by you";
    default:
      return "as generated";
  }
}

/** What a mark is, in words: "edge held by Mimas (2:1)", "cleared by Pan", "waves from Mimas (5:4)". */
export function markWords(mark: RingMark, sculptors: readonly Sculptor[]): string {
  const n = nameOf(mark.sculptor, sculptors);
  const ratio = mark.m && mark.m > 1 ? ` (${mark.m}:${mark.m - 1})` : "";
  return mark.kind === "edge" ? `edge held by ${n}${ratio}` : mark.kind === "gap" ? `cleared by ${n}` : `waves from ${n}${ratio}`;
}

/** The joins: the inner edge of every span that touches the one before it (a ring's own edges are Where's, not here). */
export function joins(spans: readonly ProfileSpan[]): number[] {
  const out: number[] = [];
  for (let j = 1; j < spans.length; j++) {
    const a = spans[j - 1]!;
    const b = spans[j]!;
    if (Math.abs(a.outer_m - b.inner_m) <= 1e-9 * Math.max(1, b.inner_m)) out.push(j);
  }
  return out;
}

/** The narrowest a zone may be squeezed by a dragged join: a thousandth of the ring's width. */
const narrowest = (spans: readonly ProfileSpan[]): number => (spans.length ? (spans[spans.length - 1]!.outer_m - spans[0]!.inner_m) * 1e-3 : 0);

/** Each span's density and opacity from its mass (tau scales with sigma for the same grains). */
function refit(spans: ProfileSpan[], from: readonly ProfileSpan[]): ProfileSpan[] {
  return spans.map((s, i) => {
    const was = from[i]!;
    const sigma = s.mass_kg / annulus(s.inner_m, s.outer_m);
    return { ...s, sigma_kg_m2: sigma, tau: was.sigma_kg_m2 > 0 ? was.tau * (sigma / was.sigma_kg_m2) : was.tau };
  });
}

/** Scale every mass so they sum to `total` again, and give each its share. */
function keepTotal(spans: ProfileSpan[], total: number): ProfileSpan[] {
  const now = profileMass(spans);
  const k = now > 0 ? total / now : 0;
  return spans.map((s) => ({ ...s, mass_kg: s.mass_kg * k, share: total > 0 ? (s.mass_kg * k) / total : 0 }));
}

/**
 * Move the join at the inner edge of span `j` to `at_m`. Both zones keep how dense they were, so the moved edge is
 * the only change the eye sees; the mass the swept strip gains or loses is given back by every zone in proportion, so
 * the whole is unchanged. The join stops short of the next join on either side.
 */
export function moveJoin(spans: readonly ProfileSpan[], j: number, at_m: number): ProfileSpan[] {
  if (!joins(spans).includes(j)) return [...spans];
  const a = spans[j - 1]!;
  const b = spans[j]!;
  const gap = narrowest(spans);
  const at = Math.min(b.outer_m - gap, Math.max(a.inner_m + gap, at_m));
  if (at === b.inner_m) return [...spans];
  const total = profileMass(spans);
  const moved = spans.map((s, i) => {
    if (i === j - 1) return { ...s, outer_m: at, mass_kg: s.sigma_kg_m2 * annulus(s.inner_m, at), yours: true };
    if (i === j) return { ...s, inner_m: at, mass_kg: s.sigma_kg_m2 * annulus(at, s.outer_m), yours: true };
    return { ...s };
  });
  return refit(keepTotal(moved, total), spans);
}

/**
 * Set span `i`'s share of the ring and trade it against the rest, which keep their proportions (the air mix's rule),
 * so the whole is unchanged. A share of nothing is a cleared zone, kept so the edges stay.
 */
export function setShare(spans: readonly ProfileSpan[], i: number, share: number): ProfileSpan[] {
  const s = spans[i];
  if (!s) return [...spans];
  const total = profileMass(spans);
  const want = Math.min(1, Math.max(0, share));
  const rest = total - s.mass_kg;
  const out = spans.map((x, k) => {
    if (k === i) return { ...x, mass_kg: want * total, yours: true };
    const m = rest > 0 ? (x.mass_kg / rest) * (1 - want) * total : spans.length > 1 ? ((1 - want) * total) / (spans.length - 1) : 0;
    return { ...x, mass_kg: m };
  });
  return refit(keepTotal(out, total), spans);
}

/**
 * Make span `i` this dense (as a drag up or down sets it): its share is what that density over its face holds,
 * short of the whole ring, and the rest give way. The zone then sits exactly where the pointer put it.
 */
export function setDensity(spans: readonly ProfileSpan[], i: number, sigma_kg_m2: number): ProfileSpan[] {
  const s = spans[i];
  if (!s) return [...spans];
  const total = profileMass(spans);
  if (!(total > 0)) return [...spans];
  const m = Math.max(0, sigma_kg_m2) * annulus(s.inner_m, s.outer_m);
  // The rest keep at least a millionth between them, so a zone never swallows the ring by a drag (typing can).
  return setShare(spans, i, Math.min(1 - 1e-6, m / total));
}

/** The spans as authored zones for the ask (RB1): every zone, so the shape is kept whole. */
export const authoredZones = (spans: readonly ProfileSpan[]): AuthoredZone[] =>
  spans.map((s) => ({ name: s.name, inner_m: s.inner_m, outer_m: s.outer_m, share: s.share, source: "authored" }));

/** The marks that hold a span's edges: "Outer edge held by Mimas." */
export function edgesHeld(span: ProfileSpan, marks: readonly RingMark[], sculptors: readonly Sculptor[]): string[] {
  const near = (a: number, b: number) => Math.abs(a - b) <= 2e-3 * Math.max(a, b);
  const out: string[] = [];
  for (const m of marks) {
    if (m.kind !== "edge") continue;
    if (near(m.r_m, span.inner_m)) out.push(`Inner edge held by ${nameOf(m.sculptor, sculptors)}.`);
    else if (near(m.r_m, span.outer_m)) out.push(`Outer edge held by ${nameOf(m.sculptor, sculptors)}.`);
  }
  return out;
}

/** The fraction of light straight through a zone of this optical depth (the Curious reading of opacity). */
export const lightThrough = (tau: number): number => Math.exp(-Math.max(0, tau));

/** A linear 0..1 colour (the kernel's) as an sRGB CSS colour to draw. */
export function srgb(c: readonly number[]): string {
  const enc = (v: number) => {
    const l = Math.min(1, Math.max(0, v));
    return Math.round((l <= 0.0031308 ? 12.92 * l : 1.055 * l ** (1 / 2.4) - 0.055) * 255);
  };
  return `rgb(${enc(c[0] ?? 0)},${enc(c[1] ?? 0)},${enc(c[2] ?? 0)})`;
}
