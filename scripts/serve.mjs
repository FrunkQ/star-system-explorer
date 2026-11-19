// Minimal static server: node scripts/serve.mjs <dir> [port]
import { createServer } from "node:http";
import { readFile } from "node:fs/promises";
import { join, extname, normalize, resolve } from "node:path";
const root = resolve(process.argv[2] ?? ".");
const port = Number(process.argv[3] ?? 8080);
const types = { ".html": "text/html", ".js": "text/javascript", ".wasm": "application/wasm", ".json": "application/json", ".jsonl": "text/plain", ".css": "text/css", ".svg": "image/svg+xml", ".png": "image/png", ".woff2": "font/woff2", ".woff": "font/woff" };
createServer(async (req, res) => {
  const p = normalize(join(root, decodeURIComponent(new URL(req.url, "http://x").pathname)));
  const file = p.endsWith("/") ? join(p, "index.html") : p;
  if (!file.startsWith(root)) { res.writeHead(403).end(); return; }
  let body;
  try { body = await readFile(file); } catch { res.writeHead(404).end(); return; }
  res.writeHead(200, { "content-type": types[extname(file)] ?? "application/octet-stream" }).end(body);
}).listen(port, () => console.log(`serving ${root} at http://localhost:${port}/`));
