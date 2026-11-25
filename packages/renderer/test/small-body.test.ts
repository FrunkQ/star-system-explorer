// SB3: small bodies' figures as meshes.
import { test } from "node:test";
import assert from "node:assert/strict";
import { figureRadius, meanRadius, smallBodyColour, smallBodyMesh, type SmallFigure } from "../src/small-body.ts";

const itokawa: SmallFigure = { kind: "irregular", axes_m: [267, 147, 104], lobes: [], relief: 0.1, albedo: 0.25, weathering: 0.5 };
// Arrokoth-like: two flattened lobes touching, the larger on the left of the barycentre.
const arrokoth: SmallFigure = {
  kind: "two-lobed",
  axes_m: [17_500, 10_000, 4_900],
  lobes: [
    { axes_m: [10_300, 9_950, 4_700], offset_m: -4_000 },
    { axes_m: [7_700, 6_900, 4_900], offset_m: 11_000 },
  ],
  relief: 0.03,
};

function volume(m: { positions: Float32Array; index: Uint32Array }): number {
  let v = 0;
  const p = m.positions;
  for (let t = 0; t < m.index.length; t += 3) {
    const [a, b, c] = [m.index[t]! * 3, m.index[t + 1]! * 3, m.index[t + 2]! * 3];
    v += (p[a]! * (p[b + 1]! * p[c + 2]! - p[b + 2]! * p[c + 1]!) - p[a + 1]! * (p[b]! * p[c + 2]! - p[b + 2]! * p[c]!) + p[a + 2]! * (p[b]! * p[c + 1]! - p[b + 1]! * p[c]!)) / 6;
  }
  return v;
}

test("an ellipsoid's mesh keeps its axes and, in units of its mean radius, about a unit sphere's volume", () => {
  const f = { ...itokawa, relief: 0 };
  assert.ok(Math.abs(figureRadius(f, [1, 0, 0]) - 267) < 1e-6);
  assert.ok(Math.abs(figureRadius(f, [0, 0, 1]) - 104) < 1e-6);
  const m = smallBodyMesh(f, "u:t/sys:0/cloud:0/obj:9");
  const v = volume(m);
  assert.ok(v > 0, "triangles face outward");
  assert.ok(Math.abs(v / ((4 / 3) * Math.PI) - 1) < 0.02, `${v}`);
});

test("lumps are seeded by the address: the same body looks the same, another differs", () => {
  const a = smallBodyMesh(itokawa, "u:t/sys:0/cloud:0/obj:9");
  const b = smallBodyMesh({ ...itokawa }, "u:t/sys:0/cloud:0/obj:9");
  const c = smallBodyMesh(itokawa, "u:t/sys:0/cloud:0/obj:10");
  assert.deepEqual(Array.from(a.positions), Array.from(b.positions));
  assert.notDeepEqual(Array.from(a.positions), Array.from(c.positions));
});

test("a contact binary is two lobes with a neck between them", () => {
  const R = meanRadius(arrokoth);
  // Along the long axis each way the ray reaches the far end of its own lobe.
  assert.ok(Math.abs(figureRadius(arrokoth, [-1, 0, 0]) - 14_300) < 50);
  assert.ok(Math.abs(figureRadius(arrokoth, [1, 0, 0]) - 18_700) < 50);
  // The neck, sideways at the lobes' meeting point, is narrower than either lobe's width.
  const neckDir: [number, number, number] = [0.29, 0.957, 0];
  const n = Math.hypot(...neckDir);
  const neck = figureRadius(arrokoth, [neckDir[0] / n, neckDir[1] / n, 0]);
  assert.ok(neck < 9_950, `${neck}`);
  const m = smallBodyMesh(arrokoth, "u:t/sys:0/cloud:1/obj:3");
  assert.ok(volume(m) > 0);
  assert.ok(R > 0);
});

test("a dark body is drawn dark and a weathered one a little warm", () => {
  assert.equal(smallBodyColour({ ...itokawa, albedo: 0.05, weathering: 0 }), "rgb(26, 26, 26)");
  const [r, , b] = smallBodyColour(itokawa).match(/\d+/g)!.map(Number);
  assert.ok(r! > b!);
});

test("trueLook draws a body with a figure as that shape, a plain world as before", async () => {
  const { trueLook } = await import("../src/true-body.ts");
  const plain = trueLook({ key: "w", star: false, colour: "#888", pole: [0, 0, 1] });
  assert.equal(plain.mesh, undefined);
  const rock = trueLook({ key: "u:t/sys:0/cloud:0/obj:9", star: false, colour: "#888", pole: [0, 0, 1], figure: itokawa });
  assert.ok(rock.mesh && rock.mesh.index.length > 0);
  assert.notEqual(rock.colour, "#888");
});
