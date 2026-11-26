// The three desks as data (track TM1, three desks design section 10, accepted by Alex 25 Nov 2026 13:30 UTC).
//
// A desk is three answers, the same on every display (design section 1): where a change goes (`writes`), what time
// can do (`time`) and how much is on screen (the `desks` field on everything that can be on screen). The three shipped
// desks are three `sse.desk/1` records, and the desk switch reads its words from them, so renaming a desk is a change
// of data.
//
// "Is this on screen on this desk?" has one answer, `shows(thing, desk)`. A rail entry (`RailEntry.desks`), a part's
// control (`PartControl.desks`) and the shell's own parts (`SHELL_PARTS` below) carry the field; left out, the thing is
// on all three desks. No screen tests the desk by name: `tests/ui/desk-ledger.test.ts` counts the places that still do,
// and the count only goes down.
//
// TM1 is the contract only. SHELL_PARTS says what each part does TODAY (design section 2), so nothing on screen moves;
// TM2 gives each part its row of section 3.1 as the shell starts reading `shows()` for it, one small change at a time.

import type { PartControl } from "./part-controls.ts";

/** The desks, in the order the switch shows them. */
export const DESKS = ["explore", "build", "run"] as const;
export type Desk = (typeof DESKS)[number];

export const DESK_SCHEMA = "sse.desk/1";

/** Where a change goes: onto a what-if and not kept; into the campaign; or only the acts of play (clock, reveal, send, fuel, crew, cargo). */
export const DESK_WRITES = ["try", "campaign", "play"] as const;
export type DeskWrites = (typeof DESK_WRITES)[number];
/** What time can do: look anywhere; look and set dates; the campaign's now in human time. */
export const DESK_TIMES = ["look", "set", "clock"] as const;
export type DeskTime = (typeof DESK_TIMES)[number];

export interface DeskDoc {
  schema: typeof DESK_SCHEMA;
  id: Desk;
  /** As the desk switch says it. */
  label: string;
  /** The key that switches to it (one letter). */
  key: string;
  /** The switch's tooltip. */
  hint: string;
  /** The desk in one line (design 1.3), for the Reference and Orac. */
  line: string;
  writes: DeskWrites;
  time: DeskTime;
  /** The lens the desk opens with; left out: the device's default lens. */
  lens?: string;
}

/**
 * The three shipped desks. The hints are the switch's words today; `writes` and `time` are the accepted design's
 * answers. Today the shell reads `writes` only as Build arms editing ("campaign"); TM3 and TM4 make the other answers
 * true on screen.
 */
export const SHIPPED_DESKS: readonly DeskDoc[] = [
  { schema: DESK_SCHEMA, id: "explore", label: "Explore", key: "V", hint: "Look around: nothing changes by accident (V)", line: "Try anything, nothing is kept.", writes: "try", time: "look" },
  { schema: DESK_SCHEMA, id: "build", label: "Build", key: "E", hint: "Change things: values edit, bodies drag, undo and redo (E)", line: "What you change is the campaign.", writes: "campaign", time: "set", lens: "building" },
  { schema: DESK_SCHEMA, id: "run", label: "Run", key: "R", hint: "At the table: what players know, their screens and requests (R)", line: "The picture, the place and the clock.", writes: "play", time: "clock", lens: "running" },
];

export function isDesk(v: unknown): v is Desk {
  return typeof v === "string" && (DESKS as readonly string[]).includes(v);
}

/** A shipped desk's record. */
export function deskDoc(id: Desk): DeskDoc {
  return SHIPPED_DESKS.find((d) => d.id === id)!;
}

/** Whether a desk's changes go into the campaign (today: whether the chart is armed for editing). */
export function deskKeeps(id: Desk): boolean {
  return deskDoc(id).writes === "campaign";
}

/** Problems with a desk record; empty when it is fine. */
export function checkDesk(doc: unknown): string[] {
  if (!doc || typeof doc !== "object" || Array.isArray(doc)) return ["a desk must be an object"];
  const d = doc as Partial<DeskDoc>;
  const errors: string[] = [];
  if (d.schema !== DESK_SCHEMA) errors.push(`unknown desk schema ${String(d.schema)}`);
  if (!isDesk(d.id)) errors.push(`unknown desk ${String(d.id)} (desks are ${DESKS.join(", ")})`);
  if (typeof d.label !== "string" || !d.label.trim()) errors.push("a desk needs a label");
  if (typeof d.key !== "string" || !/^[A-Z]$/.test(d.key)) errors.push("a desk's key is one capital letter");
  if (typeof d.hint !== "string" || !d.hint.trim()) errors.push("a desk needs a hint");
  if (typeof d.line !== "string" || !d.line.trim()) errors.push("a desk needs its one line");
  if (!(DESK_WRITES as readonly string[]).includes(d.writes as string)) errors.push(`writes must be one of ${DESK_WRITES.join(", ")}`);
  if (!(DESK_TIMES as readonly string[]).includes(d.time as string)) errors.push(`time must be one of ${DESK_TIMES.join(", ")}`);
  if (d.lens !== undefined && (typeof d.lens !== "string" || !d.lens)) errors.push("a desk's lens is a lens id");
  return errors;
}

/** Anything that can be on screen: it says which desks it shows on; left out, all three. */
export interface OnDesks {
  desks?: readonly Desk[];
}

/** The one answer to "is this on screen on this desk?". */
export function shows(thing: OnDesks | undefined, desk: Desk): boolean {
  return !thing?.desks || thing.desks.includes(desk);
}

