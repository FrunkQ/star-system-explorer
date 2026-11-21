// A world's air as the atmosphere pass needs it (R2): scattering
// coefficients at three wavelengths and a scale height. The physics lives in
// the kernel (light track, `read_air_optics`: Rayleigh cross-sections per gas
// with dispersion and King factors from the base-light pack, n = P / kT,
// H = kT / (mu m_u g)); this file only builds the request from a body's facts
// and turns the answer into the renderer's numbers. Gases the kernel's table
// lacks are left out and listed, never invented. Aerosols and absorption are
// not modelled yet (docs/fudges.md).
import type { Json } from "../../ui-contract/src/index.ts";

export const REFERENCES = [
  { text: "Bucholtz, A. (1995). Rayleigh-scattering calculations for the terrestrial atmosphere. Applied Optics 34(15), 2765-2773.", doi: "10.1364/AO.34.002765" },
  { text: "Nishita, T., Sirai, T., Tadamura, K., Nakamae, E. (1993). Display of the Earth taking into account atmospheric scattering. SIGGRAPH 93, 175-182.", doi: "10.1145/166117.166140" },
  { text: "Hillaire, S. (2020). A scalable and production ready sky and atmosphere rendering technique. Computer Graphics Forum 39(4), 13-22.", doi: "10.1111/cgf.14050" },
];

/** The three wavelengths the renderer carries (red, green, blue), metres. */
export const WAVELENGTHS_M = [680e-9, 550e-9, 440e-9] as const;

/** Gases the kernel's `read_air_optics` knows (base-light pack), by the formula the facts use. */
export const KERNEL_GASES: Readonly<Record<string, string>> = { N2: "n2", O2: "o2", Ar: "ar", CO2: "co2", H2: "h2", He: "he", CH4: "ch4", H2O: "h2o", NH3: "nh3", SO2: "so2", CO: "co", Ne: "ne" };

export interface AirFacts {
  pressure_pa: number;
  temperature_k: number;
  gravity_m_s2: number;
  radius_m: number;
  /** Mole fractions by formula; need not sum to 1 (they are normalised). */
  composition: Record<string, number>;
  /** The kernel's optics for this air (`read_air_optics`), once asked. Without it nothing is drawn. */
  optics?: Air | null;
}

export interface Air {
  /** Rayleigh scattering at the surface, per metre, at the three wavelengths. */
  rayleigh_m: [number, number, number];
  scale_height_m: number;
  /** Molecules per m^3 at the surface. */
  density_m3: number;
  /** Mean molecular mass in atomic mass units. */
  mean_molar_mass: number;
  /** Optical depth straight up, per wavelength. */
  zenith_depth: [number, number, number];
  /** Gases in the mix the kernel's table lacks (left out, and said). */
  unknown_gases: string[];
  /** The share of the mix left out because it is unknown. */
  unknown_share: number;
}

/** The `read_air_optics` request entry for a body's facts: known gases renormalised, the rest set aside. Null when nothing is known. */
export function airRequest(address: string, f: AirFacts): { air: Json; unknown_gases: string[]; unknown_share: number } | null {
  if (!(f.pressure_pa > 0 && f.temperature_k > 0 && f.gravity_m_s2 > 0)) return null;
  const entries = Object.entries(f.composition).filter(([, x]) => x > 0);
  const total = entries.reduce((a, [, x]) => a + x, 0);
  const known = entries.filter(([g]) => KERNEL_GASES[g]);
  const knownTotal = known.reduce((a, [, x]) => a + x, 0);
  if (!knownTotal) return null;
  return {
    air: { address, gases: known.map(([g, x]) => ({ gas: KERNEL_GASES[g]!, fraction: x / knownTotal })), surface_pressure_pa: f.pressure_pa, surface_temperature_k: f.temperature_k, gravity_m_s2: f.gravity_m_s2 },
    unknown_gases: entries.filter(([g]) => !KERNEL_GASES[g]).map(([g]) => g),
    unknown_share: total ? 1 - knownTotal / total : 0,
  };
}

/** The renderer's air from one `read_air_optics` answer (asked at the three default wavelengths). */
export function airFromOptics(o: { mean_molecular_mass_kg: number; scale_height_m: number; surface_density_m3: number; at: { wavelength_m: number; beta_per_m: number; tau: number }[] }, req: { unknown_gases: string[]; unknown_share: number }): Air {
  const pick = (lam: number) => o.at.reduce((b, x) => (Math.abs(x.wavelength_m - lam) < Math.abs(b.wavelength_m - lam) ? x : b));
  const at = WAVELENGTHS_M.map(pick);
  return {
    rayleigh_m: at.map((x) => x.beta_per_m) as [number, number, number],
    scale_height_m: o.scale_height_m,
    density_m3: o.surface_density_m3,
    mean_molar_mass: o.mean_molecular_mass_kg / 1.66053906660e-27,
    zenith_depth: at.map((x) => x.tau) as [number, number, number],
    unknown_gases: req.unknown_gases,
    unknown_share: req.unknown_share,
  };
}

/** The air to draw: the kernel's optics when they have been read, else none. */
export function air(f: AirFacts): Air | null {
  return f.optics ?? null;
}

/** The air facts of one body from `resolved_system` facts and its surface record, or null when it has no air. */
export function airFactsOf(facts: Record<string, Json> | undefined, surface: { gravity_m_s2?: number; t_surface_now_k?: number; reference_radius_m?: number } | undefined): AirFacts | null {
  if (!facts) return null;
  const p = facts["body.surface_pressure"];
  if (typeof p !== "number" || !(p > 0)) return null;
  const atm = facts["v3.atmosphere"] as { composition?: Record<string, number> } | undefined;
  const comp = atm?.composition && Object.keys(atm.composition).length ? atm.composition : null;
  const g = surface?.gravity_m_s2;
  const t = surface?.t_surface_now_k;
  const r = surface?.reference_radius_m ?? (typeof facts["body.radius"] === "number" ? (facts["body.radius"] as number) : undefined);
  if (!comp || !g || !t || !r) return null;
  return { pressure_pa: p, temperature_k: t, gravity_m_s2: g, radius_m: r, composition: comp };
}
