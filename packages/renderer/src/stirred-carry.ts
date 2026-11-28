// Carrying a giant's cloud back along its own winds, for the picture (track WS, package WS5; design
// handover/design/SSE4-stirred-clouds-design.md section 3.6).
//
// The kernel's `want: "carry"` gives everything that moves the cloud over the two renewals a picture reaches back:
// the bands' stream function and wind by latitude, the pole's polygon, every storm and every eddy. This file sums
// those winds at a place and follows them backwards, as the kernel's reference (crates/sse-weather/src/stirred.rs)
// does. The kernel's own answer for a small patch (`want: "stirred"`) holds it by test (test/stirred-carry.test.ts,
// on tests/fixtures/weather/carry.json). Nothing here is physics of its own: it is the sum of the kernel's vortices
// and jets at a point, the one sum the design lets the renderer hold, and it is in the fudge register as the place
// where the two could drift.
//
// The GPU pass (stirred-gpu.ts) runs the same steps; this one is its reference and the canvas tier's.

import { constant } from "../../ui-contract/src/index.ts";

/** One storm as the carrying holds it (kernel `CarryStorm`). */
export interface CarryStorm {
  lat_deg: number;
  lon_deg: number;
  size_km: number;
  aspect: number;
  strength: number;
  sense: number;
  born: number;
  ends: number;
  day_ref: number;
  drift_deg_d: number;
}

/** One eddy (kernel `Eddy`). */
export interface CarryEddy {
  lat_deg: number;
  lon_deg: number;
  size_km: number;
  strength: number;
  sense: number;
  born: number;
  ends: number;
  drift_m_s: number;
}

/** The weather op's `want: "carry"` (kernel `CarryTable`). */
export interface CarryTable {
  day: number;
  day0: number;
  /** The span's end (its bins run to here). */
  day1: number;
  radius_m: number;
  turn: number;
  jets: number;
  renewal_days: number;
  generation_offset_days: number[];
  dt_s: number;
  dt_poly_s: number;
  poly_lat_deg: number;
  psi_step_deg: number;
  psi_north: number[];
  psi_south_table: number[];
  psi_south: number;
  u_step_deg: number;
  u_m_s: number[];
  polar_jet_lat_deg: number;
  jet_deformation_km: number;
  poles: { north: boolean; polygon_sides: number; orient_lon_deg: number }[];
  storms: CarryStorm[];
  eddies: CarryEddy[];
}

type V3 = [number, number, number];

const DEG = 180 / Math.PI;
const DAY_S = constant("day");
/** How far a vortex's winds reach, in its radii (kernel `REACH_R`). */
export const REACH_R = 3;
/** The eddies are filed in bins of this many days (kernel `BIN_D`). */
const BIN_D = 0.5;

const clamp = (x: number, a: number, b: number) => (x < a ? a : x > b ? b : x);
export const wrap = (lon: number) => {
  const x = (lon + 180) % 360;
  return (x < 0 ? x + 360 : x) - 180;
};
export const unit = (lat: number, lon: number): V3 => {
  const p = lat / DEG;
  const l = lon / DEG;
  return [Math.cos(p) * Math.cos(l), Math.cos(p) * Math.sin(l), Math.sin(p)];
};
export const latLon = (p: V3): [number, number] => [Math.asin(clamp(p[2], -1, 1)) * DEG, Math.atan2(p[1], p[0]) * DEG];
const dot = (a: V3, b: V3) => a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
const norm = (p: V3): V3 => {
  const r = Math.sqrt(dot(p, p));
  return [p[0] / r, p[1] / r, p[2] / r];
};
const axpy = (p: V3, h: number, v: V3): V3 => [p[0] + h * v[0], p[1] + h * v[1], p[2] + h * v[2]];
/** East and north at a place. */
export const frame = (lat: number, lon: number): [V3, V3] => {
  const p = lat / DEG;
  const l = lon / DEG;
  return [
    [-Math.sin(l), Math.cos(l), 0],
    [-Math.sin(p) * Math.cos(l), -Math.sin(p) * Math.sin(l), Math.cos(p)],
  ];
};

/** The quintic smoothstep's taper of a vortex between 2.5 and 3 radii, and its slope in radii (kernel `taper`). */
function taper(x: number): [number, number] {
  const t = clamp((REACH_R - x) / 0.5, 0, 1);
  const w = t * t * t * (10 + t * (6 * t - 15));
  const dwdt = t > 0 && t < 1 ? 30 * t * t * (1 - t) * (1 - t) : 0;
  return [w, -dwdt / 0.5];
}

