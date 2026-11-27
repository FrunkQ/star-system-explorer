// Compound ships, CM7: the designer places each construct by its mount (design section 8.2). Checks: front and behind
// stack end to end on the axis; a bay is sized from what it holds (or the kernel's typed size wins) and holds it; a
// construct on the hull touches the carrier's surface along its clock line; outside adds the stand-off and a tether;
// placing moves and turns, never resizes; fit notes warn about what does not fit (never refuse); the kernel's facts
// read back as mounts, ports and bays; turned panels and port rings draw in the plans and the model.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { designShip, shipViews, type DesignBay, type DesignRules, type DesignShip, type Piece, type ShipLayout, type ShipStyle } from "../src/ship-design.ts";
import { baysFromFacts, designCompound, mountFromFacts, portsFromFacts, reachOf, ringPoints, type CompoundMember, type CompoundRules } from "../src/compound-layout.ts";
import { compoundMesh, shipMesh } from "../src/ship-mesh.ts";
import { type Propellants, type ShipRecord } from "../src/ship.ts";

const json = (p: string) => JSON.parse(readFileSync(new URL(p, import.meta.url), "utf8"));
const SHIPS = json("../../../packs/base-nav/pack.json").provides.other.nav.ships as Record<string, ShipRecord>;
const PACK = json("../../../packs/base-ships/pack.json").provides.other;
const RULES = PACK.ship_design as DesignRules;
const C = PACK.ship_design.compound as CompoundRules;
const STYLES = PACK.ship_styles as Record<string, ShipStyle>;
// Test fixtures only: NIST saturated liquids at boiling (as ship-design.test.ts).
const STORE: Propellants = {
  "hydrogen-oxygen": [
    { name: "liquid hydrogen", share: 1 / 7, kg_m3: 70.85, src: "nist-webbook" },
    { name: "liquid oxygen", share: 6 / 7, kg_m3: 1141, src: "nist-webbook" },
  ],
  hydrogen: [{ name: "liquid hydrogen", share: 1, kg_m3: 70.85, src: "nist-webbook" }],
  deuterium: [{ name: "liquid deuterium", share: 1, kg_m3: 163.5, src: "nist-webbook" }],
  "helium-3": [{ name: "liquid helium-3", share: 1, kg_m3: 59, src: "kerr1954" }],
};
// Stand-in craft from the base-nav ships, resized by mass: a crewed capsule, two stages, a small lander, boosters.
const craft: Record<string, DesignShip> = {
  csm: { ...SHIPS["chemical-transfer"]!, name: "Capsule", wet_t: 30, dry_t: 12, crew: 3 },
  stage3: { ...SHIPS["chemical-transfer"]!, name: "Third stage", wet_t: 120, dry_t: 12, crew: 0 },
  stage2: { ...SHIPS["chemical-transfer"]!, name: "Second stage", wet_t: 480, dry_t: 40, crew: 0 },
  lander: { ...SHIPS["chemical-transfer"]!, name: "Lander", wet_t: 15, dry_t: 4, crew: 2 },
  booster: { ...SHIPS["chemical-transfer"]!, name: "Booster", wet_t: 60, dry_t: 8, crew: 0 },
  probe: { ...SHIPS["chemical-transfer"]!, name: "Probe", wet_t: 2, dry_t: 1, crew: 0 },
  torch: { ...SHIPS["fusion-torch"]!, name: "Torch", crew: 6 },
};
const design = (m: CompoundMember, bays: DesignBay[]): ShipLayout => designShip({ ...craft[m.address]!, bays }, STORE, RULES, "spine", STYLES.spine!, 3);
const alone = (id: string) => designShip(craft[id]!, STORE, RULES, "spine", STYLES.spine!, 3);
const near = (a: number, b: number, tol: number, what: string) => assert.ok(Math.abs(a - b) <= tol, `${what}: ${a} against ${b}`);
const of = (pieces: Piece[], m: string) => pieces.filter((p) => p.member === m);
const bodyVol = (ps: Piece[]) => ps.filter((p) => p.shape === "body" && p.system !== "plume").reduce((a, p) => a + (Math.PI / 3) * (p.x1 - p.x0) * (p.r0 * p.r0 + p.r0 * p.r1 + p.r1 * p.r1), 0);
const span = (ps: Piece[]) => [Math.min(...ps.filter((p) => p.shape === "body").map((p) => p.x0)), Math.max(...ps.filter((p) => p.shape === "body").map((p) => p.x1))] as const;

const STACK: CompoundMember[] = [
  { address: "csm", name: "Capsule" },
  { address: "stage3", name: "Third stage", mount: { on: "csm", place: "behind" } },
  { address: "stage2", name: "Second stage", mount: { on: "stage3", place: "behind" } },
];

