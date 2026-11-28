// Aurorae in system True (render audit item 2): every world's plan read in one pass (renderer true-aurora.ts) and
// handed through the shared True door (true-body.ts) to the shells gpu-aurora.ts draws.
import { test } from "node:test";
import assert from "node:assert/strict";
import { loadSystemAurorae, type AuroraSystemReader } from "../../packages/renderer/src/true-aurora.ts";
import { exposeAurora, trueLook } from "../../packages/renderer/src/index.ts";

const oval = { driver: "wind", magnetic_latitude_deg: 66, width_deg: 6, emissions: [{ line: "o-557.7", wavelength_m: 557.7e-9, power_w: 1e10, peak_height_m: 110e3, thickness_m: 20e3 }] };
const read = (extra: Record<string, unknown>) => ({ aurora: { diffuse: false, ovals: [oval], footprints: [] }, ...extra });

function reader(sol: boolean, asks: string[]): AuroraSystemReader {
  return {
    async readField(a) {
      asks.push(a.fixture ? `field:${a.fixture}` : `field:${a.system}`);
      return { ok: true, bodies: sol ? [read({ name: "Earth" }), { name: "Mars", aurora: null }] : [read({ address: "s/body:1" }), { address: "s/body:2", aurora: { diffuse: false, ovals: [], footprints: [] } }] };
    },
    async resolvedSystem() {
      asks.push("resolved");
      return { ok: true, result: { facts: { "s/body:1": { "field.tilt": 30 } }, surfaces: [] } };
    },
    async readAirOptics() {
      asks.push("optics");
      return { ok: false, code: "x", message: "no air" };
    },
  };
}

test("the Solar System's worlds are matched by name in the measured fixture; a world with no aurora is left out", async () => {
  const asks: string[] = [];
  const m = await loadSystemAurorae(reader(true, asks), "u:k/sys:sol", [{ address: "u:k/sys:sol/body:2", label: "Earth", radius_m: 6.371e6 }, { address: "u:k/sys:sol/body:3", label: "Mars", radius_m: 3.39e6 }]);
  assert.deepEqual([...m.keys()], ["u:k/sys:sol/body:2"]);
  assert.equal(asks.filter((a) => a.startsWith("field")).length, 1, "the field is read once for the system");
  assert.match(m.get("u:k/sys:sol/body:2")!.plan.note, /oval at 66° magnetic latitude/);
});

test("another system's worlds are matched by address and take their field tilt", async () => {
  const m = await loadSystemAurorae(reader(false, []), "s", [{ address: "s/body:1", label: "A", radius_m: 6e6 }, { address: "s/body:2", label: "B", radius_m: 6e6 }]);
  assert.deepEqual([...m.keys()], ["s/body:1"]);
  const axis = m.get("s/body:1")!.plan.axis;
  assert.ok(Math.abs(Math.acos(axis[2]) * (180 / Math.PI) - 30) < 1e-9, `axis ${axis}`);
});

test("the True door hands the exposed shells to the GPU with the world's pole, and a star or a quiet world gets none", async () => {
  const m = await loadSystemAurorae(reader(true, []), "u:k/sys:sol", [{ address: "e", label: "Earth", radius_m: 6.371e6 }]);
  const d = exposeAurora(m.get("e")!.plan, 6.371e6, "long", 3, 1361)!;
  const look = trueLook({ key: "e", star: false, colour: "#888", pole: [0, 0.3, 0.95], aurora: d });
  assert.equal(look.aurora?.key, "e|aurora");
  assert.deepEqual(look.aurora?.pole, [0, 0.3, 0.95]);
  assert.ok(look.aurora!.draw.layers[0]!.radius > 1, "the shell is above the ground");
  assert.equal(trueLook({ key: "s", star: true, colour: "#fff", pole: [0, 0, 1], aurora: d }).aurora, undefined);
  assert.equal(trueLook({ key: "q", star: false, colour: "#888", pole: [0, 0, 1], aurora: null }).aurora, undefined);
});
