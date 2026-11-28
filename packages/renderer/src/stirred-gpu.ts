// The stirred face's carrying on the GPU (track WS, package WS5; design SSE4-stirred-clouds-design.md 3.6 and 6).
//
// The same sums and steps as stirred-carry.ts `sourcePlaces`, its processor twin, run for every pixel at once: each
// fragment follows its place's cloud back along the kernel's jets, storms and eddies (the carry table) in fixed
// fourth-order Runge-Kutta steps, for the younger generation and on to the older, and writes where today's face is to
// be read for each. One shader source for WebGPU and WebGL2 (three.js TSL), drawn into a float target and read back.
//
// Float32 on the GPU, so the sums are arranged to keep large numbers apart:
// - every date is counted from the table's start (`day0`), never the campaign's;
// - the storms' barrier works on the background's stream function relative to its value here, D (1 − G), not on
//   Ψ itself (Ψ reaches 10^10 m²/s on Jupiter, beyond what a float keeps to the metre a second);
// - angles between nearby places come from the chord by a series, not acos, so near a centre they keep their digits;
// - a vortex's own winds are its stream function's slopes across a hundredth of its radius;
// - a place's frame and its neighbours a step away are taken from the place's own vector, with no angles;
// - sine, cosine and arc tangent are written out as series: a device's own may be off by 1e-3 (SwiftShader's are), which
//   carried over a thousand steps moves cloud by hundreds of kilometres.
// Each draw takes one step for every pixel, with the state kept between draws in float targets, so no draw runs long
// and no loop wraps the wind (a loop round it made SwiftShader's compiler take 13 GB).
// The test (tests/ui/stirred-gpu.e2e.mjs) holds this to stirred-carry.ts on the kernel's own table.
//
// The table is filed for the GPU in stirred-pack.ts.

import { ClampToEdgeWrapping, DataTexture, type Texture, FloatType, Mesh, NearestFilter, NoBlending, OrthographicCamera, PlaneGeometry, RedFormat, RGBAFormat, RenderTarget, RGFormat, Scene } from "three";
import { MeshBasicNodeMaterial, type WebGPURenderer } from "three/webgpu";
import { Fn, If, Loop, abs, atan, ceil, clamp, cross, dot, exp, float, floor, int, ivec2, length, max, min, normalize, screenCoordinate, select, sqrt, textureLoad, uniform, vec2, vec3, vec4 } from "three/tsl";
import { CELL_DEG, COLS, GRID_D, ROWS, SPLIT, W, mostSteps, type PackedCarry } from "./stirred-pack.ts";
import { constant } from "../../ui-contract/src/index.ts";

export { packCarry, mostSteps, type PackedCarry } from "./stirred-pack.ts";

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type N = any;

const DEG = 180 / Math.PI;
const DAY_S = constant("day");
const dataTexture = (data: Float32Array, per: 1 | 2 | 4, w = W) => {
  const tex = new DataTexture(data, w, data.length / per / w, per === 1 ? RedFormat : per === 2 ? RGFormat : RGBAFormat, FloatType);
  tex.magFilter = NearestFilter;
  tex.minFilter = NearestFilter;
  tex.wrapS = ClampToEdgeWrapping;
  tex.wrapT = ClampToEdgeWrapping;
  tex.generateMipmaps = false;
  tex.needsUpdate = true;
  return tex;
};

/**
 * The carrying as a TSL program: reads a place per texel from `places` (lat, lon in degrees, RG) and writes
 * vec4(younger lat, lon, older lat, lon) of where today's face is read.
 */
/** A material whose output is the numbers given, untouched by colour, blending or alpha. */
function raw(): MeshBasicNodeMaterial {
  const m = new MeshBasicNodeMaterial();
  m.blending = NoBlending;
  m.depthTest = false;
  m.depthWrite = false;
  m.toneMapped = false;
  return m;
}

