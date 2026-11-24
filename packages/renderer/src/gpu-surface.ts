// The ground at eye height on the GPU (SL1): the kernel's terrain round a landing site as meshes in the site's
// frame (metres, x east, y north, z up), a sky dome lit by single Rayleigh scattering (surface-sky.ts, the same sums
// in TSL), sunlight dimmed by the air, and aerial perspective (the air between the eye and far ground scatters sky
// light in and dims the ground behind it). One TSL source: WGSL on WebGPU, GLSL on WebGL2. The camera sits at the
// origin and the meshes are moved by −eye, so the GPU only sees numbers within the patch (rendering rule 3).
//
// Three looks: "true" (the light as it is, tone-mapped), "3d" (the same ground as a diagram: shaded relief in the
// chart's colours with contour lines), "wireframe" (the mesh's edges on the chart's ground).
import { AdditiveBlending, BackSide, BufferAttribute, BufferGeometry, Color, LinearSRGBColorSpace, Mesh, NoToneMapping, PerspectiveCamera, Points, Scene, SphereGeometry, SRGBColorSpace, Vector3, Vector4 } from "three";
import { startFinish, type Finish } from "./finish.ts";
import { MeshBasicNodeMaterial, PointsNodeMaterial, WebGPURenderer, type Node } from "three/webgpu";
import { fresnelNode } from "./sea-node.ts";
import { frothToDraw, MAX_TRAINS, trainsToDraw, type SeaOptics, type TrainDraw } from "./sea.ts";
import { Fn, abs, attribute, cos, dot, exp, float, fract, fwidth, length, max, min, mix, normalWorld, normalize, positionLocal, positionWorld, pow, select, sin, smoothstep, sqrt, uniform, vec2, vec3, vec4 } from "three/tsl";

export interface SurfaceMesh {
  key: string;
  /** x, y, z per vertex, metres in the site's frame. */
  positions: Float32Array;
  /** Linear reflectance, r, g, b per vertex (true look). */
  albedo: Float32Array;
  /** Display colour, r, g, b per vertex, 0..1 (diagram looks). */
  chart: Float32Array;
  /** The kernel's height per vertex before decoration, metres (contour lines follow it). */
  kernel_z: Float32Array;
  /** 1 where a liquid covers the ground (its sea surface), else 0. Absent: all dry. */
  wet?: Float32Array;
  /** Depth of the liquid over the ground, m (0 where dry). Absent: taken as deep. */
  depth?: Float32Array;
  /** The ground's own reflectance under the liquid (the kernel's tile albedo). Absent: as `albedo`. */
  floor?: Float32Array;
  indices: Uint32Array;
  /** Drawn over lower layers where they overlap (a finer ring inside a coarser one). */
  layer: number;
}

export interface SurfaceFrame {
  look: "true" | "3d" | "wireframe";
  /** The eye in the site's frame, metres. */
  eye: [number, number, number];
  forward: [number, number, number];
  up: [number, number, number];
  fovY: number;
  width: number;
  height: number;
  /** Toward the sun, unit, site frame. */
  sun: [number, number, number];
  /** The star's colour (linear), its light at the top of the air taken as 1 in its brightest channel. */
  sunRgb: [number, number, number];
  /** Optical depth straight up at red, green, blue; 0 with no air. */
  tau: [number, number, number];
  scale_height_m: number;
  radius_m: number;
  /** Angular radius of the star's disc, radians. */
  sun_radius: number;
  exposure: number;
  /** Diagram looks: background, ink and contour interval (metres). */
  ground: string;
  ink: string;
  contour_m: number;
  /**
   * The sea at the site, when there is one (the kernel's weather report, `sse.ocean.wind_waves`): significant wave
   * height and peak period, the way the wind blows them (radians from east), the world's gravity, the tide's height
   * now above mean sea level, and the clock the waves run on (seconds).
   */
  sea?: { wave_m: number; period_s: number; dir_rad: number; gravity_m_s2: number; tide_m: number; time_s: number; optics?: SeaOptics };
  /**
   * True look: the stars over the site (night-sky.ts `starsOverSite`), keyed so they are uploaded once a moment:
   * unit directions in the site's frame, and each star's light as a share of the home star's, times its colour.
   */
  stars?: { key: string; dirs: Float32Array; light: Float32Array };
}

