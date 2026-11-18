// The theme token schema (H8.4, H8.9): every token a view may draw with,
// described for Orac (type, meaning, default from the base theme) with a few
// promoted for people. Views draw only with these names, as CSS custom
// properties `--sse-<name>`.
export interface TokenSpec {
  name: string;
  type: "colour" | "length" | "font" | "number" | "shadow";
  meaning: string;
  promoted?: boolean;
  /** For colours that carry text: the token they sit on, checked for contrast. */
  text_on?: string[];
  /** For colours that must be seen but are not text (lines, focus): the ground they sit on. */
  line_on?: string[];
}

export const TOKENS: readonly TokenSpec[] = [
  { name: "ground", type: "colour", meaning: "Paper of the chart: the page background.", promoted: true },
  { name: "ground-raised", type: "colour", meaning: "Paper of cards, bars and the time ruler, a shade off the ground." },
  { name: "ink", type: "colour", meaning: "Main ink: lines, text, filled marks.", promoted: true, text_on: ["ground", "ground-raised"] },
  { name: "ink-muted", type: "colour", meaning: "Secondary ink: kickers, captions, minor labels.", text_on: ["ground", "ground-raised"] },
  { name: "ink-faint", type: "colour", meaning: "Orbit lines and other drawn structure that must stay quiet.", line_on: ["ground"] },
  { name: "grid", type: "colour", meaning: "Survey grid lines on the ground. Decorative; never carries information." },
  { name: "rule", type: "colour", meaning: "Hairline dividers inside cards." },
  { name: "accent", type: "colour", meaning: "The one accent: selection and anything authored.", promoted: true, line_on: ["ground", "ground-raised"] },
  { name: "accent-ink", type: "colour", meaning: "Accent when it carries text (stamps, 'set' labels).", text_on: ["ground", "ground-raised"] },
  { name: "on-ink", type: "colour", meaning: "Text on an ink-filled control (a pressed button).", text_on: ["ink"] },
  { name: "zone", type: "colour", meaning: "Wash for a zone band (for example a temperate zone)." },
  { name: "zone-ink", type: "colour", meaning: "Text naming a zone.", text_on: ["ground"] },
  { name: "hatch", type: "colour", meaning: "Hatching ink for uncertainty and unsurveyed regions." },
  { name: "focus", type: "colour", meaning: "Keyboard focus ring.", line_on: ["ground", "ground-raised"] },
  { name: "font-display", type: "font", meaning: "Place names and titles (serif; italic for places)." },
  { name: "font-body", type: "font", meaning: "Interface text." },
  { name: "font-mono", type: "font", meaning: "Values and numbers." },
  { name: "line-hair", type: "length", meaning: "Hairline weight." },
  { name: "line", type: "length", meaning: "Normal ink weight." },
  { name: "line-heavy", type: "length", meaning: "Heavy ink weight (frames, selected orbit)." },
  { name: "card-shadow", type: "shadow", meaning: "Hard offset shadow that lifts a card off the chart.", promoted: true },
  { name: "radius", type: "length", meaning: "Corner radius of frames; 0 is the chart look." },
  { name: "grid-step", type: "number", meaning: "Survey grid spacing in chart pixels.", promoted: true },
];
