/**
 * A world's one climate, as the kernel's `survey_surface` summarises it
 * (W7a): the circulation regime, temperature and rain by latitude, and the
 * kernel's own "why" and "assumed" lines. Read-only: the chart room shows it
 * in plain words; the Workshop's climate blocks come later (W12).
 */
export interface ClimateSummary {
  key: string;
  regime: string;
  cells: number;
  hadley_edge_deg: number;
  /** Band centres, south to north, with the band's mean temperature and rain. */
  lat_deg: number[];
  t_k: number[];
  precip_m_yr: number[];
  lapse_k_km: number;
  liquid: string | null;
  water: boolean;
  pressure_bar: number;
  assumed: string[];
  why: string[];
}

export interface ClimateBelt {
  /** "60–90° N", "Equator to 30° S" ... */
  label: string;
  t_k: number;
  precip_m_yr: number;
}

const EDGES = [90, 60, 30, 0, -30, -60, -90];

/** "earth-like" reads "Earth-like"; the kernel's regime key in words. */
export function regimeWords(regime: string): string {
  const w = regime.replace(/-/g, " ").replace(/\bearth\b/i, "Earth").replace(/ like\b/, "-like");
  return w.charAt(0).toUpperCase() + w.slice(1);
}

const edge = (d: number) => (d === 0 ? "the equator" : `${Math.abs(d)}° ${d > 0 ? "N" : "S"}`);

/**
 * The climate's bands gathered into six 30° belts, north to south, each the
 * area-weighted mean (by the cosine of latitude) of the bands whose centres
 * fall in it. A short table: the kernel's own bands stay one step away.
 */
export function climateBelts(c: Pick<ClimateSummary, "lat_deg" | "t_k" | "precip_m_yr">): ClimateBelt[] {
  const out: ClimateBelt[] = [];
  for (let i = 0; i + 1 < EDGES.length; i++) {
    const hi = EDGES[i]!;
    const lo = EDGES[i + 1]!;
    let w = 0;
    let t = 0;
    let p = 0;
    c.lat_deg.forEach((lat, k) => {
      if (lat > hi || lat < lo || (lat === lo && lo !== -90)) return;
      const a = Math.cos((lat * Math.PI) / 180);
      w += a;
      t += a * (c.t_k[k] ?? 0);
      p += a * (c.precip_m_yr[k] ?? 0);
    });
    if (w <= 0) continue;
    const label = hi === 0 || lo === 0 ? (hi === 0 ? `Equator to ${edge(lo)}` : `Equator to ${edge(hi)}`) : `${Math.min(Math.abs(lo), Math.abs(hi))}–${Math.max(Math.abs(lo), Math.abs(hi))}° ${hi > 0 ? "N" : "S"}`;
    out.push({ label, t_k: t / w, precip_m_yr: p / w });
  }
  return out;
}

/** A kernel "why" line that says the world is outside the climate's tested range. */
export function isExtrapolation(line: string): boolean {
  return /extrapolat|outside the tested range/i.test(line);
}