/** How much of the background reaches a place r storm radii out (kernel `shield`). */
function shield(r: number): number {
  const t = clamp((r - 1) / 0.5, 0, 1);
  return t * t * t * (10 + t * (6 * t - 15));
}

/** A place in a storm's own frame, km east and north along the great circle (kernel `storm_xy`). */
function stormXy(a: number, lat: number, lon: number, lat_s: number, lon_s: number): [number, number] {
  const p = unit(lat, lon);
  const c = unit(lat_s, lon_s);
  const [e, n] = frame(lat_s, lon_s);
  const pe = dot(p, e);
  const pn = dot(p, n);
  const len = Math.sqrt(pe * pe + pn * pn);
  if (len < 1e-15) return [0, 0];
  const d = (Math.acos(clamp(dot(p, c), -1, 1)) * a) / 1000;
  return [(d * pe) / len, (d * pn) / len];
}

interface Vortex {
  lat_deg: number;
  lon_deg: number;
  size_km: number;
  aspect: number;
  strength: number;
}

/** A storm's distance in its own radii, in its ellipse (kernel `storm_r_at`). */
function stormR(a: number, s: Vortex, lat: number, lon: number): number {
  const rr = 0.5 * Math.max(s.size_km, 1);
  const k = s.aspect > 0 ? Math.sqrt(s.aspect) : 1;
  const [x, y] = stormXy(a, lat, lon, s.lat_deg, s.lon_deg);
  return Math.sqrt((x / k) * (x / k) + y * k * (y * k)) / rr;
}

/** A storm's own winds, tapered by three radii, m/s [east, north] (kernel `tapered_storm_wind`). */
function stormWind(a: number, s: Vortex, turn: number, sense: number, lat: number, lon: number): [number, number] | null {
  const rr = 0.5 * Math.max(s.size_km, 1);
  if (stormR(a, s, lat, lon) > REACH_R + 0.01) return null;
  const h = s.lat_deg >= 0 ? 1 : -1;
  const q = sense * h * turn * s.strength * rr * 1000;
  const phi = (la: number, lo: number) => {
    const x = stormR(a, s, la, lo);
    return -q * Math.exp(0.5 * (1 - x * x)) * taper(x)[0];
  };
  const dd = ((1e-5 * rr * 1000) / a) * DEG;
  const cp = Math.cos(lat / DEG);
  if (cp < 1e-6) return null;
  const dphi = (phi(lat + dd, lon) - phi(lat - dd, lon)) / ((2 * dd) / DEG);
  const dlam = (phi(lat, lon + dd / cp) - phi(lat, lon - dd / cp)) / ((2 * dd) / cp / DEG);
  return [-dphi / a, dlam / (a * cp)];
}

/** The pole's polygon edge along a longitude and its slope, km and km a radian (kernel `polygon_edge_km`). */
function polygonAt(t: CarryTable, north: boolean, rJet: number, lon: number): [number, number] {
  const q = t.poles.find((p) => p.north === north);
  if (!q || q.polygon_sides <= 0) return [rJet, 0];
  const n = q.polygon_sides;
  const seg = (2 * Math.PI) / n;
  const tt = (((((lon - q.orient_lon_deg) / DEG) % (2 * Math.PI)) + 2 * Math.PI) % (2 * Math.PI)) % seg;
  const edge = (rJet * Math.cos(Math.PI / n)) / Math.cos(tt - Math.PI / n);
  const ts = tt - Math.PI / n;
  const c = Math.cos(ts);
  return [edge, (rJet * Math.cos(Math.PI / n) * Math.sin(ts)) / (c * c)];
}

/** The eastward wind at a latitude, m/s, from the kernel's table. */
export function zonalWind(t: CarryTable, lat: number): number {
  const x = clamp((lat + 90) / t.u_step_deg, 0, t.u_m_s.length - 1);
  const k = Math.min(Math.floor(x), t.u_m_s.length - 2);
  const f = x - k;
  return t.u_m_s[k]! + (t.u_m_s[k + 1]! - t.u_m_s[k]!) * f;
}

