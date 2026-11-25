// Control sets (track AT, AT15; atlas design 11A.4 rule 5): "buttons and interface elements need to follow style"
// (Alex, 24 Nov 20:06). A part declares a control as a kind (button, switch, choice, step, dial, tab, link: AT13); a
// look's control set says how each kind is drawn, lit and pressed in that look. So Print is `[P]RINT` on Hauler's
// command line, a rounded cap in Deck, a glass pill in Corporate and an inked word in the Guide, from one declaration.
// A set without a drawing for a kind takes its parent's, so a variant costs nothing. Touch size, the focus ring and
// reduced motion are fixed minimums the drawing component keeps whatever the set says (KitControl in chart-grammar).
// Data only; `checkControlSet` is pure and returns sentences, as the kit loader does.
import { PART_CONTROL_KINDS, type PartControlKind } from "../part-controls.ts";

/** The outline a control wears. */
export const CONTROL_FRAMES = ["plain", "bracket", "pill", "cap", "ink", "key"] as const;
export type ControlFrame = (typeof CONTROL_FRAMES)[number];
/** How a control shows it is on, pressed or chosen. */
export const CONTROL_LIT = ["fill", "invert", "glow", "underline", "bold"] as const;
export type ControlLit = (typeof CONTROL_LIT)[number];

export interface ControlDrawing {
  frame: ControlFrame;
  /** Words as written, or in capitals (a terminal, a console). */
  case?: "as-written" | "upper";
  /** Show the key that presses it, inside the word: `[P]RINT`. Needs a frame of `bracket` or `key`. */
  hotkey?: boolean;
  lit?: ControlLit;
}

export interface ControlSet {
  id: string;
  name: string;
  /** The set whose drawings fill any kind this one leaves out; the plain set at the root. */
  extends?: string;
  kinds: Partial<Record<PartControlKind, ControlDrawing>>;
}

/** The root of every set: the chart room's own plain buttons. */
export const PLAIN_SET: ControlSet = {
  id: "plain",
  name: "Plain",
  kinds: {
    button: { frame: "plain", lit: "fill" },
    switch: { frame: "plain", lit: "fill" },
    choice: { frame: "plain", lit: "fill" },
    step: { frame: "plain", lit: "fill" },
    dial: { frame: "plain", lit: "fill" },
    tab: { frame: "plain", lit: "underline" },
    link: { frame: "ink", lit: "underline" },
  },
};

/** The four starters' sets (11A.3), each a look's own. */
export const STARTER_CONTROL_SETS: readonly ControlSet[] = [
  PLAIN_SET,
  {
    id: "hauler",
    name: "Hauler command line",
    kinds: {
      button: { frame: "bracket", case: "upper", hotkey: true, lit: "invert" },
      switch: { frame: "bracket", case: "upper", hotkey: true, lit: "invert" },
      choice: { frame: "bracket", case: "upper", lit: "invert" },
      step: { frame: "bracket", case: "upper", hotkey: true, lit: "invert" },
      tab: { frame: "bracket", case: "upper", lit: "invert" },
      link: { frame: "plain", case: "upper", lit: "invert" },
    },
  },
  {
    id: "corporate",
    name: "Corporate glass",
    kinds: {
      button: { frame: "pill", lit: "glow" },
      switch: { frame: "pill", lit: "glow" },
      choice: { frame: "pill", lit: "glow" },
      step: { frame: "pill", lit: "glow" },
      tab: { frame: "pill", lit: "glow" },
    },
  },
  {
    id: "deck",
    name: "Deck panels",
    kinds: {
      button: { frame: "cap", case: "upper", lit: "fill" },
      switch: { frame: "cap", case: "upper", lit: "fill" },
      choice: { frame: "cap", case: "upper", lit: "fill" },
      step: { frame: "cap", case: "upper", lit: "fill" },
      tab: { frame: "cap", case: "upper", lit: "fill" },
      dial: { frame: "cap", case: "upper", lit: "fill" },
    },
  },
  {
    id: "guide",
    name: "Guide keys",
    kinds: {
      button: { frame: "key", lit: "bold" },
      switch: { frame: "key", lit: "bold" },
      choice: { frame: "key", lit: "bold" },
      step: { frame: "key", lit: "bold" },
      tab: { frame: "ink", lit: "underline" },
      link: { frame: "ink", lit: "underline" },
    },
  },
];

