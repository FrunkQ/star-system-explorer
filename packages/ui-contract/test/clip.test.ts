import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { copyClip, missingPacks, placeClip, readClip, subtree } from "../src/clip.ts";
import { AU_M, M_EARTH } from "../src/make.ts";
import type { SystemScene } from "../src/scene.ts";

const scene = JSON.parse(readFileSync(new URL("../../shell/src/documents/kestrel.scene.json", import.meta.url), "utf8")) as SystemScene;
const TARN = "u:kestrel/sys:1/body:3";
const STAR = "u:kestrel/sys:1/body:0";

test("a copy takes the body and everything that orbits it, parents first", () => {
  const tree = subtree(scene, TARN);
  assert.deepEqual(tree.map((s) => s.address), [TARN, `${TARN}/body:0`]);
  // The star takes the whole system, but never the temperate zone, which follows from the star.
  const all = subtree(scene, STAR);
  assert.ok(all.some((s) => s.kind === "belt"));
  assert.ok(!all.some((s) => s.kind === "zone"));
  assert.equal(all[0]!.address, STAR);
});

test("a clip keeps authored facts in the kernel's SI fields and leaves derived values to the physics", () => {
  const c = copyClip(scene, TARN, { packs: [{ id: "sse.base", version: "0.1.0" }] })!;
  assert.equal(c.clip.sseClip, 2);
  assert.equal(c.clip.generation, "v4");
  assert.deepEqual(c.clip.nodes.map((n) => [n.id, n.kind, n.parent]), [["n0", "planet", null], ["n1", "moon", "n0"]]);
  const tarn = scene.subjects.find((s) => s.address === TARN)!;
  const fact = (field: string) => c.clip.events.find((e) => e.node === "n0" && e.field === field)?.value;
  assert.equal(fact("name"), tarn.label);
  const a = tarn.facts.find((f) => f.field === "a_au")!.value as number;
  assert.ok(Math.abs((fact("orbit.semi_major_axis") as number) - a * AU_M) < 1);
  const m = tarn.facts.find((f) => f.field === "mass_earth")?.value as number | undefined;
  if (m !== undefined) assert.ok(Math.abs((fact("body.mass") as number) / M_EARTH - m) < 1e-9);
  assert.match(c.words, /with a moon$/);
  assert.equal(c.clip.packs[0]!.id, "sse.base");
  // The text is the clip, and reads back as itself.
  const back = readClip(c.text);
  assert.ok(back.ok);
  assert.deepEqual(back.ok && back.clip, c.clip);
});

test("reading a clip: V3 clips are recognised, other text is refused in words", () => {
  const v3 = readClip(JSON.stringify({ sseClip: 1, root: "a", nodes: [] }));
  assert.ok(!v3.ok && v3.v3);
  const junk = readClip("hello");
  assert.ok(!junk.ok && !junk.v3);
  assert.match(!junk.ok ? junk.why : "", /does not hold/);
  const future = readClip(JSON.stringify({ sseClip: 9 }));
  assert.match(!future.ok ? future.why : "", /newer/);
  const orphan = readClip(JSON.stringify({ sseClip: 2, root: "n0", nodes: [{ id: "n0", kind: "planet", parent: null }, { id: "n1", kind: "moon", parent: "n9" }], events: [] }));
  assert.ok(!orphan.ok);
});

test("packs a clip needs that the stack lacks are named, with the version in force", () => {
  const c = copyClip(scene, TARN, { packs: [{ id: "sse.base", version: "0.1.0" }, { id: "traveller", version: "1.0.0" }] })!.clip;
  assert.deepEqual(missingPacks(c, [{ id: "sse.base", version: "0.2.0" }]), [
    { id: "sse.base", version: "0.1.0", have: "0.2.0" },
    { id: "traveller", version: "1.0.0" },
  ]);
});

test("placing a clip gives new addresses, never ones in use, and moves only the root's orbit", () => {
  const c = copyClip(scene, TARN)!.clip;
  const taken = scene.subjects.map((s) => s.address);
  const p = placeClip(c, { system: "u:kestrel/sys:1", host: STAR, hostKind: "star", a_au: 2.5, taken });
  assert.equal(p.placed.length, 2);
  const [root, moon] = p.placed;
  assert.ok(!taken.includes(root!.address));
  assert.equal(root!.address, "u:kestrel/sys:1/body:6");
  assert.equal(root!.parent, STAR);
  assert.equal(moon!.address, `${root!.address}/body:0`);
  assert.equal(moon!.parent, root!.address);
  const sma = (a: string) => p.edits.find((e) => e.address === a && e.field === "orbit.semi_major_axis")?.value as number;
  assert.ok(Math.abs(sma(root!.address) - 2.5 * AU_M) < 1);
  const moonA = c.events.find((e) => e.node === "n1" && e.field === "orbit.semi_major_axis")?.value;
  assert.equal(sma(moon!.address), moonA);
  // Pasting twice never reuses an address.
  const again = placeClip(c, { system: "u:kestrel/sys:1", host: STAR, hostKind: "star", taken: [...taken, ...p.placed.map((x) => x.address)] });
  assert.equal(again.placed[0]!.address, "u:kestrel/sys:1/body:7");
});
