// The Field layer's shared line tracer (MF14a part 1, magnetic fields design
// sections 5, 8 and 9). A body's own field is the gradient of a potential
// written as Gauss coefficients (Schmidt semi-normalised, as IGRF and the
// giant-planet models are published); the wind's bubble is Shue's surface
// from read_field's stand-off and flaring. Lines are traced once per change
// of field or wind, never per frame, and the same traced geometry serves the
// Chart cut, the 3D layer and (MF14c) the Field instrument.
// Presentation only: a line is a picture of a fact, not a new fact.

export type V3 = [number, number, number];

/** A body's internal field. Coordinates are body-fixed: z along the spin axis, x toward longitude 0; lengths in body radii. */
export interface FieldModel {
  /** Reference radius of the coefficients, m (lengths below are in these radii). */
  radius_m: number;
  /** g[n][m] and h[n][m], T, Schmidt semi-normalised; index 0 unused for n. */
  g: number[][];
  h: number[][];
  /** Centre of the expansion from the body's centre, radii (an offset dipole). */
  offset?: V3;
  /**
   * Dipoles outside the body standing in for currents beyond it: `moment` is the field's size and direction as an
   * equatorial surface field (T) times its direction. Used for the wind's compression (see `compressed`).
   */
  images?: { at: V3; moment: V3 }[];
  /** A uniform field added everywhere, T (the wind's own field, joined to the body's: Dungey 1961). */
  uniform?: V3;
}

/** Where the wind holds the field: Shue et al. 1998's r = r0 (2 / (1 + cos θ))^α, θ from the sunward direction. */
export interface Bubble {
  standoff_radii: number;
  flaring: number;
  /** Unit vector toward the wind's source, body frame. */
  sunward: V3;
  /** The wind's own field at the body, T, body frame. Without it every line closes (no reconnection, no open cap). */
  wind_field_t?: V3;
}

/** A dipole as coefficients: equatorial surface field B0 (T), tilt from the spin axis and the longitude it leans toward (degrees). */
export function dipole(radius_m: number, b0_t: number, tilt_deg = 0, toward_lon_deg = 0, offset?: V3, southward = true): FieldModel {
  const t = (tilt_deg * Math.PI) / 180;
  const p = (toward_lon_deg * Math.PI) / 180;
  // Earth's sign: the moment points south, so g10 < 0 and the field leaves the southern hemisphere.
  const s = southward ? -1 : 1;
  const g = [[], [s * b0_t * Math.cos(t), s * b0_t * Math.sin(t) * Math.cos(p)]];
  const h = [[], [0, s * b0_t * Math.sin(t) * Math.sin(p)]];
  return { radius_m, g, h, ...(offset ? { offset } : {}) };
}

export function degreeOf(f: FieldModel): number {
  return Math.max(1, f.g.length - 1);
}

/** Schmidt semi-normalised P_n^m(cos θ) and dP/dθ for n ≤ N (no Condon–Shortley phase). */
export function schmidt(N: number, theta: number): { P: number[][]; dP: number[][] } {
  const x = Math.cos(theta);
  const s = Math.sin(theta);
  const P: number[][] = [];
  const dP: number[][] = [];
  for (let n = 0; n <= N; n++) ((P[n] = new Array(n + 1).fill(0)), (dP[n] = new Array(n + 1).fill(0)));
  P[0]![0] = 1;
  // Schmidt-normalised recursions (Winch et al. 2005, eqs. for the semi-normalised functions).
  for (let n = 1; n <= N; n++) {
    for (let m = 0; m <= n; m++) {
      if (n === m) {
        const k = n === 1 ? 1 : Math.sqrt((2 * n - 1) / (2 * n));
        P[n]![n] = k * s * P[n - 1]![n - 1]!;
        dP[n]![n] = k * (x * P[n - 1]![n - 1]! + s * dP[n - 1]![n - 1]!);
      } else {
        const a = Math.sqrt(n * n - m * m);
        const b = n >= 2 && m <= n - 2 ? Math.sqrt((n - 1) * (n - 1) - m * m) : 0;
        const p2 = m <= n - 2 ? P[n - 2]![m]! : 0;
        const d2 = m <= n - 2 ? dP[n - 2]![m]! : 0;
        P[n]![m] = ((2 * n - 1) * x * P[n - 1]![m]! - b * p2) / a;
        dP[n]![m] = ((2 * n - 1) * (x * dP[n - 1]![m]! - s * P[n - 1]![m]!) - b * d2) / a;
      }
    }
  }
  return { P, dP };
}

/** Scratch for fieldAt: flat (n, m) tables reused between calls, so tracing allocates nothing per step. */
const scratch = { N: -1, P: new Float64Array(0), dP: new Float64Array(0), cm: new Float64Array(0), sm: new Float64Array(0), A: new Float64Array(0), B: new Float64Array(0) };
const ix = (n: number, m: number) => (n * (n + 1)) / 2 + m;
function prepare(N: number) {
  if (scratch.N === N) return;
  const size = ((N + 1) * (N + 2)) / 2;
  scratch.N = N;
  scratch.P = new Float64Array(size);
  scratch.dP = new Float64Array(size);
  scratch.cm = new Float64Array(N + 1);
  scratch.sm = new Float64Array(N + 1);
  // The recursion's constants (see schmidt above).
  scratch.A = new Float64Array(size);
  scratch.B = new Float64Array(size);
  for (let n = 1; n <= N; n++)
    for (let m = 0; m <= n; m++) {
      if (n === m) scratch.A[ix(n, m)] = n === 1 ? 1 : Math.sqrt((2 * n - 1) / (2 * n));
      else ((scratch.A[ix(n, m)] = Math.sqrt(n * n - m * m)), (scratch.B[ix(n, m)] = m <= n - 2 ? Math.sqrt((n - 1) * (n - 1) - m * m) : 0));
    }
}

