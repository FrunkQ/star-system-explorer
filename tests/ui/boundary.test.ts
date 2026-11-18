import { test } from "node:test";
import assert from "node:assert/strict";
import { fileURLToPath } from "node:url";
import { readFileSync } from "node:fs";
import { checkRepo, checkSource } from "./boundary.ts";

const root = fileURLToPath(new URL("../../", import.meta.url));

test("the shell and views import only the client API", () => {
  const { files, breaches } = checkRepo(root);
  assert.ok(files >= 20, `expected the UI sources, found ${files} files`);
  assert.deepEqual(breaches, []);
});

test("the checker catches the private paths it is meant to catch", () => {
  const bad = [
    `import { KernelClient } from "../../kernel-client/src/index.ts";`,
    `import type { Request } from "../../contracts/src/index.ts";`,
    `import { Session } from "../../ui-contract/src/session.ts";`,
    `const w = await import("../../kernel-client/wasm/sse_wasm.js");`,
    `import x from "@sse/kernel-client";`,
    `client.callRaw(JSON.stringify({ op: "set_authored", edit }));`,
  ];
  for (const src of bad) assert.ok(checkSource(root, "packages/shell/src/x.ts", src).length > 0, src);
  const good = [
    `import { gmSeat } from "../../ui-contract/src/index.ts";`,
    `import { openBrowserSession } from "../../ui-contract/src/host-browser.ts";`,
    `import { mount } from "svelte";`,
    `import "@fontsource/newsreader/400.css";`,
    `import { Card } from "../../chart-grammar/src/index.ts";`,
  ];
  for (const src of good) assert.deepEqual(checkSource(root, "packages/shell/src/x.ts", src), [], src);
});

test("the client API's public index exposes no kernel client and no request shapes", () => {
  const index = readFileSync(new URL("packages/ui-contract/src/index.ts", `file://${root}`), "utf8");
  for (const line of index.split("\n").filter((l) => l.includes("session.ts"))) {
    assert.ok(line.startsWith("export type "), `only types may come from session.ts: ${line}`);
  }
  for (const name of ["Session ", "KernelClient", "intentToRequest", "encodeRequest"]) {
    assert.ok(!new RegExp(`export \\{[^}]*\\b${name.trim()}\\b`).test(index), `${name} must not be exported`);
  }
});