/** The drawing for one kind in a set, following `extends` to the plain set. */
export function drawingOf(setId: string | undefined, kind: PartControlKind, sets: readonly ControlSet[] = STARTER_CONTROL_SETS): ControlDrawing {
  const seen: string[] = [];
  let id: string | undefined = setId ?? "plain";
  while (id && !seen.includes(id)) {
    seen.push(id);
    const s: ControlSet | undefined = sets.find((x) => x.id === id) ?? (id === "plain" ? PLAIN_SET : undefined);
    if (!s) break;
    const d = s.kinds[kind];
    if (d) return d;
    id = s.extends ?? (s.id === "plain" ? undefined : "plain");
  }
  return PLAIN_SET.kinds[kind] ?? { frame: "plain", lit: "fill" };
}

/** A control's words as drawn: the case, and the hot key marked inside the word (`[P]RINT`), with the key itself. */
export function controlWords(words: string, d: ControlDrawing, key?: string): { text: string; key?: string } {
  const text = d.case === "upper" ? words.toUpperCase() : words;
  if (!d.hotkey || (d.frame !== "bracket" && d.frame !== "key")) return { text };
  const k = (key ?? words.replace(/[^A-Za-z0-9]/g, "")[0] ?? "").toUpperCase();
  if (!k) return { text };
  const i = text.toUpperCase().indexOf(k);
  return i < 0 ? { text: `[${k}] ${text}`, key: k } : { text: `${text.slice(0, i)}[${text[i]}]${text.slice(i + 1)}`, key: k };
}

/** Why a control set will not load, as sentences; empty when it loads. */
export function checkControlSet(doc: unknown, sets: readonly ControlSet[] = STARTER_CONTROL_SETS): string[] {
  const s = doc as ControlSet;
  if (!s || typeof s !== "object" || Array.isArray(s)) return ["A control set is a JSON object."];
  const out: string[] = [];
  if (typeof s.id !== "string" || !s.id) out.push("A control set needs an id.");
  if (typeof s.name !== "string" || !s.name) out.push("A control set needs a name.");
  if (s.extends !== undefined && !sets.some((x) => x.id === s.extends)) out.push(`The control set ${s.id} extends "${s.extends}", which is not loaded (there are ${sets.map((x) => x.id).join(", ")}).`);
  if (!s.kinds || typeof s.kinds !== "object") return [...out, "A control set needs kinds: how each kind of control is drawn."];
  for (const [kind, d] of Object.entries(s.kinds)) {
    if (!(PART_CONTROL_KINDS as readonly string[]).includes(kind)) {
      out.push(`The control set ${s.id} draws "${kind}", which is not a kind of control (the kinds are ${PART_CONTROL_KINDS.join(", ")}).`);
      continue;
    }
    if (!d || !(CONTROL_FRAMES as readonly string[]).includes(d.frame)) out.push(`In the control set ${s.id}, ${kind} needs a frame: one of ${CONTROL_FRAMES.join(", ")}.`);
    if (d?.lit !== undefined && !(CONTROL_LIT as readonly string[]).includes(d.lit)) out.push(`In the control set ${s.id}, ${kind} is lit as "${d.lit}"; use one of ${CONTROL_LIT.join(", ")}.`);
    if (d?.case !== undefined && d.case !== "as-written" && d.case !== "upper") out.push(`In the control set ${s.id}, ${kind}'s case is "as-written" or "upper".`);
    if (d?.hotkey && d.frame !== "bracket" && d.frame !== "key") out.push(`In the control set ${s.id}, ${kind} shows its key, which needs a frame of bracket or key.`);
  }
  return out;
}