/** The field at a point (radii, body frame), T, in Cartesian components. */
export function fieldAt(f: FieldModel, p: V3): V3 {
  const o = f.offset ?? [0, 0, 0];
  const x = p[0] - o[0];
  const y = p[1] - o[1];
  const z = p[2] - o[2];
  // Math.sqrt, not Math.hypot: hypot is several times slower in browsers and the values here never overflow.
  const r = Math.sqrt(x * x + y * y + z * z);
  if (r === 0) return [0, 0, 0];
  const rxy = Math.sqrt(x * x + y * y);
  // Keep off the poles, where the spherical components are singular; the field is smooth there.
  const rho = Math.max(rxy, r * 1e-9);
  const rr = Math.sqrt(rho * rho + z * z);
  const ct = z / rr;
  const st = rho / rr;
  const cp = rxy > 0 ? x / rxy : 1;
  const sp = rxy > 0 ? y / rxy : 0;
  const N = degreeOf(f);
  prepare(N);
  const { P, dP, cm, sm, A, B: Bk } = scratch;
  // cos mφ and sin mφ by the angle-sum recurrence.
  cm[0] = 1;
  sm[0] = 0;
  for (let m = 1; m <= N; m++) ((cm[m] = cm[m - 1]! * cp - sm[m - 1]! * sp), (sm[m] = sm[m - 1]! * cp + cm[m - 1]! * sp));
  P[0] = 1;
  dP[0] = 0;
  let br = 0;
  let bt = 0;
  let bp = 0;
  const ir = 1 / r;
  let k = ir * ir;
  for (let n = 1; n <= N; n++) {
    k *= ir;
    const gn = f.g[n];
    const hn = f.h[n];
    for (let m = 0; m <= n; m++) {
      const i = ix(n, m);
      if (n === m) {
        const j = ix(n - 1, n - 1);
        P[i] = A[i]! * st * P[j]!;
        dP[i] = A[i]! * (ct * P[j]! + st * dP[j]!);
      } else {
        const j = ix(n - 1, m);
        const p2 = m <= n - 2 ? P[ix(n - 2, m)]! : 0;
        const d2 = m <= n - 2 ? dP[ix(n - 2, m)]! : 0;
        P[i] = ((2 * n - 1) * ct * P[j]! - Bk[i]! * p2) / A[i]!;
        dP[i] = ((2 * n - 1) * (ct * dP[j]! - st * P[j]!) - Bk[i]! * d2) / A[i]!;
      }
      const g = gn?.[m] ?? 0;
      const h = hn?.[m] ?? 0;
      if (g === 0 && h === 0) continue;
      const gh = g * cm[m]! + h * sm[m]!;
      br += (n + 1) * k * gh * P[i]!;
      bt -= k * gh * dP[i]!;
      bp += (k * m * (g * sm[m]! - h * cm[m]!) * P[i]!) / st;
    }
  }
  const out: V3 = [br * st * cp + bt * ct * cp - bp * sp, br * st * sp + bt * ct * sp + bp * cp, br * ct - bt * st];
  if (f.images) for (const im of f.images) addDipole(out, im.at, im.moment, p);
  if (f.uniform) ((out[0] += f.uniform[0]), (out[1] += f.uniform[1]), (out[2] += f.uniform[2]));
  return out;
}

/** Add a point dipole's field at p: B = [3 (m·n) n − m] / d³, with m in tesla at unit distance. */
function addDipole(B: V3, at: V3, m: V3, p: V3) {
  const dx = p[0] - at[0];
  const dy = p[1] - at[1];
  const dz = p[2] - at[2];
  const d2 = dx * dx + dy * dy + dz * dz;
  if (d2 === 0) return;
  const d = Math.sqrt(d2);
  const nx = dx / d;
  const ny = dy / d;
  const nz = dz / d;
  const mn = m[0] * nx + m[1] * ny + m[2] * nz;
  const k = 1 / (d2 * d);
  B[0] += k * (3 * mn * nx - m[0]);
  B[1] += k * (3 * mn * ny - m[1]);
  B[2] += k * (3 * mn * nz - m[2]);
}

/**
 * The wind's squeeze on the field (Chapman and Ferraro 1931): the currents on the bubble's sunward face act like an
 * image of the body's dipole mirrored in a plane at the stand-off, so the field at the nose doubles, the factor of 2
 * in read_field's own pressure balance. Only the dipole part is mirrored; the night side's tail currents are not
 * modelled (fudge register, "sse.field.trace").
 */