export function carryMaterial(p: PackedCarry, places: DataTexture) {
  const t = p.table;
  const textures = {
    u: dataTexture(p.u, 1),
    psiN: dataTexture(p.psiN, 1),
    psiS: dataTexture(p.psiS, 1),
    storms: dataTexture(p.storms, 4),
    eddies: dataTexture(p.eddies, 4),
    grid: dataTexture(p.grid, 4),
    rates: dataTexture(p.rates, 1),
    list: dataTexture(p.list, 1, p.listW),
    ages: dataTexture(p.ages, 2),
  };
  // Shader functions that read the table's textures are written out where they are called, not as functions of their
  // own: on WebGL, three.js r187 names a texture inside a function of its own after another uniform, and the shader
  // fails to compile. The layouts are kept beside each for the day that is mended.
  const inline = (f: N, _layout: unknown): N => f;
  const A = float(t.radius_m);
  const TURN = float(t.turn);
  const uDay = uniform(t.day - t.day0);
  // Loop counts held as uniforms, not constants, so no compiler unrolls a loop into copies of its body.
  const uStorms = uniform(p.stormCount);
  // The table's textures are data read by index, never pictures, so they are never flipped. On WebGL three.js adds a
  // flip check per texture (a uniform it declares as it builds), and inside a shader function of its own that uniform
  // is given the texture's name (three r187): the check is left out here.
  const at = (tex: DataTexture, i: N, w = W): N => {
    const node = textureLoad(tex, ivec2(int(i).mod(int(w)), int(i).div(int(w))));
    (node as unknown as { setupUV: (b: unknown, uv: N) => N }).setupUV = (_b, uv) => uv;
    return node;
  };
  const poleN = t.poles.find((q) => q.north);
  const poleS = t.poles.find((q) => !q.north);
  const sidesN = float(poleN?.polygon_sides ?? 0);
  const sidesS = float(poleS?.polygon_sides ?? 0);
  const orientN = float(poleN?.orient_lon_deg ?? 0);
  const orientS = float(poleS?.orient_lon_deg ?? 0);
  const rJet = float((((90 - t.polar_jet_lat_deg) / DEG) * t.radius_m) / 1000);
  const LN = float(2 * Math.max(t.jet_deformation_km, 1));
  const aKm = float(t.radius_m / 1000);
  const uCount = t.u_m_s.length;
  const psiCount = t.psi_north.length;

  const unitOf = (lat: N, lon: N): N => {
    const a = sinCos(lat.div(DEG));
    const b = sinCos(lon.div(DEG));
    return vec3(a.y.mul(b.y), a.y.mul(b.x), a.x);
  };
  // Sine and cosine written out for the same reason: a quarter turn taken off (π/2 in two parts, so nothing is lost
  // taking it), then the series to the twelfth power, good to the float's grain over the quarter turn left.
  const sinCos: N = inline(Fn(([a]: [N]) => {
    const q = floor(a.mul(2 / Math.PI).add(0.5));
    const r = a.sub(q.mul(1.5707963705062866)).sub(q.mul(-4.371139000186243e-8));
    const r2 = r.mul(r);
    // Horner's rule on r²: sin r = r (1 − r²/3! + r⁴/5! − …), cos r = 1 − r²/2! + r⁴/4! − ….
    const series = (terms: number[]): N => terms.reduceRight((acc: N, c: number) => acc.mul(r2).add(c), float(0));
    const sn = r.mul(series([1, -1 / 6, 1 / 120, -1 / 5040, 1 / 362880, -1 / 39916800]));
    const cs = series([1, -1 / 2, 1 / 24, -1 / 720, 1 / 40320, -1 / 3628800, 1 / 479001600]);
    const m = q.sub(floor(q.div(4)).mul(4));
    // The quarter turns taken off: 0 (s, c), 1 (c, −s), 2 (−s, −c), 3 (−c, s).
    const odd = m.equal(1).or(m.equal(3));
    const sx = select(odd, cs, sn).mul(select(m.greaterThanEqual(2), float(-1), float(1)));
    const cx = select(odd, sn, cs).mul(select(m.equal(1).or(m.equal(2)), float(-1), float(1)));
    return vec2(sx, cx);
  }), { name: "ssSinCos", type: "vec2", inputs: [{ name: "a", type: "float" }] });
  // The arc tangent written out: some GPUs' own (SwiftShader's among them) are off by 1e-4 radians, kilometres on a
  // giant, where this is good to the float's own grain. Halved twice to under tan(π/8), then eight terms of the series.
  const atan2: N = inline(Fn(([y, x]: [N, N]) => {
    const ax = abs(x);
    const ay = abs(y);
    const big = max(ax, ay);
    const tt = min(ax, ay).div(max(big, 1e-30)).toVar();
    const far = tt.greaterThan(Math.SQRT2 - 1);
    tt.assign(select(far, tt.sub(1).div(tt.add(1)), tt));
    const t2 = tt.mul(tt);
    let series: N = float(-1 / 15);
    for (const k of [13, 11, 9, 7, 5, 3]) series = series.mul(t2).add((k % 4 === 1 ? 1 : -1) / k);
    series = series.mul(t2).add(1).mul(tt);
    const r = series.add(select(far, float(Math.PI / 4), float(0))).toVar();
    r.assign(select(ay.greaterThan(ax), float(Math.PI / 2).sub(r), r));
    r.assign(select(x.lessThan(0), float(Math.PI).sub(r), r));
    return select(y.lessThan(0), r.negate(), r);
  }), { name: "ssAtan2", type: "float", inputs: [{ name: "y", type: "float" }, { name: "x", type: "float" }] });
  const latOf = (q: N): N => atan2(q.z, length(q.xy)).mul(DEG);
  const lonOf = (q: N): N => atan2(q.y, q.x).mul(DEG);
  // A place's frame from the place itself, with no angles taken: east along the row, north up the meridian (at a pole,
  // east is taken as it is at longitude 0, as the processor's frame takes it).
  const eastOf = (q: N): N => {
    const l = length(q.xy);
    return select(l.greaterThan(1e-12), vec3(q.y.negate(), q.x, 0).div(max(l, 1e-12)), vec3(0, 1, 0));
  };
  const northOf = (q: N, e: N): N => cross(q, e);
  /** Sine and cosine of a small angle (under a tenth of a radian here), by their series. */
  const small = (d: N): N => {
    const d2 = d.mul(d);
    return vec2(d.mul(float(1).sub(d2.div(6)).add(d2.mul(d2).div(120))), float(1).sub(d2.div(2)).add(d2.mul(d2).div(24)).sub(d2.mul(d2).mul(d2).div(720)));
  };
  /** The place `d` radians up its meridian (unitOf(lat + d, lon)). */
  const upBy = (q: N, n: N, d: N): N => {
    const sc = small(d);
    return q.mul(sc.y).add(n.mul(sc.x));
  };
  /** The place `d` radians east about the axis (unitOf(lat, lon + d)). */
  const eastBy = (q: N, d: N): N => {
    const sc = small(d);
    return vec3(q.x.mul(sc.y).sub(q.y.mul(sc.x)), q.x.mul(sc.x).add(q.y.mul(sc.y)), q.z);
  };
  // The angle between two places: near, from the chord by the arc sine's series (2 asin(c/2) = c + c³/24 + 3c⁵/640 +
  // 5c⁷/7168, good to 1e-8 under half a radian); far, where it is only weighed against a storm's reach, the device's own.
  const angle = (a: N, b: N): N => {
    const c = length(a.sub(b));
    const c2 = c.mul(c);
    const near = c.mul(float(1).add(c2.mul(float(1 / 24).add(c2.mul(float(3 / 640).add(c2.mul(5 / 7168)))))));
    return select(c.lessThan(0.5), near, atan(length(cross(a, b)), dot(a, b)));
  };
  const wrapLon = (l: N): N => {
    const x = l.add(180).sub(floor(l.add(180).div(360)).mul(360));
    return x.sub(180);
  };
  const smooth5 = (x: N): N => x.mul(x).mul(x).mul(x.mul(x.mul(6).sub(15)).add(10));

  const zonal: N = inline(Fn(([lat]: [N]) => {
    const x = clamp(lat.add(90).div(t.u_step_deg), 0, uCount - 1);
    const k = min(floor(x), uCount - 2);
    const f = x.sub(k);
    const u0 = at(textures.u, k).x;
    const u1 = at(textures.u, k.add(1)).x;
    return u0.add(u1.sub(u0).mul(f));
  }), { name: "ssZonal", type: "float", inputs: [{ name: "lat", type: "float" }] });

  /** The pole's polygon edge along a longitude and its slope, km and km a radian; the jet's circle without one. */
  const polygon: N = inline(Fn(([lat, lon]: [N, N]) => {
    const sides = select(lat.greaterThanEqual(0), sidesN, sidesS);
    const orient = select(lat.greaterThanEqual(0), orientN, orientS);
    const out = vec2(rJet, 0).toVar();
    If(sides.greaterThan(0), () => {
      const twoPi = float(2 * Math.PI);
      const seg = twoPi.div(sides);
      const r = lon.sub(orient).div(DEG);
      const e = r.sub(floor(r.div(twoPi)).mul(twoPi));
      const tt = e.sub(floor(e.div(seg)).mul(seg));
      const ts = tt.sub(float(Math.PI).div(sides));
      const sc = sinCos(ts);
      const c = sc.y;
      const k = rJet.mul(sinCos(float(Math.PI).div(sides)).y);
      out.assign(vec2(k.div(c) as N, k.mul(sc.x).div(c.mul(c)) as N));
    });
    return out;
  }), { name: "ssPolygon", type: "vec2", inputs: [{ name: "lat", type: "float" }, { name: "lon", type: "float" }] });

  /** The bands' and polar jet's wind, m/s [east, north] (stirred-carry.ts `bandWind`). */
  const bandWind: N = inline(Fn(([lat, lon]: [N, N]) => {
    const r = float(90).sub(abs(lat)).div(DEG).mul(aKm);
    const pe = polygon(lat, lon);
    const edge = pe.x;
    const slope = pe.y;
    const out = vec2(zonal(lat), 0).toVar();
    If(slope.notEqual(0).or(edge.notEqual(rJet)), () => {
      const h = select(lat.greaterThanEqual(0), float(1), float(-1));
      const inside = r.lessThan(edge);
      const x = r.div(edge);
      const q = float(1).sub(x.mul(x));
      const trIn = x.mul(4).mul(q).div(edge);
      const z = r.sub(edge).div(LN);
      const tOut = exp(z.mul(z).negate());
      const trOut = z.mul(-2).div(LN).mul(tOut);
      const tt = select(inside, float(1).sub(q.mul(q)), tOut);
      const tr = select(inside, trIn, trOut);
      const te = select(inside, x.negate().mul(trIn), trOut.negate());
      const off = rJet.sub(edge);
      const rho = r.add(off.mul(tt));
      const rhoR = float(1).add(off.mul(tr));
      const rhoL = slope.negate().mul(tt).add(off.mul(te).mul(slope));
      const latRho = h.mul(float(90).sub(clamp(rho.div(aKm).mul(DEG), 0, 180)));
      const u: N = zonal(latRho) as N;
      out.assign(vec2(u.mul(rhoR) as N, h.mul(u).mul(rhoL).div(aKm.mul(max(sinCos(lat.div(DEG)).y, 1e-9) as N) as N) as N));
    });
    return out;
  }), { name: "ssBandWind", type: "vec2", inputs: [{ name: "lat", type: "float" }, { name: "lon", type: "float" }] });

  /** The bands' stream function at a place, m²/s (stirred-carry.ts `bandPsi`). */
  const bandPsi: N = inline(Fn(([lat, lon]: [N, N]) => {
    const r = float(90).sub(abs(lat)).div(DEG).mul(aKm);
    const pe = polygon(lat, lon);
    const edge = pe.x;
    const rho = r.toVar();
    If(pe.y.notEqual(0).or(edge.notEqual(rJet)), () => {
      const x = r.div(edge);
      const q = float(1).sub(x.mul(x));
      const z = r.sub(edge).div(LN);
      const tt = select(r.lessThan(edge), float(1).sub(q.mul(q)), exp(z.mul(z).negate()));
      rho.assign(r.add(rJet.sub(edge).mul(tt)));
    });
    const x = clamp(rho.div(aKm).mul(DEG).div(t.psi_step_deg), 0, psiCount - 1);
    const k = min(floor(x), psiCount - 2);
    const f = x.sub(k);
    const north = lat.greaterThanEqual(0);
    const a0 = select(north, at(textures.psiN, k).x, at(textures.psiS, k).x);
    const a1 = select(north, at(textures.psiN, k.add(1)).x, at(textures.psiS, k.add(1)).x);
    const psi = a0.add(a1.sub(a0).mul(f));
    return select(north, psi, float(t.psi_south).sub(psi));
  }), { name: "ssBandPsi", type: "float", inputs: [{ name: "lat", type: "float" }, { name: "lon", type: "float" }] });

  /** How firmly a pole's polygon holds cloud at a place (stirred-carry.ts `polygonHold`). */
  const polygonHold: N = inline(Fn(([lat, lon]: [N, N]) => {
    const sides = select(lat.greaterThanEqual(0), sidesN, sidesS);
    const r = float(90).sub(abs(lat)).div(DEG).mul(aKm);
    const edge = polygon(lat, lon).x;
    const z = r.sub(edge).div(LN);
    const hold = select(r.lessThan(edge), float(1), exp(z.mul(z).negate()));
    return select(sides.greaterThan(0), hold, float(0));
  }), { name: "ssHold", type: "float", inputs: [{ name: "lat", type: "float" }, { name: "lon", type: "float" }] });

  /** A storm's centre and frame on a day, and its ellipse: [c, east, north] and (k, half size, km). */
  const stormAt = (slat: N, slon: N, size: N, aspect: N) => {
    const c = unitOf(slat, slon);
    const e = eastOf(c);
    return { c, e, n: northOf(c, e), k: select(aspect.greaterThan(0), sqrt(aspect), float(1)), rr: max(size, 1).mul(0.5) };
  };
  type Storm = ReturnType<typeof stormAt>;
  /** A storm's distance in its own radii, in its ellipse. */
  const stormR = (pp: N, S: Storm): N => {
    const pe = dot(pp, S.e);
    const pn = dot(pp, S.n);
    const len = sqrt(pe.mul(pe).add(pn.mul(pn)));
    const d = angle(pp, S.c).mul(aKm);
    const x = select(len.lessThan(1e-15), float(0), d.mul(pe).div(len));
    const y = select(len.lessThan(1e-15), float(0), d.mul(pn).div(len));
    return sqrt(x.div(S.k).mul(x.div(S.k)).add(y.mul(S.k).mul(y.mul(S.k)))).div(S.rr);
  };

  /** The quintic taper of a vortex between 2.5 and 3 radii. */
  const taperOf = (x: N): N => smooth5(clamp(float(3).sub(x).div(0.5), 0, 1));
  const taperSlope = (x: N): N => {
    const tt = clamp(float(3).sub(x).div(0.5), 0, 1);
    return select(tt.greaterThan(0).and(tt.lessThan(1)), tt.mul(tt).mul(float(1).sub(tt)).mul(float(1).sub(tt)).mul(-60), float(0));
  };
  const shield = (r: N): N => smooth5(clamp(r.sub(1).div(0.5), 0, 1));

  /** A storm's stream function at a place, m²/s. */
  const stormPhi = (pp: N, S: Storm, q: N): N => {
    const x = stormR(pp, S);
    return q.negate().mul(exp(float(0.5).mul(float(1).sub(x.mul(x))))).mul(taperOf(x));
  };

  /** The eddies' winds along the surface and their stream function at a place (vec4: wind, Ψ). */
  const eddySum: N = inline(Fn(([pp, day]: [N, N]) => {
    const lat = latOf(pp);
    const lon = lonOf(pp);
    const e = eastOf(pp);
    const n = northOf(pp, e);
    const out = vec4(0, 0, 0, 0).toVar();
    const raw = floor(day.div(GRID_D));
    If(raw.greaterThanEqual(0).and(raw.lessThanEqual(p.bins)), () => {
      const bin = min(raw, p.bins - 1);
      const row = clamp(floor(lat.add(90).div(CELL_DEG)), 0, ROWS - 1);
      // The place's longitude in its row's frame, turned back to the bin's start (packCarry files the eddies so).
      const turned = lon.sub(at(textures.rates, row).x.mul(day.sub(bin.mul(GRID_D))));
      const col = clamp(floor(wrapLon(turned).add(180).div(CELL_DEG)), 0, COLS - 1);
      const cell = at(textures.grid, bin.mul(ROWS * COLS).add(row.mul(COLS)).add(col));
      // Where its list starts, in two parts (stirred-pack.ts `SPLIT`): the whole of it is past a float's whole numbers.
      const first = int(cell.x).mul(SPLIT).add(int(cell.y));
      const count = int(cell.z);
      Loop({ start: int(0), end: count, type: "int", condition: "<" }, ({ i }: { i: N }) => {
        const idx = at(textures.list, first.add(i), p.listW).x;
        const e0 = at(textures.eddies, idx.mul(2));
        const e1 = at(textures.eddies, idx.mul(2).add(1));
        const born = e1.y;
        const ends = e1.z;
        const alive = day.greaterThanEqual(born).and(day.lessThan(ends)).and(ends.greaterThan(born));
        const sx = sinCos(float(Math.PI).mul(day.sub(born)).div(ends.sub(born))).x;
        const s = select(alive, e0.w.mul(sx).mul(sx), float(0));
        const r = e0.z.mul(500);
        const clon = e0.y.add(e1.w.mul(day.sub(born)).mul(DAY_S).div(A.mul(max(sinCos(e0.x.div(DEG)).y, 1e-6))).mul(DEG));
        const c = unitOf(e0.x, clon);
        const sigma = angle(pp, c);
        const d = sigma.mul(A);
        const x = d.div(r);
        const x2 = x.mul(x);
        If(s.greaterThan(0).and(x2.lessThanEqual(9)), () => {
          const h = select(e0.x.greaterThanEqual(0), float(1), float(-1));
          const q = e1.x.mul(h).mul(TURN).mul(s).mul(r);
          const ex = exp(float(0.5).mul(float(1).sub(x2)));
          const w = taperOf(x);
          const dw = taperSlope(x);
          const sn = length(cross(pp, c));
          const dphi = q.mul(ex).div(r).mul(x.mul(w).sub(dw));
          const ok = sn.greaterThan(1e-12);
          const u = select(ok, dphi.mul(dot(n, c)).div(sn), float(0));
          const v = select(ok, dphi.negate().mul(dot(e, c)).div(sn), float(0));
          out.addAssign(vec4(e.mul(u).add(n.mul(v)), q.negate().mul(ex).mul(w)));
        });
      });
    });
    return out;
  }), { name: "ssEddies", type: "vec4", inputs: [{ name: "p", type: "vec3" }, { name: "day", type: "float" }] });

  const storm = (i: N, k: number): N => at(textures.storms, int(i).mul(3).add(k));
  /** A live storm's centre on a day. */
  const stormLon = (s0: N, s2: N, day: N): N => wrapLon(s0.y.add(s2.y.mul(day.sub(s2.x))));

  /** The wind at a place on a day, m/s along the surface (stirred-carry.ts `Carrier.wind`). */
  const wind: N = inline(Fn(([pp, day]: [N, N]) => {
    const lat = latOf(pp);
    const lon = lonOf(pp);
    const e = eastOf(pp);
    const n = northOf(pp, e);
    const cosLat = length(pp.xy);
    const own = vec2(0, 0).toVar();
    const near = int(0).toVar();
    Loop({ start: int(0), end: int(uStorms), type: "int", condition: "<" }, ({ i }: { i: N }) => {
      const s0 = storm(i, 0);
      const s1 = storm(i, 1);
      const s2 = storm(i, 2);
      If(day.greaterThanEqual(s1.z).and(day.lessThan(s1.w)), () => {
        const S = stormAt(s0.x, stormLon(s0, s2, day), s0.z, s0.w);
        const r = stormR(pp, S);
        If(r.lessThan(1.6), () => {
          near.addAssign(1);
        });
        If(r.lessThanEqual(3.01).and(cosLat.greaterThan(1e-6)), () => {
          const h = select(s0.x.greaterThanEqual(0), float(1), float(-1));
          const q = s1.y.mul(h).mul(TURN).mul(s1.x).mul(S.rr).mul(1000);
          // The slopes across a hundredth of the storm's radius (dd in radians here, degrees in the processor's).
          const dd = S.rr.mul(10).div(A);
          const cp = cosLat;
          const dphi = stormPhi(upBy(pp, n, dd), S, q).sub(stormPhi(upBy(pp, n, dd.negate()), S, q)).div(dd.mul(2));
          const dlam = stormPhi(eastBy(pp, dd.div(cp)), S, q).sub(stormPhi(eastBy(pp, dd.div(cp).negate()), S, q)).div(dd.mul(2).div(cp));
          own.addAssign(vec2(dphi.negate().div(A), dlam.div(A.mul(cp))));
        });
      });
    });
    const ed = eddySum(pp, day);
    const bw = bandWind(lat, lon);
    const background = e.mul(bw.x).add(n.mul(bw.y)).add(ed.xyz);
    const out = e.mul(own.x).add(n.mul(own.y)).toVar();
    If(near.equal(0), () => {
      out.addAssign(background);
    }).Else(() => {
      // A storm turns as one body: Ψ' = Ψ + D (1 − G), D the near storms' drift against Ψ here, weighted by 1 − shield.
      const psi0 = bandPsi(lat, lon).add(ed.w);
      const dd = float(1e-3);
      // At the place and a step either way: [G, Σ x, Σ x δ] for north, south, east, west and here.
      const gN = float(1).toVar();
      const gS = float(1).toVar();
      const gE = float(1).toVar();
      const gW = float(1).toVar();
      const g0 = float(1).toVar();
      const wN = vec2(0, 0).toVar();
      const wS = vec2(0, 0).toVar();
      const wE = vec2(0, 0).toVar();
      const wW = vec2(0, 0).toVar();
      const cp = max(cosLat, 1e-6);
      // The four places a step either way, taken once for all the storms.
      const ddr = dd.div(DEG);
      const pN = upBy(pp, n, ddr);
      const pS = upBy(pp, n, ddr.negate());
      const pE = eastBy(pp, ddr);
      const pW = eastBy(pp, ddr.negate());
      Loop({ start: int(0), end: int(uStorms), type: "int", condition: "<" }, ({ i }: { i: N }) => {
        const s0 = storm(i, 0);
        const s1 = storm(i, 1);
        const s2 = storm(i, 2);
        If(day.greaterThanEqual(s1.z).and(day.lessThan(s1.w)), () => {
          const slon = stormLon(s0, s2, day);
          const S = stormAt(s0.x, slon, s0.z, s0.w);
          const r = stormR(pp, S);
          If(r.lessThan(1.6), () => {
            const psiC = bandPsi(s0.x, slon).add(eddySum(S.c, day).w);
            const om = s2.y.div(DEG).div(DAY_S);
            const base = psiC.sub(psi0);
            // The storm's drift against Ψ at a place: sin(lat) is the place's own z.
            const delta = (q: N) => base.sub(om.mul(A).mul(A).mul(q.z.sub(S.c.z)));
            const add = (g: N, wv: N, q: N) => {
              const sv: N = shield(stormR(q, S)) as N;
              const x: N = float(1).sub(sv) as N;
              g.mulAssign(sv);
              wv.addAssign(vec2(x, x.mul(delta(q)) as N));
            };
            add(gN, wN, pN);
            add(gS, wS, pS);
            add(gE, wE, pE);
            add(gW, wW, pW);
            g0.mulAssign(shield(r));
          });
        });
      });
      const chi = (g: N, wv: N): N => select(wv.x.greaterThan(0), wv.y.div(wv.x), float(0)).mul(float(1).sub(g));
      const step = dd.mul(2).div(DEG);
      const dPhi = chi(gN, wN).sub(chi(gS, wS)).div(step);
      const dLam = chi(gE, wE).sub(chi(gW, wW)).div(step);
      const ux = dPhi.negate().div(A);
      const vx = dLam.div(A.mul(cp));
      out.addAssign(background.mul(g0).add(e.mul(ux)).add(n.mul(vx)));
    });
    return out;
  }), { name: "ssWind", type: "vec3", inputs: [{ name: "p", type: "vec3" }, { name: "day", type: "float" }] });

  /** One fourth-order Runge-Kutta step of h seconds (negative: backwards). */
  const step: N = inline(Fn(([pp, day, h]: [N, N, N]) => {
    const hd = h.div(DAY_S);
    const k1 = wind(pp, day);
    const k2 = wind(normalize(pp.add(k1.mul(h.mul(0.5).div(A)))), day.add(hd.mul(0.5)));
    const k3 = wind(normalize(pp.add(k2.mul(h.mul(0.5).div(A)))), day.add(hd.mul(0.5)));
    const k4 = wind(normalize(pp.add(k3.mul(h.div(A)))), day.add(hd));
    const v = k1.add(k2.mul(2)).add(k3.mul(2)).add(k4).div(6);
    return normalize(pp.add(v.mul(h.div(A))));
  }), { name: "ssStep", type: "vec3", inputs: [{ name: "p", type: "vec3" }, { name: "day", type: "float" }, { name: "h", type: "float" }] });

  /** Where today's face is read for cloud that was at x `age` days ago (stirred-carry.ts `readPlace`). */
  const readPlace: N = inline(Fn(([x, day, age]: [N, N, N]) => {
    const lat = latOf(x);
    const lon = lonOf(x);
    const then = day.sub(age);
    const row = zonal(lat).mul(age).mul(DAY_S).div(A.mul(max(length(x.xy), 1e-6))).mul(DEG);
    const band = float(1).sub(polygonHold(lat, lon)).mul(row);
    const g = float(0).toVar();
    const moved = float(0).toVar();
    Loop({ start: int(0), end: int(uStorms), type: "int", condition: "<" }, ({ i }: { i: N }) => {
      const s0 = storm(i, 0);
      const s1 = storm(i, 1);
      const s2 = storm(i, 2);
      If(then.greaterThanEqual(s1.z).and(then.lessThan(s1.w)), () => {
        const r = stormR(x, stormAt(s0.x, stormLon(s0, s2, then), s0.z, s0.w));
        const gs = exp(r.mul(r).negate());
        If(r.lessThanEqual(3).and(gs.greaterThan(g)), () => {
          g.assign(gs);
          moved.assign(s2.y.mul(age));
        });
      });
    });
    return vec2(lat, wrapLon(lon.add(select(g.greaterThan(0), moved, band))));
  }), { name: "ssRead", type: "vec2", inputs: [{ name: "x", type: "vec3" }, { name: "day", type: "float" }, { name: "age", type: "float" }] });

  const ij = () => ivec2(int(screenCoordinate.x), int(screenCoordinate.y));
  /** A place's band and its two generations' ages. */
  const agesAt = (lat: N): N => {
    const j = float(Math.max(t.jets, 1));
    const k = min(floor(sinCos(abs(lat).div(DEG)).x.mul(j)), j.sub(1));
    return at(textures.ages, select(lat.greaterThanEqual(0), k, j.add(k)));
  };
  /**
   * One step of a leg, for every pixel: the state is (place, steps done); the leg starts at `start`
   * (leg 0: the picture's places, lat and lon; leg 1: where leg 0 ended, a unit vector) on its own date.
   */
  const legMaterial = (leg: 0 | 1, startTex: Texture, stateTex: Texture) => {
    const m = raw();
    m.fragmentNode = Fn(() => {
      const here = textureLoad(places, ij());
      const ages = agesAt(here.x);
      const st = textureLoad(startTex, ij());
      const start = leg === 0 ? unitOf(st.x, st.y) : st.xyz;
      const state = textureLoad(stateTex, ij());
      const q = state.xyz.toVar();
      const done = state.w;
      const days = leg === 0 ? ages.x : ages.y.sub(ages.x);
      const from = leg === 0 ? uDay : uDay.sub(ages.x);
      const dt = select(abs(latOf(start)).greaterThanEqual(t.poly_lat_deg), float(t.dt_poly_s), float(t.dt_s));
      const n = select(days.greaterThan(0), max(ceil(days.mul(DAY_S).div(dt)), 1), float(0));
      const h = days.mul(DAY_S).div(max(n, 1));
      // One step a draw, with no loop round the step: SwiftShader, the software device the tests run on, takes minutes
      // and gigabytes to build a loop round the wind, and three.js r187 cannot build one at all (its node builder
      // overflows its stack, on every device: AC-ProArt, 28 Nov). Many draws go to a frame instead (carryOnGpu).
      If(done.lessThan(n), () => {
        q.assign(step(q, from.sub(done.mul(h).div(DAY_S)), h.negate()));
      });
      return vec4(q, min(done.add(1), n));
    })();
    return m;
  };
  /** Where today's face is read for each generation, from where the two legs ended. */
  const readMaterial = (p1: Texture, p2: Texture) => {
    const m = raw();
    m.fragmentNode = Fn(() => {
      const here = textureLoad(places, ij());
      const ages = agesAt(here.x);
      const r1 = readPlace(textureLoad(p1, ij()).xyz, uDay, ages.x);
      const r2 = readPlace(textureLoad(p2, ij()).xyz, uDay, ages.y);
      return vec4(r1, r2);
    })();
    return m;
  };
  /**
   * What the carrying reads at each place, for its tests: 0, the place and its two ages; 1, the wind there today
   * (east, north) and the bands' wind alone; 2, the place one step of `h` seconds back from today, and how far a place's own lat and lon come back from it.
   */
  const probeMaterial = (kind: 0 | 1 | 2, h = 0) => {
    const m = raw();
    m.fragmentNode = Fn(() => {
      const here = textureLoad(places, ij());
      const pp = unitOf(here.x, here.y);
      if (kind === 0) return vec4(here.xy, agesAt(here.x));
      if (kind === 1) {
        const v = wind(pp, uDay);
        const e = eastOf(pp);
        return vec4(dot(v, e), dot(v, northOf(pp, e)), bandWind(here.x, here.y));
      }
      const q = step(pp, uDay, float(-h));
      return vec4(latOf(q), lonOf(q), latOf(pp).sub(here.x), lonOf(pp).sub(here.y));
    })();
    return m;
  };
  return {
    legMaterial,
    readMaterial,
    probeMaterial,
    dispose() {
      for (const x of Object.values(textures)) x.dispose();
    },
  };
}

