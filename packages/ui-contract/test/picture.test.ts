import { test } from "node:test";
import assert from "node:assert/strict";
import { checkPicture, designOf, fitSide, pictureOf, PICTURE_MAX } from "../src/picture.ts";

test("a picture is a small raster data URI", () => {
  assert.equal(checkPicture("data:image/webp;base64,AAAA"), null);
  assert.match(checkPicture("data:image/svg+xml;base64,AAAA")!, /PNG, JPEG or WebP/);
  assert.match(checkPicture(`data:image/png;base64,${"A".repeat(PICTURE_MAX)}`)!, /too big/);
  assert.equal(pictureOf(""), null);
  assert.equal(pictureOf(42), null);
  assert.equal(pictureOf("data:image/jpeg;base64,AAAA"), "data:image/jpeg;base64,AAAA");
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