export function compressed(f: FieldModel, b: Bubble): FieldModel {
  const ax = dipoleAxis(f);
  const b0 = dipoleB0(f);
  const o = f.offset ?? [0, 0, 0];
  // The moment points away from the pole the field enters (ax); its mirror in a plane facing the wind keeps the part
  // along the plane and flips the part across it, which leaves the field at the plane doubled along the plane.
  const m: V3 = [-ax[0] * b0, -ax[1] * b0, -ax[2] * b0];
  const s = b.sunward;
  const ms = m[0] * s[0] + m[1] * s[1] + m[2] * s[2];
  const mirrored: V3 = [m[0] - 2 * ms * s[0], m[1] - 2 * ms * s[1], m[2] - 2 * ms * s[2]];
  const os = o[0] * s[0] + o[1] * s[1] + o[2] * s[2];
  const at: V3 = [o[0] + 2 * (b.standoff_radii - os) * s[0], o[1] + 2 * (b.standoff_radii - os) * s[1], o[2] + 2 * (b.standoff_radii - os) * s[2]];
  return { ...f, images: [...(f.images ?? []), { at, moment: mirrored }] };
}

/**
 * The field the layer traces inside a bubble: the body's own, squeezed by the wind (Chapman and Ferraro), with the
 * wind's own field added (Dungey 1961's open magnetosphere, as a vacuum superposition: Cowley 1973). Lines that join
 * the wind's field leave through the bubble: those are the open lines over the polar caps.
 */
export function inBubble(f: FieldModel, b: Bubble | null, squeeze = true): FieldModel {
  if (!b) return f;
  const g = squeeze ? compressed(f, b) : f;
  return b.wind_field_t ? { ...g, uniform: b.wind_field_t } : g;
}

const dot = (a: V3, b: V3) => a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
const norm = (a: V3) => Math.sqrt(a[0] * a[0] + a[1] * a[1] + a[2] * a[2]);

/** Is the point inside the bubble? (No bubble: everywhere is inside.) */
export function insideBubble(b: Bubble | null, p: V3): boolean {
  if (!b) return true;
  const r = norm(p);
  if (r === 0) return true;
  const c = dot(p, b.sunward) / r;
  return r <= b.standoff_radii * Math.pow(2 / (1 + c), b.flaring);
}

export interface FieldLine {
  /** Points from the footpoint outward, radii. */
  points: V3[];
  /** |B| at each point, T. */
  strength: number[];
  /** At each point, +1 where the field points away from the body's centre, −1 toward it: the two inks of the 3D layer. */
  outward: (1 | -1)[];
  /** "closed": lands on the body again; "open": leaves through the bubble into the wind; "unfinished": ran out of steps or reach. */
  kind: "closed" | "open" | "unfinished";
  /** +1 where the field leaves the surface at the footpoint, −1 where it enters. */
  polarity: 1 | -1;
  /** The footpoint, latitude and longitude in degrees (body frame). */
  foot: { lat: number; lon: number };
}

export interface TraceOptions {
  /** Step as a share of the distance from the centre (smaller near the body). */
  step?: number;
  maxSteps?: number;
  /** Stop beyond this distance, radii, as unfinished (a bound for a body with no bubble). */
  reach?: number;
}

/** Trace one line from a footpoint on the surface (lat, lon in degrees), outward along the field, by fourth-order Runge–Kutta. */
export function traceLine(f: FieldModel, bubble: Bubble | null, lat: number, lon: number, o: TraceOptions = {}): FieldLine {
  const step = o.step ?? 0.04;
  const maxSteps = o.maxSteps ?? 1200;
  const reach = o.reach ?? (bubble ? bubble.standoff_radii * 8 : 40);
  const la = (lat * Math.PI) / 180;
  const lo = (lon * Math.PI) / 180;
  // Start a hair above the surface so the first step is not counted as landing.
  const start: V3 = [Math.cos(la) * Math.cos(lo) * 1.0001, Math.cos(la) * Math.sin(lo) * 1.0001, Math.sin(la) * 1.0001];
  const b0 = fieldAt(f, start);
  const polarity: 1 | -1 = dot(b0, start) >= 0 ? 1 : -1;
  let lastB = 0;
  let lastOut: 1 | -1 = 1;
  const dir = (p: V3): V3 => {
    const b = fieldAt(f, p);
    const n = norm(b) || 1;
    lastB = n;
    lastOut = dot(b, p) >= 0 ? 1 : -1;
    return [(polarity * b[0]) / n, (polarity * b[1]) / n, (polarity * b[2]) / n];
  };
  const points: V3[] = [start];
  const strength: number[] = [norm(b0)];
  const outward: (1 | -1)[] = [polarity];
  let p = start;
  let kind: FieldLine["kind"] = "unfinished";
  let k1 = dir(p);
  for (let i = 0; i < maxSteps; i++) {
    const h = step * Math.max(1, norm(p));
    const p2: V3 = [p[0] + 0.5 * h * k1[0], p[1] + 0.5 * h * k1[1], p[2] + 0.5 * h * k1[2]];
    const k2 = dir(p2);
    const p3: V3 = [p[0] + 0.5 * h * k2[0], p[1] + 0.5 * h * k2[1], p[2] + 0.5 * h * k2[2]];
    const k3 = dir(p3);
    const p4: V3 = [p[0] + h * k3[0], p[1] + h * k3[1], p[2] + h * k3[2]];
    const k4 = dir(p4);
    let q: V3 = [
      p[0] + (h / 6) * (k1[0] + 2 * k2[0] + 2 * k3[0] + k4[0]),
      p[1] + (h / 6) * (k1[1] + 2 * k2[1] + 2 * k3[1] + k4[1]),
      p[2] + (h / 6) * (k1[2] + 2 * k2[2] + 2 * k3[2] + k4[2]),
    ];
    const r = norm(q);
    if (r <= 1) {
      // Landed: put the last point where the step crosses the surface (|p + u (q − p)| = 1).
      const d: V3 = [q[0] - p[0], q[1] - p[1], q[2] - p[2]];
      const A = dot(d, d);
      const B = 2 * dot(p, d);
      const C = dot(p, p) - 1;
      const u = A > 0 ? (-B - Math.sqrt(Math.max(0, B * B - 4 * A * C))) / (2 * A) : 1;
      q = [p[0] + u * d[0], p[1] + u * d[1], p[2] + u * d[2]];
      points.push(q);
      const bq = fieldAt(f, q);
      strength.push(norm(bq));
      outward.push(dot(bq, q) >= 0 ? 1 : -1);
      kind = "closed";
      break;
    }
    points.push(q);
    // The next step's first slope gives this point's strength too.
    k1 = dir(q);
    strength.push(lastB);
    outward.push(lastOut);
    if (!insideBubble(bubble, q)) {
      kind = "open";
      break;
    }
    if (r > reach) break;
    p = q;
  }
  return { points, strength, outward, kind, polarity, foot: { lat, lon } };
}