/** What the carrying asks of a WebGPU device (so no WebGPU type package is needed). */
interface Device {
  limits: { maxTextureDimension2D: number };
  pushErrorScope(filter: "validation" | "out-of-memory"): void;
  popErrorScope(): Promise<{ message: string } | null>;
}

/** The largest texture side the renderer's device allows. */
function maxSide(renderer: WebGPURenderer): number {
  const b = renderer.backend as { device?: Device | null; gl?: WebGL2RenderingContext };
  if (b.device) return b.device.limits.maxTextureDimension2D;
  if (b.gl) return b.gl.getParameter(b.gl.MAX_TEXTURE_SIZE) as number;
  return 8192;
}


/**
 * Carry every place given (lat, lon in degrees, `width` × `height` of them, row by row) back on the GPU. The steps run
 * one a draw, the state kept between them in float targets, so no one draw runs long enough to stall the device.
 * With `each` (a view waits a frame there), the draws go in batches, each waited on and then `each` called, the batch
 * doubled while it takes under half of `budgetMs` and halved when over it; without, one texel is read back every
 * `wait` draws. Returns RGBA per place: younger lat, lon, older lat, lon.
 */
export async function carryOnGpu(renderer: WebGPURenderer, p: PackedCarry, places: Float32Array, width: number, height: number, opts: { wait?: number; budgetMs?: number; each?: () => Promise<void>; probe?: { kind: 0 | 1 | 2; h?: number } } = {}): Promise<Float32Array> {
  // Every table texture must fit the device: a texture too tall to exist is not an error three.js throws, it only
  // leaves every draw after it invalid, and the carrying would return its places unmoved (AC-ProArt, 28 Nov).
  const side = maxSide(renderer);
  const tallest = Math.max(p.list.length / p.listW, ...[p.u.length, p.psiN.length, p.psiS.length, p.storms.length / 4, p.eddies.length / 4, p.grid.length / 4].map((n) => n / W));
  const widest = Math.max(p.listW, W, width);
  if (tallest > side || widest > side || height > side) throw new Error(`the carry table needs a texture ${widest} wide and ${Math.ceil(tallest)} tall; this device allows ${side}`);
  const device = (renderer.backend as { device?: Device | null }).device ?? null;
  device?.pushErrorScope("out-of-memory");
  device?.pushErrorScope("validation");
  const tex = (data: Float32Array, format: typeof RGFormat | typeof RGBAFormat) => {
    const x = new DataTexture(data, width, height, format, FloatType);
    x.magFilter = NearestFilter;
    x.minFilter = NearestFilter;
    x.generateMipmaps = false;
    x.needsUpdate = true;
    return x;
  };
  const placeTex = tex(places, RGFormat);
  const prog = carryMaterial(p, placeTex);
  const target = () => new RenderTarget(width, height, { type: FloatType, format: RGBAFormat, minFilter: NearestFilter, magFilter: NearestFilter, depthBuffer: false });
  // Leg 0 starts from the places themselves, with no steps done.
  const init = new Float32Array(width * height * 4);
  for (let k = 0; k < width * height; k++) {
    const la = places[k * 2]! / DEG;
    const lo = places[k * 2 + 1]! / DEG;
    init.set([Math.cos(la) * Math.cos(lo), Math.cos(la) * Math.sin(lo), Math.sin(la), 0], k * 4);
  }
  const initTex = tex(init, RGBAFormat);
  const rts = [target(), target(), target(), target(), target()];
  const scene = new Scene();
  const quad = new Mesh(new PlaneGeometry(2, 2));
  scene.add(quad);
  const cam = new OrthographicCamera(-1, 1, 1, -1, 0, 1);
  cam.position.z = 0.5;
  const was = renderer.getRenderTarget();
  const materials: { dispose(): void }[] = [];
  // Draws queue up unrun until something waits on them: every few, one texel is read back, so the device never holds
  // a long queue of them (a long queue is lost as surely as one long draw).
  let queued = 0;
  let batch = 1;
  let since = performance.now();
  let least = Infinity;
  const budget = opts.budgetMs ?? 8;
  const built = new Set<MeshBasicNodeMaterial>();
  const draw = async (m: MeshBasicNodeMaterial, into: RenderTarget) => {
    quad.material = m;
    renderer.setRenderTarget(into);
    // Each material's shader is built before its first draw without holding the page (the carrying's is large).
    if (!built.has(m)) {
      built.add(m);
      await renderer.compileAsync(scene, cam);
    }
    renderer.render(scene, cam);
    if (++queued < (opts.each ? batch : (opts.wait ?? 4))) return;
    queued = 0;
    await renderer.readRenderTargetPixelsAsync(into, 0, 0, 1, 1);
    if (!opts.each) return;
    // What the batch cost the GPU: the wait less the least any wait has taken, since reading a texel back takes most
    // of a frame on its own on some devices (on AC-ProArt the batch never grew past one draw, 28 Nov).
    const waited = performance.now() - since;
    least = Math.min(least, waited);
    const took = waited - least;
    if (took < budget / 2 && batch < 1024) batch *= 2;
    else if (took > budget && batch > 1) batch = Math.max(1, batch >> 1);
    await opts.each();
    since = performance.now();
  };
  // A target reads back from the bottom row up on WebGL, while the shaders count rows from the top as WebGPU does.
  const readBack = async (): Promise<Float32Array> => {
    const px = (await renderer.readRenderTargetPixelsAsync(rts[4]!, 0, 0, width, height)) as Float32Array;
    if (!(renderer.backend as { isWebGLBackend?: boolean }).isWebGLBackend) return px;
    const out = new Float32Array(px.length);
    const row = width * 4;
    for (let j = 0; j < height; j++) out.set(px.subarray((height - 1 - j) * row, (height - j) * row), j * row);
    return out;
  };
  let popped = false;
  try {
    if (opts.probe) {
      const m = prog.probeMaterial(opts.probe.kind, opts.probe.h);
      materials.push(m);
      await draw(m, rts[4]!);
      return readBack();
    }
    const steps = mostSteps(p);
    const draws = Math.max(1, steps);
    // Each leg ping-pongs between two targets; the state starts from the places (leg 0) or where leg 0 ended (leg 1).
    const runLeg = async (leg: 0 | 1, start: Texture, first: Texture, a: RenderTarget, b: RenderTarget): Promise<RenderTarget> => {
      const fromFirst = prog.legMaterial(leg, start, first);
      const ab = prog.legMaterial(leg, start, a.texture);
      const ba = prog.legMaterial(leg, start, b.texture);
      materials.push(fromFirst, ab, ba);
      await draw(fromFirst, a);
      let cur = a;
      for (let k = 1; k < draws; k++) {
        const next = cur === a ? b : a;
        await draw(cur === a ? ab : ba, next);
        cur = next;
      }
      return cur;
    };
    const end0 = await runLeg(0, placeTex, initTex, rts[0]!, rts[1]!);
    // Leg 1 starts where leg 0 ended, with its steps counted afresh.
    const restart = raw();
    restart.fragmentNode = Fn(() => vec4(textureLoad(end0.texture, ivec2(int(screenCoordinate.x), int(screenCoordinate.y))).xyz, 0))();
    materials.push(restart);
    await draw(restart, rts[2]!);
    const end1 = await runLeg(1, rts[2]!.texture, rts[2]!.texture, rts[3]!, end0 === rts[0] ? rts[1]! : rts[0]!);
    const read = prog.readMaterial(end0.texture, end1.texture);
    materials.push(read);
    await draw(read, rts[4]!);
    const out = await readBack();
    // Any error on the way leaves the places unmoved or wrong: refuse them rather than show them.
    const bad = (await device?.popErrorScope()) ?? null;
    const oom = (await device?.popErrorScope()) ?? null;
    popped = true;
    if (bad || oom) throw new Error(`the GPU refused the carrying: ${(bad ?? oom)!.message}`);
    return out;
  } finally {
    if (!popped) void device?.popErrorScope().then(() => device.popErrorScope()).catch(() => {});
    renderer.setRenderTarget(was);
    for (const m of materials) m.dispose();
    for (const r of rts) r.dispose();
    placeTex.dispose();
    initTex.dispose();
    prog.dispose();
  }
}
