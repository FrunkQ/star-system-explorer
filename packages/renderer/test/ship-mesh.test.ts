// V3 gap item 14, stage 3: the 3D model is built from the same layout as the plans, so it must hold the same
// volumes and areas. Checks: each closed body's mesh volume (divergence theorem) matches the part it draws, the
// radiator faces add up to the radiator area the waste heat needs, normals are unit length and point outward,
// bounds match the layout, the mesh stays small enough to draw on a phone, and the panel texture is the same for
// the same seed and tiles without a seam.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { designShip, type DesignRules, type ShipStyle } from "../src/ship-design.ts";
import { panelTexture, shipMesh, type SurfaceBuffers } from "../src/ship-mesh.ts";
import type { Propellants, ShipRecord } from "../src/ship.ts";

const json = (p: string) => JSON.parse(readFileSync(new URL(p, import.meta.url), "utf8"));
const SHIPS = json("../../../packs/base-nav/pack.json").provides.other.nav.ships as Record<string, ShipRecord>;
const PACK = json("../../../packs/base-ships/pack.json").provides.other;
const RULES = PACK.ship_design as DesignRules;
const STYLES = PACK.ship_styles as Record<string, ShipStyle>;
// Test fixtures only: NIST saturated liquids at boiling.
const STORE: Propellants = {
  "hydrogen-oxygen": [
    { name: "liquid hydrogen", share: 1 / 7, kg_m3: 70.85, src: "nist-webbook" },
    { name: "liquid oxygen", share: 6 / 7, kg_m3: 1141, src: "nist-webbook" },
  ],
  hydrogen: [{ name: "liquid hydrogen", share: 1, kg_m3: 70.85, src: "nist-webbook" }],
  deuterium: [{ name: "liquid deuterium", share: 1, kg_m3: 163.5, src: "nist-webbook" }],
  "helium-3": [{ name: "liquid helium-3", share: 1, kg_m3: 59, src: "kerr1954" }],
};
const TORCH = designShip({ ...SHIPS["fusion-torch"]!, crew: 12, cargo_t: 200 }, STORE, RULES, "spine", STYLES.spine!, 3);
const CHEM = designShip({ ...SHIPS["chemical-transfer"]!, crew: 4, cargo_t: 20 }, STORE, RULES, "sleek", STYLES.sleek!, 2);

function triangles(b: SurfaceBuffers, f: (a: number[], c: number[], d: number[]) => void) {
  const v = (i: number) => [b.positions[3 * i]!, b.positions[3 * i + 1]!, b.positions[3 * i + 2]!];
  for (let k = 0; k < b.indices.length; k += 3) f(v(b.indices[k]!), v(b.indices[k + 1]!), v(b.indices[k + 2]!));
}
const cross = (a: number[], b: number[]) => [a[1]! * b[2]! - a[2]! * b[1]!, a[2]! * b[0]! - a[0]! * b[2]!, a[0]! * b[1]! - a[1]! * b[0]!];
const sub = (a: number[], b: number[]) => a.map((x, i) => x - b[i]!);
const dot = (a: number[], b: number[]) => a.reduce((s, x, i) => s + x * b[i]!, 0);

test("the model holds the same volumes as the plans", () => {
  const n = 512;
  for (const l of [TORCH, CHEM]) {
    const m = shipMesh(l, { segments: n, panel_m: 2 });
    // A polygon of n sides holds n sin(2 pi / n) / (2 pi) of its circle.
    const polygon = (n * Math.sin((2 * Math.PI) / n)) / (2 * Math.PI);
    const want = l.pieces
      .filter((p) => p.shape === "body" && ["tanks", "crew", "cargo", "sensors", "drive"].includes(p.system))
      .reduce((a, p) => a + (Math.PI / 3) * (p.x1 - p.x0) * (p.r0 * p.r0 + p.r0 * p.r1 + p.r1 * p.r1), 0);
    let got = 0;
    for (const s of ["hull", "tanks", "drive"] as const) if (m.surfaces[s]) triangles(m.surfaces[s]!, (a, b, c) => (got += dot(a, cross(b, c)) / 6));
    assert.ok(Math.abs(got / (want * polygon) - 1) < 1e-6, `${l.name}: ${got} against ${want * polygon}`);
  }
});

test("radiator faces add up to the area the waste heat needs, and normals point outward", () => {
  const m = shipMesh(TORCH, { segments: 24, panel_m: 2 });
  let area = 0;
  triangles(m.surfaces.radiators!, (a, b, c) => (area += Math.hypot(...cross(sub(b, a), sub(c, a))) / 2));
  // Two faces of each wing; the needed area counts one face.
  assert.ok(Math.abs(area / 2 / TORCH.needs.radiator_m2 - 1) < 1e-6, `${area / 2} against ${TORCH.needs.radiator_m2}`);
  for (const b of Object.values(m.surfaces)) {
    for (let k = 0; k < b.normals.length; k += 3) assert.ok(Math.abs(Math.hypot(b.normals[k]!, b.normals[k + 1]!, b.normals[k + 2]!) - 1) < 1e-5);
    // Each triangle's winding agrees with its vertices' normal.
    for (let k = 0; k < b.indices.length; k += 3) {
      const [i, j, h] = [b.indices[k]!, b.indices[k + 1]!, b.indices[k + 2]!];
      const p = (q: number) => [b.positions[3 * q]!, b.positions[3 * q + 1]!, b.positions[3 * q + 2]!];
      const face = cross(sub(p(j), p(i)), sub(p(h), p(i)));
      if (Math.hypot(...face) < 1e-9) continue;
      assert.ok(dot(face, [b.normals[3 * i]!, b.normals[3 * i + 1]!, b.normals[3 * i + 2]!]) > 0, "winding against the normal");
    }
  }
});

test("bounds match the layout and the mesh stays small", () => {
  for (const l of [TORCH, CHEM]) {
    const m = shipMesh(l, { segments: 24, panel_m: 2 });
    assert.ok(Math.abs(m.bounds[1][0] - m.bounds[0][0] - l.length_m) < 1e-3 * l.length_m, `${l.name} length`);
    assert.ok(Math.abs(m.bounds[1][1] - m.bounds[0][1] - l.across_m) < 1e-3 * l.across_m, `${l.name} across`);
    assert.ok(m.triangles < 5000, `${m.triangles} triangles`);
    assert.ok(m.hull_bounds[1][1] - m.hull_bounds[0][1] <= m.bounds[1][1] - m.bounds[0][1]);
  }
});

test("the panel texture is the same for a seed, differs between seeds and tiles without a seam", () => {
  const skin = { patchiness: 0.5, wear: 0.4 };
  const a = panelTexture(7, 64, skin);
  assert.deepEqual(panelTexture(7, 64, skin), a);
  assert.notDeepEqual(panelTexture(8, 64, skin), a);
  // Seams run along row 0 and column 0 only, so the right edge meets the left edge's seam: the tile repeats cleanly.
  const at = (x: number, y: number) => a[4 * (y * 64 + x)]!;
  for (let y = 1; y < 64; y++) if (y % 16) assert.ok(at(0, y) < at(1, y) || at(0, y) < 140, `seam at row ${y}`);
});
