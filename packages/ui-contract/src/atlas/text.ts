// Other outputs of the same page model (track AT, AT4; atlas design section 9): an Atlas page as plain text or
// Markdown ("Copy as text", for a chat, a forum or a virtual tabletop) and pages as one self-contained HTML file
// ("Save page"). Pure: the words and figures are the blocks' own, so a copy says exactly what the page says.
import type { AtlasPage, Block } from "./page.ts";

/** A fixed-length bar in text: the same length whatever is behind it. */
export const TEXT_BAR = "█████████";

export interface TextOptions {
  /** Markdown (headings, a table, emphasis); plain text otherwise. */
  markdown?: boolean;
  /** One line at the end: edition, book, audience, date. */
  footer?: string;
}

const gapText = (text: string, bar: boolean | undefined, md: boolean) => (bar ? TEXT_BAR : md ? `*${text}*` : text);

function blockText(b: Block, md: boolean): string[] {
  switch (b.type) {
    case "title":
      return md ? [`# ${b.text}`, `*${b.subtitle}*`, ""] : [b.text, "=".repeat(Math.max(4, b.text.length)), b.subtitle, ""];
    case "prose":
      return [b.text, ""];
    case "facts": {
      const head = md ? [`## ${b.title}`] : [b.title.toUpperCase()];
      const rows = b.rows.map((r) => {
        const v = r.state === "gap" ? gapText(r.text, r.bar, md) : r.text;
        return md ? `- **${r.label}:** ${v}` : `  ${r.label}: ${v}`;
      });
      return [...head, ...rows, ""];
    }
    case "table": {
      const cell = (c: { text: string; state: string; bar?: boolean }) => (c.state === "gap" ? gapText(c.text || "—", c.bar, md) : c.text);
      if (md) {
        return [`## ${b.title}`, `| ${b.columns.join(" | ")} |`, `| ${b.columns.map(() => "---").join(" | ")} |`, ...b.rows.map((r) => `| ${r.cells.map(cell).join(" | ")} |`), ""];
      }
      return [b.title.toUpperCase(), ...b.rows.map((r) => `  ${r.cells.map(cell).join(" · ")}`), ""];
    }
    case "plate":
      return [md ? `*${b.title}: ${b.names.join(", ")}*` : `[${b.title}: ${b.names.join(", ")}]`, ""];
    case "story":
      return [md ? `## ${b.title}` : b.title.toUpperCase(), ...b.lines.map((l) => (md ? `> ${l.label}: ${l.text}` : `  "${l.text}" (${l.label})`)), ""];
    case "note":
      return [md ? `*${b.text}*` : b.text, ""];
    case "rule":
      return [md ? "---" : "----", ""];
    case "letterhead": {
      const art = b.art ? (md ? ["```", ...b.art, "```"] : b.art) : [md ? `**${b.name}**` : b.name.toUpperCase()];
      return [...art, ...(b.motto ? [md ? `*${b.motto}*` : b.motto] : []), ...b.lines, ...(b.stamp ? [md ? `**${b.stamp}**` : `[ ${b.stamp} ]`] : []), ""];
    }
  }
}

/** One Atlas page as plain text or Markdown. */
export function pageText(page: AtlasPage, opts: TextOptions = {}): string {
  const md = !!opts.markdown;
  const lines = page.blocks.flatMap((b) => blockText(b, md));
  if (opts.footer) lines.push(md ? `*${opts.footer}*` : opts.footer);
  while (lines.length && lines[lines.length - 1] === "") lines.pop();
  return lines.join("\n") + "\n";
}

const esc = (s: string) => s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");

function blockHtml(b: Block): string {
  switch (b.type) {
    case "title":
      return `<header><p class="sub">${esc(b.subtitle)}</p><h1>${esc(b.text)}</h1></header>`;
    case "prose":
      return `<p class="prose">${esc(b.text)}</p>`;
    case "facts":
      return `<section><h2>${esc(b.title)}</h2><dl>${b.rows
        .map((r) => `<div><dt>${esc(r.label)}</dt><dd${r.state === "gap" ? ' class="gap"' : ""}>${r.state === "gap" && r.bar ? '<span class="bar"></span>' : esc(r.text)}</dd></div>`)
        .join("")}</dl></section>`;
    case "table":
      return `<section><h2>${esc(b.title)}</h2><table><thead><tr>${b.columns.map((c) => `<th>${esc(c)}</th>`).join("")}</tr></thead><tbody>${b.rows
        .map((r) => `<tr>${r.cells.map((c) => `<td${c.state === "gap" ? ' class="gap"' : ""}>${c.bar ? '<span class="bar short"></span>' : esc(c.text)}</td>`).join("")}</tr>`)
        .join("")}</tbody></table></section>`;
    case "plate":
      return `<p class="plate">${esc(b.title)}: ${esc(b.names.join(", "))}</p>`;
    case "story":
      return `<section><h2>${esc(b.title)}</h2>${b.lines.map((l) => `<blockquote>${esc(l.label)}: ${esc(l.text)}</blockquote>`).join("")}</section>`;
    case "note":
      return `<p class="note">${esc(b.text)}</p>`;
    case "rule":
      return "<hr>";
    case "letterhead":
      return `<div class="lh"${b.accent ? ` style="--lh-accent:${esc(b.accent)}"` : ""}>${b.logo ? (b.logoMono ? `<span class="mono" role="img" aria-label="${esc(b.name)}" style="--logo:url('${esc(b.logo)}')"></span>` : `<img src="${esc(b.logo)}" alt="${esc(b.name)}">`) : b.art ? `<pre aria-label="${esc(b.artLabel ?? b.name)}">${esc(b.art.join("\n"))}</pre>` : ""}<div><p class="lhname">${esc(b.name)}</p>${b.motto ? `<p class="lhmotto">${esc(b.motto)}</p>` : ""}${b.lines.map((l) => `<p class="lhline">${esc(l)}</p>`).join("")}</div>${b.stamp ? `<p class="stamp">${esc(b.stamp)}</p>` : ""}</div>`;
  }
}