/**
 * The wind-sea drawn on the water (V10k): a few long-crested deep-water wave trains round the wind's direction, each
 * with its own wavelength and the deep-water dispersion ω² = g k, scaled so their combined variance is the kernel's
 * significant height (Hs = 4σ). Wavelength ratio to the peak, angle from the wind (radians), share of amplitude and
 * phase. A stand-in for a full spectrum (fudge register).
 */
export const WAVE_TRAINS: readonly (readonly [number, number, number, number])[] = [
  [1, 0, 0.6, 0],
  [0.62, 0.45, 0.4, 1.7],
  [1.5, -0.35, 0.45, 4.1],
  [0.4, -0.8, 0.25, 2.9],
];
/** The peak train's amplitude for a significant height: Σ (A·share)²/2 = (Hs/4)². */
export function waveAmplitude(wave_m: number): number {
  const sum = WAVE_TRAINS.reduce((t, w) => t + w[2] * w[2], 0);
  return (Math.max(0, wave_m) / 4) * Math.sqrt(2 / sum);
}

/** The stand-in trains for a significant height and peak period (V10k), in the form the shader draws. */
export function standInTrains(wave_m: number, period_s: number, dir_rad: number, gravity_m_s2: number): TrainDraw[] {
  if (!(wave_m > 0 && period_s > 0)) return [];
  const kPeak = (4 * Math.PI * Math.PI) / (gravity_m_s2 * period_s * period_s);
  const a = waveAmplitude(wave_m);
  return WAVE_TRAINS.map(([len, ang, share, ph]) => {
    const k = kPeak / len;
    return { k, a: a * share, w: Math.sqrt(gravity_m_s2 * k), dir: dir_rad + ang, phase: ph };
  });
}

/**
 * The mean share of a foam cell its soft streak covers: an ellipse fading from 0.35 to 0.95 of the cell's half-size,
 * jittered by up to a fifth (∫ over the cell of 1 − smoothstep(0.35, 0.95, r), taken numerically: 0.35). Shares
 * above it saturate at it (whitecaps rarely cover more than a tenth of a sea).
 */
export const FOAM_FILL = 0.35;

export interface SurfaceGpu {
  readonly backend: "webgpu" | "webgl2";
  setMeshes(meshes: SurfaceMesh[]): void;
  draw(f: SurfaceFrame): void;
  dispose(): void;
}