/** The bands' and polar jet's wind, m/s [east, north] (kernel `band_wind_at` through `banded_wind`). */
function bandWind(t: CarryTable, a: number, lat: number, lon: number): [number, number] {
  const rJet = ((90 - t.polar_jet_lat_deg) / DEG) * a / 1000;
  const r = ((90 - Math.abs(lat)) / DEG) * a / 1000;
  const [edge, slope] = polygonAt(t, lat >= 0, rJet, lon);
  if (slope === 0 && edge === rJet) return [zonalWind(t, lat), 0];
  const h = lat >= 0 ? 1 : -1;
  const aKm = a / 1000;
  const ln = 2 * Math.max(t.jet_deformation_km, 1);
  let tt: number, tr: number, te: number;
  if (r < edge) {
    const x = r / edge;
    const q = 1 - x * x;
    tr = (4 * x * q) / edge;
    tt = 1 - q * q;
    te = -x * tr;
  } else {
    const z = (r - edge) / ln;
    tt = Math.exp(-z * z);
    tr = ((-2 * z) / ln) * tt;
    te = -tr;
  }
  const off = rJet - edge;
  const rho = r + off * tt;
  const rhoR = 1 + off * tr;
  const rhoL = -slope * tt + off * te * slope;
  const latRho = h * (90 - clamp((rho / aKm) * DEG, 0, 180));
  const u = zonalWind(t, latRho);
  return [u * rhoR, (h * u * rhoL) / (aKm * Math.max(Math.cos(lat / DEG), 1e-9))];
}

/** Where a place sits in the bands' stream function: hemisphere and ρ, km (kernel `band_rho_km`). */
function bandRhoKm(t: CarryTable, a: number, lat: number, lon: number): [number, number] {
  const h = lat >= 0 ? 1 : -1;
  const rJet = ((90 - t.polar_jet_lat_deg) / DEG) * a / 1000;
  const r = ((90 - Math.abs(lat)) / DEG) * a / 1000;
  const [edge, slope] = polygonAt(t, lat >= 0, rJet, lon);
  if (slope === 0 && edge === rJet) return [h, r];
  const ln = 2 * Math.max(t.jet_deformation_km, 1);
  let tt: number;
  if (r < edge) {
    const x = r / edge;
    const q = 1 - x * x;
    tt = 1 - q * q;
  } else {
    const z = (r - edge) / ln;
    tt = Math.exp(-z * z);
  }
  return [h, r + (rJet - edge) * tt];
}

/** How firmly a pole's polygon holds cloud at a place: 1 inside, a Gaussian outside (kernel `polygon_hold`). */
export function polygonHold(t: CarryTable, lat: number, lon: number): number {
  const north = lat >= 0;
  const q = t.poles.find((p) => p.north === north);
  if (!q || q.polygon_sides <= 0) return 0;
  const a = t.radius_m;
  const rJet = ((90 - t.polar_jet_lat_deg) / DEG) * a / 1000;
  const r = ((90 - Math.abs(lat)) / DEG) * a / 1000;
  const [edge] = polygonAt(t, north, rJet, lon);
  if (r < edge) return 1;
  const z = (r - edge) / (2 * Math.max(t.jet_deformation_km, 1));
  return Math.exp(-z * z);
}

/** The jets' count, a band's index as the kernel files them (kernel `band_of`). */
export function bandOf(t: CarryTable, lat: number): number {
  const j = Math.max(t.jets, 1);
  const k = Math.min(Math.floor(Math.sin(Math.abs(lat) / DEG) * j), j - 1);
  return lat >= 0 ? k : j + k;
}

/** The two generations alive at a place on the table's date, [age, weight], the younger first (kernel `generations`). */
export function generations(t: CarryTable, lat: number, day = t.day): [[number, number], [number, number]] {
  const tau = t.renewal_days;
  const off = t.generation_offset_days[bandOf(t, lat)] ?? 0;
  const x = (day - off) % tau;
  const a1 = x < 0 ? x + tau : x;
  return [
    [a1, a1 / tau],
    [a1 + tau, 1 - a1 / tau],
  ];
}

/** An eddy's centre longitude and strength on a day (kernel `Eddy::lon_at`, `strength_at`). */
function eddyLon(e: CarryEddy, a: number, day: number): number {
  return wrap(e.lon_deg + ((e.drift_m_s * (day - e.born) * DAY_S) / (a * Math.max(Math.cos(e.lat_deg / DEG), 1e-6))) * DEG);
}
function eddyStrength(e: CarryEddy, day: number): number {
  if (day < e.born || day >= e.ends || e.ends <= e.born) return 0;
  const x = Math.sin((Math.PI * (day - e.born)) / (e.ends - e.born));
  return e.strength * x * x;
}