test("front and behind stack end to end on the axis, the root's identity kept", () => {
  const c = designCompound(STACK, design, C);
  assert.equal(c.root, "csm");
  const [a0] = span(of(c.pieces, "csm"));
  const [, b1] = span(of(c.pieces, "stage3"));
  const [c0, c1] = span(of(c.pieces, "stage2"));
  near(b1, a0, 1e-9, "third stage's nose at the capsule's stern");
  near(c1, span(of(c.pieces, "stage3"))[0], 1e-9, "second stage's nose at the third's stern");
  near(c0, 0, 1e-9, "the compound's stern is at 0");
  near(c.length_m, alone("csm").length_m + alone("stage3").length_m + alone("stage2").length_m, 1e-6, "length is the sum");
  for (const p of c.pieces.filter((p) => p.system === "drive" || p.system === "crew")) near(Math.hypot(p.y, p.z), 0, 1e-9, `${p.member} ${p.system} on the axis`);
  // The plume comes from the bottom stage: its pieces lead and its needs stand for the compound's.
  assert.equal(c.pieces[0]!.member, "stage2");
  assert.deepEqual(c.needs, alone("stage2").needs);
  assert.deepEqual(designCompound(STACK, design, C), c, "same mounts, same picture");
});

test("placing moves and turns a construct, never resizes it", () => {
  const members: CompoundMember[] = [...STACK, { address: "booster", name: "Booster", mount: { on: "stage2", place: "hull", clock_deg: 90, along_m: 10 } }, { address: "lander", name: "Lander", mount: { on: "stage3", place: "inside" } }];
  const c = designCompound(members, design, C);
  for (const id of ["csm", "stage2", "booster", "lander"]) near(bodyVol(of(c.pieces, id)), bodyVol(alone(id).pieces), 1e-6 * bodyVol(alone(id).pieces), `${id} volume`);
  assert.equal(c.members.length, members.length);
});

test("a bay is sized from what it holds, holds it, and the kernel's typed size wins", () => {
  const members: CompoundMember[] = [{ address: "stage3", name: "Third stage" }, { address: "lander", name: "Lander", mount: { on: "stage3", place: "inside" } }];
  const c = designCompound(members, design, C);
  const bay = c.pieces.find((p) => p.system === "bay")!;
  const lander = alone("lander");
  near(bay.x1 - bay.x0, lander.length_m * C.bay_clearance, 1e-9, "bay length");
  near(bay.r0, reachOf(lander) * C.bay_clearance, 1e-9, "bay radius");
  assert.match(bay.label!, /holds Lander/);
  assert.ok(c.notes.some((n) => /has no bay part/.test(n)), "a bay drawn without a bay part says so");
  const [l0, l1] = span(of(c.pieces, "lander"));
  assert.ok(l0 >= bay.x0 - 1e-9 && l1 <= bay.x1 + 1e-9, "the lander lies inside the bay");
  for (const p of of(c.pieces, "lander").filter((p) => p.shape === "body")) assert.ok(Math.hypot(p.y - bay.y, p.z - bay.z) + Math.max(p.r0, p.r1) <= bay.r0 + 1e-9, "and inside its radius");
  // Typed by the kernel, too small: drawn at its size, with notes; nothing refused.
  const tight = designCompound([{ ...members[0]!, bays: [{ id: "adapter", length_m: 1, radius_m: 0.5, door_m: 0.4 }] }, { ...members[1]!, mount: { on: "stage3", place: "inside", bay: "adapter" } }], design, C);
  const tb = tight.pieces.find((p) => p.system === "bay")!;
  assert.equal(tb.part, "adapter");
  near(tb.x1 - tb.x0, 1, 1e-12, "typed length wins");
  for (const re of [/longer than its room/, /wider than/, /door is 0\.4 m across/]) assert.ok(tight.notes.some((n) => re.test(n)), `note ${re}`);
  assert.ok(!tight.notes.some((n) => /has no bay part/.test(n)));
});

test("several in one bay sit round each layer and along it", () => {
  const n = 6;
  const members: CompoundMember[] = [{ address: "torch", name: "Torch", bays: [{ id: "hangar" }] }, ...Array.from({ length: n }, (_, i) => ({ address: `probe`, name: `Probe ${i + 1}`, mount: { on: "torch", place: "inside" as const } }))];
  // Six of the same craft, each its own construct.
  const ms = members.map((m, i) => (i ? { ...m, address: `probe${i}` } : m));
  const c = designCompound(ms, (m, bays) => designShip({ ...craft[m.address.replace(/\d+$/, "")]!, bays }, STORE, RULES, "spine", STYLES.spine!, 3), C);
  const bay = c.pieces.find((p) => p.system === "bay")!;
  const probe = alone("probe");
  const per = Math.min(n, C.bay_per_layer);
  near(bay.x1 - bay.x0, Math.ceil(n / per) * probe.length_m * C.bay_clearance, 1e-9, "layers along the bay");
  assert.ok(!c.notes.some((x) => /overlap|longer than|wider than/.test(x)), `no clashes: ${c.notes.join(" | ")}`);
});

