// The interface reads constants from the master data sheet, never typed copies (MP22): the generated copy matches
// packs/base-reference exactly, and the values the chart room leans on are the sheet's.
import { test } from "node:test";
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { constant, constantOf, constantIds, SHEET_BANDS, SHEET_PACK } from "../src/sheet.ts";
import { AU_M, M_EARTH, M_SUN, R_EARTH_M, R_SUN_M, L_SUN_W } from "../src/make.ts";

const root = new URL("../../../", import.meta.url);

test("the generated constants are the sheet's, unchanged", () => {
  execFileSync("python3", ["scripts/build-ui-reference.py", "--check"], { cwd: root });
  const pack = JSON.parse(readFileSync(new URL("packs/base-reference/pack.json", root), "utf8"));
  assert.equal(SHEET_PACK.id, pack.id);
  const want = pack.provides.reference.constants as { id: string; value: { v: number; unit: string } }[];
  assert.deepEqual(constantIds().sort(), want.map((c) => c.id).sort());
  for (const c of want) assert.equal(constant(c.id), c.value.v, c.id);
  assert.equal(constantOf("au").unit, "m");
  assert.throws(() => constant("no-such-constant"), /no constant/);
});

test("the chart room's yardsticks come from the sheet", () => {
  assert.equal(AU_M, constant("au"));
  assert.equal(M_EARTH, constant("M_earth"));
  assert.equal(M_SUN, constant("M_sun"));
  assert.equal(R_EARTH_M, constant("R_earth_mean"));
  assert.equal(R_SUN_M, constant("R_sun"));
  assert.equal(L_SUN_W, constant("L_sun"));
});

test("the Bands table: every light band meets the next and is seen in a view row (RK10)", () => {
  const views = new Set(SHEET_BANDS.filter((b) => b.kind === "view").map((b) => b.id));
  assert.deepEqual(
    SHEET_BANDS.filter((b) => b.kind === "view").map((b) => b.view_id),
    ["visible", "infrared", "ultraviolet", "xray", "radio", "mag"],
  );
  const light = SHEET_BANDS.filter((b) => b.kind === "light");
  assert.equal(light.length, 9);
  for (let i = 1; i < light.length; i++) assert.equal(light[i - 1]!.to_m, light[i]!.from_m, light[i]!.id);
  for (const b of SHEET_BANDS.filter((x) => x.kind !== "view")) assert.ok(views.has(b.view!), b.id);
});
