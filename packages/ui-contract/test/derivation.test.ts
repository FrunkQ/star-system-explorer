// Scene assembly against the real kernel: derived values come from the
// kernel's flow, edits feed back in, and every derived fact can say why.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { Session } from "../src/session.ts";
import { gmSeat } from "../src/seat.ts";
import { applyEdits, deriveScene, sig, type DerivationDoc } from "../src/derivation.ts";
import type { SystemScene } from "../src/scene.ts";
import { nodeKernel } from "./node-kernel.ts";

const root = new URL("../../../", import.meta.url);
const json = (p: string) => JSON.parse(readFileSync(new URL(p, root), "utf8"));
const scene: SystemScene = json("packages/shell/src/documents/kestrel.scene.json");
const doc: DerivationDoc = json("packages/shell/src/documents/derivations.base.json");
const pack = json("packs/base-physics/pack.json");
const TARN = "u:kestrel/sys:1/body:3";

test("significant figures", () => {
  assert.equal(sig(281.43, 3), "281");
  assert.equal(sig(0.9134, 2), "0.91");
  assert.equal(sig(1271.6, 3), "1270");
});

test("derived facts come from the kernel and answer why", async () => {
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, gmSeat(), scene.universe, [pack]);
    assert.ok(s.flows().includes("sse.base.planet-climate"));
    const d = await deriveScene(s, scene, doc);
    const tarn = d.subjects.find((x) => x.address === TARN)!;
    const surface = tarn.facts.find((f) => f.field === "surface_k")!;
    assert.equal(surface.source, "derived");
    assert.ok(surface.why && surface.validity === "in-range");
    assert.ok(Number(surface.value) > 250 && Number(surface.value) < 320, `Tarn surface ${surface.display}`);
    if (process.env.SSE_PRINT) console.log("Tarn:", tarn.facts.filter((f) => f.source === "derived").map((f) => `${f.label} ${f.display} (${f.validity})`).join("; "));
    const star = d.subjects.find((x) => x.kind === "star")!;
    const vault = d.subjects.find((x) => x.label === "Vault")!;
    assert.equal(vault.facts.find((f) => f.field === "surface_k")!.source, "fixture", "no greenhouse surface for a gas giant");
    const e = await s.explain(surface.why!, surface.why!.kind);
    assert.ok(e.ok);
    if (e.ok) {
      assert.equal(e.explanation.subject, "planet.surface_temperature");
      assert.ok(e.explanation.lenses.curious && e.explanation.formula_display);
    }

    // Move Tarn out: colder and a longer year, through an intent.
    const r = await s.send({ kind: "set_value", address: TARN, field: "a_au", value: 0.9, at_yr: scene.now_yr });
    assert.equal(r.status, "applied");
    const moved = await deriveScene(s, applyEdits(scene, { [`${TARN}#a_au`]: { value: 0.9, display: "0.9 AU", event_id: "x" } }), doc);
    const t2 = moved.subjects.find((x) => x.address === TARN)!;
    assert.equal(t2.orbit!.a_au, 0.9);
    assert.ok(Number(t2.facts.find((f) => f.field === "surface_k")!.value) < Number(surface.value));
    assert.ok(Number(t2.facts.find((f) => f.field === "year_d")!.value) > Number(tarn.facts.find((f) => f.field === "year_d")!.value));
    assert.equal(t2.facts.find((f) => f.field === "a_au")!.source, "authored");
    assert.ok(t2.stamps.includes("authored"));
  } finally {
    await k.close();
  }
});
