// Letterhead (track AT, AT6; atlas design 7 and 12.4): whose paper a page is on. The Company edition has no name of
// its own; it takes the brand of the role reading it. The order for the mark is: the logo the GM supplied, else a
// built-in one they picked (which is the same, a data URI), else the placeholder logo drawn in text. Every line is the
// edition's own template with slots filled; nothing here is a fact about a subject.
import type { SeatBrand } from "../brand.ts";
import { asciiLogo, logoLetters } from "./ascii.ts";
import { editionWord, type Edition } from "./edition.ts";
import type { AtlasPage } from "./page.ts";
import builtin from "./logos.json" with { type: "json" };

/** SSE3's starter logos, built in (atlas design 14A note 2), each with its credit. */
export interface BuiltinLogo {
  id: string;
  name: string;
  logo: string;
  credit: string;
  /** A light mark for a dark ground: drawn as a luminance mask in ink. */
  mono?: boolean;
}
export const BUILTIN_LOGOS: readonly BuiltinLogo[] = (builtin as { logos: BuiltinLogo[] }).logos;

/** A brand as the letterhead reads it: the seat's brand with the two optional fields of design 7.1. */
export type LetterheadBrand = SeatBrand & { accent?: string; motto?: string; /** The logo is a light mark drawn in ink (the built-in ones). */ mono?: boolean };

const ACCENT = /^#[0-9a-fA-F]{6}$/;

export interface LetterheadContext {
  /** The brand of the role reading; absent, the edition's fallback name and the drawn logo. */
  brand?: LetterheadBrand;
  /** The clearance words: the role's name, or "GM EYES ONLY" on the GM's copy. */
  clearance: string;
  /** The issue date, in the campaign's calendar. */
  date: string;
  /** The book's place (the system's name). */
  place: string;
  /** The year the brand's paper says it was established ("est"), from the campaign's calendar at its start. */
  est?: string;
  /** Sheets in this print, for {pages}. */
  pages?: number;
}

/** A page's document number: brand initials, the place, the page, the subject (`HV-KES-03/TARN`). */
export function docNumber(brand: string, place: string, page: number, title: string): string {
  const ini = logoLetters(brand).slice(0, 3);
  const at = logoLetters(place.split(/\s+/)[0] ?? place).slice(0, 3);
  const subject = title.toUpperCase().replace(/[^A-Z0-9]+/g, "");
  return `${ini}-${at}-${String(page).padStart(2, "0")}/${subject || "PAGE"}`;
}

/** A line with its {slots} filled; an unknown slot is left as it is (the loader refuses those). */
export function fillSlots(text: string, slots: Readonly<Record<string, string>>): string {
  return text.replace(/\{([^}]+)\}/g, (m, k: string) => slots[k] ?? m);
}

/** The letterhead block a page carries, or null when the edition has none. */
export function letterheadBlock(ed: Edition, page: AtlasPage, ctx: LetterheadContext) {
  if (ed.letterhead.use === "none") return null;
  const name = ctx.brand?.name?.trim() || editionWord(ed, "brand.fallback", "Independent Survey Office");
  const logo = ctx.brand?.logo;
  const slots: Record<string, string> = {
    "brand.name": name,
    "brand.motto": ctx.brand?.motto ?? "",
    "brand.logo": "",
    audience: ctx.clearance,
    date: ctx.date,
    place: ctx.place,
    chapter: page.title,
    "doc.number": docNumber(name, ctx.place, page.number, page.title),
    page: String(page.number),
    pages: String(ctx.pages ?? 1),
  };
  const art = logo ? null : asciiLogo(name, ctx.est);
  return {
    type: "letterhead" as const,
    name,
    ...(ctx.brand?.motto ? { motto: ctx.brand.motto } : {}),
    ...(logo ? { logo, ...(ctx.brand?.mono ? { logoMono: true } : {}) } : {}),
    ...(art ? { art: art.lines, artLabel: art.label } : {}),
    lines: ed.letterhead.lines.map((l) => fillSlots(l, slots)).filter((l) => l.trim()),
    ...(ed.letterhead.stamp ? { stamp: fillSlots(ed.letterhead.stamp, slots) } : {}),
    ...(ctx.brand?.accent && ACCENT.test(ctx.brand.accent) ? { accent: ctx.brand.accent } : {}),
  };
}

/** The page with its letterhead on top (unchanged when the edition has none). */
export function withLetterhead(page: AtlasPage, ed: Edition, ctx: LetterheadContext): AtlasPage {
  const b = letterheadBlock(ed, page, ctx);
  return b ? { ...page, blocks: [b, ...page.blocks] } : page;
}
