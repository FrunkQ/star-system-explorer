// U7 part 2, the LifeFlow board: the kernel's life_view sorted into the pipeline's stages, placed as a flow.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { Session } from "../src/session.ts";
import { gmSeat } from "../src/seat.ts";
import { LIFE_EXAMPLES, type LifeView } from "../src/life.ts";
import { STAGES, lifeFlow, placeStages, stageOfEvent } from "../src/life-flow.ts";
import type { PackDoc } from "../src/physics.ts";
import { nodeKernel } from "./node-kernel.ts";

const root = new URL("../../../", import.meta.url);
const base: PackDoc = JSON.parse(readFileSync(new URL("packs/base-physics/pack.json", root), "utf8"));

test("every example world's life sorts into the pipeline and places as a flow", async () => {
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, gmSeat(), { id: "kestrel", seed: "kestrel reach campaign" }, [base], k.scratch);
    const views: Record<string, LifeView> = {};
    for (const e of LIFE_EXAMPLES) {
      const r = await s.lifeView({ fixture: e.fixture });
      assert.ok(r.ok);
      views[e.fixture] = r.view;
    }
    for (const [f, v] of Object.entries(views)) {
      const st = lifeFlow(v);
      assert.deepEqual(
        st.map((x) => x.id),
        STAGES.map((x) => x.id),
        f,
      );
      // Every dated event lands in exactly one stage, and every dial on at least one.
      assert.equal(st.reduce((n, x) => n + (x.id === "hooks" ? 0 : x.events.length), 0), v.events.length, `${f}: events`);
      for (const d of Object.keys(v.dials)) assert.ok(st.some((x) => x.dials.some((y) => y.id === d)), `${f}: dial ${d}`);
      const p = placeStages(st, { w: 100, h: 50, gapX: 20, gapY: 10 });
      assert.equal(p.placed.length, st.length);
      for (const w of p.wires) {
        const a = p.placed.find((x) => x.stage.id === w.from)!;
        const b = p.placed.find((x) => x.stage.id === w.to)!;
        assert.ok(a.col < b.col, `${f}: ${w.from} flows forward into ${w.to}`);
      }
      for (const x of p.placed) assert.ok(x.x >= 0 && x.y >= 0 && x.x + 100 <= p.width + 1e-9 && x.y + 50 <= p.height + 1e-9, `${f}: ${x.stage.id} inside`);
    }

    const earth = lifeFlow(views.earth!);
    const at = (id: string) => earth.find((x) => x.id === id)!;
    assert.ok(earth.every((x) => x.id === "hooks" || x.live), "on Earth every stage but the hooks has something");
    assert.match(at("origin").headline, /^began at /);
    assert.deepEqual(
      at("origin").dials.map((d) => d.id),
      ["origin"],
    );
    assert.ok(at("innovations").dials.some((d) => d.id === "oxygenic"));
    assert.ok(at("innovations").events.some((e) => e.subject === "oxygenic"));
    assert.ok(at("catastrophes").events.some((e) => e.kind === "catastrophe"));
    assert.match(at("signatures").headline, /candidate, 99%/);
    assert.match(at("fluxes").headline, /O₂ 21%/);
    assert.ok(at("habitats").lines.some((l) => /soil/.test(l)));

    const europa = lifeFlow(views.europa!);
    assert.equal(europa.find((x) => x.id === "signatures")!.headline, "blind from outside");
    assert.ok(europa.find((x) => x.id === "hooks")!.live, "hidden life is a hook");
    assert.equal(europa.find((x) => x.id === "opportunity")!.live, false, "no timeline kept");

    const desert = lifeFlow(views["red-dwarf-desert"]!);
    assert.equal(desert.find((x) => x.id === "ecosystem")!.live, false, "a false alarm has no ecosystem");
    assert.equal(desert.find((x) => x.id === "origin")!.headline, "never began");
  } finally {
    await k.close();
  }
});

test("events go to the stage that made them", () => {
  assert.equal(stageOfEvent({ kind: "origin" }), "origin");
  assert.equal(stageOfEvent({ kind: "capability" }), "innovations");
  assert.equal(stageOfEvent({ kind: "capability_lost" }), "catastrophes");
  assert.equal(stageOfEvent({ kind: "recovery" }), "catastrophes");
  assert.equal(stageOfEvent({ kind: "sterilisation" }), "catastrophes");
});
