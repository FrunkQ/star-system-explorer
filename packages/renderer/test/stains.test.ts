// Surface stains in True (TRUE4): hues from the data sheet's measured optics, laid on the kernel's bare ground.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { grainAlbedo, indexTableOf, layerReflectance, seenColour, type IndexTable } from "../src/sheet-optics.ts";
import { stainFacts, stainHue, stainNote, stainPrepared } from "../src/stain-look.ts";
import type { Prepared } from "../src/surface.ts";

const sheet = JSON.parse(readFileSync(new URL("../../../packs/base-reference/pack.json", import.meta.url), "utf8"));
const tholinRow = sheet.provides.reference.mixtures.find((m: { id: string }) => m.id === "titan-tholin");
const clear: IndexTable = { lambda_nm: [300, 900], n: [1.31, 1.31], k: [0, 0] };

test("a grain that absorbs nothing scatters everything; absorption darkens it", () => {
  assert.ok(Math.abs(grainAlbedo(1.31, 0, 550, 50) - 1) < 1e-9);
  assert.ok(grainAlbedo(1.6, 0.01, 550, 50) < 0.3);
  assert.equal(layerReflectance(1), 1);
  assert.ok(seenColour(clear).albedo > 0.99, "a layer of clear grains is white");
});

test("tholin from the sheet is red-brown: red over green over blue, and fine grains brighter than coarse", () => {
  const t = indexTableOf(tholinRow)!;
  assert.ok(t, "the sheet's titan-tholin row has optics");
  const fine = seenColour(t, 1);
  assert.ok(fine.rgb[0] > fine.rgb[1] && fine.rgb[1] > fine.rgb[2], `${fine.rgb}`);
  assert.ok(fine.albedo > seenColour(t, 50).albedo);
  // A little tholin among clear ice grains tints them, as on Pluto: reddened but still bright.
  const mix = seenColour([{ table: clear, share: 0.97 }, { table: t, share: 0.03, d_um: 1 }]);
  assert.ok(mix.rgb[0] > mix.rgb[2] && mix.albedo > 0.4, `${mix.rgb} ${mix.albedo}`);
});

test("stains tint bare ground by their share and leave water alone; a row without optics is not drawn", () => {
  const facts = stainFacts({ stains: [{ kind: "tholin", material_id: "titan-tholin", share: 1, why: "…" }, { kind: "sulphur", material_id: "sulfur", share: 0.5, why: "…" }] });
  assert.equal(facts.length, 2);
  const h = stainHue(facts[0]!, tholinRow)!;
  assert.ok(h.hue[0] > h.hue[2]);
  assert.equal(stainHue(facts[1]!, { id: "sulfur" }), null);
  const face = { s: 1, rgb: new Uint8ClampedArray([150, 150, 150]), cover: new Uint8Array([0]) };
  const sea = { s: 1, rgb: new Uint8ClampedArray([40, 80, 160]), cover: new Uint8Array([1]) };
  const p = { faces: [face, sea] } as unknown as Prepared;
  stainPrepared(p, [h]);
  assert.ok(face.rgb[0]! > face.rgb[2]!, `${face.rgb}`);
  assert.deepEqual([...sea.rgb], [40, 80, 160]);
  assert.match(stainNote("Titan", facts, [h]), /tholin 100%, sulphur 50% \(not drawn/);
});
