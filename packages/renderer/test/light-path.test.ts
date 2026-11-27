// The light-path stage (XO4): the renderer's reads of the kernel's light near a mass agree with the kernel. The bend
// table and the disc come from `read_light_paths` through a real kernel, and the disc's shift and glow from the table
// the kernel bakes for it (`bake`, `sse.light.disc`; ER11a); the picture's rays must land where the kernel's traced
// points landed, with the same shift and seen temperature within the table's stated sampling bound, and the dimming
// note must give the kernel's figure. The GPU pass does the same reads (gpu-light-path.ts); the e2e measures it.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { Session } from "../../ui-contract/src/session.ts";
import { constant, type Json, type Table } from "../../ui-contract/src/index.ts";
import { gmSeat } from "../../ui-contract/src/seat.ts";
import { nodeKernel } from "../../ui-contract/test/node-kernel.ts";
import { B_CRIT, bandMean, bandTable, BAND_SLOT, brightest, discAt, discOf, discSubject, landing, LOG_T0, LOG_T1, marksOf, PICTURE_BANDS, RAMP, rayOf, rowOfB, traceRay, turnOf, uAt, type BendTable, type LightPathsRead } from "../src/light-path.ts";
import { holeSky, skyFrame, galactic } from "../src/hole-sky.ts";

const root = new URL("../../../", import.meta.url);
const block = (file: string, id: string) => {
  const d = JSON.parse(readFileSync(new URL(`packs/base-physics/formulas/${file}`, root), "utf8")) as { atoms: { id: string; equation?: { examples?: Record<string, number>[] }; examples?: Record<string, number>[] }[] };
  const a = d.atoms.find((x) => x.id === id);
  assert.ok(a, `block ${id} is in ${file}`);
  return a.examples ?? a.equation?.examples ?? [];
};

test("the baked shift is the block's: every worked example of sse.light.disc_shift read off the kernel's table", async () => {
  const ex = block("light.json", "sse.light.disc_shift");
  assert.ok(ex.length >= 2, "the block has worked examples");
  const k = nodeKernel();
  try {
    const base = JSON.parse(readFileSync(new URL("packs/base-physics/pack.json", root), "utf8"));
    const s = await Session.open(k.client, gmSeat(), { id: "holes", seed: "holes" }, [base]);
    // A disc whose inner edge is the examples' radius, so they fall on the table's first column of nodes.
    for (const e of ex) {
      const r = e.r! / e.rg!;
      const b = await s.bake({ table: "sse.light.disc", subject: { gravitational_radius_m: 1, rate_kg_s: 1, inner_m: r, outer_m: 100 * r } as unknown as Json });
      assert.ok(b.ok, JSON.stringify(b));
      const g = discAt(discOf(b.table)!, r, e.b! / e.rg!, e.i!, e.al!).g;
      const bound = b.table.sampling.bound[0]!;
      assert.ok(Math.abs(g - e.g!) <= bound * Math.abs(e.g!), `g ${g} against ${e.g} (bound ${bound})`);
    }
  } finally {
    await k.close();
  }
});

test("rows never mix across √27, and the weak bender takes over past 64 gravitational radii", () => {
  assert.equal(rowOfB(B_CRIT - 1e-9)! < 128, true);
  assert.equal(rowOfB(B_CRIT + 1e-9), 128);
  assert.equal(rowOfB(64.0001), null);
  const t: BendTable = { hash: "", rows: 256, cols: 256, row_length: 260, flat: new Float64Array(256 * 260) };
  assert.equal(turnOf(t, 4), null, "swallowed");
  assert.equal(turnOf(t, 100), 4 / 100, "4 r_g / b");
  for (const e of block("gravity.json", "sse.gravity.weak_bend")) if (e.alpha !== undefined) assert.ok(Math.abs(turnOf(t, e.b! / e.rg!)! - e.alpha) < 1e-9 * Math.max(1, e.alpha) || e.b! / e.rg! <= 64);
});

