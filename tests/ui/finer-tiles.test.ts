// VP3 part 2: finer tiles are read four at a time, stop as soon as they are not wanted, are kept for the page, and
// are not asked for again.
import { test } from "node:test";
import assert from "node:assert/strict";
import { FINER_BATCH, finerTiles, readFiner } from "../../packages/views/body/src/finer-tiles.ts";
import type { SurfaceTile } from "../../packages/ui-contract/src/index.ts";

const fake = (refuseFace = -1) => {
  const asked: number[] = [];
  return {
    asked,
    session: {
      async surfaceTilesAt(_s: string, _b: string, keys: readonly { face: number; level: number; x: number; y: number }[]) {
        asked.push(keys.length);
        const tiles = keys.filter((k) => k.face !== refuseFace).map((k) => ({ key: { ...k }, samples: 32, spacing_m: 1, hash: "h" }) as unknown as SurfaceTile);
        return { ok: true as const, tiles };
      },
    },
  };
};

test("reads both levels four tiles at a time, keeps them, and asks nothing twice", async () => {
  const f = fake(5);
  let calls = 0;
  const n = await readFiner(f.session, "s", "a", () => true, () => calls++);
  assert.equal(n, 24 + 96 - 4 - 16);
  assert.ok(f.asked.every((k) => k <= FINER_BATCH));
  assert.equal(finerTiles("s|a").length, n);
  assert.ok(calls > 0);
  const again = fake(5);
  assert.equal(await readFiner(again.session, "s", "a", () => true, () => {}), 0);
  assert.equal(again.asked.length, 0, "refused and kept tiles are not asked again");
});

test("stops between requests once it is no longer wanted", async () => {
  const f = fake();
  let left = 3;
  const n = await readFiner(f.session, "s", "b", () => left-- > 0, () => {});
  assert.ok(n <= 2 * FINER_BATCH, `read ${n}`);
  assert.ok(f.asked.length <= 2);
});

test("the deep patch: read round the point at the level the pixels need, dropping to the fact floor a refusal names", async () => {
  const { readDeep } = await import("../../packages/views/body/src/finer-tiles.ts");
  const asked: { level: number }[][] = [];
  const S = 8;
  const session = {
    async surfaceTilesAt(_s: string, _b: string, keys: readonly { face: number; level: number; x: number; y: number }[]) {
      asked.push(keys.map((k) => ({ level: k.level })));
      const ok = keys.filter((k) => k.level <= 5);
      const tiles = ok.map((k) => ({ key: { ...k }, samples: S, spacing_m: 1, hash: "h", elevation_dm: new Array(S * S).fill(0), cover: new Array(S * S).fill(0), material: new Array(S * S).fill(0), albedo: new Array(S * S).fill(300) }) as unknown as SurfaceTile);
      const refused = keys.filter((k) => k.level > 5).map(() => ({ why: "BelowFactFloor { level: 9, floor_level: 5 }" }));
      return { ok: true as const, tiles, refused };
    },
  };
  const R = 1e6;
  // 10 m a pixel wants a level far below the floor; the patch comes back at the floor.
  const p = await readDeep(session, "s", "deep", { toward: [0.3, 0.4, 0.86], m_per_px: 10 }, R, S, () => true);
  assert.ok(p, "a patch");
  assert.equal(p!.level, 5);
  assert.ok(asked.flat().every((k) => k.level <= 30));
  assert.ok(asked.every((b) => b.length <= FINER_BATCH));
  // Asked again: the floor is kept and the tiles too, so nothing is read.
  const before = asked.length;
  const q = await readDeep(session, "s", "deep", { toward: [0.3, 0.4, 0.86], m_per_px: 10 }, R, S, () => true);
  assert.equal(q!.level, 5);
  assert.equal(asked.length, before);
  // From far off, the globe's faces do: no patch.
  assert.equal(await readDeep(session, "s", "deep", { toward: [1, 0, 0], m_per_px: 1e5 }, R, S, () => true), null);
});
