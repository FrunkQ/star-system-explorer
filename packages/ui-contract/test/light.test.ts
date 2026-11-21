// O1 (UI side) in Node: a thermal source at a known temperature gives the
// published colour; every shipped recipe is valid and says what its colours
// mean; a recipe that overclaims honesty is refused; stubbed descriptions say
// they are stubbed; heat below the Draper point shows no visible light; the
// mid infrared lets worlds glow by their own warmth.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import type { SystemScene } from "../src/scene.ts";
import type { PlayerScene } from "../src/player-scene.ts";
import { checkLightRecipe, describeLight, litFromPlayers, litFromSystem, paint, planck, readLight, teffFromSpectral, thermalColour, type BandRecipe } from "../src/light.ts";

const root = new URL("../../../", import.meta.url);
const json = (p: string) => JSON.parse(readFileSync(new URL(p, root), "utf8"));
const recipes: BandRecipe[] = json("packages/shell/src/documents/light.recipes.json").recipes;
const R = (id: string) => recipes.find((r) => r.id === id)!;
const rgb = (h: string) => [1, 3, 5].map((i) => parseInt(h.slice(i, i + 2), 16));

test("a thermal source gives the published blackbody colour", () => {
  // Mitchell Charity's blackbody table (CIE 1931 2°, D65, sRGB),
  // http://www.vendian.org/mncharity/dir3/blackbody/ ; within 10 of 255 per channel.
  const charity: [number, string][] = [
    [3000, "#ffb46b"],
    [6500, "#fff9fd"],
    [10000, "#ccdbff"],
  ];
  for (const [t, want] of charity) {
    const got = rgb(thermalColour(t));
    rgb(want).forEach((w, i) => assert.ok(Math.abs(got[i]! - w) <= 10, `${t} K: ${thermalColour(t)} vs ${want}`));
  }
  // Wien: a hotter source is bluer.
  assert.ok(rgb(thermalColour(4000))[2]! < rgb(thermalColour(8000))[2]!);
  // Planck at 500 nm and 5772 K: 2hc²/λ⁵ / (exp(hc/λkT) − 1) = 2.6239e13 W sr⁻¹ m⁻³.
  assert.ok(Math.abs(planck(500e-9, 5772) / 2.6239e13 - 1) < 1e-4);
  assert.equal(planck(1e-12, 300), 0, "no overflow far in the Wien tail");
});

test("every shipped recipe is valid and says what its colours mean", () => {
  assert.ok(recipes.length >= 9);
  for (const r of recipes) {
    assert.deepEqual(checkLightRecipe(r), [], r.id);
    assert.ok(r.legend.length > 40 && r.why.length > 40, `${r.id} explains itself`);
  }
  const ids = recipes.map((r) => r.id);
  for (const want of ["eye", "camera", "sho", "hoo", "near-ir", "mid-ir", "uv", "xray", "radio"]) assert.ok(ids.includes(want), want);
});

