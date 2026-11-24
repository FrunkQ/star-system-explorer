// RK10: the light bands and the band each is seen in are read off the sheet's Bands table, not typed.
import { test } from "node:test";
import assert from "node:assert/strict";
import { BANDS, SHEET_BANDS, VIEW_BANDS, recipeBand, type BandRecipe } from "../src/index.ts";

/** The view band a recipe painting one source is seen in. */
const sourceViewBand = (source: unknown): string => recipeBand({ id: "t", mode: "false", channels: [{ source }] } as unknown as BandRecipe);

test("every light row of the sheet is a band, end to end, out to the sheet's radio edge", () => {
  const rows = SHEET_BANDS.filter((b) => b.kind === "light");
  assert.deepEqual(BANDS.map((b) => b.id), rows.map((b) => b.id));
  for (let i = 1; i < BANDS.length; i++) assert.equal(BANDS[i]!.from_m, BANDS[i - 1]!.to_m);
  assert.equal(BANDS.at(-1)!.to_m, rows.at(-1)!.to_m);
});

test("each light band is seen in the view band its sheet row names", () => {
  assert.equal(sourceViewBand({ band: "near-ir" }), "infrared");
  assert.equal(sourceViewBand({ band: "gamma" }), "xray");
  assert.equal(sourceViewBand({ band: "mm" }), "radio");
  // Long radio (100 m) was off the old typed list's end; the sheet runs radio to 10 km.
  assert.equal(sourceViewBand({ from_m: 50, to_m: 200 }), "radio");
  for (const b of BANDS) assert.ok((VIEW_BANDS as readonly string[]).includes(sourceViewBand({ band: b.id })), b.id);
});