function stormAt(s: CarryStorm, day: number): Vortex | null {
  if (!(day >= s.born && day < s.ends)) return null;
  return { lat_deg: s.lat_deg, lon_deg: wrap(s.lon_deg + s.drift_deg_d * (day - s.day_ref)), size_km: s.size_km, aspect: s.aspect, strength: s.strength };
}

/** A path being carried back, with the Jacobian of the map from where it started, 2^log2 × j (kernel `Path`). */
export interface Path {
  p: V3;
  j: [[number, number], [number, number]];
  log2: number;
  area_log2: number;
}

const STEP_STRAIN = 0.05;
const MAX_HALVINGS = 8;
const RESCALE = 64;
const NUDGE_M = 1;

/** One generation at a place, as the kernel's `Generation`. */
export interface Generation {
  age_days: number;
  weight: number;
  lat_deg: number;
  lon_deg: number;
  stretch_log2: number;
  area: number;
}

/** The winds of a carry table, filed for quick sums: the same sums as the kernel's `Carrier`. */
export class Carrier {
  readonly t: CarryTable;
  readonly a: number;
  /** Per bin of BIN_D days: the eddies alive in it. */
  private bins: number[][];
  constructor(t: CarryTable) {
    this.t = t;
    this.a = t.radius_m;
    const n = Math.max(1, Math.round((t.day1 - t.day0) / BIN_D));
    this.bins = Array.from({ length: n }, () => []);
    t.eddies.forEach((e, i) => {
      const lo = Math.max(0, Math.floor((e.born - t.day0) / BIN_D));
      const hi = Math.min(n - 1, Math.max(0, Math.floor((e.ends - t.day0) / BIN_D)));
      for (let b = lo; b <= hi; b++) this.bins[b]!.push(i);
    });
  }

  /** The bands' stream function at a place, m²/s (kernel `band_psi`). */
  bandPsi(lat: number, lon: number): number {
    const t = this.t;
    const [h, rhoKm] = bandRhoKm(t, this.a, lat, lon);
    const tab = h > 0 ? t.psi_north : t.psi_south_table;
    const x = clamp((((rhoKm * 1000) / this.a) * DEG) / t.psi_step_deg, 0, tab.length - 1);
    const k = Math.min(Math.floor(x), tab.length - 2);
    const f = x - k;
    const psi = tab[k]! + (tab[k + 1]! - tab[k]!) * f;
    return h > 0 ? psi : t.psi_south - psi;
  }

  /** The eddies' winds along the surface and their stream function at a place (kernel `eddy_sum`). */
  eddySum(p: V3, day: number): [V3, number] {
    const a = this.a;
    const t = this.t;
    const [lat, lon] = latLon(p);
    const [e, n] = frame(lat, lon);
    const ve: V3 = [0, 0, 0];
    let psi = 0;
    const raw = Math.floor((day - t.day0) / BIN_D);
    if (raw < 0 || raw > this.bins.length) return [ve, psi];
    // The span's last instant falls in its last bin (as the kernel files it).
    const bin = Math.min(raw, this.bins.length - 1);
    for (const i of this.bins[bin]!) {
      const ed = t.eddies[i]!;
      const s = eddyStrength(ed, day);
      if (s <= 0) continue;
      const r = 0.5 * ed.size_km * 1000;
      // A quick reject by latitude before the great circle.
      if ((Math.abs(ed.lat_deg - lat) / DEG) * a > REACH_R * r) continue;
      const c = unit(ed.lat_deg, eddyLon(ed, a, day));
      const sigma = Math.acos(clamp(dot(p, c), -1, 1));
      const d = sigma * a;
      const x2 = (d / r) * (d / r);
      if (x2 > REACH_R * REACH_R) continue;
      const hh = ed.lat_deg >= 0 ? 1 : -1;
      const q = ed.sense * hh * t.turn * s * r;
      const ex = Math.exp(0.5 * (1 - x2));
      const x = d / r;
      const [w, dw] = taper(x);
      psi += -q * ex * w;
      const sn = Math.sin(sigma);
      if (sn < 1e-12) continue;
      const dphi = ((q * ex) / r) * (x * w - dw);
      const u = (dphi * dot(n, c)) / sn;
      const v = (-dphi * dot(e, c)) / sn;
      for (let k = 0; k < 3; k++) ve[k]! += u * e[k]! + v * n[k]!;
    }
    return [ve, psi];
  }