test("on the hull a construct touches the carrier's surface along its clock line; strap-ons must balance", () => {
  const carrier = alone("stage2");
  const two: CompoundMember[] = [{ address: "stage2", name: "Core" }, ...[90, 270].map((clock, i) => ({ address: i ? "booster" : "probe", name: `Booster ${i}`, mount: { on: "stage2", place: "hull" as const, bond: "fixed", clock_deg: clock, along_m: carrier.length_m / 2 } }))];
  const des = (m: CompoundMember, bays: DesignBay[]) => designShip({ ...craft.booster!, bays }, STORE, RULES, "spine", STYLES.spine!, 3);
  const c = designCompound(two, (m, b) => (m.address === "stage2" ? design(m, b) : des(m, b)), C);
  const booster = designShip(craft.booster!, STORE, RULES, "spine", STYLES.spine!, 3);
  const core = of(c.pieces, "stage2");
  const xc = c.members.find((m) => m.address === "stage2")!.offset[0] + carrier.length_m / 2;
  // The core's surface along +y at the middle: its farthest body there.
  const surface = Math.max(...core.filter((p) => p.shape === "body" && p.system !== "plume" && p.x0 <= xc && p.x1 >= xc).map((p) => (Math.abs(p.z) < p.r0 ? p.y + Math.sqrt(p.r0 * p.r0 - p.z * p.z) : -Infinity)));
  const m = c.members.find((x) => x.name === "Booster 0")!;
  // Turned to clock 90, its own up faces +y; its belly is its own reach below its axis.
  const belly = Math.max(...booster.pieces.filter((p) => p.shape === "body" && p.system !== "plume").map((p) => Math.max(p.r0, p.r1) - p.z));
  near(m.offset[1], surface + belly, 1e-6, "touching the hull");
  near(m.offset[2], 0, 1e-9, "at clock 90, beside it");
  near(m.roll_deg, 90, 1e-12, "turned to face out");
  near(m.offset[0] + booster.length_m / 2, xc, 1e-9, "centred at its station along");
  assert.ok(!c.notes.some((n) => /not balanced/.test(n)), "two at 90 and 270 balance");
  const one = designCompound(two.slice(0, 2), (m, b) => (m.address === "stage2" ? design(m, b) : des(m, b)), C);
  assert.ok(one.notes.some((n) => /not balanced/.test(n)), "one alone would turn the stack");
  const latched = designCompound([two[0]!, { ...two[1]!, mount: { ...two[1]!.mount!, bond: "latch" } }], (m, b) => (m.address === "stage2" ? design(m, b) : des(m, b)), C);
  assert.ok(!latched.notes.some((n) => /not balanced/.test(n)), "a latch takes no thrust, so a docked craft never counts");
  // Overlap: two at the same station clash, and are told so.
  const clash = designCompound([two[0]!, two[1]!, { ...two[2]!, mount: { ...two[2]!.mount!, clock_deg: 90 } }], (m, b) => (m.address === "stage2" ? design(m, b) : des(m, b)), C);
  assert.ok(clash.notes.some((n) => /overlap/.test(n)), clash.notes.join(" | "));
});

test("outside adds the stand-off and draws the tether; a missing station is drawn and said", () => {
  const off = 12;
  const base: CompoundMember[] = [{ address: "stage3", name: "Crane" }, { address: "probe", name: "Rover", mount: { on: "stage3", place: "outside", clock_deg: 180, along_m: 1, stand_off_m: off } }];
  const c = designCompound(base, design, C);
  const t = c.pieces.find((p) => p.system === "tether")!;
  near(t.span!, off, 1e-12, "tether as long as the stand-off");
  const hull = designCompound([base[0]!, { ...base[1]!, mount: { ...base[1]!.mount!, place: "hull", stand_off_m: undefined } }], design, C);
  const dz = hull.members[1]!.offset[2] - c.members[1]!.offset[2];
  near(dz, off, 1e-6, "held clear by the stand-off, below at clock 180");
  const loose = designCompound([base[0]!, { address: "probe", name: "Rover", mount: { on: "stage3", place: "hull" } }], design, C);
  assert.ok(loose.notes.some((n) => /not given; it is drawn at the middle/.test(n)));
  const orphan = designCompound([{ address: "probe", name: "Rover", mount: { on: "elsewhere", place: "hull" } }], design, C);
  assert.ok(orphan.notes.some((n) => /not in this picture/.test(n)));
});

