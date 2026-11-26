// The `light-path` stage on the GPU (XO4): a black hole in True, every pixel's light followed back through the
// kernel's bend table. The expressions are light-path.ts's, written in TSL so WebGPU and WebGL2 run the same shader;
// light-path.ts is the reference the e2e and unit checks hold this to. Per pixel, near the hole:
//
// 1. its ray's impact parameter b and its angle α round the picture (the disc's axis drawn upright);
// 2. the disc: at the crossing angles φ₀ = π/2 + atan(cos α tan i) and φ₀ + π, the table's u = r_g / r; the first
//    crossing inside the disc's edges is what shows (order 0 the direct image and the arch, order 1 the thin ring),
//    a ray that escapes or is swallowed before a crossing shows no disc;
// 3. there: the shift g and the ring's T⁴, one bilinear read of the disc table the kernel baked (`sse.light.disc`,
//    from the block in force `sse.light.disc_shift` and `sse_light::disc_temperature`; ER11a) at r and
//    b sin i sin α / r, and the seen temperature gT, whose colour or band brightness is one read of the band table
//    (Planck through the sheet's bands and the eye's CIE curves, light-path.ts `bandTable`);
// 4. otherwise the sky: an escaping ray turned by the table's turn (4 r_g / b beyond 64 r_g), looked up where it came
//    from; a swallowed ray is the shadow.
//
// The picture is linear light into the shared finish (VP1). Visible is natural colour, dimmed so the disc's
// brightest point reads white (the note says by how much); every other band is its own brightness, mapped; Composite
// lays the bands in their colours, each scaled to its own brightest (XO17). Presentation only: nothing here reaches
// the kernel or the state hash.
import { DataTexture, FloatType, HalfFloatType, LinearFilter, Mesh, NearestFilter, OrthographicCamera, PlaneGeometry, RedFormat, RGBAFormat, Scene, Vector2, Vector3, Vector4, DataUtils, ClampToEdgeWrapping, RepeatWrapping } from "three";
import { MeshBasicNodeMaterial, WebGPURenderer } from "three/webgpu";
import { Fn, If, abs, asin, atan, clamp, cos, dot, exp2, float, floor, int, ivec2, log, max, min, mix, normalize, select, sin, sqrt, tan, texture, textureLoad, uniform, uv, vec2, vec3, vec4 } from "three/tsl";
import { startFinish, type Finish } from "./finish.ts";
import { B_CRIT, B_MAX, CAPTURED_ROWS, COLS, LOG_T0, LOG_T1, PHI_MAX, PICTURE_BANDS, RAMP, ROW_LENGTH, ROWS, U_HORIZON, bandTable, type BendTable, type Disc, type HoleView, type PictureBand } from "./light-path.ts";
import { channelOf, rgbaOf } from "../../ui-contract/src/index.ts";

/** One band's layer in the picture: its colour (linear 0..1; visible's is "natural"), and log10 of its brightest. */
export interface HoleLayer {
  band: PictureBand;
  colour: [number, number, number] | "natural";
  /** log10 of the band's brightest mean radiance in this view (light-path.ts `brightest`): it is shown at 1. */
  top: number;
}

/** The sky behind, as the light-path stage looks it up: a fine patch round the hole and the whole sky coarsely. */
export interface HoleSky {
  key: string;
  /** A gnomonic patch centred on the hole's direction: RGBA linear light, `size` square, spanning ±`half` (tan). */
  patch: { data: Float32Array; size: number; half: number };
  /** The whole sky, equirectangular (longitude across from −π, latitude down from +π/2), in the camera's frame. */
  whole: { data: Float32Array; width: number; height: number };
}

export interface HoleFrame {
  table: BendTable;
  disc: Disc | null;
  view: HoleView;
  layers: HoleLayer[];
  sky: HoleSky | null;
  /** The sky's gain (long exposure lifts the stars, as VP2); 0 hides it. */
  skyGain: number;
  /** Image orders drawn: 2 with the thin ring, 1 without (the degrade order's second step). */
  orders: 1 | 2;
  width: number;
  height: number;
  /** Drawn at this fraction of the canvas's pixels and enlarged (the tier's knob). */
  renderScale?: number;
}