/** The magnetic north pole's direction (where the dipole part's field points in): from g10, g11, h11. */
export function dipoleAxis(f: FieldModel): V3 {
  const g10 = f.g[1]?.[0] ?? 0;
  const g11 = f.g[1]?.[1] ?? 0;
  const h11 = f.h[1]?.[1] ?? 0;
  const n = Math.hypot(g10, g11, h11) || 1;
  // The moment is along (g11, h11, g10); the field enters the body at the pole the moment points away from.
  return [-g11 / n, -h11 / n, -g10 / n];
}

/** Equatorial field of the dipole part at the reference radius, T. */
export function dipoleB0(f: FieldModel): number {
  return Math.hypot(f.g[1]?.[0] ?? 0, f.g[1]?.[1] ?? 0, f.h[1]?.[1] ?? 0);
}

/**
 * Footpoints so that line spacing means strength: in each magnetic hemisphere, colatitudes at equal steps of the
 * dipole's flux through the cap (∝ sin² of the magnetic colatitude), on `meridians` magnetic longitudes.
 */
export function seeds(f: FieldModel, perHemisphere: number, meridians: number): { lat: number; lon: number; hemi: 1 | -1 }[] {
  const ax = dipoleAxis(f);
  // A frame round the magnetic axis.
  const ref: V3 = Math.abs(ax[2]) < 0.9 ? [0, 0, 1] : [1, 0, 0];
  const e1n: V3 = [ref[1] * ax[2] - ref[2] * ax[1], ref[2] * ax[0] - ref[0] * ax[2], ref[0] * ax[1] - ref[1] * ax[0]];
  const l1 = norm(e1n);
  const e1: V3 = [e1n[0] / l1, e1n[1] / l1, e1n[2] / l1];
  const e2: V3 = [ax[1] * e1[2] - ax[2] * e1[1], ax[2] * e1[0] - ax[0] * e1[2], ax[0] * e1[1] - ax[1] * e1[0]];
  const out: { lat: number; lon: number; hemi: 1 | -1 }[] = [];
  for (const hemi of [1, -1] as const) {
    for (let k = 1; k <= perHemisphere; k++) {
      // Equal flux steps from the pole down to the equator (not including it).
      const c = Math.asin(Math.sqrt(k / (perHemisphere + 1)));
      for (let j = 0; j < meridians; j++) {
        const a = (2 * Math.PI * (j + 0.5 * (k % 2))) / meridians;
        const z = hemi * Math.cos(c);
        const s = Math.sin(c);
        const d: V3 = [
          z * ax[0] + s * (Math.cos(a) * e1[0] + Math.sin(a) * e2[0]),
          z * ax[1] + s * (Math.cos(a) * e1[1] + Math.sin(a) * e2[1]),
          z * ax[2] + s * (Math.cos(a) * e1[2] + Math.sin(a) * e2[2]),
        ];
        out.push({ lat: (Math.asin(Math.max(-1, Math.min(1, d[2]))) * 180) / Math.PI, lon: (Math.atan2(d[1], d[0]) * 180) / Math.PI, hemi });
      }
    }
  }
  return out;
}

export interface FieldTrace {
  lines: FieldLine[];
  /** Strongest and weakest field met along the lines, T (for brightness or line weight). */
  range: [number, number];
  /** Milliseconds the trace took (for the cost table; not part of the result's meaning). */
  ms: number;
  counts: { closed: number; open: number; unfinished: number };
}

/**
 * Trace the layer's lines. Lines from the magnetic northern hemisphere are kept whole; from the southern one only
 * the open ones are kept, since a closed southern line repeats a northern one.
 */