test("the kernel's facts read back as mounts, ports and bays; a dock is a latch on the hull", () => {
  assert.deepEqual(mountFromFacts({ "mount.on": "u:1/c:csm", "mount.place": "inside", "mount.bay": "sla", "mount.bond": "rest", "mount.lines": ["power"], "mount.station.clock_deg": 0 }), { on: "u:1/c:csm", place: "inside", bay: "sla", bond: "rest", lines: ["power"], clock_deg: 0 });
  assert.deepEqual(mountFromFacts({ "docked.to": "u:1/c:iss" }), { on: "u:1/c:iss", place: "hull", bond: "latch", lines: [] });
  assert.equal(mountFromFacts({ "mount.on": "x", "mount.place": "sideways" }), undefined);
  assert.deepEqual(portsFromFacts({ "port.nose.place": "front", "port.nose.size_m": 0.8, "port.nose.lines": ["people", "air"], "port.aft.place": "behind" }), [
    { id: "aft", place: "behind", lines: [] },
    { id: "nose", place: "front", lines: ["people", "air"], size_m: 0.8 },
  ]);
  assert.deepEqual(baysFromFacts({ "part.sla.bay_length_m": 8.5, "part.sla.bay_radius_m": 3.3, "part.sla.mass_kg": 1800, "part.tank.mass_kg": 5 }), [{ id: "sla", length_m: 8.5, radius_m: 3.3 }]);
});

test("ports are rings on the hull, drawn in the plans and the model; turned panels draw at their slant", () => {
  const members: CompoundMember[] = [
    { address: "torch", name: "Torch", ports: [{ id: "nose", place: "front", size_m: 2 }, { id: "a", place: "hull", size_m: 2 }, { id: "b", place: "hull", size_m: 2 }] },
    { address: "probe", name: "Probe", mount: { on: "torch", place: "hull", port: "a", along_m: 20 } },
  ];
  const c = designCompound(members, (m, b) => designShip({ ...craft[m.address]!, bays: b }, STORE, RULES, "spine", STYLES.spine!, 3), C);
  assert.equal(c.ports.length, 3);
  const nose = c.ports.find((p) => p.id === "nose")!;
  near(nose.at[0], c.members[0]!.offset[0] + designShip(craft.torch!, STORE, RULES, "spine", STYLES.spine!, 3).length_m, 1e-9, "nose port at the nose");
  assert.deepEqual(nose.normal, [1, 0, 0]);
  assert.equal(c.ports.find((p) => p.id === "a")!.used_by, "probe");
  for (const p of c.ports)
    for (const q of ringPoints(p)) {
      const d = [q[0] - p.at[0], q[1] - p.at[1], q[2] - p.at[2]];
      near(Math.hypot(...d), p.r_m, 1e-9, "on the ring");
      near(d[0]! * p.normal[0] + d[1]! * p.normal[1] + d[2]! * p.normal[2], 0, 1e-9, "across its face");
    }
  const v = shipViews(c, [560, 220]);
  assert.equal(v.side.paths.filter((p) => p.system === "port").length, 3);
  const mesh = compoundMesh(c, { segments: 16, panel_m: 2 });
  assert.ok(mesh.triangles > shipMesh(c, { segments: 16, panel_m: 2 }).triangles, "the rings add to the model");
  // A torch on another's hull at clock 90: its radiator wings turn with it, so they reach up and down, not sideways.
  const turned = designCompound([{ address: "stage2", name: "Core" }, { address: "torch", name: "Torch", mount: { on: "stage2", place: "hull", clock_deg: 90, along_m: 5 } }], design, C);
  const wing = turned.pieces.find((p) => p.member === "torch" && p.system === "radiators")!;
  near(wing.roll_deg!, 90, 1e-12, "the wing's roll is the construct's");
  const tm = shipMesh({ ...turned, pieces: [wing] }, { segments: 8, panel_m: 2 });
  const pos = tm.surfaces.radiators!.positions;
  let dy = 0;
  let dz = 0;
  for (let k = 0; k < pos.length; k += 3) (dy = Math.max(dy, Math.abs(pos[k + 1]! - wing.y))), (dz = Math.max(dz, Math.abs(pos[k + 2]! - wing.z)));
  near(dz, Math.abs(wing.span!), 1e-3 * Math.abs(wing.span!), "turned wing reaches along z");
  assert.ok(dy < 0.01 * Math.abs(wing.span!), "and is thin across y");
  const tv = shipViews(turned, [560, 220]);
  assert.ok(tv.side.paths.length > 0 && tv.top.paths.length > 0);
});
