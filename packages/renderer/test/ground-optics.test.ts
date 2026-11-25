import { test } from "node:test";
import assert from "node:assert/strict";
import { forgetGroundOptics, groundReflectance, readGroundOptics, reflectanceAt, withGround, GROUND_ROWS } from "../src/ground-optics.ts";
import { sampleColour, SURFACE_MATERIALS, type SurfaceTile } from "../../ui-contract/src/index.ts";

// The sheet's rows as MP35 writes them (Apollo 12030 fresh, 10084 mature; LSCC, Taylor et al. 2001).
const LAM = [400, 450, 500, 550, 600, 650, 700];
const ROWS: Record<string, number[]> = {
  "lunar-soil-immature-mare": [0.10514, 0.11845, 0.12714, 0.13629, 0.14455, 0.15286, 0.16113],
  "lunar-soil-mature-mare": [0.043507, 0.048341, 0.052659, 0.057191, 0.061273, 0.065615, 0.069243],
};
const sheet = {
  async readMatter(ask: { table?: string; id?: string }) {
    const r = ROWS[ask.id ?? ""];
    if (ask.table !== "mixtures" || !r) return { ok: false as const, message: "no row" };
    return { ok: true as const, value: { row: { id: ask.id, optical: { reflectance: { lambda_nm: LAM, r } } } } };
  },
};

test("a table is read linearly between its points and held at its ends", () => {
  const t = { lambda_nm: [400, 500], r: [0.1, 0.2] };
  assert.equal(reflectanceAt(t, 300), 0.1);
  assert.ok(Math.abs(reflectanceAt(t, 450) - 0.15) < 1e-12);
  assert.equal(reflectanceAt(t, 900), 0.2);
});

test("weathering darkens and reddens: fresh soil at 0, the mature row at 1, between them in log space", async () => {
  forgetGroundOptics();
  const o = await readGroundOptics(sheet);
  assert.ok(GROUND_ROWS.basalt, "basalt names its rows");
  const fresh = groundReflectance(o, "basalt", 0)!;
  const old = groundReflectance(o, "basalt", 1)!;
  assert.ok(Math.abs(fresh[1] - 0.13629) < 1e-9 && Math.abs(old[1] - 0.057191) < 1e-9);
  // Darker in every band, and the blue (450 nm) falls furthest.
  for (let i = 0; i < 3; i++) assert.ok(old[i]! < fresh[i]!);
  assert.ok(old[2]! / fresh[2]! < old[0]! / fresh[0]!);
  const half = groundReflectance(o, "basalt", 0.5)!;
  assert.ok(Math.abs(half[1] - Math.sqrt(fresh[1] * old[1])) < 1e-12);
  // A material the sheet gives nothing for keeps the palette.
  assert.equal(groundReflectance(o, "granite", 1), null);
});

test("only bare ground on a weathered tile takes the measured colour; the rest keeps the palette", async () => {
  forgetGroundOptics();
  const o = await readGroundOptics(sheet);
  const basalt = SURFACE_MATERIALS.indexOf("basalt");
  const tile = (maturity?: number[]): SurfaceTile => ({ address: "x", key: { face: 0, level: 0, x: 0, y: 0 }, samples: 2, spacing_m: 1, epoch_myr: 0, elevation_dm: [0, 0, 0, 0], material: [basalt, basalt, basalt, basalt], cover: [0, 0, 1, 0], albedo: [300, 300, 300, 300], slope_ddeg: [], surface_age_myr: [], pending: [], hash: "h", ...(maturity ? { maturity } : {}) });
  const [airy, airless] = withGround([tile(), tile([255, 0, 255, 255])], o);
  assert.equal(airy!.ground_rgb, undefined, "no maturity (air): unchanged");
  const mature = sampleColour(airless!, 0);
  const fresh = sampleColour(airless!, 1);
  // Mature mare soil is about 6 % reflective: dark grey, a touch warm, nowhere near the palette's 158, 148, 132 beige.
  assert.ok(mature[1] > 55 && mature[1] < 80, `mature ${mature}`);
  assert.ok(mature[0] >= mature[2] && mature[0] - mature[2] < 30, `mature ${mature}`);
  assert.ok(fresh[1] > mature[1] + 30, `fresh ${fresh} brighter than mature ${mature}`);
  // A sea sample is not ground.
  assert.deepEqual(sampleColour(airless!, 2), sampleColour(tile(), 2));
});
