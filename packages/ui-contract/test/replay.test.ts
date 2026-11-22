import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { Session } from "../src/session.ts";
import { gmSeat } from "../src/seat.ts";
import { replayAt, replayModel } from "../src/replay.ts";
import type { FormationRan } from "../src/make.ts";
import { nodeKernel } from "./node-kernel.ts";

const text = (p: string) => readFileSync(new URL("../../../" + p, import.meta.url), "utf8");

test("a grown system replays from its own record: disc, growth, the violent phase, leftovers", async () => {
  const k = nodeKernel();
  try {
    const packs = [JSON.parse(text("packs/base-physics/pack.json")), JSON.parse(text("packs/base-time/pack.json"))];
    const s = await Session.open(k.client, gmSeat(), { id: "kestrel", seed: "k" }, packs);
    for (const seed of ["first light", "cold harbour"]) {
      const r = await s.runFormation({ seed });
      assert.ok(r.ok, seed);
      const run: FormationRan = r.result;
      const m = replayModel(run);
      const again = replayModel(run);
      assert.deepEqual(JSON.stringify(again), JSON.stringify(m), "the same run gives the same replay");
      assert.deepEqual(m.stages.map((x) => x.n).sort((a, b) => a - b), [1, 2, 3, 4, 5, 6, 7, 8, 9], `${seed}: all nine stages`);
      assert.ok(m.stages.every((x, i) => i === 0 || x.from_myr >= m.stages[i - 1]!.from_myr), "stages in time order");
      assert.ok(m.beats.length > 40 && m.beats.every((b, i) => i === 0 || b > m.beats[i - 1]!), "beats increase");

      // Early: the disc with its condensation lines, moving inward; no worlds yet.
      const early = replayAt(m, 0.02);
      assert.ok(early.disc && early.disc.lines.length === 5 && early.disc.opacity > 0.3, `${seed}: the disc at 20 kyr`);
      assert.equal(early.bodies.length, 0);
      const later = replayAt(m, m.disc.t_disperse * 0.9);
      assert.ok(later.disc!.lines[2]! < early.disc.lines[2]!, "the water ice line creeps inward as the disc cools");
      // Growth: embryos appear and gain mass from their ledgers.
      const g = replayAt(m, m.disc.t_disperse * 0.95);
      assert.ok(g.bodies.length >= 3, `${seed}: embryos growing in the disc`);
      const first = m.bodies[0]!;
      assert.ok(replayAt(m, m.disc.t_disperse).bodies.find((b) => b.id === first.id)!.m >= replayAt(m, first.born_myr * 1.01).bodies.find((b) => b.id === first.id)!.m);
      // After the disc clears, no disc.
      assert.equal(replayAt(m, m.disc.t_disperse * 1.2).disc, null);
      // The end: the survivors are the run's worlds, where the run put them.
      const end = replayAt(m, m.t1);
      assert.equal(end.bodies.length, run.run.worlds.length, `${seed}: survivors are the run's worlds`);
      const final = (run.run as unknown as { upheaval: { worlds: { id: string; a_au: number }[] } }).upheaval.worlds;
      for (const w of final) assert.ok(Math.abs(end.bodies.find((b) => b.id === w.id)!.a_au - w.a_au) < 1e-9, `${w.id} ends at its orbit`);
      assert.equal(end.stage.n, 9);
      assert.ok(m.events.some((e) => e.kind.startsWith("impact")), `${seed}: giant impacts in words`);
      // Total mass at the end matches the run's worlds (M⊕), within rounding.
      const mEnd = end.bodies.reduce((a, b) => a + b.m, 0);
      const mRun = run.run.worlds.reduce((a, w) => a + w.mass_earth, 0);
      assert.ok(Math.abs(mEnd - mRun) / mRun < 0.15, `${seed}: mass ${mEnd} against ${mRun}`);
    }
    // A rogue planet leaves: first light throws two out.
    const fl = await s.runFormation({ seed: "first light" });
    assert.ok(fl.ok);
    const m = replayModel(fl.result);
    const rogue = m.bodies.find((b) => b.gone?.how === "ejected")!;
    assert.ok(rogue && m.events.some((e) => e.kind === "ejected" && /rogue planet/.test(e.text)));
    assert.ok(!replayAt(m, rogue.gone!.t_myr * 1.01).bodies.some((b) => b.id === rogue.id));
  } finally {
    await k.close();
  }
});
