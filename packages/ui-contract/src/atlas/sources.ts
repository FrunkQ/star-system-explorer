// Every number keeps its Why (track AT, AT8; atlas design section 8). An edition with `sources: endnotes` marks each
// figure on a page with a small number and ends the page with "How we know": for each mark, how the audience knows it,
// then one line of reasoning, with its citation where the reader's depth shows it. `sources: section` prints one
// "Source:" line under each facts section; `off` adds nothing. Pure: the view hands in what it has read (the audience's
// rung from `known_to`, the kernel's explanation), and this lays it on the page. It never reads the truth: a note is
// only made for a figure already on the audience's own page, so a seat's note names only its own knowledge.
import type { Depth } from "../depth.ts";
import { editionWord, type Edition } from "./edition.ts";
import type { AtlasPage, Block } from "./page.ts";

/** How one figure is known, as the view read it. */
export interface HowKnown {
  /** How the audience knows it: "Orbital survey, given to the table", "Worked out from the physics". */
  source: string;
  /** One line of reasoning (the explanation's lens at the reader's depth). */
  reason?: string;
  /** Where the reasoning comes from (the explanation's first reference). Printed at Science and Engine. */
  cite?: string;
}

/** One numbered note of "How we know". */
export interface SourceNote {
  mark: number;
  /** The line it is for: "Mass", on a page that says whose. */
  label: string;
  source: string;
  reason?: string;
  cite?: string;
}

export type HowKnownOf = (address: string, field: string) => HowKnown | undefined;

export interface SourcesOptions {
  depth?: Depth;
}

/** The first sentence of a lens: one line of reasoning, not a paragraph. */
export function firstSentence(text: string): string {
  const t = text.trim();
  const m = /^(.+?[.!?])(\s|$)/.exec(t);
  return m ? m[1]! : t;
}

/**
 * Lays the edition's sources on a page. Endnotes: each known or banded figure with a note gets `mark` (from 1, in
 * reading order) and a "How we know" block ends the page body. Section: each facts section gets a `source` line, its
 * figures' sources in order, each said once. Off, or nothing to say: the page is returned as it was.
 */
export function withSources(page: AtlasPage, edition: Edition, howOf: HowKnownOf, opts: SourcesOptions = {}): AtlasPage {
  if (edition.sources === "off") return page;
  const cites = (opts.depth ?? "science") !== "curious";
  if (edition.sources === "section") {
    let any = false;
    const blocks = page.blocks.map((b): Block => {
      if (b.type !== "facts") return b;
      const said: string[] = [];
      for (const r of b.rows) {
        if (r.state === "gap" || !r.figure) continue;
        const h = howOf(page.address, r.field);
        if (h && !said.includes(h.source)) said.push(h.source);
      }
      if (!said.length) return b;
      any = true;
      return { ...b, source: `${editionWord(edition, "sources.section", "Source")}: ${said.join("; ")}` };
    });
    return any ? { ...page, blocks } : page;
  }
  const notes: SourceNote[] = [];
  const blocks = page.blocks.map((b): Block => {
    if (b.type !== "facts") return b;
    let touched = false;
    const rows = b.rows.map((r) => {
      if (r.state === "gap" || !r.figure) return r;
      const h = howOf(page.address, r.field);
      if (!h) return r;
      touched = true;
      const mark = notes.length + 1;
      notes.push({ mark, label: r.label, source: h.source, ...(h.reason ? { reason: h.reason } : {}), ...(cites && h.cite ? { cite: h.cite } : {}) });
      return { ...r, mark };
    });
    return touched ? { ...b, rows } : b;
  });
  if (!notes.length) return page;
  // The notes end the page body: before the margin, which stays in the side column.
  const at = blocks.findIndex((b) => b.type === "margin");
  const block: Block = { type: "sources", title: editionWord(edition, "sources.title", "How we know"), notes };
  if (at < 0) blocks.push(block);
  else blocks.splice(at, 0, block);
  return { ...page, blocks };
}