  /** The wind at a place on a day, m/s along the surface (kernel `Carrier::wind`). */
  wind(p: V3, day: number): V3 {
    const a = this.a;
    const t = this.t;
    const [lat, lon] = latLon(p);
    const [e, n] = frame(lat, lon);
    const to3 = (w: [number, number]): V3 => [w[0] * e[0] + w[1] * n[0], w[0] * e[1] + w[1] * n[1], w[0] * e[2] + w[1] * n[2]];
    const own: [number, number] = [0, 0];
    const live: { v: Vortex; s: CarryStorm }[] = [];
    for (const s of t.storms) {
      const v = stormAt(s, day);
      if (!v) continue;
      live.push({ v, s });
      const sw = stormWind(a, v, t.turn, s.sense, lat, lon);
      if (sw) {
        own[0] += sw[0];
        own[1] += sw[1];
      }
    }
    const out = to3(own);
    const [ve, psiE] = this.eddySum(p, day);
    const vb = to3(bandWind(t, a, lat, lon));
    const near = live
      .filter(({ v }) => stormR(a, v, lat, lon) < 1.6)
      .map(({ v, s }) => ({ v, psiC: this.bandPsi(v.lat_deg, v.lon_deg) + this.eddySum(unit(v.lat_deg, v.lon_deg), day)[1], om: s.drift_deg_d / DEG / DAY_S }));
    if (!near.length) {
      for (let i = 0; i < 3; i++) out[i]! += vb[i]! + ve[i]!;
      return out;
    }
    const psi = this.bandPsi(lat, lon) + psiE;
    const gh = (la: number, lo: number): [number, number] => {
      let g = 1;
      let wsum = 0;
      let wpsi = 0;
      const sl = Math.sin(la / DEG);
      for (const { v, psiC, om } of near) {
        const x = 1 - shield(stormR(a, v, la, lo));
        g *= 1 - x;
        wsum += x;
        wpsi += x * (psiC - om * a * a * (sl - Math.sin(v.lat_deg / DEG)));
      }
      const ref = wsum > 0 ? wpsi / wsum : 0;
      return [g, ref * (1 - g)];
    };
    const dd = 1e-4;
    const [g0] = gh(lat, lon);
    const [gn, hn] = gh(lat + dd, lon);
    const [gs, hs] = gh(lat - dd, lon);
    const [ge, he] = gh(lat, lon + dd);
    const [gw, hw] = gh(lat, lon - dd);
    const step = (2 * dd) / DEG;
    const dPhi = (psi * (gn - gs) + (hn - hs)) / step;
    const dLam = (psi * (ge - gw) + (he - hw)) / step;
    const cosl = Math.max(Math.cos(lat / DEG), 1e-6);
    const ux = -dPhi / a;
    const vx = dLam / (a * cosl);
    for (let i = 0; i < 3; i++) out[i]! += g0 * (vb[i]! + ve[i]!) + ux * e[i]! + vx * n[i]!;
    return out;
  }

  /** One fourth-order Runge-Kutta step of h seconds (backwards when negative) from p on day. */
  step(p: V3, day: number, h: number): V3 {
    const a = this.a;
    const hd = h / DAY_S;
    const k1 = this.wind(p, day);
    const k2 = this.wind(norm(axpy(p, (0.5 * h) / a, k1)), day + 0.5 * hd);
    const k3 = this.wind(norm(axpy(p, (0.5 * h) / a, k2)), day + 0.5 * hd);
    const k4 = this.wind(norm(axpy(p, h / a, k3)), day + hd);
    const v: V3 = [0, 1, 2].map((i) => (k1[i]! + 2 * k2[i]! + 2 * k3[i]! + k4[i]!) / 6) as V3;
    return norm(axpy(p, h / a, v));
  }

  /** The step for a place, s: the table's, or the polygon's near a pole with one. */
  dtAt(p: V3): number {
    return Math.abs(latLon(p)[0]) >= this.t.poly_lat_deg ? this.t.dt_poly_s : this.t.dt_s;
  }

