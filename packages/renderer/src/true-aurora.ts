// Aurorae in system True (render audit build item 2): every world's aurora plan in one pass, so the system view draws
// the same shells the Body globe and the Land screen draw (aurora.ts `planAurora`, gpu-aurora.ts). The field is read
// once for the system (the Solar System's own worlds from read_field's measured fixture, by name), its resolved facts
// once (each world's MF3 field tilt and its air), and each world with an aurora asks the kernel for its air's optics
// (the scale height that sets a stand-in line's height). Worlds the kernel finds no aurora for are left out.
import { planAurora, type AuroraJson, type AuroraPlan } from "./aurora.ts";
import { airFactsOf, airFromOptics, airRequest } from "./atmosphere.ts";
import type { FieldReadJson } from "./field.ts";
import type { Json } from "../../ui-contract/src/index.ts";

/** What the loader asks of the session. */
export interface AuroraSystemReader {
  readField(ask: { fixture?: string; system?: string }): Promise<{ ok: true; bodies: unknown[] } | { ok: false; code: string; message: string }>;
  resolvedSystem(system: string): Promise<{ ok: true; result: { facts: Record<string, Record<string, unknown>>; surfaces: unknown[] } } | { ok: false; [k: string]: unknown }>;
  readAirOptics(airs: Json[], wavelengths_m?: number[]): Promise<{ ok: true; airs: Json[] } | { ok: false; code: string; message: string }>;
}

/** Each world's plan with layers to draw, by address; `stillWanted` stops between requests. */
export async function loadSystemAurorae(k: AuroraSystemReader, system: string, worlds: readonly { address: string; label: string; radius_m: number }[], stillWanted: () => boolean = () => true): Promise<Map<string, { plan: AuroraPlan; radius_m: number }>> {
  const out = new Map<string, { plan: AuroraPlan; radius_m: number }>();
  const sol = /\/sys:sol$/.test(system);
  const r = await k.readField(sol ? { fixture: "solar-system" } : { system });
  const reads = r.ok ? (r.bodies as (FieldReadJson & { aurora?: AuroraJson | null; address?: string; name?: string })[]) : [];
  if (!reads.some((x) => x.aurora && (x.aurora.diffuse || x.aurora.ovals.length))) return out;
  const rs = await k.resolvedSystem(system);
  const facts = rs.ok ? rs.result.facts : {};
  const surfaces = rs.ok ? (rs.result.surfaces as { address?: string }[]) : [];
  for (const w of worlds) {
    if (!stillWanted()) return out;
    const read = sol ? reads.find((x) => x.name === w.label) : reads.find((x) => x.address === w.address);
    if (!read?.aurora || (!read.aurora.diffuse && !read.aurora.ovals.length)) continue;
    const f = facts[w.address] as Record<string, Json> | undefined;
    const air = airFactsOf(f, surfaces.find((x) => x.address === w.address) as Parameters<typeof airFactsOf>[1]);
    let scale_height_m: number | undefined;
    const req = air ? airRequest(w.address, air) : null;
    if (req) {
      const ao = await k.readAirOptics([req.air]);
      if (ao.ok && ao.airs[0]) scale_height_m = airFromOptics(ao.airs[0] as Parameters<typeof airFromOptics>[0], req).scale_height_m;
    }
    const tilt = !sol && typeof f?.["field.tilt"] === "number" ? (f["field.tilt"] as number) : undefined;
    const radius_m = w.radius_m > 0 ? w.radius_m : (read.field?.reference_radius_m ?? 0);
    if (!(radius_m > 0)) continue;
    const plan = planAurora(read, { radius_m, ...(scale_height_m ? { scale_height_m } : {}), ...(air ? { composition: air.composition } : {}), ...(tilt !== undefined ? { tilt_deg: tilt } : {}) });
    if (plan.layers.length) out.set(w.address, { plan, radius_m });
  }
  return out;
}
