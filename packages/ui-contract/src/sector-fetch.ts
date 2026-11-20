/**
 * Fetching community sector data through the site's own Worker (handoff 7A
 * part 4; worker/index.js `/tmap/*`), only when the user asks. The text goes
 * straight to the kernel's `import_sector` op; the attribution travels with
 * it into the import report.
 */

export interface SectorText {
  text: string;
  attribution: string;
  /** What was asked for, for the report. */
  source: string;
}

export type Fetch = (url: string) => Promise<{ ok: boolean; status: number; text(): Promise<string>; headers: { get(k: string): string | null } }>;

/** The Worker path for a sector, or one subsector of it (A to P). */
export function sectorPath(sector: string, subsector?: string): string {
  const q = new URLSearchParams({ sector });
  if (subsector) q.set("subsector", subsector);
  return `/tmap/sector?${q.toString()}`;
}

export async function fetchSector(get: Fetch, sector: string, subsector?: string, base = ""): Promise<SectorText> {
  const res = await get(base + sectorPath(sector, subsector));
  const body = await res.text();
  if (!res.ok) throw new Error(res.status === 404 ? `no sector called ${sector}` : body || `sector fetch failed (${res.status})`);
  return {
    text: body,
    attribution: res.headers.get("x-attribution") ?? "Sector data from the Traveller Map (travellermap.com).",
    source: subsector ? `${sector}, subsector ${subsector}` : sector,
  };
}
