// The placeholder logo (track AT, AT6; atlas design 7.3): until a brand has a logo, its letterhead draws one from the
// name in our own 5 by 5 block font, each letter made of itself. Endearingly bad on purpose, and the same every time
// for the same name: the three flaws are chosen by a seed from the name, never by a clock or Math.random.
import font from "./blockfont.json" with { type: "json" };

const GLYPHS = (font as { glyphs: Record<string, string[]> }).glyphs;

/** A small, stable hash of a string (FNV-1a, 32 bits). */
export function seedOf(s: string): number {
  let h = 0x811c9dc5;
  for (let i = 0; i < s.length; i++) {
    h ^= s.charCodeAt(i);
    h = Math.imul(h, 0x01000193) >>> 0;
  }
  return h >>> 0;
}

/** The letters a name is drawn with: the initials of up to four words, or a one-word name spelt out to eight letters. */
export function logoLetters(name: string): string {
  const words = name
    .normalize("NFD")
    .replace(/[̀-ͯ]/g, "")
    .toUpperCase()
    .split(/[^A-Z0-9]+/)
    .filter(Boolean);
  if (!words.length) return "?";
  if (words.length === 1) return words[0]!.slice(0, 8);
  return words
    .slice(0, 4)
    .map((w) => w[0])
    .join("");
}

export const FLOURISHES: readonly string[] = ["=>", "(o)", "*", "~~"];
export const APOLOGIES: readonly string[] = ["(logo pending. do not print.)", "(placeholder. ask marketing.)", "(drawn by the intern.)"];

export interface AsciiLogo {
  /** The art, one string per line, monospaced. */
  lines: string[];
  /** What a screen reader says for it. */
  label: string;
}

/**
 * The placeholder logo for a brand name. `est` is the year line ("1098"), from the campaign's calendar at its start;
 * absent, no "est". Pure: the same name and est give the same lines.
 */
export function asciiLogo(name: string, est?: string): AsciiLogo {
  const letters = logoLetters(name);
  const seed = seedOf(name);
  const rows = [0, 1, 2, 3, 4].map((r) =>
    [...letters]
      .map((ch) => (GLYPHS[ch] ?? GLYPHS["O"]!)[r]!.replace(/#/g, ch === "?" ? "#" : ch).replace(/\./g, " "))
      .join("  ")
      .replace(/\s+$/, ""),
  );
  const flourish = FLOURISHES[seed % FLOURISHES.length]!;
  const apology = APOLOGIES[(seed >>> 4) % APOLOGIES.length]!;
  // The flourish sits on one of the lower rows, nudged a step out of line.
  const fRow = 2 + ((seed >>> 8) % 3);
  const fNudge = (seed >>> 12) % 2;
  const textW = Math.max(...rows.map((r) => r.length));
  const inner = textW + 4 + flourish.length + 3;
  const body = rows.map((r, i) => {
    let line = "  " + r.padEnd(textW);
    if (i === fRow) line += " ".repeat(2 + fNudge) + flourish;
    return `|${line.padEnd(inner)}|`;
  });
  // The bottom edge is one dash short or one long.
  const edge = inner + ((seed >>> 16) % 2 ? 1 : -1);
  const caption = name.toUpperCase() + (est ? ` *est ${est}*` : "");
  return {
    lines: [`+${"-".repeat(inner)}+`, ...body, `+${"-".repeat(edge)}+`, ` ${caption}`, `   ${apology}`],
    label: `${name}, placeholder logo drawn in text`,
  };
}
