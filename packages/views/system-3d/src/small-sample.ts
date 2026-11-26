// Belt members in True (render audit build item 3, track SB part SB3 wiring): a belt is a probability field until a
// member is looked at (sse-small), so True asks the kernel to resolve a few members of each belt round the star and
// draws them in their own shapes, where their orbits put them. Which members: the 1st, 10th, 100th and 1000th largest
// (`SAMPLE_RANKS`), so the picture spans the big round ones and the small lumpy ones the kernel makes. Nothing is
// invented here: the figure, albedo, weathering, orbit and, for an icy body, its activity at the distance it is now
// at are all the kernel's (`small_body`). The ranks shown are a drawing's choice (fudge register, "Belt members in True").
import { KM_PER_AU } from "../../../ui-contract/src/index.ts";
import type { CometActivity, SmallFigure } from "../../../renderer/src/index.ts";

/** Which members of each belt are resolved and drawn, by size rank (1 the largest). */
export const SAMPLE_RANKS = [1, 10, 100, 1000] as const;

/** A resolved member as True draws it. */
export interface SmallSample {
  address: string;
  /** The belt it belongs to (its cloud address). */
  cloud: string;
  rank: number;
  /** "Main belt, 10th largest (S type)". */
  name: string;
  /** Its figure, albedo and weathering (renderer small-body.ts). */
  figure: SmallFigure;
  /** Mean radius, metres. */
  radius_m: number;
  /** Distance from the star now and its longitude in the belt's plane, from the kernel's orbit. */
  r_m: number;
  longitude_deg: number;
  /** An icy body's activity at `r_m` (kernel SB4), when it holds ice. */
  activity?: CometActivity;
  /** The kernel's type label and why, for the note. */
  type_label: string;
}

/** What the kernel's `small_body` answer holds that True reads. */
interface MemberJson {
  address: string;
  cloud: string;
  rank: number;
  diameter_km: number;
  albedo: number;
  weathering: number;
  type_label: string;
  makeup?: { ice?: number };
  orbit: { a_au: number; e: number; peri_deg: number; node_deg: number; mean_anomaly_deg: number };
  figure: { kind: string; axes_m: [number, number, number]; lobes: { axes_m: [number, number, number]; offset_m: number }[]; relief: number };
  activity?: CometActivity;
}

export interface SmallReader {
  smallBody(ask: { system: string; member?: string; cloud?: string; rank?: number; r_au?: number }): Promise<{ ok: true; body: unknown; notables: unknown[] } | { ok: false; code: string; message: string }>;
}

/**
 * Where the orbit puts the body now: Kepler's equation for the eccentric anomaly from the kernel's mean anomaly, then
 * the distance a(1 − e cos E) and the true anomaly. The longitude is node + perihelion argument + true anomaly, in the
 * belt's plane (the system view lays orbits flat; the inclination is left out of the drawing).
 */
export function placeOnOrbit(o: MemberJson["orbit"]): { r_au: number; longitude_deg: number } {
  const e = Math.min(0.99, Math.max(0, o.e));
  const M = (((o.mean_anomaly_deg % 360) + 360) % 360) * (Math.PI / 180);
  let E = e < 0.8 ? M : Math.PI;
  for (let i = 0; i < 30; i++) {
    const d = (E - e * Math.sin(E) - M) / (1 - e * Math.cos(E));
    E -= d;
    if (Math.abs(d) < 1e-12) break;
  }
  const nu = 2 * Math.atan2(Math.sqrt(1 + e) * Math.sin(E / 2), Math.sqrt(1 - e) * Math.cos(E / 2));
  const lon = o.node_deg + o.peri_deg + (nu * 180) / Math.PI;
  return { r_au: o.a_au * (1 - e * Math.cos(E)), longitude_deg: ((lon % 360) + 360) % 360 };
}

const ordinal = (n: number) => (n === 1 ? "largest" : `${n}${n % 10 === 1 && n % 100 !== 11 ? "st" : n % 10 === 2 && n % 100 !== 12 ? "nd" : n % 10 === 3 && n % 100 !== 13 ? "rd" : "th"} largest`);

function asMember(j: unknown): MemberJson | null {
  const m = j as Partial<MemberJson> | null;
  if (!m || typeof m.address !== "string" || !m.orbit || !m.figure || !Array.isArray(m.figure.axes_m) || !(Number(m.diameter_km) > 0)) return null;
  return m as MemberJson;
}

/** One member resolved, with an icy one's activity asked again at the distance its orbit puts it now. */
async function readOne(k: SmallReader, system: string, cloud: string, rank: number, beltName: string): Promise<SmallSample | null> {
  const first = await k.smallBody({ system, cloud, rank });
  if (!first.ok) return null;
  let m = asMember(first.body);
  if (!m) return null;
  const at = placeOnOrbit(m.orbit);
  if (m.activity) {
    const again = await k.smallBody({ system, cloud, rank, r_au: at.r_au });
    const n = again.ok ? asMember(again.body) : null;
    if (n) m = n;
  }
  const figure: SmallFigure = { kind: m.figure.kind, axes_m: m.figure.axes_m, lobes: m.figure.lobes ?? [], relief: m.figure.relief ?? 0, albedo: m.albedo, weathering: m.weathering };
  return {
    address: m.address,
    cloud,
    rank,
    name: `${beltName}, ${ordinal(rank)}${m.type_label ? ` (${m.type_label} type)` : ""}`,
    figure,
    radius_m: (m.diameter_km * 1000) / 2,
    r_m: at.r_au * KM_PER_AU * 1000,
    longitude_deg: at.longitude_deg,
    ...(m.activity ? { activity: m.activity } : {}),
    type_label: m.type_label ?? "",
  };
}

/**
 * The sample members of each belt, resolved one after another (the kernel is shared with the surfaces being read).
 * `stillWanted` stops between requests when the view has moved on. A belt with fewer members than a rank stops there.
 */
export async function readSmallSample(k: SmallReader, system: string, belts: readonly { address: string; name: string }[], stillWanted: () => boolean = () => true): Promise<SmallSample[]> {
  const out: SmallSample[] = [];
  for (const b of belts)
    for (const rank of SAMPLE_RANKS) {
      if (!stillWanted()) return out;
      const s = await readOne(k, system, b.address, rank, b.name);
      if (!s) break;
      out.push(s);
    }
  return out;
}
