/**
 * Depth (top-level design 2.4): one dial, three stops, for the whole app.
 * Curious, Science and Engine are the same three depths "why?" uses. Depth
 * never hides an instrument, a scale or a desk; it only changes how much each
 * one says. Presentation only: it never changes data.
 */
import type { SceneFact, SceneSubject } from "./scene.ts";
import { constant } from "./sheet.ts";
import { sayNumber } from "./figures.ts";

export type Depth = "curious" | "science" | "engine";

export const DEPTHS: { value: Depth; label: string; hint: string }[] = [
  { value: "curious", label: "Curious", hint: "A sentence and the headline facts, in words: about 15 °C, 365 days" },
  { value: "science", label: "Science", hint: "Every value, set or derived, with its evidence and honest precision" },
  { value: "engine", label: "Engine", hint: "Addresses, who published each value, the kernel's hashes" },
];

export function isDepth(x: unknown): x is Depth {
  return x === "curious" || x === "science" || x === "engine";
}

/** How many facts a card leads with at Curious. */
export const CURIOUS_FACTS = 4;

const HEADLINE = [/^spectral$/, /^a_au$/, /surface|temperature|teff/, /^day|rotation/, /year|period/, /^mass/, /lumin/, /^radius/, /gravity/];

/**
 * The facts a card leads with at Curious: a star's class, where it goes round, how warm, how
 * long its day and year are, how heavy, in that order where the card has them,
 * then the card's own order. Presentation only: the rest stay one tap away.
 */
export function headline<F extends Pick<SceneFact, "field">>(facts: readonly F[], n = CURIOUS_FACTS): F[] {
  const out: F[] = [];
  for (const re of HEADLINE) {
    const f = facts.find((x) => re.test(x.field) && !out.includes(x));
    if (f) out.push(f);
  }
  for (const f of facts) if (!out.includes(f)) out.push(f);
  return out.slice(0, n);
}

/** A number to two significant figures, with thousands grouped ("6,000", "0.52", "12"). */
export function roughly(x: number): string {
  if (!Number.isFinite(x)) return String(x);
  if (x === 0) return "0";
  const mag = Math.floor(Math.log10(Math.abs(x)));
  const step = Math.pow(10, mag - 1);
  const r = Math.round(x / step) * step;
  const digits = Math.max(0, 1 - mag);
  return sayNumber(r, 0, { upTo: digits });
}

const NUM = /^(-?\d[\d,]*(?:\.\d+)?(?:e[+-]?\d+)?)\s*(.*)$/i;

/**
 * A fact as Curious says it: words before numbers, rounded honestly ("about").
 * Temperatures a person could stand in read in °C; hours and days read as
 * hours, days or years. Anything that is not a plain number keeps its text.
 */
export function curiousDisplay(f: Pick<SceneFact, "display" | "unit">): string {
  const m = NUM.exec(f.display.trim());
  if (!m) return f.display;
  const n = Number(m[1]!.replace(/,/g, ""));
  if (!Number.isFinite(n)) return f.display;
  const unit = (f.unit ?? m[2] ?? "").trim();
  if (unit === "K") {
    const c = n - constant("T0C");
    if (c >= -150 && c <= 250) return `about ${Math.round(c)} °C`;
    return `about ${roughly(n)} K`;
  }
  if (unit === "h") return n < 48 ? `about ${roughly(n)} hours` : days(n / 24);
  if (unit === "d") return days(n);
  if (unit === "yr" || unit === "y") return `about ${roughly(n)} years`;
  return `about ${roughly(n)}${unit ? ` ${unit}` : ""}`;
}

function days(d: number): string {
  if (d < 1.5) return "about a day";
  if (d < 730) return `about ${roughly(d)} days`;
  return `about ${roughly(d / 365.25)} years`;
}

const ORDINAL = /^[IVXLC]+$/;

/**
 * The card's one sentence at Curious: the body's own prose when it has some,
 * otherwise a sentence made from what it is and where it goes round.
 */
export function curiousSentence(s: Pick<SceneSubject, "kind" | "subtitle" | "prose" | "orbit" | "parent">, parentLabel?: string): string {
  if (s.prose) return s.prose;
  const what = (s.subtitle ?? "")
    .split("·")
    .map((x) => x.trim())
    .filter((x) => x && !ORDINAL.test(x))
    .pop();
  const noun = what || s.kind;
  const a = /^[aeiou]/i.test(noun) ? "An" : "A";
  const first = `${a} ${noun}.`;
  if (!s.parent || !s.orbit) return first;
  const round = parentLabel ? `It goes round ${parentLabel}` : "It goes round its star";
  return s.kind === "moon" ? `${first} ${round}.` : `${first} ${round} at about ${roughly(s.orbit.a_au)} AU.`;
}