test("the band table is Planck through the sheet's bands: hotter is brighter in every band, and visible goes blue", () => {
  const t = bandTable();
  for (const b of PICTURE_BANDS) {
    const [row, ch] = BAND_SLOT[b];
    let last = -Infinity;
    for (let i = 0; i < RAMP; i += 8) {
      const v = t[(row * RAMP + i) * 4 + ch]!;
      assert.ok(v >= last - 1e-9, `${b} rises with temperature`);
      last = v;
    }
  }
  const at = (logT: number) => Math.round(((logT - LOG_T0) / (LOG_T1 - LOG_T0)) * (RAMP - 1));
  const rgb = (i: number) => [t[i * 4]!, t[i * 4 + 1]!, t[i * 4 + 2]!];
  const cool = rgb(at(Math.log10(3000)));
  const hot = rgb(at(7));
  assert.ok(cool[0]! > cool[2]!, "3000 K is redder than blue");
  assert.ok(hot[2]! > hot[0]!, "ten million kelvin is bluer than red");
  assert.ok(Math.abs(Math.log10(bandMean("xray", 1e7) / bandMean("xray", 1e6))) > 3, "X-rays rise steeply past a million kelvin");
});

test("the picture repeats the kernel: its rays land where the kernel's traced points did, with their shift and seen temperature", async () => {
  const k = nodeKernel();
  try {
    const base = JSON.parse(readFileSync(new URL("packs/base-physics/pack.json", root), "utf8"));
    // The data sheet carries the Sun the dimming note compares with (T_sun).
    const sheet = JSON.parse(readFileSync(new URL("packs/base-reference/pack.json", root), "utf8"));
    const s = await Session.open(k.client, gmSeat(), { id: "holes", seed: "holes" }, [base, sheet]);
    const r = await s.readLightPaths({ fixture: "black-holes", table: true });
    assert.ok(r.ok, JSON.stringify(r));
    const table = r.bend_table as unknown as BendTable;
    const paths = r.paths as unknown as (LightPathsRead & { table: { hash: string } })[];
    assert.equal(paths.length, 3);
    const discs: Table[] = [];
    assert.equal(table.flat.length, 256 * 260);
    for (const p of paths) {
      assert.equal(p.table.hash, table.hash, "the table handed over is the one the answer names");
      const baked = await s.bake({ table: "sse.light.disc", subject: discSubject(p) as unknown as Json, tier: "gpu-high" });
      assert.ok(baked.ok, JSON.stringify(baked));
      assert.ok(baked.table.sampling.within, JSON.stringify(baked.table.sampling));
      discs.push(baked.table);
      const disc = discOf(baked.table)!;
      assert.ok(disc, `${p.address} has a disc`);
      const [gBound, t4Bound] = baked.table.sampling.bound as [number, number];
      const img = p.image!;
      for (const spot of [img.brightest, img.approaching, img.receding]) {
        const ray = traceRay(table, disc, spot.impact_rg, spot.image_angle_rad, img.tilt_rad);
        assert.equal(ray.kind, "disc", `${p.address}: the kernel's point is on the disc`);
        if (ray.kind !== "disc") continue;
        assert.equal(ray.order, spot.order);
        const close = (a: number, b: number, what: string, tol = 1e-9) => assert.ok(Math.abs(a - b) <= tol * Math.abs(b), `${p.address} ${what}: ${a} against ${b}`);
        close(ray.r, spot.r_rg, "landing distance");
        // The shift and temperature are read off the baked table: within its stated bound (T is t4^¼, a quarter of it).
        close(ray.g, spot.shift, "shift", gBound);
        close(ray.t_emitted_k, spot.t_emitted_k, "temperature", t4Bound / 4);
        close(ray.t_seen_k, spot.t_seen_k, "seen temperature", gBound + t4Bound / 4);
      }
      // The side ratio, from two of the picture's own rays.
      const a = traceRay(table, disc, img.approaching.impact_rg, img.approaching.image_angle_rad, img.tilt_rad);
      const b = traceRay(table, disc, img.receding.impact_rg, img.receding.image_angle_rad, img.tilt_rad);
      if (a.kind === "disc" && b.kind === "disc") assert.ok(Math.abs((a.t_seen_k / b.t_seen_k) ** 4 / img.side_ratio - 1) < 4 * (2 * gBound + t4Bound / 2), "side ratio");
      // The dimming note: the kernel's figure at its viewer's tilt. The kernel integrates the visible band by Simpson's
      // rule at 64 steps a decade, ui-contract's bandRadiance by 48 trapezoids; they agree to 0.02% for the fed holes,
      // and to 0.6% for the starved one, whose 800 K disc puts the visible band deep in Planck's tail. The seen
      // temperature is read off the desk tier's baked table (gpu-high), which keeps the starved hole within the same 1%.
      const br = brightest(table, disc, img.tilt_rad)!;
      assert.ok(Math.abs(br.times_sun_surface / img.exposure.times_sun_surface - 1) < (img.brightest.t_seen_k > 1e4 ? 0.001 : 0.01), `${p.address}: dimmed ${br.times_sun_surface} against the kernel's ${img.exposure.times_sun_surface}`);
      // Geometry the frame is measured against: the shadow's edge at √27, the far side's inner edge above it.
      const view = { distance: 2000, tilt: img.tilt_rad, half: 16, turning: 1 as const };
      const m = marksOf(table, disc, view);
      assert.ok(Math.abs((m.shadow * view.half) / B_CRIT - 1) < 1e-5);
      assert.ok(m.arch !== null && m.arch > m.shadow, `${p.address}: the arch sits above the shadow`);
      // A pixel's ray: straight up the picture is α = 0, the left is +π/2 (the side turning towards the viewer).
      assert.ok(Math.abs(rayOf(view, 0, 0.5).alpha) < 1e-12);
      assert.ok(Math.abs(rayOf(view, -0.5, 0).alpha - Math.PI / 2) < 1e-12);
    }
    // The table's cells are the kernel's: a ray far out barely bends, so u at a quarter turn is near 1 / b.
    assert.ok(Math.abs(uAt(table, 60, Math.PI / 2)! * 60 - 1) < 0.1);
    // A ray inside the shadow's edge is swallowed; one beyond turns by more than the weak bender's 4 / b.
    assert.equal(traceRay(table, null, 5, 0, 1).kind, "shadow");
    const turn = traceRay(table, null, 7, 0, 1);
    assert.ok(turn.kind === "sky" && turn.turn > 4 / 7);
    // A disc seen face on is the same all round: the direct image lands at one distance whatever the angle.
    const d0 = discOf(discs[0]!)!;
    const l1 = landing(table, 10, 0.3, 0, d0.rIn, d0.rOut)!;
    const l2 = landing(table, 10, 2.1, 0, d0.rIn, d0.rOut)!;
    assert.ok(Math.abs(l1.r - l2.r) < 1e-12);
    assert.equal(discAt(d0, d0.rIn, 7, 1, 1).t_emitted_k, 0, "nothing glows at the inner edge");
  } finally {
    await k.close();
  }
});

