// The gpu-basic tier of the relief and graticule passes: a cube-sphere mesh
// built from the surface tiles (each face a grid, the same mapping as the
// kernel's atlas), coloured and shaded from the same prepared numbers as the
// canvas tier, in one TSL shader that three.js compiles to WGSL on WebGPU and
// GLSL on WebGL2. The globe is unit-sized at the origin and the camera sits
// three radii away, so only small numbers reach the GPU (rendering rule 3).
// Two lightings: "display" shades in display (sRGB) values, as the canvas
// tier does, so the two tiers match; "physical" (R2) lights linear albedo by
// the star through the air (Kasten and Young 1989 air mass for the sun's
// transmittance at the ground), adds the atmosphere pass (gpu-air.ts) and
// tone-maps the result.
import { ACESFilmicToneMapping, BufferAttribute, BufferGeometry, LineSegments, LinearSRGBColorSpace, Matrix4, Mesh, NoToneMapping, OrthographicCamera, Scene, SphereGeometry, SRGBColorSpace, Vector3 } from "three";
import { LineBasicNodeMaterial, MeshBasicNodeMaterial, WebGPURenderer } from "three/webgpu";
import { acos, attribute, clamp, dot, exp, float, max, normalView, normalize, pow, positionWorld, select, uniform, vec3, vec4, vertexColor } from "three/tsl";
import { makeAirMaterial, type AirRender } from "./gpu-air.ts";
import { cubeDirection } from "../../ui-contract/src/index.ts";
import { lightDir, lookup, viewRotation, type Prepared } from "./surface.ts";
import type { Plan } from "./recipe.ts";

export interface GlobeGpu {
  readonly backend: "webgpu" | "webgl2";
  /** Draw one frame. `jitter` shifts the view by a fraction of a pixel (the still tier averages jittered frames). */
  draw(p: Prepared, plan: Plan, lon: number, lat: number, full: number, air?: AirRender | null, jitter?: [number, number]): void;
  dispose(): void;
}

const num = (v: unknown, d: number) => (typeof v === "number" ? v : d);