export interface HtmlOptions {
  title: string;
  /** The look's tokens as CSS custom properties (`--sse-ink: …;`), put on :root. */
  tokens?: string;
  /** One footer line per page. */
  footer?: string;
}

/** Pages as one self-contained HTML file: no script, nothing loaded from the web, prints on its own. */
export function atlasHtml(pages: readonly AtlasPage[], opts: HtmlOptions): string {
  const css = `:root{${opts.tokens ?? ""}}
body{margin:0;background:var(--sse-ground,#fff);color:var(--sse-ink,#111);font-family:var(--sse-font-body,serif)}
article{max-width:46em;margin:2em auto;padding:2em;background:var(--sse-ground-raised,#fff);break-after:page}
.sub{text-transform:uppercase;letter-spacing:.12em;font-size:.8em;color:var(--sse-ink-muted,#555);margin:0}
h1{font-family:var(--sse-font-display,serif);font-style:italic;font-weight:500;font-size:2.4em;margin:.1em 0 .6em}
h2{text-transform:uppercase;letter-spacing:.12em;font-size:.8em;color:var(--sse-ink-muted,#555);margin:1.4em 0 .4em}
section{break-inside:avoid}dl{margin:0}dl div{display:grid;grid-template-columns:2fr 3fr;gap:.8em;padding:.3em 0;border-bottom:1px solid var(--sse-rule,#ddd);break-inside:avoid}
dt{color:var(--sse-ink-muted,#555)}dd{margin:0;font-family:var(--sse-font-mono,monospace)}.gap{font-style:italic;font-family:inherit;color:var(--sse-ink-muted,#777)}
.bar{display:inline-block;width:9em;height:.9em;background:var(--sse-ink,#111);vertical-align:middle}.bar.short{width:4em}
table{width:100%;border-collapse:collapse}th,td{text-align:left;padding:.3em .5em .3em 0;border-bottom:1px solid var(--sse-rule,#ddd)}tr{break-inside:avoid}
blockquote{margin:.4em 0;padding-left:.8em;border-left:2px solid var(--sse-accent,#a33);font-style:italic}.note,.plate,footer{font-size:.85em;color:var(--sse-ink-muted,#555)}
.lh{display:flex;gap:1.2em;align-items:flex-start;border-bottom:2px solid var(--lh-accent,var(--sse-ink,#111));padding-bottom:.8em;margin-bottom:1.2em;position:relative}.lh img{max-height:5em;max-width:10em}.lh .mono{display:block;width:10em;height:5em;background:var(--lh-accent,var(--sse-ink,#111));-webkit-mask:var(--logo) left center/contain no-repeat;mask:var(--logo) left center/contain no-repeat luminance}.lh pre{margin:0;font-size:.6em;line-height:1.1}
.lhname{font-weight:700;letter-spacing:.08em;text-transform:uppercase;margin:0}.lhmotto{font-style:italic;margin:0}.lhline{margin:0;font-family:var(--sse-font-mono,monospace);font-size:.8em}
.stamp{position:absolute;right:0;top:0;margin:0;padding:.2em .6em;border:2px solid var(--lh-accent,var(--sse-accent,#a33));color:var(--lh-accent,var(--sse-accent,#a33));font-weight:700;letter-spacing:.1em;transform:rotate(-4deg)}
footer{margin-top:2em;padding-top:.6em;border-top:1px solid var(--sse-line,#ccc);display:flex;justify-content:space-between;font-family:var(--sse-font-mono,monospace)}`;
  const body = pages
    .map((p) => `<article>${p.blocks.map(blockHtml).join("\n")}${opts.footer ? `<footer><span>${esc(opts.footer)}</span><span>${p.number}</span></footer>` : ""}</article>`)
    .join("\n");
  return `<!doctype html>\n<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><title>${esc(opts.title)}</title><style>${css}</style></head><body>\n${body}\n</body></html>\n`;
}