export async function createSurfaceGpu(canvas: HTMLCanvasElement): Promise<SurfaceGpu | null> {
  let renderer: WebGPURenderer;
  try {
    renderer = new WebGPURenderer({ canvas, antialias: true, alpha: false, logarithmicDepthBuffer: true });
    await renderer.init();
  } catch {
    return null;
  }
  renderer.outputColorSpace = LinearSRGBColorSpace;
  let finish: Finish | null = null;
  const backend = (renderer.backend as { isWebGPUBackend?: boolean }).isWebGPUBackend ? "webgpu" : "webgl2";
  const scene = new Scene();
  // A logarithmic depth buffer holds a pebble at 5 cm and the far ring 300 km off in one frame.
  const camera = new PerspectiveCamera(60, 1, 0.05, 4e6);
  camera.up.set(0, 0, 1);

  // Shared uniforms.
  const uSun = uniform(new Vector3(0, 0, 1));
  const uSunRgb = uniform(new Vector3(1, 1, 1));
  const uTau = uniform(new Vector3(0, 0, 0));
  // The air above the eye: the zenith depth thinned by the eye's height (a peak 10 km up sees a darker sky).
  const uTauEye = uniform(new Vector3(0, 0, 0));
  const uK = uniform(1000); // R / H
  const uH = uniform(8000);
  const uEyeZ = uniform(0);
  const uExp = uniform(1);
  const uSunR = uniform(0.0047);
  // The sea (V10k): the eye in the site's frame (the waves are fixed to the ground), the peak wavenumber, amplitude
  // and direction, gravity, time and the tide.
  const uEyeXY = uniform(new Vector3(0, 0, 0));
  // Six wave trains (sea.ts trainsToDraw): per train (k, a, ω, heading) and its phase; the kernel's own when sent.
  const uTrain = Array.from({ length: MAX_TRAINS }, () => uniform(new Vector4(0, 0, 0, 0)));
  const uTrainPh = Array.from({ length: MAX_TRAINS }, () => uniform(0));
  // Froth (W-sea4): the share of the surface white, its reflectance, the streaks' heading; cover 0 draws none.
  const uFoamCover = uniform(0);
  const uFoamAlb = uniform(0);
  const uFoamDir = uniform(0);
  const uTime = uniform(0);
  const uTide = uniform(0);
  // The liquid's own optics when the kernel sends them (W-sea2): deep-water reflectance, absorption per metre, and
  // refractive index. uHasBody is 0 until the colour numbers come; then the water's body is the kernel's.
  const uDeep = uniform(new Vector3(0, 0, 0));
  const uAbsorb = uniform(new Vector3(0, 0, 0));
  const uHasBody = uniform(0);
  const uN = uniform(0);

  const airMass = (mu: Node<"float">) => sqrt(uK.mul(mu).mul(uK.mul(mu)).add(uK.mul(2)).add(1)).sub(uK.mul(mu));
  /** Single-scattered sky light along a unit direction d (the closed form of surface-sky.ts). */
  const skyAlong = (d: Node<"vec3">) => {
    const muV = max(d.z, float(0.0));
    const muS = uSun.z;
    const mv = airMass(muV);
    const ms = airMass(muS);
    const cg = dot(d, uSun);
    const p = float(3 / (16 * Math.PI)).mul(cg.mul(cg).add(1));
    const diff = mv.sub(ms);
    const big = exp(uTauEye.mul(ms).negate()).sub(exp(uTauEye.mul(mv).negate())).div(select(abs(diff).lessThan(1e-5), float(1e-5), diff));
    const L = uSunRgb.mul(p).mul(mv).mul(big);
    return select(muS.lessThan(-0.2), vec3(0, 0, 0), L);
  };
  const sunT = () => select(uSun.z.lessThan(-0.1), vec3(0, 0, 0), exp(uTauEye.mul(airMass(uSun.z)).negate()));

  // The sky: a dome round the eye.
  const skyMat = new MeshBasicNodeMaterial({ side: BackSide, depthWrite: false });
  skyMat.colorNode = Fn(() => {
    const d = normalize(positionWorld);
    const sky = skyAlong(d);
    // The star's disc, dimmed by the air on its way to the eye, with a soft edge one tenth of its size.
    const ang = float(1).sub(dot(d, uSun));
    const r2 = uSunR.mul(uSunR).mul(0.5);
    const disc = smoothstep(r2.mul(1.2), r2.mul(0.8), ang).mul(select(d.z.lessThan(-0.01), float(0), float(1)));
    const sunL = uSunRgb.mul(sunT()).mul(disc).mul(float(1).div(max(uSunR.mul(uSunR).mul(Math.PI), 1e-9)).mul(0.02));
    // Below the horizon of a flat eye the dome is ground the meshes did not reach: the last light of the air.
    const c = sky.add(sunL).mul(uExp);
    return vec4(c, 1);
  })();
  const dome = new Mesh(new SphereGeometry(3e6, 64, 32), skyMat);
  dome.renderOrder = -1;
  dome.frustumCulled = false;
  scene.add(dome);

  // The stars (True): points on the dome, each its light over one pixel's solid angle, dimmed by the air along its
  // line of sight, added to the sky so a lit sky hides them and a dark one shows them.
  const uStarScale = uniform(1);
  const starMat = new PointsNodeMaterial({ transparent: true, blending: AdditiveBlending, depthWrite: false, sizeAttenuation: false });
  starMat.colorNode = Fn(() => {
    const d = normalize(positionWorld);
    const T = exp(uTauEye.mul(airMass(max(d.z, float(0.02)))).negate());
    const c = attribute("light", "vec3").mul(T).mul(uStarScale).mul(uExp).mul(select(d.z.lessThan(0), float(0), float(1)));
    return vec4(c, 1);
  })();
  const starGeom = new BufferGeometry();
  const stars = new Points(starGeom, starMat);
  stars.renderOrder = 1000;
  stars.frustumCulled = false;
  stars.visible = false;
  scene.add(stars);
  let starKey = "";

  // The ground, true look: albedo lit by the sun through the air and by the sky, seen through the air.
  const trueNode = Fn(() => {
    const alb = attribute("albedo", "vec3");
    const n = normalize(normalWorld);
    const ndl = max(dot(n, uSun), float(0));
    const muS = uSun.z;
    const sunLight = uSunRgb.mul(sunT()).mul(ndl);
    // The sky's light on the ground (half of what the beam lost on the way down; surface-sky.ts skyIrradiance).
    const skyLight = uSunRgb.mul(max(muS, float(0))).mul(float(1).sub(sunT())).mul(0.5).mul(n.z.mul(0.5).add(0.5));
    const land = alb.mul(sunLight.add(skyLight)).div(Math.PI);
    // The sea: wave slopes on the water's normal, the sky mirrored in it by Fresnel's law and the sun's glint.
    const wet = attribute("wet", "float");
    const site = vec2(positionWorld.x.add(uEyeXY.x), positionWorld.y.add(uEyeXY.y));
    let sx: Node<"float"> = float(0);
    let sy: Node<"float"> = float(0);
    for (let i = 0; i < MAX_TRAINS; i++) {
      const t = uTrain[i]!;
      const k = t.x;
      const dx = cos(t.w);
      const dy = cos(t.w.sub(Math.PI / 2));
      const phase = k.mul(site.x.mul(dx).add(site.y.mul(dy)));
      // A train finer than a pixel averages out: faded where its phase turns faster than once a few pixels.
      const fade = float(1).sub(smoothstep(float(0.4), float(1.5), fwidth(phase)));
      const slope = t.y.mul(k).mul(cos(phase.sub(t.z.mul(uTime)).add(uTrainPh[i]!))).mul(fade);
      sx = sx.add(slope.mul(dx));
      sy = sy.add(slope.mul(dy));
    }
    const nw = normalize(vec3(sx.negate(), sy.negate(), 1));
    const view = normalize(positionWorld);
    const cosV = max(dot(view.negate(), nw), float(0));
    // Fresnel: the exact unpolarised split at the liquid's own index when the kernel sends it; Schlick's water-like
    // curve until then (fudge register).
    const schlick = float(0.02).add(float(0.98).mul(pow(float(1).sub(cosV), 5)));
    const fres = select(uN.greaterThan(1), fresnelNode(uN, cosV), schlick);
    const refl = view.sub(nw.mul(dot(view, nw).mul(2)));
    const mirrored = skyAlong(vec3(refl.x, refl.y, max(refl.z, float(0.02))));
    // The sun's disc mirrored, spread by the slopes finer than the trains drawn (a narrow lobe; fudge register).
    const glint = uSunRgb.mul(sunT()).mul(pow(max(dot(refl, uSun), float(0)), 1500)).mul(2000);
    // The water's body: light that went in, came back up from the liquid itself (the kernel's deep reflectance) or from
    // the floor seen through it, faded by the round trip, exp(−2 a z) (Beer–Lambert). Until the kernel sends the
    // liquid's numbers, the ground colour under the sea as before.
    const depth = attribute("depth", "float");
    const floorAlb = vec3(attribute("floor", "float"));
    const through = exp(uAbsorb.mul(depth.mul(-2)));
    const own = uDeep.mul(vec3(1, 1, 1).sub(through)).add(floorAlb.mul(through));
    const seaAlb = select(uHasBody.greaterThan(0.5), own, alb);
    const body = seaAlb.mul(uSunRgb.mul(sunT()).mul(max(dot(nw, uSun), float(0))).add(skyLight)).div(Math.PI);
    const clear = body.mul(float(1).sub(fres)).add(mirrored.add(glint).mul(fres));
    // Froth (W-sea4): streaks along the wind, cells 1 m across and 6 m along it; a cell is white when its hash falls
    // under the kernel's whitecap share, so the white area is that share on average (fudge register: where the froth
    // lies is a picture choice; how much is the kernel's). Lit like the ground, at the froth's own reflectance.
    const fdx = cos(uFoamDir);
    const fdy = cos(uFoamDir.sub(Math.PI / 2));
    const along = site.x.mul(fdx).add(site.y.mul(fdy)).div(6);
    const across = site.y.mul(fdx).sub(site.x.mul(fdy));
    // Each cell holds at most one soft streak, an ellipse filling FOAM_FILL of the cell on average; a cell carries one
    // when its hash falls under cover / FOAM_FILL, so the white area is the kernel's share. The streak sits at a random
    // spot in its cell so the pattern does not read as a grid.
    const cell = vec2(along.floor(), across.floor());
    const hash = fract(sin(dot(cell, vec2(12.9898, 78.233))).mul(43758.5453));
    const jx = fract(hash.mul(17.0)).sub(0.5).mul(0.4);
    const jy = fract(hash.mul(131.0)).sub(0.5).mul(0.4);
    const fu = fract(along).sub(0.5).sub(jx).mul(2);
    const fv = fract(across).sub(0.5).sub(jy).mul(2);
    const fr = sqrt(fu.mul(fu).add(fv.mul(fv)));
    const blob = float(1).sub(smoothstep(float(0.35), float(0.95), fr));
    const carries = select(hash.lessThan(uFoamCover.div(FOAM_FILL)), float(1), float(0));
    const foamy = carries.mul(blob).mul(float(1).sub(smoothstep(float(0.5), float(2), fwidth(across))));
    const foamLit = vec3(uFoamAlb).mul(uSunRgb.mul(sunT()).mul(max(uSun.z, float(0))).add(skyLight)).div(Math.PI);
    const water = mix(clear, foamLit, foamy);
    const lit = mix(land, water, select(wet.greaterThan(0.5), float(1), float(0)));
    // Aerial perspective: the eye is at the origin, the point at positionWorld (camera-relative).
    const v = positionWorld;
    const dist = length(v);
    const d = v.div(max(dist, 1e-3));
    const z0 = uEyeZ;
    const z1 = uEyeZ.add(v.z);
    const dz = v.z;
    // Optical depth along the path through air thinning with height: τ/H · ∫ e^(−z/H) ds.
    const e0 = exp(z0.div(uH).negate());
    const e1 = exp(z1.div(uH).negate());
    const flat = abs(dz).lessThan(1);
    const column = select(flat, e0.mul(dist), uH.mul(dist).div(select(flat, float(1), dz)).mul(e0.sub(e1)));
    const od = uTau.div(uH).mul(column);
    const T = exp(od.negate());
    const inscatter = skyAlong(vec3(d.x, d.y, max(d.z, float(0.02)))).mul(float(1).sub(T));
    const c = lit.mul(T).add(inscatter).mul(uExp);
    return vec4(c, 1);
  })();

  // Diagram looks: the chart colours, shaded, with contour lines every uC metres.
  const uInk = uniform(new Vector3(0.13, 0.13, 0.13));
  const uC = uniform(50);
  const uLow = uniform(0);
  const chartNode = Fn(() => {
    const col = attribute("chart", "vec3");
    const n = normalize(normalWorld);
    const shade = float(0.55).add(float(0.45).mul(max(dot(n, normalize(vec3(-0.5, 0.5, 0.8))), float(0))));
    const z = attribute("kernel_z", "float").sub(uLow);
    // Contours at three intervals (the chosen one, five and twenty-five times it), each faded out where its lines
    // would crowd closer than a few pixels on screen: a slope seen edge-on far off keeps only the index lines, so
    // the ground reads as a diagram at every distance instead of a field of aliased lines.
    const contour = (every: Node<"float">, ink: number) => {
      const u = z.div(every);
      const f = fract(u);
      const w = fwidth(u);
      const line = float(1).sub(smoothstep(float(0), max(w, float(1e-6)).mul(1.2), min(f, float(1).sub(f))));
      // Level ground (a sea at its datum, a flat plain on a contour's level) has no lines at all: without this the
      // contour sits exactly on the surface and every pixel flickers on and off it.
      const fade = smoothstep(float(1e-5), float(1e-4), w).mul(float(1).sub(smoothstep(float(0.12), float(0.3), w)));
      return line.mul(fade).mul(ink);
    };
    // A faint ground grid every five contour intervals, east and north in the site's frame, so level ground (a plain
    // with no contour crossing it) still reads as ground running away to the horizon.
    const gridLine = (u: Node<"float">) => {
      const f = fract(u);
      const w = fwidth(u);
      const line = float(1).sub(smoothstep(float(0), max(w, float(1e-6)).mul(1.2), min(f, float(1).sub(f))));
      return line.mul(float(1).sub(smoothstep(float(0.12), float(0.3), w))).mul(0.25);
    };
    const gx = positionWorld.x.add(uEyeXY.x).div(uC.mul(5));
    const gy = positionWorld.y.add(uEyeXY.y).div(uC.mul(5));
    const lines = max(max(contour(uC, 0.45), max(contour(uC.mul(5), 0.7), contour(uC.mul(25), 0.85))), max(gridLine(gx), gridLine(gy)));
    const c = mix(col.mul(shade), vec3(uInk), lines);
    return vec4(c, 1);
  })();
  const wireNode = vec4(vec3(uInk), 1);
  // One set of materials per layer: a finer ring is pulled toward the eye where it overlaps a coarser one.
  const layers = new Map<number, { true: MeshBasicNodeMaterial; chart: MeshBasicNodeMaterial; wire: MeshBasicNodeMaterial }>();
  const matsFor = (layer: number) => {
    let e = layers.get(layer);
    if (!e) {
      const make = (node: typeof wireNode, wireframe = false) => {
        const m = new MeshBasicNodeMaterial({ wireframe });
        m.colorNode = node;
        if (layer > 0) ((m.polygonOffset = true), (m.polygonOffsetFactor = -2 * layer), (m.polygonOffsetUnits = -4 * layer));
        // The tide raises and lowers the sea surface (wet vertices) about mean sea level.
        m.positionNode = positionLocal.add(vec3(0, 0, attribute("wet", "float").mul(uTide)));
        return m;
      };
      e = { true: make(trueNode), chart: make(chartNode), wire: make(wireNode, true) };
      layers.set(layer, e);
    }
    return e;
  };
  const allMats = () => [skyMat, ...[...layers.values()].flatMap((e) => [e.true, e.chart, e.wire])];

  const meshes = new Map<string, Mesh>();
  let order: SurfaceMesh[] = [];
  const dropAll = () => {
    for (const m of meshes.values()) {
      scene.remove(m);
      m.geometry.dispose();
    }
    meshes.clear();
  };
  const bg = new Color();
  const ink = new Color();
  let lastLook = "";
  return {
    backend,
    setMeshes(list) {
      dropAll();
      order = list;
      for (const s of list) {
        const g = new BufferGeometry();
        g.setAttribute("position", new BufferAttribute(s.positions, 3));
        g.setAttribute("albedo", new BufferAttribute(s.albedo, 3));
        g.setAttribute("chart", new BufferAttribute(s.chart, 3));
        g.setAttribute("kernel_z", new BufferAttribute(s.kernel_z, 1));
        g.setAttribute("wet", new BufferAttribute(s.wet ?? new Float32Array(s.kernel_z.length), 1));
        g.setAttribute("depth", new BufferAttribute(s.depth ?? new Float32Array(s.kernel_z.length).fill(1e4), 1));
        g.setAttribute("floor", new BufferAttribute(s.floor ?? Float32Array.from({ length: s.kernel_z.length }, (_, i) => s.albedo[i * 3 + 1]!), 1));
        g.setIndex(new BufferAttribute(s.indices, 1));
        g.computeVertexNormals();
        g.computeBoundingSphere();
        const m = new Mesh(g, matsFor(s.layer).true);
        m.frustumCulled = false;
        m.renderOrder = s.layer;
        meshes.set(s.key, m);
        scene.add(m);
      }
    },
    draw(f) {
      renderer.setSize(f.width, f.height, false);
      camera.aspect = f.width / Math.max(1, f.height);
      camera.fov = (f.fovY * 180) / Math.PI;
      camera.updateProjectionMatrix();
      camera.position.set(0, 0, 0);
      camera.up.set(f.up[0], f.up[1], f.up[2]);
      camera.lookAt(f.forward[0], f.forward[1], f.forward[2]);
      const isTrue = f.look === "true";
      uSun.value.set(f.sun[0], f.sun[1], f.sun[2]);
      uSunRgb.value.set(f.sunRgb[0], f.sunRgb[1], f.sunRgb[2]);
      uTau.value.set(f.tau[0], f.tau[1], f.tau[2]);
      const thin = Math.exp(-Math.max(0, f.eye[2]) / Math.max(1, f.scale_height_m));
      uTauEye.value.set(f.tau[0] * thin, f.tau[1] * thin, f.tau[2] * thin);
      uH.value = Math.max(1, f.scale_height_m);
      uK.value = f.radius_m / Math.max(1, f.scale_height_m);
      uEyeZ.value = f.eye[2];
      uExp.value = f.exposure;
      uSunR.value = f.sun_radius;
      uC.value = Math.max(1, f.contour_m);
      uLow.value = 0;
      const sea = f.sea;
      uEyeXY.value.set(f.eye[0], f.eye[1], 0);
      const { trains } = trainsToDraw(sea?.optics ?? {}, sea ? standInTrains(sea.wave_m, sea.period_s, sea.dir_rad, sea.gravity_m_s2) : []);
      trains.forEach((t, i) => (uTrain[i]!.value.set(t.k, t.a, t.w, t.dir), (uTrainPh[i]!.value = t.phase)));
      const froth = sea?.optics ? frothToDraw(sea.optics, sea.dir_rad) : null;
      uFoamCover.value = froth?.cover ?? 0;
      uFoamAlb.value = froth?.albedo ?? 0;
      uFoamDir.value = froth?.dir ?? 0;
      uTime.value = sea?.time_s ?? 0;
      uTide.value = sea?.tide_m ?? 0;
      const op = sea?.optics;
      const hasBody = !!(op?.deep_rgb && op.absorb_per_m);
      uHasBody.value = hasBody ? 1 : 0;
      if (hasBody) (uDeep.value.set(...op!.deep_rgb!), uAbsorb.value.set(...op!.absorb_per_m!));
      uN.value = op?.refractive_index ?? 0;
      // Diagram looks pass the theme's display colours straight through; True is light, tone-mapped and encoded.
      bg.setStyle(f.ground, LinearSRGBColorSpace);
      ink.setStyle(f.ink, LinearSRGBColorSpace);
      uInk.value.set(ink.r, ink.g, ink.b);
      if (f.look !== lastLook) {
        lastLook = f.look;
        // True's tone mapping is the shared finish's (AgX, finish.ts); the diagram looks are drawn straight.
        renderer.toneMapping = NoToneMapping;
        renderer.outputColorSpace = isTrue ? SRGBColorSpace : LinearSRGBColorSpace;
        for (const m of allMats()) m.needsUpdate = true;
      }
      dome.visible = isTrue;
      // The stars: one pixel's solid angle sets how bright a point is for its light.
      if (f.stars && f.stars.key !== starKey) {
        starKey = f.stars.key;
        const pos = new Float32Array(f.stars.dirs.length);
        for (let i = 0; i < pos.length; i++) pos[i] = f.stars.dirs[i]! * 2.9e6;
        starGeom.setAttribute("position", new BufferAttribute(pos, 3));
        starGeom.setAttribute("light", new BufferAttribute(f.stars.light, 3));
        starGeom.computeBoundingSphere();
      }
      stars.visible = isTrue && !!f.stars;
      const pxAngle = f.fovY / Math.max(1, f.height);
      uStarScale.value = 1 / (pxAngle * pxAngle);
      stars.position.set(0, 0, 0);
      renderer.setClearColor(isTrue ? 0x000000 : bg, 1);
      for (const s of order) {
        const m = meshes.get(s.key)!;
        m.position.set(-f.eye[0], -f.eye[1], -f.eye[2]);
        const ms = matsFor(s.layer);
        m.material = isTrue ? ms.true : f.look === "wireframe" ? ms.wire : ms.chart;

      }
      // True through the shared finish (VP1): half-float light, the eye's glare (the sun's disc and glint bloom by
      // the CIE curve), AgX and dither; the exposure is already in the materials (`uExp`).
      if (isTrue) (finish ??= startFinish(renderer)).render(scene, camera, { exposure: 1, pixel_deg: (pxAngle * 180) / Math.PI });
      else renderer.render(scene, camera);
    },
    dispose() {
      finish?.dispose();
      dropAll();
      dome.geometry.dispose();
      starGeom.dispose();
      starMat.dispose();
      for (const m of allMats()) m.dispose();
      renderer.dispose();
    },
  };
}
