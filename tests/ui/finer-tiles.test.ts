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