export function traceField(f0: FieldModel, bubble: Bubble | null, o: TraceOptions & { perHemisphere?: number; meridians?: number; clock?: () => number; squeeze?: boolean } = {}): FieldTrace {
  const now = o.clock ?? (() => (typeof performance !== "undefined" ? performance.now() : 0));
  const t0 = now();
  const f = inBubble(f0, bubble, o.squeeze !== false);
  const lines: FieldLine[] = [];
  for (const s of seeds(f, o.perHemisphere ?? 16, o.meridians ?? 6)) {
    const l = traceLine(f, bubble, s.lat, s.lon, o);
    if (s.hemi === -1 && l.kind === "closed") continue;
    lines.push(l);
  }
  let lo = Infinity;
  let hi = 0;
  for (const l of lines) for (const b of l.strength) ((lo = Math.min(lo, b)), (hi = Math.max(hi, b)));
  const counts = { closed: 0, open: 0, unfinished: 0 };
  for (const l of lines) counts[l.kind]++;
  return { lines, range: [lo === Infinity ? 0 : lo, hi], ms: now() - t0, counts };
}

/** The bubble's outline in one plane through the centre, containing the sunward direction and `up` (radii, 2D: x sunward, y up). */
export function bubbleOutline(b: Bubble, samples = 96, maxTheta_deg = 150): { x: number; y: number }[] {
  const out: { x: number; y: number }[] = [];
  const tmax = (maxTheta_deg * Math.PI) / 180;
  for (let i = 0; i <= samples; i++) {
    const t = -tmax + (2 * tmax * i) / samples;
    const r = b.standoff_radii * Math.pow(2 / (1 + Math.cos(t)), b.flaring);
    out.push({ x: r * Math.cos(t), y: r * Math.sin(t) });
  }
  return out;
}

export interface ChartCut {
  /** In the cut's plane, radii: x toward the wind's source, y along the magnetic axis's part in that plane. */
  lines: { points: { x: number; y: number }[]; kind: FieldLine["kind"]; polarity: 1 | -1; weight: number }[];
  bubble: { x: number; y: number }[] | null;
  /** Magnetic and spin axes as directions in the plane. */
  magneticAxis: { x: number; y: number };
  spinAxis: { x: number; y: number };
  /** Angle between the magnetic and spin axes, degrees. */
  tilt_deg: number;
  trace: FieldTrace;
}

/**
 * The Body screen's Chart cut (design section 9): a plane through the centre holding the magnetic axis and the
 * wind's direction (the noon–midnight meridian). Seeds lie in that plane; lines are drawn projected onto it, which
 * is exact for a dipole and a fair cut for a many-poled field.
 */
export function chartCut(f0: FieldModel, bubble: Bubble | null, o: TraceOptions & { perHemisphere?: number; squeeze?: boolean } = {}): ChartCut {
  const f = inBubble(f0, bubble, o.squeeze !== false);
  const ax = dipoleAxis(f);
  const sun: V3 = bubble?.sunward ?? [1, 0, 0];
  // The plane's x: the sunward direction with its part along the magnetic axis removed (falls back if they align).
  let ex: V3 = [sun[0] - dot(sun, ax) * ax[0], sun[1] - dot(sun, ax) * ax[1], sun[2] - dot(sun, ax) * ax[2]];
  if (norm(ex) < 1e-6) ex = Math.abs(ax[0]) < 0.9 ? [1, 0, 0] : [0, 1, 0];
  const lx = norm(ex);
  ex = [ex[0] / lx, ex[1] / lx, ex[2] / lx];
  const ey = ax;
  const to2 = (p: V3) => ({ x: dot(p, ex), y: dot(p, ey) });
  const N = o.perHemisphere ?? 16;
  const lines: FieldLine[] = [];
  for (const side of [1, -1]) {
    for (const hemi of [1, -1]) {
      for (let k = 1; k <= N; k++) {
        const c = Math.asin(Math.sqrt(k / (N + 1)));
        const d: V3 = [
          hemi * Math.cos(c) * ey[0] + side * Math.sin(c) * ex[0],
          hemi * Math.cos(c) * ey[1] + side * Math.sin(c) * ex[1],
          hemi * Math.cos(c) * ey[2] + side * Math.sin(c) * ex[2],
        ];
        const lat = (Math.asin(Math.max(-1, Math.min(1, d[2]))) * 180) / Math.PI;
        const lon = (Math.atan2(d[1], d[0]) * 180) / Math.PI;
        const l = traceLine(f, bubble, lat, lon, o);
        if (hemi === -1 && l.kind === "closed") continue;
        lines.push(l);
      }
    }
  }
  let lo = Infinity;
  let hi = 0;
  for (const l of lines) for (const b of l.strength) ((lo = Math.min(lo, b)), (hi = Math.max(hi, b)));
  const counts = { closed: 0, open: 0, unfinished: 0 };
  for (const l of lines) counts[l.kind]++;
  const trace: FieldTrace = { lines, range: [lo, hi], ms: 0, counts };
  // Line weight by the field at the footpoint, on a log scale across what the cut holds.
  const w = (l: FieldLine) => {
    const b = l.strength[0]!;
    return hi > lo ? 0.5 + (Math.log(b / lo) / Math.log(hi / lo)) * 1 : 1;
  };
  const spin: V3 = [0, 0, 1];
  // The bubble's outline in this plane: sample directions in the plane and find the surface along each.
  let bub: { x: number; y: number }[] | null = null;
  if (bubble) {
    bub = [];
    for (let i = 0; i <= 120; i++) {
      const a = (-150 + (300 * i) / 120) * (Math.PI / 180);
      const d: V3 = [Math.cos(a) * ex[0] + Math.sin(a) * ey[0], Math.cos(a) * ex[1] + Math.sin(a) * ey[1], Math.cos(a) * ex[2] + Math.sin(a) * ey[2]];
      const c = dot(d, bubble.sunward);
      const r = bubble.standoff_radii * Math.pow(2 / (1 + c), bubble.flaring);
      bub.push({ x: r * Math.cos(a), y: r * Math.sin(a) });
    }
  }
  return {
    lines: lines.map((l) => ({ points: l.points.map(to2), kind: l.kind, polarity: l.polarity, weight: w(l) })),
    bubble: bub,
    magneticAxis: to2(ey),
    spinAxis: to2(spin),
    tilt_deg: (Math.acos(Math.max(-1, Math.min(1, Math.abs(ax[2])))) * 180) / Math.PI,
    trace,
  };
}