  /**
   * Follow the wind back from p on day for days in fixed steps (the picture's way, as the GPU does): no Jacobian,
   * no halving. The stretch for the picture comes from neighbouring pixels' paths instead.
   */
  back(p: V3, day: number, days: number, dt = this.dtAt(p)): V3 {
    if (days <= 0) return p;
    const n = Math.max(1, Math.ceil((days * DAY_S) / dt));
    const h = (days * DAY_S) / n;
    let q = p;
    for (let k = 0; k < n; k++) q = this.step(q, day - (k * h) / DAY_S, -h);
    return q;
  }

  /** As the kernel's `back_path`: halving steps in strong strain and carrying the Jacobian (the reference's way). */
  backPath(path: Path, day: number, days: number): Path {
    if (days <= 0) return path;
    const dt = this.dtAt(path.p);
    const n = Math.max(1, Math.ceil((days * DAY_S) / dt));
    const h = (days * DAY_S) / n;
    for (let k = 0; k < n; k++) this.advance(path, day - (k * h) / DAY_S, h, 0);
    return path;
  }

  private stepWithJacobian(p: V3, t: number, h: number): [V3, [number, number], [number, number]] {
    const a = this.a;
    const d = NUDGE_M / a;
    const [la, lo] = latLon(p);
    const [e, nn] = frame(la, lo);
    const p1 = this.step(p, t, -h);
    const pe = this.step(norm(axpy(p, d, e)), t, -h);
    const pn = this.step(norm(axpy(p, d, nn)), t, -h);
    const tg = (q: V3) => tangent(p1, q, a).map((v) => v / NUDGE_M) as [number, number];
    return [p1, tg(pe), tg(pn)];
  }

  private advance(path: Path, t: number, h: number, depth: number): void {
    const [p1, ce, cn] = this.stepWithJacobian(path.p, t, h);
    const strain = Math.max(Math.abs(ce[0] - 1), Math.abs(ce[1]), Math.abs(cn[0]), Math.abs(cn[1] - 1));
    if (strain > STEP_STRAIN && depth < MAX_HALVINGS) {
      this.advance(path, t, 0.5 * h, depth + 1);
      this.advance(path, t - (0.5 * h) / DAY_S, 0.5 * h, depth + 1);
      return;
    }
    path.area_log2 += Math.log2(Math.abs(ce[0] * cn[1] - cn[0] * ce[1]));
    const j = path.j;
    path.j = [
      [ce[0] * j[0][0] + cn[0] * j[1][0], ce[0] * j[0][1] + cn[0] * j[1][1]],
      [ce[1] * j[0][0] + cn[1] * j[1][0], ce[1] * j[0][1] + cn[1] * j[1][1]],
    ];
    path.p = p1;
    const big = Math.max(...path.j.flat().map(Math.abs));
    if (big > 2 ** RESCALE) {
      path.j = path.j.map((r) => r.map((v) => v * 2 ** -RESCALE)) as Path["j"];
      path.log2 += RESCALE;
    }
  }

  /**
   * Where today's face is read for cloud that was at x `age` days before `day`: within a storm's reach where the storm
   * stands now, elsewhere moved on by its row's drift, not at all where a polygon holds it (kernel `read_place`).
   */
  readPlace(x: V3, day: number, age: number): V3 {
    const a = this.a;
    const t = this.t;
    const [lat, lon] = latLon(x);
    const then = day - age;
    const row = ((zonalWind(t, lat) * age * DAY_S) / (a * Math.max(Math.cos(lat / DEG), 1e-6))) * DEG;
    const band = (1 - polygonHold(t, lat, lon)) * row;
    let g = 0;
    let moved = 0;
    for (const s of t.storms) {
      const v = stormAt(s, then);
      if (!v) continue;
      const r = stormR(a, v, lat, lon);
      if (r > REACH_R) continue;
      const gs = Math.exp(-r * r);
      if (gs > g) {
        g = gs;
        moved = s.drift_deg_d * age;
      }
    }
    return unit(lat, wrap(lon + (g > 0 ? moved : band)));
  }

