#!/usr/bin/env node
// Build the readable pages of the registers kept one file per entry (scripts/registers.py): the fudge
// register and the rulings during the build, each as one HTML page, from main's files at deploy time.
// scripts/build-site.sh runs it into dist/, so the live site serves /fudges.html and /decisions.html
// and no pull request ever rewrites a combined file.
//
//   node scripts/build-registers.mjs OUT_DIR          write OUT_DIR/fudges.html and OUT_DIR/decisions.html
//   node scripts/build-registers.mjs --markdown NAME  print the register as Markdown (the test compares it
//                                                     with python3 scripts/registers.py --print NAME)
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const REGISTERS = {
  fudges: { dir: "docs/fudges", title: "Fudge register" },
  decisions: { dir: "handover/decisions", title: "Rulings during the build" },
};

// Name order as Python's sorted() gives it (code point order), never the locale's.
const byName = (a, b) => (a < b ? -1 : a > b ? 1 : 0);

export function markdown(name) {
  const top = path.join(ROOT, REGISTERS[name].dir);
  let out = "";
  for (const entry of fs.readdirSync(top).sort(byName)) {
    const full = path.join(top, entry);
    if (fs.statSync(full).isDirectory()) {
      const files = fs.readdirSync(full).filter((f) => !f.startsWith(".")).sort(byName);
      let head = null;
      const rows = [];
      for (const f of files) {
        const lines = fs.readFileSync(path.join(full, f), "utf8").split("\n");
        if (head === null) head = `${lines[0]}\n${lines[1]}`;
        rows.push(lines[2]);
      }
      out += [head, ...rows].join("\n") + "\n";
    } else if (entry.endsWith(".md")) {
      out += fs.readFileSync(full, "utf8");
    }
  }
  return out;
}

const esc = (s) => s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");

function inline(s) {
  // Code spans first, so nothing inside them is read as emphasis or a link.
  return s
    .split(/(`[^`]*`)/)
    .map((part) => {
      if (part.startsWith("`") && part.endsWith("`") && part.length > 1) return `<code>${esc(part.slice(1, -1))}</code>`;
      return esc(part)
        .replace(/\*\*([^*]+)\*\*/g, "<b>$1</b>")
        .replace(/\[([^\]]+)\]\((https?:\/\/[^)\s]+)\)/g, '<a href="$2">$1</a>')
        .replace(/\[([^\]]+)\]\(([^)\s]+)\)/g, "$1");
    })
    .join("");
}

const cells = (line) =>
  line
    .trim()
    .replace(/^\|/, "")
    .replace(/(?<!\\)\|$/, "")
    .split(/(?<!\\)\|/)
    .map((c) => c.trim().replace(/\\\|/g, "|"));

const isSep = (line) => /^\|(\s*:?-{3,}:?\s*\|)+\s*$/.test(line);

export function html(name) {
  const lines = markdown(name).split("\n");
  const body = [];
  let para = [];
  const flush = () => {
    if (para.length) body.push(`<p>${inline(para.join(" "))}</p>`);
    para = [];
  };
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    const h = /^(#{1,4}) (.*)$/.exec(line);
    if (h) {
      flush();
      const id = h[2].toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "");
      body.push(`<h${h[1].length} id="${id}">${inline(h[2])}</h${h[1].length}>`);
    } else if (line.startsWith("|")) {
      flush();
      const head = i + 1 < lines.length && isSep(lines[i + 1]) ? cells(line) : null;
      if (head) i += 1;
      const rows = [];
      while (i + 1 < lines.length && lines[i + 1].startsWith("|")) rows.push(cells(lines[++i]));
      if (!head) rows.unshift(cells(line));
      body.push(
        "<table>" +
          (head ? `<thead><tr>${head.map((c) => `<th>${inline(c)}</th>`).join("")}</tr></thead>` : "") +
          `<tbody>${rows
            .map((r) => {
              const id = name === "fudges" && /^F-[A-Za-z0-9.-]+$/.test(r[0]) ? ` id="${r[0]}"` : "";
              return `<tr${id}>${r.map((c) => `<td>${inline(c)}</td>`).join("")}</tr>`;
            })
            .join("")}</tbody></table>`,
      );
    } else if (/^\s*[-*] /.test(line)) {
      flush();
      body.push(`<ul><li>${inline(line.replace(/^\s*[-*] /, ""))}</li></ul>`);
    } else if (line.trim() === "") {
      flush();
    } else {
      para.push(line.trim());
    }
  }
  flush();
  const t = REGISTERS[name].title;
  return `<!doctype html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1">
<title>${t}</title>
<style>
:root{color-scheme:light dark;--bg:#fbfaf7;--fg:#1d1d1b;--rule:#d8d4ca;--code:#efece4}
@media (prefers-color-scheme:dark){:root{--bg:#141413;--fg:#e8e6df;--rule:#3a3934;--code:#24231f}}
body{background:var(--bg);color:var(--fg);font:15px/1.5 system-ui,sans-serif;margin:0 auto;max-width:1400px;padding:16px}
table{border-collapse:collapse;width:100%;margin:8px 0 20px;font-size:13px;display:block;overflow-x:auto}
th,td{border:1px solid var(--rule);padding:4px 6px;vertical-align:top;text-align:left}
code{background:var(--code);padding:0 3px;border-radius:3px;font-size:12px}
tr:target{outline:2px solid #c8102e}
td:first-child{white-space:nowrap}
</style></head><body>
<p><small>Built from <code>${REGISTERS[name].dir}/</code> on main when the site was deployed.</small></p>
${body.join("\n")}
</body></html>
`;
}

const args = process.argv.slice(2);
if (args[0] === "--markdown") {
  process.stdout.write(markdown(args[1]));
} else if (args[0]) {
  fs.mkdirSync(args[0], { recursive: true });
  for (const name of Object.keys(REGISTERS)) fs.writeFileSync(path.join(args[0], `${name}.html`), html(name));
  console.log(`registers: ${Object.keys(REGISTERS).map((n) => `${n}.html`).join(", ")} in ${args[0]}`);
} else {
  console.error("usage: node scripts/build-registers.mjs OUT_DIR | --markdown NAME");
  process.exit(2);
}
