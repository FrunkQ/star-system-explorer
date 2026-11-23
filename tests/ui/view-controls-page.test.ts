import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { pageOnDisk, viewControlsPage } from "./view-controls-page.ts";

test("the view controls page says what the screens offer (node tests/ui/view-controls-page.ts --write)", () => {
  assert.equal(pageOnDisk(), viewControlsPage());
});

test("the page's copy of Galaxies nearby's inline controls matches the screen", () => {
  const src = readFileSync(new URL("../../packages/views/cluster-map/src/index.svelte.ts", import.meta.url), "utf8");
  assert.match(src, /controls: flatChart\("3D cells come with the galaxy charts' Free view"\)/);
});
