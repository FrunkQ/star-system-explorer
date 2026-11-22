import { createRequire } from "node:module"; const { chromium } = createRequire("/opt/node-tools/node_modules/")("playwright");
import { readFileSync } from "node:fs";
const b64 = readFileSync(new URL("./target/wasm32-unknown-unknown/release/gh0bench.wasm", import.meta.url)).toString("base64");
const browser = await chromium.launch();
for (const rate of [1, 4]) {
  const page = await browser.newPage();
  const cdp = await page.context().newCDPSession(page);
  await cdp.send("Emulation.setCPUThrottlingRate", { rate });
  const out = await page.evaluate(async (b64) => {
    const bytes = Uint8Array.from(atob(b64), (c) => c.charCodeAt(0));
    const { instance } = await WebAssembly.instantiate(bytes, {});
    const e = instance.exports, res = [];
    const t = (l, f) => { const a = performance.now(); f(); res.push(`${l}: ${(performance.now() - a).toFixed(0)} ms`); };
    t("setup", () => e.setup());
    t("tree chart counts", () => e.tree(1e-3, 0));
    t("tree chart + G3", () => e.tree(1e-3, 1000));
    t("tree survey counts", () => e.tree(1e-5, 0));
    t("orbits 55x14000", () => e.orbits(55, 14000));
    t("orbits 500x14000", () => e.orbits(500, 14000));
    return res;
  }, b64);
  console.log(`CPU x${rate}:`, out.join("; "));
  await page.close();
}
await browser.close();
