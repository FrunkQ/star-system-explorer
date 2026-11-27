// SC1 follow-up in Node: one star's life. Its stages are the life line's own, whatever the mass, each read at ages
// through it with a surface to draw; a remnant still to be drawn says so; the playhead's radius moves smoothly.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { Session } from "../src/session.ts";
import { gmSeat } from "../src/seat.ts";
import { nodeKernel } from "./node-kernel.ts";
import { AU, FRAMES_PER_STAGE, frameAt, largestRadius, radiusAt, readLife, sceneScale, stageAges } from "../../views/reference/src/star-life.ts";

const root = new URL("../../../", import.meta.url);
const text = (p: string) => readFileSync(new URL(p, root), "utf8");
const packs = [JSON.parse(text("packs/base-physics/pack.json")), JSON.parse(text("packs/base-time/pack.json"))];

test("a stage's ages stay inside it, closer together towards its end; the last stage's spread in log time", () => {
  const a = stageAges(100, 200, 10);
  assert.equal(a.length, 10);
  assert.ok(a.every((x) => x > 100 && x < 200));
  assert.ok(a[9]! - a[8]! < a[1]! - a[0]!, "closer towards the end");
  const last = stageAges(1e9, null, 10);
  assert.ok(last[0]! > 1e9 && last[9]! < 1e9 + 1e10 && last[9]! - last[8]! > 10 * (last[1]! - last[0]!));
});

test("the Sun's life is the life line's stages in order, drawn through each; a heavy star's ends in a remnant not drawn yet", async () => {
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, gmSeat(), { id: "kestrel", seed: "k" }, packs);
    const before = await s.stateHash();
    const sun = await readLife(s, 1);
    assert.ok(typeof sun !== "string", String(sun));
    const d = await s.describeStar!(1, 1e6);
    assert.ok(d.ok);
    // The stages are the life line's own, never a list of ours.
    assert.deepEqual(sun.stages.map((x) => x.phase), d.star.phases.map((p) => p.phase));
    for (const st of sun.stages) {
      assert.equal(st.frames.length, FRAMES_PER_STAGE);
      assert.ok(st.frames.every((f) => f.now.phase === st.phase), `${st.phase}: ${st.frames.map((f) => f.now.phase).join(",")}`);
      assert.ok(st.frames.every((f) => f.live), `${st.phase} is drawn`);
    }
    // The Sun's giant tip is near Earth's orbit (absolute: between half and twice it); its white dwarf is Earth-sized.
    const big = largestRadius(sun);
    assert.ok(big > 0.5 * AU && big < 2 * AU, `largest ${big / AU} AU`);
    const wd = sun.stages.at(-1)!;
    assert.equal(wd.phase, "white-dwarf");
    assert.ok(wd.frames[0]!.now.radius_rsun < 0.03, String(wd.frames[0]!.now.radius_rsun));
    // Between frames the radius moves in log, inside the two frames' sizes.
    const r0 = sun.stages[3]!.frames[4]!.now.radius_m;
    const r1 = sun.stages[3]!.frames[5]!.now.radius_m;
    const mid = radiusAt(sun, 3 + 4.5 / FRAMES_PER_STAGE);
    assert.ok(mid >= Math.min(r0, r1) && mid <= Math.max(r0, r1));
    assert.deepEqual(frameAt(sun, 3 + 4.5 / FRAMES_PER_STAGE), { stage: 3, frame: 4, between: frameAt(sun, 3 + 4.5 / FRAMES_PER_STAGE).between });
    // No blend across a stage's end: the giant's last frame holds its size up to the white dwarf.
    const g = sun.stages.findIndex((x) => x.phase === "giant");
    assert.equal(radiusAt(sun, g + 0.999), sun.stages[g]!.frames.at(-1)!.now.radius_m);
    // The whole life's scale fits its largest size; Earth's orbit's fits 1 AU.
    assert.ok(Math.abs(sceneScale(sun, 100, "earth") * AU - 100) < 1e-9);
    assert.ok(Math.abs(sceneScale(sun, 100, "life") * Math.max(AU, big) - 100) < 1e-9);

    const heavy = await readLife(s, 40);
    assert.ok(typeof heavy !== "string", String(heavy));
    const end = heavy.stages.at(-1)!;
    assert.equal(end.phase, heavy.remnant.phase);
    assert.ok(end.why && end.frames.every((f) => f.live === null), "a remnant still to be drawn has no stand-in disc");
    assert.equal(await s.stateHash(), before, "reading a life writes nothing");
  } finally {
    k.close();
  }
});
