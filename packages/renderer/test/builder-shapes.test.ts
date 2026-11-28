// The builder's parts as shapes (CM14c): every part on the base palette draws as at least one solid piece that stays
// inside the room its own size gives it, where its place puts it; a panel reaches straight out along its clock; a tank
// has domed ends (narrower at its ends than its middle); the name goes on the biggest crew or tank part.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { Box3, Mesh, Vector3 } from "three";
import { placeBuild, type BuildRecipe } from "../src/ship-build.ts";
import { nameCarrier, partShapes } from "../src/builder-shapes.ts";
import type { PartsPalette } from "../../ui-contract/src/index.ts";

const PALETTE = JSON.parse(readFileSync(new URL("../../../packs/base-parts/pack.json", import.meta.url), "utf8")).provides.other.palette as PartsPalette;
const box = (q: Parameters<typeof partShapes>[0]) => {
  const b = new Box3();
  for (const p of partShapes(q, 16)) b.expandByObject(new Mesh(p.geometry));
  return b;
};

test("every palette part draws as solid pieces within the reach its size gives it", () => {
  for (const part of PALETTE.parts) {
    const r: BuildRecipe = { format: "sse.build/1", name: "one", parts: [{ key: "a", part: part.id }] };
    const { placed } = placeBuild(r, PALETTE);
    const pieces = partShapes(placed[0]!, 16);
    assert.ok(pieces.length > 0, `${part.id} draws`);
    const b = box(placed[0]!);
    const s = part.shape;
    const reach = Math.max(s.length_m, s.radius_m * 6, (s.span_m ?? 0) * 1.2) + 1;
    for (const v of [b.min, b.max]) for (const c of v.toArray()) assert.ok(Number.isFinite(c) && Math.abs(c) <= reach + s.length_m, `${part.id} stays near its own size (${c})`);
    assert.ok(b.max.x - b.min.x > 0.5 * s.length_m - 0.05 || part.kind === "dish", `${part.id} spans most of its length`);
  }
});

test("a panel on the hull reaches straight out along its clock", () => {
  for (const [clock, dir] of [[90, new Vector3(0, 1, 0)], [0, new Vector3(0, 0, 1)], [270, new Vector3(0, -1, 0)]] as const) {
    const r: BuildRecipe = { format: "sse.build/1", name: "wing", parts: [{ key: "hab", part: "habitat-4m" }, { key: "w", part: "solar-panel", on: "hab", place: "hull", along: 0.5, clock_deg: clock }] };
    const { placed } = placeBuild(r, PALETTE);
    const b = box(placed[1]!);
    const c = b.getCenter(new Vector3());
    const size = b.getSize(new Vector3());
    assert.ok(c.clone().setX(0).normalize().dot(dir) > 0.95, `clock ${clock}: centre ${c.toArray()} lies out along ${dir.toArray()}`);
    assert.ok(Math.abs(size.dot(dir)) > 9, `clock ${clock}: spans its 10 m out (${size.toArray()})`);
  }
});

test("a tank has domed ends and the name goes on the biggest crew or tank part", () => {
  const r: BuildRecipe = { format: "sse.build/1", name: "pair", parts: [{ key: "t", part: "tank-4m" }, { key: "h", part: "habitat-10m", on: "t", place: "front" }, { key: "s", part: "tank-2m", on: "t", place: "hull", clock_deg: 90 }] };
  const { placed } = placeBuild(r, PALETTE);
  const g = partShapes(placed[0]!, 24)[0]!.geometry;
  const pos = g.getAttribute("position");
  const L = placed[0]!.part.shape.length_m;
  let endR = 0;
  let midR = 0;
  for (let i = 0; i < pos.count; i++) {
    const x = pos.getX(i) - placed[0]!.x0;
    const r2 = Math.hypot(pos.getY(i), pos.getZ(i));
    if (x < 0.02 * L || x > 0.98 * L) endR = Math.max(endR, r2);
    midR = Math.max(midR, r2);
  }
  assert.ok(endR < 0.7 * midR, `ends ${endR} m narrower than its widest ${midR} m`);
  assert.equal(nameCarrier(placed)?.key, "h");
});
