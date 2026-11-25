// Look-up tables for the air in True (VP-A3), after Hillaire (2020): the light a world's air lets through from any
// height in any direction (transmittance), and the light it scatters more than once (multiple scattering, his ψ_ms).
// Both are worked out once per world from the kernel's own optics (`read_air_optics`: Rayleigh scattering at the
// ground per wavelength and the scale height, atmosphere.ts), on the processor, then read by the shader of the air's
// shell (gpu-air-shell.ts) in a few texture reads a step instead of a march towards the light at every step.
// Nothing here is typed in: the air is the kernel's, and the shell's top is where the density has fallen to e^-10.
// Rayleigh only for now (no haze or droplets: VP-A5), and no light bounced off the ground (docs/fudges.md).

/** One world's air in units of its own radius (radius 1). */
export interface AirShape {
  /** Rayleigh scattering at the ground per planet radius, red, green, blue. */
  beta: [number, number, number];
  /** Scale height over the radius. */
  scale_height: number;
}

/** Scale heights from the ground to the top of the shell (the density there is e^-TOP_H of the ground's). */
export const TOP_H = 10;

export const TRANS_W = 64;
export const TRANS_H = 32;
export const MS_N = 32;

export interface AirLuts {
  /** Top of the shell over the radius. */
  top: number;
  /** RGBA float rows, TRANS_W × TRANS_H: u the cosine of the zenith angle from -1 to 1, v the height from 0 to top. */
  transmittance: Float32Array;
  /** RGBA float rows, MS_N × MS_N: u the cosine of the sun's zenith angle from -1 to 1, v the height. ψ_ms per unit sun. */
  multiple: Float32Array;
}

const topOf = (a: AirShape) => 1 + TOP_H * a.scale_height;

/** Distance along a ray from radius r with zenith cosine mu to a sphere of radius R (outward), or -1 if it misses. */
function toSphere(r: number, mu: number, R: number, near: boolean): number {
  const disc = r * r * (mu * mu - 1) + R * R;
  if (disc < 0) return -1;
  const s = Math.sqrt(disc);
  const t = near ? -r * mu - s : -r * mu + s;
  return t;
}

/** Optical depth (column of relative density) from radius r along zenith cosine mu to the top; Infinity if it meets the ground. */
function column(a: AirShape, r: number, mu: number, top: number, steps = 40): number {
  const ground = toSphere(r, mu, 1, true);
  if (ground > 0 && mu < 0) return Infinity;
  const t = toSphere(r, mu, top, false);
  if (!(t > 0)) return 0;
  const ds = t / steps;
  let sum = 0;
  for (let i = 0; i < steps; i++) {
    const s = (i + 0.5) * ds;
    const h = Math.sqrt(r * r + s * s + 2 * r * s * mu) - 1;
    sum += Math.exp(-h / a.scale_height) * ds;
  }
  return sum;
}

/** Transmittance from height h (over the radius) looking along zenith cosine mu, by the table (bilinear). */
export function readTransmittance(l: AirLuts, h: number, mu: number, out: number[]): void {
  read(l.transmittance, TRANS_W, TRANS_H, (mu + 1) / 2, h / (l.top - 1), out);
}

function read(tab: Float32Array, w: number, hgt: number, u: number, v: number, out: number[]): void {
  const x = Math.min(w - 1, Math.max(0, u * (w - 1)));
  const y = Math.min(hgt - 1, Math.max(0, v * (hgt - 1)));
  const i = Math.min(w - 2, Math.floor(x));
  const j = Math.min(hgt - 2, Math.floor(y));
  const fx = x - i;
  const fy = y - j;
  for (let c = 0; c < 3; c++) {
    const a = tab[(j * w + i) * 4 + c]! * (1 - fx) + tab[(j * w + i + 1) * 4 + c]! * fx;
    const b = tab[((j + 1) * w + i) * 4 + c]! * (1 - fx) + tab[((j + 1) * w + i + 1) * 4 + c]! * fx;
    out[c] = a * (1 - fy) + b * fy;
  }
}

/**
 * The two tables for a world's air. Multiple scattering after Hillaire (2020) section 5.5: from each point, the
 * second-order light L2 and the share f_ms the air sends on again, gathered over the sphere of directions with an
 * isotropic phase; the infinite series of further orders is L2 / (1 - f_ms).
 */
