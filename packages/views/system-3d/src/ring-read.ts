// One ring per world, read from the kernel for True (track SB, VP-R3 and VP-R2). A ringed world's ring is asked for
// whole (`ring_structure`): its catalogued bands, or the ring formation made, with the world's mass and radius and its
// moons as the sculptors. The kernel places the edges, gaps and wave trains, varies the optical depth across every
// band and colours the grains from what they are made of; the drawing only reads it (renderer rings.ts).
import type { KernelRing } from "../../../renderer/src/index.ts";
import type { Body3D } from "./bodies.ts";

/** Samples across a ring: about 9 km a pixel across Saturn's main rings, enough for its wave trains' first crests. */
export const RING_SAMPLES = 8192;

export interface RingReader {
  ringStructure(ask: { ask?: unknown; system?: string; cloud?: string; variant?: number; samples?: number; window?: [number, number] }): Promise<{ ok: true; ring: unknown } | { ok: false; code: string; message: string }>;
}

/** A world's moons as the ring's sculptors: name, distance and mass, for those whose mass is on record. */
export function sculptorsOf(bodies: readonly Body3D[], id: string): { id: string; name: string; a_m: number; mass_kg: number }[] {
  return bodies.filter((b) => b.parent === id && b.a_m !== null && b.mass_kg !== undefined).map((b) => ({ id: b.id, name: b.name, a_m: b.a_m!, mass_kg: b.mass_kg! }));
}

/** The kernel's ask for a world's ring, from what the scene holds: null when the world has no ring, mass or radius. */
export function ringAskFor(bodies: readonly Body3D[], id: string, samples = RING_SAMPLES): { system?: string; cloud?: string; ask?: unknown; samples: number } | null {
  const b = bodies.find((x) => x.id === id);
  if (!b?.rings?.length) return null;
  // A ring formation made: the kernel builds the ask from its run (system is the cloud's parent address).
  const formed = b.rings.find((r) => r.cloud);
  if (formed?.cloud) {
    const system = formed.cloud.slice(0, formed.cloud.lastIndexOf("/"));
    return { system, cloud: formed.cloud, samples };
  }
  if (!(b.mass_kg && b.mass_kg > 0 && b.radius_m && b.radius_m > 0)) return null;
  const zones = b.rings
    .filter((r) => r.outer_m > r.inner_m && r.inner_m > 0)
    .map((r) => ({
      name: r.name,
      inner_m: r.inner_m,
      outer_m: r.outer_m,
      ...(r.tau !== null ? { tau: r.tau } : {}),
      ...(r.tau_range ? { tau_range: r.tau_range } : {}),
      ...(r.albedo !== null ? { albedo: r.albedo } : {}),
      source: "catalogued",
    }));
  if (!zones.length) return null;
  return {
    ask: {
      key: id,
      host: { mass_kg: b.mass_kg, radius_m: b.radius_m, ...(b.j2 ? { j2: b.j2 } : {}) },
      sculptors: sculptorsOf(bodies, id),
      ring: { inner_m: Math.min(...zones.map((z) => z.inner_m)), outer_m: Math.max(...zones.map((z) => z.outer_m)), zones },
    },
    samples,
  };
}

function asRing(j: unknown): KernelRing | null {
  const r = j as Partial<KernelRing> | null;
  if (!r || !Array.isArray(r.zones) || !r.profile || !Array.isArray(r.profile.tau) || !Array.isArray(r.profile.zone) || !r.phase) return null;
  return r as KernelRing;
}

/**
 * Every ringed world's ring among `bodies`, asked one after another (the kernel is shared with the surfaces being
 * read). `stillWanted` stops between requests when the view has moved on. A world whose ring the kernel refuses is
 * left out, and drawn from its bands as before.
 */
export async function readRings(k: RingReader, bodies: readonly Body3D[], stillWanted: () => boolean = () => true): Promise<Map<string, KernelRing>> {
  const out = new Map<string, KernelRing>();
  for (const b of bodies) {
    if (!b.rings?.length) continue;
    if (!stillWanted()) return out;
    const ask = ringAskFor(bodies, b.id);
    if (!ask) continue;
    const r = await k.ringStructure(ask);
    const ring = r.ok ? asRing(r.ring) : null;
    if (ring) out.set(b.id, ring);
  }
  return out;
}