/**
 * The 3D layer's line data: one strip per line, positions in radii and a 0..1 brightness from the field's strength
 * (log scale over the trace's range), with polarity for the two colours. Ready for a GPU line pass or a canvas.
 */
export function lineBuffers(t: FieldTrace): { positions: Float32Array; brightness: Float32Array; starts: Uint32Array; counts: Uint32Array; polarity: Int8Array; kind: string[] } {
  let n = 0;
  for (const l of t.lines) n += l.points.length;
  const positions = new Float32Array(n * 3);
  const brightness = new Float32Array(n);
  const starts = new Uint32Array(t.lines.length);
  const counts = new Uint32Array(t.lines.length);
  const polarity = new Int8Array(t.lines.length);
  const [lo, hi] = t.range;
  const span = hi > lo ? Math.log(hi / lo) : 1;
  let i = 0;
  t.lines.forEach((l, k) => {
    starts[k] = i;
    counts[k] = l.points.length;
    polarity[k] = l.polarity;
    l.points.forEach((p, j) => {
      positions.set(p, i * 3);
      brightness[i] = hi > lo ? Math.log(l.strength[j]! / lo) / span : 1;
      i++;
    });
  });
  return { positions, brightness, starts, counts, polarity, kind: t.lines.map((l) => l.kind) };
}

/** The layer's text tier (design section 9): field at the equator, tilt, where the wind is held, how many lines are open. */
export function narrateField(f: FieldModel, bubble: Bubble | null, t?: FieldTrace): string {
  const b0 = dipoleB0(f);
  const ax = dipoleAxis(f);
  const tilt = (Math.acos(Math.min(1, Math.abs(ax[2]))) * 180) / Math.PI;
  const fmt = (x: number) => x.toLocaleString("en-GB", { maximumSignificantDigits: 2 });
  const strength = b0 >= 1e-4 ? `${fmt(b0 * 1e4)} gauss` : b0 >= 1e-6 ? `${fmt(b0 * 1e6)} µT` : `${fmt(b0 * 1e9)} nT`;
  const parts = [`Field ${strength} at the equator, tilted ${Math.round(tilt)}° from the spin axis`];
  if (f.offset && norm(f.offset) > 0.01) parts.push(`its centre ${fmt(norm(f.offset))} radii off the body's`);
  if (bubble) parts.push(`holds the wind off at ${fmt(bubble.standoff_radii)} radii`);
  else parts.push("no wind bubble given, so the lines are drawn to their own reach");
  let s = parts.join("; ") + ".";
  if (t) s += ` ${t.counts.closed} of the lines drawn loop back to the body and ${t.counts.open} open into the wind.`;
  return s;
}

export const FIELD_REFS = [
  "Gauss coefficients, Schmidt semi-normalised: Winch, D. E. et al. 2005, Geomagnetic field analysis using spherical harmonic expansions, Geophys. J. Int. 160, 487, doi:10.1111/j.1365-246X.2004.02472.x",
  "The bubble's surface: Shue, J.-H. et al. 1998, Magnetopause location under extreme solar wind conditions, J. Geophys. Res. 103, 17691, doi:10.1029/98JA01103",
  "The wind's squeeze as an image dipole: Chapman, S. and Ferraro, V. C. A. 1931, A new theory of magnetic storms, Terr. Magn. Atmos. Electr. 36, 77, doi:10.1029/TE036i002p00077; Mead, G. D. 1964, Deformation of the geomagnetic field by the solar wind, J. Geophys. Res. 69, 1181, doi:10.1029/JZ069i007p01181",
  "The open magnetosphere: Dungey, J. W. 1961, Interplanetary magnetic field and the auroral zones, Phys. Rev. Lett. 6, 47, doi:10.1103/PhysRevLett.6.47; as a vacuum superposition: Cowley, S. W. H. 1973, A qualitative study of the reconnection between the Earth's magnetic field and an interplanetary field of arbitrary orientation, Radio Sci. 8, 903, doi:10.1029/RS008i011p00903",
  "Line tracing by fourth-order Runge–Kutta along the field's direction: Press, W. H. et al. 2007, Numerical Recipes, 3rd ed., section 17.1",
];

