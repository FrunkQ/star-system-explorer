// Where the object menu opens (V3 gap 38): at the point it was asked for, kept on screen.
import { test } from "node:test";
import assert from "node:assert/strict";
import { placeMenu } from "../../packages/shell/src/object-menu.ts";

test("the menu opens at the point when it fits", () => {
  assert.deepEqual(placeMenu(100, 100, 220, 300, 1440, 900), { left: 100, top: 100 });
});

test("near the right edge it moves left; near the bottom it opens above the point", () => {
  assert.deepEqual(placeMenu(1400, 800, 220, 300, 1440, 900), { left: 1440 - 220 - 8, top: 500 });
});

test("taller than the room either side, it sits as high as it can", () => {
  assert.deepEqual(placeMenu(50, 200, 220, 700, 1440, 760), { left: 50, top: 52 });
  assert.deepEqual(placeMenu(50, 200, 220, 900, 1440, 760), { left: 50, top: 8 });
});
