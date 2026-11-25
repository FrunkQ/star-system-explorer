// A world's shape in True (TA6): the kernel's Darwin-Radau flow on its own facts flattens Jupiter and Saturn by about
// their measured amounts; a world missing a fact is drawn round and says which; the drawn volume is kept.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { Session } from "../../ui-contract/src/session.ts";
import { gmSeat } from "../../ui-contract/src/seat.ts";
import { nodeKernel } from "../../ui-contract/test/node-kernel.ts";
import { flattenScale, readTrueShape, SHAPE_FLOW, type ShapeReader } from "../src/true-shape.ts";

test("the drawn ellipsoid keeps the sphere's volume and its pole is (1 - f) of its equator", () => {
  const s = flattenScale(0.0649);
  assert.ok(Math.abs(s.equator * s.equator * s.pole - 1) < 1e-12);
  assert.ok(Math.abs(s.pole / s.equator - (1 - 0.0649)) < 1e-12);
});

test("the kernel's flow flattens Jupiter and Saturn by about their measured amounts, from their facts", async () => {
  const k = nodeKernel();
  try {
    const base = JSON.parse(readFileSync(new URL("../../../packs/base-physics/pack.json", import.meta.url), "utf8"));
    const s = await Session.open(k.client, gmSeat(), { id: "shape", seed: "shape" }, [base]);
    // Facts as resolved_system gives them (SI), with the proven core's computed inertia factors (MOI1, 24 Nov).
    const facts: Record<string, Record<string, number>> = {
      jupiter: { "body.mass": 1.898e27, "body.radius": 6.9911e7, "body.rotation_period": 35730, "body.moment_of_inertia_factor": 0.254 },
      saturn: { "body.mass": 5.683e26, "body.radius": 5.8232e7, "body.rotation_period": 38362, "body.moment_of_inertia_factor": 0.21 },
      rock: { "body.mass": 6e24, "body.radius": 6.4e6, "body.rotation_period": 86164 },
    };
    const reader: ShapeReader = { resolvedSystem: async () => ({ ok: true, result: { facts } }), derive: (ref) => s.derive(ref as never) as never };
    const j = await readTrueShape(reader, "x", "jupiter");
    assert.ok(typeof j !== "string", String(j));
    // NASA's fact sheets: Jupiter 0.06487, Saturn 0.09796.
    assert.ok(Math.abs(j.f - 0.06487) < 0.005, `Jupiter ${j.f}`);
    const sa = await readTrueShape(reader, "x", "saturn");
    assert.ok(typeof sa !== "string" && Math.abs(sa.f - 0.09796) < 0.008, JSON.stringify(sa));
    const r = await readTrueShape(reader, "x", "rock");
    assert.equal(typeof r, "string");
    assert.match(r as string, /body\.moment_of_inertia_factor/);
    assert.equal(SHAPE_FLOW, "sse.base.body-shape");
  } finally {
    k.close?.();
  }
});
