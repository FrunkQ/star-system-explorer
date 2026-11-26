// V14i part 2: the class gallery's pre-drawn globes (packs/base-presentation/class-pictures.json). The set must read, and
// every shipped atlas class has a globe or says why not. Whether it was drawn with today's look is reported, not failed:
// a renderer change should not turn every PR red, and an out-of-date set only means the gallery draws live until
// `node scripts/build-class-pictures.mjs` is run again (best on a machine with a real GPU).
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { lookFingerprint } from "../../scripts/class-pictures-look.mjs";

const root = new URL("../../", import.meta.url);
const set = JSON.parse(readFileSync(new URL("packs/base-presentation/class-pictures.json", root), "utf8"));

test("the pre-drawn class globes read, one for each shipped class or a reason", () => {
  assert.equal(set.schema, "sse.class-pictures/1");
  const atlas = JSON.parse(readFileSync(new URL("packs/base-atlas/pack.json", root), "utf8"));
  const shipped = set.atlases.find((a: { pack: string }) => a.pack === atlas.id);
  assert.ok(shipped, "the base atlas is drawn");
  for (const [id, p] of Object.entries(shipped.pictures) as [string, { sheet?: string; caption?: string; none?: string }][]) {
    if (p.none !== undefined) assert.ok(p.none.length > 10, `${id}: says why it has no globe`);
    else {
      assert.match(p.sheet ?? "", /^data:image\/webp;base64,/, `${id}: a WebP strip`);
      assert.match(p.caption ?? "", /after /, `${id}: its caption names the world it was made after`);
    }
  }
  assert.ok(Object.values(shipped.pictures).filter((p) => (p as { sheet?: string }).sheet).length >= 20, "most classes with ground have a globe (giants and Venus-likes wait on weather for a sample world)");
});

test("the pre-drawn globes' look, reported", () => {
  const now = lookFingerprint(new URL(".", root).pathname);
  if (set.look === now) console.log(`class globes: drawn with today's look (${now})`);
  else console.log(`class globes: drawn with look ${set.look}, today's is ${now}. The gallery draws live until node scripts/build-class-pictures.mjs is run again.`);
});
