// World stacks (P4, `sse.world-stack/1`): the shells, regions and features
// on, in and above a world, as plain data. The Cutaway view draws them; this
// file only reads and words them (the "above, on and within" ladder), the
// same way sse-world's ladder() does, so the two can be compared. Physics
// checks on a stack belong to the kernel.

export interface WorldShell {
  key: string;
  kind: string;
  name: string;
  top_m: number;
  bottom_m: number;
  within?: string;
  properties?: Record<string, number>;
  provides?: string[];
}
export type Extent = { type: "global" } | { type: "cap"; from_lat_deg: number } | { type: "band"; min_lat_deg: number; max_lat_deg: number } | { type: "unmapped" };
export interface WorldRegion {
  key: string;
  kind: string;
  name: string;
  on: string;
  extent: Extent;
  top_m?: number;
  bottom_m?: number;
  provides?: string[];
}
export interface WorldFeature {
  key: string;
  kind: string;
  name?: string;
  in?: string;
  altitude_m: number;
  lat_deg?: number;
  lon_deg?: number;
  provides?: string[];
}
export interface WorldStack {
  format: "sse.world-stack/1";
  body: string;
  name: string;
  radius_m: number;
  shells: WorldShell[];
  regions: WorldRegion[];
  features: WorldFeature[];
}

export function checkWorldStack(w: unknown): string[] {
  const x = w as Partial<WorldStack>;
  const out: string[] = [];
  if (!x || x.format !== "sse.world-stack/1") return ["format must be sse.world-stack/1"];
  if (!(typeof x.radius_m === "number" && x.radius_m > 0)) out.push("radius_m must be positive");
  for (const s of x.shells ?? []) if (!(s.top_m > s.bottom_m)) out.push(`shell ${s.key}: top must be above bottom`);
  const keys = new Set([...(x.shells ?? []), ...(x.regions ?? []), ...(x.features ?? [])].map((o) => o.key));
  for (const r of x.regions ?? []) if (!keys.has(r.on)) out.push(`region ${r.key}: on unknown ${r.on}`);
  for (const f of x.features ?? []) if (f.in && !keys.has(f.in)) out.push(`feature ${f.key}: in unknown ${f.in}`);
  return out;
}

/** The kernel's format_km: "+400", "0", "−0.4", "−2,900" (U+2212 minus). */
export function formatKm(alt_m: number): string {
  const km = alt_m / 1000;
  if (km === 0) return "0";
  const a = Math.abs(km);
  const body = a === Math.trunc(a) ? String(a).replace(/\B(?=(\d{3})+(?!\d))/g, ",") : a.toFixed(1);
  return (km > 0 ? "+" : "\u2212") + body;
}

const KIND_PHRASE: Record<string, string> = { lava_tube_city: "lava-tube city", magma_chamber: "magma chamber", ice_shell: "ice shell", sub_ice_ocean: "sub-ice ocean", aerostat_band: "aerostat band", polar_cap: "polar cap" };
export const kindPhrase = (k: string) => KIND_PHRASE[k] ?? k.replace(/_/g, " ");
const cap = (s: string) => s.charAt(0).toUpperCase() + s.slice(1);
const PROVISION: Record<string, string> = { dynamo: "dynamo on" };

export interface LadderLine {
  key: string;
  depth: string;
  text: string;
  where: string;
}

/**
 * The "above, on and within" ladder, top to bottom: a line-for-line port of
 * sse-world's `WorldStack::ladder()` (crates/sse-world/src/lib.rs), so the
 * view words a stack exactly as the kernel does. Features above the surface
 * highest first; one surface line (biomes, cities, other surface features);
 * below it features, regional shells (at their base, or as a range when they
 * start below the surface) and global shells at their base ("Crust ends, 9
 * plates"), the innermost at its top.
 */
