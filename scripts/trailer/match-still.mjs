#!/usr/bin/env node
// "The real photo, our version": one 1920x1080 side-by-side still for a matched reference.
//   node scripts/trailer/match-still.mjs --match DIR --ours ours.png [--out still.png]
// DIR is a match folder (ref.jpg, credit.txt, pose.json; see the renderer's MATCH1 handoff).
// The photograph goes on the left with its credit line, and SSE4's frame on the right,
// untouched apart from scaling: never retouch our side to look closer to the photograph.
import { execFileSync } from "node:child_process";
import { mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve, basename } from "node:path";

const arg = (k, d) => {
  const i = process.argv.indexOf(k);
  return i > 0 ? process.argv[i + 1] : d;
};
const match = resolve(arg("--match", ""));
const ours = resolve(arg("--ours", ""));
const out = resolve(arg("--out", `${basename(match)}-side-by-side.png`));
if (!arg("--match") || !arg("--ours")) {
  console.error("usage: match-still.mjs --match DIR --ours ours.png [--out still.png]");
  process.exit(2);
}

const credit = readFileSync(join(match, "credit.txt"), "utf8");
const field = (k) => credit.match(new RegExp(`^${k}:\\s*(.+)$`, "m"))?.[1]?.trim() ?? "";
const pose = JSON.parse(readFileSync(join(match, "pose.json"), "utf8"));

// Text goes through files so no caption needs escaping for the filter graph.
const tmp = mkdtempSync(join(tmpdir(), "match-"));
const txt = (name, s) => {
  const p = join(tmp, name);
  writeFileSync(p, s);
  return p.replace(/\\/g, "/").replace(/:/g, "\\:");
};
const fontArg = process.platform === "win32" ? "font=Arial" : "fontfile=/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf";
const boldArg = process.platform === "win32" ? "font=Arial Bold" : "fontfile=/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf";
const t = {
  l1: txt("l1", "Photograph"),
  l2: txt("l2", `${field("Title")}\n${field("Credit")}`),
  r1: txt("r1", "SSE4, computed from physics"),
  r2: txt("r2", `${pose.view ?? ""}${pose.date ? ` · ${String(pose.date).slice(0, 10)}` : ""}`),
};
const half = (inp, lab) =>
  `[${inp}]scale=940:900:force_original_aspect_ratio=decrease,pad=960:1080:(ow-iw)/2:60+(900-ih)/2:color=0x0b0f16[${lab}]`;
const draw = (file, x, y, size, bold, colour = "white") =>
  `drawtext=${bold ? boldArg : fontArg}:textfile='${file}':x=${x}:y=${y}:fontsize=${size}:fontcolor=${colour}:line_spacing=6`;
const graph = [
  half("0:v", "a"),
  half("1:v", "b"),
  `[a][b]hstack=inputs=2,drawbox=x=959:y=0:w=2:h=1080:color=0x2a3444:t=fill,` +
    [
      draw(t.l1, 40, 18, 30, true),
      draw(t.r1, 1000, 18, 30, true, "0xE86F2B"),
      draw(t.l2, 40, 980, 20, false, "0xc8d0dc"),
      draw(t.r2, 1000, 980, 20, false, "0xc8d0dc"),
    ].join(",") +
    "[v]",
].join(";");
execFileSync("ffmpeg", ["-y", "-loglevel", "error", "-i", join(match, "ref.jpg"), "-i", ours, "-filter_complex", graph, "-map", "[v]", "-frames:v", "1", out], {
  stdio: "inherit",
});
console.log(`ok   ${out}`);
