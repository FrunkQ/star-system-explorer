// SC1 in Node: the star class gallery's stars. Each kind's star is read off the life line (describe_star) and its
// surface from read_activity, with no system and nothing written; a remnant with no picture yet says so; the sizes
// switch scales by the largest on the page; the atlas's star scheme becomes tiles with its samples.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { Session } from "../src/session.ts";
import { gmSeat } from "../src/seat.ts";
import { nodeKernel } from "./node-kernel.ts";
import { radiiOnPage, readGalleryStar, shareWords, STAND_IN, tilesFromScheme } from "../../views/reference/src/star-gallery.ts";

const root = new URL("../../../", import.meta.url);
const text = (p: string) => readFileSync(new URL(p, root), "utf8");
const packs = [JSON.parse(text("packs/base-physics/pack.json")), JSON.parse(text("packs/base-time/pack.json"))];

test("every stand-in kind is the kind its tile says, and the universe does not move", async () => {
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, gmSeat(), { id: "kestrel", seed: "k" }, packs);
    const before = await s.stateHash();
    const phase: Record<string, string> = { "pre-main-sequence": "pre-main-sequence", "main-sequence": "main-sequence", subgiant: "hertzsprung-gap", giant: "giant", supergiant: "giant", "brown-dwarf": "brown-dwarf", "white-dwarf": "white-dwarf", "neutron-star": "neutron-star", "black-hole": "black-hole" };
    const got = new Map<string, Awaited<ReturnType<typeof readGalleryStar>>>();
    for (const t of STAND_IN) {
      const g = await readGalleryStar(s, t);
      assert.ok(typeof g !== "string", `${t.id}: ${g}`);
      assert.equal(g.desc.now.phase, phase[t.family], t.id);
      got.set(t.id, g);
      // Remnants still to be drawn by the exotic objects renderer are never given a stand-in disc.
      if (t.family === "neutron-star" || t.family === "black-hole") assert.ok(g.live === null && g.why, t.id);
      else assert.ok(g.live, `${t.id} is drawn`);
      assert.ok(g.desc.age_yr <= 1.4e10, `${t.id} is no older than the universe`);
    }
    const rsg = got.get("stand-in/supergiant/M") as Exclude<Awaited<ReturnType<typeof readGalleryStar>>, string>;
    const sun = got.get("stand-in/main-sequence/G") as typeof rsg;
    // The supergiant's cells are few and huge (low gravity, a tall scale height); the Sun's are a thousand across.
    assert.ok(rsg.cells && rsg.cells.across < 40 && rsg.cells.big && rsg.cells.big.count < 10, JSON.stringify(rsg.cells));
    assert.ok(sun.cells && sun.cells.across > 500, JSON.stringify(sun.cells));
    // A hot star has no cells: it turns on its own clock instead.
    const o = got.get("stand-in/main-sequence/O") as typeof rsg;
    assert.equal(o.cells, null);
    assert.match(o.clockWhy, /turn/);
    assert.equal(await s.stateHash(), before, "looking never changes the universe");
  } finally {
    k.close();
  }
});

test("Actual size scales by the largest on the page; a star too small for a pixel is a point that says how small", () => {
  const r = radiiOnPage([1, 10, 1000, 0], 50, "actual");
  assert.equal(r[2]!.r_px, 50);
  assert.ok(r[1]!.r_px === 1 && r[1]!.point, "half a pixel is drawn as one, and called a point");
  assert.ok(r[0]!.point && r[0]!.share === 0.001);
  assert.ok(radiiOnPage([1, 1000], 50, "same").every((x) => x.r_px === 50));
  assert.equal(shareWords(0.001), "1/1,000 of the largest");
  assert.equal(shareWords(1), "the largest here");
  assert.equal(shareWords(0.5), "50% of the largest");
});

test("the atlas's star scheme becomes tiles: kinds under their families, each with its sample from its example", () => {
  const cls = (id: string, parent?: string, values?: Record<string, number>) => ({ id, scheme: "star", kind: "base", ...(parent ? { parent } : {}), names: { physics: id, plain: id }, summary: values ? "" : "No star of this kind on the life line.", region: [], gate: [], needs: [], examples: values ? [{ name: "sample", in: true, values }] : [], pack: "sse.base-atlas" });
  const tiles = tilesFromScheme(
    { classes: [cls("star/main-sequence"), cls("star/main-sequence/G", "star/main-sequence", { "star.sample_mass_msun": 1, "star.sample_age_yr": 4.6e9, "star.sample_feh": 0 }), cls("star/neutron-star")] as never },
    (c) => c.names.plain,
  );
  assert.deepEqual(tiles.map((t) => t.id), ["star/main-sequence/G", "star/neutron-star"]);
  assert.equal(tiles[0]!.family, "main-sequence");
  assert.deepEqual(tiles[0]!.sample, { mass_msun: 1, age_yr: 4.6e9, feh: 0 });
  assert.equal(tiles[1]!.family, "neutron-star");
  assert.equal(tiles[1]!.sample, null);
  assert.match(tiles[1]!.why!, /life line/);
});
