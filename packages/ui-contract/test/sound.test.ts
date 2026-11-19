// N1 acceptance in Node: sound scenes, scores and recipes. A scene's score is
// identical every time it is built (and its fingerprint is pinned here, so
// the browser test can show another engine agrees); a recipe outside its
// schema is rejected; players hear only bodies whose period their data view
// gives them; sound is off unless explicitly turned on.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { Session } from "../src/session.ts";
import { gmSeat } from "../src/seat.ts";
import { enterFacts, type AuthoredFacts } from "../src/data-views.ts";
import type { PackDoc } from "../src/physics.ts";
import type { SystemScene } from "../src/scene.ts";
import { bandFor, checkRecipe, dialled, intoOctave, orbitScene, readSound, score, scoreHash, settle, voicesFromPlayers, voicesFromSystem } from "../src/sound.ts";
import { nodeKernel } from "./node-kernel.ts";

const root = new URL("../../../", import.meta.url);
const json = (p: string) => JSON.parse(readFileSync(new URL(p, root), "utf8"));
const kestrel: SystemScene = json("packages/shell/src/documents/kestrel.scene.json");
const recipe = json("packages/sound/src/recipes/orbit-tones.json");


test("periods become pitches by whole octaves, exactly", () => {
  const year = 365.25 * 86400;
  const r = intoOctave(1 / year, 110);
  assert.ok(r.freq >= 110 && r.freq < 220);
  assert.equal(r.octaves, 32);
  // Octaves are exact in binary: raising twice the frequency lands one octave lower in the count, same pitch.
  assert.equal(intoOctave(2 / year, 110).freq, r.freq);
  assert.equal(bandFor(300), 55);
  assert.equal(bandFor(undefined), 110);
  // A 3:2 resonance is a perfect fifth whatever the dial; settle pulls 1.49 to 1.5.
  assert.equal(settle(150, 100), 150);
  assert.equal(settle(149, 100), 150);
  assert.equal(dialled(149, 100, 0), 149);
  assert.equal(dialled(149, 100, 0.5), 149.5);
});

test("Kestrel's planets sound, moons stay in their own frame", () => {
  const v = voicesFromSystem(kestrel);
  assert.deepEqual(
    v.map((x) => x.name),
    ["Ember", "Sable", "Tarn", "Vault", "Hollow"],
  );
  // Kepler: Tarn at 0.52 AU round a 0.82 solar-mass star, about 0.416 years.
  const tarn = v.find((x) => x.name === "Tarn")!;
  assert.ok(Math.abs(tarn.period_s / 31_557_600 - Math.sqrt(0.52 ** 3 / 0.82)) < 1e-12);
  const vault = voicesFromSystem(kestrel, "u:kestrel/sys:1/body:4");
  assert.deepEqual(
    vault.map((x) => x.name),
    ["Hasp", "Rivet", "Latch"],
  );
  const scene = orbitScene(kestrel.system, v);
  for (const s of scene.sources) {
    assert.equal(s.label, "designed");
    assert.match(s.why, /raised \d+ octaves/);
  }
  // Vault, the giant, sits lowest.
  const f = Object.fromEntries(scene.sources.map((s) => [s.name, s.freq_true]));
  assert.ok(f.Vault! < 110 && f.Tarn! >= 110);
});

test("a scene's score is identical every time, and depends on the dial and the window", () => {
  const scene = orbitScene(kestrel.system, voicesFromSystem(kestrel));
  const a = score(scene, 0, 0, "kestrel reach campaign");
  const b = score(orbitScene(kestrel.system, voicesFromSystem(structuredClone(kestrel))), 0, 0, "kestrel reach campaign");
  assert.deepEqual(a, b);
  assert.ok(a.length >= 5, `notes in the first minute (${a.length})`);
  for (const n of a) assert.ok(n.t >= 0 && n.t < 60 && n.level > 0 && n.level <= 1);
  assert.notEqual(scoreHash(score(scene, 1, 0, "kestrel reach campaign")), scoreHash(a), "the next window differs");
  assert.notEqual(scoreHash(score(scene, 0, 1, "kestrel reach campaign")), scoreHash(a), "the settled dial moves pitches");
  // Pinned fingerprint for the browser test (tests/ui/sound.e2e.mjs): change only on purpose.
  assert.equal(scoreHash(a), json("tests/ui/sound.fixture.json").kestrel_window0);
  assert.deepEqual(score(orbitScene("empty", []), 0), [], "silence is a valid result");
});

test("the shipped recipe is valid; a recipe outside its schema is rejected", () => {
  assert.deepEqual(checkRecipe(recipe), []);
  const bad = (patch: object) => checkRecipe({ ...recipe, ...patch });
  assert.match(bad({ nodes: [...recipe.nodes, { id: "x", type: "sampler" }] }).join(), /unknown type sampler/);
  assert.match(bad({ nodes: recipe.nodes.map((n: { id: string }) => (n.id === "soft" ? { ...n, freq_hz: 90000 } : n)) }).join(), /freq_hz must be a number from 20 to 20000/);
  assert.match(bad({ wires: [...recipe.wires, ["soft", "tone"]] }).join(), /loop/);
  assert.match(bad({ wires: recipe.wires.filter((w: string[]) => w[1] !== "out") }).join(), /nothing is wired to out/);
  assert.match(bad({ why: "" }).join(), /every sound says why/);
  assert.match(bad({ layer: "voices" }).join(), /layer must be/);
  assert.match(bad({ nodes: recipe.nodes.map((n: { id: string }) => (n.id === "tone" ? { ...n, wave: "noise" } : n)) }).join(), /wave must be one of/);
});

test("sound is off unless explicitly on; bad settings fall back", () => {
  assert.equal(readSound(null).on, false);
  assert.equal(readSound("{not json").on, false);
  assert.equal(readSound(JSON.stringify({ on: "yes" })).on, false);
  assert.equal(readSound(JSON.stringify({ on: true, dial: 4, mixer: { music: 9 } })).dial, 1);
  assert.equal(readSound(JSON.stringify({ mixer: { music: 9 } })).mixer.music, 0.35);
  assert.equal(readSound(null, "basic").tier, "basic");
});

test("players hear only what their data view gives them", async () => {
  const k = nodeKernel();
  try {
    const base: PackDoc = json("packs/base-physics/pack.json");
    const playerData: PackDoc = json("packages/shell/src/documents/kestrel.player-data.pack.json");
    const facts: AuthoredFacts = json("packages/shell/src/documents/kestrel.player-facts.json");
    const s = await Session.open(k.client, gmSeat(), facts.universe, [base, playerData]);
    assert.equal(await enterFacts(s, facts), null);
    // Long-range sensors: orbits are not known, so nothing sounds.
    const far = await s.scene(facts.scope, { audience: "everyone", data_view: "long-range" });
    assert.ok(far.ok);
    assert.deepEqual(voicesFromPlayers(far.scene), []);
    assert.match(orbitScene(facts.scope, voicesFromPlayers(far.scene)).caption, /Silence/);
    // Scanned: the periods are given, so those bodies sound.
    const near = await s.scene(facts.scope, { audience: "everyone", data_view: "scanned" });
    assert.ok(near.ok);
    const names = voicesFromPlayers(near.scene).map((v) => v.name);
    assert.ok(names.includes("Tarn"), names.join());
  } finally {
    await k.close();
  }
});
