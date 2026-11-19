// Time ruler arithmetic. Pure functions so they can be tested without a DOM.

export interface RulerEvent {
  id: string;
  at_yr: number;
  label: string;
  tone: "past" | "future" | "authored";
}

/** "4.61 Gyr", "820 Myr", "12.5 kyr", "300 yr". Three significant figures. */
export function formatYears(yr: number): string {
  const a = Math.abs(yr);
  const [div, unit] = a >= 1e9 ? [1e9, "Gyr"] : a >= 1e6 ? [1e6, "Myr"] : a >= 1e3 ? [1e3, "kyr"] : [1, "yr"];
  const v = yr / div;
  const digits = Math.abs(v) >= 100 ? 0 : Math.abs(v) >= 10 ? 1 : 2;
  return `${trim(v.toFixed(digits))} ${unit}`;
}

function trim(s: string): string {
  return s.includes(".") ? s.replace(/\.?0+$/, "") : s;
}

/** A "nice" step (1, 2 or 5 times a power of ten) giving about `count` ticks. */
export function niceStep(span: number, count: number): number {
  const raw = span / Math.max(count, 1);
  const p = 10 ** Math.floor(Math.log10(raw));
  const m = raw / p;
  return (m <= 1 ? 1 : m <= 2 ? 2 : m <= 5 ? 5 : 10) * p;
}

export interface Tick {
  at: number;
  major: boolean;
  label?: string;
}

/** Ticks over [from, to]; every other nice step is major and labelled. */
export function ticks(from: number, to: number, count = 10): Tick[] {
  const step = niceStep(to - from, count);
  const out: Tick[] = [];
  const first = Math.ceil(from / step);
  for (let i = first; i * step <= to + step * 1e-9; i++) {
    const at = i * step;
    const major = i % 2 === 0;
    out.push(major ? { at, major, label: formatYears(at) } : { at, major });
  }
  return out;
}

export function fraction(at: number, from: number, to: number): number {
  return to === from ? 0 : Math.min(1, Math.max(0, (at - from) / (to - from)));
}

/** Keyboard steps for the ruler slider: arrows 1 %, Page keys 10 %, rounded to whole years. */
export function keyStep(key: string, now: number, from: number, to: number): number | null {
  const span = to - from;
  const by = (f: number) => Math.round(Math.min(to, Math.max(from, now + span * f)));
  switch (key) {
    case "ArrowRight":
    case "ArrowUp":
      return by(0.01);
    case "ArrowLeft":
    case "ArrowDown":
      return by(-0.01);
    case "PageUp":
      return by(0.1);
    case "PageDown":
      return by(-0.1);
    case "Home":
      return from;
    case "End":
      return to;
    default:
      return null;
  }
}
