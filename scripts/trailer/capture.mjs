// Trailer capture rig: films scripted shots of the GM shell as video clips.
// Usage:
//   node scripts/trailer/capture.mjs [--base URL] [--out DIR] [--only id,id] [--fps 30] [--scale 1] [--headed]
// With no --base it serves the local build (scripts/build-site.sh → dist/), which is what main deploys
// to v4.starsystemx.com; pass --base https://v4.starsystemx.com/ to film the live site.
// Each shot in scripts/trailer/shots/*.mjs is one clip: a fresh browser context (no saved campaign), the
// fixed campaign the site opens with (Kestrel), so the same script films the same shot every time.
// Frames come from Chromium's screencast (real paint times), then ffmpeg makes constant-rate H.264.
// Writes <out>/<id>.mp4, <id>.png (a still at the end, 2x) and <id>.json (what was filmed, from where, when).
import { spawn, execFileSync } from "node:child_process";
import { mkdirSync, writeFileSync, readdirSync, rmSync, existsSync } from "node:fs";
import { join, dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { createRequire } from "node:module";
import { director } from "./director.mjs";
import * as nav from "../../tests/ui/nav.mjs";

const here = dirname(fileURLToPath(import.meta.url));
const repo = resolve(here, "../..");
const { chromium } = createRequire(import.meta.url)("playwright");

const argv = process.argv.slice(2);
const opt = (k, dflt) => {
  const i = argv.indexOf(`--${k}`);
  return i >= 0 ? argv[i + 1] : dflt;
};
const flag = (k) => argv.includes(`--${k}`);
const out = resolve(opt("out", join(repo, "trailer-out")));
const only = opt("only", "")?.split(",").filter(Boolean);
const fps = Number(opt("fps", 30));
const scale = Number(opt("scale", 1));
let base = opt("base", null);

let server = null;
if (!base) {
  const dist = join(repo, "dist");
  if (!existsSync(join(dist, "index.html"))) throw new Error("no dist/: run npm run build first, or pass --base");
  const port = 9900 + Math.floor(Math.random() * 90);
  server = spawn(process.execPath, [join(repo, "scripts/serve.mjs"), dist, String(port)], { stdio: "ignore" });
  await new Promise((r) => setTimeout(r, 500));
  base = `http://localhost:${port}/`;
}
const commit = (() => {
  try {
    return execFileSync("git", ["-C", repo, "rev-parse", "--short", "HEAD"]).toString().trim();
  } catch {
    return null;
  }
})();

const files = readdirSync(join(here, "shots")).filter((f) => f.endsWith(".mjs")).sort();
const shots = [];
for (const f of files) {
  const s = (await import(join(here, "shots", f))).default;
  if (!only?.length || only.includes(s.id)) shots.push(s);
}
mkdirSync(out, { recursive: true });

const launch = (extra = []) =>
  chromium.launch({ headless: !flag("headed"), args: ["--enable-unsafe-webgpu", "--ignore-gpu-blocklist", "--hide-scrollbars", ...extra] });
const browser = await launch();
const report = [];
try {
  for (const shot of shots) {
    const vp = shot.viewport ?? { width: 1920, height: 1080 };
    // A shot may ask for its own browser flags (e.g. no WebGL, to film a Canvas tier where no GPU is present).
    const own = shot.browserArgs && !flag("gpu") ? await launch(shot.browserArgs) : null;
    const context = await (own ?? browser).newContext({ viewport: vp, deviceScaleFactor: scale, colorScheme: shot.colorScheme ?? "light" });
    const page = await context.newPage();
    const errors = [];
    page.on("pageerror", (e) => errors.push(e.message));
    const tmp = join(out, `.frames-${shot.id}`);
    rmSync(tmp, { recursive: true, force: true });
    mkdirSync(tmp, { recursive: true });

    // Load and settle off camera, then start filming.
    await page.goto(new URL(shot.path ?? "", base).href);
    if (shot.waitFor !== false) await page.waitForSelector(shot.waitFor ?? "[data-state-hash]:not([data-state-hash=''])", { timeout: 60000 });
    const d = director(page);
    await d.ready();
    if (shot.setup) await shot.setup(page, d, nav);
    await page.waitForTimeout(400);

    const cdp = await context.newCDPSession(page);
    const frames = [];
    cdp.on("Page.screencastFrame", async (f) => {
      const n = frames.length;
      const file = join(tmp, `f${String(n).padStart(6, "0")}.jpg`);
      frames.push({ file, t: f.metadata.timestamp });
      writeFileSync(file, Buffer.from(f.data, "base64"));
      await cdp.send("Page.screencastFrameAck", { sessionId: f.sessionId }).catch(() => {});
    });
    await cdp.send("Page.startScreencast", { format: "jpeg", quality: 92, maxWidth: vp.width * scale, maxHeight: vp.height * scale, everyNthFrame: 1 });
    const t0 = Date.now() / 1000;
    let failure = null;
    try {
      await shot.run(page, d, nav);
      await d.hold(shot.tail ?? 800);
    } catch (e) {
      failure = e.message.split("\n")[0];
    }
    const t1 = Date.now() / 1000;
    await cdp.send("Page.stopScreencast").catch(() => {});
    await page.screenshot({ path: join(out, `${shot.id}.png`) }).catch(() => {});

    let duration = 0;
    if (frames.length) {
      // Each frame lasts until the next one; the last until filming stopped.
      const list = frames.map((f, i) => {
        const next = i + 1 < frames.length ? frames[i + 1].t : Math.max(t1, f.t + 1 / fps);
        return `file '${f.file}'\nduration ${Math.max(1 / 1000, next - f.t).toFixed(4)}`;
      });
      list.push(`file '${frames.at(-1).file}'`);
      writeFileSync(join(tmp, "list.txt"), list.join("\n"));
      execFileSync("ffmpeg", ["-y", "-loglevel", "error", "-f", "concat", "-safe", "0", "-i", join(tmp, "list.txt"),
        "-vf", `fps=${fps},scale=trunc(iw/2)*2:trunc(ih/2)*2`, "-c:v", "libx264", "-preset", "slow", "-crf", "17",
        "-pix_fmt", "yuv420p", "-movflags", "+faststart", join(out, `${shot.id}.mp4`)]);
      duration = Math.max(0, Math.max(t1, frames.at(-1).t) - frames[0].t);
    }
    rmSync(tmp, { recursive: true, force: true });
    const meta = {
      id: shot.id, title: shot.title, feature: shot.feature, trailers: shot.trailers, notes: shot.notes ?? "",
      base, commit: server ? commit : null, filmed_at: new Date().toISOString(), viewport: vp, scale, fps,
      seconds: Number(duration.toFixed(2)), screencast_frames: frames.length,
      effective_fps: duration ? Number((frames.length / duration).toFixed(1)) : 0,
      wall_seconds: Number((t1 - t0).toFixed(2)), ok: !failure, failure, page_errors: errors,
    };
    writeFileSync(join(out, `${shot.id}.json`), JSON.stringify(meta, null, 2) + "\n");
    report.push(meta);
    console.log(`${failure ? "FAIL" : "ok  "} ${shot.id}  ${meta.seconds}s  ${meta.effective_fps} fps${failure ? "  " + failure : ""}`);
    await context.close();
    await own?.close();
  }
} finally {
  await browser.close();
  server?.kill();
}
if (report.some((r) => !r.ok)) process.exitCode = 1;