export function ladder(w: WorldStack): LadderLine[] {
  const shell = (k: string) => w.shells.find((s) => s.key === k);
  const region = (k: string) => w.regions.find((r) => r.key === k);
  const feature = (k: string) => w.features.find((f) => f.key === k);
  const container = (c: string): string => {
    const r = region(c) ?? shell(c);
    if (r) return r.name;
    const f = feature(c);
    return f ? `under ${f.name ?? kindPhrase(f.kind)}` : c;
  };
  const ftext = (f: WorldFeature, comma: boolean) => (f.name ? `${f.name}${comma ? "," : ""} ${kindPhrase(f.kind)}` : cap(kindPhrase(f.kind)));
  const lines: LadderLine[] = [];
  const above = w.features.filter((f) => f.altitude_m > 0).sort((a, b) => b.altitude_m - a.altitude_m || (a.key < b.key ? -1 : a.key > b.key ? 1 : 0));
  for (const f of above) lines.push({ key: f.key, depth: `${formatKm(f.altitude_m)} km`, text: ftext(f, true), where: f.in ? container(f.in) : "orbit" });
  const parts: string[] = [];
  if (w.regions.some((r) => r.kind === "biome")) parts.push("Biomes");
  const surface = w.features.filter((f) => f.altitude_m === 0);
  const cities = surface.filter((f) => f.kind === "city").length;
  if (cities) parts.push(`${cities} ${cities === 1 ? "city" : "cities"}`);
  for (const f of surface.filter((f) => f.kind !== "city")) parts.push(ftext(f, false));
  if (parts.length) lines.push({ key: "", depth: "0", text: parts.join(", "), where: "surface" });
  const below: [number, number, LadderLine][] = [];
  for (const f of w.features.filter((f) => f.altitude_m < 0)) below.push([f.altitude_m, f.altitude_m, { key: f.key, depth: `${formatKm(f.altitude_m)} km`, text: ftext(f, true), where: f.in ? container(f.in) : "" }]);
  for (const s of w.shells.filter((s) => s.within && s.bottom_m < 0)) {
    const [depth, top] = s.top_m >= 0 ? [`${formatKm(s.bottom_m)} km`, s.bottom_m] : [`${formatKm(s.top_m)} to ${formatKm(s.bottom_m)} km`, s.top_m];
    below.push([top, s.bottom_m, { key: s.key, depth, text: s.name, where: container(s.within!) }]);
  }
  const global = w.shells.filter((s) => !s.within && s.bottom_m < 0).sort((a, b) => b.top_m - a.top_m);
  const boundary = new Map<number, [number, LadderLine]>();
  global.forEach((s, i) => {
    const innermost = i + 1 === global.length;
    const extras: string[] = [];
    const plates = w.regions.filter((r) => r.on === s.key && r.kind === "plate").length;
    if (plates) extras.push(`${plates} plates`);
    for (const p of s.provides ?? []) if (PROVISION[p]) extras.push(PROVISION[p]);
    const at = innermost ? s.top_m : s.bottom_m;
    if (at >= 0) return;
    const text = [innermost ? s.name : `${s.name} ends`, ...extras].join(", ");
    // Millimetre buckets: lines at the same depth collide; the deeper shell (later) wins.
    boundary.set(Math.trunc(at * 1000), [at, { key: s.key, depth: `${formatKm(at)} km`, text, where: "shell" }]);
  });
  for (const [at, l] of [...boundary.values()]) below.push([at, at, l]);
  below.sort((a, b) => b[0] - a[0] || b[1] - a[1]);
  return [...lines, ...below.map((x) => x[2])];
}

/** Counts by kind for regions the stack only counts (plates, biomes): "9 plates, 3 biomes". */
export function regionCounts(w: WorldStack): string {
  const c = new Map<string, number>();
  for (const r of w.regions) c.set(r.kind, (c.get(r.kind) ?? 0) + 1);
  return [...c].map(([k, n]) => `${n} ${k.replace(/_/g, " ")}${n === 1 ? "" : k.endsWith("s") ? "" : "s"}`).join(", ");
}

/**
 * Display radius of a height, 0 at the centre and 1 at the top of the
 * highest shell (features above every shell, such as stations, come out
 * above 1 and the view puts them in the margin). `scale` "true" keeps
 * proportions. "log" gives the air above the surface the outer fifth,
 * linear in height, and spreads depth below the surface on a log scale, so
 * a 3 km ice lid and a 35 km crust show on a 6000 km world (the depth axis
 * says so).
 */
export function cutRadius(w: WorldStack, altitude_m: number, scale: "true" | "log"): number {
  const top = Math.max(0, ...w.shells.map((s) => s.top_m));
  const R = w.radius_m;
  if (scale === "true") return Math.max(0, (R + altitude_m) / (R + top));
  const surf = top > 0 ? 0.8 : 1;
  if (altitude_m >= 0) return top > 0 ? surf + (1 - surf) * (altitude_m / top) : 1 + altitude_m / R;
  const d_km = Math.min(R, -altitude_m) / 1000;
  return Math.max(0, surf * (1 - Math.log10(1 + d_km) / Math.log10(1 + R / 1000)));
}

/** `read_world_stack`'s answer (physics track, K-physics-ops.md): the stack, the kernel's ladder and physics notes. */
export interface WorldStackRead {
  type: "world_stack_read";
  body: string;
  /** "fixture:<name>", "given", later "built:<epoch>". */
  source: string;
  stack: WorldStack;
  ladder: LadderLine[];
  notes: string[];
  problems: { at: string; message: string }[];
  invalid: { at: string; message: string }[];
}
