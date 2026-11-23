// Cut a short preview from clean takes (filmed with --no-captions): each clip trimmed and sped up, captions from its
// .srt drawn on top as a dark strip (so they sit above any dialog), title and end cards, one MP4 under Discord's 10 MB.
// Usage: node scripts/trailer/cut-preview.mjs --clips DIR [--out preview.mp4] [--mb 9.3]
// Needs ffmpeg with libass on PATH. Works in DIR so no path needs escaping inside a filter (Windows drive colons).
import { execFileSync } from "node:child_process";
import { readFileSync, writeFileSync, existsSync, statSync } from "node:fs";
import { join, resolve } from "node:path";

const argv = process.argv.slice(2);
const opt = (k, d) => (argv.includes(`--${k}`) ? argv[argv.indexOf(`--${k}`) + 1] : d);
const dir = resolve(opt("clips", "."));
const out = resolve(opt("out", join(dir, "sse4-preview.mp4")));
const mb = Number(opt("mb", "9.3"));

// [clip id, from s, to s, speed]
const CUT = [
  ["slate", "Star System Explorer 4", "New this week · work in progress", 2.8],
  ["00-start-sheet", 0, 10, 1],
  ["25-true-system", 0, 99, 1.35],
  ["22-formation-replay", 14, 56, 1.8],
  ["23-reference-elements", 0, 24.5, 1.6],
  ["06-system-3d", 0, 99, 1.3],
  ["26-galaxy-true", 0, 99, 1.3],
  ["12-cosmic-web", 12, 99, 1],
  ["slate", "Star System Explorer 4", "Physics you can see, cite and edit  ·  v4.starsystemx.com", 3.5],
];

const ff = (...a) => execFileSync("ffmpeg", ["-v", "error", "-y", ...a], { cwd: dir, stdio: "inherit" });
const dur = (f) => Number(execFileSync("ffprobe", ["-v", "error", "-show_entries", "format=duration", "-of", "csv=p=0", f], { cwd: dir }).toString());
const ts = (x) => {
  x = Math.max(0, x);
  const h = Math.floor(x / 3600), m = Math.floor((x % 3600) / 60), s = (x % 60).toFixed(2).padStart(5, "0");
  return `${h}:${String(m).padStart(2, "0")}:${s}`;
};
const HEAD = `[Script Info]
ScriptType: v4.00+
PlayResX: 1280
PlayResY: 720
WrapStyle: 0

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Cap,DejaVu Sans,27,&H00F2F6F6,&H00FFFFFF,&H1A362A1F,&H1A362A1F,0,0,0,0,100,100,0,0,3,14,0,1,48,48,40,1
Style: Slate,DejaVu Serif,56,&H00362A1F,&H00FFFFFF,&H00000000,&H00000000,1,0,0,0,100,100,0,0,1,0,0,5,40,40,40,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
`;
const cap = (t, l) => `{\\fad(200,200)}{\\b1\\fs33\\c&H2B6FE8&}${t}{\\b0\\fs27\\c&HF2F6F6&}${l ? "\\N" + l : ""}`;
const srt = (f) =>
  readFileSync(f, "utf8").replace(/\r/g, "").trim().split(/\n\n+/).map((b) => b.split("\n")).filter((L) => L.length >= 3).map((L) => {
    const [a, b] = L[1].split("-->").map((s) => {
      const [h, m, r] = s.trim().split(":");
      const [sec, ms] = r.split(",");
      return +h * 3600 + +m * 60 + +sec + +ms / 1000;
    });
    return { a, b, title: L[2], line: L.slice(3).join(" ") };
  });

const parts = [];
CUT.forEach((c, i) => {
  const seg = `_seg${i}.mp4`, ass = `_seg${i}.ass`;
  if (c[0] === "slate") {
    const [, title, line, len] = c;
    writeFileSync(join(dir, ass), HEAD + `Dialogue: 0,0:00:00.00,${ts(len)},Slate,,0,0,0,,{\\fad(400,400)}${title}\\N{\\b0\\fnDejaVu Sans\\fs26\\c&H665F5A&}${line}\n`);
    ff("-f", "lavfi", "-i", `color=c=0xf6f2e8:s=1280x720:d=${len}:r=30`, "-vf", `ass=${ass},format=yuv420p`, "-c:v", "libx264", "-crf", "16", "-preset", "fast", seg);
  } else {
    const [id, from, to0, speed] = c;
    if (!existsSync(join(dir, `${id}.mp4`))) return console.log(`skip ${id}: no take`);
    const to = Math.min(to0, dur(`${id}.mp4`));
    const ev = existsSync(join(dir, `${id}.srt`)) ? srt(join(dir, `${id}.srt`)) : [];
    const lines = ev.map((e) => ({ a: Math.max(e.a, from), b: Math.min(e.b, to), title: e.title, line: e.line })).filter((e) => e.b - e.a > 0.3)
      .map((e) => `Dialogue: 0,${ts((e.a - from) / speed)},${ts((e.b - from) / speed)},Cap,,0,0,0,,${cap(e.title, e.line)}\n`);
    writeFileSync(join(dir, ass), HEAD + lines.join(""));
    ff("-ss", String(from), "-to", String(to), "-i", `${id}.mp4`, "-an", "-vf", `setpts=PTS/${speed},fps=30,scale=1280:720:flags=lanczos,ass=${ass},format=yuv420p`, "-c:v", "libx264", "-crf", "16", "-preset", "fast", seg);
  }
  parts.push(seg);
});
writeFileSync(join(dir, "_list.txt"), parts.map((p) => `file '${p}'`).join("\n"));
ff("-f", "concat", "-safe", "0", "-i", "_list.txt", "-c", "copy", "_joined.mp4");
const total = dur("_joined.mp4");
const kbps = Math.floor((mb * 8 * 1000) / total) - 20;
const nul = process.platform === "win32" ? "NUL" : "/dev/null";
ff("-i", "_joined.mp4", "-c:v", "libx264", "-b:v", `${kbps}k`, "-preset", "slow", "-pass", "1", "-an", "-f", "mp4", nul);
ff("-i", "_joined.mp4", "-c:v", "libx264", "-b:v", `${kbps}k`, "-preset", "slow", "-pass", "2", "-an", "-movflags", "+faststart", out);
console.log(`${out}: ${total.toFixed(1)} s, ${(statSync(out).size / 1e6).toFixed(1)} MB at ${kbps} kbps`);