export interface HolePicture {
  readonly backend: "webgpu" | "webgl2";
  draw(f: HoleFrame): void;
  dispose(): void;
}

const half = (a: Float32Array) => {
  const out = new Uint16Array(a.length);
  for (let i = 0; i < a.length; i++) out[i] = DataUtils.toHalfFloat(Math.min(65000, a[i]!));
  return out;
};

const LAYERS = PICTURE_BANDS.length;



/** The picture on `canvas`, or null where this device has no GPU for True. */
export async function startHolePicture(canvas: HTMLCanvasElement): Promise<HolePicture | null> {
  let renderer: WebGPURenderer;
  try {
    renderer = new WebGPURenderer({ canvas, antialias: false, alpha: false });
    await renderer.init();
  } catch {
    return null;
  }
  const backend = (renderer.backend as { isWebGPUBackend?: boolean }).isWebGPUBackend ? "webgpu" : "webgl2";
  const finish: Finish = startFinish(renderer);

  // The bend table: one float per texel, a row per impact parameter (b, captured, end angle, turn, then the cells).
  const tableTex = new DataTexture(new Float32Array(ROW_LENGTH * ROWS), ROW_LENGTH, ROWS, RedFormat, FloatType);
  tableTex.magFilter = tableTex.minFilter = NearestFilter;
  let tableHash = "";
  // The disc table the kernel baked (sse.table/1): RGBA texels, r across (log), b sin i sin α / r down (even); the
  // shift and the ring's T⁴ in the channels the table names. Read at four texels and mixed, as the kernel samples it.
  const discTex = new DataTexture(new Float32Array(16), 2, 2, RGBAFormat, FloatType);
  discTex.magFilter = discTex.minFilter = NearestFilter;
  let discHash = "";
  // The band table: log10 radiance by seen temperature (exact from Planck; read at two texels and mixed).
  const bandTex = new DataTexture(bandTable(), RAMP, 2, RGBAFormat, FloatType);
  bandTex.magFilter = bandTex.minFilter = NearestFilter;
  bandTex.needsUpdate = true;
  const blank = () => {
    const t = new DataTexture(new Uint16Array(4), 1, 1, RGBAFormat, HalfFloatType);
    t.magFilter = t.minFilter = LinearFilter;
    t.needsUpdate = true;
    return t;
  };
  let patchTex: DataTexture = blank();
  let wholeTex: DataTexture = blank();
  let skyKey = "";

  const uD = uniform(1000);
  const uTilt = uniform(1.2);
  const uHalf = uniform(30);
  const uTurning = uniform(1);
  const uAspect = uniform(1);
  const uDisc = uniform(new Vector3(6, 1000, 0)); // r_in, r_out, has disc
  const uDiscAxes = uniform(new Vector4(0, 1, -1, 1)); // ln r first and last node, b sin i sin α / r first and last
  const uDiscN = uniform(new Vector2(1, 1)); // nodes along each
  const uDiscCh = uniform(new Vector2(0, 1)); // which texel channel holds the shift, which T⁴
  const uOrders = uniform(2);
  const uSkyGain = uniform(0);
  const uPatchHalf = uniform(0.05);
  const uTop = Array.from({ length: LAYERS }, () => uniform(0));
  const uCol = Array.from({ length: LAYERS }, () => uniform(new Vector3()));
  const uNatural = uniform(0);
  const patchNode = texture(patchTex);
  const wholeNode = texture(wholeTex);

  /* eslint-disable @typescript-eslint/no-explicit-any */
  type N = any;
  // u on a ray at a fractional row and angle (BendTable::u_at; the rows never mix across √27).
  const rowOf = (b: N): N => {
    const below = b.lessThan(B_CRIT);
    const t = float(1).sub(sqrt(max(float(1).sub(b.div(B_CRIT)), 0)));
    const cap = min(t.mul(CAPTURED_ROWS), CAPTURED_ROWS - 1);
    const s = sqrt(max(b.sub(B_CRIT).div(B_MAX - B_CRIT), 0));
    const esc = max(s.mul(ROWS - CAPTURED_ROWS).sub(1).add(CAPTURED_ROWS), CAPTURED_ROWS);
    return select(below, cap, esc);
  };
  const cellAt = (i: N, k: N): N => textureLoad(tableTex, ivec2(int(k), int(i))).x;
  const rowPair = (at: N): [N, N, N] => {
    const i0 = floor(at);
    const next = i0.add(1);
    const same = next.lessThan(ROWS).and(next.lessThan(CAPTURED_ROWS).or(i0.greaterThanEqual(CAPTURED_ROWS)));
    return [i0, select(same, next, i0), at.sub(i0)];
  };
  const uAt = (b: N, phi: N): N => {
    const [i0, i1, fr] = rowPair(rowOf(b));
    const ck = phi.div(PHI_MAX).mul(COLS - 1);
    const k0 = min(floor(ck), COLS - 2);
    const fc = ck.sub(k0);
    const cell = (i: N) => {
      const a = cellAt(i, k0.add(4));
      return a.add(cellAt(i, k0.add(5)).sub(a).mul(fc));
    };
    const c0 = cell(i0);
    return c0.add(cell(i1).sub(c0).mul(fr));
  };
  const turnOf = (b: N): N => {
    const [i0, i1, fr] = rowPair(rowOf(b));
    const a = cellAt(i0, float(3));
    return select(b.greaterThan(B_MAX), float(4).div(b), a.add(cellAt(i1, float(3)).sub(a).mul(fr)));
  };
  // A band's brightness at a seen temperature, relative to its brightest: 10^(log − top), from two texels.
  const bandAt = (logT: N) => {
    const x = clamp(logT.sub(LOG_T0).div(LOG_T1 - LOG_T0).mul(RAMP - 1), 0, RAMP - 1);
    const i0 = floor(x);
    const i1 = min(i0.add(1), RAMP - 1);
    const f = x.sub(i0);
    const row = (y: number) => mix(textureLoad(bandTex, ivec2(int(i0), int(y))), textureLoad(bandTex, ivec2(int(i1), int(y))), f);
    return [row(0), row(1)] as [N, N];
  };
  // The disc table at (r, q): the shift and T⁴, bilinear between the four nodes round it (sse_physics::table::sample).
  const discAt = (r: N, q: N): [N, N] => {
    const fx = clamp(log(r).sub(uDiscAxes.x).div(uDiscAxes.y.sub(uDiscAxes.x)).mul(uDiscN.x.sub(1)), 0, uDiscN.x.sub(1));
    const fy = clamp(q.sub(uDiscAxes.z).div(uDiscAxes.w.sub(uDiscAxes.z)).mul(uDiscN.y.sub(1)), 0, uDiscN.y.sub(1));
    const x0 = min(floor(fx), uDiscN.x.sub(2));
    const y0 = min(floor(fy), uDiscN.y.sub(2));
    const tx = fx.sub(x0);
    const ty = fy.sub(y0);
    const at = (dx: number, dy: number) => textureLoad(discTex, ivec2(int(x0.add(dx)), int(y0.add(dy))));
    const v = mix(mix(at(0, 0), at(1, 0), tx), mix(at(0, 1), at(1, 1), tx), ty);
    const pick = (c: N) => select(c.lessThan(0.5), v.x, select(c.lessThan(1.5), v.y, select(c.lessThan(2.5), v.z, v.w)));
    return [pick(uDiscCh.x), pick(uDiscCh.y)];
  };
  const LOG2_10 = Math.log2(10);
  // Kept finite: an absent layer has a huge top (0 here), and an infinity times a zero colour would be NaN.
  const tenTo = (v: N) => exp2(min(v, 30).mul(LOG2_10));

  const material = new MeshBasicNodeMaterial({ depthTest: false, depthWrite: false });
  material.colorNode = Fn(() => {
    const p = uv().mul(2).sub(1);
    const px = p.x.mul(uAspect).mul(uHalf);
    const py = p.y.mul(uHalf);
    const rho = vec2(px, py).length();
    const theta = atan(rho.div(uD));
    const b = uD.mul(sin(theta));
    const alpha = atan(px.negate(), py).mul(uTurning);
    const out = vec3(0, 0, 0).toVar();
    const onDisc = float(0).toVar();
    const rHit = float(0).toVar();
    // The disc (disc_image::landing): order 0, then order 1; a ray that leaves the table's picture, escapes or is
    // swallowed before a crossing stops there.
    const live = uDisc.z.greaterThan(0.5).toVar();
    const phi0 = float(Math.PI / 2).add(atan(cos(alpha).mul(tan(uTilt))));
    for (let n = 0; n < 2; n++) {
      If(live.and(onDisc.lessThan(0.5)).and(uOrders.greaterThan(n)), () => {
        const phi = phi0.add(n * Math.PI);
        const u = (n === 0 ? select(b.greaterThan(B_MAX), sin(phi).div(b), uAt(b, phi)) : uAt(b, phi)).toVar();
        If(b.greaterThan(B_MAX).and(float(n).greaterThan(0)), () => {
          u.assign(-1);
        });
        If(u.lessThanEqual(0).or(u.greaterThanEqual(U_HORIZON * 0.999)), () => {
          live.assign(false);
        }).Else(() => {
          const r = float(1).div(u);
          If(r.greaterThanEqual(uDisc.x).and(r.lessThanEqual(uDisc.y)), () => {
            onDisc.assign(1);
            rHit.assign(r);
          });
        });
      });
    }
    If(onDisc.greaterThan(0.5), () => {
      const r = rHit;
      const [gRead, t4Read] = discAt(r, b.mul(sin(uTilt)).mul(sin(alpha)).div(r));
      const g = max(gRead, 0);
      const t4 = max(t4Read, 0);
      // log10(g T): T = t4^(1/4).
      const logT = max(t4, 1e-30).log2().mul(0.25 / LOG2_10).add(max(g, 1e-30).log2().div(LOG2_10));
      const [vis, rest] = bandAt(logT);
      const radiance = (k: number): N => (k === 0 ? vis.w : k === 1 ? rest.x : k === 2 ? rest.y : k === 3 ? rest.z : rest.w);
      const lit = g.greaterThan(0).and(t4.greaterThan(0));
      // Visible in its natural colour (each channel against the brightest's visible band), the others mapped.
      const natural = vec3(tenTo(vis.x.sub(uTop[0]!)), tenTo(vis.y.sub(uTop[0]!)), tenTo(vis.z.sub(uTop[0]!)));
      const sum = natural.mul(uNatural).toVar();
      for (let k = 0; k < LAYERS; k++) {
        const v = tenTo(radiance(k).sub(uTop[k]!));
        sum.addAssign(uCol[k]!.mul(v));
      }
      out.assign(select(lit, sum, vec3(0, 0, 0)));
    }).Else(() => {
      // The sky, or the shadow.
      If(b.greaterThanEqual(B_CRIT).and(uSkyGain.greaterThan(0)), () => {
        const tau = turnOf(b);
        const a = theta.sub(tau);
        // In the camera's frame: forward −z, right x, up y; ê the picture direction of this pixel.
        const e = normalize(vec2(px, py).add(vec2(1e-9, 0)));
        const s = vec3(e.x.mul(sin(a)), e.y.mul(sin(a)), cos(a).negate());
        const inPatch = s.z.lessThan(0);
        const g2 = vec2(s.x, s.y).div(max(s.z.negate(), 1e-6));
        const puv = g2.div(uPatchHalf).mul(0.5).add(0.5);
        const inside = inPatch.and(abs(g2.x).lessThan(uPatchHalf)).and(abs(g2.y).lessThan(uPatchHalf));
        const lon = atan(s.x, s.z.negate());
        const lat = asin(clamp(s.y, -1, 1));
        const wuv = vec2(lon.div(2 * Math.PI).add(0.5), lat.div(Math.PI).add(0.5));
        const fine = patchNode.sample(puv).xyz;
        const coarse = wholeNode.sample(wuv).xyz;
        out.assign(select(inside, fine, coarse).mul(uSkyGain));
      });
    });
    return vec4(out, 1);
  })();
  /* eslint-enable @typescript-eslint/no-explicit-any */

  const scene = new Scene();
  const quad = new PlaneGeometry(2, 2);
  scene.add(new Mesh(quad, material));
  const camera = new OrthographicCamera(-1, 1, 1, -1, 0, 1);
  const size = new Vector2();

  const upload = (f: HoleFrame) => {
    const dt = f.disc?.table;
    if (dt && dt.hash !== discHash) {
      const [ax, ay] = [dt.axes[0]!, dt.axes[1]!];
      // A new size needs the old upload let go first; the shader keeps the same texture object.
      discTex.dispose();
      discTex.image = { data: rgbaOf(dt), width: ax.n, height: ay.n };
      discTex.needsUpdate = true;
      uDiscAxes.value.set(Math.log(ax.from), Math.log(ax.to), ay.from, ay.to);
      uDiscN.value.set(ax.n, ay.n);
      uDiscCh.value.set(channelOf(dt, "shift"), channelOf(dt, "t4"));
      discHash = dt.hash;
    }
    if (f.table.hash !== tableHash) {
      const d = tableTex.image.data as Float32Array;
      for (let i = 0; i < d.length; i++) d[i] = f.table.flat[i] ?? 0;
      tableTex.needsUpdate = true;
      tableHash = f.table.hash;
    }
    const key = f.sky?.key ?? "";
    if (key !== skyKey) {
      skyKey = key;
      patchTex.dispose();
      wholeTex.dispose();
      if (f.sky) {
        patchTex = new DataTexture(half(f.sky.patch.data), f.sky.patch.size, f.sky.patch.size, RGBAFormat, HalfFloatType);
        wholeTex = new DataTexture(half(f.sky.whole.data), f.sky.whole.width, f.sky.whole.height, RGBAFormat, HalfFloatType);
        for (const t of [patchTex, wholeTex]) {
          t.magFilter = t.minFilter = LinearFilter;
          t.wrapT = ClampToEdgeWrapping;
          t.needsUpdate = true;
        }
        patchTex.wrapS = ClampToEdgeWrapping;
        wholeTex.wrapS = RepeatWrapping;
        uPatchHalf.value = f.sky.patch.half;
      } else {
        patchTex = blank();
        wholeTex = blank();
      }
      patchNode.value = patchTex;
      wholeNode.value = wholeTex;
    }
  };

  return {
    backend,
    draw(f) {
      upload(f);
      const w = Math.max(1, Math.round(f.width));
      const h = Math.max(1, Math.round(f.height));
      renderer.getSize(size);
      if (size.x !== w || size.y !== h) renderer.setSize(w, h, false);
      uD.value = f.view.distance;
      uTilt.value = Math.min(Math.max(f.view.tilt, 0), Math.PI / 2 - 1e-4);
      uHalf.value = f.view.half;
      uTurning.value = f.view.turning;
      uAspect.value = w / h;
      uOrders.value = f.orders;
      uSkyGain.value = f.sky ? f.skyGain : 0;
      if (f.disc) {
        uDisc.value.set(f.disc.rIn, f.disc.rOut, 1);
      } else uDisc.value.set(6, 6, 0);
      uNatural.value = 0;
      PICTURE_BANDS.forEach((b, k) => {
        const l = f.layers.find((x) => x.band === b);
        uTop[k]!.value = l?.top ?? 1e4;
        if (l && l.colour === "natural") {
          uNatural.value = 1;
          uTop[0]!.value = l.top;
          uCol[k]!.value.set(0, 0, 0);
        } else if (l) {
          const c = l.colour as [number, number, number];
          uCol[k]!.value.set(c[0], c[1], c[2]);
        }
        else uCol[k]!.value.set(0, 0, 0);
      });
      finish.render(scene, camera, { exposure: 1, pixel_deg: (2 * Math.atan(f.view.half / f.view.distance) * 180) / Math.PI / h, glare: false, render_scale: f.renderScale ?? 1 });
    },
    dispose() {
      finish.dispose();
      material.dispose();
      quad.dispose();
      tableTex.dispose();
      discTex.dispose();
      bandTex.dispose();
      patchTex.dispose();
      wholeTex.dispose();
      renderer.dispose();
    },
  };
}
