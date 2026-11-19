// The gpu-basic tier of the relief and graticule passes: a cube-sphere mesh
// built from the surface tiles (each face a grid, the same mapping as the
// kernel's atlas), coloured and shaded from the same prepared numbers as the
// canvas tier, in one TSL shader that three.js compiles to WGSL on WebGPU and
// GLSL on WebGL2. The globe is unit-sized at the origin and the camera sits
// three radii away, so only small numbers reach the GPU (rendering rule 3).
// Shading is done in display (sRGB) values on purpose, as the canvas tier
// does, so the two tiers match; physically lit, linear HDR comes with R2.
import { BufferAttribute, BufferGeometry, LineSegments, LinearSRGBColorSpace, Matrix4, Mesh, OrthographicCamera, Scene, Vector3 } from "three";
import { LineBasicNodeMaterial, MeshBasicNodeMaterial, WebGPURenderer } from "three/webgpu";
import { dot, float, max, normalView, uniform, vec4, vertexColor } from "three/tsl";
import { cubeDirection } from "../../ui-contract/src/index.ts";
import { lightDir, lookup, viewRotation, type Prepared } from "./surface.ts";
import type { Plan } from "./recipe.ts";

export interface GlobeGpu {
  readonly backend: "webgpu" | "webgl2";
  draw(p: Prepared, plan: Plan, lon: number, lat: number, full: number): void;
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
    draw(p, plan, lon, lat, full) {
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
      uShade.value = num(relief.shade, 0.8);
      uAmb.value = num(relief.ambient, 0.16);
      const r = viewRotation(lon, lat).rows;
      const m = new Matrix4().set(r[0][0], r[0][1], r[0][2], 0, r[1][0], r[1][1], r[1][2], 0, r[2][0], r[2][1], r[2][2], 0, 0, 0, 0, 1);
      mesh!.matrix.copy(m);
      mesh!.matrixWorldNeedsUpdate = true;
      if (lines) (lines.matrix.copy(m), (lines.matrixWorldNeedsUpdate = true));
      const n = Math.max(16, Math.round(full * plan.render_scale));
      renderer.setPixelRatio(1);
      renderer.setSize(n, n, false);
      renderer.render(scene, camera);
    },
    dispose() {
      mesh?.geometry.dispose();
      lines?.geometry.dispose();
      material.dispose();
      lineMat.dispose();
      renderer.dispose();
    },
  };
}
