// Serve tests/harness and open it in every Playwright browser available.
// Usage: node tests/harness/run-browsers.mjs [chromium firefox webkit]
import { createServer } from "node:http";
import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { dirname, join, extname, normalize } from "node:path";
import { createRequire } from "node:module";
const require = createRequire(import.meta.url);
const pw = require("playwright");
const root = dirname(fileURLToPath(import.meta.url));
const types = { ".html": "text/html", ".js": "text/javascript", ".wasm": "application/wasm", ".json": "application/json", ".jsonl": "text/plain" };
const server = createServer(async (req, res) => {
  const p = normalize(join(root, decodeURIComponent(new URL(req.url, "http://x").pathname)));
  const file = p.endsWith("/") ? join(p, "index.html") : p;
  if (!file.startsWith(root)) { res.writeHead(403).end(); return; }
  let body;
  try { body = await readFile(file); } catch { res.writeHead(404).end(); return; }
  res.writeHead(200, { "content-type": types[extname(file)] ?? "application/octet-stream" }).end(body);
}).listen(0);
const url = `http://127.0.0.1:${server.address().port}/`;
const wanted = process.argv.slice(2).length ? process.argv.slice(2) : ["chromium", "firefox", "webkit"];
let failed = 0, ran = 0;
for (const name of wanted) {
  let browser;
  try { browser = await pw[name].launch(); } catch (e) { console.log(`${name}: not installed, skipped`); continue; }
  const page = await browser.newPage();
  await page.goto(url);
  await page.waitForFunction(() => document.body.dataset.result, null, { timeout: 120000 });
  const [result, overall, kernel] = await page.evaluate(() => [document.body.dataset.result, document.body.dataset.overall, document.getElementById("kverdict")?.textContent]);
  console.log(`${name} ${browser.version()}: ${result.toUpperCase()} overall ${overall}\n  ${kernel}`);
  if (process.env.SHOT) await page.setViewportSize({ width: 390, height: 900 }), await page.screenshot({ path: process.env.SHOT, fullPage: true });
  ran++; if (result !== "pass") failed++;
  await browser.close();
}
server.close();
if (ran === 0) { console.log("no browsers ran"); process.exit(1); }
process.exit(failed ? 1 : 0);
