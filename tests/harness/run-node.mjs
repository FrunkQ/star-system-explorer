// Run the wasm harness in Node and compare with the golden file. Exit 1 on mismatch.
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";
const here = dirname(fileURLToPath(import.meta.url));
const { initSync, harness_run } = await import(join(here, "pkg/sse_wasm.js"));
initSync({ module: readFileSync(join(here, "pkg/sse_wasm_bg.wasm")) });
const r = JSON.parse(harness_run());
const g = JSON.parse(readFileSync(join(here, "../golden/harness.json"), "utf8"));
const ok = r.overall === g.overall && r.encoding_failures === 0;
console.log(`node ${process.version}: ${ok ? "PASS" : "FAIL"} overall ${r.overall}`);
process.exit(ok ? 0 : 1);
