// The sky and light at the ground (SL1), from the kernel's air optics (`read_air_optics`, atmosphere.ts `Air`):
// single scattering by molecules (Rayleigh) in an exponential atmosphere, in closed form. For an observer at the
// ground looking along a direction with air mass m_v, sunlight arriving along a direction with air mass m_s, and
// optical depth τ straight up, light scattered once toward the eye is
//
//   L = E · P(γ) · m_v · (e^(-τ m_s) − e^(-τ m_v)) / (m_v − m_s),   P(γ) = 3 (1 + cos² γ) / 16π
//
// (integrating β(z) P E e^(-τ(z) m_s) e^(-(τ − τ(z)) m_v) along the ray, with dτ = −β dz). The air mass is that
// of a uniform shell one scale height thick round the curved world (Schoenberg 1929), m(μ) = sqrt((k μ)² + 2k + 1)
// − k μ with k = R / H: 1 overhead, finite at the horizon (about 40 for Earth, against Kasten and Young's 38). Multiple scattering is left out (it lightens thick skies), as on the globe (fudges).
// The same sums run in the GPU shader (gpu-surface.ts); this file keeps them for tests, the canvas tier and words.

/** The air mass along a direction whose cosine to the zenith is μ, for a world of radius R and scale height H. */
export function airMass(mu: number, radius_m: number, scale_height_m: number): number {
  const k = radius_m / Math.max(1, scale_height_m);
  return Math.sqrt((k * mu) ** 2 + 2 * k + 1) - k * mu;
}

/** Rayleigh's phase function. */
export const rayleighPhase = (cosGamma: number): number => (3 * (1 + cosGamma * cosGamma)) / (16 * Math.PI);

/** Sky radiance per unit of sunlight in one channel. `muV` view and `muS` sun cosines to the zenith, `cosG` the angle between them. */
export function skyRadiance(tau: number, muV: number, muS: number, cosG: number, radius_m: number, scale_height_m: number): number {
  if (tau <= 0 || muS < -0.2) return 0;
  const mv = airMass(Math.max(muV, 0), radius_m, scale_height_m);
  const ms = airMass(muS, radius_m, scale_height_m);
  const p = rayleighPhase(cosG);
  const d = mv - ms;
  const f = Math.abs(d) < 1e-6 ? tau * Math.exp(-tau * mv) : (Math.exp(-tau * ms) - Math.exp(-tau * mv)) / d;
  return p * mv * f;
}

/** Sunlight reaching the ground, as a fraction of what arrives at the top of the air. */
export function sunTransmittance(tau: number, muS: number, radius_m: number, scale_height_m: number): number {
  if (muS < -0.1) return 0;
  return Math.exp(-tau * airMass(muS, radius_m, scale_height_m));
}

/**
 * Light from the whole sky on flat ground, per unit of sunlight: half of the light scattered out of the sun's beam
 * on its way down (Rayleigh scattering sends as much forward as back), on a surface tilted μ_s to it. An
 * approximation of the sky's integral, listed in the fudge register.
 */
export function skyIrradiance(tau: number, muS: number, radius_m: number, scale_height_m: number): number {
  if (tau <= 0 || muS <= -0.1) return 0;
  return 0.5 * Math.max(0, muS) * (1 - sunTransmittance(tau, muS, radius_m, scale_height_m));
}

/**
 * The sun's place in the sky at a latitude and local hour angle for a solar declination (degrees; hour 0 is local
 * noon, 15° an hour). Returns its direction in the site's frame (x east, y north, z up).
 */
export function sunDirection(lat_deg: number, hour_angle_deg: number, declination_deg = 0): [number, number, number] {
  const la = (lat_deg * Math.PI) / 180;
  const h = (hour_angle_deg * Math.PI) / 180;
  const d = (declination_deg * Math.PI) / 180;
  const z = Math.sin(la) * Math.sin(d) + Math.cos(la) * Math.cos(d) * Math.cos(h);
  const x = -Math.cos(d) * Math.sin(h);
  const y = Math.cos(la) * Math.sin(d) - Math.sin(la) * Math.cos(d) * Math.cos(h);
  const l = Math.hypot(x, y, z) || 1;
  return [x / l, y / l, z / l];
}