test("the sky behind keeps each star's light: radiance times solid angle is π times its share", () => {
  const toward = galactic(30, 10);
  const fr = skyFrame(toward);
  assert.ok(Math.abs(fr.back[0] + toward[0]) < 1e-12);
  // A star a little off the hole's direction, and one far round behind the camera.
  const near: [number, number, number] = [toward[0] + 0.01 * fr.right[0], toward[1] + 0.01 * fr.right[1], toward[2] + 0.01 * fr.right[2]];
  const l = Math.hypot(...near);
  const star = { dir: [near[0] / l, near[1] / l, near[2] / l] as [number, number, number], luminosity_lsun: 1, distance_pc: 10, teff_k: 5800 };
  const far = { dir: [-toward[0], -toward[1], -toward[2]] as [number, number, number], luminosity_lsun: 1, distance_pc: 10, teff_k: 5800 };
  const { sky } = holeSky("t", toward, [star], [star, far], { size: 256, half: 0.1, wholeW: 256 });
  const texel = (2 * 0.1) / 256;
  let sum = 0;
  for (let i = 0; i < sky.patch.data.length; i += 4) sum += sky.patch.data[i]! + sky.patch.data[i + 1]! + sky.patch.data[i + 2]!;
  const auPerPc = constant("pc") / constant("au");
  const share = 1 / (10 * auPerPc) ** 2;
  assert.ok(Math.abs((sum * texel * texel) / (3 * Math.PI * share) - 1) < 0.02, `patch keeps the light: ${(sum * texel * texel) / (3 * Math.PI * share)}`);
  let behind = 0;
  for (let i = 0; i < sky.whole.data.length; i += 4) behind += sky.whole.data[i]!;
  assert.ok(behind > 0, "the star behind the camera is in the whole sky");
});
