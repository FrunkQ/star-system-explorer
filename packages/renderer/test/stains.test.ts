// Surface stains in True (TRUE4): hues from the data sheet's measured optics, laid on the kernel's bare ground.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { grainAlbedo, indexTableOf, layerReflectance, seenColour, type IndexTable } from "../src/sheet-optics.ts";
import { stainFacts, stainHue, stainNote, stainPrepared } from "../src/stain-look.ts";
import type { Prepared } from "../src/surface.ts";
import { composeFaces, withFiner } from "../src/texture-ladder.ts";
import type { SurfaceTile } from "../../ui-contract/src/index.ts";

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

test("the GPU's face pictures are stained like the flat picture, finer tiles too; sea is not", () => {
  const h = stainHue({ kind: "tholin", material_id: "titan-tholin", share: 1, why: "…" }, tholinRow)!;
  const tile = (level: number, cover: number): SurfaceTile => ({ address: "x", key: { face: 0, level, x: 0, y: 0 }, samples: 1, spacing_m: 1000, epoch_myr: 0, elevation_dm: [0], material: [2], cover: [cover], albedo: [0.3], slope_ddeg: [0], surface_age_myr: [0], pending: [], hash: `${level}${cover}` });
  const plain = composeFaces([tile(0, 0)]).faces[0]!.rgb;
  const c = composeFaces([tile(0, 0)], [h]);
  const red = c.faces[0]!.rgb;
  assert.ok(red[0]! - red[2]! > plain[0]! - plain[2]!, `${red} vs ${plain}`);
  const finer = withFiner({ faces: c }, [tile(1, 0)]).faces!;
  assert.deepEqual([...finer.faces[0]!.rgb.slice(0, 3)], [...red.slice(0, 3)]);
  const sea = composeFaces([tile(0, 1)], [h]).faces[0]!.rgb;
  assert.deepEqual([...sea], [...composeFaces([tile(0, 1)]).faces[0]!.rgb]);
});