test("a recipe that overclaims or breaks its schema is refused", () => {
  const bad = (patch: object, id = "sho") => checkLightRecipe({ ...R(id), ...patch }).join("; ");
  assert.match(bad({ honesty: "actual" }), /only eye mode may claim actual colour/);
  assert.match(bad({ honesty: "actual-enhanced" }), /composite recipes are mapped/);
  assert.match(bad({ honesty: "mapped" }, "eye"), /eye is actual colour/);
  assert.match(bad({ legend: "" }), /legend/);
  assert.match(bad({ channels: [{ source: { line: "fe-x" }, colour: "#ff0000", label: "x" }, ...R("sho").channels] }), /unknown line fe-x/);
  assert.match(bad({ channels: [{ source: { band: "visible" }, colour: "red", label: "x" }, ...R("sho").channels] }), /#rrggbb/);
  assert.match(bad({ decades: 99 }, "uv"), /decades/);
  assert.match(bad({ mode: "hologram" }), /mode must be/);
});

const kestrel: SystemScene = json("packages/shell/src/documents/kestrel.scene.json");
// The derived scene gives the star a surface temperature and radius; the fixture alone does not.
const derived: SystemScene = structuredClone(kestrel);
derived.subjects[0]!.facts.push({ field: "teff_k", label: "Surface", value: 5150, display: "5150 K", editable: false, source: "derived" } as never, { field: "radius_sun", label: "Radius", value: 0.8, display: "0.8", editable: false, source: "derived" } as never);

test("stubbed light from scene facts, labelled so; the fixture's star without a temperature has none", () => {
  const bare = describeLight(litFromSystem(kestrel));
  assert.ok(bare.missing.includes("Kestrel"), "no derived temperature, no star light");
  const { lights, missing } = describeLight(litFromSystem(derived));
  assert.ok(!missing.includes("Kestrel"));
  assert.ok(lights.length >= 6);
  for (const l of lights) {
    assert.equal(l.stubbed, true);
    assert.match(l.why, /Stubbed/);
  }
  assert.ok(missing.includes("Hasp"), "moons without albedo or temperature have no light");
  const tarn = lights.find((l) => l.address === "u:kestrel/sys:1/body:3")!;
  assert.deepEqual(tarn.continuum.map((c) => c.kind), ["reflected", "thermal"]);
});

test("recipes paint what the physics says", () => {
  const { lights } = describeLight(litFromSystem(derived));
  const star = "u:kestrel/sys:1/body:0";
  const tarn = "u:kestrel/sys:1/body:3";
  // Eye: the star is its blackbody colour; a grey planet takes its star's tint.
  const eye = paint(R("eye"), lights);
  assert.equal(eye.get(star), thermalColour(5150));
  assert.equal(eye.get(tarn), thermalColour(5150));
  // Mid infrared: the temperate planet glows within a few decades of the star; in the UV it is far fainter.
  const val = (id: string, a: string) => rgb(paint(R(id), lights).get(a) ?? "#000000")[0]!;
  assert.ok(val("mid-ir", tarn) > val("uv", tarn), "worlds glow by their own heat in the thermal infrared");
  // X-rays: thermal light alone gives the star almost nothing, and nothing pretends otherwise.
  const x = paint(R("xray"), lights);
  assert.equal(x.get(tarn), null);
  assert.equal(x.get(star), null);
  // Heat below the Draper point gives no visible light: a body with only that shows nothing in Eye.
  const cool = paint(R("eye"), [{ address: "c", continuum: [{ kind: "thermal", t_k: 300, weight: 1 }], lines: [], stubbed: true, why: "" }]);
  assert.equal(cool.get("c"), null);
  // Painting is the same every time for the same light.
  assert.deepEqual([...paint(R("sho"), lights)], [...paint(R("sho"), structuredClone(lights))]);
});

test("players' light comes only from facts their view gives; spectral types give a stubbed temperature", () => {
  assert.ok(Math.abs(teffFromSpectral("K1V")! - 5110) < 1);
  assert.ok(Math.abs(teffFromSpectral("G2V")! - 5770) < 30, "the Sun's class lands near 5772 K");
  assert.equal(teffFromSpectral("not a star"), undefined);
  const scene: PlayerScene = {
    audience: "everyone",
    scope: "u:k/sys:1",
    subjects: [
      { address: "u:k/sys:1", view: "scanned", tags: [], facts: [{ field: "name", value: "Kestrel" }, { field: "spectral", value: "K1V" }] as never },
      { address: "u:k/sys:1/body:3", view: "scanned", tags: [], facts: [{ field: "name", value: "Tarn" }, { field: "kind", value: "planetary-body" }, { field: "surface.temp_mean_k", value: 288 }] as never },
      { address: "u:k/sys:1/body:4", view: "scanned", tags: [], facts: [{ field: "name", value: "Vault" }] as never },
    ],
  };
  const { lights, missing } = describeLight(litFromPlayers(scene));
  assert.deepEqual(lights.map((l) => l.address), ["u:k/sys:1", "u:k/sys:1/body:3"]);
  assert.deepEqual(missing, ["Vault"]);
  // Tarn's only known light is its warmth: nothing in Eye, a glow in the thermal infrared.
  assert.equal(paint(R("eye"), lights).get("u:k/sys:1/body:3"), null);
  assert.notEqual(paint(R("mid-ir"), lights).get("u:k/sys:1/body:3"), null);
});

test("the setting falls back to chart colours", () => {
  assert.equal(readLight(null, ["eye"]), "symbols");
  assert.equal(readLight("{bad", ["eye"]), "symbols");
  assert.equal(readLight(JSON.stringify({ recipe: "hologram" }), ["eye"]), "symbols");
  assert.equal(readLight(JSON.stringify({ recipe: "eye" }), ["eye"]), "eye");
});
