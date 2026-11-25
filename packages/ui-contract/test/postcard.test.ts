import { test } from "node:test";
import assert from "node:assert/strict";
import { bookmarksFrom, makeBookmark, type Whereabouts } from "../src/bookmark.ts";
import { checkPose, fovToFit, globeOfPose, poseFrom, poseOfGlobe, type Pose } from "../src/pose.ts";
import {
  DEFAULT_MESSAGE,
  MESSAGE_MAX,
  POSTCARD_SCHEMA,
  checkPostcard,
  clipAsSystem,
  clipThroughSeat,
  envelope,
  fixDate,
  forShop,
  goesWith,
  isPostcard,
  looksLikePostcard,
  makeMessage,
  packsNeeded,
  postcardBlocker,
  postcardFile,
  postcardHere,
  postcardLink,
  postcardText,
  readPostcard,
  shopRefusal,
  withMessage,
  type Postcard,
} from "../src/postcard.ts";
import type { Clip } from "../src/clip.ts";
import { postcardSetsOn } from "../src/postcard-sets.ts";
import { readFileSync } from "node:fs";

const pose: Pose = poseOfGlobe("u:sol/mars", 30, 20, 3.39e6);
const at: Whereabouts = { scope: "u:sol", now_yr: 4.6e9, instrument: "body", scale: "world", view: { style: "true", seen: "free", band: "visible" }, selected: "u:sol/mars", pose };
const address = { how: "address" as const, root: "u:sol", packs: [{ id: "base-physics", version: "4.0.0" }], made_with: "4.0.0" };

test("a pose reads leniently and comes back square", () => {
  assert.equal(checkPose(pose), null);
  const skew = poseFrom({ ...pose, eye_dir: [2, 0, 0], up: [1, 0, 1] })!;
  assert.deepEqual(skew.eye_dir, [1, 0, 0]);
  assert.ok(Math.abs(skew.up[0]) < 1e-12 && Math.abs(skew.up[2] - 1) < 1e-12, "up made square to the line of sight");
  for (const bad of [null, {}, { ...pose, eye_m: -1 }, { ...pose, up: pose.eye_dir }, { ...pose, fov_deg: 0 }, { ...pose, keep: [0.5, 0, 0.4, 1] }, { ...pose, exposure: { mode: "flash", ev: 0 } }]) assert.equal(poseFrom(bad), null);
});

test("the globe's turn and tip go into a pose and come back", () => {
  const g = globeOfPose(poseOfGlobe("x", -120, -35, 1));
  assert.ok(Math.abs(g.lon + 120) < 1e-9 && Math.abs(g.lat + 35) < 1e-9);
  // The renderer's viewRotation: toward the viewer is (cos lat cos lon, cos lat sin lon, sin lat).
  const p = poseOfGlobe("x", 90, 0, 1);
  assert.ok(Math.abs(p.eye_dir[1] - 1) < 1e-12 && Math.abs(p.up[2] - 1) < 1e-12);
});

test("the kept box decides the field of view on any screen shape", () => {
  const wide: Pose = { ...pose, keep: [0, 0.25, 1, 0.75] }; // twice as wide as tall
  assert.equal(fovToFit(wide, 2), wide.fov_deg);
  assert.equal(fovToFit(wide, 3), wide.fov_deg);
  assert.ok(fovToFit(wide, 0.5) > wide.fov_deg, "a phone held upright opens the view until the box fits");
});

test("a bookmark keeps the camera, and a bad camera is dropped, never the bookmark", () => {
  const b = makeBookmark([], at, "Mars", "Globe");
  assert.deepEqual(b.pose, pose);
  assert.ok(!("pose" in makeBookmark([], { ...at, pose: null }, "Mars", "Globe")));
  const [read] = bookmarksFrom([{ ...b, pose: { about: "u:sol/mars", eye_dir: "north" } }]);
  assert.equal(read?.name, "Mars, Globe");
  assert.ok(!("pose" in read!));
  assert.deepEqual(bookmarksFrom([b])[0]!.pose, pose);
});

test("inside a campaign a bookmark with a camera and a fixed date is a postcard", () => {
  const b = makeBookmark([], at, "Mars", "Globe");
  assert.ok(isPostcard(b));
  assert.ok(!isPostcard({ ...b, at_yr: null }));
  assert.ok(!isPostcard({ ...b, pose: undefined }));
  assert.match(postcardBlocker({ ...b, instrument: "packs" })!, /cannot draw a picture/);
  assert.match(postcardBlocker({ ...b, pose: undefined })!, /No camera/);
  assert.equal(fixDate({ ...b, at_yr: null }, 7).at_yr, 7);
});