  /** A place's two generations as the kernel gives them (kernel `stirred_patch`, one cell). */
  generationsAt(lat: number, lon: number): [Generation, Generation] {
    const t = this.t;
    const a = this.a;
    const gens = generations(t, lat);
    const p1 = this.backPath({ p: unit(lat, lon), j: [[1, 0], [0, 1]], log2: 0, area_log2: 0 }, t.day, gens[0][0]);
    const q1: Path = { p: p1.p, j: [p1.j[0].slice() as [number, number], p1.j[1].slice() as [number, number]], log2: p1.log2, area_log2: p1.area_log2 };
    const p2 = this.backPath(q1, t.day - gens[0][0], gens[1][0] - gens[0][0]);
    return ([[p1, gens[0]], [p2, gens[1]]] as const).map(([path, [age, weight]]) => {
      const r = this.readPlace(path.p, t.day, age);
      const [bl, bo] = latLon(path.p);
      const [e, nn] = frame(bl, bo);
      const d = NUDGE_M / a;
      const re = this.readPlace(norm(axpy(path.p, d, e)), t.day, age);
      const rn = this.readPlace(norm(axpy(path.p, d, nn)), t.day, age);
      const de = tangent(r, re, a).map((v) => v / NUDGE_M);
      const dn = tangent(r, rn, a).map((v) => v / NUDGE_M);
      const j = path.j;
      const c00 = de[0]! * j[0][0] + dn[0]! * j[1][0];
      const c01 = de[0]! * j[0][1] + dn[0]! * j[1][1];
      const c10 = de[1]! * j[0][0] + dn[1]! * j[1][0];
      const c11 = de[1]! * j[0][1] + dn[1]! * j[1][1];
      const [sc] = stretchAndArea([c00, c10], [c01, c11]);
      const [rl, ro] = latLon(r);
      return { age_days: age, weight, lat_deg: rl, lon_deg: ro, stretch_log2: Math.log2(sc) + path.log2, area: 2 ** path.area_log2 };
    }) as [Generation, Generation];
  }
}

/** The tangent components, m, of q − p at p. */
export function tangent(p: V3, q: V3, a: number): [number, number] {
  const [lat, lon] = latLon(p);
  const [e, n] = frame(lat, lon);
  const d: V3 = [q[0] - p[0], q[1] - p[1], q[2] - p[2]];
  return [dot(d, e) * a, dot(d, n) * a];
}

/** A 2×2 map's larger singular value and determinant, from its columns. */
export function stretchAndArea(ce: [number, number], cn: [number, number]): [number, number] {
  const det = ce[0] * cn[1] - cn[0] * ce[1];
  const s = ce[0] * ce[0] + ce[1] * ce[1] + cn[0] * cn[0] + cn[1] * cn[1];
  const disc = Math.max(s * s - 4 * det * det, 0);
  return [Math.sqrt(0.5 * (s + Math.sqrt(disc))), det];
}

/** The centre of a kernel patch's cell (i, j), as `rules::patch_dir`: azimuthal equidistant round the place. */
export function patchPlace(a: number, lat: number, lon: number, half_km: number, n: number, i: number, j: number): [number, number] {
  const at = (k: number) => half_km * (-1 + (2 * k + 1) / n);
  const x = (at(i) * 1000) / a;
  const y = (at(j) * 1000) / a;
  const [e, nn] = frame(lat, lon);
  const c = unit(lat, lon);
  const r = Math.hypot(x, y);
  if (r < 1e-15) return [lat, lon];
  const dir: V3 = [0, 1, 2].map((k) => (x * e[k]! + y * nn[k]!) / r) as V3;
  const p: V3 = [0, 1, 2].map((k) => Math.cos(r) * c[k]! + Math.sin(r) * dir[k]!) as V3;
  return latLon(p);
}

/**
 * Where the picture reads today's face for a place's two generations, the GPU's way: fixed steps, back to the younger
 * generation and on from there to the older (as the kernel's patch does), each then moved to where it is read.
 * Returns [lat, lon] of each, degrees, and the generations' weights.
 */
export function sourcePlaces(c: Carrier, lat: number, lon: number): { places: [[number, number], [number, number]]; weights: [number, number]; ages: [number, number] } {
  const t = c.t;
  const g = generations(t, lat);
  const p1 = c.back(unit(lat, lon), t.day, g[0][0]);
  const p2 = c.back(p1, t.day - g[0][0], g[1][0] - g[0][0]);
  return {
    places: [latLon(c.readPlace(p1, t.day, g[0][0])), latLon(c.readPlace(p2, t.day, g[1][0]))],
    weights: [g[0][1], g[1][1]],
    ages: [g[0][0], g[1][0]],
  };
}