/** A view of the 3D layer: orthographic, looking at the body from (lon, lat) in degrees, spin axis up; `scale` px per radius. */
export interface FieldCamera {
  lon_deg: number;
  lat_deg: number;
  scale: number;
  cx: number;
  cy: number;
}

/** One visible run of a line in the picture: points in px with brightness 0..1. */
export interface Run {
  line: number;
  /** +1: the field points away from the body along this run; −1: toward it. */
  polarity: 1 | -1;
  kind: FieldLine["kind"];
  xy: number[];
  b: number[];
  /** How far along the whole line each point is, 0 at the footpoint to 1 at its end (open lines fade by it). */
  along: number[];
}

/**
 * Project the trace through the camera and cut each line where it passes behind the body (depth-tested against
 * the unit sphere), so a line never draws across the globe it wraps round.
 */
export function project3D(t: FieldTrace, cam: FieldCamera): Run[] {
  const lo = (cam.lon_deg * Math.PI) / 180;
  const la = (cam.lat_deg * Math.PI) / 180;
  // Toward the viewer, then right and up on the picture (up keeps the spin axis's sense).
  const v: V3 = [Math.cos(la) * Math.cos(lo), Math.cos(la) * Math.sin(lo), Math.sin(la)];
  const right: V3 = [-Math.sin(lo), Math.cos(lo), 0];
  const up: V3 = [v[1] * right[2] - v[2] * right[1], v[2] * right[0] - v[0] * right[2], v[0] * right[1] - v[1] * right[0]];
  const [blo, bhi] = t.range;
  const span = bhi > blo ? Math.log(bhi / blo) : 1;
  const runs: Run[] = [];
  t.lines.forEach((l, k) => {
    let cur: Run | null = null;
    l.points.forEach((p, j) => {
      const x = dot(p, right);
      const y = dot(p, up);
      const d = dot(p, v);
      const rho2 = x * x + y * y;
      const seen = rho2 >= 1 || d >= Math.sqrt(1 - rho2) - 1e-9;
      if (!seen) {
        cur = null;
        return;
      }
      const pol = l.outward[j] ?? l.polarity;
      if (cur && cur.polarity !== pol) {
        // Carry the join so the two inks meet without a gap.
        const x0 = cur.xy[cur.xy.length - 2]!;
        const y0 = cur.xy[cur.xy.length - 1]!;
        const b0 = cur.b[cur.b.length - 1]!;
        const a0 = cur.along[cur.along.length - 1]!;
        cur = { line: k, polarity: pol, kind: l.kind, xy: [x0, y0], b: [b0], along: [a0] };
        runs.push(cur);
      }
      if (!cur) {
        cur = { line: k, polarity: pol, kind: l.kind, xy: [], b: [], along: [] };
        runs.push(cur);
      }
      cur.xy.push(cam.cx + x * cam.scale, cam.cy - y * cam.scale);
      cur.b.push(bhi > blo ? Math.log(l.strength[j]! / blo) / span : 1);
      cur.along.push(j / Math.max(1, l.points.length - 1));
    });
  });
  return runs.filter((r) => r.xy.length >= 4);
}

/** The few drawing calls the 3D layer needs (a CanvasRenderingContext2D has them all). */
export interface Pen {
  beginPath(): void;
  moveTo(x: number, y: number): void;
  lineTo(x: number, y: number): void;
  stroke(): void;
  arc(x: number, y: number, r: number, a0: number, a1: number): void;
  strokeStyle: string | unknown;
  globalAlpha: number;
  lineWidth: number;
}

/**
 * Draw the 3D layer (3D and Wireframe styles): the body as an outline, each line in its polarity's ink, brighter
 * and heavier where the field is stronger; open lines fade toward their far end. Inks come from the view's theme.
 */
export function drawField3D(pen: Pen, runs: Run[], cam: FieldCamera, ink: { out: string; in: string; body: string }, o: { minAlpha?: number } = {}): void {
  const minA = o.minAlpha ?? 0.25;
  pen.globalAlpha = 1;
  pen.lineWidth = 1;
  pen.strokeStyle = ink.body;
  pen.beginPath();
  pen.arc(cam.cx, cam.cy, cam.scale, 0, 2 * Math.PI);
  pen.stroke();
  for (const r of runs) {
    pen.strokeStyle = r.polarity > 0 ? ink.out : ink.in;
    const n = r.b.length;
    // Short segments in a few brightness steps: enough for the eye, few calls.
    const STEPS = 6;
    let i = 0;
    while (i < n - 1) {
      const fade = r.kind === "open" ? 1 - 0.7 * r.along[i]! : 1;
      const level = Math.min(STEPS, Math.max(1, Math.round(r.b[i]! * STEPS)));
      pen.globalAlpha = (minA + ((1 - minA) * level) / STEPS) * fade;
      pen.lineWidth = 0.6 + (0.9 * level) / STEPS;
      pen.beginPath();
      pen.moveTo(r.xy[2 * i]!, r.xy[2 * i + 1]!);
      let j = i + 1;
      for (; j < n; j++) {
        pen.lineTo(r.xy[2 * j]!, r.xy[2 * j + 1]!);
        const lj = Math.min(STEPS, Math.max(1, Math.round(r.b[j]! * STEPS)));
        if (lj !== level || (r.kind === "open" && j - i > 8)) break;
      }
      pen.stroke();
      i = j;
    }
  }
  pen.globalAlpha = 1;
}