/** Problems with a `desks` field (`where` names the thing); empty when it is fine or left out. */
export function checkDesks(desks: unknown, where: string): string[] {
  if (desks === undefined) return [];
  if (!Array.isArray(desks) || !desks.length) return [`${where}: desks must be a list of one or more of ${DESKS.join(", ")} (leave it out for all three)`];
  const errors: string[] = [];
  for (const d of desks) if (!isDesk(d)) errors.push(`${where}: unknown desk ${String(d)}`);
  if (new Set(desks).size !== desks.length) errors.push(`${where}: a desk is listed twice`);
  return errors;
}

/** The desks a thing is on, in words, when it is not on all three ("Build and Run"); empty when it is on all three. */
export function deskWords(thing: OnDesks | undefined): string {
  const on = DESKS.filter((d) => shows(thing, d)).map((d) => deskDoc(d).label);
  if (on.length === DESKS.length) return "";
  return on.length > 1 ? `${on.slice(0, -1).join(", ")} and ${on[on.length - 1]}` : (on[0] ?? "no desk");
}

/** One of the shell's own parts round the picture (design 3.1). */
export interface ShellPart extends OnDesks {
  id: string;
  /** In plain words, for the Reference and Orac. */
  words: string;
  /** The part's own presses, where they differ by desk: the same shape as a player part's controls (AT13). */
  controls?: readonly PartControl[];
}

/**
 * The shell's parts and the desks each shows on TODAY (design section 2): nothing moves in TM1. TM2 changes each row to
 * its section 3.1 desks as the shell reads it. The clock (7.3) joins with TM3. Form changes on one desk (the place line
 * as a stepper, the folded rail) are the shell's, not a field here.
 */
export const SHELL_PARTS: readonly ShellPart[] = [
  { id: "place-line", words: "The place line: where you are, step by step" },
  { id: "subtitle", words: "The line under the name: the place's subtitle and the date" },
  { id: "desk-switch", words: "The desk switch: Explore, Build, Run" },
  { id: "see-as", words: "See as: the lens the chart is seen through" },
  { id: "depth", words: "Depth: how much every card, number and Why? says" },
  { id: "ask", words: "Ask: Orac and Minerva" },
  { id: "more", words: "More: campaign, Workshop, help and the rest" },
  { id: "rail", words: "The instrument rail" },
  { id: "view-controls", words: "View style, Seen from, Scale, HUD and Band" },
  {
    id: "tools",
    words: "The stage's tools",
    controls: [
      { id: "select", words: "Select", kind: "button", default: true },
      { id: "undo", words: "Undo (also on Explore while a what-if is open)", kind: "button", default: true, desks: ["build"] },
      { id: "redo", words: "Redo (also on Explore while a what-if is open)", kind: "button", default: true, desks: ["build"] },
      { id: "copy", words: "Copy", kind: "button", default: true, desks: ["build"] },
      { id: "add", words: "Add here", kind: "button", default: true, desks: ["build"] },
      { id: "cut", words: "Cut", kind: "button", default: true, desks: ["build"] },
      { id: "keys", words: "Keyboard shortcuts", kind: "button", default: true },
    ],
  },
  { id: "card", words: "The card for the thing selected" },
  { id: "eyes", words: "The \"Known to\" eyes and pencil", desks: ["build", "run"] },
  { id: "what-if-stamp", words: "The what-if stamp and dashed frame" },
  { id: "ruler", words: "The time ruler, with What if and Look ahead" },
  { id: "light", words: "Light" },
  { id: "sound", words: "Sound" },
  { id: "status-line", words: "The status line" },
];

/** A shell part by id (throws on an unknown id, so a typo fails its test, not the screen). */
export function shellPart(id: string): ShellPart {
  const p = SHELL_PARTS.find((x) => x.id === id);
  if (!p) throw new Error(`no shell part ${id}`);
  return p;
}

/** Whether one of the shell's parts, or one of its controls, is on screen on this desk. */
export function partShows(id: string, desk: Desk, control?: string): boolean {
  const p = shellPart(id);
  if (!shows(p, desk)) return false;
  if (control === undefined) return true;
  const c = p.controls?.find((x) => x.id === control);
  if (!c) throw new Error(`shell part ${id} has no control ${control}`);
  return shows(c, desk);
}

/** The seven levels of the place line (design 3.2). Local stars and Site are new steps (grand zoom sections 8 and 9). */
export const DESK_LEVELS = ["web", "cluster", "galaxy", "local", "system", "world", "site"] as const;
export type DeskLevel = (typeof DESK_LEVELS)[number];

/**
 * What each desk remembers on this device (design question 9): the view choice (View style, Seen from, Scale, HUD,
 * Band, Motion, Aids) and the ruler's span for each level, and the depth for the desk as a whole.
 */
export const DESK_MEMORY = { view: "level", "ruler-span": "level", depth: "desk" } as const;
export type DeskMemory = keyof typeof DESK_MEMORY;

/**
 * The device key a desk remembers one thing under ("desk.run.system.view", "desk.run.depth"), for `loadPref` and
 * `savePref`. A per-level thing needs its level; a per-desk thing takes none.
 */
export function deskMemoryKey(desk: Desk, what: DeskMemory, level?: DeskLevel): string {
  const per = DESK_MEMORY[what];
  if (per === "level" && level === undefined) throw new Error(`${what} is remembered per level: give the level`);
  if (per === "desk" && level !== undefined) throw new Error(`${what} is remembered per desk, not per level`);
  return level === undefined ? `desk.${desk}.${what}` : `desk.${desk}.${level}.${what}`;
}
