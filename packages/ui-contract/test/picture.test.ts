import { test } from "node:test";
import assert from "node:assert/strict";
import { checkPicture, designOf, fitSide, picturesOf, shownOf, PICTURE_MAX, PICTURES_MAX } from "../src/picture.ts";

test("a picture is a small raster data URI", () => {
  assert.equal(checkPicture("data:image/webp;base64,AAAA"), null);
  assert.match(checkPicture("data:image/svg+xml;base64,AAAA")!, /PNG, JPEG or WebP/);
  assert.match(checkPicture(`data:image/png;base64,${"A".repeat(PICTURE_MAX)}`)!, /too big/);
  assert.deepEqual(picturesOf(["", 42, "data:image/jpeg;base64,AAAA"]), ["data:image/jpeg;base64,AAAA"]);
  assert.deepEqual(picturesOf("data:image/jpeg;base64,AAAA"), []);
  assert.equal(picturesOf(Array(9).fill("data:image/png;base64,AA")).length, PICTURES_MAX);
});

test("a design recipe reads back, or not at all", () => {
  assert.deepEqual(designOf({ style: "spine", seed: 3, junk: 1 }), { style: "spine", seed: 3 });
  assert.deepEqual(designOf({ style: "spine", seed: 3, overrides: { radiators: 2 } }), { style: "spine", seed: 3, overrides: { radiators: 2 } });
  assert.equal(designOf({ style: "spine", seed: 1.5 }), null);
  assert.equal(designOf("spine"), null);
});

test("pictures scale down to fit, never up", () => {
  assert.deepEqual(fitSide(3000, 1500), [960, 480]);
  assert.deepEqual(fitSide(400, 300), [400, 300]);
});

test("it opens on the last choice while that still exists", () => {
  const ship = ["plans", "model"];
  assert.equal(shownOf("model", 0, ship), "model");
  assert.equal(shownOf("model", 2, []), "picture:0");
  assert.equal(shownOf("picture:1", 2, ship), "picture:1");
  assert.equal(shownOf("picture:2", 2, ship), "plans");
  assert.equal(shownOf(undefined, 0, ship), "plans");
  assert.equal(shownOf(undefined, 0, []), null);
  assert.equal(shownOf("globe", 0, ["chart", "globe"]), "globe");
});
