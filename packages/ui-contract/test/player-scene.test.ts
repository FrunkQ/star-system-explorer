// V1: player views draw K9 scenes through a vocabulary layer. The scenes
// here are K9's own verified outputs (sse-observe asserts equality with
// them), so these checks hold for the kernel op when it is wired.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { checkPlayerScene, describeFact, fact, parentOf, sentence, subjectName, wordsFor, type PlayerScene, type Vocab } from "../src/player-scene.ts";
import { checkScreen, type ScreenDoc } from "../src/screen.ts";
import type { PlayerContext, ViewModule } from "../src/view.ts";

const root = new URL("../../../", import.meta.url);
const json = (p: string) => JSON.parse(readFileSync(new URL(p, root), "utf8"));
const before: PlayerScene = json("tests/fixtures/observe/tarn-before.json");
const after: PlayerScene = json("tests/fixtures/observe/tarn-after.json");
const vocab: Vocab = json("packages/shell/src/documents/vocab.plain.json");
const TARN = "u:kestrel/sys:0/body:3";

test("K9 scenes are drawable scenes", () => {
  assert.equal(checkPlayerScene(before), null);
  assert.equal(checkPlayerScene(after), null);
  assert.match(checkPlayerScene({ audience: "x", scope: "u:a", subjects: [{ address: "u:b", view: "v", facts: [], tags: [] }] })!, /outside the scope/);
});

test("facts read in the plain vocabulary: values, bands, placeholders", () => {
  const tarn = after.subjects.find((s) => s.address === TARN)!;
  assert.equal(subjectName(tarn), "Tarn");
  const d = (f: string) => describeFact(vocab, fact(tarn, f)!);
  assert.deepEqual(d("surface.temp_mean_k"), { state: "band", label: "Mean temperature", text: "270 to 290 K", number: 280 });
  assert.equal(d("orbit.semi_major_axis_m").text, "0.63 AU");
  assert.equal(d("mass_kg").text, "1.1 × Earth");
  assert.equal(d("ocean.fraction").text, "71 %");
  assert.equal(d("orbit.period_s").text, "214 days");
  assert.equal(d("hazards").text, "storm belt, tides");
  assert.equal(d("kind").text, "planet");
  assert.deepEqual(d("resources"), { state: "unknown", label: "Resources", text: "survey required" });
  const pre = before.subjects.find((s) => s.address === TARN)!;
  assert.equal(describeFact(vocab, fact(pre, "mass_kg")!).text, "not yet scanned", "never a guessed number");
  assert.equal(sentence(vocab, pre, fact(pre, "orbit.semi_major_axis_m")!), "Its orbit: not yet scanned.");
  assert.equal(sentence(vocab, tarn, fact(tarn, "ocean.fraction")!), "Oceans cover 71 % of it.");
  assert.equal(wordsFor(vocab, "interior.mantle").label, "Interior", "prefix entries");
  assert.equal(wordsFor(vocab, "magnetosphere.field_t").label, "field t", "unlisted fields still read");
  assert.equal(parentOf(`${TARN}/body:0`, before.scope), TARN);
  assert.equal(parentOf(TARN, before.scope), null);
});

test("a new view is a module in any package: no kernel change (V1 acceptance)", () => {
  // Defined here, outside /crates and outside the view packages.
  const gauge: ViewModule<PlayerScene, PlayerContext> = {
    id: "example.view.ocean-gauge",
    version: "0.0.1",
    scene: "player",
    tiers: ["text"],
    inputs: [{ quantity_kinds: [] }],
    params: [{ name: "field", type: "string", default: "ocean.fraction", meaning: "Field." }, { name: "range", type: "list", default: [0, 1], meaning: "Ends." }],
    mount: () => ({ update() {}, dispose() {} }),
  };
  const doc: ScreenDoc = { schema: "sse.screen/1", id: "x", label: "X", audience: "players", layout: "grid", theme: "chart-room", symbology: "s", vocab: "plain", items: [{ view: gauge.id, area: "a", params: { range: [0, 2] } }] };
  const c = checkScreen(doc, [gauge]);
  assert.ok(c.ok);
  assert.deepEqual(c.screen.mounts[0]!.params, { field: "ocean.fraction", range: [0, 2] });
  const bad = checkScreen({ ...doc, items: [{ view: gauge.id, area: "a", params: { range: "wide" } }] }, [gauge]);
  assert.ok(!bad.ok && /list/.test(bad.errors.join()));
});
