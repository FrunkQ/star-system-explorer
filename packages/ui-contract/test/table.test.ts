// ER11a: the renderer's reader of `sse.table/1` reads a table the kernel baked exactly as the kernel does. Through a
// real kernel: the black hole disc table (`bake`, table `sse.light.disc`) gives back, at the kernel's own traced
// points (`read_light_paths`), their shift and temperature within the table's stated bound; nodes read back exactly;
// texture coordinates put texel centres on nodes; and a phone's tier gets the small size.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { Session } from "../src/session.ts";
import { gmSeat } from "../src/seat.ts";
import { channelOf, indexOf, nodeAt, rgbaOf, sampleTable, texelOf, type Table, type TableAxis } from "../src/table.ts";
import { nodeKernel } from "./node-kernel.ts";

const root = new URL("../../../", import.meta.url);

test("axes index back to their nodes and hold to their ends", () => {
  const a: TableAxis = { name: "r", unit: "r_g", spacing: "log", from: 6, to: 6000, n: 4, about: "" };
  assert.equal(nodeAt(a, 0), 6);
  assert.equal(nodeAt(a, 3), 6000);
  assert.ok(Math.abs(nodeAt(a, 1) - 60) < 1e-9);
  assert.ok(Math.abs(indexOf(a, 600) - 2) < 1e-12);
  assert.equal(indexOf(a, 1), 0);
  assert.equal(indexOf(a, 1e9), 3);
});

test("the disc table, baked by the kernel, gives the kernel's traced points their shift and temperature", async () => {
  const k = nodeKernel();
  try {
    const base = JSON.parse(readFileSync(new URL("packs/base-physics/pack.json", root), "utf8"));
    const s = await Session.open(k.client, gmSeat(), { id: "holes", seed: "holes" }, [base]);
    const r = await s.readLightPaths({ fixture: "black-holes" });
    assert.ok(r.ok, JSON.stringify(r));
    type Spot = { impact_rg: number; image_angle_rad: number; r_rg: number; shift: number; t_emitted_k: number };
    type Read = { address: string; gravitational_radius_m: number; feed?: { rate_kg_s: number; flows: { kind: string; strength: number }[]; disc: { inner_m: number; outer_m: number } } | null; image?: { tilt_rad: number; brightest: Spot; approaching: Spot; receding: Spot } | null };
    let checked = 0;
    for (const p of r.paths as unknown as Read[]) {
      const f = p.feed;
      if (!f || !p.image) continue;
      const share = (kind: string) => f.flows.find((x) => x.kind === kind)?.strength ?? 0;
      const subject = { gravitational_radius_m: p.gravitational_radius_m, rate_kg_s: (share("thin") + share("thick")) * f.rate_kg_s, inner_m: f.disc.inner_m, outer_m: f.disc.outer_m };
      const b = await s.bake({ table: "sse.light.disc", subject, tier: "gpu-high" });
      assert.ok(b.ok, JSON.stringify(b));
      const t: Table = b.table;
      assert.equal(t.format, "sse.table/1");
      assert.ok(t.sampling.within, JSON.stringify(t.sampling));
      const [gc, tc] = [channelOf(t, "shift"), channelOf(t, "t4")];
      // Nodes read back exactly.
      const node = [nodeAt(t.axes[0]!, 5), nodeAt(t.axes[1]!, 3)];
      const v = sampleTable(t, node);
      const k0 = (3 * t.axes[0]!.n + 5) * t.channels.length;
      assert.ok(Math.abs(v[gc]! - t.values[k0 + gc]!) <= 1e-9 * Math.abs(t.values[k0 + gc]!));
      // Texel centres on nodes.
      const uv = texelOf(t, node);
      assert.ok(Math.abs(uv[0]! * t.axes[0]!.n - 0.5 - 5) < 1e-9 && Math.abs(uv[1]! * t.axes[1]!.n - 0.5 - 3) < 1e-9);
      assert.equal(rgbaOf(t)[(3 * t.axes[0]!.n + 5) * 4 + gc], Math.fround(t.values[k0 + gc]!));
      const img = p.image;
      for (const spot of [img.brightest, img.approaching, img.receding]) {
        const q = (spot.impact_rg * Math.sin(img.tilt_rad) * Math.sin(spot.image_angle_rad)) / spot.r_rg;
        const [g, t4] = [sampleTable(t, [spot.r_rg, q])[gc]!, sampleTable(t, [spot.r_rg, q])[tc]!];
        const te = Math.sqrt(Math.sqrt(t4));
        assert.ok(Math.abs(g - spot.shift) / spot.shift < t.sampling.bound[gc]!, `${p.address}: shift ${g} against traced ${spot.shift}`);
        assert.ok(Math.abs(te - spot.t_emitted_k) / spot.t_emitted_k < t.sampling.bound[tc]!, `${p.address}: T ${te} against traced ${spot.t_emitted_k}`);
        checked++;
      }
      const phone = await s.bake({ table: "sse.light.disc", subject, tier: "gpu-basic" });
      assert.ok(phone.ok && phone.table.size === "small" && phone.table.values.length < t.values.length);
    }
    assert.ok(checked >= 6, `checked ${checked} points`);
  } finally {
    await k.close();
  }
});