export async function createGlobeGpu(canvas: HTMLCanvasElement): Promise<GlobeGpu | null> {
  let renderer: WebGPURenderer;
  try {
    renderer = new WebGPURenderer({ canvas, antialias: true, alpha: true });
    await renderer.init();
  } catch {
    return null;
  }
  renderer.outputColorSpace = LinearSRGBColorSpace;
  renderer.setClearColor(0x000000, 0);
  const backend = (renderer.backend as { isWebGPUBackend?: boolean }).isWebGPUBackend ? "webgpu" : "webgl2";
  const scene = new Scene();
  const camera = new OrthographicCamera(-1, 1, 1, -1, 0.1, 10);
  camera.position.set(0, 0, 3);
  camera.lookAt(0, 0, 0);

  const uL = uniform(new Vector3(0, 0, 1));
  const uShade = uniform(0.8);
  const uAmb = uniform(0.16);
  const material = new MeshBasicNodeMaterial();
  // lit = 1 - shade + shade * (ambient + (1 - ambient) * max(0, n·L)), as surface.ts `lit`.
  const ndl = max(float(0), dot(normalView, uL));
  const k = float(1).sub(uShade).add(uShade.mul(uAmb.add(float(1).sub(uAmb).mul(ndl))));
  material.colorNode = vec4(vertexColor().rgb.mul(k), 1);

  // Physical lighting: linear albedo × max(0, n·L) × the sun's transmittance through the air at that ground point / pi.
  const uBeta = uniform(new Vector3(0, 0, 0));
  const uH = uniform(0.0013);
  const uSun = uniform(1);
  const uNight = uniform(0.002);
  const physical = new MeshBasicNodeMaterial();
  {
    // Hue from the palette, brightness from the kernel's albedo layer (a physical reflectance, 0 to 1).
    const lin = pow(vertexColor().rgb, vec3(2.2, 2.2, 2.2));
    const lum = max(dot(lin, vec3(0.2126, 0.7152, 0.0722)), 0.01);
    const albedo = clamp(lin.mul(attribute("albedo", "float").div(lum)), 0, 1);
    const up = normalize(positionWorld);
    const cosz = clamp(dot(up, uL), -1, 1);
    const zdeg = acos(cosz).mul(180 / Math.PI);
    // Kasten and Young (1989): air mass = 1 / (cos z + 0.50572 (96.07995 - z)^-1.6364), z in degrees.
    const airmass = float(1).div(cosz.add(float(0.50572).mul(pow(max(float(96.07995).sub(zdeg), 0.5), -1.6364))));
    const tsun = select(zdeg.lessThan(95), exp(uBeta.mul(uH).mul(airmass).negate()), vec3(0, 0, 0));
    const ndlp = max(float(0), dot(normalView, uL));
    physical.colorNode = vec4(albedo.mul(tsun.mul(ndlp).mul(uSun).div(Math.PI).add(uNight)), 1);
  }
  let airMat: ReturnType<typeof makeAirMaterial> | null = null;
  const shell = new Mesh(new SphereGeometry(1, 128, 64));
  shell.visible = false;
  shell.renderOrder = 2;
  const lineMat = new LineBasicNodeMaterial({ transparent: true });
  const uLineA = uniform(0.3);
  lineMat.colorNode = vec4(1, 1, 1, uLineA);

  let mesh: Mesh | null = null;
  let lines: LineSegments | null = null;
  let built: { p: Prepared; grid: number } | null = null;
  let builtLines: number | null = null;

  function buildMesh(p: Prepared, grid: number) {
    const G = grid;
    const per = (G + 1) * (G + 1);
    const pos = new Float32Array(6 * per * 3);
    const col = new Float32Array(6 * per * 3);
    const nor = new Float32Array(6 * per * 3);
    const alb = new Float32Array(6 * per);
    const idx: number[] = [];
    for (let f = 0; f < 6; f++) {
      for (let j = 0; j <= G; j++) {
        for (let i = 0; i <= G; i++) {
          const u = -1 + (2 * i) / G;
          const v = -1 + (2 * j) / G;
          const d = cubeDirection(f, u, v);
          const o = (f * per + j * (G + 1) + i) * 3;
          pos.set(d, o);
          // Nudge inward so edge vertices take their own face's sample.
          const hit = lookup(p, cubeDirection(f, u * 0.9999, v * 0.9999));
          if (hit) {
            col[o] = hit.f.rgb[hit.k * 3]! / 255;
            col[o + 1] = hit.f.rgb[hit.k * 3 + 1]! / 255;
            col[o + 2] = hit.f.rgb[hit.k * 3 + 2]! / 255;
            nor.set(hit.f.normal.subarray(hit.k * 3, hit.k * 3 + 3), o);
            alb[o / 3] = hit.f.albedo[hit.k]!;
          } else {
            col.fill(40 / 255, o, o + 3);
            nor.set(d, o);
          }
        }
      }
      for (let j = 0; j < G; j++) {
        for (let i = 0; i < G; i++) {
          const a = f * per + j * (G + 1) + i;
          const b = a + 1;
          const c = a + G + 1;
          const e = c + 1;
          idx.push(a, b, e, a, e, c);
        }
      }
    }
    const g = new BufferGeometry();
    g.setAttribute("position", new BufferAttribute(pos, 3));
    g.setAttribute("color", new BufferAttribute(col, 3));
    g.setAttribute("normal", new BufferAttribute(nor, 3));
    g.setAttribute("albedo", new BufferAttribute(alb, 1));
    g.setIndex(idx);
    if (mesh) (scene.remove(mesh), mesh.geometry.dispose());
    mesh = new Mesh(g, material);
    mesh.matrixAutoUpdate = false;
    scene.add(mesh);
    built = { p, grid };
  }

  function buildLines(spacing: number) {
    const pts: number[] = [];
    const r = 1.003;
    const seg = 96;
    const at = (la: number, lo: number) => [r * Math.cos(la) * Math.cos(lo), r * Math.cos(la) * Math.sin(lo), r * Math.sin(la)];
    const s = (spacing * Math.PI) / 180;
    for (let la = -Math.PI / 2 + s; la < Math.PI / 2 - 1e-6; la += s)
      for (let q = 0; q < seg; q++) pts.push(...at(la, (q / seg) * 2 * Math.PI), ...at(la, ((q + 1) / seg) * 2 * Math.PI));
    for (let lo = 0; lo < 2 * Math.PI - 1e-6; lo += s)
      for (let q = 0; q < seg / 2; q++) pts.push(...at(-Math.PI / 2 + (q / (seg / 2)) * Math.PI, lo), ...at(-Math.PI / 2 + ((q + 1) / (seg / 2)) * Math.PI, lo));
    const g = new BufferGeometry();
    g.setAttribute("position", new BufferAttribute(new Float32Array(pts), 3));
    if (lines) (scene.remove(lines), lines.geometry.dispose());
    lines = new LineSegments(g, lineMat);
    lines.matrixAutoUpdate = false;
    scene.add(lines);
    builtLines = spacing;
  }

  return {
    backend,
    draw(p, plan, lon, lat, full, air, jitter) {
      const relief = plan.passes.find((x) => x.pass === "relief-globe")?.params ?? {};
      const grat = plan.passes.find((x) => x.pass === "graticule")?.params;
      const grid = Math.round(num(relief.grid, 128));
      if (!built || built.p !== p || built.grid !== grid) buildMesh(p, grid);
      if (grat) {
        if (builtLines !== num(grat.spacing_deg, 30)) buildLines(num(grat.spacing_deg, 30));
        uLineA.value = num(grat.opacity, 0.3);
        lines!.visible = true;
      } else if (lines) lines.visible = false;
      const L = lightDir(num(relief.sun_azimuth_deg, 315), num(relief.sun_elevation_deg, 40));
      uL.value.set(L[0], L[1], L[2]);
      const isPhysical = relief.lighting === "physical";
      const atm = plan.passes.find((x) => x.pass === "atmosphere")?.params;
      const sun = num(atm?.sun, 6);
      mesh!.material = isPhysical ? physical : material;
      renderer.toneMapping = isPhysical ? ACESFilmicToneMapping : NoToneMapping;
      renderer.toneMappingExposure = num(atm?.exposure, 1);
      renderer.outputColorSpace = isPhysical ? SRGBColorSpace : LinearSRGBColorSpace;
      if (air) {
        uBeta.value.set(air.beta[0], air.beta[1], air.beta[2]);
        uH.value = air.scale_height;
      } else uBeta.value.set(0, 0, 0);
      uSun.value = sun;
      if (atm && air) {
        const steps = Math.round(num(atm.steps, 16));
        const lsteps = Math.round(num(atm.light_steps, 6));
        if (!airMat || airMat.steps !== steps || airMat.lightSteps !== lsteps) {
          airMat?.material.dispose();
          airMat = makeAirMaterial(steps, lsteps);
          shell.material = airMat.material;
          if (!shell.parent) scene.add(shell);
        }
        airMat.set(air, L, sun);
        shell.scale.setScalar(air.top);
        shell.visible = true;
      } else shell.visible = false;
      uShade.value = num(relief.shade, 0.8);
      uAmb.value = num(relief.ambient, 0.16);
      const r = viewRotation(lon, lat).rows;
      const m = new Matrix4().set(r[0][0], r[0][1], r[0][2], 0, r[1][0], r[1][1], r[1][2], 0, r[2][0], r[2][1], r[2][2], 0, 0, 0, 0, 1);
      mesh!.matrix.copy(m);
      mesh!.matrixWorldNeedsUpdate = true;
      if (lines) (lines.matrix.copy(m), (lines.matrixWorldNeedsUpdate = true));
      // Leave room for the air's limb when it is drawn; otherwise the globe fills the view, as on the canvas tier.
      const half = shell.visible && air ? Math.max(1, air.top * 1.01) : 1;
      if (camera.right !== half) {
        camera.left = camera.bottom = -half;
        camera.right = camera.top = half;
        camera.updateProjectionMatrix();
      }
      const n = Math.max(16, Math.round(full * plan.render_scale));
      if (jitter) camera.setViewOffset(n, n, jitter[0], jitter[1], n, n);
      else if (camera.view?.enabled) camera.clearViewOffset();
      renderer.setPixelRatio(1);
      renderer.setSize(n, n, false);
      renderer.render(scene, camera);
    },
    dispose() {
      mesh?.geometry.dispose();
      lines?.geometry.dispose();
      material.dispose();
      physical.dispose();
      airMat?.material.dispose();
      shell.geometry.dispose();
      lineMat.dispose();
      renderer.dispose();
    },
  };
}
