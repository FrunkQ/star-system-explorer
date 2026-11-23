// GH9: a galaxy's scars drawn from the kernel's dated marks (galaxy histories design 9.4). Checks: a mark's
// strength follows the kernel's rule; a warp is an integral sign (up one side, down the other, flat inside its start);
// ripples from many passes stay a few hundred parsecs high; a punched ring expands at its set speed round an emptied
// middle; scars heal as marks fade, except the thick disc; the Chart paths fit their frames; and a looked-at galaxy
// from the kernel carries marks the words can name.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { brightnessAt, edgeOnPaths, faceOnPaths, heightAt, markStrengthAt, SCAR_SHAPES, scarsAt, scarWords } from "../src/galaxy-marks.ts";
import { constant, type GalaxyMark } from "../../ui-contract/src/index.ts";

const mark = (kind: string, began_gyr: number, strength: number, fades_gyr: number | null, shape: Record<string, number> = {}, caused_by = "u:t/halo:h/prog:b1-1"): GalaxyMark => ({ kind, caused_by, began_gyr, strength, fades_gyr, shape });
const RD = 3;

test("a mark's strength is nothing before it began, falls by e over its fading time, and the thick disc lasts", () => {
  const m = mark("warp", 5, 0.8, 1);
  assert.equal(markStrengthAt(m, 4.9), 0);
  assert.equal(markStrengthAt(m, 5), 0.8);
  assert.ok(Math.abs(markStrengthAt(m, 6) - 0.8 / Math.E) < 1e-12);
  assert.equal(markStrengthAt(mark("thick_disc", 2, 0.5, null), 13), 0.5);
});

test("a warp is an integral sign: flat inside its start, up on one side and down on the other", () => {
  const sc = scarsAt([mark("warp", 10, 1, 1)], 10, RD);
  const start = SCAR_SHAPES.warp.start_rd * RD;
  assert.equal(heightAt(sc, 0.9 * start, 1), 0);
  for (const r of [1.5 * start, 3 * start]) {
    for (const phi of [0.3, 1.2, 2.5]) assert.ok(Math.abs(heightAt(sc, r, phi) + heightAt(sc, r, phi + Math.PI)) < 1e-9, "opposite sides opposite");
    const top = Math.max(...Array.from({ length: 72 }, (_, i) => heightAt(sc, r, (i * Math.PI) / 36)));
    assert.ok(top > 0 && top <= SCAR_SHAPES.warp.max_rise * r + 1e-9, `rise ${top} at ${r}`);
  }
});

test("ripples from many passes stay within a few hundred parsecs", () => {
  const ms = Array.from({ length: 30 }, (_, i) => mark("ripples", 12 + i * 0.01, 0.99, 1, {}, `u:t/halo:h/prog:b${i}`));
  const sc = scarsAt(ms, 12.5, RD);
  let most = 0;
  for (let r = 0.5; r < 25; r += 0.25) for (let p = 0; p < 6.3; p += 0.3) most = Math.max(most, Math.abs(heightAt(sc, r, p)));
  assert.ok(most > 0.05 && most <= SCAR_SHAPES.ripples.max_kpc + 1e-9, `${most} kpc`);
});

test("a punched ring expands at its set speed round an emptied middle, and fades", () => {
  const m = mark("ring", 8, 0.9, 0.3, { r_kpc: 1, speed_kms: 400 });
  const a = scarsAt([m], 8.05, RD).rings[0]!;
  const b = scarsAt([m], 8.15, RD).rings[0]!;
  const kpcPerGyr = (SCAR_SHAPES.ring.expand_kms * 1e3 * 1e9 * constant("yr_julian")) / (1e3 * constant("pc"));
  assert.ok(Math.abs((b.r_kpc - a.r_kpc) / 0.1 - kpcPerGyr) < 1e-9);
  const sc = scarsAt([m], 8.15, RD);
  assert.ok(brightnessAt(sc, b.r_kpc, 0) > 1.5, "bright at the ring");
  assert.ok(brightnessAt(sc, 0.3 * b.r_kpc, 0) < 0.8, "emptied inside");
  assert.ok(b.strength < a.strength);
});

test("scars heal as their marks fade, except the thick disc", () => {
  const ms = [mark("warp", 2, 0.9, 1), mark("lopsided", 2, 0.5, 1), mark("ring", 2, 0.6, 0.3), mark("thick_disc", 2, 0.4, null)];
  const soon = scarsAt(ms, 2.2, RD);
  const late = scarsAt(ms, 13, RD);
  assert.ok(soon.warp.rise > 0 && soon.lopsided.amp > 0 && soon.rings.length === 1);
  assert.equal(late.warp.rise, 0);
  assert.equal(late.lopsided.amp, 0);
  assert.equal(late.rings.length, 0);
  assert.equal(late.thick, 0.4);
  assert.deepEqual(late.live.map((l) => l.mark.kind), ["thick_disc"]);
});

test("Chart paths fit their frames", () => {
  const ms = [mark("warp", 5, 0.9, 1), mark("ripples", 5, 0.9, 1), mark("tilt", 5, 0.5, 4), mark("ring", 5, 0.7, 0.3), mark("burst_knots", 5, 0.8, 0.05), mark("lopsided", 5, 0.5, 1), mark("driven_arms", 5, 0.6, 0.5)];
  const sc = scarsAt(ms, 5.05, RD);
  const nums = (d: string) => [...d.matchAll(/(-?[\d.]+) (-?[\d.]+)/g)].map((m) => [Number(m[1]), Number(m[2])] as const);
  const e = edgeOnPaths(sc, [520, 110], 20);
  for (const d of [e.mid, e.thin, e.thick]) for (const [x, y] of nums(d)) assert.ok(x >= 0 && x <= 520 && y >= 0 && y <= 110, `${x} ${y}`);
  const f = { cx: 260, cy: 260, k: 244 / 20, outer_kpc: 20 };
  const p = faceOnPaths(sc, f);
  for (const [x, y] of nums(p.edge)) assert.ok(x >= 0 && x <= 520 && y >= 0 && y <= 520);
  assert.ok(p.knots.length > 0 && p.rings.length === 1);
});

test("a looked-at galaxy from the kernel carries marks the words can name", async () => {
  const { Session } = await import("../../ui-contract/src/session.ts");
  const { gmSeat } = await import("../../ui-contract/src/seat.ts");
  const { nodeKernel } = await import("../../ui-contract/test/node-kernel.ts");
  const pack = (n: string) => JSON.parse(readFileSync(new URL(`../../../packs/${n}/pack.json`, import.meta.url), "utf8"));
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, gmSeat(), { id: "scars", seed: "scars" }, ["base-physics", "base-reference", "base-cosmos", "base-cosmology"].map(pack));
    const r = await s.galaxyChart({ halo: "milky-way-like" });
    assert.ok(r.ok);
    const marks = r.chart.assembly?.marks ?? [];
    assert.ok(marks.length > 0, "GH3 writes marks for a fed galaxy");
    const sc = scarsAt(marks, r.chart.now_gyr, r.chart.disc_kpc);
    const w = scarWords(sc);
    assert.ok(w.length > 0);
    assert.match(w.join(" "), /thick disc|warp|ripples/);
  } finally {
    await k.close();
  }
});