export function airLuts(a: AirShape, dirs = 64, steps = 20): AirLuts {
  const top = topOf(a);
  const transmittance = new Float32Array(TRANS_W * TRANS_H * 4);
  for (let j = 0; j < TRANS_H; j++) {
    const r = 1 + (j / (TRANS_H - 1)) * (top - 1);
    for (let i = 0; i < TRANS_W; i++) {
      const mu = (i / (TRANS_W - 1)) * 2 - 1;
      const col = column(a, r, mu, top);
      for (let c = 0; c < 3; c++) transmittance[(j * TRANS_W + i) * 4 + c] = Number.isFinite(col) ? Math.exp(-a.beta[c]! * col) : 0;
      transmittance[(j * TRANS_W + i) * 4 + 3] = 1;
    }
  }
  const luts: AirLuts = { top, transmittance, multiple: new Float32Array(MS_N * MS_N * 4) };
  // Directions spread evenly over the sphere (a Fibonacci lattice), each with the same solid angle 4π / dirs.
  const golden = Math.PI * (3 - Math.sqrt(5));
  const D: [number, number, number][] = [];
  for (let k = 0; k < dirs; k++) {
    const z = 1 - (2 * (k + 0.5)) / dirs;
    const s = Math.sqrt(1 - z * z);
    D.push([s * Math.cos(golden * k), s * Math.sin(golden * k), z]);
  }
  const iso = 1 / (4 * Math.PI);
  const Ts = [0, 0, 0];
  for (let j = 0; j < MS_N; j++) {
    const r0 = 1 + (j / (MS_N - 1)) * (top - 1);
    for (let i = 0; i < MS_N; i++) {
      const muS = (i / (MS_N - 1)) * 2 - 1;
      // The point at (0, 0, r0); the sun in the x-z plane.
      const sun: [number, number, number] = [Math.sqrt(Math.max(0, 1 - muS * muS)), 0, muS];
      const L2 = [0, 0, 0];
      const F = [0, 0, 0];
      for (const d of D) {
        const ground = toSphere(r0, d[2], 1, true);
        const t = ground > 0 && d[2] < 0 ? ground : toSphere(r0, d[2], top, false);
        if (!(t > 0)) continue;
        const ds = t / steps;
        const tau = [0, 0, 0];
        const l2 = [0, 0, 0];
        const f = [0, 0, 0];
        for (let s = 0; s < steps; s++) {
          const x = d[0] * (s + 0.5) * ds;
          const y = d[1] * (s + 0.5) * ds;
          const z = r0 + d[2] * (s + 0.5) * ds;
          const r = Math.sqrt(x * x + y * y + z * z);
          const h = r - 1;
          const rho = Math.exp(-h / a.scale_height);
          const mu = (x * sun[0] + y * sun[1] + z * sun[2]) / r;
          readTransmittance(luts, h, mu, Ts);
          for (let c = 0; c < 3; c++) {
            const sigma = a.beta[c]! * rho;
            const half = tau[c]! + 0.5 * sigma * ds;
            const tv = Math.exp(-half);
            l2[c]! += tv * sigma * Ts[c]! * iso * ds;
            f[c]! += tv * sigma * ds;
            tau[c]! += sigma * ds;
          }
        }
        for (let c = 0; c < 3; c++) {
          // Second order: the light reaching the point from this direction, scattered isotropically towards any one.
          L2[c]! += l2[c]! * iso * ((4 * Math.PI) / dirs);
          F[c]! += f[c]! * iso * ((4 * Math.PI) / dirs);
        }
      }
      const o = (j * MS_N + i) * 4;
      for (let c = 0; c < 3; c++) luts.multiple[o + c] = L2[c]! / Math.max(1e-6, 1 - F[c]!);
      luts.multiple[o + 3] = 1;
    }
  }
  return luts;
}

/** ψ_ms at a height and sun zenith cosine, by the table. */
export function readMultiple(l: AirLuts, h: number, muS: number, out: number[]): void {
  read(l.multiple, MS_N, MS_N, (muS + 1) / 2, h / (l.top - 1), out);
}

/** A world's air shape in its own radius from the kernel's optics (atmosphere.ts `Air`) and its radius in metres. */
export function airShapeOf(air: { rayleigh_m: [number, number, number]; scale_height_m: number }, radius_m: number): AirShape {
  return { beta: air.rayleigh_m.map((b) => b * radius_m) as [number, number, number], scale_height: air.scale_height_m / radius_m };
}

export const AIR_LUT_REFS = [{ what: "Transmittance and multiple-scattering look-up tables for sky and aerial perspective", cite: "Hillaire 2020, Computer Graphics Forum 39(4), 13-22", url: "https://doi.org/10.1111/cgf.14050" }];
