// The theme token schema (H8.4, H8.9): every token a view may draw with,
// described for Orac (type, meaning, default from the base theme) with a few
// promoted for people. Views draw only with these names, as CSS custom
// properties `--sse-<name>`.
export interface TokenSpec {
  name: string;
  type: "colour" | "length" | "font" | "number" | "shadow" | "keyword" | "duration" | "easing";
  /** For keywords: the values a theme may set. */
  options?: readonly string[];
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
  { name: "field-out", type: "colour", meaning: "Mag layer: a field line where the field points away from the body (orange; reads apart from field-in for colour-blind eyes).", line_on: ["ground", "ground-raised"] },
  { name: "field-in", type: "colour", meaning: "Mag layer: a field line where the field points into the body (blue).", line_on: ["ground", "ground-raised"] },
  { name: "origin-big-bang", type: "colour", meaning: "Where an element was made (one legend app-wide: periodic table, composition bars, resource cards): the Big Bang. A fill with an ink border, never a line or text." },
  { name: "origin-cosmic-rays", type: "colour", meaning: "Where an element was made (one legend app-wide: periodic table, composition bars, resource cards): cosmic rays breaking heavier atoms. A fill with an ink border, never a line or text." },
  { name: "origin-low-mass-stars", type: "colour", meaning: "Where an element was made (one legend app-wide: periodic table, composition bars, resource cards): dying low-mass stars. A fill with an ink border, never a line or text." },
  { name: "origin-massive-stars", type: "colour", meaning: "Where an element was made (one legend app-wide: periodic table, composition bars, resource cards): exploding massive stars. A fill with an ink border, never a line or text." },
  { name: "origin-white-dwarfs", type: "colour", meaning: "Where an element was made (one legend app-wide: periodic table, composition bars, resource cards): exploding white dwarfs. A fill with an ink border, never a line or text." },
  { name: "origin-neutron-star-mergers", type: "colour", meaning: "Where an element was made (one legend app-wide: periodic table, composition bars, resource cards): merging neutron stars. A fill with an ink border, never a line or text." },
  { name: "origin-human-made", type: "colour", meaning: "Where an element was made (one legend app-wide: periodic table, composition bars, resource cards): made by people, or gone by decay. A fill with an ink border, never a line or text." },
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

  // Frame parts (V2): the named pieces every view is built from. A view says
  // "this is a panel, this is its cap, this is a label"; the theme decides
  // what each looks like, so one view can be a chart card, a lit deck panel
  // or a phosphor readout without knowing which.
  { name: "panel-ground", type: "colour", meaning: "Fill of a panel (a card, a table, a readout).", promoted: true },
  { name: "panel-ink", type: "colour", meaning: "Text and lines inside a panel.", text_on: ["panel-ground"] },
  { name: "panel-muted", type: "colour", meaning: "Secondary text inside a panel: labels, captions, unknowns.", text_on: ["panel-ground"] },
  { name: "frame-line", type: "colour", meaning: "Colour of a panel's outline. Decorative: a panel is also set apart by its fill and shadow, so no contrast rule." },
  { name: "frame-width", type: "length", meaning: "Weight of a panel's outline; 0 for none." },
  { name: "frame-radius", type: "length", meaning: "Corner radius of panels." },
  { name: "frame-shadow", type: "shadow", meaning: "What lifts a panel off the ground: a hard offset, a soft blur or a glow." },
  { name: "cap-ground", type: "colour", meaning: "Fill of a panel's cap, the strip that carries its title." },
  { name: "cap-ink", type: "colour", meaning: "Title text on the cap.", text_on: ["cap-ground"] },
  { name: "cap-rule", type: "colour", meaning: "Line between the cap and the panel body." },
  { name: "cap-end", type: "length", meaning: "Width of the solid block that ends a cap at its left; 0 for none." },
  { name: "cap-end-colour", type: "colour", meaning: "Colour of the cap's end block." },
  { name: "cap-end-radius", type: "length", meaning: "Rounding of the cap's end block (a pill when large)." },
  { name: "title-font", type: "font", meaning: "Font of panel titles and names." },
  { name: "title-style", type: "keyword", options: ["normal", "italic"], meaning: "Slant of titles." },
  { name: "title-case", type: "keyword", options: ["none", "uppercase", "capitalize"], meaning: "Case of titles." },
  { name: "title-size", type: "length", meaning: "Size of a panel title." },
  { name: "label-case", type: "keyword", options: ["none", "uppercase", "lowercase"], meaning: "Case of field labels and captions." },
  { name: "label-tracking", type: "length", meaning: "Letter spacing of field labels." },
  { name: "value-font", type: "font", meaning: "Font of values and numbers." },
  { name: "value-ink", type: "colour", meaning: "Colour of values.", text_on: ["panel-ground"] },
  { name: "unknown-style", type: "keyword", options: ["normal", "italic"], meaning: "Slant of a placeholder for an unknown fact." },
  { name: "row-rule", type: "colour", meaning: "Divider between rows of facts." },
  { name: "row-rule-style", type: "keyword", options: ["solid", "dotted", "dashed", "none"], meaning: "Style of row dividers." },
  { name: "tag-ground", type: "colour", meaning: "Fill of a tag chip." },
  { name: "tag-ink", type: "colour", meaning: "Text of a tag chip.", text_on: ["tag-ground"] },
  { name: "tag-radius", type: "length", meaning: "Rounding of tag chips." },
  { name: "text-glow", type: "shadow", meaning: "Glow around text (phosphor screens); none for print." },
  { name: "chart-ground", type: "colour", meaning: "Ground behind an orbital chart drawing." },

  // Motion (dynamic minimalism, handover/interface-style.md): motion explains
  // a change of state and never delays input. Reduced-motion settings make
  // every duration 0 whatever the theme says.
  { name: "motion-quick", type: "duration", meaning: "Feedback on touch: hover, press, focus." },
  { name: "motion", type: "duration", meaning: "A change of state: a card opening from its body, the ruler zooming, a tab moving." },
  { name: "ease", type: "easing", meaning: "Easing for every motion (eases out, so it answers at once and settles)." },
];
