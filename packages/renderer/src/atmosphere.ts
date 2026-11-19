// A world's air as the atmosphere pass needs it (R2): scattering
// coefficients at three wavelengths and a scale height, computed from the
// kernel's facts (surface pressure, surface temperature, gravity, gas mix)
// with published physics, never picked to look right.
//
// Rayleigh scattering per molecule (Bucholtz 1995, eq. 2):
//   sigma(lambda) = 24 pi^3 / (lambda^4 Ns^2) * ((ns^2 - 1) / (ns^2 + 2))^2 * Fk
// with ns the gas's refractive index at the reference density Ns and Fk its
// King correction factor. The air's coefficient is the number density at
// the surface (ideal gas, n = P / kT) times the mole-weighted cross-section,
// and it falls off with height over the scale height H = kT / (mu m_u g).
// Dispersion of the refractive index with wavelength is ignored (it changes
// the blue by a few per cent); aerosols and absorption (ozone, methane) are
// not modelled yet and are reported as missing, not invented.
import type { Json } from "../../ui-contract/src/index.ts";

export const REFERENCES = [
  { text: "Bucholtz, A. (1995). Rayleigh-scattering calculations for the terrestrial atmosphere. Applied Optics 34(15), 2765-2773.", doi: "10.1364/AO.34.002765" },
  { text: "Nishita, T., Sirai, T., Tadamura, K., Nakamae, E. (1993). Display of the Earth taking into account atmospheric scattering. SIGGRAPH 93, 175-182.", doi: "10.1145/166117.166140" },
  { text: "Hillaire, S. (2020). A scalable and production ready sky and atmosphere rendering technique. Computer Graphics Forum 39(4), 13-22.", doi: "10.1111/cgf.14050" },
];

const K_B = 1.380649e-23;
const M_U = 1.66053906660e-27;
/** Loschmidt's number: molecules per m^3 at 0 °C and 1 atm, where the refractivities below are given. */
const N_LOSCHMIDT = 2.6867811e25;
/** The three wavelengths the renderer carries (red, green, blue), metres. */
export const WAVELENGTHS_M = [680e-9, 550e-9, 440e-9] as const;

/**
 * Per gas: molar mass (g/mol), refractivity (n - 1) at 0 °C and 1 atm near
 * 550 nm, and King factor. Values from standard tables (CRC Handbook;
 * Bucholtz 1995 for N2, O2, Ar, CO2 King factors); others 1.0. Marked for
 * the citations thread to verify.
 */
export const GASES: Readonly<Record<string, { mass: number; refractivity: number; king: number }>> = {
  N2: { mass: 28.014, refractivity: 2.98e-4, king: 1.034 },
  O2: { mass: 31.998, refractivity: 2.71e-4, king: 1.096 },
  Ar: { mass: 39.948, refractivity: 2.81e-4, king: 1.0 },
  CO2: { mass: 44.009, refractivity: 4.49e-4, king: 1.15 },
  H2: { mass: 2.016, refractivity: 1.39e-4, king: 1.0 },
  He: { mass: 4.0026, refractivity: 3.5e-5, king: 1.0 },
  CH4: { mass: 16.043, refractivity: 4.41e-4, king: 1.0 },
  H2O: { mass: 18.015, refractivity: 2.54e-4, king: 1.0 },
  NH3: { mass: 17.031, refractivity: 3.76e-4, king: 1.0 },
  SO2: { mass: 64.066, refractivity: 6.86e-4, king: 1.0 },
  CO: { mass: 28.010, refractivity: 3.38e-4, king: 1.0 },
  Ne: { mass: 20.180, refractivity: 6.7e-5, king: 1.0 },
};

export interface AirFacts {
  pressure_pa: number;
  temperature_k: number;
  gravity_m_s2: number;
  radius_m: number;
  /** Mole fractions by formula; need not sum to 1 (they are normalised). */
  composition: Record<string, number>;
}

export interface Air {
  /** Rayleigh scattering at the surface, per metre, at the three wavelengths. */
  rayleigh_m: [number, number, number];
  scale_height_m: number;
  /** Molecules per m^3 at the surface. */
  density_m3: number;
  mean_molar_mass: number;
  /** Optical depth straight up, per wavelength. */
  zenith_depth: [number, number, number];
  /** Gases in the mix the table lacks (left out, and said). */
  unknown_gases: string[];
  /** The share of the mix left out because it is unknown. */
  unknown_share: number;
}

export function air(f: AirFacts): Air | null {
  if (!(f.pressure_pa > 0 && f.temperature_k > 0 && f.gravity_m_s2 > 0)) return null;
  const entries = Object.entries(f.composition).filter(([, x]) => x > 0);
  const total = entries.reduce((a, [, x]) => a + x, 0);
  const known = entries.filter(([g]) => GASES[g]);
  const knownTotal = known.reduce((a, [, x]) => a + x, 0);
  if (!knownTotal) return null;
  const n = f.pressure_pa / (K_B * f.temperature_k);
  let mu = 0;
  const sigma = [0, 0, 0];
  for (const [g, x] of known) {
    const gas = GASES[g]!;
    const w = x / knownTotal;
    mu += w * gas.mass;
    const ns = 1 + gas.refractivity;
    const lorentz = (ns * ns - 1) / (ns * ns + 2);
    WAVELENGTHS_M.forEach((lam, i) => {
      sigma[i]! += w * ((24 * Math.PI ** 3) / (lam ** 4 * N_LOSCHMIDT ** 2)) * lorentz * lorentz * gas.king;
    });
  }
  const H = (K_B * f.temperature_k) / (mu * M_U * f.gravity_m_s2);
  const beta = sigma.map((s) => s * n) as [number, number, number];
  return {
    rayleigh_m: beta,
    scale_height_m: H,
    density_m3: n,
    mean_molar_mass: mu,
    zenith_depth: beta.map((b) => b * H) as [number, number, number],
    unknown_gases: entries.filter(([g]) => !GASES[g]).map(([g]) => g),
    unknown_share: total ? 1 - knownTotal / total : 0,
  };
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