/**
 * The Chart cut as path data for an SVG of `size` px across, showing `extent` radii from the centre to each side
 * (x sunward to the left, the way the wind blows across the page, y up): the inked lines, the bubble's edge and both axes.
 */
export function cutPaths(c: ChartCut, size: number, extent: number): {
  body: { cx: number; cy: number; r: number };
  lines: { d: string; kind: FieldLine["kind"]; polarity: 1 | -1; width: number }[];
  bubble: string | null;
  magneticAxis: [number, number, number, number];
  spinAxis: [number, number, number, number];
  tilt: { x: number; y: number; text: string };
} {
  const s = size / (2 * extent);
  const X = (x: number) => size / 2 - x * s;
  const Y = (y: number) => size / 2 - y * s;
  const path = (pts: { x: number; y: number }[]) => {
    let d = "";
    let pen = false;
    for (const p of pts) {
      const inside = Math.abs(p.x) <= extent * 1.05 && Math.abs(p.y) <= extent * 1.05;
      if (!inside) {
        pen = false;
        continue;
      }
      d += `${pen ? "L" : "M"}${X(p.x).toFixed(1)} ${Y(p.y).toFixed(1)}`;
      pen = true;
    }
    return d;
  };
  const axis = (a: { x: number; y: number }, len: number): [number, number, number, number] => [X(-a.x * len), Y(-a.y * len), X(a.x * len), Y(a.y * len)];
  const len = Math.max(1.6, extent * 0.3);
  return {
    body: { cx: size / 2, cy: size / 2, r: s },
    lines: c.lines.map((l) => ({ d: path(l.points), kind: l.kind, polarity: l.polarity, width: l.weight })),
    bubble: c.bubble ? path(c.bubble) : null,
    magneticAxis: axis(c.magneticAxis, len),
    spinAxis: axis(c.spinAxis, len),
    tilt: { x: X(c.spinAxis.x * len) + 6, y: Y(c.spinAxis.y * len) + 4, text: `${Math.round(c.tilt_deg)}°` },
  };
}

/**
 * Lines per device tier (seeds per magnetic hemisphere × meridians), measured in MF14a part 1: on a desktop the
 * default 16 × 6 traces in about 8 ms; on the Moto G4 emulation (4× CPU slowdown) in about 35 ms, so lower tiers
 * trace fewer lines. A trace runs once per change of field or wind, not per frame. The text tier draws none.
 */
export const FIELD_LINES: Record<"gpu-high" | "gpu-basic" | "canvas" | "text", { perHemisphere: number; meridians: number } | null> = {
  "gpu-high": { perHemisphere: 16, meridians: 6 },
  "gpu-basic": { perHemisphere: 12, meridians: 6 },
  canvas: { perHemisphere: 10, meridians: 4 },
  text: null,
};

/** The parts of a read_field body (fixture "solar-system", or MF2's facts later) the layer reads. */
export interface FieldReadJson {
  address: string;
  name?: string;
  obstacle: { kind: string; standoff_radii: number };
  flaring?: number;
  field?: { reference_radius_m: number; g10_t: number; g11_t: number; h11_t: number; offset_radii?: number; tilt_deg?: number; grade?: string; stand_ins?: string[] };
  wind_field?: { radial_t: number; azimuthal_t: number; sector?: string };
}

/**
 * A read_field body to the layer's model and bubble. The body frame has z along the spin axis and x toward the
 * star (the wind blows along −x); the ecliptic is taken as the spin equator. The wind's field is Parker's spiral:
 * radial along −x (away from the star) for an away sector, and azimuthal along the star's spin, which is the
 * orbit's prograde sense, −y in this frame (z × −x).
 * Null when the body has no field block (no facts yet) or its own field is not what holds the wind off.
 */
export function fieldFromRead(r: FieldReadJson): { model: FieldModel; bubble: Bubble | null; notes: string[] } | null {
  const f = r.field;
  if (!f) return null;
  const g = [[], [f.g10_t, f.g11_t]];
  const h = [[], [0, f.h11_t]];
  const base: FieldModel = { radius_m: f.reference_radius_m, g, h };
  const notes = [...(f.stand_ins ?? [])];
  let model = base;
  if (f.offset_radii && f.offset_radii > 0) {
    // The offset's direction is not published yet: drawn along the magnetic axis, toward the pole the field leaves.
    const ax = dipoleAxis(base);
    model = { ...base, offset: [-ax[0] * f.offset_radii, -ax[1] * f.offset_radii, -ax[2] * f.offset_radii] };
    notes.push("the centre's offset is drawn along the magnetic axis, toward the pole the field leaves; its real direction is not published yet");
  }
  const own = r.obstacle.kind === "field";
  const w = r.wind_field;
  const bubble: Bubble | null = own
    ? {
        standoff_radii: r.obstacle.standoff_radii,
        flaring: r.flaring ?? 0.5,
        sunward: [1, 0, 0],
        ...(w ? { wind_field_t: [-w.radial_t, -w.azimuthal_t, 0] as V3 } : {}),
      }
    : null;
  if (!own) notes.push(`its ${r.obstacle.kind}, not its field, holds the wind off, so the lines are drawn to their own reach`);
  return { model, bubble, notes };
}