test("made where I am, a postcard reads back as itself and round-trips every way it travels", () => {
  const p = postcardHere([], at, "Mars", "Globe", address, { credit: ["Alex"], caption: "Dusk over Tharsis." });
  assert.equal(p.schema, POSTCARD_SCHEMA);
  assert.equal(p.bookmark.at_yr, 4.6e9);
  const back = checkPostcard(JSON.parse(postcardText(p)));
  assert.ok(back.ok);
  assert.deepEqual(back.postcard, p);
  const link = postcardLink(p);
  assert.ok(link.ok);
  assert.match(link.url, /^https:\/\/v4\.starsystemx\.com\/#postcard=[A-Za-z0-9_-]+$/);
  assert.ok(link.url.length < 1500, `a link is short (${link.url.length})`);
  const fromLink = readPostcard(link.url);
  assert.ok(fromLink.ok);
  assert.deepEqual(fromLink.postcard, p);
  const f = postcardFile(p);
  assert.equal(f.name, "mars-globe.ssepostcard.json");
  assert.deepEqual((readPostcard(f.text) as { postcard: Postcard }).postcard, p);
  assert.ok(looksLikePostcard(f.text) && looksLikePostcard(link.url) && !looksLikePostcard('{"sseClip":2}'));
});

test("refused with a plain reason, never guessed at", () => {
  const p = postcardHere([], at, "Mars", "Globe", address);
  const cases: [unknown, RegExp][] = [
    [{ ...p, schema: "sse.postcard/2" }, /newer/],
    [{ ...p, bookmark: { ...p.bookmark, at_yr: null } }, /fixed date/],
    [{ ...p, bookmark: { ...p.bookmark, instrument: "packs" } }, /cannot draw/],
    [{ ...p, bookmark: { ...p.bookmark, pose: { about: "x" } } }, /camera/],
    [{ ...p, world: { how: "fax" } }, /world/],
    [{ ...p, world: { how: "address", packs: [] } }, /universe/],
  ];
  for (const [raw, why] of cases) {
    const r = checkPostcard(raw);
    assert.ok(!r.ok);
    assert.match(r.why, why);
  }
  assert.ok(!readPostcard("https://v4.starsystemx.com/#postcard=%%%").ok);
  assert.ok(!readPostcard("hello").ok);
});

test("a bad stamp or message is dropped, the postcard still opens", () => {
  const p = postcardHere([], at, "Mars", "Globe", address);
  const r = checkPostcard({ ...p, stamp: "data:image/svg+xml;base64,AAAA", message: { text: 3, from: "x" } });
  assert.ok(r.ok);
  assert.ok(!("stamp" in r.postcard) && !("message" in r.postcard));
});

test("a message is plain words for one sending: it opens as Wish you were here!, and never goes to the shop", () => {
  assert.deepEqual(makeMessage("   ", "Sam"), { text: DEFAULT_MESSAGE, from: "Sam" });
  assert.equal(makeMessage("x".repeat(400), "").text.length, MESSAGE_MAX);
  assert.equal(makeMessage("hi\u0007 <b>there</b>", "Sam").text, "hi <b>there</b>", "control characters go; markup stays as text");
  const p = withMessage(postcardHere([], at, "Mars", "Globe", address), makeMessage("Look at that!", "Sam"));
  const link = postcardLink(p);
  assert.ok(link.ok);
  assert.equal((readPostcard(link.url) as { postcard: Postcard }).postcard.message?.text, "Look at that!");
  assert.ok(!("message" in forShop(p)));
  assert.ok(!("message" in withMessage(p, null)));
});

const clip: Clip = {
  sseClip: 2,
  generation: "v4",
  source: { site: "sse4", title: "Kestrel", creditChain: ["Alex"] },
  packs: [{ id: "base-physics", version: "4.0.0" }],
  root: "n0",
  nodes: [
    { id: "n0", kind: "star", parent: null },
    { id: "n1", kind: "planet", parent: "n0" },
    { id: "n2", kind: "moon", parent: "n1" },
    { id: "n3", kind: "planet", parent: "n0" },
  ],
  events: [
    { kind: "authored.set", node: "n0", field: "name", value: "Kestrel" },
    { kind: "authored.set", node: "n1", field: "name", value: "Vault" },
    { kind: "authored.set", node: "n1", field: "body.mass", value: 6e24 },
    { kind: "authored.set", node: "n1", field: "atmosphere.secret_gas", value: 0.1 },
    { kind: "authored.set", node: "n2", field: "name", value: "Pip" },
    { kind: "authored.set", node: "n3", field: "name", value: "Hidden" },
  ],
};

test("an authored place goes through what the maker's seat can see", () => {
  const seen = clipThroughSeat(clip, { body: (n) => n.id !== "n3", fact: (_n, f) => f !== "atmosphere.secret_gas" })!;
  assert.deepEqual(seen.nodes.map((n) => n.id), ["n0", "n1", "n2"]);
  assert.ok(!seen.events.some((e) => e.field === "atmosphere.secret_gas" || e.node === "n3"));
  // A body the seat cannot see takes everything under it with it.
  assert.deepEqual(clipThroughSeat(clip, { body: (n) => n.id !== "n1", fact: () => true })!.nodes.map((n) => n.id), ["n0", "n3"]);
  assert.equal(clipThroughSeat(clip, { body: () => false, fact: () => true }), null);
  const p = envelope(fixDate(makeBookmark([], at, "Vault", "Globe"), 1), { how: "clip", clip: seen, packs: seen.packs, made_with: "4.0.0" });
  assert.deepEqual(goesWith(p).bodies, ["Kestrel", "Vault", "Pip"]);
  assert.match(goesWith(p).words, /Kestrel, Vault and Pip: 4 facts, nothing else/);
  assert.ok(checkPostcard(JSON.parse(postcardText(p))).ok);
  const l = postcardLink(p);
  assert.ok(!l.ok && /file/.test(l.why), "a clip is too much for a link");
});

test("the back lists the packs needed with their state; the shop takes only published places", () => {
  const p = postcardHere([], at, "Kestrel", "System", { ...address, packs: [{ id: "base-physics", version: "4.0.0" }, { id: "kestrel-reach", version: "1.0.0" }, { id: "traveller", version: "2.0.0" }] });
  const need = packsNeeded(p, [{ id: "base-physics", version: "4.0.0" }, { id: "traveller", version: "1.9.0" }], (id) => (id === "kestrel-reach" ? "paid" : null));
  assert.deepEqual(need.map((n) => [n.id, n.state, n.have]), [["base-physics", "have", undefined], ["kestrel-reach", "paid", undefined], ["traveller", "other-version", "1.9.0"]]);
  assert.match(shopRefusal(p, (id) => id !== "kestrel-reach")!, /does not hold kestrel-reach/);
  assert.equal(shopRefusal(p, () => true), null);
  const authored = envelope(fixDate(makeBookmark([], at, "Vault", "Globe"), 1), { how: "clip", clip, packs: [], made_with: "4.0.0" });
  assert.match(shopRefusal(authored, () => true)!, /publish this world first/);
  assert.match(shopRefusal(envelope(fixDate(makeBookmark([], at, "x", "y"), 1), { how: "here" }), () => true)!, /packed/);
});

test("nothing about players travels", () => {
  const p = postcardHere([], at, "Mars", "Globe", address);
  const text = postcardText(p);
  for (const k of ["role", "look", "known_to", "notes", "audience"]) assert.ok(!text.includes(`"${k}"`), k);
});

test("a reference shot keeps its Composite mix, its stretch and the famous picture it is compared against", () => {
  const p = postcardHere([], { ...at, pose: { ...pose, stretch: "asinh" }, view: { style: "true", seen: "free", mix: [{ band: "visible", line: "H-alpha", weight: 1, colour: "#ff8800" }, { band: "visible", line: "O III", weight: 0.8, colour: "#0088ff" }] } }, "Pillars", "Body", address);
  const withRef = { ...p, reference: { title: "Pillars of Creation", credit: "NASA, ESA, Hubble", licence: "public domain", url: "https://example.org/pillars", taken_at: "1995-04-01" } };
  const r = readPostcard(postcardText(withRef as Postcard));
  assert.ok(r.ok);
  assert.equal(r.postcard.bookmark.pose?.stretch, "asinh");
  assert.deepEqual(r.postcard.bookmark.view?.mix?.map((m) => m.line), ["H-alpha", "O III"]);
  assert.equal(r.postcard.reference?.taken_at, "1995-04-01");
  const bad = checkPostcard({ ...withRef, bookmark: { ...withRef.bookmark, view: { ...withRef.bookmark.view, mix: [{ band: "visible", line: "Ne V", weight: 1, colour: "#fff" }] } } });
  assert.ok(bad.ok && !bad.postcard.bookmark.view?.mix, "a bad mix is dropped, the postcard still opens");
  assert.equal(poseFrom({ ...pose, stretch: "log" }), null);
});

test("a visit lays a clip out as a system of its own, the star on the system's own address", () => {
  const { edits, addressOf } = clipAsSystem(clip, "u:visit/sys:1", ["u:visit/sys:1/body:0"], 1.4e17);
  assert.equal(addressOf.get("n0"), "u:visit/sys:1");
  assert.equal(addressOf.get("n1"), "u:visit/sys:1/body:1", "a taken address is passed over");
  assert.equal(addressOf.get("n2"), "u:visit/sys:1/body:1/body:0");
  assert.ok(edits.some((e) => e.address === "u:visit/sys:1" && e.field === "name" && e.value === "Kestrel"));
  assert.ok(edits.some((e) => e.address === "u:visit/sys:1" && e.field === "system.age" && e.value === 1.4e17));
  assert.ok(edits.some((e) => e.address === "u:visit/sys:1/body:1/body:0" && e.field === "orbit.host" && e.value === "u:visit/sys:1/body:1"));
  assert.ok(!edits.some((e) => e.address === "u:visit/sys:1" && e.field.startsWith("orbit.")), "the star orbits nothing");
});

test("a postcard's origin carries the world's own facts, never notes, and only at its own addresses", () => {
  const b = fixDate(makeBookmark([], at, "Vault", "Body"), 4.6e9);
  const origin = {
    universe: { id: "kestrel", seed: "kestrel reach campaign" },
    system: "u:kestrel/sys:1",
    nodes: { n0: "u:kestrel/sys:1", n1: "u:kestrel/sys:1/body:3", n9: "u:kestrel/sys:1/body:9" },
    facts: [
      { address: "u:kestrel/sys:1/body:3", field: "body.surface_water_oceans", value: 1 },
      { address: "u:kestrel/sys:1/body:3", field: "v3.gmNotes", value: "the vault holds the key" },
      { address: "u:kestrel/sys:1/body:3", field: "v3.roleHint", value: "planet" },
      { address: "u:elsewhere/sys:2", field: "body.mass", value: 1 },
    ],
  };
  const r = checkPostcard(JSON.parse(JSON.stringify(envelope(b, { how: "clip", clip, packs: clip.packs, made_with: "4.0.0", about: "n1", age_s: 1.4e17, origin }))));
  assert.ok(r.ok, r.ok ? "" : r.why);
  const w = r.postcard.world;
  assert.ok(w.how === "clip" && w.origin);
  assert.equal(w.about, "n1");
  assert.deepEqual(w.origin.nodes, { n0: "u:kestrel/sys:1", n1: "u:kestrel/sys:1/body:3" }, "only the clip's own nodes");
  assert.deepEqual(w.origin.facts.map((f) => f.field), ["body.surface_water_oceans"], "no notes, no import bookkeeping, nothing from another system");
});

test("postcard sets read from packs; a later pack's set of the same id replaces it", () => {
  const b = fixDate(makeBookmark([], at, "Mars", "Body"), 4.6e9);
  const card = JSON.parse(postcardText(envelope(b, address, { id: "c1", name: "Mars" })));
  const pack = (id: string, sets: unknown) => ({ id, version: "1.0.0", provides: { other: { postcard_sets: sets } } }) as never;
  const r = postcardSetsOn([
    pack("a", [{ schema: "sse.postcards/1", id: "s", title: "First", cards: [card, { schema: "nope" }] }]),
    pack("b", [{ schema: "sse.postcards/1", id: "s", title: "Second", cards: [card] }, { id: "bad" }]),
    pack("c", "not a list"),
  ]);
  assert.deepEqual(r.sets.map((s) => [s.id, s.title, s.pack, s.cards.length]), [["s", "Second", "b", 1]]);
  assert.equal(r.problems.length, 3, r.problems.join("; "));
});

test("the base set in the base presentation pack reads, every card with its stamp and origin", () => {
  const doc = JSON.parse(readFileSync(new URL("../../../packs/base-presentation/pack.json", import.meta.url), "utf8"));
  const r = postcardSetsOn([doc]);
  assert.deepEqual(r.problems, []);
  const set = r.sets.find((s) => s.id === "sse.base");
  assert.ok(set && set.cards.length >= 3);
  for (const c of set.cards) {
    assert.match(c.stamp ?? "", /^data:image\//, `${c.name} has its stamp`);
    assert.ok(c.world.how === "clip" && c.world.origin, `${c.name} carries its origin`);
    assert.equal(c.message, undefined, "a set's postcards carry no message");
  }
});
